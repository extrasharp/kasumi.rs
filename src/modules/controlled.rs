use std::time::Instant;

use crate::{
    event::*,
    AudioContext,
    Sample,
};

use super::Module;

pub struct Controlled<T: Module> {
    module: T,
    rx: EventReceiver<Box<dyn FnOnce(&mut T, &AudioContext) + Send>>,
}

impl<T: Module> Controlled<T> {
    pub fn new(module: T) -> (Self, Controller<T>) {
        let (tx, rx) = event_channel(50);
        let ret = Self {
            module,
            rx,
        };
        let ctl = Controller {
            tx,
        };
        (ret, ctl)
    }
}

impl<T: Module> Module for Controlled<T> {
    fn frame(&mut self, ctx: &AudioContext) {
        while let Some(func) = self.rx.try_recv(ctx.now) {
            func(&mut self.module, ctx);
        }
    }

    fn compute(&mut self, ctx: &AudioContext, out_buf: &mut [Sample]) {
        self.module.compute(ctx, out_buf);
    }
}

//

pub struct Controller<T: Module> {
    tx: EventSender<Box<dyn FnOnce(&mut T, &AudioContext) + Send>>,
}

impl<T: Module> Controller<T> {
    #[inline]
    pub fn send<F>(&self, now: Instant, func: F)
    where
        F: FnOnce(&mut T, &AudioContext) + Send + 'static
    {
        self.tx.send(now, Box::new(func));
    }
}
