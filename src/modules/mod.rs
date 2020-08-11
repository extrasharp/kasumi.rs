use crate::{
    event::*,
    AudioContext,
    Sample,
};

pub trait Module: Send {
    fn compute(&mut self, ctx: &AudioContext);
    fn out_buffer(&self) -> &[Sample];
}

/*
mod mixer;
pub use mixer::*;

mod sine;
pub use sine::*;

mod sine2;
pub use sine2::*;

mod chain;
pub use chain::*;

mod utility;
pub use utility::*;

mod buf_player;
pub use buf_player::*;
*/

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
