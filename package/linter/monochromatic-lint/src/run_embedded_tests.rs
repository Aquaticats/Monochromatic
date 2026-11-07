//! What:
//!  Controls for the in-process entry's own decisions:
//!  how setup failures read and which statuses they carry.
//! Why:
//!  The equality with the executable's output is shown by the differential target;
//!  these controls cover the failures that need no executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => fixSource(request)).toThrow({ exitStatus: 2 });
//! ```

use super::{
    EMBEDDED_CONFIGURATION_LABEL, EmbeddedFailure, EmbeddedRequest, EmbeddedRun, fix_source,
};
use crate::test_fs::Fixture;
use std::path::Path;

/// A document selecting Markdown files with a rule that needs no repository.
const SELECTING: &str =
    "[{\"files\":[\"**/*.md\"],\"rules\":{\"markdown/lfs-image-url\":{\"severity\":\"warn\"}}}]";

/// The status of every setup failure,
///  as the executable ends such a run.
const SETUP_STATUS: u8 = 2;

/// Fix `source` as `name` under `configuration` from `top_level`.
fn run(
    top_level: &Path,
    configuration: &str,
    name: &str,
    source: &[u8],
) -> Result<EmbeddedRun, EmbeddedFailure> {
    return fix_source(&EmbeddedRequest {
        top_level,
        configuration,
        name: Path::new(name),
        source,
    });
}

/// A source with nothing to fix comes back unchanged and without findings.
#[test]
fn a_clean_source_comes_back_unchanged() {
    let fixture: Fixture = Fixture::new();
    let source: &[u8] = b"# Title\n\nText.\n";
    let outcome: EmbeddedRun =
        run(fixture.path.as_path(), SELECTING, "a.md", source).expect("usable");
    assert_eq!(outcome.fixed.as_bytes(), source);
    assert!(outcome.findings.is_empty());
}

/// A working directory that is not absolute is a setup failure naming the requirement.
#[test]
fn a_relative_top_level_is_refused() {
    let failure: EmbeddedFailure =
        run(Path::new("relative/top"), SELECTING, "a.md", b"a\n").expect_err("refused");
    assert_eq!(failure.exit_status, SETUP_STATUS);
    assert!(
        failure.explanation.contains("absolute"),
        "{}",
        failure.explanation
    );
}

/// A configuration that does not parse is a setup failure that names the embedded configuration.
#[test]
fn a_configuration_that_does_not_parse_is_refused_by_label() {
    let fixture: Fixture = Fixture::new();
    let failure: EmbeddedFailure =
        run(fixture.path.as_path(), "[{", "a.md", b"a\n").expect_err("refused");
    assert_eq!(failure.exit_status, SETUP_STATUS);
    assert!(
        failure
            .explanation
            .starts_with(format!("Configuration {EMBEDDED_CONFIGURATION_LABEL}:").as_str()),
        "{}",
        failure.explanation
    );
}

/// A name with no language and a source that is not text are setup failures with the executable's words.
#[test]
fn a_name_without_a_language_and_a_source_that_is_not_text_are_refused() {
    let fixture: Fixture = Fixture::new();
    let unnamed: EmbeddedFailure =
        run(fixture.path.as_path(), SELECTING, "a.txt", b"a\n").expect_err("refused");
    assert_eq!(unnamed.exit_status, SETUP_STATUS);
    assert!(
        unnamed.explanation.contains("must end in .rs, .md or .mdx"),
        "{}",
        unnamed.explanation
    );
    let valid_prefix: &[u8] = b"ab";
    let mut bytes: Vec<u8> = valid_prefix.to_vec();
    bytes.push(0xff);
    let binary: EmbeddedFailure =
        run(fixture.path.as_path(), SELECTING, "a.md", bytes.as_slice()).expect_err("refused");
    assert_eq!(binary.exit_status, SETUP_STATUS);
    assert!(
        binary
            .explanation
            .contains(format!("offset {}", valid_prefix.len()).as_str()),
        "{}",
        binary.explanation
    );
}
