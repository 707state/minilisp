use std::ffi::CString;

use crate::ast::*;

pub struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            src: input.as_bytes(),
            pos: 0,
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_spaces(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Main entry: parse one s-expression
    pub fn parse_expr(&mut self) -> ASTNode {
        self.skip_spaces();
        if self.eof() {
            return AST_error();
        }

        match self.peek() {
            Some(b'(') => self.parse_list(),
            Some(b'\'') => self.parse_char(),
            Some(c) if c.is_ascii_digit() || c == b'-' => self.parse_integer(),
            Some(_) => self.parse_symbol(),
            None => AST_error(),
        }
    }

    /// Parse: ( f arg0 arg1 arg2 )
    fn parse_list(&mut self) -> ASTNode {
        self.bump(); // '('
        self.skip_spaces();

        // Read elements
        let mut items: Vec<ASTNode> = Vec::new();

        while let Some(c) = self.peek() {
            if c == b')' {
                break;
            }
            let expr = self.parse_expr();
            if AST_is_error(expr) {
                return AST_error();
            }
            items.push(expr);
            self.skip_spaces();
        }

        if self.peek() != Some(b')') {
            return AST_error();
        }
        self.bump(); // ')'

        // Arity rules
        if items.is_empty() {
            // () is NIL
            return AST_nil();
        }
        if items.len() > 4 {
            // Only allow: f, f x, f x y, f x y z
            return AST_error();
        }

        // Turn Vec<ASTNode> into pairs
        let mut acc = AST_nil();
        for item in items.into_iter().rev() {
            acc = AST_new_pair(item, acc);
        }
        acc
    }

    /// Parse integer: 123 or -45
    fn parse_integer(&mut self) -> ASTNode {
        let start = self.pos;

        if self.peek() == Some(b'-') {
            self.bump();
        }

        let mut has_digit = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.bump();
                has_digit = true;
            } else {
                break;
            }
        }

        if !has_digit {
            return AST_error();
        }

        let s = std::str::from_utf8(&self.src[start..self.pos]).unwrap();
        let value: i64 = match s.parse() {
            Ok(v) => v,
            Err(_) => return AST_error(),
        };

        AST_new_integer(value)
    }
    /// Parse C-style character literal: 'a', '\n', '\\', '\'', '\x41'
    fn parse_char(&mut self) -> ASTNode {
        // Leading quote
        if self.bump() != Some(b'\'') {
            return AST_error();
        }

        let ch = match self.bump() {
            None => return AST_error(),
            Some(b'\\') => {
                // escape sequence
                match self.bump() {
                    Some(b'n') => b'\n',
                    Some(b't') => b'\t',
                    Some(b'\\') => b'\\',
                    Some(b'\'') => b'\'',
                    Some(b'"') => b'"',
                    Some(b'r') => b'\r',
                    Some(b'0') => b'\0',

                    // hex: \xNN
                    Some(b'x') => {
                        let h1 = self.bump().unwrap_or(b'?');
                        let h2 = self.bump().unwrap_or(b'?');
                        let hex = [h1, h2];
                        match std::str::from_utf8(&hex)
                            .ok()
                            .and_then(|s| u8::from_str_radix(s, 16).ok())
                        {
                            Some(v) => v,
                            None => return AST_error(),
                        }
                    }

                    _ => return AST_error(), // unsupported escape
                }
            }
            Some(c) => c,
        };

        // Expect trailing quote
        if self.bump() != Some(b'\'') {
            return AST_error();
        }

        AST_new_char(ch as i32)
    }

    /// Parse symbol: alpha or punctuation except () and numbers
    fn parse_symbol(&mut self) -> ASTNode {
        let start = self.pos;

        while let Some(c) = self.peek() {
            match c {
                // forbidden chars
                b'(' | b')' | b' ' | b'\t' | b'\n' | b'\r' => break,
                _ => self.bump(),
            };
        }

        let name = &self.src[start..self.pos];

        if name.is_empty() {
            return AST_error();
        }

        let cstr = CString::new(name).unwrap();
        AST_new_symbol(&cstr)
    }
}
#[cfg(test)]
mod tests {
    use crate::parser::*;

    #[test]
    fn parsing_simple_tuple() {
        let mut p = Parser::new("(add1 2)");
        let ast = p.parse_expr();
        assert!(AST_is_pair(ast));
    }
    #[test]
    fn parsing_simple_char() {
        assert_eq!(AST_get_char(Parser::new("'a'").parse_expr()), b'a' as i8);
        assert_eq!(AST_get_char(Parser::new("'\\n'").parse_expr()), b'\n' as i8);
        assert_eq!(AST_get_char(Parser::new("'\\''").parse_expr()), b'\'' as i8);
        assert_eq!(
            AST_get_char(Parser::new("'\\\\'").parse_expr()),
            b'\\' as i8
        );
        assert_eq!(
            AST_get_char(Parser::new("'\\x41'").parse_expr()),
            0x41 as i8
        );
    }
}
