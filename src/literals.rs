use std;


    // TODO: Bother with the license
// Taken from: https://github.com/serde-rs/syntex/blob/eb4e68ab9a89c030cf20b7d5c949eb6a4fdc8890/syntex_syntax/src/parse/mod.rs
/// Parse a string representing a character literal into its final form.
/// Rather than just accepting/rejecting a given literal, unescapes it as
/// well. Can take any slice prefixed by a character escape. Returns the
/// character and the number of characters consumed.
fn char_lit(lit: &str) -> (char, usize) {
    use std::char;

    // Handle non-escaped chars first.
    if lit.as_bytes()[0] != b'\\' {
        // If the first byte isn't '\\' it might part of a multi-byte char, so
        // get the char with chars().
        let c = lit.chars().next().unwrap();
        return (c, 1);
    }

    // Handle escaped chars.
    match lit.as_bytes()[1] as char {
        '"' => ('"', 2),
        'n' => ('\n', 2),
        'r' => ('\r', 2),
        't' => ('\t', 2),
        '\\' => ('\\', 2),
        '\'' => ('\'', 2),
        '0' => ('\0', 2),
        'x' => {
            let v = u32::from_str_radix(&lit[2..4], 16).unwrap();
            let c = char::from_u32(v).unwrap();
            (c, 4)
        }
        'u' => {
            assert_eq!(lit.as_bytes()[2], b'{');
            let idx = lit.find('}').unwrap();
            let v = u32::from_str_radix(&lit[3..idx], 16).unwrap();
            let c = char::from_u32(v).unwrap();
            (c, (idx + 1) as usize)
        }
        _ => panic!("lexer should have rejected a bad character escape {}", lit)
    }
}

pub fn parse_char(lit: &str) -> Result<(usize, char), String> {
    if lit.as_bytes()[0] != b'\'' {
        return Err("Expected \"'\" at beginning of char literal".to_string());
    }

    let (ret, size) = char_lit(&lit[1..]);

    if lit.as_bytes()[1 + size] != b'\'' {
        return Err("Expected \"'\" at end of char literal".to_string());
    }

    return Ok((size + 2, ret));
}

/// Parse a string representing a string literal into its final form. Does
/// unescaping.
pub fn str_lit(lit: &str) -> Result<(usize, String), String> {
    //debug!("parse_str_lit: given {}", escape_default(lit));
    let mut res = String::with_capacity(lit.len());
    let mut count = None;

    // FIXME #8372: This could be a for-loop if it didn't borrow the iterator
    let error = |i| format!("lexer should have rejected {} at {}", lit, i);

    /// Eat everything up to a non-whitespace
    fn eat<'a>(it: &mut std::iter::Peekable<std::str::CharIndices<'a>>) {
        loop {
            match it.peek().map(|x| x.1) {
                Some(' ') | Some('\n') | Some('\r') | Some('\t') => {
                    it.next();
                },
                _ => { break; }
            }
        }
    }

    let mut chars = lit.char_indices().peekable();

    match chars.next() {
        Some((_, '"')) => {},
        Some((_, c)) => return Err(format!("Expected '\"' at beginning of string literal. Found '{}'", c)),
        None => return Err("Tried to parse string literal, got empty string".to_string()),
    }

    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                let ch = chars.peek().unwrap_or_else(|| {
                    panic!("{}", error(i))
                }).1;

                if ch == '\n' {
                    eat(&mut chars);
                } else if ch == '\r' {
                    chars.next();
                    let ch = chars.peek().unwrap_or_else(|| {
                        panic!("{}", error(i))
                    }).1;

                    if ch != '\n' {
                        panic!("lexer accepted bare CR");
                    }
                    eat(&mut chars);
                } else {
                    // otherwise, a normal escape
                    let (c, n) = char_lit(&lit[i..]);
                    for _ in 0..n - 1 { // we don't need to move past the first \
                        chars.next();
                    }
                    res.push(c);
                }
            },
            '\r' => {
                let ch = chars.peek().unwrap_or_else(|| {
                    panic!("{}", error(i))
                }).1;

                if ch != '\n' {
                    panic!("lexer accepted bare CR");
                }
                chars.next();
                res.push('\n');
            },
            '"' => {
                /* We found the closing '"' */
                count = Some(i as usize);
                break;
            }
            c => res.push(c),
        }
    }

    match count {
        Some(i) => {
            res.shrink_to_fit(); // probably not going to do anything, unless there was an escape.
            //debug!("parse_str_lit: returning {}", res);
            return Ok((i + 1, res));
        },
        None => return Err("Didn't find string closing '\"' for string literal".to_string()),
    }
}
