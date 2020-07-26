use crate::cli::parser::info;

use rustyline::hint::HistoryHinter;
use std::sync::mpsc::{channel, Receiver, Sender};

use rustyline::{
    completion::Completer, highlight::Highlighter, hint::Hinter, validate::Validator, Context,
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

impl Highlighter for Helper {}
impl rustyline::Helper for Helper {}
