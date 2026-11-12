use std::fmt::{self, Debug, Display, Formatter};

use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{ResponseError, http::StatusCode};
use sea_orm::DbErr;

pub trait GenericError = Display + Debug + ResponseError;

#[derive(Debug)]
pub enum Error<C: GenericError> {
    Db(DbErr), // this might be stupid
    Internal(&'static str),
    Unauthorized,
    Custom(C),
}

impl<C: GenericError> std::error::Error for Error<C> {}

impl<C: GenericError> From<DbErr> for Error<C> {
    fn from(err: DbErr) -> Self {
        Self::Db(err)
    }
}

impl<C: GenericError> From<argon2::password_hash::Error> for Error<C> {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::Internal("internal server error")
    }
}

impl<C: GenericError> From<argon2::password_hash::phc::Error> for Error<C> {
    fn from(_: argon2::password_hash::phc::Error) -> Self {
        Self::Internal("internal server error")
    }
}

impl<C: GenericError> From<SessionInsertError> for Error<C> {
    fn from(_: SessionInsertError) -> Self {
        Self::Internal("session error")
    }
}

impl<C: GenericError> From<SessionGetError> for Error<C> {
    fn from(_: SessionGetError) -> Self {
        Self::Internal("session error")
    }
}

impl<C: GenericError> Display for Error<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // TODO proper errors for db
            Self::Db(_) => write!(f, "internal server error"),
            Self::Internal(s) => write!(f, "{s}"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Custom(c) => Display::fmt(&c, f),
        }
    }
}

impl<C: GenericError> ResponseError for Error<C> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Db(_) | Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Custom(c) => c.status_code(),
        }
    }
}
