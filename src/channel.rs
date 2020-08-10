use std::{
    sync::Arc,
};

use crossbeam_queue::ArrayQueue;

//

pub struct Sender<T> {
    queue: Arc<ArrayQueue<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, val: T) {
        self.queue.push(val);
    }
}

pub struct Receiver<T> {
    queue: Arc<ArrayQueue<T>>,
}

impl<T> Receiver<T> {
    pub fn try_recv(&self) -> Option<T> {
        self.queue.pop().ok()
    }
}

pub fn channel<T>(size: usize) -> (Sender<T>, Receiver<T>){
    let queue = Arc::new(ArrayQueue::new(size));
    let tx = Sender {
        queue: Arc::clone(&queue),
    };
    let rx = Receiver {
        queue,
    };
    (tx, rx)
}
