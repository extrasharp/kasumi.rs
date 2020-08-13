use super::prelude::*;

//

pub struct Mixer {
}

impl Mixer {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Module for Mixer {
    fn compute<'a>(&mut self,
                   ctx: &CallbackContext,
                   in_bufs: &[InputBuffer<'a>],
                   out_buf: &mut [Sample]) {
        for i in 0..out_buf.len() {
            out_buf[i] = 0.;
        }

        for ib in in_bufs {
            for i in 0..out_buf.len() {
                out_buf[i] += ib.buf[i];
            }
        }
    }
}
