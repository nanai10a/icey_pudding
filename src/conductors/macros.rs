use anyhow::{bail, Result};
use clap::ArgMatches;

#[macro_export]
macro_rules! extract_option {
    (opt $t:path => ref $v:ident in $d:ident) => {{
        let mut opt = $d
            .options
            .iter()
            .filter_map(|v| match v.name == stringify!($v) {
                false => None,
                true => match v.value {
                    Some($t(ref val)) => Some(Some(val)),
                    _ => Some(None),
                },
            })
            .collect::<Vec<_>>();

        match opt.len() {
            1 => Ok(opt.remove(0)),
            _ => Err(anyhow::anyhow!("cannot get value: `{}`", stringify!($v))),
        }
    }};
    ($t:path => ref $v:ident in $d:ident) => {{
        let mut opt = $d
            .options
            .iter()
            .filter_map(|v| match v.name == stringify!($v) {
                false => None,
                true => match v.value {
                    Some($t(ref val)) => Some(val),
                    _ => None,
                },
            })
            .collect::<Vec<_>>();

        match opt.len() {
            1 => Ok(opt.remove(0)),
            _ => Err(anyhow::anyhow!("cannot get value: `id`")),
        }
    }};
}

#[inline]
pub fn extract_clap_sams_fn<'a>(ams: &'a ArgMatches, name: &str) -> Result<&'a ArgMatches<'a>> {
    match ams.subcommand_matches(name) {
        Some(s) => Ok(s),
        None => bail!("cannot get arg_matches: {}", name),
    }
}

#[inline]
pub fn extract_clap_arg_fn<'a>(ams: &'a ArgMatches, name: &str) -> Result<&'a str> {
    match ams.value_of(name) {
        Some(s) => Ok(s),
        None => bail!("cannot get arg: {}", name),
    }
}
