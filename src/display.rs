use peertube_api::{Resolution, Video};

use chrono::{DateTime, FixedOffset};
use termion::{color, style};

use std::rc::Rc;

const DEFAULT_COLS: usize = 20;
const DEFAULT_ROWS: usize = 20;

pub struct Display {
    cols: usize,
    rows: usize,
}

impl Display {
    pub fn new() -> Display {
        let (cols, rows) = termion::terminal_size()
            .map(|(c, r)| (c as usize, r as usize))
            .unwrap_or((DEFAULT_COLS, DEFAULT_ROWS));
        Display { cols, rows }
    }

    pub fn search_results(&self, videos: &[Rc<Video>]) {
        let mut lengths = Vec::new();
        let mut max_len = 0;
        for v in videos.iter() {
            let len = v.name().chars().count();
            if len > max_len {
                max_len = len;
            }
            lengths.push(len);
        }

        for (id, v) in videos.iter().enumerate() {
            let spacing = " ".to_string().repeat(max_len - lengths[id]);
            let colon_spacing = " "
                .to_string()
                .repeat(display_length(videos.len() - 1) - display_length(id + 1));
            println!(
                "{}{}: {} {}[{}] {}",
                id + 1,
                colon_spacing,
                v.name(),
                spacing,
                pretty_duration(*v.duration()),
                pretty_date(v.published())
            )
        }
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
        println!("duration : {}", pretty_duration(*video.duration()));
        println!("views    : {}", video.views());
        println!("likes    : {}", video.likes());
        println!("dislikes : {}", video.dislikes());
        println!("released : {}", pretty_date(video.published()));
        println!("account  : {}", video.account_display());
        println!("channel  : {}", video.channel_display());
        println!("host     : {}", video.host());
        //println!("channel  : {}",video.get_channel_name());
        //println!("author   : {}",video.get_author_name());
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

fn pretty_size(s: u64) -> String {
    format!("{}", s)
}

fn pretty_date(d: &Option<DateTime<FixedOffset>>) -> String {
    d.map(|t| t.format("%a %b %Y").to_string())
        .unwrap_or_default()
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
}
