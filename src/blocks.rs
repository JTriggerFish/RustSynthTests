use crate::engine::{Block, Block1DRef, Block2DRef, DynBlock, Sample, SampleConstant, SampleVec2};
use nalgebra::{vector, SimdPartialOrd};
use std::f32::consts::PI;

pub struct Phasor<F: Block<SampleOutput = Sample>> {
    freq: F,
    phase: Sample,
    sample_freq: Sample,
}
impl<F: Block<SampleOutput = Sample>> Phasor<F> {
    pub fn new(freq: F, sample_freq: Sample) -> Phasor<F> {
        Phasor {
            freq,
            phase: 0.0,
            sample_freq,
        }
    }
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

pub struct SineOsc<P: Block<SampleOutput = Sample>, A: Block<SampleOutput = Sample>> {
    phase: P,
    amplitude: A,
}

impl<P: Block<SampleOutput = Sample>, A: Block<SampleOutput = Sample>> Block for SineOsc<P, A> {
    type SampleOutput = Sample;

    fn process(&mut self) -> Sample {
        let p = self.phase.process();
        let a = self.amplitude.process();
        return a * p.sin();
    }
}
impl<P: Block<SampleOutput = Sample>, A: Block<SampleOutput = Sample>> SineOsc<P, A> {
    pub fn new(phase: P, amplitude: A) -> SineOsc<P, A> {
        SineOsc { phase, amplitude }
    }
    pub fn new_fixed(
        frequency: Sample,
        amplitude: Sample,
        sample_freq: Sample,
    ) -> SineOsc<Phasor<SampleConstant>, SampleConstant> {
        let phase = Phasor::new(SampleConstant { value: frequency }, sample_freq);
        SineOsc {
            phase,
            amplitude: SampleConstant { value: amplitude },
        }
    }
}

pub struct MonoToStereoMix<
    I: Block<SampleOutput = Sample>,
    A: Block<SampleOutput = Sample>,
    P: Block<SampleOutput = Sample>,
> {
    input: I,
    amplitude_db: A,
    panning: P,
}

impl<
        I: Block<SampleOutput = Sample>,
        A: Block<SampleOutput = Sample>,
        P: Block<SampleOutput = Sample>,
    > Block for MonoToStereoMix<I, A, P>
{
    type SampleOutput = SampleVec2;
    fn process(&mut self) -> SampleVec2 {
        let x = self.input.process();
        let a = 10f32.powf(self.amplitude_db.process() / 20.0);
        let p = self.panning.process();
        let left = ((1.0 - p) / 2.0).sqrt() * x * a;
        let right = ((1.0 + p) / 2.0).sqrt() * x * a;
        let ret = vector![left, right];
        ret
    }
}

impl<
        I: Block<SampleOutput = Sample>,
        A: Block<SampleOutput = Sample>,
        P: Block<SampleOutput = Sample>,
    > MonoToStereoMix<I, A, P>
{
    pub fn new(input: I, amplitude_db: A, panning: P) -> MonoToStereoMix<I, A, P> {
        MonoToStereoMix {
            input,
            amplitude_db,
            panning,
        }
    }
    pub fn new_fixed(
        input: Block1DRef,
        amplitude_db: Sample,
        panning: Sample,
    ) -> MonoToStereoMix<DynBlock<Sample>, SampleConstant, SampleConstant> {
        MonoToStereoMix {
            input: DynBlock { block: input },
            amplitude_db: SampleConstant {
                value: amplitude_db,
            },
            panning: SampleConstant { value: panning },
        }
    }
}
