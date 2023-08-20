mod blocks;
mod engine;

use rand::Rng;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::{Arc, Mutex};

use engine::*;
use rhai::{Engine, RegisterNativeFunction};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let sample_rate = 48_000;

    let desired_spec = AudioSpecDesired {
        freq: Some(sample_rate),
        channels: Some(2),
        samples: Some(512),
    };

    let mut graph = Arc::new(Mutex::new(AudioGraph {
        output: Box::new(StereoOutput::new(vec![])),
        sample_rate: sample_rate as f32,
    }));
    let audio_device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| AudioGraphCallback {
            graph: graph.clone(),
        })
        .unwrap();

    audio_device.resume();
    {
        let mut graph = graph.lock().unwrap();
        //(*graph).add_sine(440.0, 0.0, 0.0);
        graph.add_sine(110.0, -30.0, -1.0);
        graph.add_sine(110.5, -30.0, 1.0);
    }

    //let mut rhai_interpreter = Engine::new();
    //run_repl(&mut rhai_interpreter, None).unwrap();

    // The sound plays in a separate thread. This sleep is only here so you can hear the sound before the program exits.
    std::thread::sleep(std::time::Duration::from_secs(4));
}
