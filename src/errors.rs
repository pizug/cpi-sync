use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("JSON error: {0}")]
    JSON(#[from] serde_json::Error),

    #[error("Regex error: `{0}` ")]
    Regex(#[from] regex::Error),

    #[error("Filesystem error: `{0}` ")]
    Filesystem(String),

    #[error("Zip error: `{0}` ")]
    Zip(#[from] zip::result::ZipError),

    #[error("JSON validation error: {0}")]
    JSONValidation(String),
}

impl<'a> From<jsonschema::ValidationError<'a>> for Error {
    fn from(sub: jsonschema::ValidationError<'a>) -> Error {
        Error::JSONValidation(sub.to_string())
    }
}
