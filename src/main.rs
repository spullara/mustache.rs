extern crate mustache;

fn main() {
    mustache::compiler::parse(&"{{name}}".to_string());
}