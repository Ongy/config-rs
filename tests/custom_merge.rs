#[macro_use]
extern crate rs_config_derive;

extern crate rs_config;

use rs_config::ConfigAble;


#[derive(ConfigAble, PartialEq, Eq)]
#[ConfigAttrs(merge="merge_fun")]
struct MergeTest(Option<String>, Option<String>);

impl MergeTest {
    fn merge_fun(&mut self, rhs: Self) -> Result<(), ()> {
        match (&self, &rhs) {
            (&&mut MergeTest(None, None), _) => *self = rhs,
            (_, &MergeTest(None, None)) => {},
            _ => {return Err(());},
        }

        return Ok(());
    }
}

#[test]
fn test_custom_merge1() {
    let mut left = MergeTest(Some("Test".into()), None);
    let right = MergeTest(None, Some("String".into()));

    assert!(left.merge(right) == Err(()));
    assert!(left == MergeTest(Some("Test".into()), None));
}

#[test]
fn test_custom_merge2() {
    let mut left = MergeTest(None, None);
    let right = MergeTest(Some("Test".into()), Some("String".into()));
    assert!(left.merge(right) == Ok(()));
    assert!(left == MergeTest(Some("Test".into()), Some("String".into())));
}
