use std::fmt;

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

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(match self {
            Type::Name => "Name",
            Type::Base(_) => "Base",
            Type::Specialized => "Specialized ",
            Type::Header => "Header",
            Type::Tuple => "Tuple",
        })
    }
}
