//! What:
//!  `.lfsconfig` line-grammar and file-read controls.
//! Why:
//!  The declarations found,
//!  and their order,
//!  are frozen by
//! `package/cli/markdown-lint/src/lfs-config.unit.test.ts`;
//!  the added cases cover the trim set,
//! case folding and read failures the incumbent handled implicitly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe(parseLfsConfig.name, () => { /* sections, keys, comments, order */ });
//! ```

/// Import the scanner,
///  the file reader and disposable directories.
use super::{js_trim, lfs_endpoints, read_lfs_object_base, read_optional_text};
use crate::test_fs::Fixture;

/// `lfs.url` and `remote.<name>.lfsurl` are read;
///  comments,
///  blanks and other sections are not.
#[test]
fn endpoints_come_from_lfs_and_remote_sections_in_file_order() {
    assert_eq!(
        lfs_endpoints("[lfs]\n\turl = https://lfs.example\n"),
        ["https://lfs.example"]
    );
    assert_eq!(
        lfs_endpoints("[remote \"origin\"]\n\tlfsurl = https://lfs.example/x\n"),
        ["https://lfs.example/x"]
    );
    assert!(
        lfs_endpoints(
            "# c\n; d\n\n[core]\n\turl = https://nope.example\n[lfs]\n\tconcurrenttransfers = 3\n"
        )
        .is_empty()
    );
    assert_eq!(
        lfs_endpoints(
            "[lfs]\n\turl = https://a.example\n[remote \"o\"]\n\tlfsurl = https://b.example\n"
        ),
        ["https://a.example", "https://b.example"]
    );
}

/// A commented-out declaration declares nothing,
///  with either comment character,
///  with or without a
/// space after it,
///  and under both sections that could otherwise name an endpoint.
#[test]
fn commented_out_declarations_declare_nothing() {
    for source in [
        "[lfs]\n# url = https://x.example\n",
        "[lfs]\n#url = https://x.example\n",
        "[lfs]\n; url = https://x.example\n",
        "[lfs]\n\t;url=https://x.example\n",
        "[remote \"o\"]\n# lfsurl = https://x.example\n",
        "[remote \"o\"]\n;lfsurl = https://x.example\n",
        // A commented-out section header does not open that section.
        "#[lfs]\nurl = https://x.example\n",
        ";[lfs]\nurl = https://x.example\n",
    ] {
        assert!(lfs_endpoints(source).is_empty(), "{source}");
    }
    // The positive control: the same declaration without its comment character is read,
    // and a comment between a header and its key does not close the section.
    assert_eq!(
        lfs_endpoints("[lfs]\n# note = 1\n\n; other\nurl = https://x.example\n"),
        ["https://x.example"]
    );
}

/// Section and key names fold case;
///  values keep their spelling,
///  including later `=` characters.
#[test]
fn names_fold_case_and_values_keep_their_bytes() {
    assert_eq!(
        lfs_endpoints("[ LFS ]\r\n\tURL = https://Lfs.Example/?a=b\r\n"),
        ["https://Lfs.Example/?a=b"]
    );
    assert_eq!(
        lfs_endpoints("[Remote \"Origin\"]\n LfsUrl=https://b.example\n"),
        ["https://b.example"]
    );
    // `url` under a remote section and `lfsurl` under `[lfs]` are different keys.
    assert!(lfs_endpoints("[remote \"o\"]\nurl = https://x.example\n").is_empty());
    assert!(lfs_endpoints("[lfs]\nlfsurl = https://x.example\n").is_empty());
    // `remotes` is not a remote section; the section name must start with `remote` and a space.
    assert!(lfs_endpoints("[remotes]\nlfsurl = https://x.example\n").is_empty());
}

/// Keys before any section,
///  lines without `=`,
///  and empty values declare nothing.
#[test]
fn incomplete_lines_declare_nothing() {
    assert!(lfs_endpoints("url = https://x.example\n").is_empty());
    assert!(lfs_endpoints("[lfs]\nurl\nurl =\nurl =   \n").is_empty());
    assert!(lfs_endpoints("").is_empty());
    // A section header resets the previous section even when it is empty.
    assert!(lfs_endpoints("[lfs]\n[]\nurl = https://x.example\n").is_empty());
}

/// The ECMAScript trim set includes a byte order mark and excludes NEXT LINE.
#[test]
fn trimming_follows_the_ecmascript_set() {
    assert_eq!(js_trim("\u{feff}\t value \u{a0}\r\n"), "value");
    assert_eq!(js_trim("\u{85}value\u{85}"), "\u{85}value\u{85}");
    assert_eq!(
        lfs_endpoints("\u{feff}[lfs]\nurl = https://bom.example\n"),
        ["https://bom.example"]
    );
}

/// An absent file is ordinary,
///  a present file is read,
///  and a directory in its place is a failure.
#[test]
fn optional_reads_distinguish_absence_from_failure() {
    let fixture: Fixture = Fixture::new();
    let path: std::path::PathBuf = fixture.path.join(".lfsconfig");
    assert_eq!(read_optional_text(&path).expect("absent file"), None);
    assert_eq!(
        read_lfs_object_base(&fixture.path).expect("no configuration"),
        None
    );
    std::fs::write(&path, "[lfs]\n\tconcurrenttransfers = 3\n")
        .expect("configuration without endpoint");
    assert_eq!(
        read_optional_text(&path).expect("present file"),
        Some(String::from("[lfs]\n\tconcurrenttransfers = 3\n"))
    );
    assert_eq!(
        read_lfs_object_base(&fixture.path).expect("no declared endpoint"),
        None
    );
    std::fs::remove_file(&path).expect("remove configuration");
    std::fs::create_dir(&path).expect("directory in place of the file");
    let error = read_optional_text(&path).expect_err("directory is not readable text");
    assert!(error.message.contains(".lfsconfig"), "{}", error.message);
    // The rendered error is the stored explanation; the processing finding prints it through `{error}`.
    assert!(
        error.message.starts_with("Cannot read "),
        "{}",
        error.message
    );
    assert_eq!(error.to_string(), error.message);
    assert!(read_lfs_object_base(&fixture.path).is_err());
}
