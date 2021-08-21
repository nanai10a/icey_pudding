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

#[macro_export]
macro_rules! extract_clap_sams {
    ($n:expr; in $a:expr) => {
        match $a.subcommand_matches($n) {
            Some(s) => s,
            None => bail!("cannot get arg_matches: {}", $n),
        }
    };
}

#[macro_export]
macro_rules! extract_clap_arg {
    ($n:expr; in $a:expr) => {
        match $a.value_of($n) {
            Some(s) => s,
            None => bail!("cannot get arg: {}", $n),
        }
    };
}
