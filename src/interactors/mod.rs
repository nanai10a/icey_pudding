pub mod content;
pub mod user;

use anyhow::{anyhow, bail, Error, Result};

use crate::repositories::RepositoryError;
use crate::utils::{convert_range_display, ConvertRange};

fn user_err_fmt(e: RepositoryError) -> Error {
    match e {
        RepositoryError::NotFound => anyhow!("cannot find user. not registered?"),
        e => anyhow!("repository error: {}", e),
    }
}

fn content_err_fmt(e: RepositoryError) -> Error {
    match e {
        RepositoryError::NotFound => anyhow!("cannot find content."),
        e => anyhow!("repository error: {}", e),
    }
}

fn calc_paging(
    full: impl ConvertRange<usize> + Clone,
    items: usize,
    page: usize,
) -> Result<impl ConvertRange<usize>> {
    let lim = (items * (page - 1))..(items + items * (page - 1));

    if !full.contains(&lim.start) {
        bail!(
            "out of range ({} !< {})",
            convert_range_display(full),
            convert_range_display(lim)
        );
    }

    let r: (::core::ops::Bound<usize>, ::core::ops::Bound<usize>) = if !full.contains(&lim.end) {
        let (start_bo, _) = full.to_turple();
        match start_bo {
            ::core::ops::Bound::Included(n) | ::core::ops::Bound::Excluded(n) => (n..).to_turple(),
            ::core::ops::Bound::Unbounded => (..).to_turple(),
        }
    } else {
        lim.to_turple()
    };

    Ok(r)
}
