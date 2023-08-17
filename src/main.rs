mod blocks;
mod engine;

use rand::Rng;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use engine::*;

struct SineWave {
    frequency: f64,
    phase: f64,
    volume: f64,
    panning: f64,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let sample_rate = 48_000;

    let desired_spec = AudioSpecDesired {
        freq: Some(sample_rate),
        channels: Some(2),
        samples: Some(512),
    };

    let audio_device: AudioDevice<AudioGraphCallback> = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            let sample_rate = spec.freq as f32;

            let graph = StereoOutput::new(vec![]);

            let mut engine = AudioGraphCallback {
                graph: Mutex::new(graph),
                sample_rate,
            };
            engine.add_naive_sawtooth(440.0f32, 0.0, 0.0);
            engine
        })
        .unwrap();

    audio_device.resume();

    // The sound plays in a separate thread. This sleep is only here so you can hear the sound before the program exits.
    std::thread::sleep(std::time::Duration::from_secs(4));
}
