use std;
use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;

impl<T> ConfigAble for Option<T>
    where T: ConfigAble {
    fn get_format<F>(set: &mut std::collections::HashSet<String>, fun: &mut F)
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

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {

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

#[cfg(test)]
mod test {
    use ConfigProvider;
    use ConfigAble;

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
}
