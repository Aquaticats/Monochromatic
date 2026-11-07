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
/// The chunk size bounds the read buffer,
/// not the field being built,
/// so without this a quoted field that never ends
/// would grow until the process is killed.
/// A chosen ceiling far above any metadata field of the crates.io tables read here (a guess,
/// not measured on a real dump).
pub const MAX_FIELD_BYTES: usize = 16 * 1024 * 1024;

/// The most bytes one row may hold across all its fields.
/// Each field may be as large as `MAX_FIELD_BYTES` and a row may hold `MAX_ROW_FIELDS` fields,
/// whose product is far beyond any machine,
/// so the row has its own cumulative budget:
/// a chosen ceiling of two maximal fields (a guess,
/// not measured on a real dump).
pub const MAX_ROW_BYTES: usize = 2 * MAX_FIELD_BYTES;

/// The most fields one row may hold.
/// A chosen ceiling far above the column count of the crates.io tables read here (a guess,
/// not measured on a real dump).
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
    /// Field `row_bytes: usize` is the total size of the finished fields of the row being read.
    /// It is counted on the decoded text,
    /// which lossy decoding can make larger than the bytes read.
    row_bytes: usize,
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
    fn end_field(&mut self) -> io::Result<()> {
        // A row wider than the bound is refused instead of grown, so one hostile row cannot exhaust memory.
        if self.row.len() >= MAX_ROW_FIELDS {
            // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
            // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // Ok(value)  is  return value;
            // // Err(error) is  throw error;   (the caller sees it as a returned value)
            // ```
            // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
            // Why:      Text building and printing are written as one call instead of string concatenation.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // `..${x}..`   // template literal
            // console.error(text);   // eprintln!
            // [a, b]            // vec![a, b]
            // ```
            return Err(malformed(format!("a row has more than {MAX_ROW_FIELDS} fields")));
        }
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let text: String = String::from_utf8_lossy(&self.field).to_string();
        self.row_bytes += text.len();
        if self.row_bytes > MAX_ROW_BYTES {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(malformed(format!("a row is larger than {MAX_ROW_BYTES} bytes")));
        }
        self.row.push(text);
        self.field.clear();
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }

    /// Hands the finished row to the callback;
    /// `false` from the callback stops the read.
    fn end_row(&mut self, on_row: &mut dyn RowHandler) -> io::Result<bool> {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        self.end_field()?;
        let keep_going: bool = on_row.on_row(&self.row);
        self.row.clear();
        self.row_bytes = 0;
        self.row_started = false;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(keep_going);
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
    fn push_byte(&mut self, byte: u8) -> io::Result<()> {
        // A field longer than the bound is refused instead of grown, so a quote that never closes cannot exhaust memory.
        if self.field.len() >= MAX_FIELD_BYTES {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(malformed(format!("a field is longer than {MAX_FIELD_BYTES} bytes")));
        }
        // The field being built counts against the row budget too, so the row is refused before it is held in memory.
        if self.row_bytes + self.field.len() >= MAX_ROW_BYTES {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(malformed(format!("a row is larger than {MAX_ROW_BYTES} bytes")));
        }
        self.field.push(byte);
        self.row_started = true;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }
}

/// What:
/// `fn malformed(message) -> io::Error` builds the error of a stream that is not well-formed CSV.
/// Why:
/// A damaged table must stop the read with a named reason instead of yielding rows that look complete;
/// the kind `InvalidData` says the bytes were read fine but are wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const malformed = (message: string) => new Error(`malformed CSV: ${message}`);
/// ```
fn malformed(message: String) -> io::Error {
    // Macro `format!`: builds or prints text (or a list) from a template.
    return io::Error::new(io::ErrorKind::InvalidData, format!("malformed CSV: {message}"));
}

/// What:
/// `pub trait RowHandler` is what `read_csv` hands each finished row to:
/// `on_row(row)` returns `false` to stop the read.
/// Why:
/// The reader takes the handler as a value instead of a closure,
/// so a handler can be a struct that holds its own state.
/// Every `FnMut(&[String]) -> bool` is a `RowHandler` too,
/// so a test passes a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RowHandler { onRow(row: string[]): boolean; }
/// ```
pub trait RowHandler {
    /// What:
    /// `on_row(&mut self, row)` takes one finished row.
    /// Why:
    /// `false` stops the read.
    fn on_row(&mut self, row: &[String]) -> bool;
}

/// What:
/// The implementation of `RowHandler` for every function with the handler's shape.
/// Why:
/// Tests and callers that already hold a closure need no wrapper.
impl<F: FnMut(&[String]) -> bool> RowHandler for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the handler.
    fn on_row(&mut self, row: &[String]) -> bool {
        return self(row);
    }
}

/// What:
/// `pub fn read_csv(reader: &mut dyn Read, on_row: &mut dyn RowHandler) -> io::Result<()>` reads rows until the
/// stream ends or the callback returns `false`.
/// Why:
/// A byte state machine:
/// inside quotes a doubled quote is one quote and a single quote ends the quoting;
/// outside quotes a comma
/// ends a field,
/// a line feed ends a row,
/// and a carriage return is dropped.
/// A stream that ends inside quotes,
/// text after a closing quote other than a comma or a line break,
/// a quote inside a bare field and a field or row over the bound are errors,
/// so a cut-off or hostile table never passes for a finished one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readCsv(stream, onRow: (row: string[]) => boolean): void
/// ```
pub fn read_csv(reader: &mut dyn Read, on_row: &mut dyn RowHandler) -> io::Result<()> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `state: State`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut state: State = State { field: Vec::new(), row: Vec::new(), in_quotes: false, quote_seen: false, row_started: false, row_bytes: 0 };
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
                        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                        state.push_byte(QUOTE)?;
                        continue;
                    }
                    state.in_quotes = false;
                    // Only a field or row separator may follow a closing quote.
                    if byte != COMMA && byte != LINE_FEED && byte != CARRIAGE_RETURN {
                        // `Err(...)`: builds the success or the failure variant of a `Result`.
                        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
                        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // String(value)  // JS strings are always owned values
                        // ```
                        return Err(malformed("text follows a closing quote".to_string()));
                    }
                } else if byte == QUOTE {
                    state.quote_seen = true;
                    continue;
                } else {
                    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                    state.push_byte(byte)?;
                    continue;
                }
            }
            if byte == QUOTE && state.field.is_empty() {
                state.in_quotes = true;
                state.row_started = true;
            } else if byte == QUOTE {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                return Err(malformed("a quote appears inside a bare field".to_string()));
            } else if byte == COMMA {
                state.row_started = true;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                state.end_field()?;
            } else if byte == LINE_FEED {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                if !state.end_row(on_row)? {
                    return Ok(());
                }
            } else if byte != CARRIAGE_RETURN {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                state.push_byte(byte)?;
            }
        }
    }
    if state.in_quotes && state.quote_seen {
        state.in_quotes = false;
    }
    // A stream that ends inside an unclosed quote was cut off.
    if state.in_quotes {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err(malformed("the stream ends inside a quoted field".to_string()));
    }
    if state.row_started || !state.field.is_empty() || !state.row.is_empty() {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        state.end_row(on_row)?;
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
