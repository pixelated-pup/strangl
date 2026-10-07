use std::{borrow::Cow, collections::HashMap, io, range::RangeInclusive};

use crate::{
    ast::{ArithOp, CmpOp, Expr, InfixOp, Instr, LogOp, Span, Value},
    errors::ErrorHandler,
    parse::Parser,
};

pub fn eval(instrs: &[Instr]) {
    EvalEngine::default().eval_instrs(instrs);
}

#[derive(Default)]
struct EvalEngine {
    mem: String,
    env: HashMap<String, Span>,
}

impl EvalEngine {
    fn eval_instrs(&mut self, instrs: &[Instr]) {
        for instr in instrs {
            self.eval_instr(instr);
        }
    }

    fn eval_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::Assign { var, expr } => {
                let span = self.eval_expr(expr);
                self.env.insert(var.to_string(), span);
            }
            Instr::Eval { val, cond } => {
                if let Some(cond) = cond {
                    let span = self.eval_expr(cond);
                    if !self.bool_of(span) {
                        return;
                    }
                }
                let span = self.eval_value(val);
                let src = self.str_of(span);
                let Ok(instrs) = Parser::new(&src, ErrorHandler::new(&|_, _| {})).parse() else {
                    return;
                };
                self.eval_instrs(&instrs)
            }
            Instr::Delete(var) => {
                if let Some(span) = self.normalise_span(self.get_var(var)) {
                    if span.start > span.last {
                        self.mem
                            .replace_range(RangeInclusive::from(span.last..=span.start), "");
                    } else {
                        self.mem.replace_range(span, "");
                    }
                }
            }
            Instr::Print(val) => {
                let span = self.eval_value(val);
                let str = self.str_of(span);
                println!("{str}");
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Span {
        match expr {
            Expr::Value(val) => self.eval_value(val),
            Expr::Infix { lhs, op, rhs } => match op {
                InfixOp::Arith(op) => {
                    let lhs = self.eval_value(lhs);
                    let rhs = self.eval_value(rhs);
                    let lhs = self.num_of(lhs);
                    let rhs = self.num_of(rhs);
                    let result = match op {
                        ArithOp::Add => lhs + rhs,
                        ArithOp::Sub => lhs - rhs,
                        ArithOp::Mul => lhs * rhs,
                        ArithOp::Div => lhs / rhs,
                    };
                    self.alloc(&result.to_string())
                }
                InfixOp::Log(op) => {
                    let lhs_span = self.eval_value(lhs);
                    let lhs = self.bool_of(lhs_span);
                    match op {
                        LogOp::And => {
                            if !lhs {
                                lhs_span
                            } else {
                                self.eval_value(rhs)
                            }
                        }
                        LogOp::Or => {
                            if lhs {
                                lhs_span
                            } else {
                                self.eval_value(rhs)
                            }
                        }
                    }
                }
                InfixOp::Cmp(op) => {
                    fn cmp<T: PartialOrd>(op: CmpOp, lhs: T, rhs: T) -> bool {
                        match op {
                            CmpOp::Eqq => lhs == rhs,
                            CmpOp::Neq => lhs != rhs,
                            CmpOp::Gt => lhs > rhs,
                            CmpOp::Lt => lhs < rhs,
                            CmpOp::Geq => lhs >= rhs,
                            CmpOp::Leq => lhs <= rhs,
                        }
                    }
                    let lhs = self.eval_value(lhs);
                    let rhs = self.eval_value(rhs);
                    let lhs = self.str_of(lhs);
                    let rhs = self.str_of(rhs);

                    let result = match lhs.parse::<f64>().ok().zip(rhs.parse::<f64>().ok()) {
                        Some((lhs, rhs)) => cmp(*op, lhs, rhs),
                        None => cmp(*op, lhs, rhs),
                    };

                    self.alloc(&result.to_string())
                }
            },
            Expr::Not(val) => {
                let span = self.eval_value(val);
                if self.bool_of(span) {
                    self.alloc("false")
                } else {
                    self.alloc("true")
                }
            }
            Expr::Span(range) => *range,
            Expr::Read => {
                let mut buf = String::new();
                io::stdin().read_line(&mut buf).unwrap();
                self.alloc(buf.trim_end_matches(['\n', '\r']))
            }
        }
    }

    fn eval_value(&mut self, val: &Value) -> Span {
        match val {
            Value::Var(var) => self.get_var(var),
            Value::All => Span::new(0, self.mem_len() - 1),
            Value::Literal(lit) => self.alloc(lit),
        }
    }

    fn get_var(&self, var: &str) -> Span {
        self.env.get(var).copied().unwrap_or(Span::new(0, 0))
    }

    fn mem_len(&self) -> isize {
        isize::try_from(self.mem.len()).expect("string length can never be greater than isize::MAX")
    }

    fn alloc(&mut self, str: &str) -> Span {
        let start = self.mem_len();
        self.mem.push_str(str);
        let end = self.mem_len() - 1;
        Span::new(start, end)
    }

    /// Normalises the provided span, wrapping negative indices and ensuring it's in bounds.
    /// Returns None if the memory is empty, meaning there is no valid span.
    /// Any modification to the memory after this is called invalidates the normalisation.
    /// Out-of-bounds spans become random in-bounds spans with the same size, or the entire memory if it's shorter than the span's size.
    /// The span may still have the start greater than the end. This should be further normalised as appropriate for the following operation.
    fn normalise_span(&self, span: Span) -> Option<RangeInclusive<usize>> {
        if self.mem.len() == 0 {
            return None;
        }

        let mut start = span.start;
        let mut end = span.end;
        let size = start.abs_diff(end);

        if size >= self.mem.len() {
            let span = if start > end {
                self.mem.len() - 1..=0
            } else {
                0..=self.mem.len() - 1
            };
            return Some(RangeInclusive::from(span));
        }

        // Wrap negative bounds around to index from the end.
        while start < 0 {
            start += self.mem_len()
        }
        while end < 0 {
            end += self.mem_len()
        }

        // Randomly replace out-of-bounds spans while maintaining the size of the span.
        if start >= self.mem_len() || end >= self.mem_len() {
            start = rand::random_range(0..self.mem.len() - size) as isize;
            end = start + (size as isize);
        }

        let start: usize = start
            .try_into()
            .expect("should never be negative by this point");
        let end: usize = end
            .try_into()
            .expect("should never be negative by this point");
        Some(RangeInclusive::from(start..=end))
    }

    fn str_of(&self, span: Span) -> Cow<'_, str> {
        match self.normalise_span(span) {
            Some(span) => {
                if span.start > span.last {
                    let str = &self.mem[RangeInclusive::from(span.last..=span.start)];
                    Cow::Owned(str.chars().rev().collect())
                } else {
                    Cow::Borrowed(&self.mem[span])
                }
            }
            None => Cow::Borrowed(""),
        }
    }

    fn bool_of(&self, span: Span) -> bool {
        let str = self.str_of(span);
        !(str.is_empty()
            || str.to_lowercase() == "false"
            || str.parse().is_ok_and(|n: f64| n == 0.0))
    }

    fn num_of(&self, span: Span) -> f64 {
        self.str_of(span).parse().unwrap_or(0.0)
    }
}
