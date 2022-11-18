use std::{error::Error, fmt::Display};

mod core_types;

#[derive(Debug)]
pub struct ConversionError {
    nested_error: Option<Box<dyn Error>>,
    message: String,
}

impl ConversionError {
    pub fn raise(message: &str) -> Self {
        Self {
            message: message.to_string(),
            nested_error: None,
        }
    }

    pub fn nest(message: &str, nested_error: Box<dyn Error>) -> Self {
        Self {
            message: message.to_string(),
            nested_error: Some(nested_error),
        }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.nested_error {
            Some(e) => write!(f, "{}, error caught '{}'", self.message, *e),
            None => write!(f, "{}", self.message),
        }
    }
}

impl Error for ConversionError {}

pub trait FromSQL<R> {
    fn from_sql(value: &str) -> Result<Option<R>, ConversionError>;
}

pub trait ToSQL<R> {
    fn to_sql(value: Option<R>) -> Result<String, ConversionError>;
}
