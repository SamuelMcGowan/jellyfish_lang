pub mod chunk;
pub mod value;
pub mod vm;

#[cfg(test)]
pub mod tests;

use crate::source::Source;

use self::chunk::Chunk;

pub struct CompiledProgram {
    pub source: Source,
    pub chunk: Chunk,
}
