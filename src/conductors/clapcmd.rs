use anyhow::{bail, Result};
use clap::{load_yaml, App, ArgMatches};

pub fn create_clap_app_v2() -> App<'static, 'static> {
    ::lazy_static::lazy_static! {
        static ref YAML: ::yaml_rust::Yaml = load_yaml!("clap.yml").clone();
    }

    App::from_yaml(&*YAML).version(env!("CARGO_PKG_VERSION"))
}

#[test]
fn load_clap_yaml() {
    create_clap_app_v2()
        .get_matches_from_safe(vec!["*ip", "--help"])
        .unwrap();
}

#[inline]
pub fn extract_clap_sams<'a>(ams: &'a ArgMatches, name: &str) -> Result<&'a ArgMatches<'a>> {
    match ams.subcommand_matches(name) {
        Some(s) => Ok(s),
        None => bail!("cannot get arg_matches: {}", name),
    }
}

#[inline]
pub fn extract_clap_arg<'a>(ams: &'a ArgMatches, name: &str) -> Result<&'a str> {
    match ams.value_of(name) {
        Some(s) => Ok(s),
        None => bail!("cannot get arg: {}", name),
    }
}
