use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Ident(String),
    String(String),
    Number(String),
    LParen, RParen, LBrace, RBrace, Colon, Comma, Equal,
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
            ',' => { i += 1; Kind::Comma }
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
    Call { name: String, args: Vec<Expr> },
    Unary { op: Kind, value: Box<Expr> },
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
}

#[derive(Debug)]
enum Stmt { Let { name: String, value: Expr }, Print(Expr), Return(Expr), If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> } }

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
            Kind::Ident(s) if matches!(&self.current().kind, Kind::LParen) => {
                self.advance();
                let mut args = Vec::new();
                if !matches!(&self.current().kind, Kind::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if matches!(&self.current().kind, Kind::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Kind::RParen, "')' after function arguments")?;
                let expected = match s.as_str() {
                    "input" => Some((0, 1)),
                    "len" | "abs" | "sqrt" | "floor" | "ceil" | "round" | "to_string" |
                    "parse_int" | "is_even" | "is_odd" | "sign" => Some((1, 1)),
                    "min" | "max" | "pow" | "contains" | "starts_with" | "ends_with" => Some((2, 2)),
                    "clamp" => Some((3, 3)),
                    _ => return Err(format!("line {}: unknown function '{s}'", token.line)),
                }.expect("known function");
                if args.len() < expected.0 || args.len() > expected.1 {
                    let expected_text = if expected.0 == expected.1 { expected.0.to_string() } else { format!("{} or {}", expected.0, expected.1) };
                    return Err(format!("line {}: function '{s}' expects {expected_text} argument(s), got {}", token.line, args.len()));
                }
                Ok(Expr::Call { name: s, args })
            }
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
    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(Kind::LBrace, "'{' to start block")?;
        let mut statements = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(&self.current().kind, Kind::RBrace) { self.advance(); return Ok(statements); }
            if matches!(&self.current().kind, Kind::Eof) { return Err(format!("line {}: expected '}}' to close block", self.current().line)); }
            statements.push(self.statement()?);
        }
    }
    fn statement(&mut self) -> Result<Stmt, String> {
        match &self.current().kind {
            Kind::Ident(word) if word == "let" || word == "var" => {
                self.advance(); let name = self.take_name("a variable name")?;
                if matches!(&self.current().kind, Kind::Colon) { self.advance(); let _type_name = self.take_name("a type name")?; }
                self.expect(Kind::Equal, "'=' after variable declaration")?;
                Ok(Stmt::Let { name, value: self.expression()? })
            }
            Kind::Ident(word) if word == "print" => {
                self.advance(); self.expect(Kind::LParen, "'(' after print")?;
                let value = self.expression()?; self.expect(Kind::RParen, "')' after print value")?; Ok(Stmt::Print(value))
            }
            Kind::Ident(word) if word == "return" => { self.advance(); Ok(Stmt::Return(self.expression()?)) }
            Kind::Ident(word) if word == "if" => {
                self.advance(); let condition = self.expression()?; let then_branch = self.block()?;
                self.skip_newlines();
                let else_branch = if matches!(&self.current().kind, Kind::Ident(word) if word == "else") { self.advance(); Some(self.block()?) } else { None };
                Ok(Stmt::If { condition, then_branch, else_branch })
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
            '\n' => out.push_str("\n"),
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
        Expr::Call { name, args } => {
            let values: Vec<String> = args.iter().map(cpp_expr).collect();
            match name.as_str() {
                "len" => format!("static_cast<long long>(cinder_len({}))", values[0]),
                "abs" => format!("std::abs({})", values[0]),
                "min" => format!("std::min({}, {})", values[0], values[1]),
                "max" => format!("std::max({}, {})", values[0], values[1]),
                "clamp" => format!("std::clamp({}, {}, {})", values[0], values[1], values[2]),
                "sqrt" => format!("std::sqrt({})", values[0]),
                "pow" => format!("std::pow({}, {})", values[0], values[1]),
                "floor" => format!("std::floor({})", values[0]),
                "ceil" => format!("std::ceil({})", values[0]),
                "round" => format!("std::round({})", values[0]),
                "input" => format!("cinder_input({})", values.join(", ")),
                "to_string" => format!("cinder_to_string({})", values[0]),
                "parse_int" => format!("cinder_parse_int({})", values[0]),
                "contains" => format!("({}.find({}) != std::string::npos)", values[0], values[1]),
                "starts_with" => format!("({}.starts_with({}))", values[0], values[1]),
                "ends_with" => format!("({}.ends_with({}))", values[0], values[1]),
                "is_even" => format!("(({}) % 2 == 0)", values[0]),
                "is_odd" => format!("(({}) % 2 != 0)", values[0]),
                "sign" => format!("(({}) > 0 ? 1 : (({}) < 0 ? -1 : 0))", values[0], values[0]),
                _ => unreachable!("function names validated by parser"),
            }
        }
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

fn emit_statements(statements: &[Stmt], cpp: &mut String, indent: usize) {
    let pad = "    ".repeat(indent);
    for stmt in statements {
        match stmt {
            Stmt::Let { name, value } => { writeln!(cpp, "{pad}auto {name} = {};", cpp_expr(value)).unwrap(); }
            Stmt::Print(value) => { writeln!(cpp, "{pad}std::cout << {} << '\\n';", cpp_expr(value)).unwrap(); }
            Stmt::Return(value) => { writeln!(cpp, "{pad}return {};", cpp_expr(value)).unwrap(); }
            Stmt::If { condition, then_branch, else_branch } => {
                writeln!(cpp, "{pad}if ({}) {{", cpp_expr(condition)).unwrap();
                emit_statements(then_branch, cpp, indent + 1);
                if let Some(else_branch) = else_branch {
                    writeln!(cpp, "{pad}}} else {{").unwrap();
                    emit_statements(else_branch, cpp, indent + 1);
                }
                writeln!(cpp, "{pad}}}").unwrap();
            }
        }
    }
}

/// Parse the current Cinder subset and emit C++ for native compilation.
/// This bootstrap does not yet perform static type checking.
pub fn compile_to_cpp(source: &str) -> Result<String, String> {
    let tokens = lex(source)?;
    let mut parser = Parser { tokens, at: 0 };
    let statements = parser.program()?;
    let mut cpp = String::from(
        "#include <algorithm>\n#include <cmath>\n#include <iostream>\n#include <stdexcept>\n#include <string>\n#include <type_traits>\n\n"
    );
    cpp.push_str(r#"
inline long long cinder_len(const std::string& value) {
    return static_cast<long long>(value.size());
}

inline std::string cinder_input(const std::string& prompt = {}) {
    if (!prompt.empty()) {
        std::cout << prompt << std::flush;
    }
    std::string value;
    if (!std::getline(std::cin, value)) {
        return {};
    }
    return value;
}

inline std::string cinder_to_string(const std::string& value) {
    return value;
}

inline std::string cinder_to_string(const char* value) {
    return value == nullptr ? std::string{} : std::string(value);
}

inline std::string cinder_to_string(bool value) {
    return value ? "true" : "false";
}

template <typename T>
    requires std::is_arithmetic_v<T>
inline std::string cinder_to_string(T value) {
    if constexpr (std::is_same_v<T, char> || std::is_same_v<T, signed char> ||
                  std::is_same_v<T, unsigned char>) {
        return std::to_string(static_cast<int>(value));
    } else {
        return std::to_string(value);
    }
}

inline long long cinder_parse_int(const std::string& value) {
    std::size_t consumed = 0;
    const long long result = std::stoll(value, &consumed, 10);
    if (consumed != value.size()) {
        throw std::invalid_argument("parse_int: trailing characters in integer");
    }
    return result;
}
"#);
    cpp.push_str("\nint main() {\n");
    emit_statements(&statements, &mut cpp, 1);
    cpp.push_str("    return 0;\n}\n");
    Ok(cpp)
}


#[cfg(test)]
mod tests {
    use super::compile_to_cpp;

    #[test]
    fn emits_common_standard_library_calls() {
        let source = "fn main() {\n let name = \"Cinder\"\n print(len(name))\n print(max(4, 9))\n print(clamp(12, 0, 10))\n print(contains(name, \"in\"))\n print(is_even(8))\n}\n";
        let cpp = compile_to_cpp(source).unwrap();
        assert!(cpp.contains("cinder_len(name)"));
        assert!(cpp.contains("std::max(4, 9)"));
        assert!(cpp.contains("std::clamp(12, 0, 10)"));
        assert!(cpp.contains("std::string::npos") || cpp.contains(".find("));
        assert!(cpp.contains("% 2 == 0"));
    }

    #[test]
    fn rejects_unknown_functions_and_wrong_argument_counts() {
        assert!(compile_to_cpp("fn main() { print(magic(1)) }").unwrap_err().contains("unknown function"));
        assert!(compile_to_cpp("fn main() { print(min(1)) }").unwrap_err().contains("expects 2 argument"));
    }

    #[test]
    fn emits_runtime_helpers_and_uses_cxx20_string_operations() {
        let cpp = compile_to_cpp(
            r#"fn main() {
                let text = input()
                print(to_string(contains(text, "x")))
                print(parse_int("42"))
                print(starts_with(text, "a"))
                print(ends_with(text, "z"))
            }"#,
        )
        .unwrap();
        assert!(cpp.contains("inline long long cinder_len"));
        assert!(cpp.contains("inline std::string cinder_input"));
        assert!(cpp.contains("inline std::string cinder_to_string(bool value)"));
        assert!(cpp.contains("inline long long cinder_parse_int"));
        assert!(cpp.contains(".starts_with("));
        assert!(cpp.contains(".ends_with("));
    }

    #[test]
    fn compiles_nested_control_flow_and_string_concatenation() {
        let cpp = compile_to_cpp(
            r#"fn main() {
                let name = "Cinder"
                if contains(name, "in") {
                    print("Hello, " + name)
                } else {
                    print("Nope")
                }
            }"#,
        )
        .unwrap();
        assert!(cpp.contains("find(std::string(\"in\"))"));
        assert!(cpp.contains("std::string(\"Hello, \") + name"));
    }

    #[test]
    fn emits_math_and_input_helpers() {
        let cpp = compile_to_cpp("fn main() {\n let n = sqrt(81)\n let answer = input(\"Name? \")\n print(to_string(n))\n print(answer)\n}\n").unwrap();
        assert!(cpp.contains("std::sqrt(81)"));
        assert!(cpp.contains("cinder_input(std::string(\"Name? \"))"));
        assert!(cpp.contains("cinder_to_string(n)"));
    }
}
