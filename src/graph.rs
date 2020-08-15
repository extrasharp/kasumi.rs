use std::{
    cell::{
        Ref,
        RefCell,
    },
    time::Instant,
};

use petgraph::{
    algo::{
        self,
        Cycle,
    },
    visit::{
        EdgeRef,
        IntoNodeIdentifiers,
        IntoNeighborsDirected,
    },
    stable_graph::{
        StableGraph,
    },
    graph::NodeIndex,
};
use smallvec::SmallVec;

use crate::{
    event::*,
    channel::*,

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

// TODO
struct OutBuf {
}

impl OutBuf {
    pub fn new() -> Self {
        Self {
        }
    }
}

type InputId = usize;

const MAX_INPUT_BUFFER_CT: usize = 32;

#[derive(Clone)]
struct GraphBase {
    graph: StableGraph<usize, InputId>,
    sort: Vec<NodeIndex>,
    output: Option<NodeIndex>,
}

impl GraphBase {
    fn new() -> Self {
        let graph = StableGraph::new();
        let sort = Vec::new();
        let output = None;
        Self {
            graph,
            sort,
            output,
        }
    }
}

pub struct Controller {
    tx: EventSender<Swap>,
    rx: Receiver<Swap>,

    base: GraphBase,

    total_module_count: usize,
    added_modules: Vec<Box<dyn Module>>,
    added_out_bufs: Vec<Box<[Sample; CALLBACK_BUFFER_LEN]>>,
    removed: Vec<usize>,
}

impl Controller {
    pub fn add_module<M: Module + 'static>(&mut self, module: M) -> NodeIndex {
        let id = self.total_module_count;
        let node_idx = self.base.graph.add_node(id);
        self.added_modules.push(Box::new(module));
        self.added_out_bufs.push(Box::new([0.; CALLBACK_BUFFER_LEN]));
        self.total_module_count += 1;
        node_idx
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, input_number: InputId) {
        self.base.graph.add_edge(from, to, input_number);
    }

    /*
    /// invalidates any ids that have been returned to the user
    pub fn remove_module(&mut self, id: usize) {
    }
    */

    pub fn set_as_output(&mut self, id: Option<NodeIndex>) {
        self.base.output = id;
    }

    pub fn push_changes(&mut self, now: Instant) {
        // TODO report cycles
        self.base.sort = algo::toposort(&self.base.graph, None).unwrap();
        let swp = self.make_swap();
        self.tx.send(now, swp);
    }

    pub fn frame(&self) {
        if let Some(_swap) = self.rx.try_recv() {
            // jut let it die
        }
    }

    fn make_swap(&mut self) -> Swap {
        Swap {
            base: self.base.clone(),
            added_modules: self.added_modules.drain(..).collect(),
            added_out_bufs: self.added_out_bufs.drain(..).collect(),
            removed: self.removed.drain(..).collect(),
            modules: Vec::with_capacity(self.total_module_count),
            out_bufs: Vec::with_capacity(self.total_module_count),
        }
    }
}

struct Swap {
    base: GraphBase,
    added_modules: Vec<Box<dyn Module>>,
    added_out_bufs: Vec<Box<[Sample; CALLBACK_BUFFER_LEN]>>,
    removed: Vec<usize>,
    modules: Vec<Box<dyn Module>>,
    out_bufs: Vec<Box<[Sample; CALLBACK_BUFFER_LEN]>>,
}

pub struct ControlledModGraph {
    tx: Sender<Swap>,
    rx: EventReceiver<Swap>,

    mgraph: ModGraph,
}

impl ControlledModGraph {
    pub fn new() -> (Self, Controller) {
        let base = GraphBase::new();
        let (tx, rx) = channel(50);
        let (etx, erx) = event_channel(50);

        let mgraph = ModGraph {
            base: base.clone(),
            modules: Vec::new(),
            out_bufs: Vec::new(),
            temp_buf: [0.; CALLBACK_BUFFER_LEN],
        };

        let ret = Self {
            tx,
            rx: erx,
            mgraph,
        };
        let ctl = Controller {
            tx: etx,
            rx,
            base: base,
            total_module_count: 0,
            added_modules: Vec::new(),
            added_out_bufs: Vec::new(),
            removed: Vec::new(),
        };
        (ret, ctl)
    }

    pub fn frame(&mut self, ctx: &CallbackContext) {
        while let Some(swap) = self.rx.try_recv(ctx.now) {
            let swap = self.mgraph.reload(swap);
            self.tx.send(swap);
        }

        self.mgraph.frame(ctx);
    }

    pub fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
        self.mgraph.compute(ctx, out_buf);
    }
}

struct ModGraph {
    base: GraphBase,

    modules: Vec<Box<dyn Module>>,
    out_bufs: Vec<Box<[Sample; CALLBACK_BUFFER_LEN]>>,
    temp_buf: [Sample; CALLBACK_BUFFER_LEN],
}

// TODO make sure output select is valid

impl ModGraph {
    fn reload(&mut self, mut swap: Swap) -> Swap {
        use std::mem;

        mem::swap(&mut self.base, &mut swap.base);

        // note: graphbase must already be updated by now
        //   to reflect removals and new additions
        for idx in swap.removed.drain(..) {
            self.modules.remove(idx);
            self.out_bufs.remove(idx);
        }

        mem::swap(&mut self.modules, &mut swap.modules);
        for mnode in swap.modules.drain(..) {
            self.modules.push(mnode);
        }
        for mnode in swap.added_modules.drain(..) {
            self.modules.push(mnode);
        }

        mem::swap(&mut self.out_bufs, &mut swap.out_bufs);
        for mnode in swap.out_bufs.drain(..) {
            self.out_bufs.push(mnode);
        }
        for mnode in swap.added_out_bufs.drain(..) {
            self.out_bufs.push(mnode);
        }

        swap
    }

    pub fn frame(&mut self, ctx: &CallbackContext) {
        for node_idx in &self.base.sort {
            let module_idx = self.base.graph[*node_idx];
            self.modules[module_idx].frame(&ctx);
        }
    }

    pub fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
        let buf_len = out_buf.len();

        if let Some(output_node_idx) = self.base.output {
            for node_idx in &self.base.sort {
                let module_idx = self.base.graph[*node_idx];
                {
                    let mut in_bufs: SmallVec<[InputBuffer; MAX_INPUT_BUFFER_CT]> = SmallVec::new();

                    let input_edges = self.base.graph.edges_directed(*node_idx, petgraph::Direction::Incoming);
                    for edge_ref in input_edges {
                        let input_idx = self.base.graph[edge_ref.source()];
                        in_bufs.push(InputBuffer {
                            id: *edge_ref.weight(),
                            buf: &self.out_bufs[input_idx][0..buf_len],
                        });
                    }

                    self.modules[module_idx].compute(&ctx, &in_bufs, &mut self.temp_buf[0..buf_len]);
                }

                for buf_at in 0..buf_len {
                    self.out_bufs[module_idx][buf_at] = self.temp_buf[buf_at];
                }
            }

            let output_idx = self.base.graph[output_node_idx];
            for i in 0..buf_len {
                out_buf[i] = self.out_bufs[output_idx][i];
            }
        } else {
            for i in 0..buf_len {
                out_buf[i] = 0.;
            }
        }
    }
}

/*
struct GraphBase {
}

struct GraphChange {
    graph: Graph<ModuleId, InputId>,
    sort: Vec<ModuleId>,
    output: Option<ModuleId>,
    new_modules: Vec<(ModuleId, Box<dyn Module>)>,
}


struct GraphNode {
    module: bool, // Box<dyn Module>,

    id: ModuleId,
    sort_idx: usize,
}

pub struct ModuleGraphMaximums {
    nodes: usize,
    edges: usize,
}

// for now doesnt make sense to insert modules without a known ID
//   youd have to connect them to something
// whatever wraps this could handle that

pub struct ModuleGraph {
    maxes: ModuleGraphMaximums,

    graph: Graph<GraphNode, usize>,
    output: Option<NodeIndex>,

    node_idx_lookup: Vec<(ModuleId, NodeIndex)>,

    sort: Vec<NodeIndex>,
    out_bufs: Vec<[Sample; CALLBACK_BUFFER_LEN]>,
    temp_buf: [Sample; CALLBACK_BUFFER_LEN],
}

impl ModuleGraph {
    pub fn new(maxes: ModuleGraphMaximums) -> Self {
        let graph = Graph::with_capacity(maxes.nodes, maxes.edges);
        let output = None;
        let node_idx_lookup = Vec::with_capacity(maxes.nodes);
        let sort = Vec::with_capacity(maxes.nodes);
        let out_bufs = Vec::with_capacity(maxes.nodes);
        let temp_buf = [0.; CALLBACK_BUFFER_LEN];
        Self {
            maxes,
            graph,
            output,
            node_idx_lookup,
            sort,
            out_bufs,
            temp_buf,
        }
    }

    pub fn add_module<M: Module + 'static>(&mut self, id: ModuleId, module: M) {
    }

    pub fn add_edge(&mut self, from: ModuleId, to: ModuleId, input_number: usize) {
    }

    pub fn set_as_output(&mut self, id: ModuleId) {
    }

    // updates sort and stuff
    pub fn bake(&mut self) {
    }

    pub(crate) fn frame(&mut self, ctx: &CallbackContext) {
    }

    pub(crate) fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
    }
}
*/

/*
// module ids
//   separate from nodeindex
//   allows you to add nodes at runtime and connect them up

//

const MAX_INPUT_BUFFER_CT: usize = 32;

pub struct GraphNode {
    module: Box<dyn Module>,
    sort_idx: usize,
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

    pub fn add_module<M: Module + 'static>(&mut self, module: M) -> NodeIndex {
        self.graph.add_node(GraphNode {
            module: Box::new(module),
            sort_idx: 0,
        })
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, input_idx: usize) {
        self.graph.add_edge(from, to, input_idx);
    }
}

/*
enum ModuleGraphEvent {
    AddModule(GraphNode),
    AddEdge(NodeIndex, NodeIndex, usize),
}

pub struct ModuleGraphController {
    tx: EventSender<ModuleGraphEvent>,
}

impl ModuleGraphController {
    pub fn add_module<M: Module + 'static>(&self, now: Instant, module: M) -> NodeIndex {
        self.tx.send(now, GraphNode {
            module: Box::new(module),
            sort_idx: 0,
        });
    }
}
*/

pub struct ModuleGraph {
    output: NodeIndex,
    graph: Graph<GraphNode, usize>,
    sort: Vec<NodeIndex>,

    module_id_map: Vec<NodeIndex>,

    out_bufs: Vec<[Sample; CALLBACK_BUFFER_LEN]>,
    temp_buf: [Sample; CALLBACK_BUFFER_LEN],
}

impl ModuleGraph {
    pub fn new(base: ModuleGraphBase, output: NodeIndex) -> Result<Self, Cycle<NodeIndex>> {
        let mut graph = base.graph;

        let sort = algo::toposort(&graph, None)?;
        for (i, idx) in sort.iter().enumerate() {
            graph[*idx].sort_idx = i;
        }

        let module_id_map = Vec::new();

        let out_bufs = vec![[0.; CALLBACK_BUFFER_LEN]; sort.len()];
        let temp_buf = [0.; CALLBACK_BUFFER_LEN];

        Ok(Self {
            output,
            graph,
            sort,
            module_id_map,
            out_bufs,
            temp_buf,
        })
    }

    pub fn frame(&mut self, ctx: &CallbackContext) {
        for idx in &self.sort {
            self.graph[*idx].module.frame(&ctx);
        }
    }

    pub fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
        let buf_len = out_buf.len();

        // TODO rename i and j
        for (this_sort_idx, graph_idx) in self.sort.iter().enumerate() {
            {
                let mut in_bufs: SmallVec<[InputBuffer; MAX_INPUT_BUFFER_CT]> = SmallVec::new();

                let input_edges = self.graph.edges_directed(*graph_idx, petgraph::Direction::Incoming);
                for edge_ref in input_edges {
                    let sort_idx = self.graph[edge_ref.source()].sort_idx;
                    in_bufs.push(InputBuffer {
                        id: *edge_ref.weight(),
                        buf: &self.out_bufs[sort_idx][0..buf_len],
                    });
                }

                self.graph[*graph_idx].module.compute(&ctx, &in_bufs, &mut self.temp_buf[0..buf_len]);
            }

            for buf_at in 0..buf_len {
                self.out_bufs[this_sort_idx][buf_at] = self.temp_buf[buf_at];
            }
        }

        let sort_idx = self.graph[self.output].sort_idx;
        for i in 0..buf_len {
            out_buf[i] = self.out_bufs[sort_idx][i];
        }
    }
}
*/
