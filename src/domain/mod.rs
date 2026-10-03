pub mod error;
pub mod project_spec;

pub use error::DomainError;
pub use project_spec::{AxumSpec, DbOption, DjangoSpec, FastApiSpec, GoHexSpec, TransportOption};
