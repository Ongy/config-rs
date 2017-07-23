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
enum InputSource {
    Stdin,
    Pipe(i32),
    Named(String),
    Spawn(String),
}

#[derive(Debug, ConfigAble)]
struct Input {
    source: InputSource,
    #[ConfigAttrs(default = "0")]
    layer: i32,
}

#[derive(Debug, ConfigAble)]
struct Config {
    #[ConfigAttrs(default = "Position::Global(Direction::Top)")]
    position: Position,
    inputs: Vec<Input>,
    #[ConfigAttrs(default = "\"ongybar\".to_string()")]
    title: String,
}

#[cfg(not(test))]
fn main() {
    println!("{}", Config::get_format_str());

    println!("{:?}", Config::parse_from(&mut ConfigProvider::new_from_str("{}"), &mut |x| println!("{}", x)));

    let lines = vec![
        (1, "{".to_string()),
        (2, "  inputs: [{source: Spawn(\"monk\"), source: Spawn(\"y\")}]".to_string()),
        (3, ", title: \"ongybar\"".to_string()),
        (3, ", inputs: [{source: Stdin, source: Stdin}]".to_string()),
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
Config: {position: Position, inputs: Vec < Input >, title: String}
Vec<Input>: [ Input, Input, ... ]
Input: {source: InputSource, layer: i32}
i32: Digits
InputSource: Stdin | Pipe(i32) | Named(String) | Spawn(String)
String: "Rust String"
Position: Global(Direction) | Monitor(String, Direction)
Direction: Left | Right | Top | Bottom
Ok(Config { position: Global(Top), inputs: [], title: "ongybar" })
Ok(Config { position: Global(Top), inputs: [Input { source: Spawn("monky"), layer: 0 }, Input { source: Stdin, layer: 0 }], title: "ongybar" })
```

## Disclaimer:
I'm bad/new at rust, so some things are probably horrible, while others will be
reworked soon-ish


### I know ConfigProvider copies a lot, that's what's going to change for sure.
