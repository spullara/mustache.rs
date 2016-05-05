use std::io::Write;
use std::any::Any;

#[derive(Eq)]
struct TemplateContext {
    sm: String,
    em: String,
    file: String,
    line: i64,
    start_of_line: bool
}

impl PartialEq for TemplateContext {
    fn eq(&self, other: &TemplateContext) -> bool {
        self.sm == other.sm &&
        self.em == other.em &&
        self.file == other.file &&
        self.line == other.line &&
        self.start_of_line == other.start_of_line
    }
}

struct Code {
    name: String,
    identity: String,
    appended: String,
    mustache: Mustache,
}

struct Mustache {
    codes: Vec<Box<IsCode>>,
}

pub trait IsCode {
    fn execute<'a>(&self, writer: &'a Write, scopes: &'a Vec<Box<Any>>) -> &'a Write;
}

impl IsCode for Mustache {
    fn execute<'a>(&self, writer: &'a Write, scopes: &'a Vec<Box<Any>>) -> &'a Write {
        for code in self.codes.iter() {
            code.execute(writer, scopes);
        }
        writer
    }
}

pub fn parse(template: &String) -> Box<IsCode> {
    Box::new(Mustache { codes: vec![] })
}
