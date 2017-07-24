use std;
use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;

use implementations::literals::parse_char;

impl ConfigAble for char {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("Char: 'Rust Char'");
        }

    fn get_name() -> &'static str { "char" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<char, ParseError>
        where F: FnMut(String) {

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

#[cfg(test)]
mod test {
    use ConfigProvider;
    use ConfigAble;

    #[test]
    fn test_char_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::<String>::new_from_line("'\\\\'var".to_string());

        let val = char::parse_from(&mut provider, &mut fun);
        assert!(val == Ok('\\'));

        let nxt = provider.get_next();
        assert!(nxt == Some("var".to_string()));
    }

}
