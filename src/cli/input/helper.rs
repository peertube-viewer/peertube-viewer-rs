use crate::cli::parser::info;

use std::sync::mpsc::{channel, Receiver, Sender};

use rustyline::{
    completion::Completer, highlight::Highlighter, hint::Hinter, line_buffer::LineBuffer,
    validate::Validator, Context,
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
            },
        )
    }

    pub fn set_limit(&mut self, lim: Option<usize>) {
        self.high_limit = lim;
    }
}

impl Hinter for Helper {
    fn hint(&self, line: &str, _: usize, _: &Context) -> Option<String> {
        if line.is_empty() {
            return None;
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

        None
    }
}

impl Validator for Helper {}

impl Completer for Helper {
    type Candidate = String;

    fn update(&self, _line: &mut LineBuffer, _start: usize, _elected: &str) {
        unreachable!()
    }
}

impl Highlighter for Helper {}
impl rustyline::Helper for Helper {}
