//! Main Driver.

use std::{env, fmt::Write as _, fs, range::Range};

use yansi::Paint as _;

use crate::{
    errors::{ErrorHandler, HandlerCallback, Report},
    parse::Parser,
};

mod ast;
mod errors;
mod eval;
mod parse;

fn main() {
    let Some(src_path) = env::args().nth(1) else {
        println!("source path expected as first argument");
        return;
    };
    let src = match fs::read_to_string(&src_path) {
        Ok(src) => src,
        Err(err) => {
            println!("error reading source file: {err}");
            return;
        }
    };

    let handler_inner: HandlerCallback = &|report, span| print_diagnostic(report, span, &src);
    let handler = ErrorHandler::new(handler_inner);

    let Ok(instrs) = Parser::new(&src, handler).parse() else {
        return;
    };
    eval::eval(&instrs);
}

fn print_diagnostic(report: Report, span: Range<u32>, src: &str) {
    let start = usize::try_from(span.start).expect("why are you on 16bit");
    let end = usize::try_from(span.end).expect("why are you on 16bit");
    let line_start = src[..=start]
        .rfind(['\n', '\r'])
        .map_or(0, |i| (i + 1).min(start));
    let line_end = src[end..]
        .find(['\n', '\r'])
        .map_or_else(|| src.len(), |pos| pos + end);

    let line = &src[line_start..line_end];

    let span_start = start - line_start;
    let span_end = end - line_start;

    let line_num = src[..=start].matches(['\n', '\r']).count();

    let mut buffer = String::new();
    let _ = writeln!(
        buffer,
        "{}: {} ({}:{})\n{} {line}\n  {:>span_end$}",
        "error".bright_red(),
        report.name.white().bold(),
        line_num + 1,
        span_start + 1,
        ">".white().bold(),
        str::repeat("^", span_end - span_start).bright_red()
    );
    if let Some(label) = report.label {
        let _ = writeln!(
            buffer,
            "  {:>1$}",
            label.bright_red(),
            span_start + label.len()
        );
    }
    for note in report.notes {
        let _ = writeln!(buffer, "note: {note}");
    }
    eprintln!("{buffer}");
}
