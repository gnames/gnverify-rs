mod verif;

pub use verif::Input;
use verif::Verification;

#[derive(Default)]
pub struct GNVerify {
    sources: Option<Vec<i64>>,
}

impl GNVerify {
    pub fn new() -> Self {
        GNVerify {
            ..Default::default()
        }
    }

    pub fn set_sources(mut self, s: Option<Vec<i64>>) -> Self {
        self.sources = s;
        self
    }

    pub fn verify(&self, items: &Vec<Input>) -> Verification {
        Verification::new(items, &self.sources)
    }
}
