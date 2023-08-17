mod blocks;
mod engine;

use rand::Rng;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
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


    let blocks = Mutex::new(vec![]);

    let audio_callback = AudioCallback::new(move |audio| {
        blocks,
        sample_rate: sample_rate as f32,
    };
    audio_callback.add_sine(110.0, -30.0, -0.99);
    audio_callback.add_sine(110.5, -30.0, 0.99);
    audio_callback.supersaw(110.0, 1.0, 7, -30.0);

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_| audio_callback)
        .unwrap();

    device.resume();

    // The sound plays in a separate thread. This sleep is only here so you can hear the sound before the program exits.
    std::thread::sleep(std::time::Duration::from_secs(4));
}
