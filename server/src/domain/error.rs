// Error type for the processing
use std::sync::PoisonError;

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use thiserror::Error;

/// Error type for the processing based on thiserror crate
#[derive(Error, Debug)]
pub enum SecretServerError {
    #[error("Cannot get storage with share secret at this moment [{0}]")]
    InvalidStateError(String),
    #[error("Error in consesus protocol [{0}]")]
    ConsensusError(#[from] riteraft::Error),
    #[error("Share secret not found")]
    NotFound,
}

impl<T> From<PoisonError<T>> for SecretServerError {
    fn from(value: PoisonError<T>) -> Self {
        SecretServerError::InvalidStateError(value.to_string())
    }
}

impl From<SecretServerError> for riteraft::Error {
    fn from(value: SecretServerError) -> Self {
        riteraft::Error::Other(Box::new(value))
    }
}

impl error::ResponseError for SecretServerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InvalidStateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ConsensusError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
