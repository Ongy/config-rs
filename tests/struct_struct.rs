#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;


#[derive(ConfigAble, PartialEq, Eq)]
struct StructStruct {
    s: String,
    c: char,
}

#[test]
fn test_struct_struct_format() {
    assert!(StructStruct::get_format_str().starts_with("StructStruct: {s: String, c: char}"));
}

#[test]
fn test_struct_struct_parse() {
    let mut builder = String::new();
    let mut fun = |x: String| builder.push_str(x.as_str());
    let mut provider = rs_config::ConfigProvider::new_from_str("{ c: 'C', s: \"TestStr\" }");
    assert!(StructStruct::parse_from(&mut provider, &mut fun) == Ok(StructStruct{s: "TestStr".to_string(), c:'C'}));
    assert!(provider.get_next() == None);
}

#[test]
fn test_struct_struct_parse_fail() {
    let mut builder = String::new();
    let mut fun = |x: String| builder.push_str(x.as_str());
    let mut provider = rs_config::ConfigProvider::new_from_str("{ k: 'C', s: \"TestStr\" }");
    assert!(StructStruct::parse_from(&mut provider, &mut fun) == Err(rs_config::ParseError::Final));
}
