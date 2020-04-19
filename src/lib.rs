mod error;
pub mod format;
mod verif;

use crossbeam_channel::{Receiver, Sender};
pub use error::GNVerifyError;
pub use format::Format;
use serde_json;
pub use std::io;
use std::thread;
pub use verif::output::Output;
pub use verif::Input;
use verif::{remote, Verified};

#[derive(Debug, Default, Clone)]
pub struct GNVerify {
    sources: Option<Vec<i64>>,
    preferred_only: bool,
    pub batch_size: usize,
    pub format: Format,
}

impl GNVerify {
    pub fn new() -> Self {
        GNVerify {
            batch_size: 500,
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

    pub fn verify_stream(&self, in_r: Receiver<Vec<Input>>, out_s: Sender<Vec<Output>>) {
        for _ in 0..5 {
            let in_r1 = in_r.clone();
            let out_s1 = out_s.clone();
            let gnv = self.clone();
            thread::spawn(move || gnv.verify_worker(in_r1, out_s1));
        }
    }

    pub fn verify_worker(&self, in_r: Receiver<Vec<Input>>, in_s: Sender<Vec<Output>>) {
        for inputs in in_r {
            let outputs = self.verify(&inputs);
            in_s.send(outputs).unwrap();
        }
    }

    pub fn verify(&self, inputs: &Vec<Input>) -> Vec<Output> {
        let mut retries = 0;
        loop {
            match remote::verify(inputs, &self.sources) {
                Ok(resolved) => {
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
        self.format_outputs(outputs, true);
    }

    pub fn format_outputs(&self, outputs: Vec<Output>, is_first: bool) {
        match self.format {
            Format::Pretty => self.write_json(outputs, true),
            Format::Compact => self.write_json(outputs, false),
            _ => self.write_csv(outputs, is_first).unwrap(),
        }
    }

    fn process_outputs(&self, results: Vec<Verified>, retries: i64) -> Vec<Output> {
        let mut outputs: Vec<Output> = Vec::with_capacity(results.len());
        for item in results {
            outputs.push(Output::new(item, retries, self.preferred_only))
        }
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

    fn write_json(&self, outputs: Vec<Output>, pretty: bool) {
        for o in outputs {
            if pretty {
                print!("{}\n", serde_json::to_string_pretty(&o).unwrap());
            } else {
                print!("{}\n", serde_json::to_string(&o).unwrap());
            }
        }
    }

    fn write_csv(&self, outputs: Vec<Output>, is_first: bool) -> anyhow::Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(is_first)
            .from_writer(io::stdout());
        for o in outputs {
            for c in o.to_csv(self.preferred_only) {
                wtr.serialize(c)?
            }
        }
        wtr.flush()?;
        Ok(())
    }
}
