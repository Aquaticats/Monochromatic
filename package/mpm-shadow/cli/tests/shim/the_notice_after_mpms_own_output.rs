//! What:
//! The part of the shim tests that watches what the shadow adds after mpm's own output:
//! one line after a top-level `--version`,
//! a short section after a top-level `--help`,
//! and not one byte more for a subcommand's help,
//! the `help` subcommand,
//! another output format,
//! or a failed run.
//! Why:
//! The owner's rule is that a person looking for the version or the help learns the shadow is there,
//! and every other output stays byte for byte mpm's own:
//! a single added byte in a piped run would break the caller that parsed mpm's output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shim/the_notice_after_mpms_own_output.test.ts
//! test('a top-level version gets one added line', () => { ... });
//! ```

use std::process::Output;

use crate::{
    fixture_root,
    start_shadow,
    write_fake_mpm,
};

/// The fake mpm of the notice tests:
/// it answers the way mpm 8.2.0 does for a top-level `--version` and `--help`,
/// for a subcommand's own help,
/// and for another output format.
const FAKE_MPM_WITH_VERSION_AND_HELP: &str = "#!/bin/sh\n\
case \"$1\" in\n\
  --version) printf 'mpm, version 8.2.0\\nPython 3.13\\n' ;;\n\
  --help|-h) printf 'Usage: mpm [OPTIONS] COMMAND [ARGS]...\\n\\nOther commands:\\n  help\\n' ;;\n\
  installed) printf 'Usage: mpm installed [OPTIONS]\\n' ;;\n\
  --help-format) printf '{\"help\": true}\\n' ;;\n\
  *) printf 'other\\n' ;;\n\
esac\n\
exit 0\n";

/// The real mpm's text is byte for byte first,
/// then exactly one added line.
#[test]
fn a_top_level_version_gets_one_added_line_after_mpms_own_output() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), FAKE_MPM_WITH_VERSION_AND_HELP);
    let real: String = root.path().join("real").join("mpm").display().to_string();

    let output: Output = start_shadow(root.path(), &["--version"]);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "mpm, version 8.2.0\nPython 3.13\nmpm-shadow {}, shadowing mpm at {real}\n",
            env!("CARGO_PKG_VERSION")
        )
    );
    assert_eq!(output.status.code(), Some(0));
}

/// Options before the subcommand are skipped the way mpm's parser skips them.
#[test]
fn a_version_after_other_top_level_options_still_gets_the_line() {
    let root = fixture_root();
    write_fake_mpm(
        &root.path().join("real"),
        "#!/bin/sh\nprintf 'mpm, version 8.2.0\\n'\n",
    );

    let output: Output = start_shadow(
        root.path(),
        &["--verbosity", "DEBUG", "--no-color", "--version"],
    );

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        text.starts_with("mpm, version 8.2.0\nmpm-shadow "),
        "{text}"
    );
    assert_eq!(text.lines().count(), 2, "{text}");
}

/// After mpm's help comes a blank line and a short section naming the shadow.
#[test]
fn a_top_level_help_gets_a_section_after_mpms_own_help() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), FAKE_MPM_WITH_VERSION_AND_HELP);
    let real: String = root.path().join("real").join("mpm").display().to_string();

    for flag in ["--help", "-h"] {
        let output: Output = start_shadow(root.path(), &[flag]);
        let text = String::from_utf8_lossy(&output.stdout).to_string();
        let own = "Usage: mpm [OPTIONS] COMMAND [ARGS]...\n\nOther commands:\n  help\n";
        assert!(
            text.starts_with(own),
            "{flag}: mpm's help comes first and unchanged:\n{text}"
        );
        let added = &text[own.len()..];
        assert!(
            added.starts_with(&format!("\nmpm-shadow {}:\n", env!("CARGO_PKG_VERSION"))),
            "{flag}: {added}"
        );
        assert!(
            added.contains(&real)
                && added.contains("monochromatic-")
                && added.contains("deleted stale"),
            "{flag}: {added}"
        );
        assert_eq!(output.status.code(), Some(0));
    }
}

/// The help of a subcommand only the real mpm has (`installed`),
/// the `help` subcommand,
/// a line that starts with mpm's own option,
/// and another output format print exactly what mpm printed.
/// (`mpm install --help` is the shadow's own `install` help:
/// `install` is pnpm's verb.)
#[test]
fn non_top_level_help_and_other_outputs_pass_through_untouched() {
    let root = fixture_root();
    write_fake_mpm(&root.path().join("real"), FAKE_MPM_WITH_VERSION_AND_HELP);

    let cases: [(&[&str], &str); 4] = [
        (&["installed", "--help"], "Usage: mpm installed [OPTIONS]\n"),
        (&["--verbose", "install", "--help"], "other\n"),
        (&["help", "install"], "other\n"),
        (&["--help-format", "json", "--help"], "{\"help\": true}\n"),
    ];
    for (arguments, expected) in cases {
        let output: Output = start_shadow(root.path(), arguments);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            expected,
            "{arguments:?}"
        );
    }
}

/// A failing `--version` keeps mpm's exit code and gets no added line.
#[test]
fn the_exit_code_of_mpm_passes_through_and_a_failed_version_gets_no_line() {
    let root = fixture_root();
    write_fake_mpm(
        &root.path().join("real"),
        "#!/bin/sh\nprintf 'broken\\n'\nexit 7\n",
    );

    let output: Output = start_shadow(root.path(), &["--version"]);

    assert_eq!(String::from_utf8_lossy(&output.stdout), "broken\n");
    assert_eq!(output.status.code(), Some(7));
}
