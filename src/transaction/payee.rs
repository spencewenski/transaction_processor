#[derive(Debug)]
pub enum PayeeType {
    RawName(String),
    ResolvedName(String),
}