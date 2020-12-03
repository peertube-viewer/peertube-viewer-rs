use crate::cli::display::fg_color;
use crate::cli::parser::{filter_high_ids, parse, parse_first, parse_id, ParseError, ParsedQuery};

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
    Over(rustyline::Result<ParsedQuery>),
    Unfinnished(ParsedQuery),
}

#[derive(PartialEq)]
pub enum Stade {
    First,
    IdOnly,
    Normal,
}

pub struct Helper {
    sender: Sender<Message>,
    high_limit: Option<usize>,
    use_color: bool,
    stade: Stade,
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
                stade: Stade::First,
            },
        )
    }

    pub fn set_limit(&mut self, lim: Option<usize>) {
        self.high_limit = lim;
    }

    fn message(&self, line: &str) {
        if let Ok(p) = self.parse(line) {
            if self.stade == Stade::Normal {
                self.sender.send(Message::Unfinnished(p)).unwrap();
            }
        }
    }

    pub fn parse(&self, line: &str) -> Result<ParsedQuery, ParseError> {
        let parsed = match self.stade {
            Stade::First => parse_first(line),
            Stade::IdOnly => parse_id(line),
            Stade::Normal => parse(line),
        };

        if let Some(max) = self.high_limit {
            filter_high_ids(parsed, max)
        } else {
            parsed
        }
    }

    pub fn set_stade(&mut self, stade: Stade) {
        self.stade = stade;
    }
}

impl Hinter for Helper {
    type Hint = String;

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

        if let Err(ParseError::IncompleteCommand(cmds)) = self.parse(line) {
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
        match self.parse(line) {
            Ok(ParsedQuery::Channels(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Chandle(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Info(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Comments(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Browser(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Query(_)) => bold(line),
            Ok(ParsedQuery::Id(_)) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Quit) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Help) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Trending) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Previous) => green_then_bold(line, self.use_color),
            Ok(ParsedQuery::Next) => green_then_bold(line, self.use_color),
            Err(ParseError::UnexpectedArgs)
            | Err(ParseError::BadArgType)
            | Err(ParseError::ArgTooHigh) => green_then_red(line, self.use_color),
            Err(ParseError::UnknownCommand)
            | Err(ParseError::ExpectId)
            | Err(ParseError::IdTooHigh)
            | Err(ParseError::IdZero) => red(line, self.use_color),
            Err(ParseError::MissingArgs) => cyan(line, self.use_color),
            Err(ParseError::Empty) => Cow::Borrowed(line),
            Err(ParseError::IncompleteCommand(_)) => yellow(line, self.use_color),
        }
    }

    fn highlight_char(&self, _: &str, _: usize) -> bool {
        true
    }
}
impl rustyline::Helper for Helper {}
