use pest::{error::Error, Parser, iterators::Pair};
use pest_derive::Parser;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[grammar = "./src/parser/letter-script.pest"]
struct LetterScriptParser;

#[derive(Debug)]
enum Expression {
    Int(i32),
    Float(f32),
    Bool(bool),
    Char(char)
}

#[derive(Debug)]
enum Statement {
    Let {
        identifier: String,
        type_identifier: String,
        value: Expression
    },
    Fn {
        identifier: String,
        return_type: String,
        params: Vec<(String, String)>,
        body: Option<()>
    }
}

#[derive(Debug)]
enum AstNode {
    Expr(Expression),
    Stmt(Statement)
}

fn parse_expression(pair: Pair<Rule>) -> Option<Expression> {
    match pair.as_rule() {
        Rule::expression => {
            let mut pair = pair.into_inner();
            let expr = pair.next().unwrap();
            return parse_expression(expr);
        },
        Rule::integer => {
            let int_str = pair.as_str();
            let integer: i32 = int_str.parse().unwrap();

            return Some(Expression::Int(integer))
        },
        x => {
            println!("haha: {:?}", x);
            return None
        }
    }
}

fn parse_param_list(pair: Pair<Rule>) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = vec![];
    if let Rule::function_param_list = pair.as_rule() {
        let mut pair = pair.into_inner();
        while let Some(pair) = pair.next() {
            if let Rule::function_param = pair.as_rule() {
                let mut pair = pair.into_inner();
                let identifier = pair.next().unwrap().as_str().to_string();
                let type_identifier = pair.next().unwrap().as_str().to_string();

                params.push((identifier, type_identifier));
            }
        }
    }

    return params;
}

fn parse_statement(pair: Pair<Rule>) -> Option<Statement> {
    match pair.as_rule() {
        Rule::r#let => {
            let mut pair = pair.into_inner();
            let identifier = pair.next().unwrap();
            let type_identifier = pair.next().unwrap();
            let value_expr = parse_expression(pair.next().unwrap());

            return Some(Statement::Let { identifier: identifier.as_str().to_string(), type_identifier: type_identifier.as_str().to_string(), value: value_expr.expect("expected expr") });
        },
        Rule::function => {
            let mut pair = pair.into_inner();
            let mut identifier = pair.next().unwrap().as_str().to_string();

            let param_list = parse_param_list(pair.next().unwrap());
            let return_type_identifier = pair.next().unwrap().as_str().to_string();

            return Some(Statement::Fn { identifier: identifier, return_type: return_type_identifier, params: param_list, body: None })
        },
        x => {
            println!("parse_statement::unknown -> {:?}", x);
            return None
        }
    }
}

fn parse(input: &str) -> std::result::Result<(), Error<Rule>> {
    let pairs = LetterScriptParser::parse(Rule::program, input)?;

    //let ints: Vec<LSInt> = vec![];
    let mut program : Vec<AstNode> = vec![];

    for pair in pairs {
        match pair.as_rule() {
            Rule::statement => {
                println!("statement");
                program.push(AstNode::Stmt(parse_statement(pair.into_inner().next().unwrap()).unwrap()));
            },
            x => {
                println!("unknown ->> {:?}", x);
            }
        }
    }

    dbg!(program);

    Ok(())
}

fn main() -> Result<()> {
    parse(r#"fn z(x: int, z: int) -> int { let h: char = 'c'; }"#)?;
    Ok(())
}
