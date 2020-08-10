pub mod channel;
pub mod modules;
pub mod event;

//

use std::{
    time::{
        Instant,
    },
};

pub type Sample = f32;

pub struct AudioContext {
    pub sample_rate: u32,
    pub now: Instant,
}

impl AudioContext {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            now: Instant::now(),
        }
    }
}

//

pub const CALLBACK_BUFFER_LEN: usize = 2048;

pub struct CallbackBuffer {
    size: usize,
    in_buf: Box<[Sample; CALLBACK_BUFFER_LEN]>,
    out_buf: Box<[Sample; CALLBACK_BUFFER_LEN]>,
}

impl CallbackBuffer {
    pub fn new() -> Self {
        Self {
            size: 0,
            in_buf: Box::new([0.; CALLBACK_BUFFER_LEN]),
            out_buf: Box::new([0.; CALLBACK_BUFFER_LEN]),
        }
    }

    pub fn fill_in_buf_f32(&mut self, buf: &[f32]) {
        if buf.len() > CALLBACK_BUFFER_LEN {
            panic!();
        }

        self.size = buf.len();
        for i in 0..self.size {
            self.in_buf[i] = buf[i];
        }
    }

    pub fn take_out_buf_f32(&self, buf: &mut [f32]) {
        if self.size > buf.len() {
            panic!();
        }

        for i in 0..self.size {
            buf[i] = self.out_buf[i];
        }
    }

    pub fn buffers(&mut self) -> (&[Sample], &mut [Sample]) {
        (&self.in_buf[0..self.size], &mut self.out_buf[0..self.size])
    }
}
