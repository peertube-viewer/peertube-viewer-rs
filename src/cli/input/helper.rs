use crate::cli::parser::{info, COMMANDS};

use termion::{color, style};

use std::borrow::Cow;
use std::sync::mpsc::{channel, Receiver, Sender};

use rustyline::{
    completion::Completer,
    highlight::Highlighter,
    hint::{Hinter, HistoryHinter},
    validate::Validator,
    Context,
};

/// Message to interact between the editor and the runtime
#[derive(Debug)]
pub enum Message {
    /// Input completed
    Over(rustyline::Result<String>),

    /// typed :n
    CommandNext,
    /// typed :p
    CommandPrev,

    /// typed a number (guaranteed to be bellow the set limit)
    Number(usize),
}

pub struct Helper {
    sender: Sender<Message>,
    high_limit: Option<usize>,
    hinter: HistoryHinter,
}

impl Helper {
    pub fn new() -> (Receiver<Message>, Sender<Message>, Helper) {
        let (tx, rx) = channel();
        (
            rx,
            tx.clone(),
            Helper {
                sender: tx,
                high_limit: None,
                hinter: HistoryHinter {},
            },
        )
    }

    pub fn set_limit(&mut self, lim: Option<usize>) {
        self.high_limit = lim;
    }

    fn message(&self, line: &str) {
        if line.is_empty() {
            return;
        }

        if line == ":n" {
            self.sender.send(Message::CommandNext).unwrap();
        } else if line == ":p" {
            self.sender.send(Message::CommandPrev).unwrap();
        } else if let (Ok(num), Some(limit)) = (line.parse(), self.high_limit) {
            if num <= limit && num > 0 {
                self.sender.send(Message::Number(num)).unwrap();
            }
        } else if let Some(limit) = self.high_limit {
            if let Some(num) = info(&line, limit) {
                self.sender.send(Message::Number(num)).unwrap();
            }
        }
    }
}

impl Hinter for Helper {
    fn hint(&self, line: &str, pos: usize, ctx: &Context) -> Option<String> {
        self.message(line);
        self.hinter.hint(line, pos, ctx)
    }
}

impl Validator for Helper {}

impl Completer for Helper {
    type Candidate = String;
}

impl Highlighter for Helper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Owned(format!("{}{}{}", style::Bold, prompt, style::Reset))
    }

    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        if !line.is_empty() && line.starts_with(":") {
            if let Some(idx) = line.find(' ') {
                if let Ok(_) = COMMANDS.binary_search(&&line[..idx]) {
                    return Cow::Owned(format!(
                        "{}{}{}{}{}{}{}",
                        style::Bold,
                        color::Fg(color::Green),
                        &line[..idx],
                        style::Reset,
                        style::Bold,
                        &line[idx..],
                        style::Reset,
                    ));
                }
            } else {
                if let Ok(_) = COMMANDS.binary_search(&line) {
                    return Cow::Owned(format!(
                        "{}{}{}{}",
                        style::Bold,
                        color::Fg(color::Green),
                        line,
                        style::Reset,
                    ));
                }
            }
        }
        Cow::Owned(format!("{}{}{}", style::Bold, line, style::Reset))
    }

    fn highlight_char(&self, _: &str, _: usize) -> bool {
        true
    }
}
impl rustyline::Helper for Helper {}
