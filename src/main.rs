use std::{
    thread,
    time::{
        Duration,
        Instant,
    },
};

use cpal::{
    self,
    Data,
    SampleRate,
    BufferSize,
    traits::*,
};
use petgraph::Graph;

use kasumi::{
    AudioContext,
    CallbackBuffer,
    CALLBACK_BUFFER_LEN,
    modules::*,
    sample_buffer::*,
    event::*,
    audio_graph::*,
};

//

trait Wowo {
    fn wow(&self) {
        println!("wow");
    }
}

impl Wowo for u8 {
}

struct SpecialBox<T: Wowo + ?Sized> {
    bx: Box<T>,
}

struct Blah {
    vec: Vec<Box<dyn Wowo>>,
}

impl Blah {
    fn add<T: Wowo + 'static>(&mut self, mut bx: Vec<Box<T>>) {
        self.vec.push(bx.drain(..).next().unwrap());
    }

    /*
    fn add2(&mut self, mut bx: Vec<Box<dyn Wowo>>) {
        self.vec.push(bx.drain(..).next().unwrap());
    }
    */
}

fn main() {
    /*
    let mut blah = Blah { vec: vec![] };
    let k = vec![Box::new(5_u8)];
    blah.add(k);
    // let j = vec![Box::new(5_u8)];
    // blah.add2(j);

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

    let sine = Sine::new();
    let (tx, rx) = event_channel::<Box<dyn FnOnce(&mut Sine) + Send>>(50);
    // let gmod = GraphModule::new(Box::new(sine), Some(rx));

    let mut graph = Graph::<Box<dyn Module>, ()>::new();
    graph.add_node(Box::new(sine));
    // graph.add_node(gmod);
    // graph.add_node(GraphModule::new(Box::new(sine), Some(rx)));

    let mut a_graph = AudioGraph::new(graph).unwrap();
    // a_graph.add_node(gmod);

    let mut ac = AudioContext::new();
    let mut callback_buf = CallbackBuffer::new();

    let stream = device.build_output_stream(
        &config,
        move | data: &mut [f32], _ | {
            ac.now = Instant::now();

            // a_graph.recv(ac.now);

            // mixer.frame(&ac);

            for chunk in data.chunks_mut(CALLBACK_BUFFER_LEN) {
                callback_buf.fill_in_buf_f32(chunk);
                let (in_buf, out_buf) = callback_buf.buffers();
                a_graph.compute(&ac, out_buf);
                // mixer.compute(&ac, in_buf, out_buf);
                callback_buf.take_out_buf_f32(chunk);
            }
        },
        move | err | {
        },
    );

    loop {
        thread::sleep(Duration::from_secs(1));
        tx.send(Instant::now(), Box::new(| s | {
            s.set_frequency(880.);
        }));
        /*
        thread::sleep(Duration::from_secs(1));
        tx.send(Instant::now(), | s | {
            s.set_frequency(440.);
        });
        */
    }
*/
}

/*
fn main() {
    println!("acell fl {}", AtomicCell::<f32>::is_lock_free());

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

    /*
    let amen = SampleBuffer::from_file("content/rock_scratch.wav");
    let (bplay, bpctl) = BufPlayer::new();
    bpctl.set_buffer(Instant::now(), std::sync::Arc::new(amen));
    bpctl.set_play_rate(Instant::now(), 1.75);
    bpctl.play(Instant::now());

    // let (sine2, s2ctl) = Sine2::new();
    let (mut csine2, csine2ctl) = make_controlled(Sine2::new());

    let (sine, sctl) = Sine::new();
    let (util, tx) = Utility::new();
    let chain = Chain::new(vec![
        // Box::new(sine),
        // Box::new(bplay),
        // Box::new(sine2),
        Box::new(util),
    ]);
    let (mut mixer, mctl) = Mixer::new(vec![
        Box::new(chain),
    ]);
    */

    let mut ac = AudioContext::new();
    let mut callback_buf = CallbackBuffer::new();

    let stream = device.build_output_stream(
        &config,
        move | data: &mut [f32], _ | {
            ac.now = Instant::now();

            mixer.frame(&ac);

            for chunk in data.chunks_mut(CALLBACK_BUFFER_LEN) {
                callback_buf.fill_in_buf_f32(chunk);
                let (in_buf, out_buf) = callback_buf.buffers();
                mixer.compute(&ac, in_buf, out_buf);
                callback_buf.take_out_buf_f32(chunk);
            }
        },
        move | err | {
        },
    );

    tx.send(Instant::now(), UtilityEvent::Volume(0.5));

    loop {
        thread::sleep(Duration::from_secs(1));
        csine2ctl.send(Instant::now(), | s2 | {
            s2.set_frequency(880.);
        });
        thread::sleep(Duration::from_secs(1));
        csine2ctl.send(Instant::now(), | s2 | {
            s2.set_frequency(440.);
        });
        /*
        thread::sleep(Duration::from_secs(1));
        // s2ctl.do_action(Instant::now(), | s2 | s2.set_frequency(880.));
        // tx.send(Instant::now() + Duration::from_millis(500), UtilityEvent::Pan(1.));
        // sctl.set_frequency(Instant::now() + Duration::from_millis(250), 880.);
        thread::sleep(Duration::from_secs(1));
        // s2ctl.do_action(Instant::now(), | s2 | s2.set_frequency(440.));
        // sctl.set_frequency(Instant::now() + Duration::from_millis(250), 440.);
        // tx.send(Instant::now(), UtilityEvent::Pan(-1.));
        */
    }
}
*/
