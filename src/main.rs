extern crate mustache;
use std::collections::LinkedList;
use mustache::compiler::DefaultMustacheVisitor;
use mustache::compiler::Mustache;
use mustache::compiler;

fn main() {

    let mv = &mut DefaultMustacheVisitor{mustache: Mustache{codes: LinkedList::new()}};

    match mustache::compiler::compile_read(mv, &mut "{{name}}".as_bytes(), "name") {
        Ok(_) => println!("worked"),
        Err(e) => println!("{}", e.to_string())
    }
    match mustache::compiler::compile_read(mv, &mut "{{#test}}{{name}}{{/test}}".as_bytes(), "name") {
        Ok(_) => println!("worked"),
        Err(e) => println!("{}", e.to_string())
    }

}