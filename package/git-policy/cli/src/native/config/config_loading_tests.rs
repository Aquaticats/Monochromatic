//! What:
//!  Configuration fast-path decisions over native Git argument shapes.
//! Why:
//!  An optional presentation flag must not consume a branch/tag name and hide a mutation.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Cover listing, mutation, separator and opaque-value paths independently.
//! ```

/// Import actual classification,
///  the private mutation list and native argument ownership.
use super::{ConfigLoading, classify_config_loading, mutating_long};
use std::ffi::OsString;

/// Build one exact argv vector and classify it without side effects.
fn classify(values: &[&str]) -> ConfigLoading {
    let mut arguments: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        arguments.push(OsString::from(value));
    }
    return classify_config_loading(arguments.as_slice());
}

/// Known inspection commands and explicit list/verify forms skip policy initialization.
#[test]
fn recognized_inspection_forms_keep_the_fast_path() {
    for args in [
        vec!["status"],
        vec!["-C", "/repo", "show", "HEAD"],
        vec!["--version"],
        vec!["branch"],
        vec!["branch", "--list", "feature-*"],
        vec!["branch", "-vl", "feature-*"],
        vec!["branch", "--contains", "HEAD"],
        vec!["branch", "--contains"],
        vec!["branch", "--show-current"],
        vec!["tag"],
        vec!["tag", "--list", "v*"],
        vec!["tag", "--verify", "v1"],
        vec!["tag", "-n3", "v*"],
        vec!["branch", "--list", "--", "-D"],
        vec!["branch", "--format", "--delete"],
    ] {
        assert_eq!(classify(args.as_slice()), ConfigLoading::Skip, "{args:?}");
    }
}

/// A global option Git refuses,
///  and one whose value is missing,
///  are Git's to report:
///  each skips on its own.
#[test]
fn global_option_errors_keep_the_fast_path() {
    for args in [
        vec!["--no-such-global-option"],
        vec!["--no-such-global-option", "commit", "-m", "x"],
        vec![
            "-C",
            "/repo",
            "--no-such-global-option",
            "branch",
            "-D",
            "topic",
        ],
        vec!["-C"],
        vec!["-c"],
        vec!["--no-pager", "--git-dir"],
    ] {
        assert_eq!(classify(args.as_slice()), ConfigLoading::Skip, "{args:?}");
    }
}

/// Mutations remain configuration-requiring even when mixed with a listing flag.
#[test]
fn mutation_flags_and_positionals_never_become_false_inspection() {
    for args in [
        vec!["commit"],
        vec!["future-command"],
        vec![],
        vec!["branch", "feature"],
        vec!["branch", "-uorigin/main"],
        vec!["branch", "--delete-merged"],
        vec!["branch", "--list", "--delete", "feature"],
        vec!["branch", "--del", "feature"],
        vec!["tag", "v1"],
        vec!["tag", "-mmessage", "v1"],
        vec!["tag", "--delete=v1"],
        vec!["branch", "--color", "new-branch"],
        vec!["tag", "--column", "new-tag"],
        vec!["branch", "--color=always", "new-branch"],
        vec!["branch", "--format"],
        vec!["branch", "--unknown"],
        vec!["branch", "-x"],
    ] {
        assert_eq!(
            classify(args.as_slice()),
            ConfigLoading::Required,
            "{args:?}"
        );
    }
}

/// Run real Git only inside a newly created fixture,
///  with no global/system configuration.
fn git(directory: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    let output = std::process::Command::new("/usr/bin/git")
        .current_dir(directory)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", directory.join("absent-global-config"))
        .args(arguments)
        .output()
        .expect("native Git fixture command");
    assert!(
        output.status.success(),
        "{:?}: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// Optional --color does not consume a separated branch name in the selected native release.
#[test]
fn native_optional_color_flag_still_creates_a_named_branch() {
    let directory = crate::test_support::fixture("native-color-classification");
    git(&directory, &["init", "--initial-branch=main"]);
    git(
        &directory,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--allow-empty",
            "--message=initial",
        ],
    );
    git(&directory, &["branch", "--color", "created-with-color"]);
    git(
        &directory,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            "refs/heads/created-with-color",
        ],
    );
    assert_eq!(
        classify(&["branch", "--color", "created-with-color"]),
        ConfigLoading::Required
    );
    std::fs::remove_dir_all(directory).expect("remove only the fixture");
}

/// Required option values are consumed without interpreting their spelling as action flags.
#[test]
fn option_values_and_separators_remain_opaque() {
    assert_eq!(
        classify(&["branch", "--format=-D", "--list", "name"]),
        ConfigLoading::Skip
    );
    assert_eq!(
        classify(&["branch", "--sort", "-name"]),
        ConfigLoading::Skip
    );
    assert_eq!(
        classify(&["branch", "--", "--list"]),
        ConfigLoading::Required
    );
    assert_eq!(
        classify(&["-c", "alias.command=commit", "command"]),
        ConfigLoading::Required
    );
}

/// A commit filter lists even with a pattern;
///  `--format` and `--sort` never turn a name into a pattern.
#[test]
fn only_commit_filters_imply_listing() {
    for args in [
        vec!["branch", "--contains", "HEAD", "feature-*"],
        vec!["branch", "--merged=main", "feature-*"],
        vec![
            "branch",
            "--no-contains",
            "HEAD",
            "--sort",
            "-name",
            "feature-*",
        ],
        vec!["tag", "--points-at", "HEAD", "v*"],
        vec!["tag", "--sort=-creatordate", "--merged", "main", "v*"],
    ] {
        assert_eq!(classify(args.as_slice()), ConfigLoading::Skip, "{args:?}");
    }
    for args in [
        vec!["branch", "--sort", "-name", "new-branch"],
        vec!["branch", "--sort=-name", "new-branch"],
        vec!["branch", "--format", "%(refname)", "new-branch"],
        vec!["tag", "--format=%(refname)", "new-tag"],
        vec!["tag", "--sort", "-creatordate", "new-tag"],
        vec!["tag", "--sort"],
    ] {
        assert_eq!(
            classify(args.as_slice()),
            ConfigLoading::Required,
            "{args:?}"
        );
    }
}

/// Each short letter keeps its own command's meaning;
///  a lone dash is a name,
///  not a flag cluster.
#[test]
fn short_letters_are_judged_per_command() {
    for args in [
        vec!["tag", "-v", "v1"],
        vec!["tag", "-i"],
        vec!["tag", "-il", "V*"],
        vec!["tag", "-n", "v*"],
        vec!["tag", "-n12", "v*"],
        vec!["branch", "-vvr"],
        vec!["branch", "-qai"],
        vec!["branch", "-rl", "origin/*"],
    ] {
        assert_eq!(classify(args.as_slice()), ConfigLoading::Skip, "{args:?}");
    }
    for args in [
        // Verbose is presentation for `branch`, so the name is still created.
        vec!["branch", "-v", "new-branch"],
        vec!["branch", "-qr", "new-branch"],
        // `-n` and `-x` are not `branch` listing letters.
        vec!["branch", "-n"],
        vec!["branch", "-n3"],
        // Ignore-case is presentation for `tag`, so the name is still created.
        vec!["tag", "-i", "new-tag"],
        vec!["tag", "-x"],
        vec!["tag", "-q"],
        vec!["tag", "-r"],
        // After the line count only digits may follow.
        vec!["tag", "-n3l"],
        vec!["tag", "-n3d", "v1"],
        // A lone dash is a positional name for both commands.
        vec!["branch", "-"],
        vec!["tag", "-"],
    ] {
        assert_eq!(
            classify(args.as_slice()),
            ConfigLoading::Required,
            "{args:?}"
        );
    }
}

/// The explicit mutation list holds each command's own long forms and nothing else.
#[test]
fn mutating_long_forms_are_listed_per_command() {
    for name in [
        "--copy",
        "--delete",
        "--delete-merged",
        "--edit-description",
        "--force",
        "--move",
        "--set-upstream",
        "--set-upstream-to",
        "--unset-upstream",
        "--create-reflog",
    ] {
        assert!(mutating_long(name.as_bytes(), true), "branch {name}");
    }
    for name in [
        "--annotate",
        "--delete",
        "--edit",
        "--force",
        "--sign",
        "--local-user",
        "--message",
        "--file",
        "--trailer",
        "--create-reflog",
    ] {
        assert!(mutating_long(name.as_bytes(), false), "tag {name}");
    }
    // Listing forms, abbreviations and the other command's forms are not in either list.
    for name in ["--list", "--del", "--delete=x", "--contains", "", "--"] {
        assert!(!mutating_long(name.as_bytes(), true), "branch {name}");
        assert!(!mutating_long(name.as_bytes(), false), "tag {name}");
    }
    for name in ["--annotate", "--edit", "--sign", "--message", "--file"] {
        assert!(!mutating_long(name.as_bytes(), true), "branch {name}");
    }
    for name in ["--copy", "--move", "--set-upstream-to", "--delete-merged"] {
        assert!(!mutating_long(name.as_bytes(), false), "tag {name}");
    }
}
