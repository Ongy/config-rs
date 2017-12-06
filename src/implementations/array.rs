use std;
use ::provider::ConfigProvider;
use ::ConfigAble;
use ParseError;

use std::vec::Vec;

impl<T: ConfigAble> ConfigAble for [T;4] {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("[T;4]: [T, T, T, T]");
        }

    fn get_name() -> &'static str { "[T;4]" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {
        let vec: Vec<T> = ConfigAble::parse_from(provider, fun)?;

        if vec.len() != 4 {
            fun(format!("Expected array of size 4, got array of size: {}", vec.len()));
            return Err(ParseError::Recoverable);
        }

        let mut iter = vec.into_iter();

        let ret = [iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap()];
        return Ok(ret);
    }

    fn get_default() -> Result<Self, ()> {
        Ok([T::get_default()?, T::get_default()?, T::get_default()?, T::get_default()?])
    }

    fn merge(&mut self, _: Self) -> Result<(), ()> {
        return Err(());
    }
}

impl<T: ConfigAble> ConfigAble for [T;6] {
    fn get_format<F>(_: &mut std::collections::HashSet<String>, fun: &mut F) 
        where F: FnMut(&str) {
            fun("[T;4]: [T, T, T, T, T, T]");
        }

    fn get_name() -> &'static str { "[T;6]" }

    fn parse_from<F>(provider: &mut ConfigProvider, fun: &mut F) -> Result<Self, ParseError>
        where F: FnMut(String) {
        let vec: Vec<T> = ConfigAble::parse_from(provider, fun)?;

        if vec.len() != 6 {
            fun(format!("Expected array of size 6, got array of size: {}", vec.len()));
            return Err(ParseError::Recoverable);
        }

        let mut iter = vec.into_iter();

        let ret = [iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap()];
        return Ok(ret);
    }

    fn get_default() -> Result<Self, ()> {
        Ok([T::get_default()?, T::get_default()?, T::get_default()?, T::get_default()?, T::get_default()?, T::get_default()?])
    }

    fn merge(&mut self, _: Self) -> Result<(), ()> {
        return Err(());
    }
}
