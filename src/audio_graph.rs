use std::time::Instant;
use std::any::Any;

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
};

/*
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

    /*
    pub fn new_with_channel(module: Box<T>
    ) -> (Self, EventSender<Box<dyn FnOnce(&mut T) + Send>>) {
        let (tx, rx) = event_channel(50);
        (Self {
            module,
            rx: Some(rx),
        }, tx)
    }
    */
}
*/

pub struct AudioGraph {
    graph: Graph<Box<dyn Module>, ()>,
    sort: Vec<NodeIndex>,
}

impl AudioGraph {
    pub fn new(graph: Graph<Box<dyn Module>, ()>) -> Result<Self, Cycle<NodeIndex>> {
        let sort = algo::toposort(&graph, None)?;
        Ok(Self {
            graph,
            sort,
        })
    }

    // pub fn add_node(&mut self, gmod: GraphModule<impl Module>) {
        // self.graph.add_node(gmod);
    // }

    /*
    pub fn recv(&mut self, now: Instant) {
        for gmod in self.graph.node_weights_mut() {
            if let Some(rx) = &gmod.rx {
                if let Some(event) = rx.try_recv(now) {
                    event(&mut *gmod.module);
                }
            }
        }
    }
    */

    pub fn compute(&mut self, ctx: &AudioContext, out_buf: &mut [Sample]) {
        for idx in &self.sort {
            // TODO keep a buffer in gmod
            let module = &mut self.graph[*idx];
            module.compute(ctx, out_buf);
        }
    }
}
