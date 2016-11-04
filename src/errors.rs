use std::error;

// Error type, imported into main name space
#[derive(Debug)]
pub enum DictError {
    /// the character at which the parser failed an optionally the line number
    ///and position on the line
    InvalidCharacter(char, Option<usize>, Option<usize>),
    /// not enough columns given for specified line
    MissingColumnInIndex(usize),
    /// invalid file format, contains an explanation an an optional path to the
    /// file with the invalid file format
    InvalidFileFormat(String, Option<String>),
    /// if there's not enough memory
    MemoryError,
    /// a wrapped io::Error
    IoError(::std::io::Error),
    /// UTF8 error while reading data
    Utf8Error(::std::string::FromUtf8Error),
}

impl ::std::fmt::Display for DictError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            DictError::IoError(ref e) => e.fmt(f),
            DictError::Utf8Error(ref e) => e.fmt(f),
            DictError::MemoryError => write!(f, "not enough memory available"),
            DictError::InvalidCharacter(ref ch, ref line, ref pos) =>
                write!(f, "Invalid character {}{}{}", ch,
                        match *line {
                            Some(ln) => format!(" on line {}", ln),
                            _ => String::new() // ToDo: more leegant solution
                        },
                        match *pos {
                            Some(pos) => format!(" at position {}", pos),
                            _ => String::new() // ToDo: more leegant solution
                        }),
            DictError::MissingColumnInIndex(ref lnum) => write!(f, "line {}: not \
                    enough <tab>-separated columns found, expected 3", lnum),
            DictError::InvalidFileFormat(ref explanation, ref path) =>
                write!(f, "{}{}", path.clone().unwrap_or_else(String::new), explanation)
        }
    }
}

impl error::Error for DictError {
    fn description(&self) -> &str {
        match *self {
            DictError::InvalidCharacter(_, _, _) => "invalid character",
            DictError::MemoryError => "not enough memory available",
            DictError::MissingColumnInIndex(_) =>
                    "not enough <tab>-separated columnss given",
            DictError::InvalidFileFormat(ref _explanation, ref _path) => "could not \
                    determine file format",
            DictError::IoError(ref err) => err.description(),
            DictError::Utf8Error(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DictError::IoError(ref err) => err.cause(),
            DictError::Utf8Error(ref err) => err.cause(),
            _ => None,
        }
    }
}

/// allow seamless coercion from::Error 
impl From<::std::io::Error> for DictError {
    fn from(err: ::std::io::Error) -> DictError {
        DictError::IoError(err)
    }
}

impl From<::std::string::FromUtf8Error> for DictError {
    fn from(err: ::std::string::FromUtf8Error) -> DictError {
        DictError::Utf8Error(err)
    }
}

