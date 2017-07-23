#[allow(unused_imports)]
#[macro_use]
extern crate rs_config_derive;

mod provider;
mod implementations;

use std::collections::HashSet;
pub use provider::ConfigProvider;

#[derive(Debug, PartialEq, Eq)]
/// The error type used by config-rs.
/// This will be returned by (sub) parses
pub enum ParseError {
    /// We encountered some error that invalidates the config, but it's still possible to continue
    /// parsing.
    /// This can happen e.g. when a key was specified twice, but couldn't be merged.
    Recoverable,
    /// Something happened while parsing, that breaks the parser state.
    /// This happens when the config provided doesn't follow the required format.
    Final,
}

#[derive(Debug, PartialEq, Eq)]
/// Helper enum for saving named field parser state
enum ParseTmpI<T> {
    /// Didn't find the key yet, and don't have a default
    /// This can return the type default
    Empty,
    /// Didn't find the value yet. Contains the default given with attributes
    Default(T),
    /// Found the key. Contains the current content for the value
    Found(T),
    /// Failed to parse (sub-parser failed, merge failed, etc.)
    Failed,
}

#[derive(Debug, PartialEq, Eq)]
/// Helper struct for named field parsing
pub struct ParseTmp<T> {
    /// The actual value
    value: ParseTmpI<T>,
    /// The name of the field to parse. Used for error messages
    name: String,
}

impl<T> ParseTmp<T> {
    /// Creates a new temporary holder for the fields value
    /// # Arguments
    /// * `name`: The name of the field
    pub fn new(name: String) -> Self {
        return Self { value: ParseTmpI::Empty, name: name };
    }

    /// Set the default value from the attribute
    /// # Arguments
    /// * `val`: The default value
    pub fn set_default(&mut self, val: T) {
        self.value = ParseTmpI::Default(val);
    }
}

impl<T> ParseTmp<T>
    where T: ConfigAble {
    /// Push a value found during parsing into the ParseTmp
    /// # Arguments
    /// * `rhs`: The value found while parsing
    /// * `provider`: The ConfigProvider currently in use. This is required for error reporting
    /// * `fun`: The error reporting function. Most likely either printing, or appending to string.
    pub fn push_found<I, F>(&mut self, rhs: Result<T, ParseError>, provider: &ConfigProvider<I>, fun: &mut F) -> Result<(), ParseError>
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

    /// Get the final value of the field.
    ///
    /// This will be in descending priority:
    /// * Value found
    /// * Attribute default
    /// * Type default
    /// * ParseError
    /// # Arguments
    /// * `fun`: The error reporting function
    pub fn get_value<F>(self, fun: &mut F) -> Result<T, ParseError>
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
    /// Internal function for printing/building format
    ///
    /// This does include all types used (transitivly).
    /// # Arguments
    /// * `set`: A HashSet to keep track of types already printed
    /// * `fun`: The function to print/append the format with
    fn get_format<F>(set: &mut HashSet<String>, fun: &mut F)
       where  F: FnMut(&str);

    /* Semi-Internal function. don't rely on the format. Used to identify types */
    /// Get a name of this type
    fn get_name() -> &'static str;

    /// Get the format that will be parsed as String.
    ///
    /// See get_format
    fn get_format_str() -> String {
        let mut ret = String::new();
        let mut set = HashSet::new();
        Self::get_format(&mut set, &mut |x| ret.push_str(x));
        return ret;
    }

    /// Parse an object from a ConfigProvider.
    ///
    /// # Arguments
    /// * `provider`: The ConfigProvider providing the config lines
    /// * `fun`: The error reporting function
    fn parse_from<I, F>(provider: &mut ConfigProvider<I>, fun: &mut F) -> Result<Self, ParseError>
        where I: std::iter::Iterator<Item=(usize, String)>,
              F: FnMut(String);

    // TODO: Should the error type be something more serious?
    /// Get a default value for this type
    fn get_default() -> Result<Self, ()>;

    /// Try to merge an object of this type with another (in case multiple are specified in the
    /// config)
    fn merge(&mut self, rhs: Self) -> Result<(), ()>;
}

#[cfg(test)]
mod test {
    //use ::self as rs_config;
    use ConfigAble;
    use ConfigProvider;
    use ParseError;
    use ParseTmp;

    #[derive(ConfigAble, PartialEq, Eq)]
    struct DefaultTest1(Option<String>, Option<String>);

    #[derive(ConfigAble, PartialEq, Eq)]
    struct DefaultTest2(Option<String>, String);

    #[test]
    fn test_default_struct() {
        assert!(DefaultTest1::get_default() == Ok(DefaultTest1(None, None)));
        assert!(DefaultTest2::get_default() == Err(()));
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

    #[test]
    fn test_struct_enum_format() {
        assert!(StructEnum::get_format_str().starts_with("StructEnum: Var1{s: String, c: char} | Var2{c: String, s: char}"));
    }

    #[test]
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
}
