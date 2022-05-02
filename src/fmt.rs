use std::fmt;

use crate::source::Source;

pub trait DisplayWithSource {
    fn fmt(&self, f: &mut fmt::Formatter, source: &Source) -> fmt::Result;

    fn display<'inner, 'sess>(
        &'inner self,
        source: &'sess Source,
    ) -> Displayable<'inner, 'sess, Self>
    where
        Self: Sized,
    {
        Displayable {
            inner: self,
            source,
        }
    }
}

pub struct Displayable<'inner, 'sess, T: DisplayWithSource> {
    inner: &'inner T,
    source: &'sess Source,
}

impl<'inner, 'sess, T: DisplayWithSource> fmt::Display for Displayable<'inner, 'sess, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f, self.source)
    }
}
