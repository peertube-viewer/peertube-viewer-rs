const COMMANDS: [&str; 12] = [
    //Sorted list of available commands
    ":browser",
    ":chandle",
    ":channels",
    ":comments",
    ":info",
    ":n",
    ":next",
    ":p",
    ":previous",
    ":q",
    ":quit",
    ":trending",
];

const COMMANDS_FIRST: [&str; 5] = [
    //Sorted list of available commands
    ":chandle",
    ":channels",
    ":q",
    ":quit",
    ":trending",
];

const NO_ARGS_FIRST_CMDS_WITH_SPACE: [&str; 3] = [":q ", ":quit ", ":trending "];

const NO_ARGS_CMDS_WITH_SPACE: [&str; 7] = [
    ":n ",
    ":next ",
    ":p ",
    ":previous ",
    ":q ",
    ":quit ",
    ":trending ",
];

#[derive(Debug)]
pub enum ParsedQuery {
    Channels(String),
    Chandle(String),
    Info(usize),
    Comments(usize),
    Browser(usize),
    Query(String),
    Id(usize),
    Quit,
    Next,
    Previous,
    Trending,
}

impl ParsedQuery {
    pub fn should_preload(&self) -> Option<usize> {
        match *self {
            ParsedQuery::Info(id) | ParsedQuery::Comments(id) | ParsedQuery::Id(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedArgs,
    UnknownCommand,
    MissingArgs,
    ArgTooHigh,
    IdZero,
    BadArgType,
    ExpectId,
    Empty,
    IncompleteCommand(Vec<&'static str>),
}

pub fn filter_high_ids(
    parsed: Result<ParsedQuery, ParseError>,
    max: usize,
) -> Result<ParsedQuery, ParseError> {
    match &parsed {
        Ok(ParsedQuery::Info(id))
        | Ok(ParsedQuery::Comments(id))
        | Ok(ParsedQuery::Browser(id))
        | Ok(ParsedQuery::Id(id))
            if *id > max =>
        {
            Err(ParseError::ArgTooHigh)
        }
        Ok(ParsedQuery::Info(id))
        | Ok(ParsedQuery::Comments(id))
        | Ok(ParsedQuery::Browser(id))
        | Ok(ParsedQuery::Id(id))
            if *id == 0 =>
        {
            Err(ParseError::IdZero)
        }
        _ => parsed,
    }
}

pub fn parse(input: &str) -> Result<ParsedQuery, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Ok(id) = input.parse::<usize>() {
        return Ok(ParsedQuery::Id(id));
    } else if !input.starts_with(':') {
        return Ok(ParsedQuery::Query(input.to_string()));
    }

    if input.starts_with(":chandle ") || input == ":chandle" {
        Ok(ParsedQuery::Chandle(
            input
                .get(8..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input.starts_with(":channels ") || input == ":channels" {
        Ok(ParsedQuery::Channels(
            input
                .get(9..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input.starts_with(":comments ") || input == ":comments" {
        Ok(ParsedQuery::Comments(
            input
                .get(8..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string()
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input.starts_with(":browser ") || input == ":browser" {
        Ok(ParsedQuery::Browser(
            input
                .get(8..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string()
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input.starts_with(":info ") || input == ":info" {
        Ok(ParsedQuery::Info(
            input
                .get(5..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string()
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input == ":trending" {
        Ok(ParsedQuery::Trending)
    } else if input == ":q" || input == ":quit" {
        Ok(ParsedQuery::Quit)
    } else if input == ":p" || input == ":previous" {
        Ok(ParsedQuery::Previous)
    } else if input == ":n" || input == ":next" {
        Ok(ParsedQuery::Next)
    } else {
        for cmd in &NO_ARGS_FIRST_CMDS_WITH_SPACE {
            if input.starts_with(cmd) {
                return Err(ParseError::UnexpectedArgs);
            }
        }

        let starts = COMMANDS_FIRST.iter().fold(Vec::new(), |mut acc, cmd| {
            if cmd.starts_with(input) {
                acc.push(*cmd);
            }
            acc
        });

        if !starts.is_empty() {
            Err(ParseError::IncompleteCommand(starts))
        } else {
            Err(ParseError::UnknownCommand)
        }
    }
}

pub fn parse_first(input: &str) -> Result<ParsedQuery, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Ok(id) = input.parse::<usize>() {
        return Ok(ParsedQuery::Id(id));
    } else if !input.starts_with(':') {
        return Ok(ParsedQuery::Query(input.to_string()));
    }

    if input.starts_with(":chandle ") || input == ":chandle" {
        Ok(ParsedQuery::Chandle(
            input
                .get(8..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input.starts_with(":channels ") || input == ":channels" {
        Ok(ParsedQuery::Channels(
            input
                .get(9..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input == ":trending" {
        Ok(ParsedQuery::Trending)
    } else if input == ":q" || input == ":quit" {
        Ok(ParsedQuery::Quit)
    } else {
        for cmd in &NO_ARGS_CMDS_WITH_SPACE {
            if input.starts_with(cmd) {
                return Err(ParseError::UnexpectedArgs);
            }
        }

        let starts = COMMANDS.iter().fold(Vec::new(), |mut acc, cmd| {
            if cmd.starts_with(input) {
                acc.push(*cmd);
            }
            acc
        });

        if !starts.is_empty() {
            Err(ParseError::IncompleteCommand(starts))
        } else {
            Err(ParseError::UnknownCommand)
        }
    }
}

pub fn parse_id(input: &str) -> Result<ParsedQuery, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Ok(id) = input.parse::<usize>() {
        return Ok(ParsedQuery::Id(id));
    }

    return Err(ParseError::ExpectId);
}

pub fn clean_spaces(input: &str) -> Option<&str> {
    let mut start: usize = 0;
    let mut chars = input.chars();
    while chars.next() == Some(' ') {
        start += 1;
    }

    let mut end = input.len();

    while chars.next_back() == Some(' ') {
        end -= 1;
    }

    if start < end {
        Some(&input[start..end])
    } else {
        None
    }
}

#[cfg(test)]
mod parser {
    use super::*;

    #[test]
    fn test_spaces() {
        assert_eq!(clean_spaces(""), None);
        assert_eq!(clean_spaces(" "), None);
        assert_eq!(clean_spaces("  "), None);
        assert_eq!(clean_spaces(" aeaeaz "), Some("aeaeaz"));
        assert_eq!(clean_spaces("eaz"), Some("eaz"));
        assert_eq!(clean_spaces(" eaze"), Some("eaze"));
        assert_eq!(clean_spaces("eae "), Some("eae"));
        assert_eq!(clean_spaces("1"), Some("1"));
    }
}
