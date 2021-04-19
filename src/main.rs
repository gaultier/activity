use chrono::{DateTime, Duration, Local, NaiveDateTime, NaiveTime, Utc};
use clap::{AppSettings, Clap};
use regex::Regex;
use std::path::PathBuf;

/// Summarize today's work activity based on the zsh command history.
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// History file
    file: PathBuf,
    #[clap(short, long, default_value = "30")]
    linger_minutes: u16,
    #[clap(short, long, default_value = "8")]
    workday_hours: u8,
    #[clap(short = 'W', long, default_value = "17")]
    workday_end_hour: u8,
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
    let linger: Duration = Duration::minutes(opts.linger_minutes as i64);
    let end_of_day = NaiveTime::from_hms(opts.workday_end_hour as u32, 0, 0);
    let workday = Duration::hours(opts.workday_hours as i64);
    {
        let mut command_date_times = history
            .lines()
            .rev()
            .filter_map(|cmd| re.captures_iter(cmd).next())
            .filter_map(|capture| NaiveDateTime::parse_from_str(&capture[1], "%s").ok())
            .map(|naive_date_time| DateTime::<Utc>::from_utc(naive_date_time, Utc))
            .skip_while(|utc_date_time| utc_date_time.time() > end_of_day)
            .take_while(|utc_date_time| utc_date_time.date() == today)
            .collect::<Vec<_>>();
        command_date_times.reverse();

        let (worked_spans, break_spans): (Vec<Span>, Vec<Span>) = command_date_times
            .windows(2)
            .filter_map(|span| match span {
                [start, end] => Some(Span {
                    start: *start,
                    end: *end,
                    duration: end.signed_duration_since(*start),
                }),
                _ => None,
            })
            .partition(|span| span.duration < linger);

        println!(
            "Breaks (more than {}m): {}",
            linger.num_minutes(),
            if break_spans.is_empty() { "None" } else { "" }
        );
        for s in &break_spans {
            let start = DateTime::<Local>::from(s.start).time();
            let end = DateTime::<Local>::from(s.end).time();
            println!("  - {}-{} {}m", start, end, s.duration.num_minutes());
        }

        let first_command_today: Option<DateTime<Utc>> =
            worked_spans.first().map(|span| span.start);
        let last_command_today: Option<DateTime<Utc>> = worked_spans.last().map(|span| span.end);

        let worked_duration: Duration =
            worked_spans
                .iter()
                .fold(Duration::zero(), |total_duration, span| {
                    total_duration
                        .checked_add(&span.duration)
                        .unwrap_or(total_duration)
                });

        let start = first_command_today
            .map(|datetime| DateTime::<Local>::from(datetime).time().to_string())
            .unwrap_or_else(|| "<Unknown>".to_string());
        let end = last_command_today
            .map(|datetime| DateTime::<Local>::from(datetime).time().to_string())
            .unwrap_or_else(|| "<Unknown>".to_string());
        println!("Start: {}", start);
        println!("End: {}", end);

        let worked_minutes = worked_duration.num_minutes();
        println!("Worked: {}h{}m", worked_minutes / 60, worked_minutes % 60);

        let remaining_duration = workday.checked_sub(&worked_duration).unwrap_or(workday);
        println!(
            "Remaining: {}h{}m",
            remaining_duration.num_minutes() / 60,
            remaining_duration.num_minutes() % 60
        );
    }
    Ok(())
}
