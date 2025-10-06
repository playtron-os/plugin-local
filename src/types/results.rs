pub type EmptyResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type ResultWithError<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type U8Result = Result<u8, Box<dyn std::error::Error + Send + Sync>>;
