use std::range::Range;

use derive_more::Display;
use logos::Logos;

/// Produces a vec of tokens extracted from the source.
pub fn lex(src: &str) -> Vec<Result<Tok, Range<u32>>> {
    TokKind::lexer(src)
        .spanned()
        .map(|(tok, span)| {
            let span = u32::try_from(span.start).expect("file too long")
                ..u32::try_from(span.end).expect("file too long");
            match tok {
                Ok(tok) => Ok(tok.span(span)),
                Err(()) => Err(Range::from(span)),
            }
        })
        .collect()
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Tok {
    pub kind: TokKind,
    pub span: Range<u32>,
}

#[derive(Logos, PartialEq, Eq, Debug, Display, Clone, Copy)]
#[logos(skip(r"//.*", allow_greedy = true))]
#[logos(skip(r"\p{Pattern_White_Space}+"))]
#[logos(subpattern escape = r#"(\\\\)|(\\")|(\\0)|(\\t)|(\\n)|(\\r)"#)]
pub enum TokKind {
    /* LITERALS */
    /// integer literal.
    #[regex("-?[0-9]+")]
    IntLit,
    /// string literal.
    #[regex(r#"("([^"\\]|(?&escape))*")"#)]
    StringLit,

    /* DELIMITERS */
    /// `[`.
    #[display("`[`")]
    #[token("[")]
    LBracket,
    /// `]`.
    #[display("`]`")]
    #[token("]")]
    RBracket,

    /* SYMBOLS */
    /// `=`.
    #[display("`=`")]
    #[token("=")]
    Eq,
    /// `,`.
    #[display("`,`")]
    #[token(",")]
    Comma,

    /* OPERATORS */
    /// `+`.
    #[display("`+`")]
    #[token("+")]
    Plus,
    /// `-`.
    #[display("`-`")]
    #[token("-")]
    Minus,
    /// `*`.
    #[display("`*`")]
    #[token("*")]
    Times,
    /// `/`.
    #[display("`/`")]
    #[token("/")]
    Divide,
    /// `&&`.
    #[display("`&&`")]
    #[token("&&")]
    And,
    /// `||`.
    #[display("`||`")]
    #[token("||")]
    Or,
    /// `!`.
    #[display("`!`")]
    #[token("!")]
    Bang,
    /// `==`.
    #[display("`==`")]
    #[token("==")]
    Eqq,
    /// `!=`.
    #[display("`!=`")]
    #[token("!=")]
    Neq,
    /// `<`.
    #[display("`<`")]
    #[token("<")]
    Lt,
    /// `>`.
    #[display("`>`")]
    #[token(">")]
    Gt,
    /// `<=`.
    #[display("`<=`")]
    #[token("<=")]
    Leq,
    /// `>=`.
    #[display("`>=`")]
    #[token(">=")]
    Geq,

    /* KEYWORDS */
    /// `eval`.
    #[display("`eval`")]
    #[token("eval")]
    Eval,
    /// `if`.
    #[display("`if`")]
    #[token("if")]
    If,
    /// `delete`.
    #[display("`delete`")]
    #[token("delete")]
    Delete,
    /// `print`.
    #[display("`print`")]
    #[token("print")]
    Print,
    /// `read`.
    #[display("`read`")]
    #[token("read")]
    Read,

    /* MISC */
    /// identifier.
    #[regex(r"\p{XID_Start}\p{XID_Continue}*")]
    Ident,
    /// end-of-file.
    Eof,
}

impl TokKind {
    pub fn span(self, span: impl Into<Range<u32>>) -> Tok {
        Tok {
            kind: self,
            span: span.into(),
        }
    }
}
