use chrono::{DateTime, Local, NaiveDateTime, Utc, Duration};
use clap::{AppSettings, Clap};
use regex::Regex;
use std::path::PathBuf;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Input file path
    file: PathBuf,
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();
    let re = Regex::new(r#"^: (\d+):"#).unwrap();
    let history = std::fs::read(&opts.file)?;
    let history = String::from_utf8_lossy(&history);

    let today = Utc::now().date();
    let mut first_command_today: Option<DateTime<Utc>> = None;
    let mut last_command_today: Option<DateTime<Utc>> = None;
    for command in history.lines().rev() {
        let capture = re.captures_iter(command).next();
        if capture.is_none() {
            continue;
        }
        let captured = &capture.unwrap()[1];
        let command_datetime_naive = NaiveDateTime::parse_from_str(captured, "%s");
        if command_datetime_naive.is_err() {
            continue;
        }
        let command_datetime_utc = DateTime::from_utc(command_datetime_naive.unwrap(), Utc);

        if last_command_today.is_none() {
            last_command_today = Some(command_datetime_utc);
        }
        if command_datetime_utc.date() < today {
            let start = first_command_today
                .map(|datetime| DateTime::<Local>::from(datetime).time().to_string())
                .unwrap_or("<Unknown>".to_string());
            let end = last_command_today
                .map(|datetime| DateTime::<Local>::from(datetime).time().to_string())
                .unwrap_or("<Unknown>".to_string());
            println!("Start: {}", start);
            println!("End: {}", end);
            println!("Duration: {}", last_command_today.sub(first_command_today));

            break;
        }
        first_command_today = Some(command_datetime_utc);
    }
    Ok(())
}
