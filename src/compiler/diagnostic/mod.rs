use ansi_term::{Colour, Style};

use std::path::PathBuf;

use crate::source::{Source, Span};

mod parser;

pub use self::parser::*;

#[derive(Debug, Clone)]
pub enum Error {
    ParseError(ParseError),
    FileNotFoundError(PathBuf),
}

impl Error {
    pub fn report(&self) -> Report {
        match self {
            Self::ParseError(err) => err.report(),
            Self::FileNotFoundError(path) => Report {
                title: "file not found",
                msg: format!("couldn't find file `{}`", path.display()),
                snippet: None,
            },
        }
    }
}

pub struct Report {
    pub title: &'static str,
    pub msg: String,
    pub snippet: Option<Span>,
}

#[derive(Default)]
pub struct Diagnostics {
    errors: Vec<Error>,
}

impl Diagnostics {
    pub fn report(&mut self, err: Error) {
        self.errors.push(err);
    }

    pub fn had_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[allow(clippy::result_unit_err)]
    pub fn assert_ok(&self) -> Result<(), ()> {
        match self.errors.is_empty() {
            true => Ok(()),
            false => Err(()),
        }
    }

    pub fn print(&self, source: &Source) {
        let err_style = Style::new().fg(Colour::Red).bold();
        let header_style = Style::new().fg(Colour::Blue);

        if self.had_errors() {
            eprintln!("ENCOUNTERED ERROR(S) WHILE COMPILING:\n");

            for error in &self.errors {
                let report = error.report();
                eprintln!("{}: {}", err_style.paint("error"), report.title);

                if let Some(span) = report.snippet {
                    let loc = source.line_col(span.start);
                    let line = source.span_str(source.line_span(span.start)).trim_end();

                    let snippet_header =
                        format!("   in {}, line {}, col {}:", source.name, loc.line, loc.col);
                    eprintln!("{}", header_style.paint(snippet_header));
                    eprintln!("      {} | {}", loc.line, line);
                    eprintln!();
                }

                let msg = format!("   msg: {}", report.msg);
                eprintln!("{}", header_style.paint(msg));
                eprintln!();
            }
        }
    }
}
