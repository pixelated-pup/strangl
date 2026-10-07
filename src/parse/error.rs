use crate::errors::{Diagnostic, Report};

use super::lex::TokKind;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorKind {
    BadToken,
    Unexpected(TokKind, Option<&'static str>),
    Mismatched { expected: TokKind, found: TokKind },
}

impl Diagnostic for ErrorKind {
    fn report(self) -> Report {
        match self {
            Self::BadToken => Report::new("invalid token"),
            Self::Unexpected(found, msg) => {
                let report = Report::new("unexpected token").with_label(format!("found {found}"));
                match msg {
                    Some(msg) => report.with_note(msg),
                    None => report,
                }
            }
            Self::Mismatched { expected, found } => Report::new("unexpected token")
                .with_label(format!("expected {expected}, found {found}")),
        }
    }
}
