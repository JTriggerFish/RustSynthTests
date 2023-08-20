use crate::blocks::*;
use nalgebra::SVector;
use nalgebra::{vector, SimdPartialOrd};
use optargs::optfn;
use rand::seq::index::sample;
use rand::Rng;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::boxed::Box;
use std::sync::{Arc, Mutex};

pub type Sample = f32;
pub type SampleVec2 = SVector<Sample, 2>;
pub type SampleVec4 = SVector<Sample, 4>;
pub type SampleVec8 = SVector<Sample, 8>;

pub trait Block {
    type SampleOutput;

    fn process(&mut self) -> Self::SampleOutput;
}
pub type Block1DRef = Box<dyn Block<SampleOutput = Sample> + Send>;
pub type Block2DRef = Box<dyn Block<SampleOutput = SampleVec2> + Send>;
pub type Block4DRef = Box<dyn Block<SampleOutput = SampleVec4> + Send>;
pub type Block8DRef = Box<dyn Block<SampleOutput = SampleVec8> + Send>;

pub struct SampleConstant {
    pub value: Sample,
}

impl Block for SampleConstant {
    type SampleOutput = Sample;
    fn process(&mut self) -> Sample {
        return self.value;
    }
}

pub struct DynBlock<T> {
    pub block: Box<dyn Block<SampleOutput = T> + Send>,
}

impl<T> Block for DynBlock<T> {
    type SampleOutput = T;
    fn process(&mut self) -> T {
        return self.block.process();
    }
}

pub struct StereoOutput {
    pub blocks: Vec<Block2DRef>,
    output: SampleVec2,
}

impl StereoOutput {
    pub fn new(blocks: Vec<Block2DRef>) -> StereoOutput {
        StereoOutput {
            blocks,
            output: SampleVec2::new(0.0, 0.0),
        }
    }
}

impl Block for StereoOutput {
    type SampleOutput = SampleVec2;
    fn process(&mut self) -> SampleVec2 {
        self.output.fill(0.0);
        for block in &mut self.blocks {
            let result = block.process();
            self.output += result;
            // Here we're just accumulating in the left channel for simplicity.
            // For a true stereo mix, you would modify this.
        }
        self.output
            .simd_clamp(vector![-1.0, 1.0], vector![1.0, 1.0]);
        self.output
    }
}
pub struct AudioGraph {
    pub output: Box<StereoOutput>,
    pub sample_rate: f32,
}
pub struct AudioGraphCallback {
    pub graph: Arc<Mutex<AudioGraph>>,
}

impl AudioCallback for AudioGraphCallback {
    type Channel = f32;

    fn callback(&mut self, output_buffer: &mut [f32]) {
        for (_i, output) in output_buffer.chunks_exact_mut(2).enumerate() {
            let mut graph_guard = self.graph.lock().unwrap();
            let result = graph_guard.output.process();
            output[0] = result[0];
            output[1] = result[1];
        }
    }
}
impl AudioGraph {
    pub fn add_sine(&mut self, frequency: f32, amplitude_db: f32, panning: f32) {
        // Create your sine generator block here, assuming you have a block named `SineOscillator`
        // and it can be constructed with a given frequency.
        let sine_block: Block1DRef = Box::new(
            SineOsc::<Phasor<SampleConstant>, SampleConstant>::new_fixed(
                frequency,
                1.0f32,
                self.sample_rate,
            ),
        );
        let mono_to_stereo_block = Box::new(MonoToStereoMix::<
            DynBlock<Sample>,
            SampleConstant,
            SampleConstant,
        >::new_fixed(sine_block, amplitude_db, panning));
        self.output.blocks.push(mono_to_stereo_block);
    }
    pub fn add_naive_sawtooth(&mut self, frequency: f32, volume_db: f32, panning: f32) {
        let volume = 10f32.powf(volume_db / 20f32);
        let mut n = 1;

        while (frequency * n as f32) < { self.sample_rate / 2.0 } {
            let harmonic_volume: f32 = volume / n as f32;
            let amplitude_db: f32 = 20.0 * (harmonic_volume).log10();
            self.add_sine(frequency * n as f32, amplitude_db, panning);
            n += 1;
        }
    }

    pub fn add_supersaw(
        &mut self,
        center_frequency: f32,
        variance_hz: f32,
        num_saws: usize,
        volume_db: f32,
    ) {
        let saw_volume = volume_db - 20.0 * (num_saws as f32).sqrt().log10();
        let mut rng = rand::thread_rng();

        for _ in 0..num_saws {
            let frequency = center_frequency + (rng.gen::<f32>() - 0.5) * 2.0 * variance_hz;
            let saw_panning = 2.0 * rng.gen::<f32>() - 1.0;
            self.add_naive_sawtooth(frequency, saw_volume, saw_panning);
        }
        println!("total number of sines: {}", self.output.blocks.len());
    }
}
