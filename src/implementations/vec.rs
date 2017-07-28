use std;
use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;

impl<T> ConfigAble for Vec<T>
    where T: ConfigAble {
    fn get_format<F>(set: &mut std::collections::HashSet<String>, fun: &mut F)
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

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {

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


#[cfg(test)]
mod test {
    use ConfigProvider;
    use ConfigAble;

    #[test]
    fn test_vec_format() {
        assert!(<Vec<char> as ConfigAble>::get_format_str().starts_with("Vec<char>: [ char, char, ... ]\n"));
    }

    #[test]
    fn test_vec_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_str("[]");
        assert!(<Vec<char> as ConfigAble>::parse_from(&mut provider, &mut fun) == Ok(vec![]));
        assert!(provider.get_next() == None);

        let mut provider2 = ConfigProvider::new_from_str("[ '1', '2', '3' ]");
        assert!(<Vec<char> as ConfigAble>::parse_from(&mut provider2, &mut fun) == Ok(vec!['1', '2', '3']));
        assert!(provider2.get_next() == None);
    }
}
