//! Parsing client commands. Plain synchronous code: keeping it out of the
//! networking code means it can be unit-tested without any sockets.

/// The longest line a client may send, in bytes, not counting the newline.
pub const MAX_LINE: usize = 1024;

/// One line from a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `NICK <name>`: join the chat as `name`.
    Nick(String),
    /// `MSG <text>`: say something to everyone.
    Msg(String),
    /// `WHO`: list the names in use.
    Who,
    /// `QUIT`: leave.
    Quit,
    /// Anything else.
    Unknown,
}

impl Command {
    /// Parses one line (without its newline). Command words are upper case.
    pub fn parse(line: &str) -> Command {
        let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
        match word {
            "NICK" => Command::Nick(rest.to_owned()),
            "MSG" => Command::Msg(rest.to_owned()),
            "WHO" if rest.is_empty() => Command::Who,
            "QUIT" if rest.is_empty() => Command::Quit,
            _ => Command::Unknown,
        }
    }
}

/// A name is 1 to 16 ASCII letters, digits, `_` or `-`.
pub fn valid_name(name: &str) -> bool {
    (1..=16).contains(&name.len())
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_commands() {
        assert_eq!(Command::parse("NICK alice"), Command::Nick("alice".into()));
        assert_eq!(Command::parse("NICK"), Command::Nick("".into()));
        assert_eq!(Command::parse("MSG hello there"), Command::Msg("hello there".into()));
        assert_eq!(Command::parse("WHO"), Command::Who);
        assert_eq!(Command::parse("QUIT"), Command::Quit);
        assert_eq!(Command::parse("QUIT now"), Command::Unknown);
        assert_eq!(Command::parse("msg hi"), Command::Unknown);
        assert_eq!(Command::parse(""), Command::Unknown);
    }

    #[test]
    fn checks_names() {
        assert!(valid_name("alice"));
        assert!(valid_name("Bob_2-x"));
        assert!(!valid_name(""));
        assert!(!valid_name("two words"));
        assert!(!valid_name("émile"));
        assert!(!valid_name("seventeen-chars-x"));
    }
}
