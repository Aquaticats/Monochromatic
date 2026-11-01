//! What:
//!  Native literal/path-glob selection controls.
//! Why:
//!  Existing filenames win over glob punctuation,
//!  and unmatched patterns retain their requested failure policy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Populate an owned tree and compare the exact deduplicated selected paths.
//! ```

use crate::file_discovery::DiscoveryOptions;
/// Import actual path expansion and traversal configuration.
use crate::path_inputs::collect_inputs;
/// Import exclusively owned temporary directories.
use crate::test_fs::Fixture;
/// Import native path storage.
use std::path::PathBuf;

/// Construct traversal settings scoped to this fixture.
fn options(fixture: &Fixture) -> DiscoveryOptions {
    return DiscoveryOptions {
        cwd: fixture.path.clone(),
        no_ignore: true,
        ignore_patterns: Vec::<String>::new(),
        ignore_paths: Vec::<PathBuf>::new(),
    };
}

/// Literal/glob overlap is deduplicated and './' does not change a pattern's meaning.
#[test]
fn relative_absolute_and_literal_inputs_share_one_native_result_set() {
    let fixture: Fixture = Fixture::new();
    let directory: PathBuf = fixture.path.join("src");
    std::fs::create_dir(&directory).expect("source directory");
    let rust: PathBuf = directory.join("one.rs");
    let markdown: PathBuf = directory.join("two.md");
    std::fs::write(&rust, "").expect("Rust source");
    std::fs::write(&markdown, "").expect("Markdown source");
    let opts: DiscoveryOptions = options(&fixture);
    let first: Vec<PathBuf> = collect_inputs(
        &[
            PathBuf::from("./src/*.rs"),
            rust.clone(),
            fixture.path.join("src/**/*.rs"),
        ],
        &opts,
        false,
    )
    .expect("combined selection");
    assert_eq!(first.as_slice(), std::slice::from_ref(&rust));
    let mut expected: Vec<PathBuf> = vec![rust, markdown];
    expected.sort();
    assert_eq!(
        collect_inputs(&[], &opts, false).expect("default directory"),
        expected
    );
}

/// Existing glob-looking file names are treated literally before pattern expansion.
#[cfg(unix)]
#[test]
fn glob_punctuation_does_not_reinterpret_existing_files() {
    let fixture: Fixture = Fixture::new();
    let literal: PathBuf = fixture.path.join("[literal].rs");
    std::fs::write(&literal, "").expect("literal path");
    assert_eq!(
        collect_inputs(&[PathBuf::from("[literal].rs")], &options(&fixture), false)
            .expect("literal selection"),
        [literal]
    );
}

/// A literal or glob input below a regular file names nothing,
///  like a missing path:
///  the operating system answers
/// "not a directory" for it,
///  and that answer is an unmatched input,
///  not an inspection failure.
#[test]
fn inputs_below_a_regular_file_are_unmatched_not_unreadable() {
    let fixture: Fixture = Fixture::new();
    std::fs::write(fixture.path.join("a.md"), "").expect("regular file");
    let opts: DiscoveryOptions = options(&fixture);
    for input in ["a.md/x.md", "a.md/sub/*.md"] {
        assert!(
            collect_inputs(&[PathBuf::from(input)], &opts, true)
                .expect("an input below a file is allowed to match nothing")
                .is_empty(),
            "{input}"
        );
        let refused: String = collect_inputs(&[PathBuf::from(input)], &opts, false)
            .expect_err("an unmatched input is refused without the allowance")
            .message;
        assert_eq!(
            refused,
            format!(
                "Lint input {input} matched no supported source files. Correct the path/pattern or use --no-error-on-unmatched-pattern."
            )
        );
    }
}

/// Invalid patterns differ from valid patterns that matched nothing.
#[test]
fn unmatched_policy_does_not_hide_invalid_patterns() {
    let fixture: Fixture = Fixture::new();
    let opts: DiscoveryOptions = options(&fixture);
    assert!(collect_inputs(&[PathBuf::from("missing/**/*.rs")], &opts, false).is_err());
    assert!(
        collect_inputs(&[PathBuf::from("missing/**/*.rs")], &opts, true)
            .expect("allowed no matches")
            .is_empty()
    );
    assert!(
        collect_inputs(&[PathBuf::from("missing.rs")], &opts, true)
            .expect("allowed missing literal")
            .is_empty()
    );
    assert!(collect_inputs(&[PathBuf::from("[")], &opts, true).is_err());
}
