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

struct Span {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    duration: Duration,
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

        let spans = command_date_times
            .windows(2)
            .filter_map(|span| match span {
                [start, end] => Some(Span {
                    start: *start,
                    end: *end,
                    duration: end.signed_duration_since(*start),
                }),
                _ => None,
            })
            .filter(|span| span.duration < linger)
            .collect::<Vec<_>>();

        let first_command_today: Option<DateTime<Utc>> = spans.first().map(|span| span.start);
        let last_command_today: Option<DateTime<Utc>> = spans.last().map(|span| span.end);

        for s in &spans {
            println!("{}-{} {}", s.start.time(), s.end.time(), s.duration);
        }
        let total_duration: Duration =
            spans
                .iter()
                .fold(Duration::zero(), |total_duration, span| {
                    total_duration
                        .checked_add(&span.duration)
                        .unwrap_or(total_duration)
                });

        let start = first_command_today
            .map(|datetime| DateTime::<Local>::from(datetime).time().to_string())
            .unwrap_or("<Unknown>".to_string());
        let end = last_command_today
            .map(|datetime| DateTime::<Local>::from(datetime).time().to_string())
            .unwrap_or("<Unknown>".to_string());
        println!("Start: {}", start);
        println!("End: {}", end);
        let total_duration_minutes = total_duration.num_minutes();
        println!(
            "Duration: {}h{}m",
            total_duration_minutes / 60,
            total_duration_minutes % 60
        );
    }
    Ok(())
}
