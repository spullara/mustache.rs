use std::io::Write;
use std::any::Any;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;

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
    compile_read(&mut try!(File::open(file)), file)
}

pub fn compile_read(reader: &mut Read, file: &str) -> Result<Box<IsCode>, Error> {
    compile_internal(&mut BufReader::new(reader), "", 0, file, DEFAULT_SM, DEFAULT_EM, true)
}

fn compile_internal(br: &mut BufRead, tag: &str, currentLine: u32, file: &str, sm: &str, em: &str, startOfLine: bool) -> Result<Box<IsCode>, Error> {
    let startLine = currentLine;
    let iterable = currentLine != 0;

    let mut sawCR = false;
    let mut onlywhitespace = true;
    let mut trackingLine = match currentLine {
        0 => 1,
        _ => currentLine
    };
    let mut out = String::new();

    let mut iter = br.chars();
    loop {
        let c = match iter.next() {
            Some(a) => match a {
                Ok(b) => b,
                Err(err) => return Err(Error::new(ErrorKind::InvalidData, err))
            },
            None => break
        };
        out.push(c);
    }
    print!("{}", out);
    Ok(Box::new(Mustache { codes: vec![] }))
}