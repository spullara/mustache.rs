use std::io::Write;
use std::any::Any;
use std::io::Read;
use std::io::Error;
use std::fs::File;

static DEFAULT_SM: &'static str = "{{";
static DEFAULT_EM: &'static str = "}}";

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

pub fn compile(file: &str) -> Result<Box<IsCode>, Error> {
    let openFile = try!(File::open(file));
    compileRead(&openFile, file)
}

pub fn compileRead(reader: &Read, file: &str) -> Result<Box<IsCode>, Error> {
    compileInternal(reader, "", 0, file, DEFAULT_SM, DEFAULT_EM, true)
}

fn compileInternal(reader: &Read, tag: &str, currentLine: u32, file: &str, sm: &str, em: &str, startOfLine: bool) -> Result<Box<IsCode>, Error> {
    Ok(Box::new(Mustache { codes: vec![] }))

}