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
    position: Position,
    spawn: String,
}

#[cfg(not(test))]
fn main() {
    let mut set = HashSet::new();

    print!("{}", Config::get_format(&mut set));

    let lines = vec![(1, "{".to_string()), (2, "position: Global(Top)".to_string()), (3, "spawn: \"monky\"".to_string()), (4, "}".to_string())];
    let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "ExampleString".to_string());

    print!("{:?}\n", Config::parse_from(&mut provider, &mut |x| println!("{}", x)));

    let lines2 = vec![(1, "{".to_string()), (2, "position: Global(Top), position: Global(Bottom)".to_string()), (3, "spawn: \"monky\", spawn: \"monky\"".to_string()), (4, "}".to_string())];
    let mut provider2 = ConfigProvider::new_with_provider(lines2.into_iter(), "ExampleString".to_string());

    print!("{:?}\n", Config::parse_from(&mut provider2, &mut |x| println!("{}", x)));
}

```
This currently produces the output:
```
$ cargo run
    Compiling config-test v0.1.0 (file:///[..]/config-test)
    Finished dev [unoptimized + debuginfo] target(s) in 0.89 secs
    Running `target/debug/config-test`
Config: {position: Position, spawn: String}
String: "Rust String"
Position: Global(Direction) | Monitor(String, Direction)
Direction: Left | Right | Top | Bottom
Ok(Config { position: Global(Top), spawn: "monky" })
Tried to parse something twice. Gonna fail, this isn't implemented yet
Tried to parse something twice. Gonna fail, this isn't implemented yet
Err(Recoverable)
```

## Disclaimer:
I'm bad/new at rust, so some things are probably horrible, while others will be
reworked soon-ish


### I know ConfigProvider copies a lot, that's what's going to change for sure.
