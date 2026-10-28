//! Cinder's first explicit, backend-independent intermediate representation.
//!
//! The frontend lowers its parsed syntax tree into this representation after
//! semantic checking. Backends can consume this IR without depending on parser
//! implementation details. This is an initial structural IR, not yet SSA.

use super::{BinaryOp, Expr, Function, Kind, Stmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrFunction {
    pub name: String,
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
    }).collect()
}

pub(crate) fn lower_program(functions: &[Function]) -> IrProgram {
    IrProgram {
        functions: functions.iter().map(|function| IrFunction {
            name: function.name.clone(),
            parameters: function.params.iter().map(|(name, ty)| IrParameter {
                name: name.clone(), ty: ty.clone(),
            }).collect(),
            return_type: function.return_type.clone(),
            body: lower_statements(&function.body),
        }).collect(),
    }
}
