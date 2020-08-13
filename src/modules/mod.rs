use crate::{
    system::CallbackContext,
    Sample,
};

//

pub struct InputBuffer<'a> {
    pub id: usize,
    pub buf: &'a [Sample],
}

pub trait Module: Send {
    fn frame(&mut self, _ctx: &CallbackContext) { }
    fn compute<'a>(&mut self,
                   ctx: &CallbackContext,
                   in_bufs: &[InputBuffer<'a>],
                   out_buf: &mut [Sample]);
}

pub mod prelude {
    pub use crate::{
        system::CallbackContext,
        Sample,
    };
    pub use super::Module;
    pub use super::InputBuffer;
}

//

mod sine;
pub use sine::*;

mod controlled;
pub use controlled::*;

mod utility;
pub use utility::*;

mod mixer;
pub use mixer::*;

mod buf_player;
pub use buf_player::*;

/*
mod sfx_player;
pub use sfx_player::*;
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
