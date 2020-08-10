use std::{
    time::{
        Instant,
    },
};

use crate::{
    event::*,
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

pub trait Module: Send {
    fn frame(&mut self, _ctx: &AudioContext) {}
    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]);
}

//

enum MixerEvent {
    AddModule(Box<dyn Module>),
    RemoveModule(usize),
}

pub struct Mixer {
    rx: EventReceiver<MixerEvent>,

    modules: Vec<Box<dyn Module>>,
}

impl Mixer {
    pub fn new(modules: Vec<Box<dyn Module>>) -> (Self, MixerController) {
        let (tx, rx) = event_channel(50);
        let ret = Self {
            rx,
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
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]) {
        let mut out = [0.; 2];
        for m in self.modules.iter_mut() {
            let mut computed = [0.; 2];
            m.compute(ctx, &in_buf, &mut computed);
            out[0] += computed[0];
            out[1] += computed[1];
        }
        *out_buf = out;
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

//

pub struct Chain {
    modules: Vec<Box<dyn Module>>,
}

impl Chain {
    pub fn new(modules: Vec<Box<dyn Module>>) -> Self {
        Self {
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
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]) {
        let mut local_in_buf = *in_buf;
        let mut local_out_buf = [0.; 2];
        for m in self.modules.iter_mut() {
            m.compute(ctx, &local_in_buf, &mut local_out_buf);
            local_in_buf = local_out_buf;
        }
        *out_buf = local_out_buf;
    }
}

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
        if let Some(event) = self.rx.try_recv(ctx.now) {
            match event {
                UtilityEvent::Volume(val) => self.volume = val,
                UtilityEvent::Pan(val) => self.pan = val,
            }
        }
    }

    fn compute(&mut self,
               _ctx: &AudioContext,
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]) {
        let pan = self.pan / 2. + 0.5;
        out_buf[0] = in_buf[0] * self.volume * (1. - pan);
        out_buf[1] = in_buf[1] * self.volume * pan;
    }
}

//

pub struct Sine {
    frame_ct: u32,
}

impl Sine {
    pub fn new() -> Self {
        Self {
            frame_ct: 0,
        }
    }
}

impl Module for Sine {
    fn compute(&mut self,
               _ctx: &AudioContext,
               _in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]) {
        out_buf[0] = f32::sin(self.frame_ct as f32 / 10.);
        out_buf[1] = f32::sin(self.frame_ct as f32 / 10.);
        self.frame_ct += 1;
    }
}

//

/*

struct Instrument {
    bank: SampleBank,
}

//

struct SampleBuffer {
    sample_rate: u32,
    // 1 or 2
    channel_count: u8,
    // interleaved
    samples: Vec<Sample>,
}

struct SampleBank {
    bank: Vec<SampleBuffer>,
}

//

struct FileStream {
}

// inst.add_sample()
*/
