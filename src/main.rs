use rand::Rng;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

struct SineWave {
    frequency: f64,
    phase: f64,
    volume: f64,
    panning: f64,
}

struct SineWaveAudioCallback {
    sines: Mutex<Vec<SineWave>>,
    sample_rate: Arc<f64>,
}

impl SineWaveAudioCallback {
    fn process_wave(&self, wave: &mut SineWave) -> (f32, f32) {
        let c = 2.0 * PI * wave.frequency / *self.sample_rate;
        let output: f64 = wave.volume * wave.phase.sin();

        let sample_left: f32 = (((1.0 - wave.panning) / 2.0).sqrt() * output) as f32;
        let sample_right: f32 = (((1.0 + wave.panning) / 2.0).sqrt() * output) as f32;

        wave.phase = (wave.phase + c) % (2.0 * PI);

        (sample_left, sample_right)
    }
    fn add_sine(&self, frequency: f64, volume_db: f64, panning: f64) {
        let volume = 10f64.powf(volume_db / 20.0);
        let mut sines_guard = self.sines.lock().unwrap();
        sines_guard.push(SineWave {
            frequency,
            phase: 0.0,
            volume,
            panning,
        });
    }
    fn add_naive_sawtooth(&self, frequency: f64, volume_db: f64, panning: f64) {
        let volume = 10f64.powf(volume_db / 20.0);
        let mut n = 1;
        let mut sines_guard = self.sines.lock().unwrap();

        while (frequency * n as f64) < { 44100.0 / 2.0 } {
            let harmonic_volume = volume / n as f64;
            sines_guard.push(SineWave {
                frequency: frequency * n as f64,
                phase: 0.0,
                volume: harmonic_volume,
                panning,
            });
            n += 1;
        }
    }

    fn supersaw(&self, center_frequency: f64, variance_hz: f64, num_saws: usize, volume_db: f64) {
        let saw_volume = volume_db - 20.0 * (num_saws as f64).sqrt().log10();
        let mut rng = rand::thread_rng();
        let mut sines_guard = self.sines.lock().unwrap();

        for _ in 0..num_saws {
            let frequency = center_frequency + (rng.gen::<f64>() - 0.5) * 2.0 * variance_hz;
            let saw_panning = 2.0 * rng.gen::<f64>() - 1.0;
            drop(sines_guard); // Drop the lock so it can be reacquired in add_naive_sawtooth
            self.add_naive_sawtooth(frequency, saw_volume, saw_panning);
            sines_guard = self.sines.lock().unwrap(); // Reacquire the lock
        }
        println!("total number of sines: {}", sines_guard.len());
    }
}

impl AudioCallback for SineWaveAudioCallback {
    type Channel = f32;

    fn callback(&mut self, output_buffer: &mut [f32]) {
        for (_i, output) in output_buffer.chunks_exact_mut(2).enumerate() {
            let mut sample_left: f32 = 0.0;
            let mut sample_right: f32 = 0.0;

            let mut sines_guard = self.sines.lock().unwrap();

            for wave in sines_guard.iter_mut() {
                let (left, right) = self.process_wave(wave);
                sample_left += left;
                sample_right += right;
            }

            output[0] = sample_left;
            output[1] = sample_right;
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(48000),
        channels: Some(2),
        samples: Some(512),
    };

    let sample_rate = Arc::new(44100.0);

    let sines = Mutex::new(vec![]);

    let audio_callback = SineWaveAudioCallback {
        sines,
        sample_rate: Arc::clone(&sample_rate),
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
