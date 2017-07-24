#[allow(unused_imports)]
#[macro_use]
extern crate rs_config_derive;

mod provider;
mod parsetmp;
mod implementations;

use std::collections::HashSet;
pub use provider::ConfigProvider;
pub use parsetmp::ParseTmp;

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
}
