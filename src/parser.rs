use crate::tokenizer::{ Token };
use super::{Expr, Error, Position, BinaryExpr};
use std::iter::Peekable;
use std::slice::Iter;
use crate::operator as operator;

pub fn parse(tokens: Vec<Token>) -> Result<Expr, Error> {
    let mut enumerator = tokens.iter().peekable();
    let result = expr(&mut enumerator, 0);
    if let Some(token) = enumerator.peek() {
        return error("Unexpected token ", **token);
    }
    
    result
}


fn expr(tokens: &mut Peekable<Iter<Token>>, precedence: u8) -> Result<Expr, Error> {
    let mut left = singular(tokens);
    while let Some(token) = tokens.peek() {
        match token {
            Token::Operator{at:_, operator_ix} => {
                let new_prec = operator::from(*operator_ix).precedence;
                if  new_prec > precedence {
                    tokens.next();
                    let right = expr(tokens, new_prec);
                    left = Ok(Expr::Binary(Box::new(BinaryExpr {
                        left: left?,
                        operator_ix: *operator_ix,
                        right: right?
                    })))
                } else {
                    return left
                }
            },        
            _ => return left
        }
    }
    left
}

fn singular(tokens: &mut Peekable<Iter<Token>>) -> Result<Expr, Error> {
    if let Some(token) = tokens.peek() {
        match token {
            Token::Operator{ at:_, operator_ix } => {
                tokens.next();
                Ok(Expr::Unary{
                    operator_ix: *operator_ix, 
                    expr: Box::new(expr(tokens, 0)?)
                })
            },
            Token::Str(name) => {
                tokens.next(); //consume string
                match tokens.peek() {
                    Some(Token::LParen(_)) => {
                        Ok(Expr::Func {
                            name: *name, 
                            params: params(tokens)?
                        })
                    },
                    _ => {
                        Ok(Expr::Variable(*name))
                    }
                }
            },
            Token::LParen(_) => parentheses(tokens),
            Token::Number(pos) => {
                let number = Ok(Expr::Number(*pos));
                tokens.next();
                number
            },
            _ => error("Expected operator, variable, function or number but found ", **token)
        }
    } else {
        Err(Error {
            error: "Expected expression but reached the end".to_string(),
            at: 0
        })
    }
}

fn parentheses(tokens: &mut Peekable<Iter<Token>>) -> Result<Expr, Error> {
    tokens.next(); // consume left parenthesis
    let expr = expr(tokens, 0)?;
    match tokens.next() {
        Some(Token::RParen(..)) => Ok(expr),
        Some(token) => error("Expected closing parenthesis ')' but found ", *token),
        None => Err(Error {
            error: "Missing closing parenthesis ')'".to_string(),
            at: 0
        })
    }
}

fn params(tokens: &mut Peekable<Iter<Token>>) -> Result<Vec<Expr>, Error> {
    tokens.next(); // consume left parenthesis
    let mut vec = vec![];
    // function may have any number of parameters separated by comma
    // consume everything until closing (right) parenthesis
    loop {
        match tokens.peek() {
            Some(Token::RParen(..)) => {
                tokens.next();
                return Ok(vec);
            },
            Some(Token::Comma(..)) => {
                tokens.next();
                vec.push(expr(tokens, 0)?);
            },
            Some(_) => vec.push(expr(tokens, 0)?),
            None => return Err(Error {
                error: "Missing closing parenthesis ')'".to_string(),
                at: 0
            })
        };
    }
    
}

fn error(error: &str, _token:Token) -> Result<Expr, Error> {
    Err(Error {
        error: error.to_string(),
        at: 0
    })
}

#[cfg(test)]
mod parse_should {
    use super::*;
    use crate::operator as operator;

    const NUMBER: Token = Token::Number(Position { at: 0, len: 0 });
    const string: Token = Token::Str(Position { at: 0, len: 0 });
    const L_PAREN: Token = Token::LParen(0);
    const R_PAREN: Token = Token::RParen(0);
    const operator: Token = Token::Operator { at: 0, operator_ix: 0 };

    #[test]
    fn handle_numbers() {
        let tokens = vec![NUMBER];
        assert_matches!(parse(tokens), Ok(Expr::Number(..)));
    }

    #[test]
    fn handle_single_unary() {
        let tokens = vec![operator, NUMBER];
        assert_matches!(parse(tokens), Ok(Expr::Unary {..}))
    }

    #[test]
    fn handle_nested_unary() {
        let tokens = vec![operator, operator, NUMBER];
        if let Ok(Expr::Unary{expr:unary, operator_ix:_}) = parse(tokens) {
            if let Expr::Unary{expr:num, operator_ix:_} = *unary {
                assert_matches!(*num, Expr::Number(..));
                return;
            }
        }
        assert!(false);
    }
    
    #[test]
    fn handle_parentheses() {
        let tokens = vec![L_PAREN, NUMBER, R_PAREN];
        let tree = parse(tokens);
        assert_matches!(tree, Ok(Expr::Number(..)));
    }

    #[test]
    fn handle_binary_expr() {
        let tokens = vec![NUMBER, operator, NUMBER];
        assert_matches!(parse(tokens), Ok(Expr::Binary(..)));
    }

    #[test]
    fn handle_multiple_binary_expr() {
        let tokens = vec![
            NUMBER,
            operator,
            NUMBER,
            operator,
            NUMBER];
        if let Ok(Expr::Binary(bin_expr)) = parse(tokens) {
            let expr = *bin_expr;
            assert_matches!(expr.left, Expr::Binary(..));        
            assert_matches!(expr.right, Expr::Number(..));
        }
        else { assert!(false) }
    }

    #[test]
    fn handle_variable() {
        let tokens = vec![string];
        let expr = parse(tokens).unwrap();
        assert_matches!(expr, Expr::Variable(..));
    }

    #[test]
    fn handle_func_no_params() {
        let tokens = vec![string, L_PAREN, R_PAREN];
        assert_matches!(parse(tokens), Ok(Expr::Func{..}));
    }

    #[test]
    fn handle_func_with_params() {
        let tokens = vec![
            string, 
            L_PAREN, 
            NUMBER, 
            Token::Comma(0), 
            NUMBER, 
            R_PAREN
        ];
        if let Ok(Expr::Func{name:_, params}) = parse(tokens) { 
            assert_eq!(params.len(), 2);
        }
        else { assert!(false) }
    }

    #[test]
    fn respect_operator_precedence() {
        let tokens = vec![
            NUMBER, 
            Token::Operator{at: 0, operator_ix: operator::is_operator('+').unwrap()}, 
            NUMBER, 
            Token::Operator{at: 0, operator_ix: operator::is_operator('*').unwrap()}, 
            NUMBER
        ];
        if let Expr::Binary(bin1) = parse(tokens).unwrap() {
            if let Expr::Binary(bin2) = (*bin1).right {
                let bin2 = *bin2;
                assert_eq!(operator::from(bin1.operator_ix).char1, '+');
                assert_eq!(operator::from(bin2.operator_ix).char1, '*');
            }
        } else {
            assert!(false);
        }
    }

    #[test]
    fn error_on_missing_parenthesis() {
        let tokens = vec![L_PAREN, NUMBER];
        let expr = parse(tokens);
        assert_matches!(expr, Err(..));
    }

    #[test]
    fn error_on_extra_parenthesis() {
        let tokens = vec![L_PAREN, NUMBER, R_PAREN, R_PAREN];
        let expr = parse(tokens);
        assert_matches!(expr, Err(..));
    }

    #[test]
    fn error_on_incomplete() {
        let tokens = vec![NUMBER, operator];
        let expr = parse(tokens);
        assert_matches!(expr, Err(..));
    }
}