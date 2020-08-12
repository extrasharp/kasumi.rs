use std::{
    cell::{
        Ref,
        RefCell,
    },
};

use petgraph::{
    algo::{
        self,
        Cycle,
    },
    Graph,
    graph::NodeIndex,
};

use crate::{
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
    /*
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
    */

    pub fn with_output_buffer<F>(&self, idx: NodeIndex, func: F) -> Result<(), &str>
    where
        F: FnOnce(&[Sample])
    {
        if idx == self.curr_idx {
            Err("1")
        } else if !self.graph.graph.contains_edge(idx, self.curr_idx) {
            Err("2")
        } else if let Some(gnode) = self.graph.graph.node_weight(idx) {
            func(&gnode.borrow().out_buf[0..self.audio_context.chunk_len]);
            Ok(())
        } else {
            Err("3")
        }
    }
}

pub struct GraphNode {
    name: String,
    module: Box<dyn Module>,
    out_buf: [Sample; CALLBACK_BUFFER_LEN],
}

impl GraphNode {
    pub fn output_buffer(&self) -> &[Sample] {
        &self.out_buf
    }
}

pub struct ModuleGraphBase {
    graph: Graph<RefCell<GraphNode>, ()>,
}

impl ModuleGraphBase {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    pub fn add_module<M: Module + 'static>(&mut self, name: String, module: M) -> NodeIndex {
        self.graph.add_node(RefCell::new(GraphNode {
            name,
            module: Box::new(module),
            out_buf: [0.; CALLBACK_BUFFER_LEN],
        }))
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ());
    }
}

pub struct ModuleGraph {
    output: NodeIndex,
    graph: Graph<RefCell<GraphNode>, ()>,
    sort: Vec<NodeIndex>,
}

impl ModuleGraph {
    pub fn new(base: ModuleGraphBase, output: NodeIndex) -> Result<Self, Cycle<NodeIndex>> {
        let graph = base.graph;
        let sort = algo::toposort(&graph, None)?;
        Ok(Self {
            output,
            graph,
            sort,
        })
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
}
