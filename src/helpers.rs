use std::{
    error::Error, 
    fmt::{
        Formatter, 
        Display
    }
};

pub type Void = Result<(), Box<dyn std::error::Error>>;
pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

pub trait IntoError<T> {
    fn into_error(self) -> Res<T>;
}

impl<T, S> IntoError<T> for S 
    where S: AsRef<str> + ToString
{
    fn into_error(self) -> Res<T> {
        Err(Box::new(GenericError::from(self)))
    }
}

#[derive(Debug)]
pub struct GenericError {
    message: String
}

impl<T> From<T> for GenericError 
    where T: AsRef<str> + ToString 
{
    fn from(message: T) -> Self {
        GenericError { message: message.to_string() }
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