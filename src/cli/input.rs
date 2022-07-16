// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

mod helper;

use crate::error;
pub use helper::Message;
use helper::{Helper, Stade};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use std::io::{self, Write};
use std::path::Path;

use rustyline::config::{Builder, EditMode};

use super::parser::{filter_high_ids, parse, parse_first, parse_id, ParsedQuery};
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

    pub fn readline(
        &mut self,
        prompt: String,
        limit: Option<usize>,
    ) -> rustyline::Result<ParsedQuery> {
        let mut guard = self.rl.lock().unwrap();
        if let Some(h) = guard.helper_mut() {
            h.set_limit(limit)
        };
        loop {
            let res = guard.readline(&prompt);
            match res {
                Ok(l) => {
                    let parsed = if let Some(id) = limit {
                        filter_high_ids(parse(&l), id)
                    } else {
                        parse(&l)
                    };
                    if let Ok(q) = parsed {
                        return Ok(q);
                    }
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn readline_id(
        &mut self,
        prompt: String,
        limit: Option<usize>,
    ) -> rustyline::Result<usize> {
        let mut guard = self.rl.lock().unwrap();
        if let Some(h) = guard.helper_mut() {
            h.set_limit(limit);
            h.set_stade(Stade::IdOnly);
        }
        loop {
            let res = guard.readline(&prompt);
            match res {
                Ok(l) => {
                    if let Ok(ParsedQuery::Id(id)) = parse_id(&l) {
                        if limit.filter(|max| max > &id && id > 0).is_none() {
                            continue;
                        }
                        return Ok(id);
                    }
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    pub fn first_readline(&mut self, prompt: String) -> rustyline::Result<ParsedQuery> {
        let mut guard = self.rl.lock().unwrap();
        if let Some(h) = guard.helper_mut() {
            h.set_stade(Stade::First);
        };
        loop {
            let res = guard.readline(&prompt);
            match res {
                Ok(l) if !l.is_empty() => {
                    if let Ok(q) = parse_first(&l) {
                        if let Some(h) = guard.helper_mut() {
                            h.set_stade(Stade::Normal);
                        }
                        return Ok(q);
                    }
                    continue;
                }
                Ok(_) => continue,
                Err(e) => {
                    if let Some(h) = guard.helper_mut() {
                        h.set_stade(Stade::Normal);
                    }
                    return Err(e);
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
                h.set_limit(limit);
                h.set_stade(Stade::Normal);
            };
            loop {
                let res = ed.readline(&prompt);
                match res {
                    Ok(line) => {
                        let parsed = if let Some(l) = limit {
                            filter_high_ids(parse(&line), l)
                        } else {
                            parse(&line)
                        };
                        if let Ok(p) = parsed {
                            tx_cloned.send(Message::Over(Ok(p))).unwrap();
                            if let Some(h) = ed.helper_mut() {
                                h.set_limit(None)
                            };
                            break;
                        } else {
                            // TODO add an error message
                            continue;
                        }
                    }
                    Err(e) => tx_cloned.send(Message::Over(Err(e))).unwrap(),
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
    ) -> rustyline::Result<ParsedQuery> {
        let mut handle = self.helped_readline(prompt, Some(list.current().len() + 1));
        loop {
            match handle.next() {
                Message::Over(res) => {
                    return res;
                }
                Message::Unfinnished(ParsedQuery::Next) => list.preload_next(),
                Message::Unfinnished(p) => {
                    if let Some(id) = p.should_preload() {
                        list.preload_id(id - 1);
                    }
                }
            }
        }
    }

    pub fn load_history(&mut self, path: &Path) -> rustyline::Result<()> {
        let mut ed = self.rl.lock().unwrap();
        ed.load_history(path)
    }

    pub fn save_history(&mut self, path: &Path) -> rustyline::Result<()> {
        let mut ed = self.rl.lock().unwrap();
        ed.save_history(path)
    }

    pub fn add_history_entry(&mut self, entry: &str) -> bool {
        let mut ed = self.rl.lock().unwrap();
        ed.add_history_entry(entry)
    }
}
