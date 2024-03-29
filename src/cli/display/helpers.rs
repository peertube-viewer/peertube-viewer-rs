// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use time::{format_description, Duration, OffsetDateTime};

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

pub fn pretty_date(d: OffsetDateTime) -> String {
    let now: OffsetDateTime =
        OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    pretty_duration_since(now - d)
}

pub fn pretty_duration_since(d: Duration) -> String {
    if d.whole_milliseconds() < 0 {
        return "From the future. Bug?".to_string();
    }
    match d {
        d if d.whole_minutes() < 1 => format!("{}s", d.whole_seconds()),
        d if d.whole_hours() < 1 => format!("{}min", d.whole_minutes()),
        d if d.whole_days() < 1 => format!("{}h", d.whole_hours()),
        d if d.whole_weeks() < 1 => format!("{}d", d.whole_days()),
        d if d.whole_weeks() < 5 => format!("{}w", d.whole_weeks()),
        d if d.whole_days() < 365 => format!("{}m", d.whole_days() / 30),
        d => format!("{}y", d.whole_days() / 365),
    }
}

pub fn pretty_duration(d: u64) -> String {
    match d {
        d if d < 10 => format!("00:0{d}"),
        d if d < 60 => format!("00:{d}"),
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

pub fn pretty_duration_or_live(d: u64, is_live: bool) -> String {
    if is_live {
        // Currently doesn't matter since lives aren't returned by search
        "LIVE".to_owned()
    } else {
        pretty_duration(d)
    }
}

pub fn full_date(d: OffsetDateTime) -> String {
    let format =
        format_description::parse("[weekday] [day padding:none] [month repr:long] [year]").unwrap();
    d.format(&format).unwrap_or_default()
}

pub fn display_length(mut i: usize) -> usize {
    let mut len = 1;
    while i >= 10 {
        len += 1;
        i /= 10;
    }

    len
}

pub fn remove_html(mut input: &str) -> String {
    let mut s = String::new();

    while !input.is_empty() {
        if input.starts_with("&amp") {
            s.push('&');
            input = &input[5..];
            continue;
        }
        if input.starts_with("&lt") {
            s.push('<');
            input = &input[4..];
            continue;
        }
        if input.starts_with("&gt") {
            s.push('>');
            input = &input[4..];
            continue;
        }
        if input.starts_with("&apos") {
            s.push('\'');
            input = &input[6..];
            continue;
        }
        if input.starts_with('&') {
            s.push('&');
            input = &input[1..];
            continue;
        }

        match (input.find('<'), input.find('&')) {
            (None, None) => {
                s.push_str(input);
                return s;
            }
            (None, Some(idx)) => {
                s.push_str(&input[..idx]);
                input = &input[idx..];
                continue;
            }
            (Some(idx), None) => {
                s.push_str(&input[..idx]);
                input = &input[idx + 1..];
                if let Some(idx) = input.find('>') {
                    if &input[..idx] == "br /" {
                        s.push('\n');
                    }
                    input = &input[idx + 1..];
                } else {
                    return s;
                }
            }
            (Some(idx1), Some(idx2)) if idx1 < idx2 => {
                s.push_str(&input[..idx1]);
                input = &input[idx1 + 1..];
                if let Some(idx) = input.find('>') {
                    if &input[..idx] == "br /" {
                        s.push('\n');
                    }
                    input = &input[idx + 1..];
                    continue;
                } else {
                    return s;
                }
            }
            (Some(_), Some(idx)) => {
                s.push_str(&input[..idx]);
                input = &input[idx..];
                continue;
            }
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn html() {
        assert_eq!(
            remove_html("Some text<br /><br />More text?"),
            "Some text\n\nMore text?"
        );

        assert_eq!(
            remove_html(
                r#"<p><span class="h-card"><a href="https://framatube.org/accounts/framasoft" class="u-url mention">@<span>framasoft</span></a></span> I love this. ❤️</p>"#
            ),
            "@framasoft I love this. ❤️"
        );
    }

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

    #[test]
    pub fn duration_since() {
        assert_eq!(pretty_duration_since(Duration::weeks(1)), "1w");
        assert_eq!(pretty_duration_since(Duration::weeks(4)), "4w");
        assert_eq!(pretty_duration_since(Duration::weeks(5)), "1m");
        assert_eq!(pretty_duration_since(Duration::seconds(1)), "1s");
        assert_eq!(pretty_duration_since(Duration::seconds(59)), "59s");
        assert_eq!(pretty_duration_since(Duration::seconds(61)), "1min");
        assert_eq!(pretty_duration_since(Duration::minutes(1)), "1min");
        assert_eq!(pretty_duration_since(Duration::minutes(59)), "59min");
        assert_eq!(pretty_duration_since(Duration::minutes(61)), "1h");
        assert_eq!(pretty_duration_since(Duration::hours(1)), "1h");
        assert_eq!(pretty_duration_since(Duration::hours(23)), "23h");
        assert_eq!(pretty_duration_since(Duration::hours(24)), "1d");
        assert_eq!(pretty_duration_since(Duration::weeks(51)), "11m");
        assert_eq!(pretty_duration_since(Duration::weeks(52)), "12m");
        assert_eq!(pretty_duration_since(Duration::weeks(53)), "1y");
    }

    #[test]
    pub fn date() {
        use time::format_description::well_known::Rfc3339;
        assert_eq!(
            full_date(OffsetDateTime::parse("2016-07-08T09:10:11+00:00", &Rfc3339).unwrap()),
            "Friday 8 July 2016"
        );
    }
}
