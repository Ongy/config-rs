use std;
use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;

impl ConfigAble for i32 {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("i32: Digits");
        }

    fn get_name() -> &'static str { "i32" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            /* Fix this, it's majorly wrong */
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

impl ConfigAble for u32 {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("u32: Digits");
        }

    fn get_name() -> &'static str { "u32" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            /* Fix this, it's majorly wrong */
            let max = content.find(|c: char| c.is_whitespace()).unwrap_or(content.len());
            let tmp = &content[0 .. max];
            match tmp.parse::<u32>() {
                Ok(ret) => {
                    provider.consume(max, fun)?;
                    return Ok(ret);
                },
                Err(x) => {
                    fun(format!("Failed to parse '{}' into an u32: {}", tmp, x));
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


impl ConfigAble for u8 {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("u8: Digits");
        }

    fn get_name() -> &'static str { "u8" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {

        if let Some(content) = provider.get_next() {
            /* Fix this, it's majorly wrong */
            let max = content.find(|c: char| c.is_whitespace()).unwrap_or(content.len());
            let tmp = &content[0 .. max];
            match tmp.parse::<u8>() {
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
