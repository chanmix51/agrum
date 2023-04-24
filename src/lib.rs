type StdError = Box<dyn std::error::Error + Sync + Send>;
type StdResult<T> = Result<T, StdError>;

pub mod core;
