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
            let node_ref = ctx.get_output_buffer(*idx).unwrap();
            let buf = &node_ref.out_buf;
            for i in 0..out_buf.len() {
                out_buf[i] += buf[i];
            }
        }

        // self.in_modules.map(ctx.graph.get_buffer)
        /*
        for m in self.modules.iter_mut() {
            for i in 0..out_buf.len() {
                out_buf[i] = 0.;
            }
            m.compute(ctx, &in_buf, &mut self.dummy[0..in_buf.len()]);
            for i in 0..out_buf.len() {
                out_buf[i] += self.dummy[i];
            }
        }
        */
    }
}
