#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;

#[derive(ConfigAble, PartialEq, Eq)]
struct TupleStruct (String, char);

#[test]
fn test_tuple_struct_format() {
    assert!(TupleStruct::get_format_str().starts_with("TupleStruct: (String, char)"));
}

#[test]
fn test_tuple_struct_parse() {
    let mut builder = String::new();
    let mut fun = |x: String| builder.push_str(x.as_str());
    let mut provider = rs_config::ConfigProvider::<String>::new_from_line("( \"TestStr1\", 'C' )".to_string());
    assert!(TupleStruct::parse_from(&mut provider, &mut fun) == Ok(TupleStruct("TestStr1".to_string(), 'C')));
    assert!(provider.get_next() == None);
}
