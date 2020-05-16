//! Takes a name or a list of names and verifies them against a variety of
//! biodiversity [Data Sources][data_source_ids]
//!
//! ## Example
//!
//! ```rust
//! use gnverify::{GNVerify, Input, MatchType};
//!
//! let gnv = GNVerify::new();
//! let inputs: Vec<Input> = vec![Input{id: None, name: "Homo sapiens".to_owned()}];
//! let outputs = gnv.verify(&inputs);
//! assert_eq!(outputs.len(), 1);
//! if let Some(output) = outputs.iter().next() {
//!     assert_eq!(output.match_type.to_string(), "Exact".to_owned());
//! }
//! ```
//!
//! [data_source_ids]: http://resolver.globalnames.org/data_sources
//!
#[warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
mod error;
/// format determines output format for name verification. It can be set to
/// CSV, JSON, and Pretty JSON.
pub mod format;
mod verif;

use crossbeam_channel::{Receiver, Sender};
pub use error::GNVerifyError;
pub use format::Format;
use log::error;
use serde_json;
pub use std::io;
use std::thread;
pub use verif::output::{MatchType, Output};
pub use verif::Input;
use verif::{remote, Verified};

/// Keeps configuration parameters and organizes main functions for changing
/// configuration and performing name-strings verification and formatting of
/// verification output.
#[derive(Debug, Default, Clone)]
pub struct GNVerify {
    /// list of IDs of Data Sources. Each Data Source is a checklist of scientific names / (e.g
    /// Encyclopedia of Life, GBIF, Catalogue of Life) and can be curated to a
    /// degree, automatically curated or not curated. If a name-string has a match to any
    /// of these sources, the matching result will always be returned in preferred_results
    /// section of the output.
    pub sources: Option<Vec<i64>>,
    /// Normally output would
    pub preferred_only: bool,
    /// Position of ScientificName field in the document. Default value is 1
    /// (the first field is 1, not 0). If gnverify verifies names from a txt
    /// file, it assumes that the text has one name per line, and nothing else.
    pub name_field: i64,
    /// size of a bach of names sent as a unit for verification to
    /// gnindex.
    pub batch_size: usize,
    /// sets format of the final output. It can be CSV, JSON, or Pretty JSON.
    pub format: Format,
}

impl GNVerify {
    /// Creates a new instance of GNVerify and sets default values for all fields.
    pub fn new() -> Self {
        GNVerify {
            batch_size: 500,
            name_field: 1,
            ..Default::default()
        }
    }

    /// Sets sources field. Sources is a list of IDs for data sources. If a
    /// match found for these data-sources, such data will be always returned
    /// to the user even if such results are not the best-scored results.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::GNVerify;
    ///
    /// let mut gnv = GNVerify::new();
    /// assert!(gnv.sources.is_none());
    /// gnv.sources(vec![1,11,169]);
    /// assert_eq!(gnv.sources.unwrap()[1], 11_i64);
    /// ```
    pub fn sources(&mut self, sources: Vec<i64>) {
        self.sources = Some(sources);
    }

    /// Sets the index of name-string field. For example, if your TSV file
    /// contains "ID", "ScientificName", "Reference", use name_index 2.
    ///
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::GNVerify;
    ///
    /// let mut gnv = GNVerify::new();
    /// assert_eq!(gnv.name_field, 1);
    /// gnv.name_field(3);
    /// assert_eq!(gnv.name_field, 3);
    /// ```
    pub fn name_field(&mut self, name_field: i64) {
        self.name_field = name_field;
    }
    /// Sets preferred_only field to true
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::GNVerify;
    ///
    /// let mut gnv = GNVerify::new();
    /// assert_eq!(gnv.preferred_only, false);
    /// gnv.preferred_only();
    /// assert_eq!(gnv.preferred_only, true);
    /// ```
    pub fn preferred_only(&mut self) {
        self.preferred_only = true;
    }

    /// Sets output format to one of: CSV, JSON, Pretty JSON.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::{GNVerify, Format};
    ///
    /// let mut gnv = GNVerify::new();
    /// assert_eq!(gnv.format.to_string(), "CSV");
    /// gnv.format(Format::Pretty);
    /// assert_eq!(gnv.format.to_string(), "Pretty");
    /// ```
    pub fn format(&mut self, format: Format) {
        self.format = format;
    }

    /// Takes input channel with name-strings to verify and uses output channel
    /// to send back results of verification. The input channel is then cloned
    /// for several workers, so they all send data to gnindex server in parallel.
    /// There input send name-string in batches and their size is determined by
    /// batch_size field.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::{GNVerify, Input};
    /// use crossbeam_channel::bounded;
    /// use std::thread;
    ///
    /// let mut gnv = GNVerify::new();
    ///
    /// let (in_s, in_r) = bounded(0);
    /// let (out_s, out_r) = bounded(0);
    /// thread::spawn(move || gnv.clone().verify_stream(in_r, out_s));
    /// let inputs: Vec<Input> = vec![Input{id: None, name: "Homo sapiens".to_owned()}];
    /// in_s.send(inputs).unwrap();
    /// let o = out_r.recv().unwrap();
    /// assert_eq!(o.iter().next().unwrap().name, "Homo sapiens");
    /// ```
    pub fn verify_stream(&self, in_r: Receiver<Vec<Input>>, out_s: Sender<Vec<Output>>) {
        for _ in 0..5 {
            let in_r1 = in_r.clone();
            let out_s1 = out_s.clone();
            let gnv = self.clone();
            thread::spawn(move || gnv.verify_worker(in_r1, out_s1));
        }
    }

    fn verify_worker(&self, in_r: Receiver<Vec<Input>>, in_s: Sender<Vec<Output>>) {
        for inputs in in_r {
            let outputs = self.verify(&inputs);
            in_s.send(outputs).unwrap();
        }
    }

    /// Takes as input a vector name-strings and returns back a vector of
    /// corresponding verification outputs for the name-strings.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::{GNVerify, Input, MatchType};
    ///
    /// let gnv = GNVerify::new();
    /// let inputs: Vec<Input> = vec![Input{id: None, name: "Homo sapiens".to_owned()}];
    /// let outputs = gnv.verify(&inputs);
    /// assert_eq!(outputs.len(), 1);
    /// if let Some(output) = outputs.iter().next() {
    ///     assert_eq!(output.match_type.to_string(), "Exact".to_owned());
    /// }
    /// ```
    ///
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
                        let err_str = Some(format!("{}", err));
                        error!("{}", err);
                        return self.bad_outputs(inputs, retries, err_str);
                    }
                }
            };
        }
    }

    /// Convenience function that takes as an input a vector of name-strings
    /// and prints out results in desired output format.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::{GNVerify, Input, MatchType};
    ///
    /// let gnv = GNVerify::new();
    /// let inputs: Vec<Input> = vec![Input{id: None, name: "Homo sapiens".to_owned()}];
    /// gnv.verify_and_format(&inputs);
    /// ```
    ///
    pub fn verify_and_format(&self, inputs: &Vec<Input>) {
        let outputs = self.verify(inputs);
        self.format_outputs(outputs, true);
    }

    /// Takes outputs of name-verification process and prints out the outputs
    /// in a desired format. It also takes with_headers parameter. If it is
    /// true, the printed output will have corresponding headers in CSV format.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use gnverify::{GNVerify, Input, MatchType};
    ///
    /// let gnv = GNVerify::new();
    /// let inputs: Vec<Input> = vec![Input{id: None, name: "Homo sapiens".to_owned()}];
    /// let outputs = gnv.verify(&inputs);
    /// assert_eq!(outputs.len(), 1);
    /// gnv.format_outputs(outputs, true);
    /// ```
    pub fn format_outputs(&self, outputs: Vec<Output>, with_headers: bool) {
        match self.format {
            Format::Pretty => self.write_json(outputs, true),
            Format::Compact => self.write_json(outputs, false),
            _ => self.write_csv(outputs, with_headers).unwrap(),
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

    fn write_csv(&self, outputs: Vec<Output>, with_headers: bool) -> anyhow::Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(with_headers)
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
