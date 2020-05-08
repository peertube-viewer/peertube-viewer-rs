use std::collections::HashSet;
use std::rc::Rc;

use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::PathBuf;

pub struct History {
    videos: HashSet<Rc<String>>,
    order: Vec<Rc<String>>, //Avoids duplication of keys
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
                let to_hash = Rc::new(line_unwrapped);
                let to_order = to_hash.clone();
                self.videos.insert(to_hash);
                reversed.push(to_order);
            }
        }
        self.order = reversed.into_iter().rev().collect();
        Ok(())
    }

    pub fn add_video(&mut self, uuid: String) {
        if !self.videos.contains(&uuid) {
            let to_hash = Rc::new(uuid);
            let to_order = to_hash.clone();
            self.videos.insert(to_hash);
            self.order.push(to_order);
        }
    }

    #[allow(clippy::ptr_arg)]
    pub fn is_viewed(&self, uuid: &String) -> bool {
        self.videos.contains(uuid)
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
