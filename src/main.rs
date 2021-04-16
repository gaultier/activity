use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, Utc};
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
    let linger: Duration = Duration::minutes(40);
    let end_of_day = NaiveTime::from_hms(17, 0, 0);
    {
        let mut command_date_times = history
            .lines()
            .rev()
            .filter_map(|cmd| re.captures_iter(cmd).next())
            .map(|capture| NaiveDateTime::parse_from_str(&capture[1], "%s"))
            .filter_map(|res| res.ok())
            .map(|naive_date_time| DateTime::<Utc>::from_utc(naive_date_time, Utc))
            .skip_while(|utc_date_time| utc_date_time.time() > end_of_day)
            .take_while(|utc_date_time| utc_date_time.date() == today)
            .collect::<Vec<_>>();
        command_date_times.reverse();
        let durations = command_date_times
            .windows(2)
            .filter(|span| match span {
                [start, end] => end.signed_duration_since(*start) < linger,
                _ => false,
            })
            .collect::<Vec<_>>();
        let first_command_today = durations.first().and_then(|span| span.first());
        let last_command_today = durations.last().and_then(|span| span.last());

        for d in &durations {
            println!("{:#?}", d);
        }
        let total_duration_minutes: i64 = durations
            .iter()
            .map(|span| match span {
                [start, end] => end.signed_duration_since(*start).num_minutes(),
                _ => 0,
            })
            .sum();

        let start = first_command_today
            .map(|datetime| DateTime::<Local>::from(*datetime).time().to_string())
            .unwrap_or("<Unknown>".to_string());
        let end = last_command_today
            .map(|datetime| DateTime::<Local>::from(*datetime).time().to_string())
            .unwrap_or("<Unknown>".to_string());
        println!("Start: {}", start);
        println!("End: {}", end);
        println!(
            "Duration: {}h{}m",
            total_duration_minutes / 60,
            total_duration_minutes % 60
        );
    }
    Ok(())
}
