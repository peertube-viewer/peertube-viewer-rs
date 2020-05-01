use std::sync::{Arc, Mutex};

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

    pub async fn readline(
        &mut self,
        prompt: String,
    ) -> Result<rustyline::Result<String>, Box<dyn std::error::Error>> {
        let rl_cloned = self.rl.clone();
        Ok(spawn_blocking(move || {
            let mut ed = rl_cloned.lock().unwrap();
            ed.readline(&prompt)
        })
        .await?)
    }

    pub async fn readline_static(
        &mut self,
        prompt: &'static str,
    ) -> Result<rustyline::Result<String>, Box<dyn std::error::Error>> {
        let rl_cloned = self.rl.clone();
        Ok(spawn_blocking(move || {
            let mut ed = rl_cloned.lock().unwrap();
            ed.readline(&prompt)
        })
        .await?)
    }

    pub fn load_history(&mut self, path: &str) -> rustyline::Result<()> {
        let mut ed = self.rl.lock().unwrap();
        ed.load_history(path)
    }

    pub fn save_history(&mut self, path: &str) -> rustyline::Result<()> {
        let ed = self.rl.lock().unwrap();
        ed.save_history(path)
    }

    pub fn add_history_entry(&mut self, entry: &str) -> bool {
        let mut ed = self.rl.lock().unwrap();
        ed.add_history_entry(entry)
    }
}
