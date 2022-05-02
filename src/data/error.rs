use std::fmt::{Display, Formatter, Error};

#[derive(Debug)]
pub struct ParseError {
    pub file: String,
    pub message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Error parsing {}: {}", self.file, self.message)
    }
}
