#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;

#[derive(ConfigAble, PartialEq, Eq)]
enum TupleEnum {
    TupleCon1(String),
    TupleCon2(String),
}


#[test]
fn test_tuple_enum_parse() {
    let mut builder = String::new();
    let mut fun = |x: String| builder.push_str(x.as_str());
    let mut provider = rs_config::ConfigProvider::new_from_str("TupleCon1 ( \"TestStr\")");
    assert!(TupleEnum::parse_from(&mut provider, &mut fun) == Ok(TupleEnum::TupleCon1("TestStr".to_string())));
    assert!(provider.get_next() == None);

    let mut provider2 = rs_config::ConfigProvider::new_from_str("TupleCon2(\"TestStr\") asdf".to_string());
    assert!(TupleEnum::parse_from(&mut provider2, &mut fun) == Ok(TupleEnum::TupleCon2("TestStr".to_string())));
    assert!(provider2.get_next() == Some("asdf".to_string()));

    let mut provider3 = rs_config::ConfigProvider::new_from_str("TupleCon3(\"TestStr\")");
    match TupleEnum::parse_from(&mut provider3, &mut fun) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }

    let mut provider4 = rs_config::ConfigProvider::new_from_str("TupleCon2\"TestStr\")");
    match TupleEnum::parse_from(&mut provider4, &mut fun) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }

    let mut provider5 = rs_config::ConfigProvider::new_from_str("TupleCon2(\"TestStr)");
    match TupleEnum::parse_from(&mut provider5, &mut fun) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true),
    }
}
#[test]
fn test_tuple_enum_format() {
    assert!(TupleEnum::get_format_str() == "TupleEnum: TupleCon1(String) | TupleCon2(String)\nString: \"Rust String\"");
    assert!(TupleEnum::get_name() == "TupleEnum");
}
