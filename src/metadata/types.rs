#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BaseType {
    Bool,
    Error,
    MatchKind,
    String,
    Int,
    Bit,
    Varbit,
    SizedInt(Option<u32>),
    SizedVarbit(Option<u32>),
    SizedBit(Option<u32>),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    Base(BaseType),
    Name,
    Specialized,
    Header,
    Tuple,
}
