extern crate mustache;

fn main() {
    match mustache::compiler::compile_read(&mut "{{name}}".as_bytes(), "name") {
        Ok(_) => println!("worked"),
        Err(e) => println!("{}", e.to_string())
    }
    match mustache::compiler::compile_read(&mut "{{#test}}{{name}}{{/test}}".as_bytes(), "name") {
        Ok(_) => println!("worked"),
        Err(e) => println!("{}", e.to_string())
    }

}