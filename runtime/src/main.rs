use std::{
    net::TcpStream, pin::Pin, task::Poll, time::{Duration, Instant}
};

use runtime::MiniTokio;

pub mod runtime;
pub mod task;

//
// use std::future::Future;
// use std::pin::Pin;
// use std::task::{Context, Poll};
// use std::time::{Duration, Instant};

// enum MainFuture {
//     // Initialized, never polled
//     State0,
//     // Waiting on `Delay`, i.e. the `future.await` line.
//     State1(Delay),
//     // The future has completed.
//     Terminated,
// }

// impl Future for MainFuture {
//     type Output = ();

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
//         -> Poll<()>
//     {
//         use MainFuture::*;

//         状态机的移动
//         loop {
//             match *self {
//                 State0 => {
//                     let when = Instant::now() +
//                         Duration::from_millis(10);
//                     let future = Delay { when };
//                     *self = State1(future);
//                 }
//                 State1(ref mut my_future) => {
//                     match Pin::new(my_future).poll(cx) {
//                         Poll::Ready(out) => {
//                             assert_eq!(out, "done");
//                             *self = Terminated;
//                             return Poll::Ready(());
//                         }
//                         Poll::Pending => {
//                             return Poll::Pending;
//                         }
//                     }
//                 }
//                 Terminated => {
//                     panic!("future polled after completion")
//                 }
//             }
//         }
//     }
// }

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        println!("Delay polled");
        if Instant::now() >= self.when {
            println!("ready!");
            Poll::Ready("done")
        } else {
            println!("Delay pending and registering waker");
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_secs(3);
        let future = Delay { when };
        let out = future.await;
        assert_eq!(out, "done")
    });
    mini_tokio.run();
}
