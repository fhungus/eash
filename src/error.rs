#[derive(Debug)]
pub enum EASHError {
    FlushFaliure,
    MalformedElement,
    ColorNotFlat,
    MLuaError(mlua::Error),
    IOError(std::io::Error),
}

impl From<mlua::Error> for EASHError {
    fn from(value: mlua::Error) -> Self {
        return EASHError::MLuaError(value);
    }
}

impl From<std::io::Error> for EASHError {
    fn from(error: std::io::Error) -> EASHError {
        EASHError::IOError(error)
    }
}