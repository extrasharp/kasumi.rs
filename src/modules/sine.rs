use super::prelude::*;

//

pub struct Sine {
    frame_ct: u32,
    freq: f32,
}

impl Sine {
    pub fn new() -> Self {
        Self {
            frame_ct: 0,
            freq: 440.
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.freq = freq;
    }
}

impl Module for Sine {
    fn compute<'a>(&mut self, ctx: &CallbackContext, _: &[InputBuffer<'a>], out_buf: &mut [Sample]) {
        let sr = ctx.sample_rate as f32;
        let p_base = std::f32::consts::PI * 2. * self.freq / sr;

        for ch in out_buf.chunks_mut(2) {
            let period = self.frame_ct as f32 * p_base;
            let s = f32::sin(period);
            ch[0] = s;
            ch[1] = s;
            self.frame_ct += 1;
        }
    }
}
