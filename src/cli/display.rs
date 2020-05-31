use peertube_api::{Resolution, Video};

use super::{
    config::{Blacklist, NsfwBehavior},
    history::History,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use std::{fmt, time::SystemTime};
use termion::{color, style};

use std::rc::Rc;

use unicode_width::UnicodeWidthStr;

const DEFAULT_COLS: usize = 20;

pub struct Display {
    cols: usize,
    nsfw: NsfwBehavior,
    colors: bool,
    video_layout: Vec<VideoLayoutItem>,
}

#[derive(Debug)]
enum MaybeColor<T: color::Color> {
    No,
    Fg(color::Fg<T>),
}

impl<T: color::Color> fmt::Display for MaybeColor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeColor::No => write!(f, ""),
            MaybeColor::Fg(c) => write!(f, "{}", c),
        }
    }
}

impl Display {
    pub fn new(nsfw: NsfwBehavior, colors: bool) -> Display {
        let cols = termion::terminal_size()
            .map(|(c, _r)| c as usize)
            .unwrap_or(DEFAULT_COLS);

        let video_layout = vec![
            VideoLayoutItem::Color(Box::new(color::Fg(color::Blue))),
            VideoLayoutItem::Name,
            VideoLayoutItem::String(" ".to_string()),
            VideoLayoutItem::Alignement,
            VideoLayoutItem::Color(Box::new(color::Fg(color::Green))),
            VideoLayoutItem::Channel,
            VideoLayoutItem::String(" ".to_string()),
            VideoLayoutItem::Alignement,
            VideoLayoutItem::Color(Box::new(color::Fg(color::Cyan))),
            VideoLayoutItem::Host,
            VideoLayoutItem::Alignement,
            VideoLayoutItem::Color(Box::new(color::Fg(color::Yellow))),
            VideoLayoutItem::String(" [".to_string()),
            VideoLayoutItem::Duration,
            VideoLayoutItem::String("] ".to_string()),
            VideoLayoutItem::Alignement,
            VideoLayoutItem::Color(Box::new(color::Fg(color::Green))),
            VideoLayoutItem::Views,
            VideoLayoutItem::String(" ".to_string()),
            VideoLayoutItem::Alignement,
            VideoLayoutItem::Published,
            VideoLayoutItem::String(" ".to_string()),
            VideoLayoutItem::Alignement,
            VideoLayoutItem::Color(Box::new(color::Fg(color::Red))),
            VideoLayoutItem::Nsfw,
            VideoLayoutItem::Color(Box::new(color::Fg(color::Reset))),
        ];
        Display {
            cols,
            nsfw,
            colors,
            video_layout,
        }
    }

    /// Abstract removing colors
    /// This should always be used instead of termions colors for displaying output
    fn fg_color<C: color::Color>(&self, c: C) -> MaybeColor<C> {
        if self.colors {
            MaybeColor::Fg(color::Fg(c))
        } else {
            MaybeColor::No
        }
    }

    /// Display a list of video results
    pub fn video_list(&self, videos: &[Rc<Video>], history: &History, blacklist: &impl Blacklist) {
        let mut video_parts = Vec::new();
        let mut alignements_total =
            vec![0; self.video_layout.iter().filter(|i| i.is_align()).count()];
        for v in videos {
            let mut align_off: usize = 0;
            let mut align_id = 0;
            let mut tmp_str = Vec::new();
            let mut tmp_align = Vec::new();
            for item in &self.video_layout {
                if !item.is_align() {
                    let dsp = item.display(v, self.colors);
                    let s: &str = &dsp;
                    if item.length_matters() {
                        align_off += UnicodeWidthStr::width(s);
                    }
                    tmp_str.push(dsp);
                } else {
                    if alignements_total[align_id] < align_off {
                        alignements_total[align_id] = align_off;
                    }
                    tmp_align.push(align_off);
                    align_off = 0;
                    align_id += 1;
                }
            }
            video_parts.push((tmp_str, tmp_align));
        }

        let mut buffer = String::new();
        for (id, parts) in video_parts.into_iter().enumerate() {
            buffer.push_str(&(id + 1).to_string());
            buffer.push_str(
                &" ".to_string()
                    .repeat(display_length(videos.len()) - display_length(id + 1)),
            );
            buffer.push_str(": ");

            let mut layout_align_it = alignements_total.iter();
            let mut layout_it = self.video_layout.iter();
            let mut parts_it = parts.0.iter();
            let mut parts_align_it = parts.1.iter();

            loop {
                if let Some(item) = layout_it.next() {
                    if item.is_align() {
                        let spacing = layout_align_it
                            .next()
                            .expect("Internal error: align smaller than expected")
                            - parts_align_it
                                .next()
                                .expect("Internal error: align smaller than expected");
                        buffer.push_str(&" ".to_string().repeat(spacing));
                    } else {
                        buffer.push_str(
                            &parts_it
                                .next()
                                .expect("Internal Error: parts smaller than alignement"),
                        );
                    }
                } else {
                    break;
                }
            }

            buffer.push('\n');
        }

        print!("{}", buffer);
    }

    pub fn resolutions(&self, resolutions: Vec<Resolution>) {
        self.line('=');
        self.print_centered("Resolution selection");
        self.line('=');
        let mut lengths = Vec::new();
        let mut max_len = 10; //Length of "Resolution"
        for r in resolutions.iter() {
            let len = r.label().chars().count();
            if len > max_len {
                max_len = len;
            }
            lengths.push(len);
        }

        println!(
            "{}Resolution{}Size",
            " ".to_string()
                .repeat(display_length(resolutions.len()) + 2),
            " ".to_string().repeat(max_len - 10 + 2),
        );

        for (id, r) in resolutions.iter().enumerate() {
            let spacing = " ".to_string().repeat(max_len - lengths[id]);
            let colon_spacing = " "
                .to_string()
                .repeat(display_length(resolutions.len()) - display_length(id + 1));
            println!(
                "{}{}: {} {} {}",
                id + 1,
                colon_spacing,
                r.label(),
                spacing,
                pretty_size(*r.size()),
            )
        }
    }

    pub fn welcome(&self, instance: &str) {
        self.line('=');
        self.print_centered(&format!("Connecting to: {}", instance));
        self.line('=');
    }

    pub fn err<T: fmt::Display>(&self, err: &T) {
        println!(
            "{}{}{}{}{}",
            self.fg_color(color::Red),
            style::Bold,
            err,
            style::Reset,
            self.fg_color(color::Reset)
        );
    }

    pub fn message(&self, msg: &str) {
        println!("{}", msg);
    }

    pub fn mode_info(&self, mode: &str, total: Option<usize>, offset: usize, current_len: usize) {
        if let Some(total) = total {
            println!(
                "{}{}{} results {} to {} out of {}{}",
                style::Bold,
                style::Underline,
                mode,
                offset,
                offset + current_len,
                total,
                style::Reset
            );
        } else {
            println!(
                "{}{}{} results {} to {}{}",
                style::Bold,
                style::Underline,
                mode,
                offset,
                offset + current_len,
                style::Reset
            );
        }
    }

    pub async fn info(&self, video: &Video) {
        self.line('=');
        self.print_centered(video.name());
        self.line('=');
        if let Ok(Some(d)) = video.description().await {
            if !d.is_empty() {
                self.print_centered("DESCRIPTION");
                self.line('=');
                println!("{}", d);
                self.line('=');
            }
        }
        self.print_centered("INFORMATIONS");
        self.line('=');
        println!("duration : {}", pretty_duration(video.duration()));
        println!("views    : {}", video.views());
        println!("likes    : {}", video.likes());
        println!("dislikes : {}", video.dislikes());
        println!("released : {}", full_date(video.published()));
        println!("account  : {}", video.account_display());
        println!("channel  : {}", video.channel_display());
        println!("host     : {}", video.host());
        println!("url      : {}", video.watch_url());
        if video.nsfw() {
            println!("{}nsfw{}", self.fg_color(color::Red), style::Reset,);
        }
        self.line('=');
    }

    fn line(&self, c: char) {
        let line_str = c.to_string().repeat(self.cols);
        println!("{}", line_str);
    }

    fn print_centered(&self, s: &str) {
        let len = s.chars().count();
        if len > self.cols {
            println!("{}", s);
            return;
        }

        let before = ' '.to_string().repeat((self.cols - len) / 2);
        println!("{}{}", before, s);
    }
}

fn pretty_size(mut s: u64) -> String {
    const PREFIXES: [&str; 5] = ["", "K", "M", "G", "E"];
    let mut id = 0;
    while s >= 1024 && id < 5 {
        s /= 1024;
        id += 1;
    }

    format!("{}{}B", s, PREFIXES[id])
}

fn display_count(mut c: u64) -> String {
    const PREFIXES: [&str; 5] = ["", "K", "M", "G", "E"];
    let mut id = 0;
    while c >= 1000 && id < 5 {
        c /= 1000;
        id += 1;
    }

    format!("{}{}", c, PREFIXES[id])
}

fn pretty_date(d: Option<&DateTime<FixedOffset>>) -> String {
    let now: DateTime<Utc> = SystemTime::now().into();
    d.map(|t| pretty_duration_since(now.naive_local().signed_duration_since(t.naive_local())))
        .unwrap_or_default()
}

fn full_date(d: Option<&DateTime<FixedOffset>>) -> String {
    d.map(|t| t.format("%a %b %Y").to_string())
        .unwrap_or_default()
}

fn pretty_duration_since(d: Duration) -> String {
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

fn pretty_duration(d: u64) -> String {
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

fn display_length(mut i: usize) -> usize {
    let mut len = 1;
    while i >= 10 {
        len += 1;
        i /= 10;
    }

    len
}

enum VideoLayoutItem {
    Color(Box<dyn fmt::Display>),
    Name,
    Channel,
    Account,
    Host,
    Nsfw,
    Views,
    Likes,
    Duration,
    Alignement,
    Published,
    String(String),
}

impl VideoLayoutItem {
    fn display(&self, v: &Video, colors: bool) -> String {
        match self {
            VideoLayoutItem::Color(c) => {
                if colors {
                    format!("{}", *c)
                } else {
                    String::new()
                }
            }
            VideoLayoutItem::Name => v.name().to_owned(),
            VideoLayoutItem::Channel => v.channel_display().to_owned(),
            VideoLayoutItem::Account => v.account_display().to_owned(),
            VideoLayoutItem::Host => v.host().to_owned(),
            VideoLayoutItem::Nsfw => {
                if v.nsfw() {
                    "nsfw".to_string()
                } else {
                    "".to_string()
                }
            }
            VideoLayoutItem::Views => display_count(v.views()),
            VideoLayoutItem::Likes => display_count(v.likes()),
            VideoLayoutItem::Duration => pretty_duration(v.duration()),
            VideoLayoutItem::Published => pretty_date(v.published()),
            VideoLayoutItem::String(s) => s.clone(),
            VideoLayoutItem::Alignement => {
                panic!("Internal error, trying to display an alignement")
            }
        }
    }

    fn is_align(&self) -> bool {
        if let VideoLayoutItem::Alignement = self {
            true
        } else {
            false
        }
    }

    fn length_matters(&self) -> bool {
        match self {
            VideoLayoutItem::Name
            | VideoLayoutItem::Channel
            | VideoLayoutItem::Account
            | VideoLayoutItem::Host
            | VideoLayoutItem::Nsfw
            | VideoLayoutItem::Views
            | VideoLayoutItem::Likes
            | VideoLayoutItem::Duration
            | VideoLayoutItem::Alignement
            | VideoLayoutItem::Published
            | VideoLayoutItem::String(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod helpers {
    use super::*;

    #[test]
    fn length() {
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
    fn count() {
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
    fn size() {
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
    fn duration() {
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
