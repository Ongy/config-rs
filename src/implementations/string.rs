use std;
use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;

use implementations::literals::str_lit;

impl ConfigAble for String {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str){
        fun("String: \"Rust String\"");
    }

    fn get_name() -> &'static str { "String" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<String, ParseError>
        where F: FnMut(String) {

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

#[cfg(test)]
mod test {
    use ConfigProvider;
    use ConfigAble;

    #[test]
    fn test_string_format() {
        assert!(String::get_format_str() == "String: \"Rust String\"");
    }

    #[test]
    fn test_string_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_line("\"This is\\n \\\"a line\"var".to_string());

        let val = String::parse_from(&mut provider, &mut fun);
        assert!(val == Ok("This is\n \"a line".to_string()));

        let nxt = provider.get_next();
        assert!(nxt == Some("var".to_string()));
    }

}
