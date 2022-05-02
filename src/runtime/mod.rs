pub mod chunk;
pub mod value;
pub mod vm;

#[cfg(test)]
pub mod tests;

use crate::source::Source;

use self::chunk::Chunk;

pub struct CompiledProgram<'sess> {
    pub source: &'sess Source,
    pub chunk: Chunk,
}
