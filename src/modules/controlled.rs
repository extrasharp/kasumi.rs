use std::time::Instant;

use crate::{
    event::*,
};

use super::prelude::*;

//

pub struct Controlled<T: Module> {
    module: T,
    rx: EventReceiver<Box<dyn FnOnce(&mut T, &CallbackContext) + Send>>,
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
    fn is_finished(&self) -> bool {
        self.module.is_finished()
    }

    fn frame(&mut self, ctx: &CallbackContext) {
        while let Some(func) = self.rx.try_recv(ctx.now) {
            func(&mut self.module, ctx);
        }
    }

    fn compute<'a>(&mut self,
        ctx: &CallbackContext,
        in_bufs: &[InputBuffer<'a>],
        out_buf: &mut [Sample]) {
        self.module.compute(ctx, in_bufs, out_buf);
    }
}

//

pub struct Controller<T: Module> {
    tx: EventSender<Box<dyn FnOnce(&mut T, &CallbackContext) + Send>>,
}

impl<T: Module> Controller<T> {
    #[inline]
    pub fn send<F>(&self, now: Instant, func: F)
    where
        F: FnOnce(&mut T, &CallbackContext) + Send + 'static
    {
        self.tx.send(now, Box::new(func));
    }
}
