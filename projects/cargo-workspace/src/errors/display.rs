use super::*;

impl Error for CargoError {}

impl Display for CargoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoError::MissingWorkspace => write!(f, "No workspace found"),
            CargoError::InvalidToml(msg) => write!(f, "Invalid TOML: {}", msg),
            CargoError::IoError(msg) => write!(f, "IO error: {}", msg),
            CargoError::PublishError(msg) => write!(f, "Publish error: {}", msg),
            CargoError::DependencyError(msg) => write!(f, "Dependency error: {}", msg),
            CargoError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
        }
    }
}
