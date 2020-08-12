use crate::{
    audio_graph::GraphContext,
    Sample,
};

use super::Module;

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
    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
        let sr = ctx.audio_context.sample_rate as f32;
        let p_base = std::f32::consts::PI * 2. * self.freq / sr;
        for i in 0..(out_buf.len() / 2) {
            let period = self.frame_ct as f32 * p_base;
            let s = f32::sin(period);
            out_buf[i * 2] = s;
            out_buf[i * 2 + 1] = s;
            self.frame_ct += 1;
        }
    }
}
