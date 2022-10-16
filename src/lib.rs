use pest::iterators::Pairs;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use std::io::{self, BufRead};

#[derive(pest_derive::Parser)]
#[grammar = "calc.pest"]
pub struct CalculatorParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left))
            .op(Op::prefix(unary_minus))
    };
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    UnaryMinus(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

impl Expr {
    pub fn eval(&self) -> i32 {
        match self {
            Expr::Integer(val) => *val as i32,
            Expr::UnaryMinus(val) => val.eval() - val.eval() * 2,
            Expr::BinOp { lhs, op, rhs } => match op {
                Op::Add => lhs.eval() + rhs.eval(),
                Op::Subtract => lhs.eval() - rhs.eval(),
                Op::Multiply => lhs.eval() * rhs.eval(),
                Op::Divide => lhs.eval() / rhs.eval(),
                Op::Modulo => lhs.eval() % rhs.eval(),
            },
        }
    }
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer => Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            Rule::expr => parse_expr(primary.into_inner()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                Rule::modulo => Op::Modulo,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => Expr::UnaryMinus(Box::new(rhs)),
            _ => unreachable!(),
        })
        .parse(pairs)
}

pub fn repl() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        match CalculatorParser::parse(Rule::equation, &line?) {
            Ok(mut pairs) => {
                let inner = parse_expr(pairs.next().unwrap().into_inner());
                println!(
                    "Parsed: {:#?}",
                    // inner of expr
                    inner
                );
                println!("{}", inner.eval());
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<i32, Box<dyn std::error::Error>>;

    fn test_expr_parse(input: &str) -> TestResult {
        match CalculatorParser::parse(Rule::equation, input) {
            Ok(mut pairs) => {
                let inner = parse_expr(pairs.next().unwrap().into_inner());
                Ok(inner.eval())
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
                unreachable!()
            }
        }
    }

    #[test]
    fn run_tests() {
        let test_table = vec![
            ("5 + 5", 10),
            ("5 - 5", 0),
            ("5 * 5", 25),
            ("5 / 5", 1),
            ("(13 * 25 / 2) - ((25 - 4) + (16 / 3) * 2)", 131),
            ("5 * 6 * 7 + 24 - 16", 218),
            ("750 / 5 + (6 * 2) / 2", 156),
            ("1024 + 256 + 256 + 256 + 256 / (2)", 1920)
        ];
        for test in test_table.into_iter() {
            let (input, expected) = test;
            assert_eq!(test_expr_parse(input).unwrap(), expected);
        }
    }
}
