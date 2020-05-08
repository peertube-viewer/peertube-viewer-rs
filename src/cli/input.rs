mod helper;

use helper::Helper;
pub use helper::Message;

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

use std::path::PathBuf;
use tokio::task::{spawn_blocking, JoinHandle};

use futures::{
    future::{Fuse, FutureExt}, // for `.fuse()`
    pin_mut,
    select,
};

pub struct HelpedHandle<'editor> {
    rx: &'editor mut UnboundedReceiver<Message>,
    editor_handle: Fuse<JoinHandle<rustyline::Result<String>>>,
}

impl<'editor> HelpedHandle<'editor> {
    pub async fn next(&mut self) -> Message {
        let recv_fut = self.rx.recv().fuse();
        let mut ed = Pin::new(&mut self.editor_handle);
        pin_mut!(recv_fut);
        select! {
            msg = recv_fut => msg.unwrap(),
            res = ed => Message::Over(res.expect("Readline thread panicked")),
        }
    }
}

pub struct Editor {
    rx: UnboundedReceiver<Message>,
    rl: Arc<Mutex<rustyline::Editor<Helper>>>,
}

impl Editor {
    pub fn new() -> Editor {
        let (rx, h) = Helper::new();
        let mut rl = rustyline::Editor::new();
        rl.set_helper(Some(h));
        Editor {
            rx,
            rl: Arc::new(Mutex::new(rl)),
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

    pub fn helped_readline(&mut self, prompt: String, limit: Option<usize>) -> HelpedHandle<'_> {
        let rl_cloned = self.rl.clone();
        HelpedHandle {
            rx: &mut self.rx,
            editor_handle: spawn_blocking(move || {
                let mut ed = rl_cloned.lock().unwrap();
                if let Some(h) = ed.helper_mut() {
                    h.set_limit(limit)
                };
                loop {
                    match ed.readline(&prompt) {
                        Ok(l) if l != "" => return Ok(l),
                        Ok(_) => continue,
                        e @ Err(_) => return e,
                    }
                }
            })
            .fuse(),
        }
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
