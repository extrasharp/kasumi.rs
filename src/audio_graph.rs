use std::time::Instant;
use std::any::Any;
use std::cell::RefCell;
use std::sync::Arc;

use petgraph::{
    algo::{
        self,
        Cycle,
    },
    Graph,
    graph::NodeIndex,
};

use crate::{
    event::*,
    modules::Module,
    AudioContext,
    Sample,
    CALLBACK_BUFFER_LEN,
};

pub struct GraphNode {
    // TODO, proper ::new()
    pub name: String,
    pub module: Box<dyn Module>,
    pub out_buf: [Sample; CALLBACK_BUFFER_LEN],
}

pub struct ModuleGraph {
    output: NodeIndex,
    graph: Graph<GraphNode, ()>,
    sort: Vec<NodeIndex>,
}

impl ModuleGraph {
    pub fn new(graph: Graph<GraphNode, ()>, output: NodeIndex) -> Result<Self, Cycle<NodeIndex>> {
        let sort = algo::toposort(&graph, None)?;
        Ok(Self {
            output,
            graph,
            sort,
        })
    }

    pub fn frame(&mut self, ctx: &AudioContext) {
        for node in self.graph.node_weights_mut() {
            node.module.frame(ctx);
        }
    }

    pub fn compute(&mut self, ctx: &AudioContext, out_buf: &mut [Sample]) {
        let buf_len = out_buf.len();
        for idx in &self.sort {
            let node = &mut self.graph[*idx];
            node.module.compute(ctx, &mut node.out_buf[0..buf_len]);
        }
        let out_node = &mut self.graph[self.output];
        for i in 0..buf_len {
            out_buf[i] = out_node.out_buf[i];
        }
    }
}
