use std::fmt;

use crate::source::Source;

pub mod ast;

pub mod token;

pub trait DisplayWithContext<'a> {
    type Context: 'a;

    fn fmt(&self, f: &mut fmt::Formatter, context: &Self::Context) -> fmt::Result;

    fn display<'inner, 'ctxt>(
        &'inner self,
        context: &'ctxt Self::Context,
    ) -> Displayable<'a, 'inner, 'ctxt, Self>
    where
        Self: Sized,
    {
        Displayable {
            inner: self,
            context,
        }
    }
}

pub struct Displayable<'a, 'inner, 'ctxt, T: DisplayWithContext<'a>> {
    inner: &'inner T,
    context: &'ctxt T::Context,
}

impl<'a, 'inner, 'ctxt, T: DisplayWithContext<'a>> fmt::Display
    for Displayable<'a, 'inner, 'ctxt, T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f, self.context)
    }
}
