use crate::{
    error::EASHError, evaluate::{
        Token, TokenType
    }
};

use std::process::{Child as PChild, Command as PCommand};

pub struct Command {
    inner: PCommand,

}

impl TryFrom<Vec<Token>> for Command {
    type Error = EASHError;


    fn try_from(tokens: Vec<Token>) -> Result<Self, Self::Error> {
    }
}

pub struct Child {
    inner: PChild,
    command: Command
}

impl Child {
    pub fn spawn() {
        
    }
}