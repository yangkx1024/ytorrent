use std::fmt::{Display, Formatter};

/// All possible token types for bencode
#[derive(PartialEq)]
pub(super) enum Token<'a> {
    List,
    Dict,
    String(&'a [u8]),
    Num(&'a str),
    End,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::List => write!(f, "List"),
            Token::Dict => write!(f, "Dict"),
            Token::String(bytes) => write!(f, "String({})", bytes.len()),
            Token::Num(str) => write!(f, "Num({:?})", str),
            Token::End => write!(f, "End"),
        }
    }
}
