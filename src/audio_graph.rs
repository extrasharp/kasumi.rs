use std::time::Instant;

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
};

pub struct GraphModule<T: Module + ?Sized> {
    module: Box<T>,
    rx: Option<EventReceiver<Box<dyn FnOnce(&mut T) + Send>>>,
}

impl<T: Module + ?Sized> GraphModule<T> {
    pub fn new(module: Box<T>,
               rx: Option<EventReceiver<Box<dyn FnOnce(&mut T) + Send>>>
    ) -> Self {
        Self {
            module,
            rx,
        }
    }
}

pub struct ModuleGraph {
    graph: Graph<GraphModule<dyn Module>, ()>,
    sort: Vec<NodeIndex>,
}

impl ModuleGraph {
    pub fn new(graph: Graph<GraphModule<dyn Module>, ()>) -> Result<Self, Cycle<NodeIndex>> {
        let sort = algo::toposort(&graph, None)?;
        Ok(Self {
            graph,
            sort,
        })
    }

    pub fn recv(&mut self, now: Instant) {
        for gmod in self.graph.node_weights_mut() {
            if let Some(rx) = &gmod.rx {
                if let Some(event) = rx.try_recv(now) {
                    event(&mut *gmod.module);
                }
            }
        }
    }

    pub fn compute(&mut self, ctx: &AudioContext) {
        for idx in &self.sort {
            self.graph[*idx].module.compute(ctx);
        }
    }
}
