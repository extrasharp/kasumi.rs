use std::{
    thread,
    time::{
        Duration,
        Instant,
    },
};

use std::cell::RefCell;
use std::sync::Arc;

use cpal::{
    self,
    Data,
    SampleRate,
    BufferSize,
    traits::*,
};
use petgraph::Graph;

use kasumi::{
    ComputeContext,
    CallbackBuffer,
    CALLBACK_BUFFER_LEN,
    modules::*,
    sample_buffer::*,
    event::*,
    audio_graph::*,
};

//

fn main() {
    let host = cpal::default_host();

    let mut devices = host.devices().unwrap();
    let device = devices.find(| d | {
        d.name().unwrap() == "pulse"
    }).unwrap();

    let mut supported_configs_ranges = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config_range = supported_configs_ranges.next()
        .expect("no supported configs");

    let s_conf = supported_config_range.with_sample_rate(SampleRate(44100));

    let mut config = s_conf.config();
    config.channels = 2;

    let mut graph = Graph::<std::cell::RefCell<GraphNode>, ()>::new();

    let sine = Sine::new();
    let (sine, ctl_sine) = Controlled::new(sine);
    let s_idx = graph.add_node(std::cell::RefCell::new(GraphNode {
        name: "sine".to_owned(),
        module: Box::new(sine),
        out_buf: [0.; CALLBACK_BUFFER_LEN],
    }));

    let util = Utility::new(s_idx);
    let (util, ctl_util) = Controlled::new(util);
    let u_idx = graph.add_node(std::cell::RefCell::new(GraphNode {
        name: "util".to_owned(),
        module: Box::new(util),
        out_buf: [0.; CALLBACK_BUFFER_LEN],
    }));

    graph.add_edge(s_idx, u_idx, ());

    let mixer = Mixer::new(vec![u_idx]);
    let m_idx = graph.add_node(std::cell::RefCell::new(GraphNode {
        name: "mixer".to_owned(),
        module: Box::new(mixer),
        out_buf: [0.; CALLBACK_BUFFER_LEN],
    }));

    graph.add_edge(u_idx, m_idx, ());

    let mut a_graph = ModuleGraph::new(graph, m_idx).unwrap();

    let mut ac = ComputeContext::new();
    let mut callback_buf = CallbackBuffer::new();

    let stream = device.build_output_stream(
        &config,
        move | data: &mut [f32], _ | {
            ac.now = Instant::now();

            a_graph.frame(&ac);

            for chunk in data.chunks_mut(CALLBACK_BUFFER_LEN) {
                callback_buf.fill_in_buf_f32(chunk);
                let (in_buf, out_buf) = callback_buf.buffers();
                a_graph.compute(&ac, out_buf);
                callback_buf.take_out_buf_f32(chunk);
            }
        },
        move | err | {
        },
    );

    loop {
        thread::sleep(Duration::from_secs(1));
        ctl_sine.send(Instant::now(), | s, _ | {
            s.set_frequency(880.);
        });
        thread::sleep(Duration::from_secs(1));
        ctl_sine.send(Instant::now(), | s, _ | {
            s.set_frequency(440.);
        });
    }
}
