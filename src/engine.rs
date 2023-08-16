use nalgebra::SVector;
use std::boxed::Box;

pub type Sample = f32;
pub type SampleVec2 = SVector<Sample, 2>;
pub type SampleVec4 = SVector<Sample, 4>;
pub type SampleVec8 = SVector<Sample, 8>;

pub trait Block {
    type SampleOutput;

    fn process(&mut self) -> Self::SampleOutput;
}
pub type RefSampleBlock = Box<dyn Block<SampleOutput = Sample>>;
pub type RefSampleBlock2 = Box<dyn Block<SampleOutput = SampleVec2>>;
pub type RefSampleBlock4 = Box<dyn Block<SampleOutput = SampleVec4>>;
pub type RefSampleBlock8 = Box<dyn Block<SampleOutput = SampleVec8>>;

pub struct SampleConstant {
    value: Sample,
}

impl Block for SampleConstant {
    type SampleOutput = Sample;
    fn process(&mut self) -> Sample {
        return self.value;
    }
}
//struct SineWaveAudioCallback {
//    sines: Mutex<Vec<SineWave>>,
//    sample_rate: Sample;
//}
