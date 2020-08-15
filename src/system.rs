use std::{
    time::Instant,
};

use cpal::{
    self,
    SampleRate,
    traits::*,
};

use crate::{
    graph::ControlledModGraph,
};

//

pub struct CallbackContext {
    pub sample_rate: u32,
    pub now: Instant,
}

impl CallbackContext {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            now: Instant::now(),
        }
    }
}

//

pub const CALLBACK_BUFFER_LEN: usize = 2048;

//

pub struct System {
    _stream: cpal::Stream,
}

impl System {
    pub fn new(mut graph: ControlledModGraph) -> Self {
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

        let mut ac = CallbackContext::new();

        let stream = device.build_output_stream(
            &config,
            move | data: &mut [f32], _ | {
                ac.now = Instant::now();

                graph.frame(&ac);

                for chunk in data.chunks_mut(CALLBACK_BUFFER_LEN) {
                    graph.compute(&ac, chunk);
                }
            },
            move | err | {
            },
        ).unwrap();

        Self {
            _stream: stream,
        }
    }
}
