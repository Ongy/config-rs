use std;
use ParseError;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

//#[derive(Debug)]
/// The main struct that will provide the config lines.
///
/// This skips whitespaces and comment lines (starting with '#')
pub struct ConfigProvider {
    file: String,
    line: usize,
    column: usize,
    line_str: String,
    line_it: Box<std::iter::Iterator<Item=(usize, String)>>,
    child: Option<Box<ConfigProvider>>,
}

impl ConfigProvider {
    /// Get the next string. This will be from the current offset to the end of line.
    /// This does not do any token sanitize! Handle with starts_with over equality comparison.
    pub fn get_next(&self) -> Option<String> {
        if self.column >= self.line_str.len() {
            return None;
        }

        if let Some(ref child) = self.child {
            return child.get_next();
        }

        /* TODO: Avoid the copy here */
        unsafe {
            let slice = self.line_str.slice_unchecked(self.column, self.line_str.len());
            return Some(String::from(slice));
        }
    }

    /// Get the next char of the config
    pub fn peek_char(&self) -> Option<char> {
        self.line_str.chars().nth(self.column)
    }


    /// Print an error with current file, line and offset
    /// # Arguments
    /// * `index`: The offset from the current internal offset (equal to offset in string gotten by
    /// get_next()
    /// * `fun`: The error reporting function
    pub fn print_error<F>(&self, index: usize, fun: &mut F)
        where F: FnMut(String) {
        if let Some(ref child) = self.child {
            child.print_error(index, fun);
            fun("Included from: ".to_string());
        }
        fun(format!("Encountered error in {}:{},{}", self.file, self.line, self.column + index + 1));
    }

    /// This will be true if there's no more config to read
    pub fn is_at_end(&self) -> bool {
        return self.column == self.line_str.len();
    }

    pub fn new_from_str<S: Into<String>>(line: S) -> ConfigProvider {
        return ConfigProvider::new_with_provider(Some((0, line.into())).into_iter(), "memory".to_string());
    }

    /// Skip the current line. E.g. when a comment, or only whitespace left
    fn skip_current(&mut self) -> Result<(), String> {
        self.column = self.line_str.len();
        return self.get_next_line();
    }

    /// Skip the upcomming list of whitespaces. This will be called by every consume
    fn skip_whitespace(&mut self) -> Result<(), String> {
        unsafe {
            let pos = self.line_str.slice_unchecked(self.column, self.line_str.len()).find(|c: char| !c.is_whitespace());

            match pos {
                Some(0) => {},
                Some(x) => {
                    self.consume(x, &mut |_| {}).unwrap();
                },
                None => {
                    self.skip_current()?;
                },
            }
        }

        return Ok(());
    }

    /// Handle a special line. Marked by starting with !
    fn handle_special(&mut self) -> Result<(), String> {
        /* This is guaranteed to return Some, or handle_special wouldn't be called */
        let line = self.get_next().unwrap();

        if line.starts_with("!include ") {
            match line.split(' ').nth(1) {
                Some(x) => {
                    self.child = Some(Box::new(provider_from_file(x)));
                    return Ok(());
                },
                None => {
                    return Err("Found !include, but couldn't figure out which file to include".to_string());
                },
            }
        }

        return Err(format!("Failed while parsing config. Found special line: {}, which I can't handle.", line));
    }

    /// Read the next line from 
    fn get_next_line(&mut self) -> Result<(), String> {
        if let Some(ref mut child) = self.child {
            child.get_next_line()?;
            if !child.is_at_end() {
                return Ok(());
            }
        }
        /* This should be part of the previous if, but the borrow checker doesn't allow that (yet)
         * since child is borrowed there
         */
        self.child = None;

        if let Some(line) = self.line_it.next() {
            self.line_str = line.1;
            self.line = line.0;
            self.column = 0;

            self.skip_whitespace()?;
            match self.peek_char() {
                Some('#') => {
                    self.skip_current()?;
                },
                Some('!') => {
                    self.handle_special()?;
                },
                _ => {},
            }
        }

        return Ok(());
    }

    /// Get a ConfigProvider form a line iterator enumerator.
    /// # Arguments
    /// * `it`: The line iterator
    /// * `file`: The file name (should be a global path)
    pub fn new_with_provider<J>(it: J, file: String) -> Self
        where J: std::iter::Iterator<Item=(usize, String)> + 'static {
        let mut ret = ConfigProvider { file: file,
            line: 1, column: 0,
            line_str: String::new(),
            line_it: Box::new(it),
            child: None,
        };

        //TODO: Bother to change this into propagation
        ret.get_next_line().unwrap();

        return ret;
    }

    /// Consume a single character if it's the upcoming char, otherwise return error
    /// # Arguments
    /// * `c`: The character to skip
    /// * `fun`: The error reporting function
    pub fn consume_char<F>(&mut self, c: char, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String){
        if self.line_str.chars().nth(self.column) == Some(c) {
            self.consume(1, fun)?;
            return Ok(());
        }

        self.print_error(0, fun);
        fun(format!("Tried to consume {}", c));
        return Err(ParseError::Final);
    }

    /// Consume a number of characters.
    ///
    /// This should be called whenever a token was recognized, as it updates the internal state and
    /// therefor error reporting and get_next().
    /// # Arguments
    /// * `count`: The number of characters to consume
    /// * `fun`: The error reporting function
    pub fn consume<F>(&mut self, count: usize, fun: &mut F) -> Result<(), ParseError>
        where F: FnMut(String) {
        if self.column + count > self.line_str.len() {
            self.print_error(0, fun);
            fun(format!("Tried to consume more than currently available: {}", count));
            return Err(ParseError::Final);
        }

        self.column = self.column + count;

        match self.skip_whitespace() {
            Err(x) => {
                self.print_error(0, fun);
                fun(x);
                return Err(ParseError::Final);
            },
            Ok(()) => {},
        }

        if self.column == self.line_str.len() {
            self.get_next_line()?;
        }

        return Ok(());
    }
}

/// Get a provider from a single file.
/// This is used to build the nesting providers
pub fn provider_from_file<P: AsRef<Path>>(path: P) -> ConfigProvider {
    let p = path.as_ref();
    let f = File::open(p).unwrap();

    let lines = BufReader::new(f).lines().map(|x| x.unwrap()).enumerate();

    let path_str = p.to_str().unwrap_or_else(|| "ERROR");

    return ConfigProvider::new_with_provider(lines, path_str.into());
}

/// Get a provider for a single file, and wrap it in {}, so the final config doesn't have to be in
/// an initial {} wrapper.
pub fn provider_from_file_wrap<P: AsRef<Path>>(path: P) -> ConfigProvider {
    let p = path.as_ref();
    let f = File::open(p).unwrap();

    let open = std::iter::once((0, "{".into()));
    let lines = BufReader::new(f).lines().map(|x| x.unwrap()).enumerate();
    let close = std::iter::once((usize::max_value(), "}".into()));

    let fin = open.chain(lines).chain(close);

    let path_str = p.to_str().unwrap_or_else(|| "ERROR");

    return ConfigProvider::new_with_provider(fin, path_str.into());
}


#[cfg(test)]
mod test {
    use ConfigProvider;

    #[test]
    fn test_config_provider_string() {
        let mut provider = ConfigProvider::new_from_str("This is a line");

        assert!(provider.get_next() == Some("This is a line".to_string()));

        provider.consume(4, &mut |_| {}).unwrap();
        assert!(provider.get_next() == Some("is a line".to_string()));

        provider.consume(9, &mut |_| {}).unwrap();
        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_config_provider_nextline() {
        let lines = vec![(1, "Line1   \n".to_string()), (2, "  Line2".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("Line1   \n".to_string()));
        provider.consume(5, &mut |_| {}).unwrap();

        assert!(provider.get_next() == Some("Line2".to_string()));

        provider.consume(5, &mut |_| {}).unwrap();
        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_config_provider_err_str() {
        let lines = vec![(1, "Line1   \n".to_string()), (2, "  Line2".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());
        let mut val = String::new();

        {
            let mut fun = |x| val = x;
            provider.print_error(2, &mut fun);
        }
        assert!(val == "Encountered error in Testfile:1,3");

        provider.consume(5, &mut |_| {}).unwrap();

        {
            let mut fun = |x| val = x;
            provider.print_error(1, &mut fun);
        }
        assert!(val == "Encountered error in Testfile:2,4");
    }

    #[test]
    fn test_provider_skips_whiteline() {
        let lines = vec![(1, "  \t".to_string()), (2, "  ".to_string()), (3, "  line".to_string()), (4, "  \t\t".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("line".to_string()));
        provider.consume(4, &mut |_| {}).unwrap();

        assert!(provider.get_next() == None);
    }

    #[test]
    fn test_provider_skips_comment() {
        let lines = vec![(1, "#lin1".to_string()), (2, "   #line2".to_string()), (3, "line#3   #another".to_string()), (4, "#line4".to_string())];
        let mut provider = ConfigProvider::new_with_provider(lines.into_iter(), "Testfile".to_string());

        assert!(provider.get_next() == Some("line#3   #another".to_string()));
        provider.consume(6, &mut |_| {}).unwrap();

        assert!(provider.get_next() == Some("#another".to_string()));
        provider.consume(8, &mut |_| {}).unwrap();

        assert!(provider.get_next() == None);
    }
}
