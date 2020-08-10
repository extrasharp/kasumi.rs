

type Sample = f32;

struct Event {
    timestamp: Instant,
    id: usize,
    // path: Vec<String> a path
}

struct AudioContext {
    sample_rate: u32,
    mixer: Mixer,
}

trait Module: Send + Sync {
    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]);
    fn handle_event(&mut self, _ctx: &AudioContext, _ev: &Event) {}
}

//

struct Mixer {
    tracks: Vec<Track>,
    chains: Vec<ChainSend>,
}

struct Track {
    volume: f32,
    pan: f32,
}

struct ChainSend {
    track_id: usize,
    chain: Chain,
}

//

struct Chain {
    modules: Vec<Box<dyn Module>>,
}

impl Module for Chain {
    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]) {
        let mut local_in_buf = *in_buf;
        let mut local_out_buf = [0_f32; 2];
        for m in self.modules.iter_mut() {
            m.compute(ctx, &local_in_buf, &mut local_out_buf);
            local_in_buf = local_out_buf;
        }
        *out_buf = local_out_buf;
    }

    /*
    fn handle_event(&mut self, ctx: &AudioContext, ev: &Event) {
        for m in self.modules.iter_mut() {
            m.handle_event(ctx, ev);
        }
    }
    */
}

//

// instrument is shared by main thread and audio thread
struct InstrumentModule {
    inst: Arc<Mutex<Instrument>>,
}

impl Module for InstrumentModule {
    fn compute(&mut self,
               ctx: &AudioContext,
               in_buf: &[Sample; 2],
               out_buf: &mut [Sample; 2]) {
    }
}

struct Instrument {
    bank: SampleBank,
}

//

struct SampleBuffer {
    sample_rate: u32,
    // 1 or 2
    channel_count: u8,
    // interleaved
    samples: Vec<Sample>,
}

struct SampleBank {
    bank: Vec<SampleBuffer>,
}

//

struct FileStream {
}

// inst.add_sample()
