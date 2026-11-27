//! What:
//! Reads one table of the crates.io database dump:
//! the wanted columns of a header,
//! a check that every row has the header's width,
//! and a handler that receives the wanted fields of each row.
//! Why:
//! `dump.rs` streams the archive and writes the histories it finds;
//! the table reading was split out of it to keep each file under the code-line limit.
//! The handler is a value the reader calls,
//! never a closure that captures state,
//! so a handler can be a struct that holds what the reader fills.
//! The items are re-exported from `dump`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // dump-table-reading.ts
//! function readTable(entry: Readable, columns: string[], onRow: TableRow): void  // throws DumpFormatError
//! type TableRow = (fields: string[]) => void                                     // throws DumpFormatError
//! ```

/// Imports `std::collections::{BTreeMap, BTreeSet}` for this file.
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// Imports `std::io::Read` for this file.
use std::io::Read;

/// Imports the CSV reader, the column check and the format error for this file.
use crate::csv::{
    DumpFormatError,
    column_indexes,
    read_csv,
};
/// Imports the dump timestamp conversion for this file.
use crate::dump::{
    MAX_VERSIONS_PER_CRATE,
    dump_timestamp,
};
/// Imports the version record that a versions row builds for this file.
use crate::policy::VersionRecord;
/// Reads one table of the archive into rows of the wanted columns;
/// the callback gets the row's wanted fields in order.
pub(crate) fn read_table(
    entry: &mut dyn Read,
    columns: &[&str],
    on_row: &mut dyn TableRow,
) -> Result<(), DumpFormatError> {
    let mut reader: TableReader = TableReader {
        columns,
        on_row,
        positions: None,
        failure: None,
        header_width: 0,
        row_number: 0,
    };
    let outcome = read_csv(entry, &mut reader);
    if let Some(error) = reader.failure {
        return Err(error);
    }
    return match outcome {
        Ok(found) => Ok(found),
        Err(error) => Err(DumpFormatError {
            message: format!("reading the crates.io dump failed: {error}"),
        }),
    };
}

/// What:
/// `trait TableRow` is what `read_table` hands the wanted fields of each row to.
/// Why:
/// The reader takes the handler as a value instead of a closure,
/// so a handler can be a struct that holds its own state.
/// Every `FnMut(&[String]) -> Result<(), DumpFormatError>` is a `TableRow` too,
/// so a test passes a closure.
pub(crate) trait TableRow {
    /// What:
    /// `row(&mut self, fields)` takes the wanted fields of one row,
    /// in the order of the wanted columns.
    /// Why:
    /// An error stops the read and is returned by it.
    fn row(
        &mut self,
        fields: &[String],
    ) -> Result<(), DumpFormatError>;
}

/// What:
/// The implementation of `TableRow` for every function with the handler's shape.
/// Why:
/// Callers that hold a closure need no wrapper.
impl<F: FnMut(&[String]) -> Result<(), DumpFormatError>> TableRow for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the handler.
    fn row(
        &mut self,
        fields: &[String],
    ) -> Result<(), DumpFormatError> {
        return self(fields);
    }
}

/// What:
/// `struct TableReader<'a>` is the row handler `read_table` gives to `read_csv`:
/// it finds the wanted columns in the header,
/// checks every row's width,
/// and passes the wanted fields on.
/// Why:
/// It holds the state that the closure that stood here captured.
struct TableReader<'a> {
    /// The names of the wanted columns.
    columns: &'a [&'a str],
    /// The handler of the wanted fields of each row.
    on_row: &'a mut dyn TableRow,
    /// The positions of the wanted columns,
    /// known once the header was read.
    positions: Option<Vec<usize>>,
    /// The first failure,
    /// which stops the read.
    failure: Option<DumpFormatError>,
    /// The header's field count.
    header_width: usize,
    /// The number of the row being read (the header is row 1).
    row_number: usize,
}

/// What:
/// The implementation of `RowHandler` for `TableReader`.
/// Why:
/// `read_csv` hands each finished row to it.
impl crate::csv::RowHandler for TableReader<'_> {
    /// What:
    /// Reads the header on the first row and checks and passes on every later row.
    /// Why:
    /// `false` stops the read after the first failure.
    fn on_row(
        &mut self,
        row: &[String],
    ) -> bool {
        self.row_number += 1;
        match &self.positions {
            None => match column_indexes(row, self.columns) {
                Ok(found) => {
                    self.header_width = row.len();
                    self.positions = Some(found);
                }
                Err(error) => {
                    self.failure = Some(error);
                    return false;
                }
            },
            Some(found) => {
                if row.len() != self.header_width {
                    self.failure = Some(DumpFormatError {
                        message: format!(
                            "row {} of a crates.io dump table has {} fields, but its header has {}",
                            self.row_number,
                            row.len(),
                            self.header_width
                        ),
                    });
                    return false;
                }
                let mut picked: Vec<String> = Vec::new();
                for index in found {
                    picked.push(row.get(*index).cloned().unwrap_or_default());
                }
                if let Err(error) = self.on_row.row(&picked) {
                    self.failure = Some(error);
                    return false;
                }
            }
        }
        return true;
    }
}

/// What:
/// `struct CrateNameRows<'a>` is the handler of the `crates.csv` rows:
/// it keeps the id of each wanted crate name.
/// Why:
/// It holds what the closure that stood here captured.
pub(crate) struct CrateNameRows<'a> {
    /// The crate names that are wanted,
    /// read by `dump` when it builds this handler.
    pub(crate) names: &'a BTreeSet<String>,
    /// The crate id of each wanted name,
    /// filled by the rows,
    /// and handed in by `dump` when it builds this handler.
    pub(crate) id_to_name: &'a mut BTreeMap<String, String>,
}

/// What:
/// The implementation of `TableRow` for `CrateNameRows`.
/// Why:
/// `read_table` hands each row's id and name to it.
impl TableRow for CrateNameRows<'_> {
    /// What:
    /// Keeps the id of the row's crate when its name is wanted.
    /// Why:
    /// The versions table is read against these ids.
    fn row(
        &mut self,
        fields: &[String],
    ) -> Result<(), DumpFormatError> {
        if self.names.contains(&fields[1]) {
            self.id_to_name.insert(fields[0].clone(), fields[1].clone());
        }
        return Ok(());
    }
}

/// What:
/// `struct VersionRows<'a>` is the handler of the `versions.csv` rows:
/// it appends each release of a wanted crate to that crate's history.
/// Why:
/// It holds what the closure that stood here captured.
pub(crate) struct VersionRows<'a> {
    /// The crate name of each wanted crate id,
    /// handed in by `dump` when it builds this handler.
    pub(crate) id_to_name: &'a BTreeMap<String, String>,
    /// The history of each wanted crate,
    /// filled by the rows,
    /// and handed in by `dump` when it builds this handler.
    pub(crate) histories: &'a mut BTreeMap<String, Vec<VersionRecord>>,
}

/// What:
/// The implementation of `TableRow` for `VersionRows`.
/// Why:
/// `read_table` hands each row's wanted fields to it.
impl TableRow for VersionRows<'_> {
    /// What:
    /// Appends the release to the history of its crate when the crate is wanted,
    /// and refuses a crate with more releases than the bound.
    /// Why:
    /// The retained history of one crate is bounded.
    fn row(
        &mut self,
        fields: &[String],
    ) -> Result<(), DumpFormatError> {
        let name: &String = match self.id_to_name.get(&fields[0]) {
            Some(found) => found,
            None => return Ok(()),
        };
        let kept: usize = self.histories.get(name).map(Vec::len).unwrap_or(0);
        if kept >= MAX_VERSIONS_PER_CRATE {
            return Err(DumpFormatError {
                message: format!(
                    "the crates.io dump lists more than {MAX_VERSIONS_PER_CRATE} releases of {name}"
                ),
            });
        }
        self.histories
            .entry(name.clone())
            .or_default()
            .push(VersionRecord {
                num: fields[1].clone(),
                created_at: dump_timestamp(&fields[2])?,
                publisher_id: fields[3].clone(),
                trustpub_repository: String::new(),
                repository: fields[4].clone(),
            });
        return Ok(());
    }
}
