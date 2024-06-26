use serde::Serialize;

pub(crate) fn print_yaml(v: impl Serialize) {
    let yaml = serde_yaml::to_string(&v).unwrap();
    println!("---\n{}", yaml);
}
