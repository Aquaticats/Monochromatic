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

use std::io::{self, Read};

/// The error for a dump whose shape is not what this reader expects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DumpFormatError {
    /// What was wrong.
    pub message: String,
}

impl std::fmt::Display for DumpFormatError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{}", self.message);
    }
}

const QUOTE: u8 = b'"';
const COMMA: u8 = b',';
const LINE_FEED: u8 = b'\n';
const CARRIAGE_RETURN: u8 = b'\r';

/// How many bytes are read from the stream at a time.
const CHUNK_BYTES: usize = 64 * 1024;

/// The parser state between bytes.
struct State {
    field: Vec<u8>,
    row: Vec<String>,
    in_quotes: bool,
    quote_seen: bool,
    row_started: bool,
}

impl State {
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
        match header.iter().position(|name| { return name == column; }) {
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
