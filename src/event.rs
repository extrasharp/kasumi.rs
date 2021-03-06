use std::{
    cell::RefCell,
    time::{
        Instant,
    },
};

use crate::{
    channel::*,
};

pub struct Event<T> {
    timestamp: Instant,
    data: T,
}

impl<T> Event<T> {
    pub fn new(timestamp: Instant, data: T) -> Self {
        Self {
            timestamp,
            data,
        }
    }
}

pub struct EventReceiver<T> {
    rx: Receiver<Event<T>>,
    last_event: RefCell<Option<Event<T>>>,
}

impl<T> EventReceiver<T> {
    pub fn new(rx: Receiver<Event<T>>) -> Self {
        Self {
            rx,
            last_event: RefCell::new(None),
        }
    }

    pub fn try_recv(&self, now: Instant) -> Option<T> {
        if self.last_event.borrow().is_some() {
            let timestamp = self.last_event.borrow().as_ref().unwrap().timestamp;
            if timestamp <= now {
                let ev = self.last_event.borrow_mut().take().unwrap();
                Some(ev.data)
            } else {
                None
            }
        } else {
            let get = self.rx.try_recv()?;
            if get.timestamp <= now {
                Some(get.data)
            } else {
                *self.last_event.borrow_mut() = Some(get);
                None
            }
        }
    }
}

pub struct EventSender<T> {
    tx: Sender<Event<T>>,
}

impl<T> EventSender<T> {
    pub fn new(tx: Sender<Event<T>>) -> Self {
        Self {
            tx,
        }
    }

    pub fn send(&self, timestamp: Instant, data: T) {
        // TODO if this timestamp is earlier than the last timestamp
        //  just panic
        self.tx.send(Event::new(timestamp, data));
    }
}

pub fn event_channel<T>(size: usize) -> (EventSender<T>, EventReceiver<T>) {
    let (tx, rx) = channel(size);
    (EventSender::new(tx), EventReceiver::new(rx))
}
