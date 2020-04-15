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
        let outputs: Vec<Output> = Vec::with_capacity(inputs.len());
        loop {
            match remote::verify(inputs, &self.sources) {
                Ok(resolved) => {
                    // println!("{:#?}", resolved.name_resolver.responses);
                    return GNVerify::process_outputs(
                        outputs,
                        resolved.name_resolver.responses,
                        retries,
                    );
                }
                Err(err) => {
                    if retries < 3 {
                        retries += 1;
                    } else {
                        return outputs;
                    }
                    let error = Some(format!("{}", err));
                    return self.bad_outputs(inputs, retries, error);
                }
            };
        }
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

    pub fn verify_and_format(&self, inputs: &Vec<Input>) {
        let outputs = self.verify(inputs);
        match self.format {
            Format::Pretty => print!("{}", serde_json::to_string_pretty(&outputs).unwrap()),
            Format::Compact => print!("{}", serde_json::to_string(&outputs).unwrap()),
            _ => write_csv(outputs).unwrap(),
        }
    }

    fn process_outputs(
        mut outputs: Vec<Output>,
        results: Vec<Verified>,
        retries: i64,
    ) -> Vec<Output> {
        for item in results {
            outputs.push(Output::new(item, retries))
        }
        outputs
    }
}

fn write_csv(outputs: Vec<Output>) -> anyhow::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    for o in outputs {
        for c in o.to_csv() {
            wtr.serialize(c)?
        }
    }
    Ok(())
}
