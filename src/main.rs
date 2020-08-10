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
    traits::*,
};

use kasumi::{
    modules::*,
};

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

    let (util, tx) = Utility::new();
    let sine = Sine::new();
    let chain = Chain::new(vec![
        Box::new(sine),
        Box::new(util),
    ]);
    let (mut mixer, mctl) = Mixer::new(vec![
        Box::new(chain),
    ]);

    let mut ac = AudioContext::new();

    let stream = device.build_output_stream(
        &config,
        move | data: &mut [f32], _ | {
            ac.now = Instant::now();

            mixer.frame(&ac);

            let dummy = [0.; 2];
            for i in 0..(data.len()/2) {
                let mut buf = [0.; 2];
                mixer.compute(&ac, &dummy, &mut buf);
                data[i * 2] = buf[0];
                data[i * 2 + 1] = buf[1];
            }
        },
        move | err | {
        },
    );

    tx.send(Instant::now(), UtilityEvent::Volume(0.5));

    loop {
        thread::sleep(Duration::from_secs(1));
        tx.send(Instant::now() + Duration::from_millis(500), UtilityEvent::Pan(1.));
        thread::sleep(Duration::from_secs(1));
        tx.send(Instant::now(), UtilityEvent::Pan(-1.));
    }
}
