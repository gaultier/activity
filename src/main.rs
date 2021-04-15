use std::path::PathBuf;
use regex::Regex;
use chrono::{offset::TimeZone, Local, NaiveDateTime};
use clap::{AppSettings, Clap};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Input file path
    file: PathBuf,
    /// Bucket size
     #[clap(short, long, default_value = "3600")]
    bucket_size: u64
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    let re = Regex::new(r#"^: (\d+):"#).unwrap();

    let history = std::fs::read(&opts.file)?;
    let history = String::from_utf8_lossy(&history);
    for command in history.lines() {
        let capture = re.captures_iter(command).next();
        if capture.is_none() {
            continue;
        }
        let captured = &capture.unwrap()[1];
        let utc_date_time = NaiveDateTime::parse_from_str(captured, "%s");
        if utc_date_time.is_err() {continue}
        let local_date_time = Local.from_local_datetime(&utc_date_time.unwrap()).unwrap();
        println!("{}", local_date_time);
    }

    Ok(())
}
