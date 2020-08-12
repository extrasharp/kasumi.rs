use std::time::Instant;
use std::cell::RefCell;
use std::cell::Ref;

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
    ComputeContext,
    Sample,
    CALLBACK_BUFFER_LEN,
};

pub struct GraphContext<'a> {
    pub audio_context: &'a ComputeContext,
    graph: &'a ModuleGraph,
    curr_idx: NodeIndex,
}

impl<'a> GraphContext<'a> {
    pub fn get_output_buffer(&self, idx: NodeIndex) -> Result<Ref<GraphNode>, &str> {
        if idx == self.curr_idx {
            Err("1")
        } else if !self.graph.graph.contains_edge(idx, self.curr_idx) {
            Err("2")
        } else if let Some(gnode) = self.graph.graph.node_weight(idx) {
            Ok(gnode.borrow())
        } else {
            Err("3")
        }
    }
}

pub struct GraphNode {
    // TODO, proper ::new()
    pub name: String,
    pub module: Box<dyn Module>,
    pub out_buf: [Sample; CALLBACK_BUFFER_LEN],
}

pub struct ModuleGraph {
    pub output: NodeIndex,
    pub graph: Graph<RefCell<GraphNode>, ()>,
    pub sort: Vec<NodeIndex>,
}

impl ModuleGraph {
    pub fn new(graph: Graph<RefCell<GraphNode>, ()>, output: NodeIndex) -> Result<Self, Cycle<NodeIndex>> {
        let sort = algo::toposort(&graph, None)?;
        Ok(Self {
            output,
            graph,
            sort,
        })
    }

    pub fn add_module<M: Module + 'static>(&mut self, name: String, module: M) -> NodeIndex {
        self.graph.add_node(RefCell::new(GraphNode {
            name,
            module: Box::new(module),
            out_buf: [0.; CALLBACK_BUFFER_LEN],
        }))
    }

    pub fn frame(&mut self, ctx: &ComputeContext) {
        let mut ctx = GraphContext {
            audio_context: ctx,
            graph: self,
            curr_idx: NodeIndex::end(),
        };

        for idx in &self.sort {
            ctx.curr_idx = *idx;
            let mut node = self.graph[*idx].borrow_mut();
            node.module.frame(&ctx);
        }
    }

    pub fn compute(&mut self, ctx: &ComputeContext, out_buf: &mut [Sample]) {
        let mut ctx = GraphContext {
            audio_context: ctx,
            graph: self,
            curr_idx: NodeIndex::end(),
        };

        let buf_len = out_buf.len();
        for idx in &self.sort {
            ctx.curr_idx = *idx;
            let mut node_ref = self.graph[*idx].borrow_mut();
            let node = &mut *node_ref;
            node.module.compute(&ctx, &mut node.out_buf[0..buf_len]);
        }

        let out_node = &mut self.graph[self.output].borrow();
        for i in 0..buf_len {
            out_buf[i] = out_node.out_buf[i];
        }
    }

    /*
    fn get_buffer(&self, node_idx: NodeIndex) -> &[Sample] {
    }
    */
}
