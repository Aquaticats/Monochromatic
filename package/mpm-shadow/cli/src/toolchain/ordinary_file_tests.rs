//! Unit tests of reading a file this tool did not just create:
//! a file another command keeps growing ends the read at the bound, and the refusal names the bound.

/// Imports `fs` from `std`.
use std::fs;
/// Imports `Write` from `std::io`,
/// for the writer that keeps the file growing.
use std::io::Write;
/// Imports what this module uses from `std::path`.
use std::path::PathBuf;
/// Imports the atomics the writer stops after its last append.
use std::sync::atomic::{AtomicUsize, Ordering};
/// Imports `Arc` from `std::sync`.
use std::sync::Arc;

/// The bound the control reads with, small enough that the file need not reach the shipped one.
const SMALL_BOUND: u64 = 32 * 1024;

/// How many appends the writer makes, so a reader past the bound still ends.
const APPENDS: usize = 8;

/// The bytes each append adds, larger than one read block so the writer stays ahead of the reader.
const APPEND_BYTES: usize = 8 * 1024;

#[test]
fn a_file_that_grows_between_two_reads_is_refused_at_the_bound() {
    let directory = crate::test_scratch::scratch();
    let path: PathBuf = directory.path().join("growing");
    fs::write(&path, vec![b'x'; APPEND_BYTES * 2]).expect("write the start of the file");
    let growing: PathBuf = path.clone();
    let appends: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let appends_in_hook: Arc<AtomicUsize> = Arc::clone(&appends);
    // The writer appends between two blocks of the read, which is how a file grows under a reader.
    super::ordinary_file::seam::set_after_a_block(Some(Box::new(move |_path| {
        if appends_in_hook.fetch_add(1, Ordering::SeqCst) >= APPENDS {
            return;
        }
        let mut file: fs::File = fs::OpenOptions::new().append(true).open(&growing).expect("keep the file open for appending");
        file.write_all(&vec![b' '; APPEND_BYTES]).expect("append between two reads");
    })));

    let read = super::ordinary_file::read_ordinary_at_most(&path, SMALL_BOUND);

    super::ordinary_file::seam::set_after_a_block(None);
    let refusal: String = read.expect_err("a file that keeps growing must be refused at the bound");
    assert!(refusal.contains(&format!("larger than the {SMALL_BOUND} bytes")), "{refusal}");
}
