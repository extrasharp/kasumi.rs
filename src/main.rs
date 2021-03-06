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
    let (graph, mut graph_ctl) = ControlledGraph::new();
    let sine = graph_ctl.add_module(Sine::new());

    let util = Utility::new();
    let (util, ctl_util) = Controlled::new(util);
    let util = graph_ctl.add_module(util);
    graph_ctl.add_edge(sine, util, 0);

    graph_ctl.set_as_output(Some(util));
    graph_ctl.push_changes(Instant::now());

    /*
    let mut base = ModuleGraphBase::new();

    let sine = Sine::new();
    let (sine, ctl_sine) = Controlled::new(sine);
    let s_idx = base.add_module(sine);

    let util = Utility::new();
    let (util, ctl_util) = Controlled::new(util);
    let u_idx = base.add_module(util);

    base.add_edge(s_idx, u_idx, 0);

    let bplay = BufPlayer::new();
    let (bplay, ctl_bplay) = Controlled::new(bplay);
    let b_idx = base.add_module(bplay);

    let b_util = Utility::new();
    let (b_util, ctl_b_util) = Controlled::new(b_util);
    let bu_idx = base.add_module(b_util);

    base.add_edge(b_idx, bu_idx, 0);

    let mixer = Mixer::new();
    let m_idx = base.add_module(mixer);
    base.add_edge(u_idx, m_idx, 0);
    base.add_edge(bu_idx, m_idx, 1);

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

    let a_graph = ModuleGraph::new(base, m_idx).unwrap();
    */

    let audio_sys = System::new(graph);

    let start = Instant::now();
    let mut del_first = false;
    let mut made_new = false;

    loop {
        graph_ctl.frame();

        thread::sleep(Duration::from_secs(1));
        ctl_util.send(Instant::now(), | u, _ | {
            u.set_volume(0.5);
        });
        thread::sleep(Duration::from_secs(1));
        ctl_util.send(Instant::now(), | u, _ | {
            u.set_volume(0.25);
        });

        let tm = start.elapsed().as_secs_f32();
        if tm > 5. && !del_first {
            println!("del");
            graph_ctl.remove_module(sine);
            graph_ctl.push_changes(Instant::now());
            del_first = true;
        } else if tm > 10. && !made_new {
            println!("new");
            let (s, s_ctl) = Controlled::new(Sine::new());
            s_ctl.send(Instant::now(), | s, _ | {
                s.set_frequency(550.);
            });
            let s = graph_ctl.add_module(s);
            graph_ctl.add_edge(s, util, 0);
            graph_ctl.push_changes(Instant::now());
            made_new = true;
        }

        /*
        ctl_sine.send(Instant::now(), | s, _ | {
            s.set_frequency(880.);
        });
        ctl_sine.send(Instant::now(), | s, _ | {
            s.set_frequency(440.);
        });
        */
    }
}
