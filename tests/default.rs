#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;


#[derive(ConfigAble, PartialEq, Eq)]
struct DefaultTest1(Option<String>, Option<String>);

#[derive(ConfigAble, PartialEq, Eq)]
struct DefaultTest2(Option<String>, String);

#[test]
fn test_default_struct() {
    assert!(DefaultTest1::get_default() == Ok(DefaultTest1(None, None)));
    assert!(DefaultTest2::get_default() == Err(()));
}
