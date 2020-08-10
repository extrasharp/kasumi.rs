use std::{
    time::{
        Instant,
    },
};

use crate::{
    event::*,
    AudioContext,
    Sample,
    CALLBACK_BUFFER_LEN,
};

use super::{
    Module,
};

//

enum MixerEvent {
    AddModule(Box<dyn Module>),
    RemoveModule(usize),
}

pub struct Mixer {
    rx: EventReceiver<MixerEvent>,

    dummy: Box<[Sample; CALLBACK_BUFFER_LEN]>,
    modules: Vec<Box<dyn Module>>,
}

impl Mixer {
    pub fn new(modules: Vec<Box<dyn Module>>) -> (Self, MixerController) {
        let (tx, rx) = event_channel(50);
        let ret = Self {
            rx,
            dummy: Box::new([0.; CALLBACK_BUFFER_LEN]),
            modules,
        };
        let ctl = MixerController {
            now: Instant::now(),
            tx,
        };
        (ret, ctl)
    }
}

impl Module for Mixer {
    fn frame(&mut self, ctx: &AudioContext) {
        if let Some(event) = self.rx.try_recv(ctx.now) {
            match event {
                MixerEvent::AddModule(module) => self.modules.push(module),
                MixerEvent::RemoveModule(idx) => {
                }
            }
        }

        for m in self.modules.iter_mut() {
            m.frame(ctx);
        }
    }

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

pub struct MixerController {
    now: Instant,
    tx: EventSender<MixerEvent>,
}

impl MixerController {
    pub fn set_time(&mut self, now: Instant) {
        self.now = now;
    }

    pub fn add_module(&self, m: Box<dyn Module>) {
        self.tx.send(self.now, MixerEvent::AddModule(m));
    }
}
