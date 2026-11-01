//! What:
//!  Controls of the prerequisite manifest grammar.
//! Why:
//!  The manifest selects prerequisites from a fixed list,
//! so every accepted shape and every refusal is pinned,
//!  including names that would be dangerous if they reached a process argument.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test.each(cases)('manifest %s', (source, expected) => expect(parsePrerequisites(source)).toEqual(expected));
//! ```

use super::{KINDS, PREREQUISITES_FILE, Prerequisite, parse_prerequisites, read_prerequisites};
use crate::test_fs::Fixture;

/// Every kind in one manifest,
/// with comments and trailing commas as JSONC allows.
#[test]
fn accepts_every_kind_in_order() {
    let found: Vec<Prerequisite> = parse_prerequisites(
        r#"{
          // system libraries
          "pkg-config": ["gtk4", "fontconfig"],
          "executables": ["zig"],
          "qmake": true,
          "ide-runtime-artifacts": true,
          "android-ndk": true,
        }"#,
    )
    .expect("a valid manifest");
    assert_eq!(
        found,
        vec![
            Prerequisite::PkgConfig(vec![String::from("gtk4"), String::from("fontconfig")]),
            Prerequisite::Executables(vec![String::from("zig")]),
            Prerequisite::Qmake,
            Prerequisite::IdeRuntimeArtifacts,
            Prerequisite::AndroidNdk,
        ]
    );
}

/// An empty object names no prerequisite.
#[test]
fn accepts_an_empty_object() {
    assert_eq!(parse_prerequisites("{}"), Ok(Vec::<Prerequisite>::new()));
}

/// An unknown kind is refused and the message lists the fixed kinds.
#[test]
fn refuses_an_unknown_prerequisite_and_lists_the_kinds() {
    let message: String =
        parse_prerequisites(r#"{"shell": ["rm -rf /"]}"#).expect_err("unknown kind");
    assert!(message.contains("\"shell\""), "{message}");
    for kind in KINDS {
        assert!(message.contains(kind), "{message} lacks {kind}");
    }
}

/// A command line is not a name:
/// separators,
/// spaces,
/// a leading dash and the empty string are refused.
#[test]
fn refuses_names_that_could_act_as_options_or_paths() {
    for bad in [
        "--version",
        "-x",
        "a b",
        "a/b",
        "../tool",
        "tool;ls",
        "tool\nls",
        "",
        "$(id)",
        "tool\u{0}",
    ] {
        let quoted: String = serde_json::to_string(bad).expect("a string is JSON");
        for kind in ["executables", "pkg-config"] {
            let source: String = format!("{{\"{kind}\": [{quoted}]}}");
            assert!(
                parse_prerequisites(source.as_str()).is_err(),
                "{bad:?} was accepted for {kind}"
            );
        }
    }
}

/// Dotted,
/// versioned and plus-suffixed names that real packages use are accepted.
#[test]
fn accepts_real_module_names() {
    assert!(
        parse_prerequisites(
            r#"{"pkg-config": ["gtk4", "libadwaita-1", "gtk+-3.0", "Qt6Core", "x_y"]}"#
        )
        .is_ok()
    );
}

/// A list kind needs at least one name and strings only.
#[test]
fn refuses_empty_and_non_string_lists() {
    assert!(parse_prerequisites(r#"{"executables": []}"#).is_err());
    assert!(parse_prerequisites(r#"{"executables": [1]}"#).is_err());
    assert!(parse_prerequisites(r#"{"pkg-config": "gtk4"}"#).is_err());
}

/// A flag kind takes only the literal `true`.
#[test]
fn refuses_other_values_for_flag_kinds() {
    for kind in ["qmake", "ide-runtime-artifacts", "android-ndk"] {
        for value in ["false", "1", "\"yes\"", "[]", "{}"] {
            let source: String = format!("{{\"{kind}\": {value}}}");
            assert!(parse_prerequisites(source.as_str()).is_err(), "{source}");
        }
    }
}

/// Duplicate keys,
/// nulls,
/// arrays at the root and broken JSONC are refused with a message.
#[test]
fn refuses_ambiguous_and_malformed_documents() {
    assert!(parse_prerequisites(r#"{"qmake": true, "qmake": true}"#).is_err());
    assert!(parse_prerequisites(r#"{"qmake": null}"#).is_err());
    assert!(parse_prerequisites("[]").is_err());
    assert!(parse_prerequisites("{").is_err());
    assert!(parse_prerequisites("").is_err());
}

/// A package without the file has no prerequisites.
#[test]
fn a_missing_file_means_no_prerequisites() {
    let fixture: Fixture = Fixture::new();
    assert_eq!(
        read_prerequisites(&fixture.path),
        Ok(Vec::<Prerequisite>::new())
    );
}

/// The file beside the manifest is read,
/// including from a directory whose name has a space.
#[test]
fn reads_the_file_in_a_directory_with_a_space() {
    let fixture: Fixture = Fixture::new();
    let directory = fixture.path.join("my package");
    std::fs::create_dir_all(&directory).expect("directory");
    std::fs::write(directory.join(PREREQUISITES_FILE), r#"{"qmake": true}"#).expect("manifest");
    assert_eq!(
        read_prerequisites(&directory),
        Ok(vec![Prerequisite::Qmake])
    );
}

/// A bad manifest names the file in its message.
#[test]
fn an_invalid_file_names_itself() {
    let fixture: Fixture = Fixture::new();
    std::fs::write(fixture.path.join(PREREQUISITES_FILE), r#"{"nope": true}"#).expect("manifest");
    let message: String = read_prerequisites(&fixture.path).expect_err("unknown kind");
    assert!(message.starts_with(PREREQUISITES_FILE), "{message}");
}

/// A path that exists but cannot be read as a file is an error,
/// not an empty manifest.
#[test]
fn an_unreadable_file_is_an_error_not_an_empty_manifest() {
    let fixture: Fixture = Fixture::new();
    std::fs::create_dir(fixture.path.join(PREREQUISITES_FILE)).expect("a directory in its place");
    assert!(read_prerequisites(&fixture.path).is_err());
}
