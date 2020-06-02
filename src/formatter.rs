use std::fmt;
use std::io;
use std::iter;

/// Facade for writing formatted strings to io::Write types.
pub struct IoFmt<'write, W: 'write>(pub &'write mut W);

impl<'write, W> fmt::Write for IoFmt<'write, W>
where
    W: io::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

/// Formatter implementation for write types.
pub struct Formatter<'write> {
    write: &'write mut dyn fmt::Write,
    /// if last line was empty.
    current_line_empty: bool,
    /// Current indentation level.
    indent: usize,
    /// Number of indentations per level.
    pub(crate) indentation: usize,
    /// Holds the current indentation level as a string.
    buffer: String,
}

impl<'write> Formatter<'write> {
    /// Create a new write formatter.
    pub fn new(write: &mut dyn fmt::Write) -> Formatter {
        Formatter {
            write: write,
            current_line_empty: true,
            indent: 0usize,
            indentation: 2usize,
            buffer: String::from("  "),
        }
    }

    /// Configure the indentation associated with the formatter.
    pub fn with_indentation(self, indentation: usize) -> Self {
        Self {
            indentation,
            ..self
        }
    }

    fn check_indent(&mut self) -> fmt::Result {
        if self.current_line_empty && self.indent > 0 {
            self.write
                .write_str(&self.buffer[0..(self.indent * self.indentation)])?;
            self.current_line_empty = false;
        }

        Ok(())
    }

    /// Write the given string.
    pub fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > 0 {
            self.check_indent()?;
            self.write.write_str(s)?;
            self.current_line_empty = false;
        }

        Ok(())
    }

    /// Push a new line.
    pub fn new_line(&mut self) -> fmt::Result {
        self.write.write_char('\n')?;
        self.current_line_empty = true;
        Ok(())
    }

    /// Push a new line, unless the current line is empty.
    pub fn new_line_unless_empty(&mut self) -> fmt::Result {
        if !self.current_line_empty {
            self.new_line()?;
        }

        Ok(())
    }

    /// Increase indentation level.
    pub fn indent(&mut self) {
        self.indent += 1;

        let extra = (self.indent * self.indentation).saturating_sub(self.buffer.len());

        // check that buffer contains the current indentation.
        for c in iter::repeat(' ').take(extra) {
            self.buffer.push(c);
        }
    }

    /// Decrease indentation level.
    pub fn unindent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }
}

impl<'write> fmt::Write for Formatter<'write> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > 0 {
            self.check_indent()?;
            self.write.write_str(s)?;
            self.current_line_empty = false;
        }

        Ok(())
    }
}
