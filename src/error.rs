use rocket::{
    http::Status,
    response::{self, Responder},
    trace, Request,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error")]
    Database(#[from] sqlx::error::Error),
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        match self {
            Self::Database(error) => {
                trace::error!(%error, "");
                Err(Status::ServiceUnavailable)
            }
        }
    }
}
