use crate::error::Error;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub type Result<T> = std::result::Result<T, Error>;

pub fn build_audio_device(context: &sdl2::Sdl) -> Result<AudioDevice<Wave>> {
    let audio_subsystem = context.audio().map_err(to_sdl_err)?;
    let spec = AudioSpecDesired {
        freq: None,
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &spec, |spec| Wave {
            phase_inc: 392.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.08,
        })
        .map_err(|_| Error::SystemError("Error initilizating audio".to_string()))?;

    Ok(device)
}

pub struct Audio {
    device: AudioDevice<Wave>,
}

impl Audio {
    pub fn new(context: &sdl2::Sdl) -> Result<Self> {
        let device = build_audio_device(context)?;
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

pub struct Wave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for Wave {
    type Channel = f32;

    fn callback(&mut self, output: &mut [Self::Channel]) {
        // square wave
        for x in output.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };

            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn to_sdl_err(err: String) -> Error {
    Error::SystemError(err)
}
