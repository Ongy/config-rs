#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;

#[derive(ConfigAble, PartialEq, Eq)]
enum SimpleEnum {
    SimpleCon1,
    SimpleCon2,
}

#[test]
fn test_simple_enum_format() {
    assert!(SimpleEnum::get_format_str() == "SimpleEnum: SimpleCon1 | SimpleCon2");
    assert!(SimpleEnum::get_name() == "SimpleEnum");
}

#[test]
fn test_simple_enum_parse() {
    let mut builder = String::new();
    let mut fun = |x: String| builder.push_str(x.as_str());
    let mut provider = rs_config::ConfigProvider::<String>::new_from_line("SimpleCon1".to_string());
    assert!(SimpleEnum::parse_from(&mut provider, &mut fun) == Ok(SimpleEnum::SimpleCon1));
    assert!(provider.get_next() == None);

    let mut provider2 = rs_config::ConfigProvider::<String>::new_from_line("SimpleCon2asdf".to_string());
    assert!(SimpleEnum::parse_from(&mut provider2, &mut fun) == Ok(SimpleEnum::SimpleCon2));
    assert!(provider2.get_next() == Some("asdf".to_string()));

    let mut provider3 = rs_config::ConfigProvider::<String>::new_from_line("SimpleCon3".to_string());
    match SimpleEnum::parse_from(&mut provider3, &mut fun) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}
