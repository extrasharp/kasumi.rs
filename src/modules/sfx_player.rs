use ika::Pool;

use crate::{
    system::{
        CALLBACK_BUFFER_LEN,
    },
    sample_buffer::*,
};

use super::{
    prelude::*,
    Utility,
    BufPlayer,
};

//

// buf player needs volume and pan ?

struct UtilPlay {
    bp: BufPlayer,
    u: Utility,
    buf: [Sample; CALLBACK_BUFFER_LEN],
}

impl Module for UtilPlay {
    fn compute<'a>(&mut self,
                   ctx: &CallbackContext,
                   _in_bufs: &[InputBuffer<'a>],
                   out_buf: &mut [Sample]) {
        let frame_len = out_buf.len();
        self.bp.compute(ctx, &[], &mut self.buf[0..frame_len]);
        self.u.compute(ctx, &[InputBuffer::new(0, &self.buf)], &mut out_buf[0..frame_len]);
    }
}

pub struct SfxPlayer {
    pool: Vec<UtilPlay>,
}

impl SfxPlayer {
    pub fn new(max: usize) -> Self {
        let pool = Vec::with_capacity(max);
        Self {
            pool,
        }
    }
}

impl Module for SfxPlayer {
    fn frame(&mut self, ctx: &CallbackContext) {
        /*
        self.pool.reclaim_unstable(| up | {
            up.bp.is_stopped()
        });
        */
    }

    fn compute<'a>(&mut self,
                   ctx: &CallbackContext,
                   _in_bufs: &[InputBuffer<'a>],
                   out_buf: &mut [Sample]) {
    }
}
