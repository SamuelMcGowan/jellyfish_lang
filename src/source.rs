use std::ops::Range;
use std::str::Chars;

const EOF_CHAR: char = '\0';

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
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

    pub fn len(self) -> usize {
        self.end - self.start
    }

    /// Get the overlap of two spans.
    pub fn overlap(self, other: Span) -> Self {
        Span {
            start: usize::max(self.start, other.start),
            end: usize::min(self.end, other.end),
        }
        .normalise()
    }

    /// The `Span` relative to a position.
    pub fn relative_to(mut self, start: usize) -> Self {
        self.start -= start;
        self.end -= start;
        self
    }

    /// Ensure that the length of the `Span` is at least zero.
    pub fn normalise(mut self) -> Self {
        self.end = usize::max(self.start, self.end);
        self
    }

    /// Get a `Span` enclosing two other `Span`s.
    pub fn join(self, other: Span) -> Self {
        Span {
            start: usize::min(self.start, other.start),
            end: usize::max(self.end, other.end),
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
        let span = span.overlap(self.file_span());
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

        Span { start, end }.overlap(self.file_span())
    }

    pub fn line_col(&self, byte_pos: usize) -> LineCol {
        let line_index = self.line_index(byte_pos);
        let col_index = byte_pos - self.line_offsets[line_index];

        LineCol {
            line: line_index + 1,
            col: col_index + 1,
        }
    }

    pub fn file_span(&self) -> Span {
        Span {
            start: 0,
            end: self.source.len(),
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
