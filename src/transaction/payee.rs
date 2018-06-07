#[derive(Debug, Eq, PartialEq)]
pub enum PayeeType {
    RawName(String),
    ResolvedName(String),
}