//! What:
//!  The place one holder scan's result is read at:
//!  the lines,
//!  how many are read,
//!  the line whose key is the one expected,
//!  and the canonical decimal count a value carries.
//! Why:
//!  Every field of a result is read in the order the writer wrote it,
//!  so one cursor owns the position,
//!  refuses a line that is not the key it expected,
//!  and refuses a count larger than the lines beneath it;
//!  it lives apart from the whole result so each file says one thing
//!  (`crate::clone_holder_scan::holder_scan_parse`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cursor = { lines, at: 0 }; const value = keyed(cursor, 'dataset');
//! ```

/// Why a line that is not the field the reader expected refuses the whole result.
use crate::clone_holder_scan::holder_scan_refusals::HolderScanRefusal;
/// One text read line by line,
/// with the place the next line is expected at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cursor = { lines, at: 0 }; const value = keyed(cursor, 'dataset');
/// ```
pub(crate) struct Cursor<'a> {
    /// The text's lines.
    lines: &'a [&'a str],
    /// How many lines have been read.
    at: usize,
}

impl<'a> Cursor<'a> {
    /// What:
    ///  A cursor whose next line is the first one after the result's version line.
    /// Why:
    ///   The version line is read before the cursor exists,
    ///  so the caller hands the lines over and the cursor starts below that line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const cursor = { lines, at: 1 };
    /// ```
    pub(crate) fn after_the_version_line(lines: &'a [&'a str]) -> Cursor<'a> {
        return Cursor {
            lines,
            at: 1,
        };
    }

    /// The value of the line whose key is `key`,
    /// refusing a line that is not there or begins another key.
    pub(crate) fn keyed(
        &mut self,
        key: &'static str,
    ) -> Result<&'a str, HolderScanRefusal> {
        let Some(line) = self.lines.get(self.at).copied() else {
            return Err(HolderScanRefusal::MissingLine {
                key,
            });
        };
        self.at += 1;
        let Some((found, value)) = line.split_once(' ') else {
            return Err(HolderScanRefusal::Key {
                number: self.at,
                expected: key,
                found: String::from(line),
            });
        };
        if found != key {
            return Err(HolderScanRefusal::Key {
                number: self.at,
                expected: key,
                found: String::from(found),
            });
        }
        if value.is_empty() {
            return Err(HolderScanRefusal::Value {
                key,
                shown: String::from("<empty>"),
            });
        }
        return Ok(value);
    }

    /// One canonical decimal count of the lines that follow it,
    /// refused before any line is read under it when more are claimed than are
    /// there.
    pub(crate) fn section_count(
        &mut self,
        key: &'static str,
    ) -> Result<usize, HolderScanRefusal> {
        let value: &'a str = self.keyed(key)?;
        let count: usize = decimal_count(key, value)?;
        let remaining: usize = self.lines.len().saturating_sub(self.at);
        if count > remaining {
            return Err(HolderScanRefusal::Count {
                key,
                shown: String::from(value),
            });
        }
        return Ok(count);
    }

    /// One canonical decimal count of something other than the lines below it,
    /// which is what the namespaces read are.
    pub(crate) fn bare_count(
        &mut self,
        key: &'static str,
    ) -> Result<usize, HolderScanRefusal> {
        let value: &'a str = self.keyed(key)?;
        return decimal_count(key, value);
    }

    /// Nothing may follow the last field.
    pub(crate) fn finish(self) -> Result<(), HolderScanRefusal> {
        let Some(line) = self.lines.get(self.at).copied() else {
            return Ok(());
        };
        return Err(HolderScanRefusal::TrailingLine {
            found: String::from(line),
        });
    }
}

/// One canonical decimal count,
/// which is one or more digits with no sign and no leading zero.
pub(crate) fn decimal_count(
    key: &'static str,
    value: &str,
) -> Result<usize, HolderScanRefusal> {
    let canonical: bool = !value.is_empty()
        && value.bytes().all(|byte| return byte.is_ascii_digit())
        && !(value.len() > 1 && value.starts_with('0'));
    let Ok(count) = value.parse::<usize>() else {
        return Err(HolderScanRefusal::Count {
            key,
            shown: String::from(value),
        });
    };
    if !canonical {
        return Err(HolderScanRefusal::Count {
            key,
            shown: String::from(value),
        });
    }
    return Ok(count);
}
