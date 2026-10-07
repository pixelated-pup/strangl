mod error;
mod lex;

use std::range::Range;

use crate::{
    ast::{ArithOp, CmpOp, Expr, InfixOp, Instr, LogOp, Span, Value},
    errors::{ErrorHandler, HandledError, Result},
};

use error::ErrorKind;
use lex::{Tok, TokKind};

/// Manages the state needing for parsing.
///
/// Construct with [`Parser::new()`], then produce an [`Ast`] (or errors) with [`Parser::parse()`].
pub struct Parser<'src> {
    handler: ErrorHandler<'src>,
    src: &'src str,
    toks: Vec<Result<Tok, Range<u32>>>,
    pos: usize,
}

/// Main parsing functions.
impl<'src> Parser<'src> {
    /// Constructs a [`Parser`] for `src`, reporting errors through `handler`.
    pub fn new(src: &'src str, handler: ErrorHandler<'src>) -> Self {
        Self {
            handler,
            src,
            toks: lex::lex(src),
            pos: 0,
        }
    }

    /// Parses the source this [`Parser`] was constructed with into a list of [`instructions`](Instr).
    ///
    /// # Errors
    /// If parsing produces any errors, they will be sent to the [`ErrorHandler`] this [`Parser`] was constructed with,
    /// and an error will be returned.
    pub fn parse(mut self) -> Result<Vec<Instr>> {
        let mut instrs = Vec::new();

        while !self.peek().is_ok_and(|tok| tok.kind == TokKind::Eof) {
            match self.instr() {
                Ok(instr) => instrs.push(instr),
                Err(_) => self.skip_to(&[
                    TokKind::Eval,
                    TokKind::Delete,
                    TokKind::Print,
                    TokKind::Ident,
                ]),
            }
        }

        self.handler.checked(instrs)
    }

    fn instr(&mut self) -> Result<Instr> {
        let tok = self.next()?;
        match tok.kind {
            TokKind::Eval => {
                let val = self.value()?;
                let cond = self
                    .consume_at(TokKind::If)
                    .map(|_| self.expr())
                    .transpose()?;
                Ok(Instr::Eval { val, cond })
            }
            TokKind::Delete => {
                let tok = self.next()?;
                if tok.kind != TokKind::Ident {
                    return Err(self.err(
                        ErrorKind::Mismatched {
                            expected: TokKind::Ident,
                            found: tok.kind,
                        },
                        tok.span,
                    ));
                }
                let var = self.src_of(tok).to_string();
                Ok(Instr::Delete(var))
            }
            TokKind::Print => self.value().map(Instr::Print),
            TokKind::Ident => {
                let var = self.src_of(tok).to_string();
                self.consume(TokKind::Eq)?;
                let expr = self.expr()?;
                Ok(Instr::Assign { var, expr })
            }
            _ => Err(self.unexpected(tok, None)),
        }
    }

    fn expr(&mut self) -> Result<Expr> {
        let tok = self.peek()?;
        match tok.kind {
            TokKind::LBracket => self
                .span()
                .map(|(start, end)| Expr::Span(Span::new(start, end))),
            TokKind::Bang => {
                self.pos += 1;
                self.value().map(Expr::Not)
            }
            TokKind::Read => {
                self.pos += 1;
                Ok(Expr::Read)
            }
            TokKind::StringLit | TokKind::Ident => {
                let lhs = self.value()?;
                let op = match self.peek()?.kind {
                    TokKind::Plus => InfixOp::Arith(ArithOp::Add),
                    TokKind::Minus => InfixOp::Arith(ArithOp::Sub),
                    TokKind::Times => InfixOp::Arith(ArithOp::Mul),
                    TokKind::Divide => InfixOp::Arith(ArithOp::Div),
                    TokKind::And => InfixOp::Log(LogOp::And),
                    TokKind::Or => InfixOp::Log(LogOp::Or),
                    TokKind::Eqq => InfixOp::Cmp(CmpOp::Eqq),
                    TokKind::Neq => InfixOp::Cmp(CmpOp::Neq),
                    TokKind::Lt => InfixOp::Cmp(CmpOp::Lt),
                    TokKind::Gt => InfixOp::Cmp(CmpOp::Gt),
                    TokKind::Leq => InfixOp::Cmp(CmpOp::Leq),
                    TokKind::Geq => InfixOp::Cmp(CmpOp::Geq),
                    _ => return Ok(Expr::Value(lhs)),
                };
                self.pos += 1;
                let rhs = self.value()?;
                Ok(Expr::Infix { lhs, op, rhs })
            }
            _ => Err(self.unexpected(tok, None)),
        }
    }

    fn span(&mut self) -> Result<(isize, isize)> {
        self.consume(TokKind::LBracket)?;
        let start = self.int()?;
        self.consume(TokKind::Comma)?;
        let end = self.int()?;
        self.consume(TokKind::RBracket)?;
        Ok((start, end))
    }

    fn int(&mut self) -> Result<isize> {
        let tok = self.next()?;
        if tok.kind != TokKind::IntLit {
            return Err(self.err(
                ErrorKind::Mismatched {
                    expected: TokKind::IntLit,
                    found: tok.kind,
                },
                tok.span,
            ));
        }
        Ok(self
            .src_of(tok)
            .parse()
            .expect("ICE: lexer produced invalid int token"))
    }

    fn value(&mut self) -> Result<Value> {
        let tok = self.next()?;
        match tok.kind {
            TokKind::StringLit => {
                let src = self.src_of(tok);
                let lit = process_escapes(&src[1..src.len() - 1]);
                Ok(Value::Literal(lit))
            }
            TokKind::Times => Ok(Value::All),
            TokKind::Ident => Ok(Value::Var(self.src_of(tok).to_string())),
            _ => Err(self.unexpected(tok, None)),
        }
    }
}

fn process_escapes(src: &str) -> String {
    let mut chars = src.chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        let c = match c {
            '\\' => match chars.next().expect("ICE: lexer produced standalone `\\`") {
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                '0' => '\0',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                c => c,
            },
            c => c,
        };
        out.push(c);
    }
    out
}

/// Helper functions.
impl<'src> Parser<'src> {
    fn src_of(&self, tok: Tok) -> &'src str {
        let start = usize::try_from(tok.span.start).expect("why are you on 16bit");
        let end = usize::try_from(tok.span.end).expect("why are you on 16bit");
        &self.src[start..end]
    }

    fn get_tok(&self, pos: usize) -> Result<Tok> {
        self.toks
            .get(pos)
            .copied()
            .unwrap_or_else(|| {
                let src_len = u32::try_from(self.src.len()).expect("file too long") - 1;
                Ok(TokKind::Eof.span(src_len..src_len))
            })
            .map_err(|span| self.err(ErrorKind::BadToken, span))
    }

    /// Consumes the next token.
    fn next(&mut self) -> Result<Tok> {
        let tok = self.get_tok(self.pos)?;
        self.pos += 1;
        Ok(tok)
    }

    /// Peeks the current token.
    fn peek(&self) -> Result<Tok> {
        self.get_tok(self.pos)
    }

    /// Checks if the next token is of the given kind.
    fn at(&self, tok: TokKind) -> bool {
        self.peek().is_ok_and(|t| t.kind == tok)
    }

    /// Consumes the next token and checks that it was the expected kind.
    fn consume(&mut self, expected: TokKind) -> Result<Tok> {
        self.next().and_then(|next| {
            if next.kind == expected {
                Ok(next)
            } else {
                Err(self.err(
                    ErrorKind::Mismatched {
                        expected,
                        found: next.kind,
                    },
                    next.span,
                ))
            }
        })
    }

    fn consume_at(&mut self, token: TokKind) -> Option<Tok> {
        self.at(token).then(|| {
            self.next()
                .expect("known to be at a valid token because `at` returned true")
        })
    }

    /// Consumes tokens up until any of the provided kinds, or until end-of-file.
    fn skip_to(&mut self, kinds: &[TokKind]) {
        while !self
            .peek()
            .is_ok_and(|tok| tok.kind == TokKind::Eof || kinds.contains(&tok.kind))
        {
            self.pos += 1;
        }
    }

    fn err(&self, error: ErrorKind, span: Range<u32>) -> HandledError {
        self.handler.report(error, span)
    }

    fn unexpected(&self, tok: Tok, msg: Option<&'static str>) -> HandledError {
        self.err(ErrorKind::Unexpected(tok.kind, msg), tok.span)
    }
}
