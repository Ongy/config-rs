use ::ConfigAble;
use ::ConfigProvider;
use ::ParseError;

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


#[cfg(test)]
mod test {
    use ParseError;
    use ParseTmp;

    #[test]
    fn parsetmp_get_error() {
        assert!(ParseTmp::<String>::new("TestField".into()).get_value(&mut |_| {}) == Err(ParseError::Recoverable));
    }

    #[test]
    fn parsetmp_get_none() {
        assert!(ParseTmp::<Option<String>>::new("TestField".into()).get_value(&mut |_| {}) == Ok(None));
    }

    #[test]
    fn parsetmp_get_default() {
        let mut field = ParseTmp::<String>::new("TestField".into());
        field.set_default("TestStr".into());
        assert!(field.get_value(&mut |_| {}) == Ok("TestStr".into()));
    }

    //TODO: MAke more tests!
//    #[test]
//    fn parsetmp_replace_default() {
//        let mut field = ParseTmp::<String>::new("TestField".into());
//        field.set_default("TestStr".into());
//        field.push_found(
//
//    }
}
