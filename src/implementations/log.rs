extern crate log;

use self::log::LogLevel;

use std::collections::HashSet;

use ::ConfigAble;
use ::provider::ConfigProvider;
use ParseError;

impl ConfigAble for LogLevel {
    fn get_format<F>(_: &mut HashSet<String>, fun: &mut F)
           where  F: FnMut(&str) {
       fun("Error | Warn | Info | Debug | Trace")
   }

    fn get_name() -> &'static str { "LogLevel" }

    fn get_default() -> Result<Self, ()> { Ok(LogLevel::Warn) }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
           where  F: FnMut(String) {
        if let Some(tmp) = provider.get_next() {

            let word: String = tmp.chars().take_while(|c| c.is_alphabetic()).collect();

            let ret = match word.as_ref() {
                    "Warn" => Ok(LogLevel::Warn),
                    "Info" => Ok(LogLevel::Info),
                    "Debug" => Ok(LogLevel::Debug),
                    "Trace" => Ok(LogLevel::Trace),
                    _ => Err(ParseError::Recoverable),
                };

            if let Ok(_) = ret {
                provider.consume(word.len(), fun)?;
            }

            return ret;
        }

        fun(String::from("At end of file :("));
        Err(ParseError::Final)
   }
}
