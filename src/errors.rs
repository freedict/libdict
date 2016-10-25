use std::error;

// Error class, imported into main name space
#[derive(Debug)]
pub enum DictError {
    /// the character at which the parser failed an optionally the line number
    ///and position on the line
    InvalidCharacter(char, Option<usize>, Option<usize>),
    /// not enough columns given for specified line
    MissingColumnInIndex(usize),
    IoError(::std::io::Error)
}

impl ::std::fmt::Display for DictError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            DictError::IoError(ref e) => e.fmt(f),
            DictError::InvalidCharacter(ch, line, pos) =>
                write!(f, "Invalid character {}{}{}", ch,
                        match line {
                            Some(ln) => format!(" on line {}", ln),
                            _ => String::new() // ToDo: more leegant solution
                        },
                        match pos {
                            Some(pos) => format!(" at position {}", pos),
                            _ => String::new() // ToDo: more leegant solution
                        }),
            DictError::MissingColumnInIndex(lnum) => write!(f, "line {}: not \
                    enough <tab>-separated columns found, expected 3", lnum),
        }
    }
}

impl error::Error for DictError {
    fn description(&self) -> &str {
        match *self {
            DictError::InvalidCharacter(_, _, _) => "invalid character",
            DictError::MissingColumnInIndex(_) =>
                    "not enough <tab>-separated columnss given",
            DictError::IoError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DictError::IoError(ref err) => err.cause(),
            DictError::InvalidCharacter(_, _, _) => None,
            DictError::MissingColumnInIndex(_) => None
        }
    }
}

/// allow seamless coercion fromo::Error 
impl From<::std::io::Error> for DictError {
    fn from(err: ::std::io::Error) -> DictError {
    DictError::IoError(err)
    }
}
