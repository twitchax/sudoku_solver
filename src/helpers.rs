use std::{
    error::Error, 
    fmt::{
        Formatter, 
        Display
    }
};

pub type Void = Result<(), Box<dyn std::error::Error>>;
pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

pub trait IntoError {
    fn into_error(self) -> Void;
}

impl IntoError for &str {
    fn into_error(self) -> Void {
        Err(Box::new(GenericError::from(self)))
    }
}

#[derive(Debug)]
pub struct GenericError {
    message: String
}

impl From<&str> for GenericError {
    fn from(message: &str) -> Self {
        GenericError { message: message.to_owned() }
    }
}

impl From<String> for GenericError {
    fn from(message: String) -> Self {
        GenericError { message }
    }
}

impl Display for GenericError {
    fn fmt<'a>(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}

impl Error for GenericError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}