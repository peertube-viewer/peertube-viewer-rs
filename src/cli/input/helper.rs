use crate::cli::display::fg_color;
use crate::cli::parser::{info, parse, ParseError, ParsedQuery};

use termion::{color, style};

use std::borrow::Cow;
use std::sync::mpsc::{channel, Receiver, Sender};

use rustyline::{
    completion::{Candidate, Completer},
    highlight::Highlighter,
    hint::Hinter,
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
    use_color: bool,
}

impl Helper {
    pub fn new(use_color: bool) -> (Receiver<Message>, Sender<Message>, Helper) {
        let (tx, rx) = channel();
        (
            rx,
            tx.clone(),
            Helper {
                sender: tx,
                high_limit: None,
                use_color,
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
        if let Ok((0, cmds)) = self.complete(line, pos, ctx) {
            if !cmds.is_empty() {
                return Some(cmds[0].0[pos..].to_string());
            }
        }

        None
    }
}

impl Validator for Helper {}

pub struct CompletionCandidate(&'static str);

impl Candidate for CompletionCandidate {
    fn display(&self) -> &str {
        self.0
    }

    fn replacement(&self) -> &str {
        self.0
    }
}

impl Completer for Helper {
    type Candidate = CompletionCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context,
    ) -> rustyline::Result<(usize, Vec<CompletionCandidate>)> {
        if pos == 0 {
            return Ok((0, vec![]));
        }

        if let Err(ParseError::IncompleteCommand(cmds)) = parse(line) {
            Ok((
                0,
                cmds.into_iter().map(|i| CompletionCandidate(i)).collect(),
            ))
        } else {
            Ok((0, vec![]))
        }
    }
}

fn green_then_bold(line: &str, use_colors: bool) -> Cow<'_, str> {
    if let Some(idx) = line.find(' ') {
        Cow::Owned(format!(
            "{}{}{}{}{}{}{}",
            style::Bold,
            fg_color(color::Green, use_colors),
            &line[..idx],
            style::Reset,
            style::Bold,
            &line[idx..],
            style::Reset,
        ))
    } else {
        Cow::Owned(format!(
            "{}{}{}{}",
            style::Bold,
            fg_color(color::Green, use_colors),
            line,
            style::Reset,
        ))
    }
}

fn green_then_red(line: &str, use_colors: bool) -> Cow<'_, str> {
    if let Some(idx) = line.find(' ') {
        Cow::Owned(format!(
            "{}{}{}{}{}{}",
            style::Bold,
            fg_color(color::Green, use_colors),
            &line[..idx],
            fg_color(color::Red, use_colors),
            &line[idx..],
            style::Reset,
        ))
    } else {
        Cow::Owned(format!(
            "{}{}{}{}",
            style::Bold,
            fg_color(color::Green, use_colors),
            line,
            style::Reset,
        ))
    }
}

fn yellow(line: &str, use_colors: bool) -> Cow<'_, str> {
    Cow::Owned(format!(
        "{}{}{}{}",
        style::Bold,
        fg_color(color::Yellow, use_colors),
        line,
        style::Reset,
    ))
}

fn cyan(line: &str, use_colors: bool) -> Cow<'_, str> {
    Cow::Owned(format!(
        "{}{}{}{}",
        style::Bold,
        fg_color(color::Cyan, use_colors),
        line,
        style::Reset,
    ))
}

fn red(line: &str, use_colors: bool) -> Cow<'_, str> {
    Cow::Owned(format!(
        "{}{}{}{}",
        style::Bold,
        fg_color(color::Red, use_colors),
        line,
        style::Reset,
    ))
}

fn bold(line: &str) -> Cow<'_, str> {
    Cow::Owned(format!("{}{}{}", style::Bold, line, style::Reset,))
}

impl Highlighter for Helper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, _: bool) -> Cow<'b, str> {
        Cow::Owned(format!("{}{}{}", style::Bold, prompt, style::Reset))
    }

    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        match parse(line) {
            Ok(ParsedQuery::Channels(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Chandle(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Info(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Comments(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Browser(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Query(_)) => bold(line),
            Ok(ParsedQuery::Quit) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Trending) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Previous) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Next) => green_then_bold(line, self.use_color),
            Err(ParseError::UnexpectedArgs) | Err(ParseError::BadArgType) => {
                green_then_red(line, self.use_color)
            }
            Err(ParseError::UnknownCommand) => red(line, self.use_color),
            Err(ParseError::MissingArgs) => cyan(line, self.use_color),
            Err(ParseError::IncompleteCommand(_)) => yellow(line, self.use_color),
        }
    }

    fn highlight_char(&self, _: &str, _: usize) -> bool {
        true
    }
}
impl rustyline::Helper for Helper {}
