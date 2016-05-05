extern crate mustache;

fn main() {
    mustache::compiler::compile_read(&mut "{{name}}".as_bytes(), "name");
}