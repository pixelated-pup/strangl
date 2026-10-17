pub enum Instr {
    Assign { var: String, expr: Expr },
    Eval { val: Value, cond: Option<Expr> },
    Delete(Value),
    Print(Value),
}

#[derive(Debug)]
pub enum Expr {
    Value(Value),
    Infix { lhs: Value, op: InfixOp, rhs: Value },
    Not(Value),
    Read,
}

#[derive(Clone, Copy, Debug)]
pub enum InfixOp {
    Arith(ArithOp),
    Log(LogOp),
    Cmp(CmpOp),
}

#[derive(Clone, Copy, Debug)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug)]
pub enum LogOp {
    And,
    Or,
}

#[derive(Clone, Copy, Debug)]
pub enum CmpOp {
    Eqq,
    Neq,
    Gt,
    Lt,
    Geq,
    Leq,
}

#[derive(Debug)]
pub enum Value {
    Var(String),
    All,
    Literal(String),
    Span(Span),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: isize,
    pub end: isize,
}

impl Span {
    pub const fn new(start: isize, end: isize) -> Self {
        Self { start, end }
    }
}
