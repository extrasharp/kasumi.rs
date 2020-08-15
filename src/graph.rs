use std::time::Instant;

use petgraph::{
    algo,
    visit::EdgeRef,
    stable_graph::StableGraph,
    graph::{
        NodeIndex,
        EdgeIndex,
    },
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

/*
is there a way to do supercollider 'module finish' so itll remove itself from the graph
imagine modules as SynthDefs, remove per module, dont 'remove an entire chain if one finishes'
check on .frame()
  could send in a &mut Status struct that it changes to indicate whatever
  optional
but how to report changes back to the controller
*/

//

const MAX_INPUT_BUFFER_CT: usize = 32;
const MAX_REMOVE_PER_FRAME: usize = 64;

//

// TODO use this
struct OutBuf {
    buf: [Sample; CALLBACK_BUFFER_LEN],
}

impl OutBuf {
    fn new() -> Self {
        Self {
            buf: [0.; CALLBACK_BUFFER_LEN],
        }
    }

    #[inline]
    fn buffer(&self) -> &[Sample] {
        &self.buf
    }

    #[inline]
    fn buffer_mut(&mut self) -> &mut [Sample] {
        &mut self.buf
    }
}

//

#[derive(Clone)]
struct GraphBase {
    graph: StableGraph<usize, usize>,
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

//

struct Graph {
    base: GraphBase,
    modules: Vec<Box<dyn Module>>,
    out_bufs: Vec<Box<OutBuf>>,
    temp_buf: OutBuf,
}

impl Graph {
    fn frame(&mut self, ctx: &CallbackContext) {
        for node_idx in &self.base.sort {
            let module_idx = self.base.graph[*node_idx];
            self.modules[module_idx].frame(&ctx);
        }
    }

    fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
        let buf_len = out_buf.len();

        if let Some(output_node_idx) = self.base.output {
            for node_idx in &self.base.sort {
                let module_idx = self.base.graph[*node_idx];
                {
                    let mut in_bufs: SmallVec<[InputBuffer; MAX_INPUT_BUFFER_CT]> = SmallVec::new();

                    // TODO you could memoize this
                    // probably not a big deal though honestly
                    let input_edges = self.base.graph.edges_directed(*node_idx, petgraph::Direction::Incoming);
                    for edge_ref in input_edges {
                        let input_idx = self.base.graph[edge_ref.source()];
                        in_bufs.push(InputBuffer {
                            id: *edge_ref.weight(),
                            buf: &self.out_bufs[input_idx].buffer()[0..buf_len],
                        });
                    }

                    self.modules[module_idx].compute(&ctx, &in_bufs, &mut self.temp_buf.buffer_mut()[0..buf_len]);
                }

                for buf_at in 0..buf_len {
                    self.out_bufs[module_idx].buffer_mut()[buf_at] = self.temp_buf.buffer()[buf_at];
                }
            }

            let output_idx = self.base.graph[output_node_idx];
            for i in 0..buf_len {
                out_buf[i] = self.out_bufs[output_idx].buffer()[i];
            }
        } else {
            for i in 0..buf_len {
                out_buf[i] = 0.;
            }
        }
    }
}

//

struct Swap {
    base: GraphBase,
    added_modules: Vec<Box<dyn Module>>,
    added_out_bufs: Vec<Box<OutBuf>>,
    removed: Vec<usize>,
    modules: Vec<Box<dyn Module>>,
    out_bufs: Vec<Box<OutBuf>>,
}

//

// TODO make sure output select is valid
//   if you remove the last node set output to None

pub struct Controller {
    tx: EventSender<Swap>,
    rx: Receiver<Swap>,

    base: GraphBase,

    total_module_count: usize,
    added_modules: Vec<Box<dyn Module>>,
    added_out_bufs: Vec<Box<OutBuf>>,
    removed: Vec<usize>,
}

impl Controller {
    pub fn add_module<M: Module + 'static>(&mut self, module: M) -> NodeIndex {
        let id = self.total_module_count;
        let node_idx = self.base.graph.add_node(id);
        self.added_modules.push(Box::new(module));
        self.added_out_bufs.push(Box::new(OutBuf::new()));
        self.total_module_count += 1;
        node_idx
    }

    pub fn add_edge(&mut self, from: NodeIndex, to: NodeIndex, input_number: usize) -> EdgeIndex {
        self.base.graph.add_edge(from, to, input_number)
    }

    pub fn remove_module(&mut self, idx: NodeIndex) {
        // TODO report not found error
        let removed_idx = self.base.graph.remove_node(idx).unwrap();
        // TODO do this on push changes,
        //   build vec of removals and go through all at once
        for idx in self.base.graph.node_weights_mut() {
            if *idx > removed_idx {
                *idx -= 1;
            }
        }
        self.removed.push(removed_idx);
        self.total_module_count -= 1;
    }

    pub fn remove_edge(&mut self, idx: EdgeIndex) {
        // TODO report error
        self.base.graph.remove_edge(idx);
    }

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
        self.rx.try_recv();
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

//

pub struct ControlledGraph {
    tx: Sender<Swap>,
    rx: EventReceiver<Swap>,
    graph: Graph,
}

impl ControlledGraph {
    pub fn new() -> (Self, Controller) {
        let base = GraphBase::new();
        let (tx, rx) = channel(50);
        let (etx, erx) = event_channel(50);

        let graph = Graph {
            base: base.clone(),
            modules: Vec::new(),
            out_bufs: Vec::new(),
            temp_buf: OutBuf::new(),
        };

        let ret = Self {
            tx,
            rx: erx,
            graph,
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
            let swap = self.reload(swap);
            self.tx.send(swap);
        }

        self.graph.frame(ctx);
    }

    pub fn compute(&mut self, ctx: &CallbackContext, out_buf: &mut [Sample]) {
        self.graph.compute(ctx, out_buf);
    }

    //

    fn reload(&mut self, mut swap: Swap) -> Swap {
        use std::mem;

        let graph = &mut self.graph;

        mem::swap(&mut graph.base, &mut swap.base);

        // note: graphbase must already be updated by now
        //   to reflect removals and new additions
        for idx in swap.removed.drain(..) {
            graph.modules.remove(idx);
            graph.out_bufs.remove(idx);
        }

        mem::swap(&mut graph.modules, &mut swap.modules);
        for mnode in swap.modules.drain(..) {
            graph.modules.push(mnode);
        }
        for mnode in swap.added_modules.drain(..) {
            graph.modules.push(mnode);
        }

        mem::swap(&mut graph.out_bufs, &mut swap.out_bufs);
        for mnode in swap.out_bufs.drain(..) {
            graph.out_bufs.push(mnode);
        }
        for mnode in swap.added_out_bufs.drain(..) {
            graph.out_bufs.push(mnode);
        }

        swap
    }
}
