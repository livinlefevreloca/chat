use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ChatError {
    kind: ChatErrorKind
}

impl ChatError {
    pub fn new(kind: ChatErrorKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}", self.kind.get_message())
    }
} 

impl Error for ChatError {
    fn description(&self) -> &str {
        &self.kind.get_message()
    }
}

#[derive(Debug)]
pub enum ChatErrorKind {
    RemoveClientError(&'static str),
}

impl ChatErrorKind {
    fn get_message(&self) -> &'static str {
        match self {
            ChatErrorKind::RemoveClientError(msg) => msg,
            _ => "Unknown Error"
        }
    }
}

