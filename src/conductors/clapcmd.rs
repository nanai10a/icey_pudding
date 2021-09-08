use clap::{load_yaml, App};

pub fn create_clap_app() -> App<'static> {
    ::lazy_static::lazy_static! {
        static ref YAML: ::yaml_rust::Yaml = load_yaml!("clap.yml").clone();
    }

    App::from(&*YAML).version(env!("CARGO_PKG_VERSION"))
}

#[test]
fn load_clap_yaml() {
    create_clap_app()
        .try_get_matches_from(vec!["*ip", "--help"])
        .unwrap();
}
