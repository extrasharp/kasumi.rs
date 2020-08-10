use std::{
    time::{
        Instant,
    },
};

use flume::{
    self,
    Sender,
    Receiver,
};

//

pub type Sample = f32;

pub struct AudioContext {
    sample_rate: u32,
    now: Instant,
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

pub struct Mixer {
    modules: Vec<Box<dyn Module>>,
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
    }
}

impl Module for Mixer {
    fn frame(&mut self, ctx: &AudioContext) {
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

//

pub struct Chain {
    modules: Vec<Box<dyn Module>>,
}

impl Chain {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        self.modules.push(module);
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

pub struct Event<T> {
    pub timestamp: Instant,
    pub data: T,
}

struct EventReciever<T> {
    recv: Receiver<Event<T>>,
    last_event: Option<Event<T>>,
}

//

pub enum UtilityEvent {
    Volume(f32),
    Pan(f32),
}

pub struct Utility {
    recv: Receiver<Event<UtilityEvent>>,
    last_event: Option<Event<UtilityEvent>>,

    // 0 to 1
    volume: f32,

    // -1 to 1
    pan: f32,
}

impl Utility {
    pub fn new() -> (Self, Sender<Event<UtilityEvent>>) {
        let (tx, rx) = flume::unbounded();
        let ret = Self {
            recv: rx,
            last_event: None,
            volume: 1.,
            pan: 0.,
        };
        (ret, tx)
    }

    fn do_event(&mut self, ev: UtilityEvent) {
        match ev {
            UtilityEvent::Volume(val) => self.volume = val,
            UtilityEvent::Pan(val) => self.pan = val,
        }
    }
}

impl Module for Utility {
    fn frame(&mut self, _ctx: &AudioContext) {
        let ev = self.recv.try_recv();
        match ev {
            Ok(ev) => {
                self.do_event(ev.data);
            },
            Err(_) => {}
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
