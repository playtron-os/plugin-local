pub type EmptyResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type ResultWithError<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
