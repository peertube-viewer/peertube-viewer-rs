use std::sync::{Arc, Mutex};

use std::path::PathBuf;
use tokio::task::spawn_blocking;

pub struct Editor {
    rl: Arc<Mutex<rustyline::Editor<()>>>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            rl: Arc::new(Mutex::new(rustyline::Editor::<()>::new())),
        }
    }

    pub async fn readline(&mut self, prompt: String) -> rustyline::Result<String> {
        let rl_cloned = self.rl.clone();
        spawn_blocking(move || {
            let mut ed = rl_cloned.lock().unwrap();
            loop {
                match ed.readline(&prompt) {
                    Ok(l) if l != "" => return Ok(l),
                    Ok(_) => continue,
                    e @ Err(_) => return e,
                }
            }
        })
        .await
        .expect("readline thread panicked")
    }

    pub fn load_history(&mut self, path: &PathBuf) -> rustyline::Result<()> {
        let mut ed = self.rl.lock().unwrap();
        ed.load_history(path)
    }

    pub fn save_history(&mut self, path: &PathBuf) -> rustyline::Result<()> {
        let ed = self.rl.lock().unwrap();
        ed.save_history(path)
    }

    pub fn add_history_entry(&mut self, entry: &str) -> bool {
        let mut ed = self.rl.lock().unwrap();
        ed.add_history_entry(entry)
    }
}
