//! Cinder's first explicit, backend-independent intermediate representation.
//!
//! The frontend lowers its parsed syntax tree into this representation after
//! semantic checking. Backends can consume this IR without depending on parser
//! implementation details. This is an initial structural IR, not yet SSA.

use std::fmt::Write as _;

use super::{BinaryOp, Expr, Function, Kind, Pattern, Stmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrFunction {
    pub name: String,
    pub type_parameters: Vec<String>,
    pub parameters: Vec<IrParameter>,
    pub return_type: String,
    pub body: Vec<IrStatement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrParameter {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrStatement {
    Let { name: String, ty: Option<String>, mutable: bool, value: IrExpression },
    Match { value: IrExpression, arms: Vec<IrMatchArm> },
    Assign { name: String, value: IrExpression },
    IndexAssign { name: String, index: IrExpression, value: IrExpression },
    Push { name: String, value: IrExpression },
    Print(IrExpression),
    Return(IrExpression),
    Break,
    Continue,
    If { condition: IrExpression, then_body: Vec<IrStatement>, else_body: Option<Vec<IrStatement>> },
    While { condition: IrExpression, body: Vec<IrStatement> },
    ForRange { name: String, start: IrExpression, end: IrExpression, body: Vec<IrStatement> },
    ForEach { name: String, iterable: IrExpression, body: Vec<IrStatement> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrPattern { Number(String), String(String), Bool(bool), Wildcard }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrMatchArm { pub pattern: IrPattern, pub body: Vec<IrStatement> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrExpression {
    String(String),
    Number(String),
    Bool(bool),
    Variable(String),
    Call { name: String, arguments: Vec<IrExpression> },
    Array(Vec<IrExpression>),
    Index { value: Box<IrExpression>, index: Box<IrExpression> },
    Unary { operator: String, value: Box<IrExpression> },
    Binary { left: Box<IrExpression>, operator: String, right: Box<IrExpression> },
}

fn lower_expr(expr: &Expr) -> IrExpression {
    match expr {
        Expr::Text(value) => IrExpression::String(value.clone()),
        Expr::Number(value) => IrExpression::Number(value.clone()),
        Expr::Bool(value) => IrExpression::Bool(*value),
        Expr::Name(name) => IrExpression::Variable(name.clone()),
        Expr::Call { name, args } => IrExpression::Call {
            name: name.clone(),
            arguments: args.iter().map(lower_expr).collect(),
        },
        Expr::Array(items) => IrExpression::Array(items.iter().map(lower_expr).collect()),
        Expr::Index { value, index } => IrExpression::Index {
            value: Box::new(lower_expr(value)),
            index: Box::new(lower_expr(index)),
        },
        Expr::Unary { op, value } => IrExpression::Unary {
            operator: match op {
                Kind::Minus => "-",
                Kind::Bang => "!",
                Kind::Tilde => "~",
                other => return IrExpression::Unary {
                    operator: format!("{other:?}"),
                    value: Box::new(lower_expr(value)),
                },
            }.to_string(),
            value: Box::new(lower_expr(value)),
        },
        Expr::Binary { left, op, right } => IrExpression::Binary {
            left: Box::new(lower_expr(left)),
            operator: match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
                BinaryOp::Remainder => "%",
                BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Greater => ">",
                BinaryOp::GreaterEqual => ">=",
                BinaryOp::Equal => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::And => "&&",
                BinaryOp::Or => "||",
                BinaryOp::BitAnd => "&",
                BinaryOp::BitOr => "|",
                BinaryOp::BitXor => "^",
                BinaryOp::ShiftLeft => "<<",
                BinaryOp::ShiftRight => ">>",
            }.to_string(),
            right: Box::new(lower_expr(right)),
        },
    }
}

fn lower_statements(statements: &[Stmt]) -> Vec<IrStatement> {
    statements.iter().map(|stmt| match stmt {
        Stmt::Let { name, declared_type, mutable, value } => IrStatement::Let {
            name: name.clone(), ty: declared_type.clone(), mutable: *mutable, value: lower_expr(value),
        },
        Stmt::Assign { name, value } => IrStatement::Assign { name: name.clone(), value: lower_expr(value) },
        Stmt::IndexAssign { name, index, value } => IrStatement::IndexAssign {
            name: name.clone(), index: lower_expr(index), value: lower_expr(value),
        },
        Stmt::Push { name, value } => IrStatement::Push { name: name.clone(), value: lower_expr(value) },
        Stmt::Print(value) => IrStatement::Print(lower_expr(value)),
        Stmt::Return(value) => IrStatement::Return(lower_expr(value)),
        Stmt::Break => IrStatement::Break,
        Stmt::Continue => IrStatement::Continue,
        Stmt::If { condition, then_branch, else_branch } => IrStatement::If {
            condition: lower_expr(condition),
            then_body: lower_statements(then_branch),
            else_body: else_branch.as_ref().map(|body| lower_statements(body)),
        },
        Stmt::While { condition, body } => IrStatement::While {
            condition: lower_expr(condition), body: lower_statements(body),
        },
        Stmt::ForRange { name, start, end, body } => IrStatement::ForRange {
            name: name.clone(), start: lower_expr(start), end: lower_expr(end), body: lower_statements(body),
        },
        Stmt::ForEach { name, iterable, body } => IrStatement::ForEach {
            name: name.clone(), iterable: lower_expr(iterable), body: lower_statements(body),
        },
        Stmt::Match { value, arms } => IrStatement::Match {
            value: lower_expr(value),
            arms: arms.iter().map(|arm| IrMatchArm {
                pattern: match &arm.pattern {
                    Pattern::Number(value) => IrPattern::Number(value.clone()),
                    Pattern::Text(value) => IrPattern::String(value.clone()),
                    Pattern::Bool(value) => IrPattern::Bool(*value),
                    Pattern::Wildcard => IrPattern::Wildcard,
                },
                body: lower_statements(&arm.body),
            }).collect(),
        },
    }).collect()
}

pub(crate) fn lower_program(functions: &[Function]) -> IrProgram {
    IrProgram {
        functions: functions.iter().map(|function| IrFunction {
            name: function.name.clone(),
            type_parameters: function.type_params.clone(),
            parameters: function.params.iter().map(|(name, ty)| IrParameter {
                name: name.clone(), ty: ty.clone(),
            }).collect(),
            return_type: function.return_type.clone(),
            body: lower_statements(&function.body),
        }).collect(),
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
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn cpp_number(number: &str) -> String {
    if let Some(digits) = number.strip_prefix("0o").or_else(|| number.strip_prefix("0O")) {
        return format!("0{digits}");
    }
    if number.starts_with("0x") || number.starts_with("0X") || number.starts_with("0b") || number.starts_with("0B") {
        return number.to_string();
    }
    if number.chars().all(|ch| ch.is_ascii_digit()) && number.len() > 1 && number.starts_with('0') {
        let normalized = number.trim_start_matches('0');
        return if normalized.is_empty() { "0".to_string() } else { normalized.to_string() };
    }
    number.to_string()
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
        array if array.starts_with("Array<") && array.ends_with('>') =>
            format!("std::vector<{}>", cpp_type(&array[6..array.len() - 1])),
        _ => ty.to_string(),
    }
}

fn cpp_expr(expr: &IrExpression) -> String {
    match expr {
        IrExpression::String(s) => format!("std::string({})", cpp_string(s)),
        IrExpression::Array(items) => format!("std::vector{{{}}}", items.iter().map(cpp_expr).collect::<Vec<_>>().join(", ")),
        IrExpression::Index { value, index } => format!("{}.at({})", cpp_expr(value), cpp_expr(index)),
        IrExpression::Number(n) => cpp_number(n),
        IrExpression::Bool(b) => b.to_string(),
        IrExpression::Variable(n) => n.clone(),
        IrExpression::Call { name, arguments } => {
            let values: Vec<String> = arguments.iter().map(cpp_expr).collect();
            match name.as_str() {
                "len" => format!("static_cast<long long>(cinder_len({}))", values[0]),
                "is_empty" => format!("({}).empty()", values[0]),
                "capacity" => format!("static_cast<long long>(({}).capacity())", values[0]),
                "first" => format!("cinder_first({})", values[0]),
                "last" => format!("cinder_last({})", values[0]),
                "pop" => format!("cinder_pop({})", values[0]),
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
        IrExpression::Unary { operator, value } => format!("({operator}{})", cpp_expr(value)),
        IrExpression::Binary { left, operator, right } =>
            format!("({} {operator} {})", cpp_expr(left), cpp_expr(right)),
    }
}

fn emit_statements(statements: &[IrStatement], cpp: &mut String, indent: usize) {
    use std::fmt::Write as _;
    let pad = "    ".repeat(indent);
    for stmt in statements {
        match stmt {
            IrStatement::Let { name, ty, mutable, value } => {
                let qualifier = if *mutable { "" } else { "const " };
                let declared = ty.as_deref().map(cpp_type).unwrap_or_else(|| "auto".to_string());
                let initializer = match (ty.as_deref(), value) {
                    (Some(ty), IrExpression::Array(items)) if items.is_empty() =>
                        format!("{}{{}}", cpp_type(ty)),
                    _ => cpp_expr(value),
                };
                writeln!(cpp, "{pad}{qualifier}{declared} {name} = {initializer};").unwrap();
            }
            IrStatement::Assign { name, value } => { writeln!(cpp, "{pad}{name} = {};", cpp_expr(value)).unwrap(); }
            IrStatement::IndexAssign { name, index, value } => { writeln!(cpp, "{pad}{name}.at({}) = {};", cpp_expr(index), cpp_expr(value)).unwrap(); }
            IrStatement::Push { name, value } => { writeln!(cpp, "{pad}{name}.push_back({});", cpp_expr(value)).unwrap(); }
            IrStatement::Print(value) => { writeln!(cpp, "{pad}std::cout << {} << '\\n';", cpp_expr(value)).unwrap(); }
            IrStatement::Return(value) => { writeln!(cpp, "{pad}return {};", cpp_expr(value)).unwrap(); }
            IrStatement::Break => { writeln!(cpp, "{pad}break;").unwrap(); }
            IrStatement::Continue => { writeln!(cpp, "{pad}continue;").unwrap(); }
            IrStatement::If { condition, then_body, else_body } => {
                writeln!(cpp, "{pad}if ({}) {{", cpp_expr(condition)).unwrap();
                emit_statements(then_body, cpp, indent + 1);
                if let Some(other) = else_body {
                    writeln!(cpp, "{pad}}} else {{").unwrap();
                    emit_statements(other, cpp, indent + 1);
                }
                writeln!(cpp, "{pad}}}").unwrap();
            }
            IrStatement::While { condition, body } => {
                writeln!(cpp, "{pad}while ({}) {{", cpp_expr(condition)).unwrap();
                emit_statements(body, cpp, indent + 1);
                writeln!(cpp, "{pad}}}").unwrap();
            }
            IrStatement::ForRange { name, start, end, body } => {
                writeln!(cpp, "{pad}for (std::int64_t {name} = {}; {name} < {}; ++{name}) {{", cpp_expr(start), cpp_expr(end)).unwrap();
                emit_statements(body, cpp, indent + 1);
                writeln!(cpp, "{pad}}}").unwrap();
            }
            IrStatement::ForEach { name, iterable, body } => {
                writeln!(cpp, "{pad}for (const auto& {name} : {}) {{", cpp_expr(iterable)).unwrap();
                emit_statements(body, cpp, indent + 1);
                writeln!(cpp, "{pad}}}").unwrap();
            }
            IrStatement::Match { value, arms } => {
                writeln!(cpp, "{pad}{{").unwrap();
                let inner_pad = format!("{pad}    ");
                writeln!(cpp, "{inner_pad}const auto __cinder_match_value = {};", cpp_expr(value)).unwrap();
                let mut started = false;
                for arm in arms {
                    let pattern = match &arm.pattern {
                        IrPattern::Number(number) => cpp_number(number),
                        IrPattern::String(value) => cpp_string(value),
                        IrPattern::Bool(value) => value.to_string(),
                        IrPattern::Wildcard => String::new(),
                    };
                    if matches!(arm.pattern, IrPattern::Wildcard) {
                        if started { writeln!(cpp, "{inner_pad}}} else {{").unwrap(); }
                        emit_statements(&arm.body, cpp, if started { indent + 2 } else { indent + 1 });
                    } else if !started {
                        writeln!(cpp, "{inner_pad}if (__cinder_match_value == {pattern}) {{").unwrap();
                        emit_statements(&arm.body, cpp, indent + 2);
                        started = true;
                    } else {
                        writeln!(cpp, "{inner_pad}}} else if (__cinder_match_value == {pattern}) {{").unwrap();
                        emit_statements(&arm.body, cpp, indent + 2);
                    }
                }
                if started { writeln!(cpp, "{inner_pad}}}").unwrap(); }
                writeln!(cpp, "{pad}}}").unwrap();
            }
        }
    }
}

/// Emit native C++ exclusively from the backend-independent Cinder IR.
pub(crate) fn emit_cpp(program: &IrProgram) -> String {
    let mut cpp = String::from("#include <algorithm>\n#include <cmath>\n#include <cstdint>\n#include <iostream>\n#include <stdexcept>\n#include <string>\n#include <type_traits>\n#include <vector>\n\n");
    cpp.push_str(r#"
inline long long cinder_len(const std::string& value) {
    return static_cast<long long>(value.size());
}
template <typename T>
inline long long cinder_len(const std::vector<T>& value) {
    return static_cast<long long>(value.size());
}
template <typename T>
inline const T& cinder_first(const std::vector<T>& value) {
    if (value.empty()) throw std::out_of_range("first: array is empty");
    return value.front();
}
template <typename T>
inline const T& cinder_last(const std::vector<T>& value) {
    if (value.empty()) throw std::out_of_range("last: array is empty");
    return value.back();
}
template <typename T>
inline T cinder_pop(std::vector<T>& value) {
    if (value.empty()) throw std::out_of_range("pop: array is empty");
    T result = value.back();
    value.pop_back();
    return result;
}
inline std::string cinder_input(const std::string& prompt = {}) {
    if (!prompt.empty()) std::cout << prompt << std::flush;
    std::string value;
    if (!std::getline(std::cin, value)) return {};
    return value;
}
inline std::string cinder_to_string(const std::string& value) { return value; }
inline std::string cinder_to_string(const char* value) {
    return value == nullptr ? std::string{} : std::string(value);
}
inline std::string cinder_to_string(bool value) { return value ? "true" : "false"; }
template <typename T>
    requires std::is_arithmetic_v<T>
inline std::string cinder_to_string(T value) {
    if constexpr (std::is_same_v<T, char> || std::is_same_v<T, signed char> || std::is_same_v<T, unsigned char>)
        return std::to_string(static_cast<int>(value));
    else return std::to_string(value);
}
inline long long cinder_parse_int(const std::string& value) {
    std::size_t consumed = 0;
    const long long result = std::stoll(value, &consumed, 10);
    if (consumed != value.size()) throw std::invalid_argument("parse_int: trailing characters in integer");
    return result;
}
"#);
    for function in &program.functions {
        if function.name == "main" { continue; }
        let params = function.parameters.iter().map(|p| format!("{} {}", cpp_type(&p.ty), p.name)).collect::<Vec<_>>().join(", ");
        if !function.type_parameters.is_empty() {
            writeln!(cpp, "template <{}>", function.type_parameters.iter().map(|ty| format!("typename {ty}")).collect::<Vec<_>>().join(", ")).unwrap();
        }
        writeln!(cpp, "{} {}({params});", cpp_type(&function.return_type), function.name).unwrap();
    }
    cpp.push('\n');
    for function in &program.functions {
        if function.name == "main" { continue; }
        let params = function.parameters.iter().map(|p| format!("{} {}", cpp_type(&p.ty), p.name)).collect::<Vec<_>>().join(", ");
        if !function.type_parameters.is_empty() {
            writeln!(cpp, "template <{}>", function.type_parameters.iter().map(|ty| format!("typename {ty}")).collect::<Vec<_>>().join(", ")).unwrap();
        }
        writeln!(cpp, "{} {}({params}) {{", cpp_type(&function.return_type), function.name).unwrap();
        emit_statements(&function.body, &mut cpp, 1);
        cpp.push_str("}\n\n");
    }
    cpp.push_str("int main() {\n");
    let main_function = program.functions.iter().find(|f| f.name == "main").expect("frontend requires main");
    emit_statements(&main_function.body, &mut cpp, 1);
    cpp.push_str("    return 0;\n}\n");
    cpp
}
