mod verif;

use serde_json;
pub use verif::output::Output;
pub use verif::Input;
use verif::{remote, Verified};

pub fn verify(inputs: &Vec<Input>, sources: &Option<Vec<i64>>) -> Vec<Output> {
    let mut retries = 0;
    let outputs: Vec<Output> = Vec::with_capacity(inputs.len());
    loop {
        match remote::verify(inputs, sources) {
            Ok(resolved) => {
                // println!("{:#?}", resolved.name_resolver.responses);
                return process_outputs(outputs, resolved.name_resolver.responses, retries);
            }
            Err(_err) => {
                if retries < 3 {
                    retries += 1;
                } else {
                    return outputs;
                }
            }
        };
    }
}

pub fn verify_and_format(inputs: &Vec<Input>, sources: &Option<Vec<i64>>) -> String {
    let outputs = verify(inputs, sources);
    format!("{}", serde_json::to_string_pretty(&outputs).unwrap())
}

fn process_outputs(mut outputs: Vec<Output>, results: Vec<Verified>, retries: i64) -> Vec<Output> {
    for item in results {
        outputs.push(Output::new(item, retries))
    }
    outputs
}
