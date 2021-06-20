use super::{Error, Position};
use crate::operator as operator;

#[derive(Debug, PartialEq)]
#[derive(Clone, Copy)]
pub enum Token {
    Number(Position),
    Str (Position),
    Operator { at: u32, operator_ix: u8 }, // second param is an index into operators array
    Comma  (u32),
    LParen (u32),
    RParen (u32),
}

pub fn tokenize(val: &str) -> Result<Vec<Token>, Error> {
    let mut result = Vec::new();
    let mut enumerator = Chars::new(val);
    let mut char_num = 0;

    while let Some((byte_ix, ch)) = enumerator.next() {
        char_num += 1;
        if ch.is_ascii_whitespace() {
            continue;
        } else if ch.is_ascii_digit() || ch == '.' {
            let position = number(byte_ix, &mut enumerator);
            if position.len == 1 && ch == '.' {
                return Err(Error{
                    error: format!("Unexpected token '.' at position {}", char_num),
                    at: char_num
                })
            }
            result.push(Token::Number(position));        
        } else if let Some(ix1) = operator::is_operator(ch){
            let mut operator_ix = ix1;
            // attempt to resolve multi char operators
            if let Some((_, char2)) = enumerator.peek() {
                if let Some(ix2) = operator::is_multi_char(ch, char2) {
                    enumerator.next();
                    operator_ix = ix2;    
                }
            }

            result.push(Token::Operator {
                at: byte_ix,
                operator_ix: operator_ix,
            });
        } else if ch == ',' {
            result.push(Token::Comma(byte_ix));
        } else if ch == '(' {
            result.push(Token::LParen(byte_ix));
        } else if ch == ')' {
            result.push(Token::RParen(byte_ix));
        } else if ch.is_ascii_punctuation() && ch != '_' {
            return Err(Error{
                error: format!("Found reserved character {} at {}",
                                                            ch, char_num),
                at: char_num
            })
        } else {
            // this must be allowed
            result.push(str_token(byte_ix, &mut enumerator))
        }
    }

    Ok(result)
}

fn number(at:u32, enumerator: &mut Chars) -> Position {
    let mut len = 1;
    loop {        
        match enumerator.peek() {
            Some((.., ch)) if ch.is_ascii_digit() || ch == '.' => {
                len += 1;
                let _ = enumerator.next();
            },
            _ => break
        }
    }

    Position { at, len }
}

fn str_token(at:u32, enumerator: &mut Chars) -> Token {
    let mut len = 1;
    loop {        
        match enumerator.peek() {
            // strings can have digits in them
            Some((.., ch)) if ch.is_alphanumeric() || ch == '_' => {
                len += 1;
                let _ = enumerator.next();
            },
            _ => break
        }
    }

    Token::Str(Position { at, len })
}

struct Chars<'a> {
    enumerator: std::iter::Peekable<std::str::Chars<'a>>,
    byte_ix: u32
}

impl Chars<'_> {
    fn new(val:&str) -> Chars {
        Chars{
            enumerator: val.chars().peekable(),
            byte_ix:0,
        }
    }

    fn next(&mut self) -> Option<(u32, char)> {
        if let Some(ch) = self.enumerator.next() {
            let result = Some((self.byte_ix, ch));
            self.byte_ix += ch.len_utf8() as u32;
            return result;
        }
        None
    }

    fn peek(&mut self) -> Option<(u32, char)> {
        if let Some(ch) = self.enumerator.peek() {
            let byte_ix = self.byte_ix + ch.len_utf8() as u32;
            let result = (byte_ix + 1, *ch);
            return Some(result);
        }
        None
    }
}


#[cfg(test)]
mod tokenize_should {

    use super::*;
    use matches::assert_matches;
    
    #[test]
    fn handle_numbers() {
        let tokens = tokenize("123.123").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_matches!(tokens[0], Token::Number(..));
    }

    #[test]
    fn handle_negative_numbers() {
        let tokens = tokenize("-123").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_matches!(tokens[0], Token::Operator{..});
        assert_matches!(tokens[1], Token::Number(..));
    }

    #[test]
    fn ignore_spaces() {
        let tokens = tokenize(" 1 + 2 ").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_matches!(tokens[0], Token::Number(..));
        assert_matches!(tokens[1], Token::Operator{..});
        assert_matches!(tokens[2], Token::Number(..));
    }

    #[test]
    fn handle_strings() {
        let tokens = tokenize(" _abc34_8_ ").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_matches!(tokens[0], Token::Str(..));
    }

    #[test]
    fn handle_comma() {
        let tokens = tokenize(" (1,2) ").unwrap();
        assert_matches!(tokens[0], Token::LParen(..));
        assert_matches!(tokens[1], Token::Number(..));
        assert_matches!(tokens[2], Token::Comma(..));
        assert_matches!(tokens[3], Token::Number(..));
        assert_matches!(tokens[4], Token::RParen(..));
    }

    #[test]
    fn handle_operators() {
        let string = "+-*/=".to_string();
        let tokens = tokenize(&string).unwrap();
        for token in tokens {
            assert_matches!(token, Token::Operator{..})
        }
    }

    #[test]
    fn handle_mutli_char_operators() {
        let tokens = tokenize(">=<=").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_matches!(tokens[0], Token::Operator{..});
        assert_matches!(tokens[1], Token::Operator{..});
    }

    #[test]
    fn handle_single_dot_error() {
        let error = tokenize(" . ").unwrap_err();
        assert_eq!(error.at, 2);
    }
}