use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Ident(String),
    String(String),
    Number(String),
    LParen, RParen, LBrace, RBrace, Colon, Equal,
    Plus, Minus, Star, Slash, Percent, Bang,
    Less, LessEqual, Greater, GreaterEqual, EqualEqual, BangEqual,
    AndAnd, OrOr, Newline, Eof,
}

#[derive(Clone, Debug)]
struct Token { kind: Kind, line: usize }

fn lex(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let (mut i, mut line) = (0, 1);
    let mut tokens = Vec::new();
    while i < chars.len() {
        let start_line = line;
        let kind = match chars[i] {
            ' ' | '\t' | '\r' => { i += 1; continue; }
            '\n' => { i += 1; line += 1; Kind::Newline }
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' { i += 1; }
                continue;
            }
            '(' => { i += 1; Kind::LParen }
            ')' => { i += 1; Kind::RParen }
            '{' => { i += 1; Kind::LBrace }
            '}' => { i += 1; Kind::RBrace }
            ':' => { i += 1; Kind::Colon }
            '+' => { i += 1; Kind::Plus }
            '-' => { i += 1; Kind::Minus }
            '*' => { i += 1; Kind::Star }
            '%' => { i += 1; Kind::Percent }
            '!' if chars.get(i + 1) == Some(&'=') => { i += 2; Kind::BangEqual }
            '!' => { i += 1; Kind::Bang }
            '=' if chars.get(i + 1) == Some(&'=') => { i += 2; Kind::EqualEqual }
            '=' => { i += 1; Kind::Equal }
            '<' if chars.get(i + 1) == Some(&'=') => { i += 2; Kind::LessEqual }
            '<' => { i += 1; Kind::Less }
            '>' if chars.get(i + 1) == Some(&'=') => { i += 2; Kind::GreaterEqual }
            '>' => { i += 1; Kind::Greater }
            '&' if chars.get(i + 1) == Some(&'&') => { i += 2; Kind::AndAnd }
            '|' if chars.get(i + 1) == Some(&'|') => { i += 2; Kind::OrOr }
            '/' => { i += 1; Kind::Slash }
            '"' => {
                i += 1;
                let mut value = String::new();
                let mut closed = false;
                while i < chars.len() {
                    match chars[i] {
                        '"' => { i += 1; closed = true; break; }
                        '\\' => {
                            i += 1;
                            let escaped = *chars.get(i).ok_or_else(|| format!("line {line}: unfinished escape"))?;
                            value.push(match escaped {
                                'n' => '\n', 'r' => '\r', 't' => '\t', '"' => '"', '\\' => '\\',
                                other => return Err(format!("line {line}: unsupported escape \\{other}")),
                            });
                            i += 1;
                        }
                        '\n' => return Err(format!("line {start_line}: newline inside string literal")),
                        ch => { value.push(ch); i += 1; }
                    }
                }
                if !closed { return Err(format!("line {start_line}: unterminated string literal")); }
                Kind::String(value)
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                Kind::Number(chars[start..i].iter().collect())
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                Kind::Ident(chars[start..i].iter().collect())
            }
            other => return Err(format!("line {line}: unexpected character '{other}'")),
        };
        tokens.push(Token { kind, line: start_line });
    }
    tokens.push(Token { kind: Kind::Eof, line });
    Ok(tokens)
}

#[derive(Clone, Debug)]
enum BinaryOp { Add, Subtract, Multiply, Divide, Remainder, Less, LessEqual, Greater, GreaterEqual, Equal, NotEqual, And, Or }

#[derive(Debug)]
enum Expr {
    Text(String),
    Number(String),
    Bool(bool),
    Name(String),
    Unary { op: Kind, value: Box<Expr> },
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
}

#[derive(Debug)]
enum Stmt { Let { name: String, value: Expr }, Print(Expr), Return(Expr) }

struct Parser { tokens: Vec<Token>, at: usize }

impl Parser {
    fn current(&self) -> &Token { &self.tokens[self.at] }
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.at].clone();
        if !matches!(&token.kind, Kind::Eof) { self.at += 1; }
        token
    }
    fn skip_newlines(&mut self) {
        while matches!(&self.current().kind, Kind::Newline) { self.advance(); }
    }
    fn expect(&mut self, expected: Kind, description: &str) -> Result<(), String> {
        if self.current().kind == expected { self.advance(); Ok(()) }
        else { Err(format!("line {}: expected {description}", self.current().line)) }
    }
    fn expect_word(&mut self, word: &str) -> Result<(), String> {
        match &self.current().kind {
            Kind::Ident(actual) if actual == word => { self.advance(); Ok(()) }
            _ => Err(format!("line {}: expected '{word}'", self.current().line)),
        }
    }
    fn take_name(&mut self, description: &str) -> Result<String, String> {
        match self.advance() {
            Token { kind: Kind::Ident(name), line } if !matches!(name.as_str(), "fn" | "let" | "var" | "return" | "true" | "false") => Ok(name),
            token => Err(format!("line {}: expected {description}", token.line)),
        }
    }
    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.advance();
        match token.kind {
            Kind::String(s) => Ok(Expr::Text(s)),
            Kind::Number(n) => Ok(Expr::Number(n)),
            Kind::Ident(s) if s == "true" => Ok(Expr::Bool(true)),
            Kind::Ident(s) if s == "false" => Ok(Expr::Bool(false)),
            Kind::Ident(s) => Ok(Expr::Name(s)),
            Kind::LParen => {
                let value = self.expression()?;
                self.expect(Kind::RParen, "')' after expression")?;
                Ok(value)
            }
            _ => Err(format!("line {}: expected an expression", token.line)),
        }
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if matches!(&self.current().kind, Kind::Minus | Kind::Bang) {
            let op = self.advance().kind;
            let value = self.unary()?;
            Ok(Expr::Unary { op, value: Box::new(value) })
        } else {
            self.primary()
        }
    }
    fn precedence(kind: &Kind) -> Option<(u8, BinaryOp)> {
        Some(match kind {
            Kind::OrOr => (1, BinaryOp::Or),
            Kind::AndAnd => (2, BinaryOp::And),
            Kind::EqualEqual => (3, BinaryOp::Equal),
            Kind::BangEqual => (3, BinaryOp::NotEqual),
            Kind::Less => (4, BinaryOp::Less),
            Kind::LessEqual => (4, BinaryOp::LessEqual),
            Kind::Greater => (4, BinaryOp::Greater),
            Kind::GreaterEqual => (4, BinaryOp::GreaterEqual),
            Kind::Plus => (5, BinaryOp::Add),
            Kind::Minus => (5, BinaryOp::Subtract),
            Kind::Star => (6, BinaryOp::Multiply),
            Kind::Slash => (6, BinaryOp::Divide),
            Kind::Percent => (6, BinaryOp::Remainder),
            _ => return None,
        })
    }
    fn binary(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let mut left = self.unary()?;
        while let Some((precedence, _)) = Self::precedence(&self.current().kind) {
            if precedence < min_precedence { break; }
            let (_, op) = Self::precedence(&self.advance().kind).expect("operator was checked");
            let right = self.binary(precedence + 1)?;
            left = Expr::Binary { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }
    fn expression(&mut self) -> Result<Expr, String> { self.binary(1) }
    fn statement(&mut self) -> Result<Stmt, String> {
        match &self.current().kind {
            Kind::Ident(word) if word == "let" || word == "var" => {
                self.advance();
                let name = self.take_name("a variable name")?;
                if matches!(&self.current().kind, Kind::Colon) {
                    self.advance();
                    let _type_name = self.take_name("a type name")?;
                }
                self.expect(Kind::Equal, "'=' after variable declaration")?;
                Ok(Stmt::Let { name, value: self.expression()? })
            }
            Kind::Ident(word) if word == "print" => {
                self.advance();
                self.expect(Kind::LParen, "'(' after print")?;
                let value = self.expression()?;
                self.expect(Kind::RParen, "')' after print value")?;
                Ok(Stmt::Print(value))
            }
            Kind::Ident(word) if word == "return" => {
                self.advance();
                Ok(Stmt::Return(self.expression()?))
            }
            _ => Err(format!("line {}: unsupported statement", self.current().line)),
        }
    }
    fn program(&mut self) -> Result<Vec<Stmt>, String> {
        self.skip_newlines();
        self.expect_word("fn")?;
        self.expect_word("main")?;
        self.expect(Kind::LParen, "'(' after main")?;
        self.expect(Kind::RParen, "')' after main")?;
        self.expect(Kind::LBrace, "'{' to start main body")?;
        let mut statements = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(&self.current().kind, Kind::RBrace) { self.advance(); break; }
            if matches!(&self.current().kind, Kind::Eof) {
                return Err(format!("line {}: expected '}}' to close main", self.current().line));
            }
            statements.push(self.statement()?);
        }
        self.skip_newlines();
        if !matches!(&self.current().kind, Kind::Eof) {
            return Err(format!("line {}: only one main function is supported in this prototype", self.current().line));
        }
        Ok(statements)
    }
}

fn cpp_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => { let _ = write!(out, "\\u{:04x}", c as u32); }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn cpp_expr(expr: &Expr) -> String {
    match expr {
        Expr::Text(s) => format!("std::string({})", cpp_string(s)),
        Expr::Number(n) => n.clone(),
        Expr::Bool(b) => b.to_string(),
        Expr::Name(n) => n.clone(),
        Expr::Unary { op, value } => {
            let symbol = match op { Kind::Minus => "-", Kind::Bang => "!", _ => unreachable!() };
            format!("({symbol}{})", cpp_expr(value))
        }
        Expr::Binary { left, op, right } => {
            let symbol = match op {
                BinaryOp::Add => "+", BinaryOp::Subtract => "-", BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/", BinaryOp::Remainder => "%", BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=", BinaryOp::Greater => ">", BinaryOp::GreaterEqual => ">=",
                BinaryOp::Equal => "==", BinaryOp::NotEqual => "!=", BinaryOp::And => "&&",
                BinaryOp::Or => "||",
            };
            format!("({} {symbol} {})", cpp_expr(left), cpp_expr(right))
        }
    }
}

/// Parse the current Cinder subset and emit C++ for native compilation.
/// This bootstrap does not yet perform static type checking.
pub fn compile_to_cpp(source: &str) -> Result<String, String> {
    let tokens = lex(source)?;
    let mut parser = Parser { tokens, at: 0 };
    let statements = parser.program()?;
    let mut cpp = String::from("#include <iostream>\n#include <string>\n\nint main() {\n");
    for stmt in statements {
        match stmt {
            Stmt::Let { name, value } => { writeln!(cpp, "    auto {name} = {};", cpp_expr(&value)).unwrap(); }
            Stmt::Print(value) => { writeln!(cpp, "    std::cout << {} << '\\n';", cpp_expr(&value)).unwrap(); }
            Stmt::Return(value) => { writeln!(cpp, "    return {};", cpp_expr(&value)).unwrap(); }
        }
    }
    cpp.push_str("    return 0;\n}\n");
    Ok(cpp)
}

#[cfg(test)]
mod tests {
    use super::compile_to_cpp;

    #[test]
    fn compiles_hello_world_to_native_source() {
        let cpp = compile_to_cpp("fn main() {\n    print(\"Hello, Cinder!\")\n}\n").unwrap();
        assert!(cpp.contains("std::cout"));
        assert!(cpp.contains("Hello, Cinder!"));
    }

    #[test]
    fn supports_variables_and_inferred_types() {
        let cpp = compile_to_cpp("fn main() {\n let count: i32 = 42\n print(count)\n}\n").unwrap();
        assert!(cpp.contains("auto count = 42;"));
        assert!(cpp.contains("std::cout << count"));
    }

    #[test]
    fn respects_operator_precedence() {
        let cpp = compile_to_cpp("fn main() {\n print(2 + 3 * 4)\n}\n").unwrap();
        assert!(cpp.contains("(2 + (3 * 4))"));
    }

    #[test]
    fn supports_comparisons_and_logical_operators() {
        let cpp = compile_to_cpp("fn main() {\n print(3 < 4 && 5 != 6)\n}\n").unwrap();
        assert!(cpp.contains("((3 < 4) && (5 != 6))"));
    }

    #[test]
    fn supports_unary_operators_and_grouping() {
        let cpp = compile_to_cpp("fn main() {\n print(-(2 + 3))\n print(!false)\n}\n").unwrap();
        assert!(cpp.contains("(-(2 + 3))"));
        assert!(cpp.contains("(!false)"));
    }

    #[test]
    fn rejects_unclosed_function_body() {
        let err = compile_to_cpp("fn main() {\n print(\"oops\")\n").unwrap_err();
        assert!(err.contains("expected '}'"));
    }

    #[test]
    fn rejects_unsupported_characters() {
        let err = compile_to_cpp("fn main() {\n print(@)\n}\n").unwrap_err();
        assert!(err.contains("unexpected character"));
    }
}
