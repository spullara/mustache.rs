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
use std::collections::LinkedList;

static DEFAULT_SM: &'static str = "{{";
static DEFAULT_EM: &'static str = "}}";

#[derive(Eq)]
pub struct TemplateContext {
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

pub struct Code {
    name: String,
    identity: String,
    appended: String,
    mustache: Mustache,
}

pub struct Mustache {
    pub codes: LinkedList<Box<IsCode>>,
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

pub fn compile(mv: &mut MustacheVisitor, file: &str) -> Result<(), String> {
    compile_read(mv, &mut try!(File::open(file).map_err(|e| e.to_string())), file)
}

pub fn compile_read(mv: &mut MustacheVisitor, reader: &mut Read, file: &str) -> Result<(), String> {
    let mut buf = &mut BufReader::new(reader).chars();
    let iter = &mut buf.take_while(|c| {
        c.is_ok()
    }).map(|c| {
        c.ok().unwrap()
    });
    let chars = Lookahead::new(iter);

    compile_internal(mv, &chars, None, &Cell::new(0), file, DEFAULT_SM, DEFAULT_EM, true)
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

pub trait MustacheVisitor {
    fn mustache(&self, ctx: &TemplateContext) -> Mustache;
    fn iterable(&self, ctx: &TemplateContext, variable: String, mustache: &Mustache);
    fn non_iterable(&self, ctx: &TemplateContext, variable: String, mustache: &Mustache);
    fn partial(&self, ctx: &TemplateContext, variable: String);
    fn value(&self, ctx: &TemplateContext, variable: String, encoded: bool);
    fn write(&self, ctx: &TemplateContext, text: String);
    fn pragma(&self, ctx: &TemplateContext, pragma: String, args: String);
    fn eof(&self, ctx: &TemplateContext);
    fn extend(&self, ctx: &TemplateContext, variable: String, mustache: Mustache);
    fn name(&self, ctx: &TemplateContext, variable: String, mustache: Mustache);
    fn comment(&self, ctx: &TemplateContext, comment: String);
}

pub struct DefaultMustacheVisitor {
    pub mustache: Mustache,
}

impl MustacheVisitor for DefaultMustacheVisitor {
    fn mustache(&self, ctx: &TemplateContext) -> Mustache {
        Mustache{codes: LinkedList::new()}
    }
    fn iterable(&self, ctx: &TemplateContext, variable: String, mustache: &Mustache){}
    fn non_iterable(&self, ctx: &TemplateContext, variable: String, mustache: &Mustache){}
    fn partial(&self, ctx: &TemplateContext, variable: String){}
    fn value(&self, ctx: &TemplateContext, variable: String, encoded: bool){}
    fn write(&self, ctx: &TemplateContext, text: String){}
    fn pragma(&self, ctx: &TemplateContext, pragma: String, args: String){}
    fn eof(&self, ctx: &TemplateContext){}
    fn extend(&self, ctx: &TemplateContext, variable: String, mustache: Mustache){}
    fn name(&self, ctx: &TemplateContext, variable: String, mustache: Mustache){}
    fn comment(&self, ctx: &TemplateContext, comment: String){}
}

fn compile_internal(mv: &mut MustacheVisitor, br: &Lookahead, tag: Option<&str>, currentLine: &Cell<u32>, file: &str, sm_start: &str, em_start: &str, startOfLine: bool) -> Result<(), String> {
    let mut sm = sm_start;
    let mut em = em_start;
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
                        let mustache = compile_internal(mv, br, Some(variable), currentLine, file, sm, em, trackingStartOfLine);
                        let lines = currentLine.get() - line;
                        if !onlywhitespace || lines == 0 {
                            println!("WriteCode: {}", out);
                        }
                        out = String::new();
                        match ch {
                            '#' => println!("Starting block: {}", variable),
                            '^' => println!("Starting not block: {}", variable),
                            '<' => println!("Starting inherit: {}", variable),
                            '$' => println!("Starting replacement: {}", variable),
                            _ => panic!("Asserting this cant happen"),
                        }
                        iterable = lines != 0;
                    },
                    '/' => {
                        if !trackingStartOfLine || !onlywhitespace {
                            println!("WriteCode: {}", out);
                        }
                        match tag {
                            None => return Err("Missing start tag".to_string()),
                            Some(t) => {
                                if t != variable {
                                    return Err("Mismatched tags".to_string())
                                }
                            }
                        }
                        println!("End tag: {}", variable)
                    },
                    '>' => {
                        println!("WriteCode: {}", out);
                        trackingStartOfLine = trackingStartOfLine & onlywhitespace;
                        println!("PartialCode: {}", variable);
                    },
                    '{' => {
                        println!("WriteCode: {}", out);
                        let mut name = variable;
                        if em.char_at(1) != '}' {
                            let length = variable.len() - 1;
                            name = &variable[0..length];
                        } else {
                            match br.next() {
                                None => break,
                                Some(c) => {
                                    if c != '}' {
                                        return Err("Improperly closed variable".to_string());
                                    }
                                }
                            }
                        }
                        println!("UnescpaedValueCode: {}", name);
                    },
                    '&' => {
                        println!("WriteCode: {}", out);
                        println!("UnescpaedValueCode: {}", variable);
                    },
                    '%' => {
                        println!("WriteCode: {}", out);
                        println!("PragmaCode: {}", variable);
                    },
                    '!' => {
                        println!("WriteCode: {}", out);
                        println!("CommentCode: {}", variable);
                    },
                    '=' => {
                        println!("WriteCode: {}", out);
                        println!("Delimiters: {}", tagName);
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
            Ok(())
        }
        _ => return Err("Failed to close tag".to_string()),
    }
}