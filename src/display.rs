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

    pub fn search_results(&self, videos: &Vec<Rc<Video>>) {
        for (id, v) in videos.iter().enumerate() {
            println!(
                "{}: {}  [{}] {}",
                id + 1,
                v.name(),
                v.duration(),
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
        println!("duration : {}", video.duration());
        //println!("views    : {}",video.get_views());
        //println!("likes    : {}",video.get_likes());
        //println!("dislikes : {}",video.get_dislikes());
        println!("released  : {}", pretty_date(video.published()));
        //println!("channel  : {}",video.get_channel_name());
        //println!("author   : {}",video.get_author_name());
        self.line('=');
    }

    fn line(&self, c: char) {
        let line_str = vec![c; self.cols].iter().collect::<String>();
        println!("{}", line_str);
    }

    fn print_centered(&self, s: &str) {
        let len = s.chars().count();
        if len > self.cols {
            println!("{}", s);
            return;
        }

        let mut before = vec![' '; (self.cols - len) / 2].iter().collect::<String>();
        println!("{}{}", before, s);
    }
}

fn pretty_date(d: &Option<DateTime<FixedOffset>>) -> String {
    d.map(|t| t.format("%a:%b:%Y").to_string())
        .unwrap_or_default()
}
