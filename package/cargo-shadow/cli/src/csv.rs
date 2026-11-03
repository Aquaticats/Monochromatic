//! What:
//! A streaming reader for the CSV tables of the crates.io database dump:
//! quoted fields,
//! doubled quotes inside them,
//! and line
//! breaks inside quoted fields.
//! Why:
//! `versions.csv` is hundreds of megabytes,
//! so it is read as a stream one row at a time and never held whole;
//! the `csv` crate is
//! not in the repository's catalog and the format needs only this much.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // csv.ts
//! export function createCsvParser({ onRow }): { push(chunk: Uint8Array): void; end(): void }
//! ```

/// What:
/// `use std::io::{self, Read};` imports `self`,
/// `Read` from the standard library so this file can name them unqualified.
/// Why:
/// `self` is needed by the derive or macro expansions of this file;
/// `Read` is used by `read_csv`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { self, Read } from 'std/io';
/// ```
use std::io::{self, Read};

/// The error for a dump whose shape is not what this reader expects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DumpFormatError {
    /// What was wrong.
    pub message: String,
}

/// What:
/// `impl std::fmt::Display for DumpFormatError` lets a `DumpFormatError` be printed with `{}` and turned into text with `.to_string()`.
/// Why:
/// The dump reader returns this error to the registry,
/// which wraps its message into a `RegistryError` the user reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DumpFormatError extends Error { toString() { return this.message; } }
/// ```
impl std::fmt::Display for DumpFormatError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the error's message into the formatter that `{}` provides.
    /// Why:
    /// The message already names the column or the schema change,
    /// so printing it unchanged is the whole display.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.message; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{}", self.message);
    }
}

/// What:
/// `const QUOTE: u8 = b'"'` is the byte value of the double quote character (`u8`,
/// one byte;
/// siblings:
/// `char`,
/// a Unicode scalar,
/// and `u32`).
/// Why:
/// The reader walks the stream byte by byte and compares each byte with it to find where a quoted field starts and where it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const QUOTE = 0x22;
/// ```
const QUOTE: u8 = b'"';
/// What:
/// `const COMMA: u8 = b','` is the byte value of the comma.
/// Why:
/// Outside quotes a comma ends a field,
/// so the reader needs to recognize it without decoding the text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COMMA = 0x2c;
/// ```
const COMMA: u8 = b',';
/// What:
/// `const LINE_FEED: u8 = b'\n'` is the byte value of the line feed.
/// Why:
/// Outside quotes a line feed ends a row,
/// which is when the row is handed to the callback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LINE_FEED = 0x0a;
/// ```
const LINE_FEED: u8 = b'\n';
/// What:
/// `const CARRIAGE_RETURN: u8 = b'\r'` is the byte value of the carriage return.
/// Why:
/// The dump may use Windows line endings;
/// dropping the carriage return makes both endings read the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CARRIAGE_RETURN = 0x0d;
/// ```
const CARRIAGE_RETURN: u8 = b'\r';

/// How many bytes are read from the stream at a time.
const CHUNK_BYTES: usize = 64 * 1024;

/// The most bytes one field may hold.
/// The chunk size bounds the read buffer, not the field being built, so without this a quoted field that never ends
/// would grow until the process is killed.
/// A chosen ceiling far above any metadata field of the crates.io tables read here (a guess, not measured on a real dump).
pub const MAX_FIELD_BYTES: usize = 16 * 1024 * 1024;

/// The most fields one row may hold.
/// A chosen ceiling far above the column count of the crates.io tables read here (a guess, not measured on a real dump).
pub const MAX_ROW_FIELDS: usize = 1024;

/// The parser state between bytes.
struct State {
    /// What:
    /// Field `field: Vec<u8>` holds the bytes of the field being read so far (`Vec<u8>` is a growable byte list;
    /// sibling:
    /// `String`,
    /// which must be valid UTF-8).
    /// Why:
    /// Bytes are collected first and decoded once at the end of the field,
    /// so a multi-byte character split across two read chunks stays whole.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// field: number[]  // bytes
    /// ```
    field: Vec<u8>,
    /// What:
    /// Field `row: Vec<String>` holds the finished fields of the row being read.
    /// Why:
    /// The row is handed to the callback as one slice at the end of the line,
    /// so the callback sees complete rows only.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// row: string[]
    /// ```
    row: Vec<String>,
    /// What:
    /// Field `in_quotes: bool` says whether the reader is inside a quoted field.
    /// Why:
    /// Inside quotes commas and line feeds are data,
    /// not separators,
    /// so the byte rules differ.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// inQuotes: boolean
    /// ```
    in_quotes: bool,
    /// What:
    /// Field `quote_seen: bool` says the previous byte inside quotes was a quote.
    /// Why:
    /// Two quotes in a row are one literal quote and a single one closes the field;
    /// the reader only knows which after it has seen the next byte.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// quoteSeen: boolean
    /// ```
    quote_seen: bool,
    /// What:
    /// Field `row_started: bool` says some byte of the current row was already read.
    /// Why:
    /// A final row without a trailing line feed,
    /// or a row of empty fields,
    /// must still be delivered at the end of the stream.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowStarted: boolean
    /// ```
    row_started: bool,
}

/// What:
/// `impl State { ... }` attaches the reader's helper methods to the `State` struct (a TS class body).
/// Why:
/// The byte loop in `read_csv` stays readable because ending a field and appending a byte are named steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class State { endField() { ... } pushByte(byte) { ... } }
/// ```
impl State {
    /// What:
    /// `fn end_field(&mut self)` decodes the collected bytes into text and appends it to the row;
    /// `&mut self` borrows the state for changing.
    /// Why:
    /// A comma or the end of the row finishes a field,
    /// and `String::from_utf8_lossy` replaces invalid bytes instead of failing a whole dump.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// endField(): void { this.row.push(decode(this.field)); this.field = []; }
    /// ```
    fn end_field(&mut self) {
        self.row.push(String::from_utf8_lossy(&self.field).to_string());
        self.field.clear();
    }

    /// Hands the finished row to the callback;
    /// `false` from the callback stops the read.
    fn end_row(&mut self, on_row: &mut dyn FnMut(&[String]) -> bool) -> bool {
        self.end_field();
        let keep_going: bool = on_row(&self.row);
        self.row.clear();
        self.row_started = false;
        return keep_going;
    }

    /// What:
    /// `fn push_byte(&mut self, byte)` appends one byte to the current field and marks the row as started.
    /// Why:
    /// Every ordinary byte,
    /// quoted or not,
    /// goes through the same step,
    /// so the row-started flag cannot be forgotten.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pushByte(byte: number): void { this.field.push(byte); this.rowStarted = true; }
    /// ```
    fn push_byte(&mut self, byte: u8) {
        self.field.push(byte);
        self.row_started = true;
    }
}

/// What:
/// `pub fn read_csv(reader: &mut dyn Read, on_row: &mut dyn FnMut(&[String]) -> bool) -> io::Result<()>` reads rows until the
/// stream ends or the callback returns `false`.
/// Why:
/// A byte state machine:
/// inside quotes a doubled quote is one quote and a single quote ends the quoting;
/// outside quotes a comma
/// ends a field,
/// a line feed ends a row,
/// and a carriage return is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readCsv(stream, onRow: (row: string[]) => boolean): void
/// ```
pub fn read_csv(reader: &mut dyn Read, on_row: &mut dyn FnMut(&[String]) -> bool) -> io::Result<()> {
    let mut state: State = State { field: Vec::new(), row: Vec::new(), in_quotes: false, quote_seen: false, row_started: false };
    let mut chunk: Vec<u8> = vec![0; CHUNK_BYTES];
    loop {
        let count: usize = reader.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        for raw in &chunk[..count] {
            let byte: u8 = *raw;
            if state.in_quotes {
                if state.quote_seen {
                    state.quote_seen = false;
                    if byte == QUOTE {
                        state.push_byte(QUOTE);
                        continue;
                    }
                    state.in_quotes = false;
                } else if byte == QUOTE {
                    state.quote_seen = true;
                    continue;
                } else {
                    state.push_byte(byte);
                    continue;
                }
            }
            if byte == QUOTE && state.field.is_empty() {
                state.in_quotes = true;
                state.row_started = true;
            } else if byte == COMMA {
                state.row_started = true;
                state.end_field();
            } else if byte == LINE_FEED {
                if !state.end_row(on_row) {
                    return Ok(());
                }
            } else if byte != CARRIAGE_RETURN {
                state.push_byte(byte);
            }
        }
    }
    if state.in_quotes && state.quote_seen {
        state.in_quotes = false;
    }
    if state.row_started || !state.field.is_empty() || !state.row.is_empty() {
        state.end_row(on_row);
    }
    return Ok(());
}

/// The column positions of the wanted names in a header row,
/// or an error naming the missing column.
pub fn column_indexes(header: &[String], columns: &[&str]) -> Result<Vec<usize>, DumpFormatError> {
    let mut indexes: Vec<usize> = Vec::new();
    for column in columns {
        match crate::adapt::position_of_text(header, column) {
            Some(found) => indexes.push(found),
            None => {
                return Err(DumpFormatError {
                    message: format!(
                        "the crates.io database dump has no \"{column}\" column (header: {}); its schema changed, so update csv.rs and dump.rs in package/cargo-shadow/cli",
                        header.join(", ")
                    ),
                });
            }
        }
    }
    return Ok(indexes);
}
