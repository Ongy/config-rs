use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;
use std;

use std::net::Ipv4Addr;
use std::str::FromStr;

impl ConfigAble for Ipv4Addr {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("a.b.c.d");
        }

    fn get_name() -> &'static str { "Ipv4Addr" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            let parse_str: String = content.chars().take_while(|x| x.is_digit(10) || *x == '.').collect();
            provider.consume(parse_str.len(), fun)?;

            return match Ipv4Addr::from_str(parse_str.as_str()) {
                Ok(x) => Ok(x),
                Err(_) => Err(ParseError::Recoverable),
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
    use std::net::Ipv4Addr;

    #[test]
    fn test_ip_parse() {
        let mut builder = String::new();
        let mut fun = |x: String| builder.push_str(x.as_str());
        let mut provider = ConfigProvider::new_from_str("127.0.0.1var");

        let val = Ipv4Addr::parse_from(&mut provider, &mut fun);
        assert!(val == Ok(Ipv4Addr::new(127, 0, 0, 1)));

        let nxt = provider.get_next();
        assert!(nxt == Some("var".to_string()));
    }

}
