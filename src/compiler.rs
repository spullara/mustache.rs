use std::io::Write;
use std::any::Any;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::error::Error;
use std::iter::Peekable;
use std::io::CharsError;
use std::cell::Cell;
use std::cell::RefCell;

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
    let mut buf = &mut BufReader::new(reader).chars();
    let iter = &mut buf.take_while(|c| {
        c.is_ok()
    }).map(|c| {
        c.ok().unwrap()
    });
    let chars = Lookahead::new(iter);

    compile_internal(&chars, None, &Cell::new(0), file, DEFAULT_SM, DEFAULT_EM, true)
}

struct Lookahead<'a> {
    iter: RefCell<&'a mut Iterator<Item = char>>,
    peeked: Cell<Option<char>>,
}

impl <'a> Lookahead<'a>  {
    fn new(iter: &mut Iterator<Item = char>) -> Lookahead {
        Lookahead { iter: RefCell::new(iter), peeked: Cell::new(None) }
    }

    fn peek(&self) -> Option<char> {
        match self.peeked.get() {
            None => {
                self.peeked.set(self.iter.borrow_mut().next());
                self.peeked.get()
            }
            Some(c) => {
                self.peeked.set(None);
                Some(c)
            }
        }
    }
    fn next(&self) -> Option<char> {
        match self.peeked.get() {
            None => self.iter.borrow_mut().next(),
            Some(p) => {
                self.peeked.set(None);
                Some(p)
            }
        }
    }
}

fn compile_internal(br: &Lookahead, tag: Option<&str>, currentLine: &Cell<u32>, file: &str, sm: &str, em: &str, startOfLine: bool) -> Result<Box<IsCode>, String> {
    let startLine = currentLine;

    let mut iterable = currentLine.get() != 0;
    let mut sawCR = false;
    let mut onlywhitespace = true;
    let mut trackingLine = match currentLine.get() {
        0 => 1,
        _ => currentLine.get()
    };
    let mut out = String::new();
    let mut trackingStartOfLine = startOfLine;

    loop {
        let c = match br.next() {
            Some(a) => a,
            None => break
        };
        // Next line handling
        if c == '\r' {
            sawCR = true;
            continue;
        }
        if c == '\n' {
            currentLine.set(currentLine.get() + 1);
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
                {
                    let peeked = br.peek();
                    matches = peeked.map(|c| {
                        c == sm.char_at(1)
                    }).unwrap_or(false);
                }
                if matches {
                    // Consume the character from the stream
                    br.next();
                }
            }
            if matches {
                // We are in a tag! No we grab the tag name.
                let mut tagName = String::new();
                loop {
                    let c = match br.next() {
                        Some(a) => a,
                        None => break
                    };
                    if c == em.char_at(0) {
                        let mut matches = em.len() == 1;
                        if !matches {
                            {
                                let peeked = br.peek();
                                matches = peeked.map(|c| {
                                    c == em.char_at(1)
                                }).unwrap_or(false);
                            }
                            if matches {
                                // Consume the character from the stream
                                br.next();
                            }
                        }
                        if matches {
                            break;
                        }
                    }
                    tagName.push(c);
                }
                if tagName.len() == 0 {
                    return Err("Empty mustache".to_string());
                }
                let ch = tagName.char_at(0);
                let variable = tagName[1..].trim();
                println!("Matching on: {}", ch);
                match ch {
                    '#' |
                    '^' |
                    '<' |
                    '$' => {
                        let oldStartOfLine = trackingStartOfLine;
                        trackingStartOfLine = trackingStartOfLine & onlywhitespace;
                        let line = currentLine.get();
                        let mustache = compile_internal(br, Some(variable), currentLine, file, sm, em, trackingStartOfLine);
                        let lines = currentLine.get() - line;
                        if !onlywhitespace || lines == 0 {
                            println!("WriteCode: {}", out);
                        }
                        out = String::new();
                    },
                    '/' => {

                    },
                    '>' => {

                    },
                    '{' => {

                    },
                    '&' => {

                    },
                    '%' => {

                    },
                    '!' => {

                    },
                    '=' => {

                    },
                    _ => {
                        println!("WriteCode: {}", out);
                        println!("ValueCode: {}", tagName);
                    }
                }
                continue;
            }
        }
        out.push(c);
    }
    // WriteCode
    // Check if tag is set
    match tag {
        None => {
            println!("{}", out);
            // EOFCode
            // return MustacheCode
            Ok(Box::new(Mustache { codes: vec![] }))
        }
        _ => return Err("Failed to close tag".to_string()),
    }
}