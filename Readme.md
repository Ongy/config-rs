# config-rs

This is a very WIP config config library.

The goal is to auto-derive most of the config reading part.

The config will be read in a strict manner, enforcing correctnes while parsing.
This makes dealing with the result nice, since it's just a rust struct, without
any further deciphering required.

## Example

The example code in main.rs  that's using what's going to be api looks like:

```
#[derive(Debug, ConfigAble)]
enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, ConfigAble)]
enum Position {
    Global(Direction),
    Monitor(String, Direction),
}

#[derive(Debug, ConfigAble)]
struct Config {
    #[ConfigAttrs(default = "Position::Global(Direction::Top)")]
    position: Position,
    spawn: Option<String>,
    #[ConfigAttrs(default = "\"ongybar\".to_string()")]
    title: String,
}

#[cfg(not(test))]
fn main() {
    println!("{}", Config::get_format_str());

    println!("{:?}", Config::parse_from(&mut ConfigProvider::new_from_str("{spawn: Some(\"monky\")}"), &mut |x| println!("{}", x)));

    let lines = vec![
        (1, "{".to_string()),
        (2, "spawn: Some(\"monky\")".to_string()),
        (3, ", title: \"ongybar\"".to_string()),
        (3, ", spawn: Some(\"monky\")".to_string()),
        (4, "# a Comment".to_string()),
        (5, "}".to_string())
    ];

    let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());
    println!("{:?}", Config::parse_from(&mut provider, &mut |x| println!("{}", x)));
}
```
This currently produces the output:
```
$ cargo run
    Compiling config-test v0.1.0 (file:///[..]/config-test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.89 secs
    Running `target/debug/config-test`
Config: {position: Position, spawn: Option < String >, title: String}
String: "Rust String"
Position: Global(Direction) | Monitor(String, Direction)
Direction: Left | Right | Top | Bottom
Option<String>: Some(String) | None
Ok(Config { position: Global(Top), spawn: Some("monky"), title: "ongybar" })
Encountered error in Testfile:5,1
Tried to parse something twice. This is not supported(yet)
Err(Recoverable)
```

## Disclaimer:
I'm bad/new at rust, so some things are probably horrible, while others will be
reworked soon-ish


### I know ConfigProvider copies a lot, that's what's going to change for sure.
