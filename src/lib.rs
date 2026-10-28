use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Ident(String), String(String), Number(String),
    LParen, RParen, LBrace, RBrace, Colon, Equal, Newline, Eof,
}
#[derive(Clone, Debug)]
struct Token { kind: Kind, line: usize }

fn lex(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let (mut i, mut line) = (0, 1);
    let mut tokens = Vec::new();
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' => i += 1,
            '\n' => { tokens.push(Token { kind: Kind::Newline, line }); line += 1; i += 1; }
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' { i += 1; }
            }
            '(' => { tokens.push(Token { kind: Kind::LParen, line }); i += 1; }
            ')' => { tokens.push(Token { kind: Kind::RParen, line }); i += 1; }
            '{' => { tokens.push(Token { kind: Kind::LBrace, line }); i += 1; }
            '}' => { tokens.push(Token { kind: Kind::RBrace, line }); i += 1; }
            ':' => { tokens.push(Token { kind: Kind::Colon, line }); i += 1; }
            '=' => { tokens.push(Token { kind: Kind::Equal, line }); i += 1; }
            '"' => {
                i += 1;
                let start_line = line;
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
                tokens.push(Token { kind: Kind::String(value), line: start_line });
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                tokens.push(Token { kind: Kind::Number(chars[start..i].iter().collect()), line });
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') { i += 1; }
                tokens.push(Token { kind: Kind::Ident(chars[start..i].iter().collect()), line });
            }
            other => return Err(format!("line {line}: unexpected character '{other}'")),
        }
    }
    tokens.push(Token { kind: Kind::Eof, line });
    Ok(tokens)
}

#[derive(Debug)]
enum Expr { Text(String), Number(String), Bool(bool), Name(String) }
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
    fn expression(&mut self) -> Result<Expr, String> {
        let token = self.advance();
        match token.kind {
            Kind::String(s) => Ok(Expr::Text(s)),
            Kind::Number(n) => Ok(Expr::Number(n)),
            Kind::Ident(s) if s == "true" => Ok(Expr::Bool(true)),
            Kind::Ident(s) if s == "false" => Ok(Expr::Bool(false)),
            Kind::Ident(s) => Ok(Expr::Name(s)),
            _ => Err(format!("line {}: expected a string, integer, boolean, or variable name", token.line)),
        }
    }
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
    }
}

/// Parse the initial Cinder language subset and emit C++ for native compilation.
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
