use anyhow::Context;
use chrono::{
    offset::{Local, Utc},
    DateTime, Duration, NaiveDateTime,
};
use file_rotate::suffix::{FileLimit, SuffixScheme, TimestampSuffixScheme};
use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{
    collections::HashMap,
    fmt::Write,
    fs,
    time::Instant,
    io::{self, BufRead, Read},
    path::PathBuf,
};

use flate2::read::GzDecoder;

use clap::{AppSettings, Clap};

/// Assemble logs that were rotated with the `file-rotate` crate. Given the main log, it reads all
/// rotated log files in order, decompresses if necessary, concatenates, filters using optional
/// `jq` query, and formats it.
///
/// Example:
///
/// Show only logs within a 30 minutes timespan.
///
/// assemble_logs all.log '.ts > "2021-09-02T22" and .ts < "2021-09-02T22:30"'  | less -R

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// The path to the main log file.
    log_path: PathBuf,
    /// JQ query; must return a bool; only used for filtering
    jq: Option<String>,

    /// Compact - don't print newline on each key-value
    #[clap(short, long)]
    compact: bool,

    /// Print error details (default is without details)
    #[clap(short, long)]
    error_details: bool,
    /// No formatting: just print json
    #[clap(short, long)]
    no_format: bool,
    /// Only effective with --no-format
    #[clap(long)]
    jq_transformation: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let suffix_scheme = TimestampSuffixScheme::default(FileLimit::Age(Duration::weeks(1)));
    let paths = suffix_scheme
        .scan_suffixes(&opts.log_path)
        .into_iter()
        .rev() // oldest to newest;
        .map(|suffix| (suffix.to_path(&opts.log_path), suffix.compressed))
        .chain([(opts.log_path.clone(), false)])
        .collect::<Vec<_>>();

    let mut jq = opts
        .jq
        .as_ref()
        .map(|jq| jq_rs::compile(&jq).expect("Failed compiling jq program"));
    let mut jq_trans = opts
        .jq_transformation
        .as_ref()
        .map(|jq| jq_rs::compile(&jq).expect("Failed to compile jq transformation program"));

    let start = Instant::now();
    // stream of strings = Read
    // how can we paste strings together...?
    let mut content = Vec::new();
    for (path, compressed) in paths {
        let mut file = fs::File::open(&path)?;
        if compressed {
            let start = Instant::now();
            let mut decoder = GzDecoder::new(file);
            decoder.read_to_end(&mut content)?;
            println!("{:?} decoded in {:?}", path,  Instant::now()  - start);
        } else {
            file.read_to_end(&mut content)?;
        }
    }

    let mut n_lines = 0;
    for line in io::BufReader::new(&content[..]).lines() {
        let line = line?;
        let include = if let Some(ref mut jq) = jq {
            match jq.run(&line) {
                Ok(s) => {
                    n_lines += 1;
                    s.trim()
                        .parse::<bool>()
                        .expect("jq filter must output a bool")
                }
                Err(_) => false,
            }
        } else {
            n_lines += 1;
            true
        };

        if include {
            if opts.no_format {
                if let Some(ref mut jq_trans) = jq_trans {
                    match jq_trans.run(&line) {
                        Ok(s) => print!("{}", s),
                        Err(e) => println!("{}", e),
                    }
                } else {
                    println!("{}", line);
                }
            } else {
                match format(&line, &opts) {
                    Ok(formatted) => println!("{}", formatted),
                    Err(e) => {
                        if opts.error_details {
                            println!("Format error: {:#?}, line: {}", e, line);
                        } else {
                            println!("<error; {}>", e);
                        }
                    }
                }
            }
        }
    }
    println!("END OUTPUT - n_lines={}", n_lines);
    println!("Duration: {:?}", Instant::now() - start);

    Ok(())
}

#[derive(Deserialize)]
struct Record {
    tag: String,
    msg: String,
    level: String,
    #[serde(with = "date_time")]
    ts: NaiveDateTime,
    #[serde(flatten)]
    rest: HashMap<String, Value>,
}
fn format(record: &str, opts: &Opts) -> anyhow::Result<String> {
    use termion::{color, style};
    let record: Record = from_str(record).context("serde")?;

    let level_color: &dyn color::Color = match record.level.as_str() {
        "CRIT" | "ERRO" => &color::Red,
        "WARN" => &color::Yellow,
        "INFO" => &color::Green,
        "DEBG" => &color::Cyan,
        "TRCE" => &color::Magenta,
        _ => &color::Red,
    };

    let mut output = String::new();

    // Timestamp
    write!(
        &mut output,
        "{}{} ",
        style::Reset,
        record.ts.format("%b %d %H:%M:%S%.3f")
    )?;
    write!(
        &mut output,
        "{}{}{}",
        color::Fg(level_color),
        style::Bold,
        record.level
    )?;
    write!(
        &mut output,
        " {}{}[{}] ",
        style::Reset,
        color::Fg(color::Rgb(128, 128, 128)),
        record.tag
    )?;

    write!(
        &mut output,
        "{}{}{}",
        style::Bold,
        color::Fg(color::Reset),
        record.msg
    )?;

    let len = record.rest.len();
    for (i, (key, value)) in record.rest.into_iter().enumerate() {
        let newline = if opts.compact { "" } else { "\n\t" };
        write!(
            &mut output,
            " {}{}{}: {}{}",
            newline,
            style::Bold,
            key,
            style::Reset,
            value,
        )?;
    }

    Ok(output)
}

pub mod date_time {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3f";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", date.format(FORMAT)))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        NaiveDateTime::parse_from_str(&String::deserialize(deserializer)?, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
