use std::marker::PhantomData;

pub struct Symbol<T>(usize, PhantomData<T>);

impl<T> Clone for Symbol<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Symbol<T> {}

pub struct SymbolTable<T> {
    entries: Vec<T>,
}

impl<T> SymbolTable<T> {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn add_entry(&mut self, entry: T) -> Symbol<T> {
        let idx = self.entries.len();
        self.entries.push(entry);
        Symbol(idx, PhantomData)
    }

    pub fn get(&mut self, symbol: Symbol<T>) -> &T {
        &self.entries[symbol.0]
    }

    pub fn get_mut(&mut self, symbol: Symbol<T>) -> &mut T {
        &mut self.entries[symbol.0]
    }
}
