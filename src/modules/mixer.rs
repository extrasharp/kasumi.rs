use petgraph::graph::NodeIndex;

use crate::{
    Sample,
    audio_graph::GraphContext,
};

use super::{
    Module,
};

//

pub struct Mixer {
    in_modules: Vec<NodeIndex>,
}

impl Mixer {
    pub fn new(in_modules: Vec<NodeIndex>) -> Self {
        Self {
            in_modules,
        }
    }
}

// TODO maybe turning off bounds checking would speed it up

impl Module for Mixer {
    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
        for i in 0..out_buf.len() {
            out_buf[i] = 0.;
        }

        for idx in &self.in_modules {
            ctx.with_output_buffer(*idx, | in_buf | {
                for i in 0..out_buf.len() {
                    out_buf[i] += in_buf[i];
                }
            });
        }
    }
}
