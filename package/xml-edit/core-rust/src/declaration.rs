//! What:
//!  Scanning and checking of the optional XML declaration, `<?xml version="1.0" encoding="UTF-8"?>`.
//! Why:
//!  The editor reads and writes UTF-8 `&str` text under XML 1.0, so a declaration naming another version or
//!  encoding is refused with a diagnostic rather than silently mis-decoded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scanDeclaration(scanner: Scanner): Span;
//! ```

use crate::document::XmlSpan;
use crate::error::XmlParseError;
use crate::scan::Scanner;

impl Scanner<'_> {
    /// What:
    ///  True when the unread text opens an XML declaration: `<?xml` followed by whitespace.
    /// Why:
    ///  `<?xml-stylesheet ...?>` is an ordinary processing instruction and must not be mistaken for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// atDeclaration(): boolean;
    /// ```
    pub fn at_declaration(&self) -> bool {
        if !self.starts_with("<?xml") {
            return false;
        }
        return match self.peek(5) {
            Some(byte) => byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b'\n',
            None => false,
        };
    }

    /// What: Read one `name="value"` pseudo-attribute and return the name and value texts.
    /// Why: Declaration parts look like attributes but follow a fixed order and plain-text values.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanPseudoAttribute(): [string, string];
    /// ```
    fn scan_pseudo_attribute(&mut self) -> Result<(String, String), XmlParseError> {
        let name: XmlSpan = self.scan_name("a declaration field name")?;
        self.skip_spaces();
        if self.peek(0) != Some(b'=') {
            return Err(self.fail(self.offset, "expected = after the declaration field name"));
        }
        self.offset += 1;
        self.skip_spaces();
        let quote_byte: u8 = self.peek(0).unwrap_or(0);
        if quote_byte != b'"' && quote_byte != b'\'' {
            return Err(self.fail(self.offset, "a declaration value must start with a quote"));
        }
        let value_start: usize = self.offset + 1;
        let Some(length) = self.source[value_start..].find(char::from(quote_byte)) else {
            return Err(self.fail(self.offset, "this declaration value is never closed with its quote"));
        };
        let value: String = self.source[value_start..value_start + length].to_string();
        self.offset = value_start + length + 1;
        return Ok((self.source[name.start..name.end].to_string(), value));
    }

    /// What:
    ///  Scan the XML declaration at the cursor and return its span.
    /// Why:
    ///  Only `version="1.0"`, an optional UTF-8 `encoding` and an optional `standalone="yes|no"` are in scope,
    ///  in that order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scanDeclaration(): Span;
    /// ```
    pub fn scan_declaration(&mut self) -> Result<XmlSpan, XmlParseError> {
        let start: usize = self.offset;
        self.offset += 5;
        // Rank of the last field read: 0 none, 1 version, 2 encoding, 3 standalone.
        let mut rank: u8 = 0;
        loop {
            let gap: usize = self.skip_spaces();
            if self.starts_with("?>") {
                break;
            }
            if gap == 0 {
                return Err(self.fail(self.offset, "declaration fields must be separated by whitespace"));
            }
            let field_offset: usize = self.offset;
            let (name, value): (String, String) = self.scan_pseudo_attribute()?;
            if name == "version" && rank == 0 {
                if value != "1.0" {
                    let message: String = format!("XML version {value:?} is unsupported: only 1.0 is in scope");
                    return Err(self.fail(field_offset, &message));
                }
                rank = 1;
            } else if name == "encoding" && rank == 1 {
                if !value.eq_ignore_ascii_case("utf-8") {
                    let message: String = format!("encoding {value:?} is unsupported: the editor reads UTF-8 text");
                    return Err(self.fail(field_offset, &message));
                }
                rank = 2;
            } else if name == "standalone" && rank < 3 && rank > 0 {
                if value != "yes" && value != "no" {
                    return Err(self.fail(field_offset, "standalone must be \"yes\" or \"no\""));
                }
                rank = 3;
            } else {
                let message: String = format!("unexpected or out-of-order declaration field {name:?}");
                return Err(self.fail(field_offset, &message));
            }
        }
        if rank == 0 {
            return Err(self.fail(start, "an XML declaration must start with version=\"1.0\""));
        }
        self.offset += 2;
        return Ok(XmlSpan { start, end: self.offset });
    }
}
