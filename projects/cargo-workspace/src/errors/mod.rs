use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    io,
};

mod convert;
mod display;

/// The result type of this crate.
pub type Result<T> = std::result::Result<T, CargoError>;

/// A boxed error kind, wrapping an [ExampleErrorKind].
#[derive(Clone, Debug)]
pub enum CargoError {
    MissingWorkspace,
    InvalidToml(String),
    IoError(String),
    PublishError(String),
    DependencyError(String),
    CircularDependency(String),
}

impl From<io::Error> for CargoError {
    fn from(err: io::Error) -> Self {
        CargoError::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for CargoError {
    fn from(err: toml::de::Error) -> Self {
        CargoError::InvalidToml(err.to_string())
    }
}