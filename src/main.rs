fn main() {
}

/*
use jack;
use jack::Client;

use std::time;
use std::sync::{
    mpsc,
    Arc,
    Mutex,
};
use flume;

struct Chain {
    modules: Vec<Box<dyn Module>>,
    buf: [f32; 2],
}

impl Chain {
    fn compute(&mut self) {
        let mut in_buf = [0_f32; 2];
        let mut out_buf = [0_f32; 2];
        for m in self.modules.iter_mut() {
            m.compute(&in_buf, &mut out_buf);
            in_buf = out_buf;
        }
        self.buf = out_buf;
    }
}

trait Module: Send + Sync {
    // Takes an AudioContext that has sample rate and stuff
    fn compute(&mut self, in_buf: &[f32; 2], out_buf: &mut [f32; 2]);
    fn handle_event(&self) {}
}

struct Sine {
    frame_ct: f32,
}

impl Module for Sine {
    fn compute(&mut self, _in_buf: &[f32; 2], out_buf: &mut [f32; 2]) {
        out_buf[0] = f32::sin(self.frame_ct / 10_f32);
        out_buf[1] = f32::sin(self.frame_ct / 10_f32);
        self.frame_ct += 1.;
    }
}

struct Distort;

impl Module for Distort {
    fn compute(&mut self, in_buf: &[f32; 2], out_buf: &mut [f32; 2]) {
        fn dist(f: f32) -> f32 {
            if f > 0_f32 {
                1_f32
            } else {
                -1_f32
            }
        }

        out_buf[0] = dist(in_buf[0]);
        out_buf[1] = dist(in_buf[1]);
    }
}

fn main() {
    let c_res = Client::new("kasumi", jack::ClientOptions::NO_START_SERVER);

    let (client, status) = match c_res {
        Ok(res) => res,
        Err(e) => panic!("Failed to open client because of error: {:?}", e),
    };

    let mut out_port = client.register_port("kasumi_out", jack::AudioOut::default())
        .unwrap();

    let mut ch = Chain {
        modules: vec![Box::new(Sine { frame_ct: 0_f32})],
        buf: [0f32; 2],
    };

    let (tx, rx) = flume::unbounded::<f64>();
    let (tx2, rx2) = flume::unbounded::<f64>();
    // let (tx, rx) = mpsc::channel::<f64>();
    // let (tx2, rx2) = mpsc::channel::<f64>();
    let am_tx = Arc::new(Mutex::new(tx));
    let am_tx2 = Arc::new(Mutex::new(tx2));
    let tm = Arc::new(time::Instant::now());

    let tm_c = Arc::clone(&tm);
    let cb = move |_: &Client, ps: &jack::ProcessScope | -> jack::Control {
        let out = out_port.as_mut_slice(ps);
        let time = tm_c.elapsed().as_secs_f64();
        am_tx.lock().unwrap().send(time);
        let gua = am_tx2.lock().unwrap();
        for _ in 0..1000 {
            gua.send(time);
        }
        for v in out.iter_mut() {
            ch.compute();
            *v = ch.buf[0];
        }
        jack::Control::Continue
    };

    let proc = jack::ClosureProcessHandler::new(cb);

    let active_client = client.activate_async((), proc).unwrap();

    let mut last = 0.;
    let mut max = -1.;
    let mut ct = 0;
    loop {
        let now = rx.recv().unwrap();
        max = f64::max(max, now - last);
        println!("{:1.8} {:1.8}", now - last, max);
        last = now;
        ct += 1;
        if ct > 20 {
            ct = 0;
            max = -1.;
        }
    }
}
*/
