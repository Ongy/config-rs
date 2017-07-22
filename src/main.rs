#![allow(dead_code)]

#[macro_use]
extern crate rs_config_derive;
mod literals;

use literals::str_lit;
use literals::parse_char;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Recoverable,
    Final,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseTmp<T> {
    Empty,
    Default(T),
    Found(T),
    Failed,
}

impl<T> ParseTmp<T> {
    fn push_found<F>(&mut self, rst: Result<T, ParseError>, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String) {
        match rst {
            Ok(val) => {
                match self {
                    &mut ParseTmp::Empty => {
                        *self = ParseTmp::Found(val);
                        return Ok(());
                    },
                    &mut ParseTmp::Failed => {
                        return Ok(());
                    },
                    &mut ParseTmp::Found(_) => {
                        fun("Tried to parse something twice. Gonna fail, this isn't implemented yet".to_string());
                        *self = ParseTmp::Failed;
                        return Ok(());
                    },
                    &mut ParseTmp::Default(_) => {
                        *self = ParseTmp::Found(val);
                        return Ok(());
                    },
                }
            },
            Err(ParseError::Recoverable) => {
                fun("Tried to push Recoverable error. Will continue".to_string());
                *self = ParseTmp::Failed;
                return Ok(());
            },
            Err(x) => {
                return Err(x);
            },
        }
    }

    fn get_value(self) -> Result<T, ParseError> {
        match self {
            ParseTmp::Found(x) => Ok(x),
            ParseTmp::Default(x) => Ok(x),
            _ => Err(ParseError::Recoverable),
        }
    }
}

trait ConfigAble
    where Self: std::marker::Sized {

    /* Internally used function. This takes care of deduplicating format strings */
    fn get_format(set: &mut HashSet<String>) -> String;

    /* Semi-Internal function. don't rely on the format. Used to identify types */
    fn get_name() -> &'static str;

    /* Get a String that describes the format that can be read in for this type
     * This will also contain all formats of types used transitivly.
     */
    fn get_format_str() -> String {
        let mut set = HashSet::new();
        return Self::get_format(&mut set);
    }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String);
}

impl ConfigAble for String {
    fn get_format(_: &mut HashSet<String>) -> String { String::from("String: \"Rust String\"\n") }

    fn get_name() -> &'static str { "String" }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<String, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            match  str_lit(content.as_str()) {
                Ok((count, ret)) => {
                    provider.consume(count).unwrap();
                    return Ok(ret);
                },
                Err(x) => {
                    fun(x);
                    return Err(ParseError::Recoverable);
                }
            }
        }


        fun("At end of file :(".to_string());
        return Err(ParseError::Final);
    }
}

impl ConfigAble for char {
    fn get_format(_: &mut HashSet<String>) -> String { String::from("Char: 'Rust Char'\n") }

    fn get_name() -> &'static str { "char" }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<char, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            match parse_char(content.as_str()) {
                Ok((count, ret)) => {
                    provider.consume(count).unwrap();
                    return Ok(ret);
                },
                Err(x) => {
                    fun(x);
                    return Err(ParseError::Recoverable);
                }
            }
        }

        fun("At end of file :(".to_string());
        return Err(ParseError::Final);
    }
}

#[derive(Debug, ConfigAble)]
enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, ConfigAble)]
enum Position {
    Global(Direction),
    Monitor(String, Direction),
}

#[derive(Debug, ConfigAble)]
struct Config {
    position: Position,
    spawn: String,
}

#[cfg(not(test))]
fn main() {
    let mut set = HashSet::new();

    print!("{}", Config::get_format(&mut set));

    let lines = vec![(1, "{".to_string()), (2, "position: Global(Top)".to_string()), (3, "spawn: \"monky\"".to_string()), (4, "}".to_string())];
    let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "ExampleString".to_string());

    print!("{:?}\n", Config::parse_from(&mut provider, &mut |x| println!("{}", x)));

    let lines2 = vec![(1, "{".to_string()), (2, "position: Global(Top), position: Global(Bottom)".to_string()), (3, "spawn: \"monky\", spawn: \"monky\"".to_string()), (4, "}".to_string())];
    let mut provider2 = ConfigProvider::new_with_provider(lines2.into_iter(), "ExampleString".to_string());

    print!("{:?}\n", Config::parse_from(&mut provider2, &mut |x| println!("{}", x)));
}

#[derive(Debug)]
pub struct ConfigProvider<I> {
    file: String,
    line: usize,
    column: usize,
    line_str: String,
    line_it: I,
}

impl ConfigProvider<std::option::IntoIter<(usize, String)>> {
    pub fn new_from_line(line: String) -> ConfigProvider<std::option::IntoIter<(usize, String)>> {
        return ConfigProvider::new_with_provider(Some((0, line)).into_iter(), "memory".to_string());
    }
}

impl<I> ConfigProvider<I>
    where I: std::iter::Iterator<Item=(usize, String)> {

    fn skip_current(&mut self) { 
        self.column = self.line_str.len();
        self.get_next_line();
    }

    fn skip_whitespace(&mut self) {
        unsafe {
            let pos = self.line_str.slice_unchecked(self.column, self.line_str.len()).find(|c: char| !c.is_whitespace());

            match pos {
                Some(0) => {},
                Some(x) => {
                    self.consume(x).unwrap();
                },
                None => {
                    self.skip_current();
                },
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.line_str.chars().nth(self.column)
    }

    fn get_next_line(&mut self) {
        if let Some(line) = self.line_it.next() {
            self.line_str = line.1;
            self.line = line.0;
            self.column = 0;

            self.skip_whitespace();
            // TODO: this looks sloooow
            if self.peek_char() == Some('#') {
                self.skip_current();
            }
        }
    }

    pub fn new_with_provider(it: I, file: String) -> Self {
        let mut ret = ConfigProvider { file: file,
            line: 1, column: 0,
            line_str: String::from(""),
            line_it: it
        };
        ret.get_next_line();

        return ret;
    }

    pub fn get_next(&self) -> Option<String> {
        if self.column >= self.line_str.len() {
            return None;
        }

        unsafe {
            let slice = self.line_str.slice_unchecked(self.column, self.line_str.len());
            return Some(String::from(slice));
        }
    }

    pub fn consume_char<F>(&mut self, c: char, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String){
        //TODO: nth looks slooow
        if self.line_str.chars().nth(self.column) == Some(c) {
            self.consume(1).unwrap();
            return Ok(());
        }

        self.print_error(0, fun);
        fun(format!("Tried to consume {}", c));
        return Err(ParseError::Final);
    }

    pub fn consume(&mut self, count: usize) -> Result<(), &'static str> {
        if self.column + count > self.line_str.len() {
            return Err("Cannot consume more than currently provided");
        }

        self.column = self.column + count;

        self.skip_whitespace();

        if self.column == self.line_str.len() {
            self.get_next_line();
        }

        return Ok(());
    }

    pub fn print_error<F>(&self, index: usize, fun: &mut F)
        where F: FnMut(String) {
        fun(format!("Encountered error in {}:{},{}", self.file, self.line, self.column + index + 1));
    }
}


#[cfg(test)]
mod test {
    use ConfigAble;
    use ConfigProvider;
    use ParseError;
    use ParseTmp;

    #[test]
    fn test_string_format() {
        assert!(String::get_format_str() == "String: \"Rust String\"\n");
    }

    #[derive(ConfigAble, PartialEq, Eq)]
    enum SimpleEnum {
        SimpleCon1,
        SimpleCon2,
    }

    #[test]
    fn test_simple_enum_format() {
        assert!(SimpleEnum::get_format_str() == "SimpleEnum: SimpleCon1 | SimpleCon2\n");
        assert!(SimpleEnum::get_name() == "SimpleEnum");
    }

    #[test]
    fn test_simple_enum_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("SimpleCon1".to_string());
        assert!(SimpleEnum::parse_from(&mut provider, &mut fun) == Ok(SimpleEnum::SimpleCon1));
        assert!(provider.get_next() == None);

        let mut provider2 = ConfigProvider::new_from_line("SimpleCon2asdf".to_string());
        assert!(SimpleEnum::parse_from(&mut provider2, &mut fun) == Ok(SimpleEnum::SimpleCon2));
        assert!(provider2.get_next() == Some("asdf".to_string()));

        let mut provider3 = ConfigProvider::new_from_line("SimpleCon3".to_string());
        match SimpleEnum::parse_from(&mut provider3, &mut fun) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }

    #[derive(ConfigAble, PartialEq, Eq)]
    enum TupleEnum {
        TupleCon1(String),
        TupleCon2(String),
    }

    #[test]
    fn test_tuple_enum_format() {
        assert!(TupleEnum::get_format_str() == "TupleEnum: TupleCon1(String) | TupleCon2(String)\nString: \"Rust String\"\n");
        assert!(TupleEnum::get_name() == "TupleEnum");
    }

    #[test]
    fn test_tuple_enum_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("TupleCon1 ( \"TestStr\")".to_string());
        assert!(TupleEnum::parse_from(&mut provider, &mut fun) == Ok(TupleEnum::TupleCon1("TestStr".to_string())));
        assert!(provider.get_next() == None);

        let mut provider2 = ConfigProvider::new_from_line("TupleCon2(\"TestStr\") asdf".to_string());
        assert!(TupleEnum::parse_from(&mut provider2, &mut fun) == Ok(TupleEnum::TupleCon2("TestStr".to_string())));
        assert!(provider2.get_next() == Some("asdf".to_string()));

        let mut provider3 = ConfigProvider::new_from_line("TupleCon3(\"TestStr\")".to_string());
        match SimpleEnum::parse_from(&mut provider3, &mut fun) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let mut provider4 = ConfigProvider::new_from_line("TupleCon2\"TestStr\")".to_string());
        match SimpleEnum::parse_from(&mut provider4, &mut fun) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        let mut provider5 = ConfigProvider::new_from_line("TupleCon2(\"TestStr)".to_string());
        match SimpleEnum::parse_from(&mut provider5, &mut fun) {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }


    #[derive(ConfigAble, PartialEq, Eq)]
    enum StructEnum {
        Var1 { s: String, c: char },
        Var2 { c: String, s: char },
    }

    fn test_struct_enum_format() {
        assert!(StructEnum::get_format_str().starts_with("StructEnum: Var1(s: String, c: char) | Var2(c: String, s: char)"));
    }

    fn test_struct_enum_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("Var2 { s: 'C', c: \"TestStr\" }".to_string());
        assert!(StructEnum::parse_from(&mut provider, &mut fun) == Ok(StructEnum::Var2{c: "TestStr".to_string(), s:'C'}));
        assert!(provider.get_next() == None);
    }


    #[derive(ConfigAble, PartialEq, Eq)]
    struct TupleStruct (String, char);

    #[test]
    fn test_tuple_struct_format() {
        assert!(TupleStruct::get_format_str().starts_with("TupleStruct: (String, char)\n"));
    }

    #[test]
    fn test_tuple_struct_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("( \"TestStr1\", 'C' )".to_string());
        assert!(TupleStruct::parse_from(&mut provider, &mut fun) == Ok(TupleStruct("TestStr1".to_string(), 'C')));
        assert!(provider.get_next() == None);
    }

    #[derive(ConfigAble, PartialEq, Eq)]
    struct StructStruct {
        s: String,
        c: char,
    }

    fn test_struct_struct_format() {
        assert!(StructStruct::get_format_str().starts_with("StructStruct: {s: String, c: char}"));
    }

    fn test_struct_struct_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("{ c: 'C', s: \"TestStr\" }".to_string());
        assert!(StructStruct::parse_from(&mut provider, &mut fun) == Ok(StructStruct{s: "TestStr".to_string(), c:'C'}));
        assert!(provider.get_next() == None);
    }


    #[test]
    fn test_config_provider_string() {
        let mut provider = ConfigProvider::new_from_line("This is a line".to_string());

        assert!(provider.get_next() == Some("This is a line".to_string()));

        provider.consume(4).unwrap();
        assert!(provider.get_next() == Some("is a line".to_string()));

        provider.consume(9).unwrap();
        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_config_provider_nextline() {
        let lines = vec![(1, "Line1   \n".to_string()), (2, "  Line2".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("Line1   \n".to_string()));
        provider.consume(5).unwrap();

        assert!(provider.get_next() == Some("Line2".to_string()));

        provider.consume(5).unwrap();
        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_config_provider_err_str() {
        let lines = vec![(1, "Line1   \n".to_string()), (2, "  Line2".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());
        let mut val = String::new();

        {
            let mut fun = |x| val = x;
            provider.print_error(2, &mut fun);
        }
        assert!(val == "Encountered error in Testfile:1,3");

        provider.consume(5).unwrap();

        {
            let mut fun = |x| val = x;
            provider.print_error(1, &mut fun);
        }
        assert!(val == "Encountered error in Testfile:2,4");
    }

    #[test]
    fn test_provider_skips_whiteline() {
        let lines = vec![(1, "  \t".to_string()), (2, "  ".to_string()), (3, "  line".to_string()), (4, "  \t\t".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("line".to_string()));
        provider.consume(4).unwrap();

        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_provider_skips_comment() {
        let lines = vec![(1, "#lin1".to_string()), (2, "   #line2".to_string()), (3, "line#3   #another".to_string()), (4, "#line4".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("line#3   #another".to_string()));
        provider.consume(6).unwrap();

        assert!(provider.get_next() == Some("#another".to_string()));
        provider.consume(8).unwrap();

        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_string_parsing() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("\"This is\\n \\\"a line\"var".to_string());

        let val = String::parse_from(&mut provider, &mut fun);
        assert!(val == Ok("This is\n \"a line".to_string()));

        let nxt = provider.get_next();
        assert!(nxt == Some("var".to_string()));
    }

    #[test]
    fn test_char_parsing() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("'\\\\'var".to_string());

        let val = char::parse_from(&mut provider, &mut fun);
        assert!(val == Ok('\\'));

        let nxt = provider.get_next();
        assert!(nxt == Some("var".to_string()));
    }

}
