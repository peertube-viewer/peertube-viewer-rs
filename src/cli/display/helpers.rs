use chrono::{DateTime, Duration, FixedOffset, Utc};
use std::time::SystemTime;

pub fn pretty_size(mut s: u64) -> String {
    const PREFIXES: [&str; 5] = ["", "K", "M", "G", "E"];
    let mut id = 0;
    while s >= 1024 && id < 5 {
        s /= 1024;
        id += 1;
    }

    format!("{}{}B", s, PREFIXES[id])
}

pub fn display_count(mut c: u64) -> String {
    const PREFIXES: [&str; 5] = ["", "K", "M", "G", "E"];
    let mut id = 0;
    while c >= 1000 && id < 5 {
        c /= 1000;
        id += 1;
    }

    format!("{}{}", c, PREFIXES[id])
}

pub fn pretty_date(d: Option<&DateTime<FixedOffset>>) -> String {
    let now: DateTime<Utc> = SystemTime::now().into();
    d.map(|t| pretty_duration_since(now.naive_local().signed_duration_since(t.naive_local())))
        .unwrap_or_default()
}

pub fn pretty_duration_since(d: Duration) -> String {
    if d.num_milliseconds() < 0 {
        return "From the future. Bug?".to_string();
    }
    match d {
        d if d.num_hours() < 1 => format!("{}min", d.num_minutes()),
        d if d.num_days() < 1 => format!("{}h", d.num_hours()),
        d if d.num_weeks() < 1 => format!("{}d", d.num_days()),
        d if d.num_weeks() < 5 => format!("{}w", d.num_weeks()),
        d if d.num_days() < 365 => format!("{}m", d.num_days() / 30),
        d => format!("{}y", d.num_days() / 365),
    }
}

pub fn pretty_duration(d: u64) -> String {
    match d {
        d if d < 10 => format!("00:0{}", d),
        d if d < 60 => format!("00:{}", d),
        d if d < 600 && d % 60 < 10 => format!("0{}:0{}", d / 60, d % 60),
        d if d < 600 => format!("0{}:{}", d / 60, d % 60),
        d if d < 3600 && d % 60 < 10 => format!("{}:0{}", d / 60, d % 60),
        d if d < 3600 => format!("{}:{}", d / 60, d % 60),
        d if d % 3600 < 600 && d % 60 < 10 => {
            format!("{}:0{}:0{}", d / 3600, (d % 3600) / 60, d % 60)
        }
        d if d % 3600 < 600 => format!("{}:0{}:{}", d / 3600, (d % 3600) / 60, d % 60),
        d if d % 60 < 10 => format!("{}:{}:0{}", d / 3600, (d % 3600) / 60, d % 60),
        d => format!("{}:{}:{}", d / 3600, (d % 3600) / 60, d % 60),
    }
}

pub fn full_date(d: Option<&DateTime<FixedOffset>>) -> String {
    d.map(|t| t.format("%a %b %Y").to_string())
        .unwrap_or_default()
}

pub fn display_length(mut i: usize) -> usize {
    let mut len = 1;
    while i >= 10 {
        len += 1;
        i /= 10;
    }

    len
}

#[cfg(test)]
mod helpers {
    use super::*;

    #[test]
    pub fn length() {
        assert_eq!(display_length(0), 1);
        assert_eq!(display_length(1), 1);
        assert_eq!(display_length(9), 1);
        assert_eq!(display_length(10), 2);
        assert_eq!(display_length(11), 2);
        assert_eq!(display_length(99), 2);
        assert_eq!(display_length(100), 3);
        assert_eq!(display_length(101), 3);
    }

    #[test]
    pub fn count() {
        assert_eq!(display_count(0), "0");
        assert_eq!(display_count(10), "10");
        assert_eq!(display_count(999), "999");
        assert_eq!(display_count(1000), "1K");
        assert_eq!(display_count(1001), "1K");
        assert_eq!(display_count(1999), "1K");
        assert_eq!(display_count(2000), "2K");
        assert_eq!(display_count(2001), "2K");
        assert_eq!(display_count(999999), "999K");
        assert_eq!(display_count(1000000), "1M");
    }

    #[test]
    pub fn size() {
        assert_eq!(pretty_size(0), "0B");
        assert_eq!(pretty_size(10), "10B");
        assert_eq!(pretty_size(1023), "1023B");
        assert_eq!(pretty_size(1024), "1KB");
        assert_eq!(pretty_size(1025), "1KB");
        assert_eq!(pretty_size(2047), "1KB");
        assert_eq!(pretty_size(2048), "2KB");
        assert_eq!(pretty_size(2049), "2KB");
        assert_eq!(pretty_size(1048575), "1023KB");
        assert_eq!(pretty_size(1048576), "1MB");
    }

    #[test]
    pub fn duration() {
        assert_eq!(pretty_duration(0), "00:00");
        assert_eq!(pretty_duration(1), "00:01");
        assert_eq!(pretty_duration(9), "00:09");
        assert_eq!(pretty_duration(59), "00:59");
        assert_eq!(pretty_duration(60), "01:00");
        assert_eq!(pretty_duration(119), "01:59");
        assert_eq!(pretty_duration(120), "02:00");
        assert_eq!(pretty_duration(3599), "59:59");
        assert_eq!(pretty_duration(3600), "1:00:00");
        assert_eq!(pretty_duration(7199), "1:59:59");
        assert_eq!(pretty_duration(7200), "2:00:00");
    }
}
