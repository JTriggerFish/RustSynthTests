use crate::blocks::*;
use nalgebra::SVector;
use nalgebra::{vector, SimdPartialOrd};
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
pub type RefSampleBlock = Box<dyn Block<SampleOutput = Sample> + Send>;
pub type RefSampleBlock2 = Box<dyn Block<SampleOutput = SampleVec2> + Send>;
pub type RefSampleBlock4 = Box<dyn Block<SampleOutput = SampleVec4> + Send>;
pub type RefSampleBlock8 = Box<dyn Block<SampleOutput = SampleVec8> + Send>;

pub struct SampleConstant {
    value: Sample,
}

impl Block for SampleConstant {
    type SampleOutput = Sample;
    fn process(&mut self) -> Sample {
        return self.value;
    }
}

pub struct StereoOutput {
    pub blocks: Vec<RefSampleBlock2>,
    output: SampleVec2,
}

impl StereoOutput {
    pub fn new(blocks: Vec<RefSampleBlock2>) -> StereoOutput {
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
pub struct AudioGraphCallback {
    graph: Mutex<StereoOutput>,
    sample_rate: f32,
}

impl AudioCallback for AudioGraphCallback {
    type Channel = f32;

    fn callback(&mut self, output_buffer: &mut [f32]) {
        for (_i, output) in output_buffer.chunks_exact_mut(2).enumerate() {
            let mut sample_left: f32 = 0.0;
            let mut sample_right: f32 = 0.0;

            let mut graph_guard = self.graph.lock().unwrap();
            let result = graph_guard.process();
            output[0] = result[0];
            output[1] = result[1];
        }
    }
}
impl AudioGraphCallback {
    fn add_sine(
        &mut self,
        frequency: f32,
        amplitude_dB: f32,
        panning: f32,
        sample_rate: Option<f32>,
    ) {
        let mut graph_guard = self.graph.lock().unwrap();
        // Create your sine generator block here, assuming you have a block named `SineOscillator`
        // and it can be constructed with a given frequency.
        let sine_block = Box::new(SineOsc::new(frequency, 1.0, sample_rate));
        let mono_to_stereo_block =
            Box::new(MonoToStereoMix::new(sine_block, amplitude_dB, panning));
        graph_guard.blocks.push(mono_to_stereo_block);
    }
    fn add_naive_sawtooth(&mut self, frequency: f32, volume_db: f32, panning: f32) {
        let volume = 10f32.powf(volume_db / 20f32);
        let mut n = 1;

        while (frequency * n as f32) < { self.sample_rate / 2.0 } {
            let harmonic_volume: f32 = volume as f32 / n as f32;
            let amplitude_db: f32 = 20.0 * (harmonic_volume).log10();
            self.add_sine(frequency * n as f32, amplitude_db, panning, None);
            n += 1;
        }
    }

    fn supersaw(
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
        //println!("total number of sines: {}", sines_guard.len());
    }
}
