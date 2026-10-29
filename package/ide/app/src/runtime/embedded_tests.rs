//! The embedded table:
//!  sorted lookups,
//!  digest checks,
//!  and the messages for damage and absence.

use super::{EmbeddedFile, EmbeddedRuntime};
use crate::content_digest::fnv1a;

/// An intact table,
///  sorted by path as the build script writes it.
static INTACT: [EmbeddedFile; 3] = [
    EmbeddedFile {
        path: "runtime/grammars/sql.so",
        bytes: b"parser bytes",
        digest: fnv1a(b"parser bytes"),
    },
    EmbeddedFile {
        path: "runtime/manifest.json",
        bytes: b"{\"grammars\": [\"sql.so\"]}",
        digest: fnv1a(b"{\"grammars\": [\"sql.so\"]}"),
    },
    EmbeddedFile {
        path: "runtime/queries/sql/highlights.scm",
        bytes: b"(keyword) @keyword",
        digest: fnv1a(b"(keyword) @keyword"),
    },
];

/// One byte of the parser differs from what the recorded digest describes.
static DAMAGED: [EmbeddedFile; 1] = [EmbeddedFile {
    path: "runtime/grammars/sql.so",
    bytes: b"parser bytEs",
    digest: fnv1a(b"parser bytes"),
}];

static INTACT_RUNTIME: EmbeddedRuntime = EmbeddedRuntime {
    key: "0123456789abcdef",
    files: &INTACT,
};

static DAMAGED_RUNTIME: EmbeddedRuntime = EmbeddedRuntime {
    key: "0123456789abcdef",
    files: &DAMAGED,
};

#[test]
fn lookups_find_every_file_and_nothing_else() {
    for file in &INTACT {
        let found = INTACT_RUNTIME.find(file.path).expect("listed file");
        assert_eq!(found.bytes, file.bytes);
    }
    assert!(INTACT_RUNTIME.find("runtime/grammars/rust.so").is_none());
    assert!(INTACT_RUNTIME.find("").is_none());
    assert_eq!(
        INTACT_RUNTIME
            .verified("runtime/queries/sql/locals.scm")
            .expect("an absent optional file is not an error"),
        None
    );
}

#[test]
fn a_damaged_file_is_refused_with_its_name_and_the_remedy() {
    let error = DAMAGED_RUNTIME
        .verified("runtime/grammars/sql.so")
        .expect_err("damaged bytes were accepted");
    let message = format!("{error:#}");
    assert!(message.contains("runtime/grammars/sql.so"), "{message}");
    assert!(message.contains("is damaged"), "{message}");
    assert!(message.contains("Replace the executable"), "{message}");
    assert!(DAMAGED_RUNTIME.required("runtime/grammars/sql.so").is_err());
}

#[test]
fn a_required_file_missing_from_the_table_is_reported() {
    let error = INTACT_RUNTIME
        .required("runtime/grammars/rust.so")
        .expect_err("a missing required file was accepted");
    let message = format!("{error:#}");
    assert!(
        message.contains("lacks the language file runtime/grammars/rust.so"),
        "{message}"
    );
    assert!(
        message.contains("Replace it with a fresh copy"),
        "{message}"
    );
    assert_eq!(
        INTACT_RUNTIME
            .required("runtime/grammars/sql.so")
            .expect("intact"),
        b"parser bytes"
    );
}
