
pub fn is_operator(char1: char) -> Option<u8> {
    OPERATORS.iter().position(|op| op.char1 == char1).map(|pos| pos as u8)
}

pub fn is_multi_char(char1: char, char2: char) -> Option<u8> {
    let char2 = Some(char2);
    OPERATORS.iter().position(|op| op.char1 == char1 && op.char2 == char2).map(|pos| pos as u8)
}

pub fn from(operator_ix: u8) -> Operator {
    OPERATORS[operator_ix as usize]
}

#[derive(Debug, PartialEq)]
#[derive(Clone, Copy)]
pub struct Operator {
    pub char1: char,
    pub char2: Option<char>,
    pub precedence: u8,  
    pub prefix: bool, // can be used as prefix?
}

impl Operator {
    const fn new(char1: char, char2: Option<char>, precedence: u8, prefix: bool) -> Operator {
        Operator { char1, char2, precedence, prefix }
    }
}

const OPERATORS: [Operator; 9] = [ 
    Operator::new('/', None, 60, false),
    Operator::new('*', None, 60, false),
    Operator::new('+', None, 50, true),
    Operator::new('-', None, 50, true),
    Operator::new('<', None, 40, false),
    Operator::new('>', None, 40, false),
    Operator::new('<', Some('='), 40, false),
    Operator::new('>', Some('='), 40, false),
    Operator::new('=', None, 30, false)
];