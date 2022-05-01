use internment::Intern;

#[derive(Clone, PartialEq, Eq)]
pub enum Type {
    Parameterised(TypeIdent, Vec<Type>),
    Simple(TypeIdent),
    Unknown,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TypeIdent(Intern<String>);
