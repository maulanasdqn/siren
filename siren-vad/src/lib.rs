use siren_domain::{AudioSamples, Utterance, VoiceActivityDetector};

pub struct EnergyVad {
    threshold: f32,
    max_silence_frames: usize,
    min_speech_frames: usize,
    frame_len: usize,
    speech: Vec<f32>,
    speech_frames: usize,
    silence_frames: usize,
    active: bool,
    pending: Vec<f32>,
}

impl EnergyVad {
    pub fn new() -> Self {
        let frame_len = (AudioSamples::SAMPLE_RATE as usize / 50).max(1);
        Self {
            threshold: 0.012,
            max_silence_frames: 35,
            min_speech_frames: 10,
            frame_len,
            speech: Vec::new(),
            speech_frames: 0,
            silence_frames: 0,
            active: false,
            pending: Vec::new(),
        }
    }

    fn process_frame(&mut self, frame: &[f32], out: &mut Vec<Utterance>) {
        let rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();
        if rms > self.threshold {
            self.active = true;
            self.speech_frames += 1;
            self.silence_frames = 0;
            self.speech.extend_from_slice(frame);
        } else if self.active {
            self.silence_frames += 1;
            self.speech.extend_from_slice(frame);
            if self.silence_frames >= self.max_silence_frames {
                self.finish(out);
            }
        }
    }

    fn finish(&mut self, out: &mut Vec<Utterance>) {
        if self.speech_frames >= self.min_speech_frames {
            out.push(Utterance::new(AudioSamples::new(std::mem::take(&mut self.speech))));
        }
        self.speech.clear();
        self.active = false;
        self.speech_frames = 0;
        self.silence_frames = 0;
    }
}

impl Default for EnergyVad {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceActivityDetector for EnergyVad {
    fn push(&mut self, samples: &[f32]) -> Vec<Utterance> {
        let mut out = Vec::new();
        self.pending.extend_from_slice(samples);
        while self.pending.len() >= self.frame_len {
            let frame: Vec<f32> = self.pending.drain(..self.frame_len).collect();
            self.process_frame(&frame, &mut out);
        }
        out
    }

    fn flush(&mut self) -> Option<Utterance> {
        if self.active && self.speech_frames >= self.min_speech_frames {
            let audio = AudioSamples::new(std::mem::take(&mut self.speech));
            self.active = false;
            self.speech_frames = 0;
            self.silence_frames = 0;
            return Some(Utterance::new(audio));
        }
        None
    }
}
