use clap::crate_version;
use std::fs::File;
use std::io;
use std::path;
use std::process;

#[macro_use]
extern crate clap;

fn main() {
    use clap::App;
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("gnverify.yml");
    let mut app = App::from_yaml(yaml).version(crate_version!());
    let matches = app.clone().get_matches();
    let mut sources: Option<Vec<i64>> = None;

    if let Some(ref input) = matches.value_of("INPUT") {
        if let Some(srs) = matches.value_of("sources") {
            sources = Some(parse_sources(srs));
        }
        if path::Path::new(input).exists() {
            match verify_file(input, sources) {
                Ok(_) => process::exit(0),
                Err(err) => {
                    println!("{:#?}", err);
                    process::exit(1);
                }
            }
        } else {
            let gnv = gnverify::GNVerify::new().set_sources(sources);
            let _res = gnv.verify(&vec![gnverify::Input {
                id: None,
                name: input.to_string(),
            }]);
        }
    } else {
        app.print_long_help().unwrap();
    }
}

fn verify_file(path: &str, sources: Option<Vec<i64>>) -> io::Result<()> {
    let mut inputs: Vec<gnverify::Input> = Vec::new();
    let f = File::open(path)?;
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(f);
    let mut fields_num = 0;
    for result in rdr.records() {
        let record = result?;
        if fields_num == 0 {
            fields_num = record.len();
        }
        match fields_num {
            0 => (),
            1 => {
                if record.len() > 0 {
                    inputs.push(gnverify::Input {
                        id: None,
                        name: record[0].to_owned(),
                    });
                }
            }
            _ => {
                if record.len() > 1 {
                    inputs.push(gnverify::Input {
                        id: Some(record[0].to_owned()),
                        name: record[1].to_owned(),
                    });
                }
            }
        };
    }
    let gnv = gnverify::GNVerify::new().set_sources(sources);
    let _res = gnv.verify(&inputs);
    Ok(())
}

fn parse_sources(sources: &str) -> Vec<i64> {
    let mut res: Vec<i64> = Vec::new();
    for v in sources.split(',') {
        let source = match v.trim().parse::<i64>() {
            Ok(i) => i,
            Err(err) => {
                print!("Cannot convert source arg '{}' to integer: {}\n", v, err);
                process::exit(1);
            }
        };
        res.push(source)
    }
    res
}
