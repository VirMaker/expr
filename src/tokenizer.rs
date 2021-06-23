use super::{Error, Position, operator};

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

pub struct Tokens<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    byte_ix: u32
}

impl Tokens<'_> {

    pub fn new(val:&str) -> Tokens {
        Tokens {
            chars: val.chars().peekable(),
            byte_ix:0,
        }
    }

    fn next_char(&mut self) -> Option<(u32, char)> {
        if let Some(ch) = self.chars.next() {
            let prev_ix = self.byte_ix;
            self.byte_ix += ch.len_utf8() as u32;
            return Some((prev_ix, ch));
        }
        None
    }

    fn number(&mut self, at:u32) -> Position {
        let mut len = 1;
        while let Some(ch) = self.chars.peek() {
            if ch.is_ascii_digit() || *ch == '.' {
                len += 1;
                let _ = self.next_char();
            } else {
                break;
            }
        }    
        Position { at, len }
    }
    
    fn string(&mut self, at:u32) -> Token {
        let mut len = 1;
        while let Some(ch) = self.chars.peek() {
            // strings can have digits in them
            if ch.is_alphanumeric() || *ch == '_' {
                len += 1;
                let _ = self.next_char();
            } else {
                break;
            }
        }
    
        Token::Str(Position { at, len })
    }
}

impl Iterator for Tokens<'_> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut char_num = 0;

        while let Some((byte_ix, ch)) = self.next_char() {
            char_num += 1;
            if ch.is_ascii_whitespace() {
                continue;
            } else if ch.is_ascii_digit() || ch == '.' {
                let position = self.number(byte_ix);
                if position.len == 1 && ch == '.' {
                    return Some(Err(Error{
                        error: format!("Unexpected token '.' at position {}", char_num),
                        at: char_num
                    }));
                }
                return Some(Ok(Token::Number(position)));
            } else if let Some(ix1) = operator::is_operator(ch) {
                let mut operator_ix = ix1;
                // attempt to resolve multi char operators
                if let Some(char2) = self.chars.peek() {
                    if let Some(ix2) = operator::is_multi_char(ch, *char2) {
                        self.next_char();
                        operator_ix = ix2;    
                    }
                }    
                return Some(Ok(Token::Operator {
                    at: byte_ix,
                    operator_ix: operator_ix,
                }));
            } else if ch == ',' {
                return Some(Ok(Token::Comma(byte_ix)));
            } else if ch == '(' {
                return Some(Ok(Token::LParen(byte_ix)));
            } else if ch == ')' {
                return Some(Ok(Token::RParen(byte_ix)));
            } else if ch.is_ascii_punctuation() && ch != '_' {
                return Some(Err(Error{
                    error: format!("Found reserved character {} at {}",
                                                                ch, char_num),
                    at: char_num
                }));
            } else {
                // this must be allowed
                return Some(Ok(self.string(byte_ix)));
            }
        }
        None
    }
}


#[cfg(test)]
mod tokenize_should {

    use super::*;
    use matches::assert_matches;

    fn next(val: &mut Tokens) -> Token {
        val.next().unwrap().unwrap()
    }

    #[test]
    fn have_16_bytes_token_max() {
        assert!(std::mem::size_of::<Token>() <= 16);
    }
    
    #[test]
    fn handle_numbers() {
        let mut tokens = Tokens::new("123.123");
        assert_matches!(tokens.next().unwrap().unwrap(), Token::Number(..));
        assert_matches!(tokens.next(), None);
    }

    #[test]
    fn handle_negative_numbers() {
        let mut tokens = Tokens::new("-123");
        assert_matches!(next(&mut tokens), Token::Operator{..});
        assert_matches!(next(&mut tokens), Token::Number(..));
        assert_matches!(tokens.next(), None);
    }

    #[test]
    fn ignore_spaces() {
        let mut tokens = Tokens::new(" 1 + 2 ");
        assert_matches!(next(&mut tokens), Token::Number(..));
        assert_matches!(next(&mut tokens), Token::Operator{..});
        assert_matches!(next(&mut tokens), Token::Number(..));
        assert_matches!(tokens.next(), None);
    }

    #[test]
    fn handle_strings() {
        let mut tokens = Tokens::new(" _abc34_8_ ");
        let opt = tokens.next();
        let res = opt.unwrap();
        let token = res.unwrap();
        assert_matches!(token, Token::Str(..));
        assert_matches!(tokens.next(), None);
    }

    #[test]
    fn handle_comma() {
        let mut tokens = Tokens::new(" (1,2) ");
        assert_matches!(next(&mut tokens), Token::LParen(..));
        assert_matches!(next(&mut tokens), Token::Number(..));
        assert_matches!(next(&mut tokens), Token::Comma(..));
        assert_matches!(next(&mut tokens), Token::Number(..));
        assert_matches!(next(&mut tokens), Token::RParen(..));
        assert_matches!(tokens.next(), None);
    }

    #[test]
    fn handle_operators() {
        let string = "+-*/=".to_string();
        let tokens = Tokens::new(&string);
        for token in tokens {
            assert_matches!(token.unwrap(), Token::Operator{..});
        }
    }

    #[test]
    fn handle_mutli_char_operators() {
        let mut tokens = Tokens::new(">=<=");
        assert_matches!(next(&mut tokens), Token::Operator{..});
        assert_matches!(next(&mut tokens), Token::Operator{..});
        assert_matches!(tokens.next(), None)
    }

    #[test]
    fn handle_single_dot_error() {
        let error = Tokens::new(" . ").next().unwrap().unwrap_err();
        assert_eq!(error.at, 2);
    }
}