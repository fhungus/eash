// tiny ass trinkets for parsing a syntax that is unfortunately oderous of sh
#[derive(PartialEq, Debug)]
pub enum TokenType {
    Value(String),
    String(String),
    Directory(String),
    Flag(String),
    AndThen,
    Pipe,
    Nonsense(String),
}

impl TokenType {
    pub fn not_a_symbol(&self) -> Option<&String> {
        match self {
            TokenType::Directory(s) => Some(s),
            TokenType::String(s) => Some(s),
            TokenType::Value(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub contents: TokenType,
}

pub type TokenString = Vec<Token>;

enum ConsumptionMode {
    Default,
    String(char), // what character started the string
    Flag,
    DoubleFlag,
}

fn looks_like_directory(s: &str) -> bool {
    // TODO)) find cases where this doesn't work and make them work!!!
    s.starts_with(".") || s.contains("/") || s.starts_with("~")
}

fn str_to_token(s: &str, mode: &ConsumptionMode, st: usize, en: usize) -> Token {
    let content = s.to_string();
    let token_type = match mode {
        ConsumptionMode::Default | ConsumptionMode::String(_) => {
            if looks_like_directory(s) {
                TokenType::Directory(content)
            } else if let ConsumptionMode::String(_) = mode {
                TokenType::String(content)
            } else {
                TokenType::Value(content)
            }
        }
        ConsumptionMode::Flag => TokenType::Flag(s.to_string()),
        ConsumptionMode::DoubleFlag => TokenType::Flag(s.to_string()),
    };

    Token {
        start: st,
        end: en,
        contents: token_type,
    }
}

// notice: i aint got no langdev experience pls hold the tomatoes
// also i don't like that i made tokenization just one fatass fucking function
pub fn tokenize(s: &str) -> Vec<Token> {
    // the sizes were random numbers i chose...
    let mut tokens = Vec::with_capacity(5);
    let mut current_token = String::with_capacity(20);
    let mut current_token_start = 0;

    let mut mode = ConsumptionMode::Default;
    let mut chars = s.chars().enumerate().peekable();
    loop {
        let c;
        let pos;
        if let Some((np, nc)) = chars.next() {
            c = nc;
            pos = np;
        } else {
            break;
        };

        // idk how i should mutate mode here so for now you're just supposed to mutate it after using this...
        // ):
        let mut push_token = |mode, current_token: &mut String, pos: usize| {
            tokens.push(str_to_token(current_token, mode, current_token_start, pos));
            current_token_start = pos + 1;
            current_token.clear();
        };

        let mut string_check = |mode: &ConsumptionMode| {
            // channeling my inner yanderedev....
            // the character should never be an ending string so we SHOULD be good....
            if let ConsumptionMode::String(_) = mode {
                current_token.push(c);
                return true;
            }
            false
        };

        match c {
            ' ' => {
                // if we're not in a string then tokenize, otherwise string stuff happens...
                if !string_check(&mode) && !current_token.is_empty() {
                    push_token(&mode, &mut current_token, pos);
                    mode = ConsumptionMode::Default;
                }
            }
            '-' => {
                if let Some((_, '-')) = chars.peek() {
                    _ = chars.next();
                    mode = ConsumptionMode::DoubleFlag;
                } else {
                    mode = ConsumptionMode::Flag;
                }
            }
            '"' | '\'' | '`' => {
                // with this implementation i can EASILY add as many string indicators as i want... i cooked!
                if let ConsumptionMode::String(sc) = mode {
                    if c == sc {
                        push_token(&mode, &mut current_token, pos);
                        mode = ConsumptionMode::Default;
                        continue;
                    }
                } else {
                    mode = ConsumptionMode::String(c);
                }
            }
            '|' => {
                if string_check(&mode) {
                    continue;
                }
                if !current_token.is_empty() {
                    push_token(&mode, &mut current_token, pos - 1);
                }
                mode = ConsumptionMode::Default;
                tokens.push(Token {
                    start: current_token_start,
                    end: pos,
                    contents: TokenType::Pipe,
                });
                current_token_start = pos + 1;
            }
            '&' if matches!(chars.peek(), Some((_, '&'))) => {
                // functionally just &&
                if string_check(&mode) {
                    continue;
                }
                if !current_token.is_empty() {
                    push_token(&mode, &mut current_token, pos - 1);
                }
                _ = chars.next();
                mode = ConsumptionMode::Default;
                tokens.push(Token {
                    start: current_token_start,
                    end: pos + 1,
                    contents: TokenType::AndThen,
                });
                current_token_start = pos + 2;
            }
            _ => {
                if string_check(&mode) {
                    continue;
                }
                current_token.push(c);
            }
        }
    }
    if !current_token.is_empty() {
        tokens.push(str_to_token(
            &current_token,
            &mode,
            current_token_start,
            s.len() - 1,
        ));
    }

    tokens
}

struct TreeCommand {
    program_path: String,
    flags: Vec<(String, String)>,
    values: Vec<String>,
    pipe: bool, // will pipe this command to the next if it should (|) and not if it shouldnt (&& / nothing i guess)
    next: Option<Box<TreeCommand>>
}

fn new_treecommand_with_token(t: &Token) -> Result<TreeCommand, EASHUncomfortable> {


    if let Some(s) = treeable {
        return Ok(
            TreeCommand {
                // im cloning up a STORM!!!
                program_path: s.clone(),
                flags: Vec::new(), 
                values: Vec::new(), 
                pipe: false, 
                next: None 
            })
    }  else {
        return Err(EASHUncomfortable::CommandStartedWithoutProgram(t.clone()));
    }
}

fn to_ast(tokens: &Vec<Token>) -> Result<TreeCommand, EASHUncomfortable> {
    let commands: Vec<TreeCommand> = Vec::new();
    let mut processing: Option<TreeCommand> = None;
    let tokens_iter = tokens.iter().peekable();
    loop {
        let t = match tokens_iter.next() {
            Some(t) => t,
            None => {
                break
            },
        };
        match processing {
            None => {
                processing = Some(new_treecommand_with_token(t)?);
            },
            Some(p) => {
                match &t.contents {
                    TokenType::Flag(s) => {
                        let after = tokens_iter.peek();
                        match after {
                            None => {

                            },
                            Some(t2) => {
                                
                            }
                        }
                    },
                    TokenType::Directory(s) => {
                        p.values.push(s.clone());
                    },
                    TokenType::String(s) => {
                        p.values.push(s.clone());
                    }
                }
            }
        }
    }

    return tokens
}

#[cfg(test)]
mod tests {
    use crate::evaluate::{Token, TokenType, tokenize};

    #[test]
    fn tokenize_pipes_and_strings() {
        let command = "echo \"Hello, Air Jordans...\" | cat && clear";
        let expected = vec![
            Token {
                start: 0,
                end: 4,
                contents: TokenType::String("echo".to_string()),
            },
            Token {
                start: 5,
                end: 27,
                contents: TokenType::String("Hello, Air Jordans...".to_string()),
            },
            Token {
                start: 28,
                end: 29,
                contents: TokenType::Pipe,
            },
            Token {
                start: 30,
                end: 34,
                contents: TokenType::String("cat".to_string()),
            },
            Token {
                start: 35,
                end: 35,
                contents: TokenType::AndThen,
            },
            Token {
                start: 36,
                end: 42,
                contents: TokenType::String("clear".to_string()),
            },
        ];

        let actual = tokenize(command);

        assert_eq!(actual, expected);
    }

    // #[test]
    // fn one_eashillion_strings() {
    //     let command = "'Oh' \"My\" `Goodness` gracious";
    //     let expected = vec![
    //         Token::String("Oh".to_string()),
    //         Token::String("My".to_string()),
    //         Token::String("Goodness".to_string()),
    //         Token::String("gracious".to_string()),
    //     ];

    //     let actual = tokenize(command);

    //     assert_eq!(actual, expected);
    // }
}
