use crate::{
    event::*,
    AudioContext,
    Sample,
};

pub trait Module: Send {
    fn frame(&mut self, _ctx: &AudioContext) {}
    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample],
               out_buf: &mut [Sample]);
}

mod mixer;
pub use mixer::*;

mod sine;
pub use sine::*;

mod chain;
pub use chain::*;

mod utility;
pub use utility::*;

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
