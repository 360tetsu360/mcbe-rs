use std::{
    pin::Pin,
    task::Context,
    thread::{self, JoinHandle},
};

use futures::{executor::block_on, Future, FutureExt};

pub struct LoopTask {
    pub task: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl LoopTask {
    pub fn run_in_new_thread(self) -> JoinHandle<()> {
        thread::spawn(|| block_on(self.boxed()))
    }
}

impl Future for LoopTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        self.task.poll_unpin(cx)
    }
}
