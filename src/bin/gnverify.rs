use clap::crate_version;
use std::fs::File;
use std::io::{self, BufRead};
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

    if let Some(ref input) = matches.value_of("INPUT") {
        if let Some(srs) = matches.value_of("sources") {
            print!("{:?}", parse_sources(srs));
        }
        if path::Path::new(input).exists() {
            match verify_file(input) {
                Ok(_) => process::exit(0),
                Err(err) => {
                    println!("{:#?}", err);
                    process::exit(1);
                }
            }
        } else {
            println!("{}", input);
        }
    } else {
        app.print_long_help().unwrap();
    }
}

fn verify_file(path: &str) -> io::Result<()> {
    let f = File::open(path)?;
    let lines = io::BufReader::new(f).lines();
    for l in lines {
        if let Ok(res) = l {
            println!("{}", res);
        }
    }
    Ok(())
}

fn parse_sources(sources: &str) -> Vec<u32> {
    let mut res: Vec<u32> = Vec::new();
    for v in sources.split(',') {
        let source = match v.trim().parse::<u32>() {
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