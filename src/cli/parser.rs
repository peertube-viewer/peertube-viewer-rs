const COMMANDS: [&str; 14] = [
    //Sorted list of available commands
    ":browser",
    ":chandle",
    ":channels",
    ":comments",
    ":h",
    ":help",
    ":info",
    ":n",
    ":next",
    ":p",
    ":previous",
    ":q",
    ":quit",
    ":trending",
];

const COMMANDS_FIRST: [&str; 7] = [
    //Sorted list of available commands
    ":chandle",
    ":channels",
    ":h",
    ":help",
    ":q",
    ":quit",
    ":trending",
];

const NO_ARGS_FIRST_CMDS_WITH_SPACE: [&str; 5] = [":h ", ":help ", ":q ", ":quit ", ":trending "];

const NO_ARGS_CMDS_WITH_SPACE: [&str; 9] = [
    ":h ",
    ":help ",
    ":n ",
    ":next ",
    ":p ",
    ":previous ",
    ":q ",
    ":quit ",
    ":trending ",
];

#[derive(Debug, PartialEq)]
pub enum ParsedQuery {
    Channels(String),
    Chandle(String),
    Info(usize),
    Comments(usize),
    Browser(usize),
    Query(String),
    Id(usize),
    Help,
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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedArgs,
    UnknownCommand,
    MissingArgs,
    ArgTooHigh,
    IdTooHigh,
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
            if *id >= max =>
        {
            Err(ParseError::ArgTooHigh)
        }
        Ok(ParsedQuery::Id(id)) if *id >= max => Err(ParseError::IdTooHigh),
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
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input.starts_with(":channels ") || input == ":channels" {
        Ok(ParsedQuery::Channels(
            input
                .get(9..)
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input.starts_with(":comments ") || input == ":comments" {
        Ok(ParsedQuery::Comments(
            input
                .get(9..)
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string()
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input.starts_with(":browser ") || input == ":browser" {
        Ok(ParsedQuery::Browser(
            input
                .get(8..)
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string()
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input.starts_with(":info ") || input == ":info" {
        Ok(ParsedQuery::Info(
            input
                .get(5..)
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string()
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input == ":trending" {
        Ok(ParsedQuery::Trending)
    } else if input == ":h" || input == ":help" {
        Ok(ParsedQuery::Help)
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

pub fn parse_first(input: &str) -> Result<ParsedQuery, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if !input.starts_with(':') {
        Ok(ParsedQuery::Query(input.to_string()))
    } else if input.starts_with(":chandle ") || input == ":chandle" {
        Ok(ParsedQuery::Chandle(
            input
                .get(8..)
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input.starts_with(":channels ") || input == ":channels" {
        Ok(ParsedQuery::Channels(
            input
                .get(9..)
                .and_then(clean_spaces)
                .ok_or(ParseError::MissingArgs)?
                .to_string(),
        ))
    } else if input == ":trending" {
        Ok(ParsedQuery::Trending)
    } else if input == ":h" || input == ":help" {
        Ok(ParsedQuery::Help)
    } else if input == ":q" || input == ":quit" {
        Ok(ParsedQuery::Quit)
    } else {
        for cmd in &NO_ARGS_CMDS_WITH_SPACE {
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

pub fn parse_id(input: &str) -> Result<ParsedQuery, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if let Ok(id) = input.parse::<usize>() {
        return Ok(ParsedQuery::Id(id));
    }

    Err(ParseError::ExpectId)
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
    use ParseError::*;
    use ParsedQuery::*;

    fn is_sorted<T: Ord>(input: &[T]) -> bool {
        for i in 0..(input.len() - 1) {
            if input[i] > input[i + 1] {
                return false;
            }
        }

        return true;
    }

    #[test]
    fn normal() {
        assert_eq!(parse(""), Err(Empty));
        assert_eq!(parse("eazazd"), Ok(Query(String::from("eazazd"))));
        assert_eq!(parse(":channels foo"), Ok(Channels(String::from("foo"))));
        assert_eq!(parse(":channels foo"), Ok(Channels(String::from("foo"))));
        assert_eq!(parse(":trending"), Ok(Trending));
        assert_eq!(parse(":help"), Ok(Help));
        assert_eq!(parse(":h"), Ok(Help));
        assert_eq!(parse(":info eazeaz"), Err(BadArgType));
        assert_eq!(parse(":comments eazeaz"), Err(BadArgType));
        assert_eq!(parse(":comments 12"), Ok(Comments(12)));
        assert_eq!(parse(":info 12"), Ok(Info(12)));
        assert_eq!(parse(":browser 12"), Ok(Browser(12)));
        assert_eq!(parse(":info"), Err(MissingArgs));
        assert_eq!(parse(":browser"), Err(MissingArgs));
        assert_eq!(parse(":next"), Ok(Next));
        assert_eq!(parse(":previous"), Ok(Previous));
        assert_eq!(parse(":n"), Ok(Next));
        assert_eq!(parse(":p"), Ok(Previous));
        assert_eq!(parse("12"), Ok(Id(12)));
        assert_eq!(parse("110"), Ok(Id(110)));
    }

    #[test]
    fn first() {
        assert_eq!(parse_first(""), Err(Empty));
        assert_eq!(parse_first("eazazd"), Ok(Query(String::from("eazazd"))));
        assert_eq!(
            parse_first(":channels foo"),
            Ok(Channels(String::from("foo")))
        );
        assert_eq!(parse_first(":trending"), Ok(Trending));
        assert_eq!(parse_first(":help"), Ok(Help));
        assert_eq!(parse_first(":h"), Ok(Help));
        assert_eq!(parse_first(":info eazeaz"), Err(UnknownCommand));
        assert_eq!(parse_first(":info 12"), Err(UnknownCommand));
        assert_eq!(parse_first(":browser 12"), Err(UnknownCommand));
        assert_eq!(parse_first(":info"), Err(UnknownCommand));
        assert_eq!(parse_first(":browser"), Err(UnknownCommand));
        assert_eq!(parse_first(":next"), Err(UnknownCommand));
        assert_eq!(parse_first(":previous"), Err(UnknownCommand));
        assert_eq!(parse_first(":n"), Err(UnknownCommand));
        assert_eq!(parse_first(":p"), Err(UnknownCommand));
        assert_eq!(parse_first("12"), Ok(Query(String::from("12"))));
        assert_eq!(parse_first("110"), Ok(Query(String::from("110"))));
    }

    #[test]
    fn static_data() {
        assert!(is_sorted(&COMMANDS));
        assert!(is_sorted(&COMMANDS_FIRST));
        assert!(is_sorted(&NO_ARGS_FIRST_CMDS_WITH_SPACE));
        assert!(is_sorted(&NO_ARGS_CMDS_WITH_SPACE));
    }

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

    #[test]
    fn id() {
        assert_eq!(parse_id(""), Err(Empty));
        assert_eq!(parse_id(" "), Err(ExpectId));
        assert_eq!(parse_id("  "), Err(ExpectId));
        assert_eq!(parse_id(" aeaeaz "), Err(ExpectId));
        assert_eq!(parse_id("1"), Ok(Id(1)));
        assert_eq!(parse_id("10"), Ok(Id(10)));
        assert_eq!(parse_id("119"), Ok(Id(119)));
    }
}
