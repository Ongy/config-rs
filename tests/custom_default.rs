#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;


#[derive(ConfigAble, PartialEq, Eq)]
#[ConfigAttrs(default="DefaultTest::Default")]
enum DefaultTest{
    Default,
    Other
}



#[test]
fn test_custom_default() {
    assert!(DefaultTest::get_default() == Ok(DefaultTest::Default));
}
