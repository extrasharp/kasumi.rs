use crate::{
    event::*,
    AudioContext,
    Sample,
    CALLBACK_BUFFER_LEN,
};

use super::Module2;

//

#[derive(Debug)]
pub struct Sine2 {
    frame_ct: u32,
    freq: f32,
}

impl Sine2 {
    pub fn new() -> Self {
        Self {
            frame_ct: 0,
            freq: 440.
        }
    }

    pub fn set_frequency(&mut self, to: f32) {
        self.freq = to;
    }
}

impl Module2 for Sine2 {
    fn compute(&mut self,
               ctx: &AudioContext,
               _in_buf: &[Sample],
               out_buf: &mut [Sample]) {
        let sr = ctx.sample_rate as f32;
        let p_base = std::f32::consts::PI * 2. * self.freq / sr;
        for i in 0..(out_buf.len()/2) {
            let period = self.frame_ct as f32 * p_base;
            let s = f32::sin(period);
            out_buf[i * 2] = s;
            out_buf[i * 2 + 1] = s;
            self.frame_ct += 1;
        }
    }
}

pub struct Mixer2 {
    dummy: Box<[Sample; CALLBACK_BUFFER_LEN]>,
    modules: Vec<Box<dyn Module2>>,
}

impl Mixer2 {
    pub fn new(modules: Vec<Box<dyn Module2>>) -> Self {
        Self {
            dummy: Box::new([0.; CALLBACK_BUFFER_LEN]),
            modules,
        }
    }
}

impl Module2 for Mixer2 {
    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample],
               out_buf: &mut [Sample]) {
        for m in self.modules.iter_mut() {
            for i in 0..out_buf.len() {
                out_buf[i] = 0.;
            }
            m.compute(ctx, &in_buf, &mut self.dummy[0..in_buf.len()]);
            for i in 0..out_buf.len() {
                out_buf[i] += self.dummy[i];
            }
        }
    }
}
