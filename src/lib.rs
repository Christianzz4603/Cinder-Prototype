use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Ident(String),
    String(String),
    Number(String),
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Colon, Comma, Equal, Arrow,
    Plus, Minus, Star, Slash, Percent, Bang,
    Less, LessEqual, Greater, GreaterEqual, EqualEqual, BangEqual, DotDot,
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
            '[' => { i += 1; Kind::LBracket }
            ']' => { i += 1; Kind::RBracket }
            ':' => { i += 1; Kind::Colon }
            '.' if chars.get(i + 1) == Some(&'.') => { i += 2; Kind::DotDot }
            ',' => { i += 1; Kind::Comma }
            '+' => { i += 1; Kind::Plus }
            '-' if chars.get(i + 1) == Some(&'>') => { i += 2; Kind::Arrow },
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
    Array(Vec<Expr>),
    Index { value: Box<Expr>, index: Box<Expr> },
    Unary { op: Kind, value: Box<Expr> },
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
}

#[derive(Debug)]
enum Stmt { Let { name: String, declared_type: Option<String>, mutable: bool, value: Expr }, Assign { name: String, value: Expr }, IndexAssign { name: String, index: Expr, value: Expr }, Push { name: String, value: Expr }, Print(Expr), Return(Expr), Break, Continue, If { condition: Expr, then_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> }, While { condition: Expr, body: Vec<Stmt> }, ForRange { name: String, start: Expr, end: Expr, body: Vec<Stmt> }, ForEach { name: String, iterable: Expr, body: Vec<Stmt> } }

#[derive(Debug)]
struct Function { name: String, params: Vec<(String, String)>, return_type: String, body: Vec<Stmt> }

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
            Token { kind: Kind::Ident(name), line } if !matches!(name.as_str(), "fn" | "let" | "var" | "return" | "break" | "continue" | "for" | "true" | "false") => Ok(name),
            token => Err(format!("line {}: expected {description}", token.line)),
        }
    }
    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.advance();
        match token.kind {
            Kind::String(s) => Ok(Expr::Text(s)),
            Kind::LBracket => {
                let mut items = Vec::new();
                if !matches!(&self.current().kind, Kind::RBracket) {
                    loop {
                        items.push(self.expression()?);
                        if matches!(&self.current().kind, Kind::Comma) { self.advance(); } else { break; }
                    }
                }
                self.expect(Kind::RBracket, "']' after array elements")?;
                Ok(Expr::Array(items))
            }
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
                    _ => None,
                };
                if let Some(expected) = expected {
                    if args.len() < expected.0 || args.len() > expected.1 {
                        let expected_text = if expected.0 == expected.1 { expected.0.to_string() } else { format!("{} or {}", expected.0, expected.1) };
                        return Err(format!("line {}: function '{s}' expects {expected_text} argument(s), got {}", token.line, args.len()));
                    }
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
    fn postfix(&mut self) -> Result<Expr, String> {
        let mut value = self.primary()?;
        while matches!(&self.current().kind, Kind::LBracket) {
            self.advance();
            let index = self.expression()?;
            self.expect(Kind::RBracket, "']' after array index")?;
            value = Expr::Index { value: Box::new(value), index: Box::new(index) };
        }
        Ok(value)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if matches!(&self.current().kind, Kind::Minus | Kind::Bang) {
            let op = self.advance().kind;
            let value = self.unary()?;
            Ok(Expr::Unary { op, value: Box::new(value) })
        } else {
            self.postfix()
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
                let mutable = word == "var";
                self.advance();
                let name = self.take_name("a variable name")?;
                let declared_type = if matches!(&self.current().kind, Kind::Colon) {
                    self.advance();
                    Some(self.parse_type("a variable type")?)
                } else { None };
                self.expect(Kind::Equal, "'=' after variable declaration")?;
                Ok(Stmt::Let { name, declared_type, mutable, value: self.expression()? })
            }
            Kind::Ident(word) if word == "push" => {
                self.advance();
                self.expect(Kind::LParen, "'(' after push")?;
                let name = self.take_name("a mutable array variable")?;
                self.expect(Kind::Comma, "',' after array variable")?;
                let value = self.expression()?;
                self.expect(Kind::RParen, "')' after pushed value")?;
                Ok(Stmt::Push { name, value })
            }
            Kind::Ident(word) if word == "print" => {
                self.advance(); self.expect(Kind::LParen, "'(' after print")?;
                let value = self.expression()?; self.expect(Kind::RParen, "')' after print value")?; Ok(Stmt::Print(value))
            }
            Kind::Ident(word) if word == "return" => { self.advance(); Ok(Stmt::Return(self.expression()?)) }
            Kind::Ident(word) if word == "break" => { self.advance(); Ok(Stmt::Break) }
            Kind::Ident(word) if word == "continue" => { self.advance(); Ok(Stmt::Continue) }
            Kind::Ident(name) if matches!(self.tokens.get(self.at + 1).map(|token| &token.kind), Some(Kind::LBracket)) => {
                let name = name.clone();
                self.advance();
                self.expect(Kind::LBracket, "'[' after array name")?;
                let index = self.expression()?;
                self.expect(Kind::RBracket, "']' after array index")?;
                self.expect(Kind::Equal, "'=' in indexed assignment")?;
                let value = self.expression()?;
                Ok(Stmt::IndexAssign { name, index, value })
            }
            Kind::Ident(name) if matches!(self.tokens.get(self.at + 1).map(|token| &token.kind), Some(Kind::Equal)) => {
                let name = name.clone();
                self.advance();
                self.expect(Kind::Equal, "'=' in assignment")?;
                Ok(Stmt::Assign { name, value: self.expression()? })
            }
            Kind::Ident(word) if word == "for" => {
                self.advance();
                let name = self.take_name("a loop variable")?;
                self.expect_word("in")?;
                let start = self.expression()?;
                if matches!(&self.current().kind, Kind::DotDot) {
                    self.advance();
                    let end = self.expression()?;
                    let body = self.block()?;
                    Ok(Stmt::ForRange { name, start, end, body })
                } else {
                    let body = self.block()?;
                    Ok(Stmt::ForEach { name, iterable: start, body })
                }
            }
            Kind::Ident(word) if word == "while" => {
                self.advance();
                let condition = self.expression()?;
                let body = self.block()?;
                Ok(Stmt::While { condition, body })
            }
            Kind::Ident(word) if word == "if" => {
                self.advance(); let condition = self.expression()?; let then_branch = self.block()?;
                self.skip_newlines();
                let else_branch = if matches!(&self.current().kind, Kind::Ident(word) if word == "else") { self.advance(); Some(self.block()?) } else { None };
                Ok(Stmt::If { condition, then_branch, else_branch })
            }
            _ => Err(format!("line {}: unsupported statement", self.current().line)),
        }
    }
    fn parse_type(&mut self, description: &str) -> Result<String, String> {
        let ty = self.take_name(description)?;
        match ty.as_str() {
            "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "bool" | "String" => Ok(ty),
            "str" => Ok("String".to_string()),
            "Array" => {
                self.expect(Kind::Less, "'<' after Array")?;
                let element = self.parse_type("an array element type")?;
                self.expect(Kind::Greater, "'>' after array element type")?;
                Ok(format!("Array<{element}>"))
            }
            _ => Err(format!("line {}: unsupported type '{ty}' (supported: i32, i64, u32, u64, f32, f64, bool, String, Array<T>)", self.current().line)),
        }
    }
    fn function(&mut self) -> Result<Function, String> {
        self.expect_word("fn")?;
        let name = self.take_name("a function name")?;
        self.expect(Kind::LParen, "'(' after function name")?;
        let mut params = Vec::new();
        if !matches!(&self.current().kind, Kind::RParen) {
            loop {
                let param_name = self.take_name("a parameter name")?;
                self.expect(Kind::Colon, "':' after parameter name")?;
                let param_type = self.parse_type("a parameter type")?;
                if params.iter().any(|(existing, _)| existing == &param_name) {
                    return Err(format!("line {}: duplicate parameter '{param_name}'", self.current().line));
                }
                params.push((param_name, param_type));
                if matches!(&self.current().kind, Kind::Comma) { self.advance(); }
                else { break; }
            }
        }
        self.expect(Kind::RParen, "')' after parameters")?;
        let return_type = if matches!(&self.current().kind, Kind::Arrow) {
            self.advance();
            self.parse_type("a return type")?
        } else if name == "main" {
            "i32".to_string()
        } else {
            "i64".to_string()
        };
        let body = self.block()?;
        if name == "main" && !params.is_empty() {
            return Err("line 1: main must not declare parameters".to_string());
        }
        Ok(Function { name, params, return_type, body })
    }
    fn program(&mut self) -> Result<Vec<Function>, String> {
        let mut functions = Vec::new();
        self.skip_newlines();
        while !matches!(&self.current().kind, Kind::Eof) {
            let function = self.function()?;
            if functions.iter().any(|existing: &Function| existing.name == function.name) {
                return Err(format!("line {}: duplicate function '{}'", self.current().line, function.name));
            }
            functions.push(function);
            self.skip_newlines();
        }
        if !functions.iter().any(|function| function.name == "main") {
            return Err("program must define fn main()".to_string());
        }
        Ok(functions)
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
        Expr::Array(items) => format!("std::vector{{{}}}", items.iter().map(cpp_expr).collect::<Vec<_>>().join(", ")),
        Expr::Index { value, index } => format!("{}[{}]", cpp_expr(value), cpp_expr(index)),
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
                _ => format!("{name}({})", values.join(", ")),
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
            Stmt::Let { name, declared_type, mutable, value } => {
                let qualifier = if *mutable { "" } else { "const " };
                let ty = declared_type.as_deref().map(cpp_type).unwrap_or_else(|| "auto".to_string());
                writeln!(cpp, "{pad}{qualifier}{ty} {name} = {};", cpp_expr(value)).unwrap();
            }
            Stmt::Assign { name, value } => { writeln!(cpp, "{pad}{name} = {};", cpp_expr(value)).unwrap(); }
            Stmt::IndexAssign { name, index, value } => { writeln!(cpp, "{pad}{name}[{}] = {};", cpp_expr(index), cpp_expr(value)).unwrap(); }
            Stmt::Push { name, value } => { writeln!(cpp, "{pad}{name}.push_back({});", cpp_expr(value)).unwrap(); }
            Stmt::Print(value) => { writeln!(cpp, "{pad}std::cout << {} << '\\n';", cpp_expr(value)).unwrap(); }
            Stmt::Return(value) => { writeln!(cpp, "{pad}return {};", cpp_expr(value)).unwrap(); }
            Stmt::Break => { writeln!(cpp, "{pad}break;").unwrap(); }
            Stmt::Continue => { writeln!(cpp, "{pad}continue;").unwrap(); }
            Stmt::If { condition, then_branch, else_branch } => {
                writeln!(cpp, "{pad}if ({}) {{", cpp_expr(condition)).unwrap();
                emit_statements(then_branch, cpp, indent + 1);
                if let Some(else_branch) = else_branch {
                    writeln!(cpp, "{pad}}} else {{").unwrap();
                    emit_statements(else_branch, cpp, indent + 1);
                }
                writeln!(cpp, "{pad}}}").unwrap();
            }
            Stmt::While { condition, body } => {
                writeln!(cpp, "{pad}while ({}) {{", cpp_expr(condition)).unwrap();
                emit_statements(body, cpp, indent + 1);
                writeln!(cpp, "{pad}}}").unwrap();
            }
            Stmt::ForRange { name, start, end, body } => {
                writeln!(cpp, "{pad}for (std::int64_t {name} = {}; {name} < {}; ++{name}) {{", cpp_expr(start), cpp_expr(end)).unwrap();
                emit_statements(body, cpp, indent + 1);
                writeln!(cpp, "{pad}}}").unwrap();
            }
            Stmt::ForEach { name, iterable, body } => {
                writeln!(cpp, "{pad}for (const auto& {name} : {}) {{", cpp_expr(iterable)).unwrap();
                emit_statements(body, cpp, indent + 1);
                writeln!(cpp, "{pad}}}").unwrap();
            }
        }
    }
}

fn cpp_type(ty: &str) -> String {
    match ty {
        "i32" => "std::int32_t".into(),
        "i64" => "std::int64_t".into(),
        "u32" => "std::uint32_t".into(),
        "u64" => "std::uint64_t".into(),
        "f32" => "float".into(),
        "f64" => "double".into(),
        "bool" => "bool".into(),
        "String" => "std::string".into(),
        array if array.starts_with("Array<") && array.ends_with('>') => {
            format!("std::vector<{}>", cpp_type(&array[6..array.len() - 1]))
        }
        _ => unreachable!("parser validates types"),
    }
}


#[derive(Clone, Debug)]
struct FunctionSignature {
    params: Vec<String>,
    return_type: String,
}

type TypeScopes = Vec<std::collections::HashMap<String, (String, bool)>>;

fn is_integer(ty: &str) -> bool {
    matches!(ty, "i32" | "i64" | "u32" | "u64")
}

fn is_numeric(ty: &str) -> bool {
    is_integer(ty) || matches!(ty, "f32" | "f64")
}

fn compatible_type(actual: &str, expected: &str, expr: &Expr) -> bool {
    actual == expected || (matches!(expr, Expr::Number(_)) && is_numeric(expected) && is_integer(actual))
        || (is_numeric(actual) && is_numeric(expected))
}

fn lookup_variable(scopes: &TypeScopes, name: &str) -> Option<(String, bool)> {
    scopes.iter().rev().find_map(|scope| scope.get(name).cloned())
}

fn infer_expr_type(
    expr: &Expr,
    scopes: &TypeScopes,
    signatures: &std::collections::HashMap<String, FunctionSignature>,
) -> Result<String, String> {
    match expr {
        Expr::Text(_) => Ok("String".into()),
        Expr::Array(items) => {
            if items.is_empty() { return Err("cannot infer the element type of an empty array; add at least one element".into()); }
            let first = infer_expr_type(&items[0], scopes, signatures)?;
            for item in &items[1..] {
                let item_ty = infer_expr_type(item, scopes, signatures)?;
                if item_ty != first { return Err(format!("array elements must have matching types, got {first} and {item_ty}")); }
            }
            Ok(format!("Array<{first}>"))
        }
        Expr::Index { value, index } => {
            let array_ty = infer_expr_type(value, scopes, signatures)?;
            let index_ty = infer_expr_type(index, scopes, signatures)?;
            if !is_integer(&index_ty) { return Err(format!("array index must be an integer, got {index_ty}")); }
            array_ty.strip_prefix("Array<").and_then(|ty| ty.strip_suffix('>')).map(str::to_string)
                .ok_or_else(|| format!("cannot index value of type {array_ty}"))
        }
        Expr::Number(_) => Ok("i64".into()),
        Expr::Bool(_) => Ok("bool".into()),
        Expr::Name(name) => lookup_variable(scopes, name).map(|item| item.0)
            .ok_or_else(|| format!("unknown variable '{name}'")),
        Expr::Unary { op, value } => {
            let ty = infer_expr_type(value, scopes, signatures)?;
            match op {
                Kind::Minus if is_numeric(&ty) => Ok(ty),
                Kind::Bang if ty == "bool" => Ok("bool".into()),
                Kind::Minus => Err(format!("unary '-' requires a numeric value, got {ty}")),
                Kind::Bang => Err(format!("logical '!' requires bool, got {ty}")),
                _ => unreachable!("parser only creates supported unary operators"),
            }
        }
        Expr::Binary { left, op, right } => {
            let left_ty = infer_expr_type(left, scopes, signatures)?;
            let right_ty = infer_expr_type(right, scopes, signatures)?;
            match op {
                BinaryOp::Add if left_ty == "String" && right_ty == "String" => Ok("String".into()),
                BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder
                    if is_numeric(&left_ty) && is_numeric(&right_ty) => {
                        if left_ty == "f64" || right_ty == "f64" { Ok("f64".into()) }
                        else if left_ty == "f32" || right_ty == "f32" { Ok("f32".into()) }
                        else if left_ty == "i64" || right_ty == "i64" { Ok("i64".into()) }
                        else if left_ty == "u64" || right_ty == "u64" { Ok("u64".into()) }
                        else if left_ty == "i32" || right_ty == "i32" { Ok("i32".into()) }
                        else { Ok("u32".into()) }
                    }
                BinaryOp::Add => Err(format!("operator '+' does not support {left_ty} + {right_ty}")),
                BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Remainder =>
                    Err(format!("arithmetic operator requires numeric operands, got {left_ty} and {right_ty}")),
                BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual
                    if is_numeric(&left_ty) && is_numeric(&right_ty) => Ok("bool".into()),
                BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual =>
                    Err(format!("comparison requires numeric operands, got {left_ty} and {right_ty}")),
                BinaryOp::Equal | BinaryOp::NotEqual if left_ty == right_ty || (is_numeric(&left_ty) && is_numeric(&right_ty)) => Ok("bool".into()),
                BinaryOp::Equal | BinaryOp::NotEqual => Err(format!("equality operands have incompatible types {left_ty} and {right_ty}")),
                BinaryOp::And | BinaryOp::Or if left_ty == "bool" && right_ty == "bool" => Ok("bool".into()),
                BinaryOp::And | BinaryOp::Or => Err(format!("logical operators require bool operands, got {left_ty} and {right_ty}")),
            }
        }
        Expr::Call { name, args } => {
            let mut arg_types = Vec::with_capacity(args.len());
            for arg in args { arg_types.push(infer_expr_type(arg, scopes, signatures)?); }
            let bad = |expected: String| Err(format!("function '{name}' {expected}"));
            match name.as_str() {
                "input" => {
                    if arg_types.first().is_some_and(|ty| ty != "String") { return bad("expects an optional String prompt".to_string()); }
                    Ok("String".into())
                }
                "len" => { if arg_types[0] != "String" && !arg_types[0].starts_with("Array<") { return bad(format!("expects String or Array, got {}", arg_types[0])); } Ok("i64".into()) }
                "contains" | "starts_with" | "ends_with" => {
                    if arg_types.iter().any(|ty| ty != "String") { return bad("expects two String arguments".to_string()); }
                    Ok("bool".into())
                }
                "parse_int" => { if arg_types[0] != "String" { return bad(format!("expects String, got {}", arg_types[0])); } Ok("i64".into()) }
                "to_string" => Ok("String".into()),
                "is_even" | "is_odd" => {
                    if !is_integer(&arg_types[0]) { return bad(format!("expects an integer, got {}", arg_types[0])); }
                    Ok("bool".into())
                }
                "abs" | "sign" => {
                    if !is_numeric(&arg_types[0]) { return bad(format!("expects a numeric argument, got {}", arg_types[0])); }
                    if name == "sign" { Ok("i64".into()) } else { Ok(arg_types[0].clone()) }
                }
                "sqrt" | "floor" | "ceil" | "round" => {
                    if !is_numeric(&arg_types[0]) { return bad(format!("expects a numeric argument, got {}", arg_types[0])); }
                    Ok("f64".into())
                }
                "pow" => {
                    if arg_types.iter().any(|ty| !is_numeric(ty)) { return bad("expects two numeric arguments".to_string()); }
                    Ok("f64".into())
                }
                "min" | "max" | "clamp" => {
                    if arg_types.iter().any(|ty| !is_numeric(ty)) { return bad("expects numeric arguments".to_string()); }
                    if arg_types.iter().any(|ty| ty != &arg_types[0]) { return bad("requires arguments with matching numeric types".to_string()); }
                    Ok(arg_types[0].clone())
                }
                _ => {
                    let signature = signatures.get(name).ok_or_else(|| format!("unknown function '{name}'"))?;
                    for (index, (actual, expected)) in arg_types.iter().zip(&signature.params).enumerate() {
                        if actual != expected && !(matches!(args[index], Expr::Number(_)) && is_numeric(expected)) {
                            return Err(format!("argument {} to '{name}' expects {expected}, got {actual}", index + 1));
                        }
                    }
                    Ok(signature.return_type.clone())
                }
            }
        }
    }
}

fn check_statements(
    statements: &[Stmt],
    scopes: &mut TypeScopes,
    signatures: &std::collections::HashMap<String, FunctionSignature>,
    expected_return: &str,
    loop_depth: usize,
) -> Result<(), String> {
    for stmt in statements {
        match stmt {
            Stmt::Let { name, declared_type, mutable, value } => {
                if scopes.last().is_some_and(|scope| scope.contains_key(name)) {
                    return Err(format!("variable '{name}' is already declared in this scope"));
                }
                let inferred = infer_expr_type(value, scopes, signatures)?;
                let ty = declared_type.as_deref().unwrap_or(&inferred);
                if !compatible_type(&inferred, ty, value) {
                    return Err(format!("variable '{name}' is declared as {ty}, but its initializer has type {inferred}"));
                }
                scopes.last_mut().expect("scope exists").insert(name.clone(), (ty.to_string(), *mutable));
            }
            Stmt::Assign { name, value } => {
                let (target_ty, mutable) = lookup_variable(scopes, name).ok_or_else(|| format!("cannot assign to unknown variable '{name}'"))?;
                if !mutable { return Err(format!("cannot assign to immutable variable '{name}'; declare it with 'var'")); }
                let value_ty = infer_expr_type(value, scopes, signatures)?;
                if !compatible_type(&value_ty, &target_ty, value) {
                    return Err(format!("cannot assign {value_ty} to variable '{name}' of type {target_ty}"));
                }
            }
            Stmt::IndexAssign { name, index, value } => {
                let (array_ty, mutable) = lookup_variable(scopes, name).ok_or_else(|| format!("cannot index-assign unknown variable '{name}'"))?;
                if !mutable { return Err(format!("cannot modify elements of immutable array '{name}'; declare it with 'var'")); }
                let element_ty = array_ty.strip_prefix("Array<").and_then(|ty| ty.strip_suffix(">"))
                    .ok_or_else(|| format!("indexed assignment requires an Array, got {array_ty}"))?;
                let index_ty = infer_expr_type(index, scopes, signatures)?;
                if !is_integer(&index_ty) { return Err(format!("array index must be an integer, got {index_ty}")); }
                let value_ty = infer_expr_type(value, scopes, signatures)?;
                if element_ty != value_ty { return Err(format!("array element type is {element_ty}, but assigned value has type {value_ty}")); }
            }
            Stmt::Push { name, value } => {
                let (array_ty, mutable) = lookup_variable(scopes, name).ok_or_else(|| format!("unknown array variable '{name}'"))?;
                if !mutable { return Err(format!("cannot push to immutable array '{name}'; declare it with var")); }
                let element_ty = array_ty.strip_prefix("Array<").and_then(|ty| ty.strip_suffix(">"))
                    .ok_or_else(|| format!("push requires an Array, got {array_ty}"))?;
                let value_ty = infer_expr_type(value, scopes, signatures)?;
                if element_ty != value_ty { return Err(format!("array element type is {element_ty}, but pushed value has type {value_ty}")); }
            }
            Stmt::Print(value) => { infer_expr_type(value, scopes, signatures)?; }
            Stmt::Return(value) => {
                let actual = infer_expr_type(value, scopes, signatures)?;
                if !compatible_type(&actual, expected_return, value) {
                    return Err(format!("return type mismatch: expected {expected_return}, got {actual}"));
                }
            }
            Stmt::Break if loop_depth == 0 => return Err("break used outside a loop".into()),
            Stmt::Continue if loop_depth == 0 => return Err("continue used outside a loop".into()),
            Stmt::Break | Stmt::Continue => {}
            Stmt::If { condition, then_branch, else_branch } => {
                let condition_ty = infer_expr_type(condition, scopes, signatures)?;
                if condition_ty != "bool" { return Err(format!("if condition must be bool, got {condition_ty}")); }
                scopes.push(std::collections::HashMap::new());
                let result = check_statements(then_branch, scopes, signatures, expected_return, loop_depth);
                scopes.pop();
                result?;
                if let Some(branch) = else_branch {
                    scopes.push(std::collections::HashMap::new());
                    let result = check_statements(branch, scopes, signatures, expected_return, loop_depth);
                    scopes.pop();
                    result?;
                }
            }
            Stmt::While { condition, body } => {
                let condition_ty = infer_expr_type(condition, scopes, signatures)?;
                if condition_ty != "bool" { return Err(format!("while condition must be bool, got {condition_ty}")); }
                scopes.push(std::collections::HashMap::new());
                let result = check_statements(body, scopes, signatures, expected_return, loop_depth + 1);
                scopes.pop();
                result?;
            }
            Stmt::ForRange { name, start, end, body } => {
                let start_ty = infer_expr_type(start, scopes, signatures)?;
                let end_ty = infer_expr_type(end, scopes, signatures)?;
                if !is_integer(&start_ty) || !is_integer(&end_ty) {
                    return Err(format!("range loop bounds must be integers, got {start_ty} and {end_ty}"));
                }
                scopes.push(std::collections::HashMap::new());
                scopes.last_mut().expect("scope exists").insert(name.clone(), ("i64".to_string(), false));
                let result = check_statements(body, scopes, signatures, expected_return, loop_depth + 1);
                scopes.pop();
                result?;
            }
            Stmt::ForEach { name, iterable, body } => {
                let iterable_ty = infer_expr_type(iterable, scopes, signatures)?;
                let element_ty = iterable_ty.strip_prefix("Array<").and_then(|ty| ty.strip_suffix(">"))
                    .ok_or_else(|| format!("for-each loop requires an Array, got {iterable_ty}"))?;
                scopes.push(std::collections::HashMap::new());
                scopes.last_mut().expect("scope exists").insert(name.clone(), (element_ty.to_string(), false));
                let result = check_statements(body, scopes, signatures, expected_return, loop_depth + 1);
                scopes.pop();
                result?;
            }
        }
    }
    Ok(())
}

fn type_check_program(functions: &[Function]) -> Result<(), String> {
    let signatures: std::collections::HashMap<String, FunctionSignature> = functions.iter().map(|function| {
        (function.name.clone(), FunctionSignature {
            params: function.params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: function.return_type.clone(),
        })
    }).collect();
    for function in functions {
        let mut scopes = vec![std::collections::HashMap::new()];
        for (name, ty) in &function.params {
            scopes[0].insert(name.clone(), (ty.clone(), false));
        }
        check_statements(&function.body, &mut scopes, &signatures, &function.return_type, 0)
            .map_err(|error| format!("in function '{}': {error}", function.name))?;
    }
    Ok(())
}

fn validate_expr_calls(expr: &Expr, signatures: &std::collections::HashMap<String, usize>) -> Result<(), String> {
    match expr {
        Expr::Call { name, args } => {
            let builtin = matches!(name.as_str(), "input" | "len" | "abs" | "sqrt" | "floor" | "ceil" | "round" | "to_string" | "parse_int" | "is_even" | "is_odd" | "sign" | "min" | "max" | "pow" | "contains" | "starts_with" | "ends_with" | "clamp");
            if !builtin {
                let expected = signatures.get(name).ok_or_else(|| format!("unknown function '{name}'"))?;
                if args.len() != *expected { return Err(format!("function '{name}' expects {expected} argument(s), got {}", args.len())); }
            }
            for arg in args { validate_expr_calls(arg, signatures)?; }
        }
        Expr::Unary { value, .. } => validate_expr_calls(value, signatures)?,
        Expr::Array(items) => { for item in items { validate_expr_calls(item, signatures)?; } }
        Expr::Index { value, index } => { validate_expr_calls(value, signatures)?; validate_expr_calls(index, signatures)?; }
        Expr::Binary { left, right, .. } => { validate_expr_calls(left, signatures)?; validate_expr_calls(right, signatures)?; }
        Expr::Text(_) | Expr::Number(_) | Expr::Bool(_) | Expr::Name(_) => {}
    }
    Ok(())
}

fn validate_function_calls(statements: &[Stmt], signatures: &std::collections::HashMap<String, usize>, loop_depth: usize) -> Result<(), String> {
    for stmt in statements {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Print(value) | Stmt::Return(value) => validate_expr_calls(value, signatures)?,
            Stmt::IndexAssign { index, value, .. } => { validate_expr_calls(index, signatures)?; validate_expr_calls(value, signatures)?; }
            Stmt::Push { value, .. } => validate_expr_calls(value, signatures)?,
            Stmt::Break if loop_depth == 0 => return Err("break used outside a loop".to_string()),
            Stmt::Continue if loop_depth == 0 => return Err("continue used outside a loop".to_string()),
            Stmt::Break | Stmt::Continue => {},
            Stmt::If { condition, then_branch, else_branch } => {
                validate_expr_calls(condition, signatures)?;
                validate_function_calls(then_branch, signatures, loop_depth)?;
                if let Some(branch) = else_branch { validate_function_calls(branch, signatures, loop_depth)?; }
            }
            Stmt::While { condition, body } => {
                validate_expr_calls(condition, signatures)?;
                validate_function_calls(body, signatures, loop_depth + 1)?;
            }
            Stmt::ForRange { start, end, body, .. } => {
                validate_expr_calls(start, signatures)?;
                validate_expr_calls(end, signatures)?;
                validate_function_calls(body, signatures, loop_depth + 1)?;
            }
            Stmt::ForEach { iterable, body, .. } => {
                validate_expr_calls(iterable, signatures)?;
                validate_function_calls(body, signatures, loop_depth + 1)?;
            }
        }
    }
    Ok(())
}

/// Parse the current Cinder subset and emit C++ for native compilation.
/// This bootstrap does not yet perform static type checking.
pub fn compile_to_cpp(source: &str) -> Result<String, String> {
    let tokens = lex(source)?;
    let mut parser = Parser { tokens, at: 0 };
    let functions = parser.program()?;
    type_check_program(&functions)?;
    let signatures: std::collections::HashMap<String, usize> = functions.iter()
        .map(|function| (function.name.clone(), function.params.len())).collect();
    for function in &functions {
        validate_function_calls(&function.body, &signatures, 0)?;
    }
    let mut cpp = String::from(
        "#include <algorithm>\n#include <cmath>\n#include <cstdint>\n#include <iostream>\n#include <stdexcept>\n#include <string>\n#include <type_traits>\n#include <vector>\n\n"
    );
    cpp.push_str(r#"
inline long long cinder_len(const std::string& value) {
    return static_cast<long long>(value.size());
}

template <typename T>
inline long long cinder_len(const std::vector<T>& value) {
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
    for function in &functions {
        if function.name == "main" { continue; }
        let return_type = cpp_type(&function.return_type);
        let params = function.params.iter()
            .map(|(name, ty)| format!("{} {}", cpp_type(ty), name))
            .collect::<Vec<_>>().join(", ");
        writeln!(cpp, "{return_type} {}({params});", function.name).unwrap();
    }
    cpp.push('\n');
    for function in &functions {
        if function.name == "main" { continue; }
        let return_type = cpp_type(&function.return_type);
        let params = function.params.iter()
            .map(|(name, ty)| format!("{} {}", cpp_type(ty), name))
            .collect::<Vec<_>>().join(", ");
        writeln!(cpp, "{return_type} {}({params}) {{", function.name).unwrap();
        emit_statements(&function.body, &mut cpp, 1);
        cpp.push_str("}\n\n");
    }
    cpp.push_str("int main() {\n");
    let main_function = functions.iter().find(|function| function.name == "main").expect("main checked");
    emit_statements(&main_function.body, &mut cpp, 1);
    cpp.push_str("    return 0;\n}\n");
    Ok(cpp)
}


#[cfg(test)]
mod tests {
    use super::compile_to_cpp;

    #[test]
    fn compiles_while_loops_and_variable_assignment() {
        let cpp = compile_to_cpp(r#"fn main() {
            var i: i64 = 1
            var total: i64 = 0
            while i <= 5 {
                total = total + i
                i = i + 1
            }
            print(total)
        }"#).unwrap();
        assert!(cpp.contains("while ((i <= 5))"));
        assert!(cpp.contains("total = (total + i);"));
        assert!(cpp.contains("i = (i + 1);"));
    }

    #[test]
    fn compiles_arrays_and_indexing() {
        let cpp = compile_to_cpp(r#"fn main() {
            let values = [10, 20, 30]
            print(values[1])
            print(len(values))
        }"#).unwrap();
        assert!(cpp.contains("std::vector{10, 20, 30}"));
        assert!(cpp.contains("values[1]"));
        assert!(cpp.contains("cinder_len(values)"));
    }

    #[test]
    fn assigns_array_elements() {
        let cpp = compile_to_cpp(r#"fn main() {
            var values = [1, 2]
            values[0] = 9
            print(values[0])
        }"#).unwrap();
        assert!(cpp.contains("values[0] = 9;"));
        assert!(cpp.contains("values[0]"));
    }

    #[test]
    fn rejects_index_assignment_to_immutable_or_wrongly_typed_arrays() {
        assert!(compile_to_cpp(r#"fn main() {
            let values = [1, 2]
            values[0] = 9
        }"#).unwrap_err().contains("immutable array"));
        assert!(compile_to_cpp(r#"fn main() {
            var values = [1, 2]
            values[0] = "x"
        }"#).unwrap_err().contains("assigned value has type"));
    }

    #[test]
    fn grows_mutable_arrays_with_push() {
        let cpp = compile_to_cpp(r#"fn main() {
            var values = [10, 20]
            push(values, 30)
            print(len(values))
        }"#).unwrap();
        assert!(cpp.contains("values.push_back(30);"));
        assert!(cpp.contains("cinder_len(values)"));
    }

    #[test]
    fn rejects_push_on_immutable_arrays_and_mismatched_values() {
        assert!(compile_to_cpp(r#"fn main() {
            let values = [1, 2]
            push(values, 3)
        }"#).unwrap_err().contains("immutable array"));
        assert!(compile_to_cpp(r#"fn main() {
            var values = [1, 2]
            push(values, "x")
        }"#).unwrap_err().contains("pushed value has type"));
    }

    #[test]
    fn rejects_invalid_array_element_types_and_indices() {
        assert!(compile_to_cpp(r#"fn main() {
            let values = [1, "x"]
        }"#).unwrap_err().contains("array elements must have matching types"));
        assert!(compile_to_cpp(r#"fn main() {
            let values = [1, 2]
            print(values[true])
        }"#).unwrap_err().contains("array index must be an integer"));
    }

    #[test]
    fn iterates_over_arrays_with_for_each() {
        let cpp = compile_to_cpp(r#"fn main() {
            let values = [2, 4, 6]
            var total: i64 = 0
            for value in values {
                total = total + value
            }
            print(total)
        }"#).unwrap();
        assert!(cpp.contains("for (const auto& value : values)"));
    }

    #[test]
    fn rejects_for_each_over_non_arrays() {
        assert!(compile_to_cpp("fn main() { for value in 123 { print(value) } }").unwrap_err().contains("for-each loop requires an Array"));
    }

    #[test]
    fn compiles_exclusive_range_for_loops() {
        let cpp = compile_to_cpp(r#"fn main() {
            var total: i64 = 0
            for i in 0..5 {
                total = total + i
            }
            print(total)
        }"#).unwrap();
        assert!(cpp.contains("for (std::int64_t i = 0; i < 5; ++i)"));
        assert!(cpp.contains("total = (total + i);"));
    }

    #[test]
    fn rejects_invalid_range_bounds_and_immutable_loop_variable_assignment() {
        assert!(compile_to_cpp(r#"fn main() { for i in "x"..5 { print(i) } }"#).unwrap_err().contains("range loop bounds must be integers"));
        assert!(compile_to_cpp("fn main() { for i in 0..5 { i = 2 } }").unwrap_err().contains("immutable variable 'i'"));
    }

    #[test]
    fn compiles_break_continue_and_accepts_str_alias() {
        let cpp = compile_to_cpp(r#"fn main() {
            let message: str = "Cinder"
            var i: i64 = 0
            while i < 10 {
                i = i + 1
                if i == 3 { continue }
                if i == 6 { break }
                print(message)
            }
        }"#).unwrap();
        assert!(cpp.contains("const std::string message = std::string(\"Cinder\");"));
        assert!(cpp.contains("continue;"));
        assert!(cpp.contains("break;"));
    }

    #[test]
    fn rejects_break_and_continue_outside_loops() {
        assert!(compile_to_cpp("fn main() { break }").unwrap_err().contains("break used outside a loop"));
        assert!(compile_to_cpp("fn main() { continue }").unwrap_err().contains("continue used outside a loop"));
    }

    #[test]
    fn emits_explicit_types_and_enforces_let_var_rules() {
        let cpp = compile_to_cpp(r#"fn main() {
            let count: i32 = 3
            var total: i64 = 0
            total = total + count
            print(total)
        }"#).unwrap();
        assert!(cpp.contains("const std::int32_t count = 3;"));
        assert!(cpp.contains("std::int64_t total = 0;"));
        assert!(compile_to_cpp("fn main() { let x: i64 = 1\n x = 2 }").unwrap_err().contains("immutable variable"));
        assert!(compile_to_cpp("fn main() { var x: bool = 1 }").unwrap_err().contains("initializer has type"));
    }

    #[test]
    fn rejects_unknown_variables_and_invalid_conditions() {
        assert!(compile_to_cpp("fn main() { print(missing) }").unwrap_err().contains("unknown variable"));
        assert!(compile_to_cpp("fn main() { if 1 { print(2) } }").unwrap_err().contains("if condition must be bool"));
    }

    #[test]
    fn passes_arrays_to_typed_functions() {
        let cpp = compile_to_cpp(r#"fn main() {
            let values: Array<i64> = [3, 7, 9]
            print(first(values))
        }
        fn first(values: Array<i64>) -> i64 {
            return values[0]
        }"#).unwrap();
        assert!(cpp.contains("std::vector<std::int64_t> values"));
        assert!(cpp.contains("std::int64_t first(std::vector<std::int64_t> values);"));
        assert!(cpp.contains("first(values)"));
    }

    #[test]
    fn compiles_typed_user_functions_and_forward_calls() {
        let cpp = compile_to_cpp(r#"fn main() {
            print(add(20, 22))
        }
        fn add(a: i64, b: i64) -> i64 {
            return a + b
        }"#).unwrap();
        assert!(cpp.contains("std::int64_t add(std::int64_t a, std::int64_t b);"));
        assert!(cpp.contains("return (a + b);"));
        assert!(cpp.contains("std::cout << add(20, 22)"));
    }

    #[test]
    fn rejects_unknown_user_functions_and_wrong_arity() {
        assert!(compile_to_cpp("fn main() { print(missing(1)) }").unwrap_err().contains("unknown function"));
        assert!(compile_to_cpp(r#"fn main() { print(add(1)) }
            fn add(a: i64, b: i64) -> i64 { return a + b }"#).unwrap_err().contains("expects 2 argument"));
    }

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
