use std::{
    pin::Pin,
    sync::{Arc, Mutex, mpsc},
    task::{Context, Poll},
};

use futures::task::{self, ArcWake};

/// A structure holding a future and the result of
/// the latest call to its `poll` method.

struct TaskFuture {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
    poll: Poll<()>,
}

impl TaskFuture {
    fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
            poll: Poll::Pending,
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) {
        if self.poll.is_pending() {
            println!("task future prev is pending so poll again");
            // 再来一次
            // poll inter
            self.poll = self.future.as_mut().poll(cx);
        }
    }
}

pub struct Task {
    // The `Mutex` is to make `Task` implement `Sync`. Only
    // one thread accesses `task_future` at any given time.
    // The `Mutex` is not required for correctness. Real Tokio
    // does not use a mutex here, but real Tokio has
    // more lines of code than can fit in a single tutorial
    // page.
    task_future: Mutex<TaskFuture>,
    executor: mpsc::Sender<Arc<Task>>, // send end
}

impl Task {
    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    pub fn spawn<F>(future: F, sender: &mpsc::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            executor: sender.clone(),
            task_future: Mutex::new(TaskFuture::new(future)),
        });

        let _ = sender.send(task);
    }

    pub fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone()).unwrap();
    }

    pub fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut cx = task::Context::from_waker(&waker);
        let mut future = self.task_future.try_lock().unwrap();
        // poll the inner user future
        println!("task poll task future");
        future.poll(&mut cx);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // 我又可以跑啦
        arc_self.schedule();
    }
}
