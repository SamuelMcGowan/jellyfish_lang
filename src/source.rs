use std::ops::Range;
use std::str::Chars;

const EOF_CHAR: char = '\0';

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn range(self) -> Range<usize> {
        Range {
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

pub struct Source {
    pub name: String,
    pub source: String,
    line_offsets: Vec<usize>,
}

impl Source {
    pub fn new(name: String, source: String) -> Self {
        let line_offsets = Self::calculate_line_offsets(&source);
        Self {
            name,
            source,
            line_offsets,
        }
    }

    pub fn cursor(&self) -> Cursor {
        Cursor::new(self)
    }

    pub fn span_str(&self, span: Span) -> &str {
        &self.source[span.range()]
    }

    pub fn line_index(&self, byte_pos: usize) -> usize {
        match self.line_offsets.binary_search(&byte_pos) {
            Ok(index) => index,
            Err(next_index) => next_index - 1,
        }
    }

    pub fn line_span(&self, byte_pos: usize) -> Span {
        let line_index = self.line_index(byte_pos);

        let start = self.line_offsets[line_index];
        let end = self
            .line_offsets
            .get(line_index + 1)
            .copied()
            .unwrap_or(self.source.len());

        Span { start, end }
    }

    pub fn line_col(&self, byte_pos: usize) -> LineCol {
        let line_index = self.line_index(byte_pos);
        let col_index = byte_pos - self.line_offsets[line_index];

        LineCol {
            line: line_index + 1,
            col: col_index + 1,
        }
    }

    fn calculate_line_offsets(s: &str) -> Vec<usize> {
        let mut offsets = vec![0];
        offsets.extend(s.match_indices('\n').map(|(pos, _)| pos + 1));
        offsets
    }
}

pub struct Cursor<'sess> {
    source: &'sess Source,
    chars: Chars<'sess>,
    start_length: usize,
}

impl<'sess> Cursor<'sess> {
    pub fn new(source: &'sess Source) -> Self {
        Self {
            source,
            chars: source.source.chars(),
            start_length: source.source.len(),
        }
    }

    pub fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    pub fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn eat(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek()) {
            self.advance();
        }
    }

    pub fn start_span(&mut self) {
        self.start_length = self.chars.as_str().len();
    }

    pub fn span(&self) -> Span {
        Span {
            start: self.source.source.len() - self.start_length,
            end: self.source.source.len() - self.chars.as_str().len(),
        }
    }

    pub fn lexeme(&self) -> &str {
        self.source.span_str(self.span())
    }
}
