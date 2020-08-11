use std::time::Instant;

use crate::{
    event::*,
    AudioContext,
    Sample,
};

use super::Module;

//

pub struct Sine {
    rx: EventReceiver<SineEvent>,
    frame_ct: u32,
    freq: f32,
}

impl Sine {
    pub fn new() -> (Self, SineController) {
        let (tx, rx) = event_channel(50);
        let ret = Self {
            rx,
            frame_ct: 0,
            freq: 440.
        };
        let ctl = SineController {
            tx,
        };
        (ret, ctl)
    }
}

impl Module for Sine {
    fn frame(&mut self, ctx: &AudioContext) {
        while let Some(event) = self.rx.try_recv(ctx.now) {
            match event {
                SineEvent::Frequency(freq) => self.freq = freq,
            }
        }
    }

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

//

enum SineEvent {
    Frequency(f32),
}

pub struct SineController {
    tx: EventSender<SineEvent>
}

impl SineController {
    pub fn set_frequency(&self, time: Instant, to: f32) {
        self.tx.send(time, SineEvent::Frequency(to));
    }
}
