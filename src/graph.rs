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
    visit::EdgeRef,
    Graph,
    graph::NodeIndex,
};
use smallvec::SmallVec;

use crate::{
    modules::{
        Module,
        InputBuffer,
    },
    system::{
        CallbackContext,
        CALLBACK_BUFFER_LEN,
    },
    Sample,
};

//

//

const MAX_INPUT_BUFFER_CT: usize = 32;

// TODO figure out what to do abt chunk_len
//        wrt ending it to .frame()
//      is valid across frames as long as you dont make a new ctx (but you do have to because the 'a thing
//      can have separate frame and compute contexts
//        what data should you be able to have in .frame()
//        would you ever need input buffers?
//        krate math based on what they were last frame?
//        frame context doesnt need node idx

pub struct GraphContext<'a> {
    callback_context: &'a CallbackContext,
    chunk_len: usize,
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
        /*
        if idx == self.curr_idx {
            Err("1")
        } else if !self.graph.graph.contains_edge(idx, self.curr_idx) {
            Err("2")
        } else if let Some(gnode) = self.graph.graph.node_weight(idx) {
            // func(&gnode.borrow().out_buf[0..self.chunk_len]);
            Ok(())
        } else {
            Err("3")
        }
        */

        Ok(())
    }

    //

    #[inline]
    pub fn callback_context(&self) -> &CallbackContext {
        &self.callback_context
    }
}

pub struct GraphNode {
    name: String,
    module: Box<dyn Module>,
    sort_idx: usize,
}

impl GraphNode {
    /*
    pub fn output_buffer(&self) -> &[Sample] {
        &self.out_buf
    }
    */
}

pub struct ModuleGraphBase {
    graph: Graph<GraphNode, usize>,
}

impl ModuleGraphBase {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    pub fn add_module<M: Module + 'static>(&mut self, name: String, module: M) -> NodeIndex {
        self.graph.add_node(GraphNode {
            name,
            module: Box::new(module),
            sort_idx: 0,
        })
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, 0);
    }
}

pub struct ModuleGraph {
    output: NodeIndex,
    graph: Graph<GraphNode, usize>,
    sort: Vec<NodeIndex>,
    out_bufs: Vec<[Sample; CALLBACK_BUFFER_LEN]>,
    temp_buf: [Sample; CALLBACK_BUFFER_LEN]
}

impl ModuleGraph {
    pub fn new(base: ModuleGraphBase, output: NodeIndex) -> Result<Self, Cycle<NodeIndex>> {
        let mut graph = base.graph;
        let sort = algo::toposort(&graph, None)?;
        for (i, idx) in sort.iter().enumerate() {
            graph[*idx].sort_idx = i;
        }
        let out_bufs = vec![[0.; CALLBACK_BUFFER_LEN]; sort.len()];
        let temp_buf = [0.; CALLBACK_BUFFER_LEN];
        Ok(Self {
            output,
            graph,
            sort,
            out_bufs,
            temp_buf,
        })
    }

    pub fn frame(&mut self, ctx: &CallbackContext) {
        let mut ctx = GraphContext {
            callback_context: ctx,
            chunk_len: 0,
        };

        for idx in &self.sort {
            self.graph[*idx].module.frame(&ctx);
        }
    }

    pub fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
        let buf_len = out_buf.len();

        let mut ctx = GraphContext {
            callback_context: ctx,
            chunk_len: buf_len,
        };

        let mut idx = 0;

        for (i, idx) in self.sort.iter().enumerate() {
            {
                let mut in_bufs: SmallVec<[InputBuffer; MAX_INPUT_BUFFER_CT]> = SmallVec::new();

                let input_edges = self.graph.edges_directed(*idx, petgraph::Direction::Incoming);
                for edge_ref in input_edges {
                    let sort_idx = self.graph[edge_ref.source()].sort_idx;
                    in_bufs.push(InputBuffer {
                        id: *edge_ref.weight(),
                        buf: &self.out_bufs[sort_idx][0..buf_len],
                    });
                }

                self.graph[*idx].module.compute(&ctx, &in_bufs, &mut self.temp_buf[0..buf_len]);
            }

            for j in 0..buf_len {
                self.out_bufs[i][j] = self.temp_buf[j];
            }
        }

        let sort_idx = self.graph[self.output].sort_idx;
        for i in 0..buf_len {
            out_buf[i] = self.out_bufs[sort_idx][i];
        }
    }
}