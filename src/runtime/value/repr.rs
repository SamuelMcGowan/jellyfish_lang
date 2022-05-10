use super::*;

impl Value {
    pub fn repr(&self) -> String {
        match self {
            Self::Object(obj) => obj.repr(),
            Self::String(s) => s.to_string(),
            Self::Integer(i) => format!("{}", i),
            Self::Float(f) => format!("{}f", f),
            Self::Bool(b) => format!("{}", b),
            Self::Unit => "()".to_string(),
        }
    }
}

impl Object {
    pub fn repr(&self) -> String {
        match self {
            Self::JellyFunction(func) => func.repr(),
        }
    }
}

impl JellyFunction {
    pub fn repr(&self) -> String {
        "{func}".to_string()
    }
}
