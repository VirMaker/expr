#[cfg(test)]
#[macro_use]
extern crate matches;

mod tokenizer;
mod parser;
mod operator;

#[derive(Debug, PartialEq)]
#[derive(Clone, Copy)]
pub struct Position {
    pub at: u32,
    pub len: u16
}

impl Position {
    fn to_range(&self) -> std::ops::Range<usize> {
        let start = self.at as usize;
        let end = start + self.len as usize;
        start .. end
    }
}

#[derive(Debug)]
pub enum Expr {
    Number(Position),
    Variable(Position),
    Func ( Box<FuncExpr> ),
    Unary{ expr: Box<Expr>, operator_ix: u8 },
    Binary(Box<BinaryExpr>)
}

#[derive(Debug)]
pub struct FuncExpr {
    name: Position,
    params: Vec<Expr>
}

#[derive(Debug)]
pub struct BinaryExpr {
    left: Expr,
    right: Expr,
    operator_ix: u8
}


pub fn evaluate(expression: &str) -> Result<f32, Error> {
    let mut tokens = tokenizer::Tokens::new(expression);
    let expr = parser::parse(&mut tokens)?;
    Ok(eval_expr(&expr, expression))
}

fn eval_expr(expr:&Expr, expression: &str) -> f32 {
    match expr {
        Expr::Number(pos) => expression[pos.to_range()].parse::<f32>().unwrap(),
        Expr::Binary(bin) => {
            let left = eval_expr(&bin.left, expression);
            let right = eval_expr(&bin.right, expression);
            let operator = operator::from(bin.operator_ix);
            match operator.char1 {
                '+' => left + right,
                '-' => left - right,
                '*' => left * right,
                '/' => left / right,
                '>' if operator.char2 == Some('=') => if left >= right {1.0} else {0.0},
                '<' if operator.char2 == Some('=') => if left <= right {1.0} else {0.0},
                '>' => if left > right {1.0} else {0.0},
                '<' => if left < right {1.0} else {0.0},
                '=' => if left == right {1.0} else {0.0},
                _ => panic!("Unexpected operator") // this arm should be handled by the parser
            }
        }
        Expr::Unary{ expr, operator_ix } => {
            let operator = operator::from(*operator_ix);
            match operator.char1 {
                '+' => eval_expr(expr, expression),
                '-' => -eval_expr(expr, expression),
                _ => panic!("Unexpected operator") // this arm should be handled by the parser
            }
        }
        Expr::Variable(_pos)=> {
            1f32
        }
        Expr::Func( boxed_func ) => {
            let FuncExpr { name, params } = &**boxed_func;
            match &expression[name.to_range()] {
                "pi" => std::f64::consts::PI as f32,
                "if" => {
                    if params.len() != 3 {
                        panic!("Expected 3 arguments into 'if' function");
                    }
                    if eval_expr(&params[0], expression) > 0.0 {
                        eval_expr(&params[1], expression)
                    } else {
                        eval_expr(&params[2], expression)
                    }
                }
                _ => 0f32
            }
        }
    }
}


#[derive(Debug)]
pub struct Error {
    error: String,
    at: u32,
}

#[cfg(test)]
mod evaluate_should {
    use super::*;
    
    #[test]
    fn have_16_bytes_token_max() {
        assert!(std::mem::size_of::<Expr>() <= 16);
    }

    #[test]
    fn handle_numbers() {
        assert_eq!(evaluate("1").unwrap(), 1.0);
        assert_eq!(evaluate("99.88").unwrap(), 99.88f32)
    }

    #[test]
    fn handle_unary_expr() {
        assert_eq!(evaluate("-1").unwrap(), -1f32);
        assert_eq!(evaluate("--2").unwrap(), 2f32);
    }

    #[test]
    fn handle_pi_func() {
        assert_eq!(evaluate("pi()").unwrap(), std::f64::consts::PI as f32);
    }

    #[test]
    fn handle_if_func() {
        assert_eq!(evaluate("if(1 > 0, 10, -1)").unwrap(), 10.0);
        assert_eq!(evaluate("if(1 < 0, 10, -1)").unwrap(), -1.0);
        assert_eq!(evaluate("if(1 = 1, 10, -1)").unwrap(), 10.0);
        assert_eq!(evaluate("if(1 >= 0, 10, -1)").unwrap(), 10.0);
        assert_eq!(evaluate("if(1 >= 1, 10, -1)").unwrap(), 10.0);
        assert_eq!(evaluate("if(1 <= 0, 10, -1)").unwrap(), -1.0);
        assert_eq!(evaluate("if(1 <= 1, 10, -1)").unwrap(), 10.0);
    }

    #[test]
    fn handle_variable() {
        assert_eq!(evaluate("abc").unwrap(), 1.0);
    }

    #[test]
    fn handle_binary() {
        assert_eq!(evaluate("1 + 1").unwrap(), 2f32);
        assert_eq!(evaluate("2/2").unwrap(), 1f32);
        assert_eq!(evaluate("2*3").unwrap(), 6f32);
    }

    #[test]
    fn respect_operator_precedence() {
        assert_eq!(evaluate("3 * 2 + 1").unwrap(), 7f32);
        assert_eq!(evaluate("1 + 3 * 2").unwrap(), 7f32);
        assert_eq!(evaluate("12/2/3").unwrap(), 2f32);        
    }

    #[test]
    fn respect_parentheses() {
        assert_eq!(evaluate("(1 + 3) * 2").unwrap(), 8f32);
        assert_eq!(evaluate("3 * (2 + 1)").unwrap(), 9f32);
        assert_eq!(evaluate("(1 + 3) * (2 + 1)").unwrap(), 12f32);
    }
}