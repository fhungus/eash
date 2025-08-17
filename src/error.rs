pub enum EASHError {
    FlushFaliure,
    MalformedElement,
    MLuaError(mlua::Error),
}

impl From<mlua::Error> for EASHError {
    fn from(value: mlua::Error) -> Self {
        return EASHError::MLuaError(value);
    }
}
