use std::io::Write;
use std::any::Any;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::error::Error;
use std::iter::Peekable;

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

pub fn compile(file: &str) -> Result<Box<IsCode>, String> {
    compile_read(&mut try!(File::open(file).map_err(|e| e.to_string())), file)
}

pub fn compile_read(reader: &mut Read, file: &str) -> Result<Box<IsCode>, String> {
    compile_internal(&mut BufReader::new(reader), None, 0, file, DEFAULT_SM, DEFAULT_EM, true)
}

fn compile_internal(br: &mut BufRead, tag: Option<&str>, currentLine: u32, file: &str, sm: &str, em: &str, startOfLine: bool) -> Result<Box<IsCode>, String> {
    let startLine = currentLine;

    let mut iterable = currentLine != 0;
    let mut sawCR = false;
    let mut onlywhitespace = true;
    let mut trackingLine = match currentLine {
        0 => 1,
        _ => currentLine
    };
    let mut out = String::new();
    let mut trackingStartOfLine = startOfLine;

    let mut iter = br.chars().peekable();
    loop {
        let c = match iter.next() {
            Some(a) => match a {
                Ok(b) => b,
                Err(err) => return Err(err.to_string())
            },
            None => break
        };
        // Next line handling
        if c == '\r' {
            sawCR = true;
            continue;
        }
        if c == '\n' {
            trackingLine = trackingLine + 1;
            if !iterable || (iterable && !onlywhitespace) {
                if sawCR {
                    out.push('\r');
                }
                out.push('\n');
            }
            // WriteCode

            iterable = false;
            onlywhitespace = true;
            trackingStartOfLine = true;
            continue;
        }
        sawCR = false;

        // Mustache tag handling
        if c == sm.char_at(0) {
            let mut matches = sm.len() == 1;
            if !matches {
                let peeked = iter.peek();
                matches = peeked.and_then(|v| {
                    v.as_ref().ok()
                }).map(|c| {
                    *c == sm.char_at(1)
                }).unwrap_or(false);
            }
            if matches {

            }
        }
        out.push(c);
    }
    // WriteCode
    // Check if tag is set
    match tag {
        None => return Err("Failed to close tag".to_string()),
        _ => {
            print!("{}", out);
            // EOFCode
            // return MustacheCode
            Ok(Box::new(Mustache { codes: vec![] }))
        }
    }
}