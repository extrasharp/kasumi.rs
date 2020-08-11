use std::{
    sync::Arc,
    path::Path,
};
use crate::{
    Sample,
};

pub struct SampleBuffer {
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<Sample>,
}

impl SampleBuffer {
    pub fn from_file<F: AsRef<Path>>(filepath: F) -> Self {
        use hound;

        // TODO error
        let wav_reader = hound::WavReader::open(filepath).unwrap();
        let spec = wav_reader.spec();

        let sample_rate = spec.sample_rate;
        // TODO panic if not 1 or 2 channels
        let channels = spec.channels;

        let data = match spec.sample_format {
            hound::SampleFormat::Float => {
                let data: Result<Vec<_>, _> = wav_reader.into_samples::<f32>().collect();
                data.unwrap()
            }
            hound::SampleFormat::Int => {
                let data: Result<Vec<_>, _> = match spec.bits_per_sample {
                    8  => wav_reader.into_samples::<i8>()
                                    .map(| s | s.map(| s | s as f32 /  i8::MAX as f32))
                                    .collect(),
                    16 => wav_reader.into_samples::<i16>()
                                    .map(| s | s.map(| s | s as f32 / i16::MAX as f32))
                                    .collect(),
                    32 => wav_reader.into_samples::<i32>()
                                    .map(| s | s.map(| s | s as f32 / i32::MAX as f32))
                                    .collect(),
                    _ => panic!(),
                };
                data.unwrap()
            }
        };

        Self {
            sample_rate,
            channels,
            data,
        }
    }
}

//

pub struct SampleBank {
    buffers: Vec<Arc<SampleBuffer>>,
}

impl SampleBank {
    pub fn new(buffers: Vec<SampleBuffer>) -> Self {
        Self {
            buffers: buffers.into_iter().map(| sb | Arc::new(sb)).collect(),
        }
    }
}
