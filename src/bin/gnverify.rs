use clap::crate_version;
use crossbeam_channel::{bounded, Receiver, Sender};
use gnverify::{Format, GNVerify};
use log::{error, info};
use std::fs::File;
use std::io::{self, Read};
use std::path;
use std::process;
use std::thread;
use std::time::Instant;
use stderrlog::{self, Timestamp};

#[macro_use]
extern crate clap;

fn main() {
    stderrlog::new()
        .verbosity(2)
        .timestamp(Timestamp::Second)
        .init()
        .unwrap();
    use clap::App;
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("gnverify.yml");
    let mut app = App::from_yaml(yaml).version(crate_version!());
    let matches = app.clone().get_matches();
    let mut gnv = GNVerify::new();
    if matches.is_present("preferred_only") {
        gnv.preferred_only();
    }
    if let Some(format_str) = matches.value_of("format") {
        match Format::new(format_str) {
            Ok(format) => {
                gnv.format(format);
            }
            Err(err) => {
                error!("using default format {}: {}", gnv.format.to_string(), err);
            }
        }
    }
    if let Some(name_index) = matches.value_of("name_field") {
        gnv.name_field(parse_name_index(name_index))
    }
    if let Some(srs) = matches.value_of("sources") {
        gnv.sources(parse_sources(srs));
    }
    if let Some(ref input) = matches.value_of("INPUT") {
        if path::Path::new(input).exists() {
            let f = File::open(input).unwrap();
            match verify_file(gnv, f) {
                Ok(_) => process::exit(0),
                Err(err) => {
                    println!("{:#?}", err);
                    process::exit(1);
                }
            }
        } else {
            gnv.verify_and_format(&vec![gnverify::Input {
                id: None,
                name: input.to_string(),
            }]);
        }
    } else if is_readable_stdin() {
        match verify_file(gnv, io::stdin()) {
            Ok(_) => process::exit(0),
            Err(err) => {
                println!("{:#?}", err);
                process::exit(1);
            }
        }
    } else {
        app.print_long_help().unwrap();
    }
}

fn verify_file<'a, R>(gnv: GNVerify, r: R) -> io::Result<()>
where
    R: Read,
{
    let (in_s, in_r) = bounded(0);
    let (out_s, out_r) = bounded(0);
    let (done_s, done_r) = bounded::<bool>(0);
    let gnv_clone1 = gnv.clone();
    let gnv_clone2 = gnv.clone();
    let batch_size = gnv.batch_size;
    let name_field = gnv.name_field;
    thread::spawn(move || gnv_clone1.verify_stream(in_r, out_s));
    thread::spawn(move || process_outputs(gnv_clone2, out_r, done_s));

    let rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(r);

    prepare_inputs(rdr, in_s, batch_size, name_field);
    done_r.recv().unwrap();
    Ok(())
}

fn process_outputs(
    gnv: gnverify::GNVerify,
    out_r: Receiver<Vec<gnverify::Output>>,
    done_s: Sender<bool>,
) {
    let mut is_first = true;
    for outputs in out_r {
        gnv.format_outputs(outputs, is_first);
        is_first = false;
    }
    done_s.send(true).unwrap();
}

fn prepare_inputs<R>(
    rdr: csv::Reader<R>,
    in_s: Sender<Vec<gnverify::Input>>,
    batch_size: usize,
    name_field: i64,
) where
    R: Read,
{
    let mut inputs: Vec<gnverify::Input> = Vec::with_capacity(batch_size);
    let time_start = Instant::now();
    let mut good_rows = 0;

    for (i, result) in rdr.into_records().enumerate() {
        if inputs.len() == batch_size {
            in_s.send(inputs).unwrap();
            inputs = Vec::with_capacity(batch_size);
        }
        if (i + 1) % 10_000 == 0 {
            let duration = time_start.elapsed().as_secs() as f32;
            let speed = (i + 1) as f32 / duration;
            info!("Processed {} rows, {:.0} names/sec", i + 1, speed);
        }
        if let Ok(record) = result {
            if (i + 1) / (good_rows + 1) > 100 {
                error!("Too many bad rows. Make sure you set name_field to the right number");
                process::exit(1);
            }
            if record.len() as i64 >= name_field {
                good_rows += 1;
                inputs.push(gnverify::Input {
                    id: None,
                    name: record[name_field as usize - 1].to_owned(),
                });
            };
        };
    }
    in_s.send(inputs).unwrap();
    drop(in_s);
}

fn parse_name_index(index_str: &str) -> i64 {
    if let Ok(name_index) = index_str.trim().parse::<i64>() {
        match name_index {
            i if name_index > 0 => return i,
            _ => {
                error!("Enter number 1 or larger for name_field");
                process::exit(1);
            }
        }
    }
    error!(
        "Cannot parse name_field index '{}', enter number 1 or larger",
        index_str
    );
    process::exit(1);
}

fn parse_sources(sources: &str) -> Vec<i64> {
    let mut res: Vec<i64> = Vec::new();
    for v in sources.split(',') {
        let source = match v.trim().parse::<i64>() {
            Ok(i) => i,
            Err(err) => {
                error!("Cannot convert source '{}' to integer: {}\n", v, err);
                process::exit(1);
            }
        };
        res.push(source)
    }
    res
}

/// Returns true if and only if stdin is believed to be readable.
///
/// When stdin is readable, command line programs may choose to behave
/// differently than when stdin is not readable. For example, `command foo`
/// might search the current directory for occurrences of `foo` where as
/// `command foo < some-file` or `cat some-file | command foo` might instead
/// only search stdin for occurrences of `foo`.
pub fn is_readable_stdin() -> bool {
    #[cfg(unix)]
    fn imp() -> bool {
        use same_file::Handle;
        use std::os::unix::fs::FileTypeExt;

        let ft = match Handle::stdin().and_then(|h| h.as_file().metadata()) {
            Err(_) => return false,
            Ok(md) => md.file_type(),
        };
        ft.is_file() || ft.is_fifo()
    }

    #[cfg(windows)]
    fn imp() -> bool {
        use winapi_util as winutil;

        winutil::file::typ(winutil::HandleRef::stdin())
            .map(|t| t.is_disk() || t.is_pipe())
            .unwrap_or(false)
    }

    !is_tty_stdin() && imp()
}

/// Returns true if and only if stdin is believed to be connectted to a tty
/// or a console.
pub fn is_tty_stdin() -> bool {
    atty::is(atty::Stream::Stdin)
}
