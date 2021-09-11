use anyhow::{anyhow, Error};

use crate::repositories::RepositoryError;

pub fn user_err_fmt(e: RepositoryError) -> Error {
    match e {
        RepositoryError::NotFound => anyhow!("cannot find user. not registered?"),
        e => anyhow!("repository error: {}", e),
    }
}

pub fn content_err_fmt(e: RepositoryError) -> Error {
    match e {
        RepositoryError::NotFound => anyhow!("cannot find content."),
        e => anyhow!("repository error: {}", e),
    }
}
