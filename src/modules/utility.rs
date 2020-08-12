use petgraph::graph::NodeIndex;

use crate::{
    Sample,
    audio_graph::GraphContext,
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
}

impl Module for Utility {
    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
        let frame_size = out_buf.len();
        let pan = self.pan / 2. + 0.5;

        let node_ref = ctx.get_output_buffer(self.input).unwrap();
        let in_buf = &node_ref.out_buf;

        for i in 0..(frame_size / 2) {
            out_buf[i * 2] = in_buf[i * 2] * self.volume * (1. - pan);
            out_buf[i * 2 + 1] = in_buf[i * 2 + 1] * self.volume * pan;
        }
    }
}
