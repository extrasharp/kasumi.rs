use ika::Pool;

use crate::{
    system::{
        CALLBACK_BUFFER_LEN,
    },
    sample_buffer::*,
    graph::GraphContext,
    Sample,
};

use super::{
    Module,
    InputBuffer,
    BufPlayer,
    Utility,
};

//

// buf player needs volume and pan ?

struct UtilPlay {
    bp: BufPlayer,
    u: Utility,
    buf: [Sample; CALLBACK_BUFFER_LEN],
}

impl Module for UtilPlay {
    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
        self.bp.compute(ctx, &mut self.buf);
        self.u.compute2(ctx, &[InputBuffer{id:0, buf:&self.buf}], out_buf);
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
    fn frame(&mut self, ctx: &GraphContext) {
        /*
        self.pool.reclaim_unstable(| up | {
            up.bp.is_stopped()
        });
        */
    }

    fn compute(&mut self, ctx: &GraphContext, out_buf: &mut [Sample]) {
    }
}
