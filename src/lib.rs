#[cfg(test)]
mod tests {
    #[test]
    fn compile() {

    }
}

mod mustache {

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

    struct Code<'a> {
        name: String,
        identity: String,
        appended: String,
        mustache: Mustache<'a>,
    }

    struct Mustache<'a> {
        codes: &'a Vec<Box<IsCode>>,
    }

    trait IsCode {
        fn execute<'a>(&'a self, writer: &'a Write, scopes: &'a Vec<Box<Any>>) -> &'a Write;
    }

    impl <'b> IsCode for Mustache<'b> {
        fn execute<'a>(&'a self, writer: &'a Write, scopes: &'a Vec<Box<Any>>) -> &'a Write {
            for code in self.codes {
                code.execute(writer, scopes);
            }
            writer
        }
    }

    fn parse(template: &String) -> &IsCode {
        let a = Mustache{ codes: &vec![] };
        &a
    }
}