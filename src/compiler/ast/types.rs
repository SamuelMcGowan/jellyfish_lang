#[derive(Clone, PartialEq, Eq)]
pub enum InferredType {
    Builtin(BuiltinType),
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    String,
    Integer,
    Float,
    Bool,
    Unit,
}

pub fn unify_types(
    a: InferredType,
    b: InferredType,
) -> Result<InferredType, (InferredType, InferredType)> {
    match (a, b) {
        (a, InferredType::Unknown) => Ok(a),
        (InferredType::Unknown, b) => Ok(b),

        (InferredType::Builtin(a), InferredType::Builtin(b)) if a == b => {
            Ok(InferredType::Builtin(a))
        }

        (a, b) => Err((a, b)),
    }
}
