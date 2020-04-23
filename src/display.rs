use peertube_api::Video;

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
                pretty_duration(v.duration()),
                pretty_date(v.published())
            )
        }
    }

    pub async fn info(&self, video: &Video) {
        self.line('=');
        self.print_centered(video.name());
        self.line('=');
        if let Ok(Some(d)) = video.description().await {
            self.print_centered("DESCRIPTION");
            self.line('=');
            println!("{}", d);
            self.line('=');
        }
        self.print_centered("INFORMATIONS");
        self.line('=');
        println!("duration  : {}", pretty_duration(video.duration()));
        //println!("views    : {}",video.get_views());
        //println!("likes    : {}",video.get_likes());
        //println!("dislikes : {}",video.get_dislikes());
        println!("released  : {}", pretty_date(video.published()));
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

fn pretty_date(d: &Option<DateTime<FixedOffset>>) -> String {
    d.map(|t| t.format("%a:%b:%Y").to_string())
        .unwrap_or_default()
}

fn pretty_duration(d: u64) -> String {
    if d < 60 {
        format!("{}", d)
    } else if d < 3600 {
        format!("{}:{}", d / 60, d % 60)
    } else {
        format!("{}:{}:{}", d / 3600, (d % 3600) / 60, d % 60)
    }
}

fn display_length(mut i: usize) -> usize {
    let mut len = 0;
    while i >= 10 {
        len += 1;
        i /= 10;
    }

    len
}
