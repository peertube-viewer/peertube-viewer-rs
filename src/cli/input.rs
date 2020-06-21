mod helper;

use helper::Helper;
pub use helper::Message;

use crate::error;

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

use std::io::{self, Write};
use std::path::PathBuf;
use tokio::task::{spawn_blocking, JoinHandle};

use rustyline::config::{Builder, EditMode};

use preloadable_list::{AsyncLoader, PreloadableList};

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
    pub fn new(edit_mode: EditMode) -> Editor {
        let (rx, h) = Helper::new();
        let mut rl = rustyline::Editor::with_config(Builder::new().edit_mode(edit_mode).build());
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

    pub async fn std_in(&mut self, prompt: String) -> Result<String, error::Error> {
        spawn_blocking(move || {
            print!("{}", prompt);
            io::stdout().flush().expect("Unable to print to stdout");
            let mut res = String::new();
            io::stdin()
                .read_line(&mut res)
                .map(|_| res)
                .map_err(error::Error::Stdin)
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

    pub async fn autoload_readline<Loader: AsyncLoader>(
        &mut self,
        prompt: String,
        list: &mut PreloadableList<Loader>,
    ) -> rustyline::Result<Action> {
        let mut handle = self.helped_readline(prompt, Some(list.current().len()));
        loop {
            match handle.next().await {
                Message::Over(res) => {
                    let s = res?;
                    if s == ":n" {
                        return Ok(Action::Next);
                    } else if s == ":p" {
                        return Ok(Action::Prev);
                    } else if s == ":q" {
                        return Ok(Action::Quit);
                    }
                    match s.parse::<usize>() {
                        Ok(id) if id > 0 && id <= list.current_len() => {
                            return Ok(Action::Id(id));
                        }
                        Err(_) | Ok(_) => {
                            return Ok(Action::Query(s));
                        }
                    }
                }

                Message::Number(id) => {
                    list.preload_id(id - 1);
                }
                Message::CommandNext => {
                    list.preload_next();
                }
                Message::CommandPrev => {}
            }
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

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Next,
    Prev,
    Quit,
    Id(usize),
    Query(String),
}
