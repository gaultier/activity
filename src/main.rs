use std::env;
use regex::Regex;
use chrono::{offset::TimeZone, Local, NaiveDateTime};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let re = Regex::new(r#": (\d+):"#).unwrap();

    let history = std::fs::read(&args[1])?;
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
