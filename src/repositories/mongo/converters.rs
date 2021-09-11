use anyhow::anyhow;
use mongodb::error::Result as MongoResult;

use super::{RepositoryError, Result as RepoResult};

pub fn convert_repo_err<T, E>(result: Result<T, E>) -> RepoResult<T>
where E: Sync + Send + ::std::error::Error + 'static {
    result.map_err(|e| RepositoryError::Internal(anyhow!(e)))
}

pub fn try_unique_check<T>(result: MongoResult<T>) -> RepoResult<bool> {
    match match match result {
        Ok(_) => return Ok(true),
        Err(e) => (*e.kind.clone(), e),
    } {
        (
            ::mongodb::error::ErrorKind::Write(::mongodb::error::WriteFailure::WriteError(e)),
            src,
        ) => (e.code, src),
        (_, src) => return Err(RepositoryError::Internal(anyhow!(src))),
    } {
        (11000, _) => Ok(false),
        (_, src) => Err(RepositoryError::Internal(anyhow!(src))),
    }
}

pub fn convert_404_or<T>(option: Option<T>) -> RepoResult<T> {
    match option {
        Some(t) => Ok(t),
        None => Err(RepositoryError::NotFound),
    }
}

pub fn to_bool<N>(number: N) -> bool
where N: ::core::convert::TryInto<i8> + ::core::fmt::Debug + Clone {
    match match ::core::convert::TryInto::<i8>::try_into(number.clone()) {
        Ok(n) => n,
        Err(_) => unreachable!("expected 0 or 1, found: {:?}", number),
    } {
        0 => false,
        1 => true,
        n => unreachable!("expected 0 or 1, found: {}", n),
    }
}
