use crate::error::Error;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::f32::consts::TAU;

pub type Result<T> = std::result::Result<T, Error>;

const NOTE_FREQ: f32 = 349.23; // G4
const BASE_VOLUME: f32 = 0.1;

pub struct Audio {
    device: AudioDevice<Wave>,
}

impl Audio {
    pub fn new(context: &sdl2::Sdl, volume: f32) -> Result<Self> {
        let device = build_audio_device(context, volume)?;
        Ok(Self { device })
    }

    pub fn set_status(&mut self, is_playing: bool) {
        if is_playing {
            self.device.resume();
        } else {
            self.device.pause();
        }
    }
}

struct Wave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for Wave {
    type Channel = f32;

    fn callback(&mut self, output: &mut [Self::Channel]) {
        // sine wave
        for x in output.iter_mut() {
            *x = (self.phase * TAU).sin() * self.volume;
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn build_audio_device(context: &sdl2::Sdl, volume: f32) -> Result<AudioDevice<Wave>> {
    let audio_subsystem = context.audio().map_err(to_sdl_err)?;
    let spec = AudioSpecDesired {
        freq: None,
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &spec, |spec| Wave {
            phase_inc: NOTE_FREQ / spec.freq as f32,
            phase: 0.0,
            volume: BASE_VOLUME * volume,
        })
        .map_err(|_| Error::SystemError("Error initilizating audio".to_string()))?;

    Ok(device)
}

fn to_sdl_err(err: String) -> Error {
    Error::SystemError(err)
}
