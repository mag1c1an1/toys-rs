use std::{
    cell::RefCell,
    sync::{
        Arc,
        mpsc::{self},
    },
};

use crate::task::Task;

pub struct MiniTokio {
    scheduled: mpsc::Receiver<Arc<Task>>,
    sender: mpsc::Sender<Arc<Task>>,
}

impl MiniTokio {
    pub fn new() -> Self {
        let (sender, scheduled) = mpsc::channel();
        MiniTokio { scheduled, sender }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    pub fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            println!("get task");
            task.poll();
        }
    }
}

// Used to track the current mini-tokio instance so that the `spawn` function is
// able to schedule spawned tasks.
thread_local! {
    static CURRENT: RefCell<Option<mpsc::Sender<Arc<Task>>>> =
        RefCell::new(None);
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    CURRENT.with(|cell| {
        let borrow = cell.borrow();
        let sender = borrow.as_ref().unwrap();
        Task::spawn(future, sender)
    })
}
