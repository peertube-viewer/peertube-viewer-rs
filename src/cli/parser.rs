pub const COMMANDS: [&str; 7] = [
    //Sorted list of available commands
    ":browser",
    ":chandle",
    ":channels",
    ":comments",
    ":info",
    ":quit",
    ":trending",
];

pub enum ParsedQuery<'input> {
    Channels(&'input str),
    Chandle(&'input str),
    Info(usize),
    Comments(usize),
    Browser(usize),
    Query(&'input str),
    Quit,
    Trending,
}

pub enum ParseError {
    UnexpectedArgs,
    UnknownCommand,
    MissingArgs,
    BadArgType,
    IncompleteCommand(Vec<&'static str>),
}

pub fn channels(input: &str) -> Option<&str> {
    if input.starts_with(":channels ") {
        Some(clean_spaces(&input[10..])).flatten()
    } else {
        None
    }
}

pub fn parse(input: &str) -> Result<ParsedQuery, ParseError> {
    if !input.starts_with(':') {
        return Ok(ParsedQuery::Query(input));
    }

    if input.starts_with(":chandle ") {
        Ok(ParsedQuery::Chandle(
            clean_spaces(&input[9..]).ok_or(ParseError::MissingArgs)?,
        ))
    } else if input.starts_with(":channels ") {
        Ok(ParsedQuery::Channels(
            clean_spaces(&input[10..]).ok_or(ParseError::MissingArgs)?,
        ))
    } else if input.starts_with(":comments ") {
        Ok(ParsedQuery::Comments(
            clean_spaces(&input[9..])
                .ok_or(ParseError::MissingArgs)?
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input.starts_with(":browser ") {
        Ok(ParsedQuery::Browser(
            clean_spaces(&input[8..])
                .ok_or(ParseError::MissingArgs)?
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input.starts_with(":info ") {
        Ok(ParsedQuery::Info(
            clean_spaces(&input[6..])
                .ok_or(ParseError::MissingArgs)?
                .parse()
                .map_err(|_| ParseError::BadArgType)?,
        ))
    } else if input == ":trending" {
        Ok(ParsedQuery::Trending)
    } else if input.starts_with(":trending ") {
        Err(ParseError::UnexpectedArgs)
    } else if input == ":q" || input == ":quit" {
        Ok(ParsedQuery::Quit)
    } else if input.starts_with(":q ") || input.starts_with(":quit ") {
        Err(ParseError::UnexpectedArgs)
    } else {
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
