use digest::{FixedOutput, FixedOutputReset, OutputSizeUser, Reset, Update};
use murmur2::murmur2;

#[derive(Clone)]
pub struct Murmur2(Vec<u8>);

impl Murmur2 {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Update for Murmur2 {
    fn update(&mut self, data: &[u8]) {
        self.0.extend(data.iter().copied().filter(|&e| e != 9 && e != 10 && e != 13 && e != 32))
    }
}

impl Reset for Murmur2 {
    fn reset(&mut self) {
        self.0 = Vec::new()
    }
}

impl OutputSizeUser for Murmur2 {
    type OutputSize = digest::typenum::U4;
}

impl FixedOutput for Murmur2 {
    fn finalize_into(self, out: &mut digest::Output<Self>) {
        *out = murmur2(&self.0, 1).to_be_bytes().into();
    }
}

impl FixedOutputReset for Murmur2 {
    fn finalize_into_reset(&mut self, out: &mut digest::Output<Self>) {
        *out = murmur2(&self.0, 1).to_be_bytes().into();
        self.0 = Vec::new();
    }
}
