pub const COMMANDS: [&str; 12] = [
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
pub enum ParsedQuery<'input> {
    Channels(&'input str),
    Chandle(&'input str),
    Info(usize),
    Comments(usize),
    Browser(usize),
    Query(&'input str),
    Quit,
    Next,
    Previous,
    Trending,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedArgs,
    UnknownCommand,
    MissingArgs,
    BadArgType,
    IncompleteCommand(Vec<&'static str>),
}

pub fn channels(input: &str) -> Option<&str> {
    if input.starts_with(":channels") {
        Some(clean_spaces(&input[9..])).flatten()
    } else {
        None
    }
}

pub fn parse(input: &str) -> Result<ParsedQuery, ParseError> {
    if !input.starts_with(':') {
        return Ok(ParsedQuery::Query(input));
    }

    if input.starts_with(":chandle ") || input == ":chandle" {
        Ok(ParsedQuery::Chandle(
            input
                .get(8..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?,
        ))
    } else if input.starts_with(":channels ") || input == ":channels" {
        Ok(ParsedQuery::Channels(
            input
                .get(9..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?,
        ))
    } else if input.starts_with(":comments ") || input == ":comments" {
        Ok(ParsedQuery::Comments(
            input
                .get(8..)
                .map(clean_spaces)
                .flatten()
                .ok_or(ParseError::MissingArgs)?
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

pub fn chandle(input: &str) -> Option<&str> {
    if input.starts_with(":chandle ") {
        Some(clean_spaces(&input[9..])).flatten()
    } else {
        None
    }
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

pub fn info(input: &str, max: usize) -> Option<usize> {
    if input.starts_with(":info ") {
        let res = clean_spaces(&input[6..])?.parse().ok()?;
        if 0 < res && res <= max {
            Some(res)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn comments(input: &str, max: usize) -> Option<usize> {
    if input.starts_with(":comments ") {
        let res = clean_spaces(&input[9..])?.parse().ok()?;
        if 0 < res && res <= max {
            Some(res)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn browser(input: &str, max: usize) -> Option<usize> {
    if input.starts_with(":browser ") {
        let res = clean_spaces(&input[8..])?.parse().ok()?;
        if 0 < res && res <= max {
            Some(res)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod parser {
    use super::*;

    #[test]
    fn test_channels() {
        assert_eq!(channels("eaz"), None);
        assert_eq!(channels(":channels"), None);
        assert_eq!(channels(":channels "), None);
        assert_eq!(channels(":channels test"), Some("test"));
        assert_eq!(channels(":channels test "), Some("test"));
        assert_eq!(channels(":channels  test"), Some("test"));
        assert_eq!(channels(":channels  test "), Some("test"));
    }

    #[test]
    fn test_chandle() {
        assert_eq!(chandle("eaz"), None);
        assert_eq!(chandle(":chandle"), None);
        assert_eq!(chandle(":chandle "), None);
        assert_eq!(chandle(":chandle test"), Some("test"));
        assert_eq!(chandle(":chandle test "), Some("test"));
        assert_eq!(chandle(":chandle  test"), Some("test"));
        assert_eq!(chandle(":chandle  test "), Some("test"));
    }

    #[test]
    fn test_info() {
        assert_eq!(info("eaz", 20), None);
        assert_eq!(info(":info", 20), None);
        assert_eq!(info(":info ", 20), None);
        assert_eq!(info(":info 18", 20), Some(18));
        assert_eq!(info(":info 20 ", 20), Some(20));
        assert_eq!(info(":info  21", 20), None);
        assert_eq!(info(":info  0 ", 20), None);
        assert_eq!(info(":info  1", 20), Some(1));
    }

    #[test]
    fn test_comments() {
        assert_eq!(comments("eaz", 20), None);
        assert_eq!(comments(":comments", 20), None);
        assert_eq!(comments(":comments ", 20), None);
        assert_eq!(comments(":comments 18", 20), Some(18));
        assert_eq!(comments(":comments 20 ", 20), Some(20));
        assert_eq!(comments(":comments  21", 20), None);
        assert_eq!(comments(":comments  0 ", 20), None);
        assert_eq!(comments(":comments  1", 20), Some(1));
    }

    #[test]
    fn test_browser() {
        assert_eq!(browser("eaz", 20), None);
        assert_eq!(browser(":browser", 20), None);
        assert_eq!(browser(":browser ", 20), None);
        assert_eq!(browser(":browser 18", 20), Some(18));
        assert_eq!(browser(":browser 20 ", 20), Some(20));
        assert_eq!(browser(":browser  21", 20), None);
        assert_eq!(browser(":browser  0 ", 20), None);
        assert_eq!(browser(":browser  1", 20), Some(1));
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
}
