use ansi_term::{Colour, Style};
use internment::Intern;

use crate::compiler::ast::Expr;
use crate::compiler::lexer::token::{Token, TokenKind};
use crate::source::{Source, Span};

pub type JlyResult<T> = Result<T, Error>;

pub enum Error {
    UnexpectedToken { expected: TokenKind, found: Token },
    ExpectedExpression(Token),
    ExpectedIdent(Token),
    InvalidAssignmentTarget(Expr),

    UnresolvedVariable(Intern<String>),
    TooManyLocals(Span),
}

impl Error {
    pub fn report(&self) -> ErrorReport {
        match self {
            Self::UnexpectedToken { expected, found } => ErrorReport::new("unexpected token")
                .with_labelled_source(
                    format!("expected {:?} but found {:?}", expected, found.kind),
                    found.span,
                ),

            Self::ExpectedExpression(found) => ErrorReport::new("unexpected token")
                .with_labelled_source(
                    format!("expected an expression but found {:?}", found.kind),
                    found.span,
                ),

            Self::ExpectedIdent(found) => ErrorReport::new("unexpected token")
                .with_labelled_source(
                    format!("expected an identifier but found {:?}", found.kind),
                    found.span,
                ),

            Self::InvalidAssignmentTarget(lhs) => ErrorReport::new("invalid assignment target")
                .with_labelled_source(
                    "expected a variable, found an expression".to_string(),
                    lhs.span,
                ),

            Self::UnresolvedVariable(ident) => ErrorReport::new("unresolved variable")
                .with_label(format!("unresolved variable `{}`", ident)),

            Self::TooManyLocals(span) => ErrorReport::new("too many local variables")
                .with_labelled_source(
                    "a maximum of 256 variables is allowed per function".to_string(),
                    *span,
                )
                .with_note("why do you even have that many variables?".to_string()),
        }
    }
}

pub struct Label {
    pub msg: String,
    pub span: Option<Span>,
    // TODO: Add Source IDs. (SourceSpan type?)
}

pub struct ErrorReport {
    pub title: &'static str,
    pub labels: Vec<Label>,

    pub notes: Vec<String>,
    pub hints: Vec<String>,
}

impl ErrorReport {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            labels: vec![],

            notes: vec![],
            hints: vec![],
        }
    }

    pub fn with_label(mut self, msg: String) -> Self {
        self.labels.push(Label { msg, span: None });
        self
    }

    pub fn with_labelled_source(mut self, msg: String, span: Span) -> Self {
        self.labels.push(Label {
            msg,
            span: Some(span),
        });
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_hint(mut self, hint: String) -> Self {
        self.hints.push(hint);
        self
    }

    fn print(self, source: &Source) {
        let err_style = Style::new().fg(Colour::Red).bold();

        let label_style = Style::new().fg(Colour::Blue).bold();
        let note_style = Style::new().fg(Colour::Cyan).bold();
        let hint_style = Style::new().fg(Colour::Green).bold();

        let line_num_style = Style::new().fg(Colour::White).dimmed();
        let underline_style = Style::new().fg(Colour::Red);

        eprintln!("{}: {}", err_style.paint("error"), self.title);

        for label in self.labels {
            if let Some(span) = label.span {
                let loc = source.line_col(span.start);
                let line_span = source.line_span(span.start);

                let header = format!("in `{}`, line {}, col {}:", source.name, loc.line, loc.col);

                let line_num = format!("{} | ", loc.line);
                let line = source.span_str(line_span).trim_end();

                let mut underline = span.clamp(line_span).relative_to(line_span.start);
                if underline.len() == 0 {
                    underline.end += 1;
                }

                eprintln!("   {}", label_style.paint(header));
                eprintln!("      {}{}", line_num_style.paint(&line_num), line);
                eprintln!(
                    "      {}{}",
                    " ".repeat(line_num.len() + underline.start),
                    underline_style.paint("^".repeat(underline.len()))
                );
            }

            eprintln!("   {}: {}", label_style.paint("msg"), label.msg);
        }

        for note in self.notes {
            eprintln!("   {}: {}", note_style.paint("note"), note);
        }

        for hint in self.hints {
            eprintln!("   {}: {}", hint_style.paint("hint"), hint);
        }

        eprintln!();
    }
}

#[derive(Default)]
pub struct ErrorReporter {
    reports: Vec<ErrorReport>,
}

impl ErrorReporter {
    pub fn report(&mut self, err: ErrorReport) {
        self.reports.push(err);
    }

    pub fn had_errors(&self) -> bool {
        !self.reports.is_empty()
    }

    #[allow(clippy::result_unit_err)]
    pub fn assert_ok(&self) -> Result<(), ()> {
        match self.reports.is_empty() {
            true => Ok(()),
            false => Err(()),
        }
    }

    pub fn print(self, source: &Source) {
        if self.had_errors() {
            eprintln!("ENCOUNTERED ERROR(S) WHILE COMPILING:\n");

            for report in self.reports {
                report.print(source);
            }
        }
    }
}
