use clap::{load_yaml, App};

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
