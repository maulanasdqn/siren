#[derive(Clone, Debug)]
pub struct AudioSamples {
    samples: Vec<f32>,
}

impl AudioSamples {
    pub const SAMPLE_RATE: u32 = 16_000;

    pub fn new(samples: Vec<f32>) -> Self {
        Self { samples }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.samples
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn duration_secs(&self) -> f32 {
        self.samples.len() as f32 / Self::SAMPLE_RATE as f32
    }
}

#[derive(Clone, Debug)]
pub struct Utterance {
    audio: AudioSamples,
}

impl Utterance {
    pub fn new(audio: AudioSamples) -> Self {
        Self { audio }
    }

    pub fn audio(&self) -> &AudioSamples {
        &self.audio
    }
}
