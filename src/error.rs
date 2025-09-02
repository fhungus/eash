#[derive(Debug)]
pub enum EASHError {
    FlushFaliure,
    MalformedElement,
    ColorNotFlat,
    IOError(std::io::Error),
    ConfigSyntaxError(String),
    ConfigMalformedBracket(String),
    ConfigInvalidType { expected: &'static str, got: String },
    ConfigPromptUsed,
}

impl From<std::io::Error> for EASHError {
    fn from(error: std::io::Error) -> EASHError {
        EASHError::IOError(error)
    }
}
