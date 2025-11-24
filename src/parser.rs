use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char as ch, digit1, one_of},
    combinator::{map, map_res, recognize, value},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded},
};

// Define the AST for Lisp
#[derive(Debug, Clone, PartialEq)]
pub enum LispVal {
    Integer(i64),
    Character(char),
    Bool(bool),
    Symbol(String),
    List(Vec<LispVal>),
}
fn parse_bool(input: &str) -> IResult<&str, LispVal> {
    alt((
        map(tag("#t"), |_| LispVal::Bool(true)),
        map(tag("#f"), |_| LispVal::Bool(false)),
    ))
    .parse(input)
}
// Integer parser
fn parse_integer(input: &str) -> IResult<&str, LispVal> {
    map_res(digit1, |digit_str: &str| {
        digit_str.parse::<i64>().map(LispVal::Integer)
    })
    .parse(input)
}

// Character parser: 'a', '\n', '\t'
fn parse_character(input: &str) -> IResult<&str, LispVal> {
    let c = delimited(
        ch('\''),
        alt((
            // escaped characters
            preceded(
                ch('\\'),
                alt((
                    value('\n', ch('n')),
                    value('\r', ch('r')),
                    value('\t', ch('t')),
                    value('\\', ch('\\')),
                    value('\'', ch('\'')),
                    value('\"', ch('"')),
                )),
            ),
            // single character
            one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"),
        )),
        ch('\''),
    );

    map(c, LispVal::Character).parse(input)
}

// Assuming LispVal enum exists
fn parse_symbol(input: &str) -> IResult<&str, LispVal> {

    let first_char = alt((alphanumeric1, recognize(one_of("!$%&*/:<=>?^_~"))));
    let rest_char = many0(alt((
        alphanumeric1,
        recognize(one_of("!$%&*/:<=>?^_~")),
    )));

    let parser = recognize(pair(first_char, rest_char));

    let mut parser = nom::combinator::map(parser, |s: &str| LispVal::Symbol(s.to_string()));
    parser.parse(input)
}

// Forward declaration for recursive parsing
fn parse_expr(input: &str) -> IResult<&str, LispVal> {
    alt((
        parse_integer,
        parse_character,
        parse_symbol,
        parse_bool,
        parse_list, // recursion
    ))
    .parse(input)
}

// List parser
fn parse_list(input: &str) -> IResult<&str, LispVal> {
    map(
        delimited(ch('('), separated_list0(ch(' '), parse_expr), ch(')')),
        LispVal::List,
    )
    .parse(input)
}

// Top-level parser
pub fn parse_lisp(input: &str) -> IResult<&str, LispVal> {
    parse_expr(input)
}

mod tests {
    use crate::parser::parse_lisp;

    #[test]
    fn test_parsing() {
        let src = "(define x 42 'a'  (list 1 2 3))";
        match parse_lisp(src) {
            Ok((rest, val)) => {
                println!("Parsed: {:#?}", val);
                println!("Remaining: {:?}", rest);
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
    #[test]
    fn test_parsing_let(){
        let src = "(let ((a 1) (b 1)) a)";
        match parse_lisp(src) {
            Ok((rest, val)) => {
                println!("Parsed: {:#?}", val);
                println!("Remaining: {:?}", rest);
            }
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }
}
