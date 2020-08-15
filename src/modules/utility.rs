use super::prelude::*;

//

pub struct Utility {
    volume: f32,
    pan: f32,
}

impl Utility {
    pub fn new() -> Self {
        Self {
            volume: 1.,
            pan: 0.,
        }
    }

    pub fn set_volume(&mut self, to: f32) {
        self.volume = to;
    }

    pub fn set_pan(&mut self, to: f32) {
        self.pan = to;
    }
}

impl Module for Utility {
    fn compute<'a>(&mut self,
                   ctx: &CallbackContext,
                   in_bufs: &[InputBuffer<'a>],
                   out_buf: &mut [Sample]) {
        let i_0 = in_bufs.iter().find(| ib | ib.id == 0);
        if let Some(ib) = i_0 {
            let in_buf = ib.buf;

            let frame_size = out_buf.len();
            let pan = self.pan / 2. + 0.5;

            for i in 0..(frame_size / 2) {
                out_buf[i * 2] = in_buf[i * 2] * self.volume * (1. - pan);
                out_buf[i * 2 + 1] = in_buf[i * 2 + 1] * self.volume * pan;
            }
        } else {
            for i in 0..out_buf.len() {
                out_buf[i] = 0.;
            }
        }
    }
}
