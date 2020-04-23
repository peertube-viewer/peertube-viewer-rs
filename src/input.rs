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
        promt: &str,
    ) -> Result<rustyline::Result<String>, Box<dyn std::error::Error>> {
        let rl_cloned = self.rl.clone();
        Ok(spawn_blocking(move || {
            let mut ed = rl_cloned.lock().unwrap();
            ed.readline(">> ")
        })
        .await?)
    }
}
