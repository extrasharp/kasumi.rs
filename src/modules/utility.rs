use petgraph::graph::NodeIndex;

use crate::{
    Sample,
    graph::GraphContext,
};

use super::Module;

//

pub struct Utility {
    input: NodeIndex,
    volume: f32,
    pan: f32,
}

impl Utility {
    pub fn new(input: NodeIndex) -> Self {
        Self {
            input,
            volume: 1.,
            pan: 0.,
        }
    }

    pub fn set_volume(&mut self, to: f32) {
        self.volume = to;
    }

    pub fn set_pan(&mut self, to: f32) {
        self.pan = to;
    }
}

impl Module for Utility {
    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
        let frame_size = out_buf.len();
        let pan = self.pan / 2. + 0.5;

        for i in 0..(frame_size / 2) {
            ctx.with_output_buffer(self.input, | in_buf | {
                out_buf[i * 2] = in_buf[i * 2] * self.volume * (1. - pan);
                out_buf[i * 2 + 1] = in_buf[i * 2 + 1] * self.volume * pan;
            }).unwrap();
        }
    }
}
