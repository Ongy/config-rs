#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;

#[derive(ConfigAble, PartialEq, Eq)]
enum StructEnum {
    Var1 { s: String, c: char },
    Var2 { c: String, s: char },
}

#[test]
fn test_struct_enum_format() {
    assert!(StructEnum::get_format_str().starts_with("StructEnum: Var1{s: String, c: char} | Var2{c: String, s: char}"));
}

#[test]
fn test_struct_enum_parse() {
    let mut builder = String::new();
    let mut fun = |x: String| builder.push_str(x.as_str());
    let mut provider = rs_config::ConfigProvider::new_from_str("Var2 { s: 'C', c: \"TestStr\" }");
    assert!(StructEnum::parse_from(&mut provider, &mut fun) == Ok(StructEnum::Var2{c: "TestStr".to_string(), s:'C'}));
    assert!(provider.get_next() == None);
}

