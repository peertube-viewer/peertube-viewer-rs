use std::collections::HashSet;

use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::PathBuf;

pub struct History {
    videos: HashSet<String>,
    order: Vec<String>,
}

impl History {
    pub fn new() -> History {
        History {
            videos: HashSet::new(),
            order: Vec::new(),
        }
    }

    pub fn load_file(&mut self, path: &PathBuf) -> Result<(), Error> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut reversed = Vec::new();

        for line in buf_reader.lines() {
            let line_unwrapped = line?;
            if !self.videos.contains(&line_unwrapped) {
                self.videos.insert(line_unwrapped.clone());
                reversed.push(line_unwrapped);
            }
        }
        self.order = reversed.into_iter().rev().collect();
        Ok(())
    }

    pub fn add_video(&mut self, uuid: String) {
        if !self.videos.contains(&uuid) {
            self.videos.insert(uuid.clone());
            self.order.push(uuid);
        }
    }

    pub fn save(&self, path: &PathBuf, max_len: usize) -> Result<(), Error> {
        let mut already_in = HashSet::new();
        let mut full_str = String::new();
        let mut lines = 0;
        for uuid in self.order.iter().rev() {
            if !already_in.contains(uuid) {
                already_in.insert(uuid);
                full_str.push_str(&format!("{}\n", uuid));
                lines += 1;
                if lines == max_len {
                    break;
                }
            }
        }
        std::fs::write(path, &full_str)
    }
}

pub trait HistoryT<D: ?Sized> {
    fn is_viewed(&self, uuid: &D) -> bool;
}

impl HistoryT<str> for History {
    fn is_viewed(&self, uuid: &str) -> bool {
        self.videos.contains(uuid)
    }
}

impl HistoryT<peertube_api::Video> for History {
    fn is_viewed(&self, video: &peertube_api::Video) -> bool {
        self.videos.contains(video.uuid())
    }
}

impl<T: ?Sized> HistoryT<T> for () {
    fn is_viewed(&self, _: &T) -> bool {
        false
    }
}
