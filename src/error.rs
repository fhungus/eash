use crate::evaluate::Token;

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
    DumbassDevForgotToHandleThis(EASHUncomfortable) // 💀
}

#[derive(Debug)]
pub enum EASHUncomfortable { // non-fatal error
    CommandStartedWithoutProgram(Token),
}

impl From<std::io::Error> for EASHError {
    fn from(error: std::io::Error) -> EASHError {
        EASHError::IOError(error)
    }
}

impl From<EASHUncomfortable> for EASHError {
    fn from(value: EASHUncomfortable) -> Self {
        Self::DumbassDevForgotToHandleThis(value)
    }
}
