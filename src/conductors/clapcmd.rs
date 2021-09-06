use clap::{load_yaml, App};

pub(crate) fn create_clap_app() -> App<'static, 'static> {
    ::lazy_static::lazy_static! {
        static ref YAML: ::yaml_rust::Yaml = load_yaml!("clap.yml").clone();
    }

    App::from_yaml(&*YAML).version(env!("CARGO_PKG_VERSION"))
}

#[test]
fn load_clap_yaml() {
    create_clap_app()
        .get_matches_from_safe(vec!["*ip", "--help"])
        .unwrap();
}
