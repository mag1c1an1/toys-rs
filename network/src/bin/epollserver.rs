use libc::{
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLLET, EPOLLIN, accept, bind, c_void, close, epoll_create1,
    epoll_ctl, epoll_event, epoll_wait, listen, recv, send, sockaddr, sockaddr_in, socket,
    socklen_t,
};
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::unix::io::RawFd;
use std::ptr;
use std::str;
use std::time::Duration;
use tracing::{error, info};

const PORT: u16 = 8081;
const MAX_EVENTS: usize = 1024;
const BUFFER_SIZE: usize = 8192; // 大缓冲区减少 read 调用次数

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();
    // 创建 TCP Socket
    let sockfd = unsafe { socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_NONBLOCK, 0) };
    if sockfd == -1 {
        error!("Failed to create socket");
        return;
    }

    // 设置 SO_REUSEADDR 避免 TIME_WAIT 状态阻塞
    let reuse = 1;
    unsafe {
        libc::setsockopt(
            sockfd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &reuse as *const _ as *const c_void,
            mem::size_of_val(&reuse) as socklen_t,
        );
    }

    // 绑定地址
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), PORT);
    let sockaddr: sockaddr_in = sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: addr.port().to_be(),
        sin_addr: libc::in_addr {
            s_addr: u32::from(*addr.ip()).to_be(),
        },
        sin_zero: [0; 8],
    };

    if unsafe {
        bind(
            sockfd,
            &sockaddr as *const sockaddr_in as *const sockaddr,
            mem::size_of_val(&sockaddr) as socklen_t,
        )
    } == -1
    {
        error!("Failed to bind socket");
        unsafe { close(sockfd) };
        return;
    }

    // 监听连接（设置较大的 backlog）
    if unsafe { listen(sockfd, 1024) } == -1 {
        error!("Failed to listen");
        unsafe { close(sockfd) };
        return;
    }

    info!("Server running on http://0.0.0.1:{}", PORT);

    // 创建 epoll 实例
    let epoll_fd = unsafe { epoll_create1(0) };
    if epoll_fd == -1 {
        error!("Failed to create epoll");
        unsafe { close(sockfd) };
        return;
    }

    // 注册监听 socket 到 epoll（边缘触发模式）
    let mut event = epoll_event {
        events: (EPOLLIN | EPOLLET) as u32,
        u64: sockfd as u64, // 存储 socket fd
    };

    if unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, sockfd, &mut event) } == -1 {
        error!("Failed to add socket to epoll");
        unsafe {
            close(epoll_fd);
            close(sockfd)
        };
        return;
    }

    // 事件循环
    let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];
    loop {
        let num_events = unsafe {
            epoll_wait(
                epoll_fd,
                events.as_mut_ptr(),
                MAX_EVENTS as i32,
                -1, // 无限等待
            )
        };

        if num_events == -1 {
            error!("epoll_wait error");
            continue;
        }

        for i in 0..num_events as usize {
            let fd = events[i].u64 as RawFd;
            if fd == sockfd {
                // 处理新连接
                handle_new_connection(epoll_fd, sockfd);
            } else {
                // 处理客户端请求
                handle_client_request(epoll_fd, fd);
            }
        }
    }
}

fn handle_new_connection(epoll_fd: RawFd, sockfd: RawFd) {
    // 接受所有 pending 的连接（边缘触发必须循环 accept）
    loop {
        let client_fd = unsafe { accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

        if client_fd == -1 {
            if unsafe { *libc::__errno_location() } == libc::EAGAIN {
                break; // 无更多连接
            } else {
                error!("Failed to accept connection");
                break;
            }
        }

        // 设置非阻塞模式
        unsafe { libc::fcntl(client_fd, libc::F_SETFL, libc::O_NONBLOCK) };

        // 注册客户端 socket 到 epoll
        let mut event = epoll_event {
            events: (EPOLLIN | EPOLLET) as u32,
            u64: client_fd as u64,
        };

        if unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, client_fd, &mut event) } == -1 {
            error!("Failed to add client to epoll");
            unsafe { close(client_fd) };
        }
    }
}

fn handle_client_request(epoll_fd: RawFd, client_fd: RawFd) {
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut total_bytes_read = 0;

    // 边缘触发必须读取所有数据
    loop {
        let bytes_read = unsafe {
            recv(
                client_fd,
                buffer.as_mut_ptr() as *mut c_void,
                BUFFER_SIZE,
                0,
            )
        };

        if bytes_read <= 0 {
            if bytes_read == -1 && unsafe { *libc::__errno_location() } == libc::EAGAIN {
                break; // 数据读取完毕
            } else {
                // 客户端断开或错误
                unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_DEL, client_fd, ptr::null_mut()) };
                unsafe { close(client_fd) };
                return;
            }
        }

        total_bytes_read += bytes_read as usize;
    }

    // 解析 HTTP 请求（简化版）
    let request = match str::from_utf8(&buffer[..total_bytes_read]) {
        Ok(req) => req,
        Err(_) => {
            send_response(client_fd, 400, "Bad Request: Invalid UTF-8");
            return;
        }
    };

    info!("Received request:\n{}", request);

    if !request.starts_with("GET") {
        send_response(client_fd, 405, "Method Not Allowed");
        return;
    }

    // 返回 HTTP 响应
    send_response(
        client_fd,
        200,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Hello, epoll!</h1></body></html>",
    );
}

fn send_response(client_fd: RawFd, status_code: u16, body: &str) {
    let status_line = match status_code {
        200 => "HTTP/1.1 200 OK",
        400 => "HTTP/1.1 400 Bad Request",
        404 => "HTTP/1.1 404 Not Found",
        _ => "HTTP/1.1 500 Internal Server Error",
    };

    let response = format!(
        "{}\r\n\
         Content-Type: text/html\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status_line,
        body.len(),
        body
    );

    unsafe {
        send(
            client_fd,
            response.as_ptr() as *const libc::c_void,
            response.len(),
            0,
        );
        close(client_fd);
    }
}
