use std::{
    sync::Arc,
    time::{
        Instant,
    },
};

use crate::{
    event::*,
    sample_buffer::*,
    AudioContext,
    Sample,
};

use super::{
    Module,

    Mixer,
    BufPlayer,
};

//

pub struct SfxPlayer {
    mixer: Mixer,
    buf_players: Vec<BufPlayer>,
}
