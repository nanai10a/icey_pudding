use super::{RepositoryError, Result as RepoResult};

pub fn find_mut<T, P>(v: &mut [T], preficate: P) -> RepoResult<&mut T>
where P: FnMut(&&mut T) -> bool {
    let mut res = v.iter_mut().filter(preficate).collect::<Vec<_>>();

    match res.len() {
        0 => Err(RepositoryError::NotFound),
        1 => Ok(res.remove(0)),
        i => Err(RepositoryError::NoUnique { matched: i as u32 }),
    }
}

pub fn find_ref<T, P>(v: &[T], preficate: P) -> RepoResult<&T>
where P: FnMut(&&T) -> bool {
    let mut res = v.iter().filter(preficate).collect::<Vec<_>>();

    match res.len() {
        0 => Err(RepositoryError::NotFound),
        1 => Ok(res.remove(0)),
        i => Err(RepositoryError::NoUnique { matched: i as u32 }),
    }
}
