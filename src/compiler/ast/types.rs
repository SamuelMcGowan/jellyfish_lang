#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Builtin(BuiltinType),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    String,
    Integer,
    Float,
    Bool,
    Unit,
}

pub fn unify_types(a: Type, b: Type) -> Result<Type, (Type, Type)> {
    match (a, b) {
        (a, Type::Unknown) => Ok(a),
        (Type::Unknown, b) => Ok(b),

        (Type::Builtin(a), Type::Builtin(b)) if a == b => Ok(Type::Builtin(a)),

        (a, b) => Err((a, b)),
    }
}
