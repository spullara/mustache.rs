extern crate mustache;

fn main() {
    mustache::compiler::compileRead(&"{{name}}".as_bytes(), "name");
}