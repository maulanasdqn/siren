#[derive(Clone, Debug, Default)]
pub struct Transcript(String);

impl Transcript {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn text(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }

    pub fn push_segment(&mut self, segment: &str) {
        let segment = segment.trim();
        if segment.is_empty() {
            return;
        }
        if !self.0.is_empty() {
            self.0.push(' ');
        }
        self.0.push_str(segment);
    }
}
