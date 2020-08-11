use std::{
    time::{
        Instant,
    },
};

use crate::{
    event::*,
    AudioContext,
    Sample,
};

use super::Module;

//

#[derive(Debug)]
pub enum UtilityEvent {
    // 0 to 1
    Volume(f32),
    // -1 to 1
    Pan(f32),
}

pub struct Utility {
    rx: EventReceiver<UtilityEvent>,
    volume: f32,
    pan: f32,
}

impl Utility {
    pub fn new() -> (Self, EventSender<UtilityEvent>) {
        let (tx, rx) = event_channel(50);
        let ret = Self {
            rx,
            volume: 1.,
            pan: 0.,
        };
        (ret, tx)
    }
}

impl Module for Utility {
    fn frame(&mut self, ctx: &AudioContext) {
        while let Some(event) = self.rx.try_recv(ctx.now) {
            match event {
                UtilityEvent::Volume(val) => self.volume = val,
                UtilityEvent::Pan(val) => self.pan = val,
            }
        }
    }

    fn compute(&mut self,
               _ctx: &AudioContext,
               in_buf: &[Sample],
               out_buf: &mut [Sample]) {
        let frame_size = in_buf.len();
        let pan = self.pan / 2. + 0.5;
        for i in 0..(frame_size/2) {
            out_buf[i * 2] = in_buf[i * 2] * self.volume * (1. - pan);
            out_buf[i * 2 + 1] = in_buf[i * 2 + 1] * self.volume * pan;
        }
    }
}
