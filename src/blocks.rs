use crate::engine::{Block, Sample};
use std::f32::consts::PI;

struct Phasor<F: Block<SampleOutput = Sample>> {
    freq: F,
    phase: Sample,
    sample_freq: Sample,
}

impl<F: Block<SampleOutput = Sample>> Block for Phasor<F> {
    type SampleOutput = Sample;
    fn process(&mut self) -> Sample {
        let f = self.freq.process();
        let c = 2.0 * PI * f / self.sample_freq;
        let ret = self.phase;
        self.phase = (self.phase + c) % (2.0 * PI);
        ret
    }
}
