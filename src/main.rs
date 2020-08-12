use std::{
    thread,
    sync::Arc,
    time::{
        Duration,
        Instant,
    },
};

use kasumi::{
    system::System,
    modules::*,
    sample_buffer::*,
    graph::*,
};

//

fn main() {
    let mut base = ModuleGraphBase::new();

    let sine = Sine::new();
    let (sine, ctl_sine) = Controlled::new(sine);
    let s_idx = base.add_module("sine".to_owned(), sine);

    /*

    let util = Utility::new(s_idx);
    let (util, ctl_util) = Controlled::new(util);
    let u_idx = base.add_module("util".to_owned(), util);

    base.add_edge(s_idx, u_idx);

    let bplay = BufPlayer::new();
    let (bplay, ctl_bplay) = Controlled::new(bplay);
    let b_idx = base.add_module("player".to_owned(), bplay);

    let b_util = Utility::new(b_idx);
    let (b_util, ctl_b_util) = Controlled::new(b_util);
    let bu_idx = base.add_module("player util".to_owned(), b_util);

    base.add_edge(b_idx, bu_idx);

    let mixer = Mixer::new(vec![u_idx, bu_idx]);
    let m_idx = base.add_module("mixer".to_owned(), mixer);
    base.add_edge(u_idx, m_idx);
    base.add_edge(bu_idx, m_idx);

    let amen = Arc::new(SampleBuffer::from_file("content/rock_scratch.wav"));

    ctl_bplay.send(Instant::now(), | p, _ | {
        p.set_buffer(amen);
        p.play();
    });

    ctl_b_util.send(Instant::now(), | u, _ | {
        u.set_volume(0.5);
    });

    ctl_util.send(Instant::now(), | u, _ | {
        u.set_volume(0.25);
    });
    */

    let a_graph = ModuleGraph::new(base, s_idx).unwrap();

    let audio_sys = System::new(a_graph);

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
