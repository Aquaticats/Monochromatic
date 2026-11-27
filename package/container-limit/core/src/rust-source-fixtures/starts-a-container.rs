//! Starts a container,
//! and holds the characters a reader that does not know Rust gets wrong.
//!
//! This file is a fixture of `rust-sources-in-the-scan.unit.test.ts`:
//! the answer the scan must give for it is one start,
//! whose verb is `run`.
//! It carries no limit,
//! because it is no ledger entry,
//! and no test expects a bound of it.

/// A note whose text holds the opening of a comment,
/// which the reader must keep inside the string it sits in:
/// the note holds one more `"` than the string closes on,
/// so a reader that does not know a raw string blanks the rest of the file from the `/*` here,
/// and loses the start below.
pub const NOTE: &str = r#"a "quote, and /* never a comment"#;

/// One character of the note,
/// which is a char literal and not the start of a string.
pub const QUOTE: char = '"';

/// The note itself,
/// borrowed for as long as the caller holds it.
pub fn note_of(note: &str) -> &str {
    return note;
}

/// Starts the container the scan must report,
/// with the verb as the argument array's first element.
pub fn start(image: &str) -> std::process::Output {
    return Command::new("podman")
        .args([
            "run",
            "--rm",
            image,
        ])
        .output()
        .expect("podman runs");
}
