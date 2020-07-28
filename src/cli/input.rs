mod helper;

use crate::error;
use helper::Helper;
pub use helper::Message;

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use std::io::{self, Write};
use std::path::PathBuf;

use rustyline::config::{Builder, EditMode};

use preloadable_list::{AsyncLoader, PreloadableList};

pub struct HelpedHandle<'editor> {
    rx: &'editor mut Receiver<Message>,
}

impl<'editor> HelpedHandle<'editor> {
    pub fn next(&mut self) -> Message {
        self.rx.recv().unwrap()
    }
}

pub struct Editor {
    rx: Receiver<Message>,
    tx: Sender<Message>,
    rl: Arc<Mutex<rustyline::Editor<Helper>>>,
}

impl Editor {
    pub fn new(edit_mode: EditMode, use_color: bool) -> Editor {
        let (rx, tx, h) = Helper::new(use_color);
        let mut rl = rustyline::Editor::with_config(Builder::new().edit_mode(edit_mode).build());
        rl.set_helper(Some(h));
        Editor {
            rx,
            tx,
            rl: Arc::new(Mutex::new(rl)),
        }
    }

    pub fn readline(&mut self, prompt: String) -> rustyline::Result<String> {
        let mut guard = self.rl.lock().unwrap();
        loop {
            match guard.readline(&prompt) {
                Ok(l) if l != "" => return Ok(l),

                Ok(_) => continue,
                e @ Err(_) => return e,
            }
        }
    }
    pub fn first_readline(&mut self, prompt: String) -> rustyline::Result<String> {
        let mut guard = self.rl.lock().unwrap();
        if let Some(h) = guard.helper_mut() {
            h.set_first(true);
        };
        loop {
            match guard.readline(&prompt) {
                Ok(l) if l != "" => {
                    if let Some(h) = guard.helper_mut() {
                        h.set_first(false);
                    }
                    return Ok(l);
                }
                Ok(_) => continue,
                e @ Err(_) => {
                    if let Some(h) = guard.helper_mut() {
                        h.set_first(false);
                    }
                    return e;
                }
            }
        }
    }

    pub fn std_in(&mut self, prompt: String) -> Result<String, error::Error> {
        print!("{}", prompt);
        io::stdout().flush().expect("Unable to print to stdout");
        let mut res = String::new();
        io::stdin()
            .read_line(&mut res)
            .map(|_| res)
            .map_err(error::Error::Stdin)
    }

    pub fn helped_readline(&mut self, prompt: String, limit: Option<usize>) -> HelpedHandle<'_> {
        let rl_cloned = self.rl.clone();
        let tx_cloned = self.tx.clone();
        spawn(move || {
            let mut ed = rl_cloned.lock().unwrap();
            if let Some(h) = ed.helper_mut() {
                h.set_limit(limit)
            };
            loop {
                match ed.readline(&prompt) {
                    Ok(l) if l != "" => tx_cloned.send(Message::Over(Ok(l))).unwrap(),
                    Ok(_) => continue,
                    e @ Err(_) => tx_cloned.send(Message::Over(e)).unwrap(),
                }
                break;
            }
        });
        HelpedHandle { rx: &mut self.rx }
    }

    pub fn autoload_readline<Loader: AsyncLoader>(
        &mut self,
        prompt: String,
        list: &mut PreloadableList<Loader>,
    ) -> rustyline::Result<Action> {
        let mut handle = self.helped_readline(prompt, Some(list.current().len()));
        loop {
            match handle.next() {
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
