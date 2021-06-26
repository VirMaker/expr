use crate::tokenizer::{ Token };
use super::{Expr, Error, BinaryExpr, FuncExpr};
use std::iter::Peekable;
use crate::operator as operator;


pub fn parse(tokens: &mut impl Iterator<Item = Result<Token,Error>>) -> Result<Expr, Error> {
    let mut has_error:Option<Error> = None;
    let mut enumerator = tokens
        .scan(&mut has_error, |err, res| match res {
            Ok(token)  => Some(token),
            Err(e) => {
                **err = Some(e);
                None
            }
        })
        .peekable();
    let result = expr(&mut enumerator, 0);
    // check unconsumed tokens
    if let Some(token) = enumerator.next() {
        return error("Unexpected token ", token);
    }
    // check for errors
    if let Some(err) = has_error {
        return Err(err);
    }
    
    result
}

fn peek(tokens: &mut Peekable<impl Iterator<Item=Token>>) -> Option<Token> {
    tokens.peek().map(|t| *t)
}

fn expr(tokens: &mut Peekable<impl Iterator<Item=Token>>, precedence: u8) -> Result<Expr, Error> {
    let mut left = singular(tokens);
    while let Some(token) = peek(tokens) {
        match token {
            Token::Operator {at:_, operator_ix} => {
                let new_prec = operator::from(operator_ix).precedence;
                if  new_prec > precedence {
                    tokens.next();
                    let right = expr(tokens, new_prec);
                    left = Ok(Expr::Binary(Box::new(BinaryExpr {
                        left: left?,
                        operator_ix,
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


fn singular(tokens: &mut Peekable<impl Iterator<Item=Token>>) -> Result<Expr, Error> {
    if let Some(token) = peek(tokens) {
        match token {
            Token::Operator{ at:_, operator_ix } => {
                tokens.next();
                Ok(Expr::Unary{
                    operator_ix, 
                    expr: Box::new(expr(tokens, 0)?)
                })
            },
            Token::Str(name) => {
                tokens.next(); //consume STRING
                //string followed by left parenth is a function
                match tokens.peek() {
                    Some(Token::LParen(_)) => {
                        Ok(Expr::Func(Box::new(FuncExpr {
                            name, 
                            params: params(tokens)?
                        })))
                    },
                    _ => {
                        Ok(Expr::Variable(name))
                    }
                }
            },
            Token::LParen(_) => parentheses(tokens),
            Token::Number(pos) => {
                let number = Ok(Expr::Number(pos));
                tokens.next();
                number
            },
            _ => error("Expected operator, variable, function or number but found ", token)
        }
    } else {
        Err(Error {
            error: "Expected expression but reached the end".to_string(),
            at: 0
        })
    }
}

fn parentheses(tokens: &mut Peekable<impl Iterator<Item=Token>>) -> Result<Expr, Error> {
    tokens.next(); // consume left parenthesis
    let expr = expr(tokens, 0)?;
    match tokens.next() {
        Some(Token::RParen(..)) => Ok(expr),
        Some(token) => error("Expected closing parenthesis ')' but found ", token),
        None => Err(Error {
            error: "Missing closing parenthesis ')'".to_string(),
            at: 0
        })
    }
}

fn params(tokens: &mut Peekable<impl Iterator<Item=Token>>) -> Result<Vec<Expr>, Error> {
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
    use super::super::Position;
    use crate::operator as operator;

    const NUMBER: Result<Token,Error> = Ok(Token::Number(Position { at: 0, len: 0 }));
    const STRING: Result<Token,Error> = Ok(Token::Str(Position { at: 0, len: 0 }));
    const L_PAREN: Result<Token,Error> = Ok(Token::LParen(0));
    const R_PAREN: Result<Token,Error> = Ok(Token::RParen(0));
    const OPERATOR: Result<Token,Error> = Ok(Token::Operator { at: 0, operator_ix: 0 });

    #[test]
    fn handle_numbers() {
        let mut tokens = vec![NUMBER].into_iter();
        assert_matches!(parse(&mut tokens), Ok(Expr::Number(..)));
    }

    #[test]
    fn handle_single_unary() {
        let mut tokens = vec![OPERATOR, NUMBER].into_iter();
        assert_matches!(parse(&mut tokens), Ok(Expr::Unary {..}))
    }

    #[test]
    fn handle_nested_unary() {
        let mut tokens = vec![OPERATOR, OPERATOR, NUMBER].into_iter();
        if let Ok(Expr::Unary{expr:unary, operator_ix:_}) = parse(&mut tokens) {
            if let Expr::Unary{expr:num, operator_ix:_} = *unary {
                assert_matches!(*num, Expr::Number(..));
                return;
            }
        }
        assert!(false);
    }
    
    #[test]
    fn handle_parentheses() {
        let mut tokens = vec![L_PAREN, NUMBER, R_PAREN].into_iter();
        let tree = parse(&mut tokens);
        assert_matches!(tree, Ok(Expr::Number(..)));
    }

    #[test]
    fn handle_binary_expr() {
        let mut tokens = vec![NUMBER, OPERATOR, NUMBER].into_iter();
        assert_matches!(parse(&mut tokens), Ok(Expr::Binary(..)));
    }

    #[test]
    fn handle_multiple_binary_expr() {
        let mut tokens = vec![
            NUMBER,
            OPERATOR,
            NUMBER,
            OPERATOR,
            NUMBER].into_iter();
        if let Ok(Expr::Binary(bin_expr)) = parse(&mut tokens) {
            let expr = *bin_expr;
            assert_matches!(expr.left, Expr::Binary(..));        
            assert_matches!(expr.right, Expr::Number(..));
        }
        else { assert!(false) }
    }

    #[test]
    fn handle_variable() {
        let mut tokens = vec![STRING].into_iter();
        let expr = parse(&mut tokens).unwrap();
        assert_matches!(expr, Expr::Variable(..));
    }

    #[test]
    fn handle_func_no_params() {
        let mut tokens = vec![STRING, L_PAREN, R_PAREN].into_iter();
        assert_matches!(parse(&mut tokens), Ok(Expr::Func{..}));
    }

    #[test]
    fn handle_func_with_params() {
        let mut tokens = vec![
            STRING, 
            L_PAREN, 
            NUMBER, 
            Ok(Token::Comma(0)), 
            NUMBER, 
            R_PAREN
        ].into_iter();
        if let Ok(Expr::Func(boxed)) = parse(&mut tokens) {
            let FuncExpr{name:_, params} = *boxed;
            assert_eq!(params.len(), 2);
        }
        else { assert!(false) }
    }

    #[test]
    fn respect_operator_precedence() {
        let mut tokens = vec![
            NUMBER, 
            Ok(Token::Operator{at: 0, operator_ix: operator::is_operator('+').unwrap()}), 
            NUMBER, 
            Ok(Token::Operator{at: 0, operator_ix: operator::is_operator('*').unwrap()}), 
            NUMBER
        ].into_iter();
        if let Expr::Binary(bin1) = parse(&mut tokens).unwrap() {
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
        let mut tokens = vec![L_PAREN, NUMBER].into_iter();
        let expr = parse(&mut tokens);
        assert_matches!(expr, Err(..));
    }

    #[test]
    fn error_on_extra_parenthesis() {
        let mut tokens = vec![L_PAREN, NUMBER, R_PAREN, R_PAREN].into_iter();
        let expr = parse(&mut tokens);
        assert_matches!(expr, Err(..));
    }

    #[test]
    fn error_on_incomplete() {
        let mut tokens = vec![NUMBER, OPERATOR].into_iter();
        let expr = parse(&mut tokens);
        assert_matches!(expr, Err(..));
    }

    #[test]
    fn error_on_tokenizer_error() {
        let error:Result<Token,Error> = Err(Error{error:"tokenizer".to_string(), at:0});
        let mut tokens = vec![NUMBER, error, STRING].into_iter();
        let expr = parse(&mut tokens);
        assert!(match expr {
            Err(e) if e.error.contains("tokenizer") => true,
            _ => false
        });
    }
}