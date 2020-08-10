use std::{
    time::{
        Instant,
    },
};

use crate::{
    event::*,
    AudioContext,
    CALLBACK_BUFFER_LEN,
    Sample,
};

use super::{
    Module,
};

//

pub struct Chain {
    dummy: Box<[Sample; CALLBACK_BUFFER_LEN]>,
    dummy2: Box<[Sample; CALLBACK_BUFFER_LEN]>,
    modules: Vec<Box<dyn Module>>,
}

impl Chain {
    pub fn new(modules: Vec<Box<dyn Module>>) -> Self {
        Self {
            dummy: Box::new([0.; CALLBACK_BUFFER_LEN]),
            dummy2: Box::new([0.; CALLBACK_BUFFER_LEN]),
            modules,
        }
    }
}

impl Module for Chain {
    fn frame(&mut self, ctx: &AudioContext) {
        for m in self.modules.iter_mut() {
            m.frame(ctx);
        }
    }

    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample],
               out_buf: &mut [Sample]) {
        let frame_size = in_buf.len();

        let main_buf = &mut &mut *self.dummy;
        let back_buf = &mut &mut *self.dummy2;

        let mut first = true;
        for m in self.modules.iter_mut() {
            if first {
                m.compute(ctx, in_buf, &mut (*main_buf)[0..frame_size]);
                first = false;
            } else {
                m.compute(ctx, &(*main_buf)[0..frame_size], &mut (*back_buf)[0..frame_size]);
                std::mem::swap(main_buf, back_buf);
            }
        }

        for i in 0..out_buf.len() {
            out_buf[i] = (*main_buf)[i];
        }
    }
}
