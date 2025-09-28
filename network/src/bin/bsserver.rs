use libc::{accept, bind, close, listen, recv, send, sockaddr, sockaddr_in, socket, socklen_t};
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::os::unix::io::RawFd;
use std::str;
use std::thread::sleep;
use std::time::Duration;
use tracing::{error, info};

const PORT: u16 = 8080;
const BUFFER_SIZE: usize = 1024;

fn run() {
    // 创建 TCP Socket
    let sockfd = unsafe { socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
    if sockfd == -1 {
        error!("Failed to create socket");
        return;
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

    // 监听
    if unsafe { listen(sockfd, 5) } == -1 {
        error!("Failed to listen");
        unsafe { close(sockfd) };
        return;
    }

    info!("HTTP Server running on http://0.0.0.0:{}", PORT);

    // 接受连接
    loop {
        let mut client_addr: sockaddr_in = unsafe { mem::zeroed() };
        let mut addr_len = mem::size_of_val(&client_addr) as socklen_t;

        let client_fd = unsafe {
            accept(
                sockfd,
                &mut client_addr as *mut sockaddr_in as *mut sockaddr,
                &mut addr_len,
            )
        };

        if client_fd == -1 {
            error!("Failed to accept connection");
            continue;
        }

        // 处理 HTTP 请求
        handle_http_request(client_fd);
    }
}

fn handle_http_request(client_fd: RawFd) {
    // emulate heavy cpu task
    sleep(Duration::from_millis(100));

    let mut buffer = [0u8; BUFFER_SIZE];
    let bytes_read = unsafe {
        recv(
            client_fd,
            buffer.as_mut_ptr() as *mut libc::c_void,
            BUFFER_SIZE,
            0,
        )
    };

    if bytes_read <= 0 {
        error!("Failed to read request");
        unsafe { close(client_fd) };
        return;
    }

    // 解析 HTTP 请求
    let request = match str::from_utf8(&buffer[..bytes_read as usize]) {
        Ok(req) => req,
        Err(_) => {
            error!("Invalid UTF-8 in request");
            unsafe { close(client_fd) };
            return;
        }
    };

    info!("Received request:\n{}", request);

    // 检查是否是 GET 请求
    if !request.starts_with("GET") {
        send_response(client_fd, 400, "Bad Request: Only GET is supported");
        return;
    }

    // 提取路径（如 GET /hello HTTP/1.1）
    let path = request.split_whitespace().nth(1).unwrap_or("/");

    // 返回 HTTP 响应
    let response_body = format!(
        "<html><body><h1>Hello from Rust!</h1><p>Path: {}</p></body></html>",
        path
    );

    send_response(client_fd, 200, &response_body);
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

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();
    run();
}
