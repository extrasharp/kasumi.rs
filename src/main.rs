use std::{
    thread,
    time::{
        Duration,
        Instant,
    },
};

use kasumi::{
    AudioSystem,
    modules::*,
    sample_buffer::*,
    audio_graph::*,
};

//

/*
ComputeContext -> CallbackContext
FrameContext
ComputeContext
*/

//

fn main() {
    let mut base = ModuleGraphBase::new();

    let sine = Sine::new();
    let (sine, ctl_sine) = Controlled::new(sine);
    let s_idx = base.add_module("sine".to_owned(), sine);

    let util = Utility::new(s_idx);
    let (util, ctl_util) = Controlled::new(util);
    let u_idx = base.add_module("util".to_owned(), util);

    base.add_edge(s_idx, u_idx);

    let mixer = Mixer::new(vec![u_idx]);
    let m_idx = base.add_module("mixer".to_owned(), mixer);

    base.add_edge(u_idx, m_idx);

    let a_graph = ModuleGraph::new(base, m_idx).unwrap();

    let audio_sys = AudioSystem::new(a_graph);

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
