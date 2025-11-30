use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char as ch, digit1, multispace0, multispace1, one_of},
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
    let first_char = alt((alphanumeric1, recognize(one_of("+-!$%&*/:<=>?^_~"))));
    let rest_char = many0(alt((alphanumeric1, recognize(one_of("!$%&*/:<=>?^_~")))));

    let parser = recognize(pair(first_char, rest_char));

    let mut parser = nom::combinator::map(parser, |s: &str| LispVal::Symbol(s.to_string()));
    parser.parse(input)
}

// List parser
fn parse_list(input: &str) -> IResult<&str, LispVal> {
    map(
        delimited(
            pair(ch('('), multispace0),
            separated_list0(multispace1, parse_expr),
            pair(multispace0, ch(')')),
        ),
        LispVal::List,
    )
    .parse(input)
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

// Top-level parser
pub fn parse_lisp(input: &str) -> IResult<&str, LispVal> {
    parse_expr(input)
}

impl LispVal {
    // implement a simple static checking later.
    pub fn verify(&self) -> Result<(), String> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{LispVal, parse_lisp};
    #[test]
    fn test_parsing_simple() {
        let src = "(define x 42 'a'  (list 1 2 3))";
        match parse_lisp(src) {
            Ok((rest, val)) => {
                if rest != "" {
                    assert!(false);
                }
                match val {
                    LispVal::List(x) => {
                        assert!(x[0].eq(&LispVal::Symbol("define".to_string())));
                        assert!(x[1].eq(&LispVal::Symbol("x".to_string())));
                        assert!(x[2].eq(&LispVal::Integer(42)));
                    }
                    _ => assert!(false),
                }
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
    #[test]
    fn test_parsing_let() {
        let src = "(let ((a 1) (b 1)) a)";
        match parse_lisp(src) {
            Ok((rest, val)) => {
                assert_eq!(rest, "".to_string());
                match val {
                    LispVal::List(x) => {
                        assert_eq!(x[0], LispVal::Symbol("let".to_string()));
                        assert_eq!(
                            x[1],
                            LispVal::List(vec![
                                LispVal::List(vec![
                                    LispVal::Symbol("a".to_string()),
                                    LispVal::Integer(1)
                                ]),
                                LispVal::List(vec![
                                    LispVal::Symbol("b".to_string()),
                                    LispVal::Integer(1)
                                ]),
                            ])
                        );
                        assert_eq!(x[2], LispVal::Symbol("a".to_string()))
                    }
                    _ => {
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}
