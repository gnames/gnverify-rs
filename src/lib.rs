mod error;
pub mod format;
mod verif;

pub use error::GNVerifyError;
pub use format::Format;
use serde_json;
pub use std::io;
pub use verif::output::Output;
pub use verif::Input;
use verif::{remote, Verified};

#[derive(Debug, Default)]
pub struct GNVerify {
    sources: Option<Vec<i64>>,
    preferred_only: bool,
    pub format: Format,
}

impl GNVerify {
    pub fn new() -> Self {
        GNVerify {
            ..Default::default()
        }
    }

    pub fn sources(&mut self, sources: Vec<i64>) {
        self.sources = Some(sources);
    }

    pub fn preferred_only(&mut self) {
        self.preferred_only = true;
    }

    pub fn format(&mut self, format: Format) {
        self.format = format;
    }

    pub fn verify(&self, inputs: &Vec<Input>) -> Vec<Output> {
        let mut retries = 0;
        loop {
            match remote::verify(inputs, &self.sources) {
                Ok(resolved) => {
                    // println!("{:#?}", resolved.name_resolver.responses);
                    return self.process_outputs(resolved.name_resolver.responses, retries);
                }
                Err(err) => {
                    if retries < 3 {
                        retries += 1;
                    } else {
                        let error = Some(format!("{}", err));
                        return self.bad_outputs(inputs, retries, error);
                    }
                }
            };
        }
    }

    pub fn verify_and_format(&self, inputs: &Vec<Input>) {
        let outputs = self.verify(inputs);
        match self.format {
            Format::Pretty => print!("{}", serde_json::to_string_pretty(&outputs).unwrap()),
            Format::Compact => print!("{}", serde_json::to_string(&outputs).unwrap()),
            _ => self.write_csv(outputs).unwrap(),
        }
    }

    fn process_outputs(&self, results: Vec<Verified>, retries: i64) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::with_capacity(results.len());
        for item in results {
            outputs.push(Output::new(item, retries, self.preferred_only))
        }
        println!("{:#?}", self);
        outputs
    }

    fn bad_outputs(&self, inputs: &Vec<Input>, retries: i64, error: Option<String>) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::with_capacity(inputs.len());
        for input in inputs {
            let output = Output {
                id: input.id.clone(),
                name: input.name.clone(),
                retries,
                error: error.clone(),
                ..Default::default()
            };
            outputs.push(output);
        }
        outputs
    }

    fn write_csv(&self, outputs: Vec<Output>) -> anyhow::Result<()> {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        for o in outputs {
            for c in o.to_csv(self.preferred_only) {
                wtr.serialize(c)?
            }
        }
        Ok(())
    }
}
