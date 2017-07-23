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
enum ParseTmpI<T> {
    Empty,
    Default(T),
    Found(T),
    Failed,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTmp<T> {
    value: ParseTmpI<T>,
    name: String,
}

impl<T> ParseTmp<T> {

    fn new(name: String) -> Self {
        return Self { value: ParseTmpI::Empty, name: name };
    }

    fn set_default(&mut self, val: T) {
        self.value = ParseTmpI::Default(val);
    }
}

impl<T> ParseTmp<T>
    where T: ConfigAble {
    fn push_found<I, F>(&mut self, rhs: Result<T, ParseError>, provider: &ConfigProvider<I>, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String) {
        match rhs {
            Ok(val) => {
                /* ARGH! Borrow checker be smarter ! */
                self.value = match &mut self.value {
                    &mut ParseTmpI::Empty => {
                        ParseTmpI::Found(val)
                    },
                    &mut ParseTmpI::Failed => {
                        return Ok(());
                    },
                    &mut ParseTmpI::Found(ref mut x) => {
                        match x.merge(val) {
                            Err(()) => {
                                provider.print_error(0, fun);
                                fun(format!("Couldn't merge {}.", self.name));
                                ParseTmpI::Failed
                            },
                            _ => { return Ok(()); },
                        }
                    },
                    &mut ParseTmpI::Default(_) => {
                        ParseTmpI::Found(val)
                    },
                };
                return Ok(());
            },
            Err(ParseError::Recoverable) => {
                fun(format!("Tried to push Recoverable error for {}. Will continue", self.name));
                self.value = ParseTmpI::Failed;
                return Ok(());
            },
            Err(x) => {
                return Err(x);
            },
        }
    }

    fn get_value<F>(self, fun: &mut F) -> Result<T, ParseError>
        where F: FnMut(String) {
        match self.value {
            ParseTmpI::Found(x) => Ok(x),
            ParseTmpI::Default(x) => Ok(x),
            ParseTmpI::Empty => {
                match T::get_default() {
                    Ok(x) => Ok(x),
                    Err(_) => {
                        fun(format!("Couldn't default {}. You need to provide a value", self.name));
                        return Err(ParseError::Recoverable);
                    }
                }
            },
            ParseTmpI::Failed => {
                fun(format!("Can't get a value for {} since something failed.", self.name));
                return Err(ParseError::Recoverable);
            },
        }
    }
}

pub trait ConfigAble
    where Self: std::marker::Sized {

    /* Internally used function. This takes care of deduplicating format strings */
    fn get_format<F>(set: &mut HashSet<String>, fun: &mut F)
       where  F: FnMut(&str);

    /* Semi-Internal function. don't rely on the format. Used to identify types */
    fn get_name() -> &'static str;

    /* Get a String that describes the format that can be read in for this type
     * This will also contain all formats of types used transitivly.
     */
    fn get_format_str() -> String {
        let mut ret = String::new();
        let mut set = HashSet::new();
        Self::get_format(&mut set, &mut |x| ret.push_str(x));
        return ret;
    }

    /* Parse the object from a ConfigProvider
     *
     */
    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String);

    /* Get a default value For the type */
    // TODO: Should the error type be something more serious?
    fn get_default() -> Result<Self, ()>;

    /* Try to merge 2 sets of values found */
    fn merge(&mut self, rhs: Self) -> Result<(), ()>;
}

impl ConfigAble for String {
    fn get_format<F>(_: &mut HashSet<String>, fun: &mut F) 
        where F: FnMut(&str){
        fun("String: \"Rust String\"");
    }

    fn get_name() -> &'static str { "String" }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<String, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            match  str_lit(content.as_str()) {
                Ok((count, ret)) => {
                    provider.consume(count, fun)?;
                    return Ok(ret);
                },
                Err(x) => {
                    fun(x);
                    return Err(ParseError::Recoverable);
                }
            }
        }


        fun("At end of file while parsing String :(".to_string());
        return Err(ParseError::Final);
    }

    fn get_default() -> Result<Self, ()> { Err(()) }

    fn merge(&mut self, rhs: Self) -> Result<(), ()> { 
        self.push_str(rhs.as_str());
        return Ok(());
    }
}

impl ConfigAble for char {
    fn get_format<F>(_: &mut HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("Char: 'Rust Char'");
        }

    fn get_name() -> &'static str { "char" }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<char, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            match parse_char(content.as_str()) {
                Ok((count, ret)) => {
                    provider.consume(count, fun)?;
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

    fn get_default() -> Result<Self, ()> { Err(()) }

    fn merge(&mut self, rhs: Self) -> Result<(), ()> { if *self == rhs { Ok(()) } else { Err(()) } }
}

impl ConfigAble for i32 {
    fn get_format<F>(_: &mut HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("i32: Digits");
        }

    fn get_name() -> &'static str { "i32" }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            let max = content.find(|c: char| c.is_whitespace()).unwrap_or(content.len());
            let tmp = &content[0 .. max];
            match tmp.parse::<i32>() {
                Ok(ret) => {
                    provider.consume(max, fun)?;
                    return Ok(ret);
                },
                Err(x) => {
                    fun(format!("Failed to parse '{}' into an i32: {}", tmp, x));
                    return Err(ParseError::Recoverable);
                }
            }
        }

        fun("At end of file :(".to_string());
        return Err(ParseError::Final);
    }

    fn get_default() -> Result<Self, ()> { Err(()) }

    fn merge(&mut self, rhs: Self) -> Result<(), ()> { if *self == rhs { Ok(()) } else { Err(()) } }
}

impl<T> ConfigAble for Option<T>
    where T: ConfigAble {
    fn get_format<F>(set: &mut HashSet<String>, fun: &mut F)
        where F: FnMut(&str) {
        // TODO: Re-do the newline appending
        fun(format!("Option<{}>: Some({}) | None", T::get_name(), T::get_name()).as_str());
        let key = T::get_name().to_string();

        if !set.contains(&key) {
            set.insert(T::get_name().to_string());

            fun("\n");
            T::get_format(set, fun);
        }
    }

    fn get_name() -> &'static str {
        concat!("Option<", /*T::get_name(),*/ ">")
    }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            if content.starts_with("None") {
                provider.consume(4, fun)?;
                return Ok(None);
            }

            if content.starts_with("Some") {
                provider.consume(4, fun)?;
                provider.consume_char('(', fun)?;
                let ret = T::parse_from(provider, fun);
                provider.consume_char(')', fun)?;
                return Ok(Some(ret?));
            }

        }

        fun("At end of file :(".to_string());
        return Err(ParseError::Final);
    }

    fn get_default() -> Result<Self, ()> { 
        match <T as ConfigAble>::get_default() {
            Ok(x) => Ok(Some(x)),
            _ => Ok(None),
        }
    }

    fn merge(&mut self, rhs: Self) -> Result<(), ()> {
        match self {
            &mut None => {
                *self = rhs;
                return Ok(());
            },
            &mut Some(ref mut lhs) => {
                match rhs {
                    Some(x) => lhs.merge(x),
                    None => Ok(()),
                }
            },
        }
    }
}


impl<T> ConfigAble for Vec<T>
    where T: ConfigAble {
    fn get_format<F>(set: &mut HashSet<String>, fun: &mut F)
        where F: FnMut(&str) {
        // TODO: Re-do the newline appending
        fun(format!("Vec<{}>: [ {}, {}, ... ]", T::get_name(), T::get_name(), T::get_name()).as_str());

        let key = T::get_name().to_string();
        if !set.contains(&key) {
            set.insert(T::get_name().to_string());

            fun("\n");
            T::get_format(set, fun);
        }
    }

    fn get_name() -> &'static str {
        concat!("Vec<", /*T::get_name(),*/ ">")
    }

    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String) {

        let mut first = true;
        let mut ret = Vec::new();

        provider.consume_char('[', fun)?;
        loop {
            if provider.is_at_end() {
                provider.print_error(0, fun);
                fun("Reached end of file while reading vector :(".to_string());
                return Err(ParseError::Final);
            }

            if provider.peek_char() == Some(']') {
                provider.consume(1, fun)?;
                return Ok(ret);
            }

            if first {
                first = false;
            } else {
                provider.consume_char(',', fun)?;
            }

            ret.push(T::parse_from(provider, fun)?);
        }
    }

    fn get_default() -> Result<Self, ()> { Ok(Self::new()) }

    fn merge(&mut self, rhs: Self) -> Result<(), ()> {
        self.extend(rhs.into_iter());
        return Ok(());
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
enum InputSource {
    Stdin,
    Pipe(i32),
    Named(String),
    Spawn(String),
}

#[derive(Debug, ConfigAble)]
struct Input {
    source: InputSource,
    #[ConfigAttrs(default = "0")]
    layer: i32,
}

#[derive(Debug, ConfigAble)]
struct Config {
    #[ConfigAttrs(default = "Position::Global(Direction::Top)")]
    position: Position,
    inputs: Vec<Input>,
    #[ConfigAttrs(default = "\"ongybar\".to_string()")]
    title: String,
}

#[derive(Debug, ConfigAble)]
enum TestEnum {
    TestCon{s: String, c: char},
}

#[cfg(not(test))]
fn main() {
    println!("{}", Config::get_format_str());

    println!("{:?}", Config::parse_from(&mut ConfigProvider::new_from_str("{}"), &mut |x| println!("{}", x)));

    let lines = vec![
        (1, "{".to_string()),
        (2, "  inputs: [{source: Spawn(\"monk\"), source: Spawn(\"y\")}]".to_string()),
        (3, ", title: \"ongybar\"".to_string()),
        (3, ", inputs: [{source: Stdin, source: Stdin}]".to_string()),
        (4, "# a Comment".to_string()),
        (5, "}".to_string())
    ];

    let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());
    println!("{:?}", Config::parse_from(&mut provider, &mut |x| println!("{}", x)));
}

#[derive(Debug)]
pub struct ConfigProvider<I> {
    file: String,
    line: usize,
    column: usize,
    line_str: String,
    line_it: I,
}

impl<I> ConfigProvider<I> {
    pub fn get_next(&self) -> Option<String> {
        if self.column >= self.line_str.len() {
            return None;
        }

        unsafe {
            let slice = self.line_str.slice_unchecked(self.column, self.line_str.len());
            return Some(String::from(slice));
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.line_str.chars().nth(self.column)
    }


    pub fn print_error<F>(&self, index: usize, fun: &mut F)
        where F: FnMut(String) {
        fun(format!("Encountered error in {}:{},{}", self.file, self.line, self.column + index + 1));
    }

    pub fn is_at_end(&self) -> bool {
        return self.column == self.line_str.len();
    }
}

impl ConfigProvider<std::option::IntoIter<(usize, String)>> {
    pub fn new_from_line(line: String) -> Self {
        return Self::new_with_provider(Some((0, line)).into_iter(), "memory".to_string());
    }

    pub fn new_from_str(line: &str) -> Self {
        return Self::new_from_line(line.to_string());
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
                    self.consume(x, &mut |_| {}).unwrap();
                },
                None => {
                    self.skip_current();
                },
            }
        }
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

    pub fn consume_char<F>(&mut self, c: char, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String){
        //TODO: nth looks slooow
        if self.line_str.chars().nth(self.column) == Some(c) {
            self.consume(1, fun)?;
            return Ok(());
        }

        self.print_error(0, fun);
        fun(format!("Tried to consume {}", c));
        return Err(ParseError::Final);
    }

    pub fn consume<F>(&mut self, count: usize, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String) {
        if self.column + count > self.line_str.len() {
            self.print_error(0, fun);
            fun(format!("Tried to consume more than currently available: {}", count));
            return Err(ParseError::Final);
        }

        self.column = self.column + count;

        self.skip_whitespace();

        if self.column == self.line_str.len() {
            self.get_next_line();
        }

        return Ok(());
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
        assert!(String::get_format_str() == "String: \"Rust String\"");
    }

    #[test]
    fn test_option_format() {
        assert!(<Option<String> as ConfigAble>::get_format_str().starts_with("Option<String>: Some(String) | None\n"));
    }

    #[test]
    fn test_option_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("Some(\"TestStr\")".to_string());
        assert!(<Option<String> as ConfigAble>::parse_from(&mut provider, &mut fun) == Ok(Some("TestStr".to_string())));
        assert!(provider.get_next() == None);

        let mut provider2 = ConfigProvider::new_from_line("None".to_string());
        assert!(<Option<String> as ConfigAble>::parse_from(&mut provider2, &mut fun) == Ok(None));
        assert!(provider2.get_next() == None);
    }

    #[test]
    fn test_option_default() {
        assert!(<Option<String> as ConfigAble>::get_default() == Ok(None));
        assert!(<Option<Option<String>> as ConfigAble>::get_default() == Ok(Some(None)));
    }


    #[derive(ConfigAble, PartialEq, Eq)]
    struct DefaultTest1(Option<String>, Option<String>);

    #[derive(ConfigAble, PartialEq, Eq)]
    struct DefaultTest2(Option<String>, String);

    #[test]
    fn test_default_struct() {
        assert!(DefaultTest1::get_default() == Ok(DefaultTest1(None, None)));
        assert!(DefaultTest2::get_default() == Err(()));
    }


    #[test]
    fn test_vec_format() {
        assert!(<Vec<char> as ConfigAble>::get_format_str().starts_with("Vec<char>: [ char, char, ... ]\n"));
    }

    #[test]
    fn test_vec_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("[]".to_string());
        assert!(<Vec<char> as ConfigAble>::parse_from(&mut provider, &mut fun) == Ok(vec![]));
        assert!(provider.get_next() == None);

        let mut provider2 = ConfigProvider::new_from_line("[ '1', '2', '3' ]".to_string());
        assert!(<Vec<char> as ConfigAble>::parse_from(&mut provider2, &mut fun) == Ok(vec!['1', '2', '3']));
        assert!(provider2.get_next() == None);
    }

    #[derive(ConfigAble, PartialEq, Eq)]
    enum SimpleEnum {
        SimpleCon1,
        SimpleCon2,
    }

    #[test]
    fn test_simple_enum_format() {
        assert!(SimpleEnum::get_format_str() == "SimpleEnum: SimpleCon1 | SimpleCon2");
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
        assert!(TupleEnum::get_format_str() == "TupleEnum: TupleCon1(String) | TupleCon2(String)\nString: \"Rust String\"");
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
        assert!(TupleStruct::get_format_str().starts_with("TupleStruct: (String, char)"));
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

    #[test]
    fn test_struct_struct_format() {
        assert!(StructStruct::get_format_str().starts_with("StructStruct: {s: String, c: char}"));
    }

    #[test]
    fn test_struct_struct_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("{ c: 'C', s: \"TestStr\" }".to_string());
        assert!(StructStruct::parse_from(&mut provider, &mut fun) == Ok(StructStruct{s: "TestStr".to_string(), c:'C'}));
        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_struct_struct_parse_fail() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("{ k: 'C', s: \"TestStr\" }".to_string());
        assert!(StructStruct::parse_from(&mut provider, &mut fun) == Err(ParseError::Final));
    }

    #[test]
    fn test_config_provider_string() {
        let mut provider = ConfigProvider::new_from_line("This is a line".to_string());

        assert!(provider.get_next() == Some("This is a line".to_string()));

        provider.consume(4, &mut |_| {}).unwrap();
        assert!(provider.get_next() == Some("is a line".to_string()));

        provider.consume(9, &mut |_| {}).unwrap();
        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_config_provider_nextline() {
        let lines = vec![(1, "Line1   \n".to_string()), (2, "  Line2".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("Line1   \n".to_string()));
        provider.consume(5, &mut |_| {}).unwrap();

        assert!(provider.get_next() == Some("Line2".to_string()));

        provider.consume(5, &mut |_| {}).unwrap();
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

        provider.consume(5, &mut |_| {}).unwrap();

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
        provider.consume(4, &mut |_| {}).unwrap();

        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_provider_skips_comment() {
        let lines = vec![(1, "#lin1".to_string()), (2, "   #line2".to_string()), (3, "line#3   #another".to_string()), (4, "#line4".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("line#3   #another".to_string()));
        provider.consume(6, &mut |_| {}).unwrap();

        assert!(provider.get_next() == Some("#another".to_string()));
        provider.consume(8, &mut |_| {}).unwrap();

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
