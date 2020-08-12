use std::{
    sync::Arc,
};

use crate::{
    sample_buffer::*,
    graph::GraphContext,
    Sample,
};

use super::Module;

//

pub struct BufPlayer {
    current_buffer: Option<Arc<SampleBuffer>>,
    frame_ct: f32,
    play_rate: f32,

    is_playing: bool,
    is_stopped: bool,
    do_loop: bool,
}

// TODO crossfading

impl BufPlayer {
    pub fn new() -> Self {
        Self {
            current_buffer: None,
            frame_ct: 0.,
            play_rate: 1.,

            is_playing: false,
            is_stopped: true,
            do_loop: true,
        }
    }

    pub fn play(&mut self) {
        self.is_stopped = false;
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn set_buffer(&mut self, buffer: Arc<SampleBuffer>) {
        self.current_buffer = Some(buffer);
        self.frame_ct = 0.;
    }

    pub fn set_play_rate(&mut self, play_rate: f32) {
        self.play_rate = play_rate;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn is_stopped(&self) -> bool {
        self.is_stopped
    }
}

impl Module for BufPlayer {
    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
        if self.is_stopped {
            for smp in out_buf.iter_mut() {
                *smp = 0.;
            }
            return;
        }

        let frame_size = out_buf.len();

        if let Some(buf) = &self.current_buffer {
            let mult = buf.sample_rate as f32 * self.play_rate / ctx.callback_context().sample_rate as f32;
            for i in 0..(frame_size / 2) {
                let data = &buf.data;
                let frame_at = self.frame_ct.trunc() as usize;
                if buf.channels == 1 {
                    out_buf[i * 2] = data[frame_at];
                    out_buf[i * 2 + 1] = data[frame_at];
                    self.frame_ct += mult;
                    if self.frame_ct >= data.len() as f32 {
                        self.frame_ct = 0.;
                        if !self.do_loop {
                            self.is_stopped = true;
                        }
                    }
                } else {
                    out_buf[i * 2] = data[frame_at * 2];
                    out_buf[i * 2 + 1] = data[frame_at * 2 + 1];
                    self.frame_ct += mult;
                    if self.frame_ct >= (data.len() / 2) as f32 {
                        self.frame_ct = 0.;
                        if !self.do_loop {
                            self.is_stopped = true;
                        }
                    }
                }
            }
        }
    }
}
