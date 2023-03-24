#[derive(Debug)]
pub struct Type {
    kind: Kind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BaseType {
    Bool,
    Error,
    MatchKind,
    String,
    Int,
    Bit,
    Varbit,
    SizedInt,
    SizedVarbit,
    SizedBit,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    Base(BaseType),
    Name,
    Specialized,
    Header,
    Tuple,
}
