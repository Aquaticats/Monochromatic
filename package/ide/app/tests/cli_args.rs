//! Startup grammar preserves native paths and separates parsing from project or display initialization.

/// Inspect typed clap exits without terminating the test process.
use clap::{Error, error::ErrorKind};
/// Parse arguments through the production library entry point.
use ide_app::cli::{Options, Startup, parse_args, parse_args_with_home};
/// Native byte filenames must survive parsing,
///  including paths that are not valid UTF-8.
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::Path};

/// Convert readable ASCII fixtures to owned native argv values.
fn args(values: &[&str]) -> Vec<OsString> {
    // Each borrowed literal is converted to an owned value; the list owns the variable-length argv.
    let mut argv: Vec<OsString> = Vec::new();
    for value in values {
        argv.push(OsString::from(value));
    }
    return argv;
}

/// The options of a start that opens a window;
///  any other outcome fails the test.
fn opened(startup: Startup) -> Options {
    return match startup {
        Startup::Open(options) => options,
        Startup::Licenses => panic!("expected a window start, got the license listing"),
    };
}

/// Required root and optional initial-file operands remain uninterpreted native paths during parsing.
#[test]
fn project_and_initial_file_parse_without_filesystem_access() {
    let parsed = opened(
        parse_args(&args(&["/not-created/project", "--file", "src/猫.ts"]))
            .expect("startup grammar"),
    );
    assert_eq!(parsed.project, Path::new("/not-created/project"));
    assert_eq!(parsed.file.as_deref(), Some(Path::new("src/猫.ts")));
    assert!(
        opened(parse_args(&args(&["project"])).expect("project only"))
            .file
            .is_none()
    );
    let dashed = opened(
        parse_args(&args(&["--file=-source.rs", "--", "-project"])).expect("dash-prefixed paths"),
    );
    assert_eq!(dashed.project, Path::new("-project"));
    assert_eq!(dashed.file.as_deref(), Some(Path::new("-source.rs")));
}

/// Unix filenames are not decoded through UTF-8 or lossy UI labels.
#[test]
fn non_utf8_arguments_keep_their_original_bytes() {
    // from_vec constructs a native byte filename, unlike String which requires valid UTF-8.
    let project = OsString::from_vec(vec![b'p', 0xff]);
    let file = OsString::from_vec(vec![b'f', 0xfe]);
    // Clone retains the expected byte sequences after the argument vector takes its owned inputs.
    let parsed = opened(
        parse_args(&[project.clone(), OsString::from("--file"), file.clone()])
            .expect("native byte paths"),
    );
    assert_eq!(parsed.project.as_os_str(), project);
    assert_eq!(parsed.file.expect("initial file").as_os_str(), file);
}

/// Help/version have success exit semantics even without a project argument.
#[test]
fn help_and_version_are_successful_parser_outcomes() {
    for (flag, kind) in [
        ("--help", ErrorKind::DisplayHelp),
        ("--version", ErrorKind::DisplayVersion),
    ] {
        let result = parse_args(&args(&[flag])).expect_err("display request");
        let cli = result.downcast_ref::<Error>().expect("typed clap result");
        assert_eq!(cli.kind(), kind);
        assert_eq!(cli.exit_code(), 0);
        assert!(!cli.use_stderr());
    }
}

/// Without a project argument the home folder is the project;
///  an explicit project still wins.
#[test]
fn missing_project_opens_the_home_folder() {
    let home = Path::new("/var/home/someone");
    let parsed = opened(parse_args_with_home(&args(&[]), Some(home)).expect("home default"));
    assert_eq!(parsed.project, home);
    assert!(parsed.file.is_none());
    let with_file = opened(
        parse_args_with_home(&args(&["--file", "notes/todo.md"]), Some(home))
            .expect("home default with an initial file"),
    );
    assert_eq!(with_file.project, home);
    assert_eq!(with_file.file.as_deref(), Some(Path::new("notes/todo.md")));
    let explicit =
        opened(parse_args_with_home(&args(&["/srv/project"]), Some(home)).expect("explicit"));
    assert_eq!(explicit.project, Path::new("/srv/project"));
}

/// Without a project argument and without a usable home folder,
///  the grammar reports a usage error.
#[test]
fn missing_project_without_a_home_folder_is_a_usage_error() {
    let result = parse_args_with_home(&args(&[]), None).expect_err("nothing to open");
    let cli = result.downcast_ref::<Error>().expect("typed usage error");
    assert_eq!(cli.exit_code(), 2);
    assert!(cli.use_stderr());
    assert!(cli.to_string().contains("No PROJECT was given"), "{cli}");
}

/// `--licenses` asks for the license listing even without a project or a home folder,
///  and is a
/// usage error beside a project or a file,
///  which it would otherwise silently ignore.
#[test]
fn licenses_needs_no_project_and_accepts_none() {
    assert!(matches!(
        parse_args_with_home(&args(&["--licenses"]), None).expect("license listing"),
        Startup::Licenses
    ));
    assert!(matches!(
        parse_args_with_home(&args(&["--licenses"]), Some(Path::new("/var/home/someone")))
            .expect("license listing"),
        Startup::Licenses
    ));
    for values in [
        vec!["--licenses", "project"],
        vec!["project", "--licenses"],
        vec!["--licenses", "--file", "notes.md"],
    ] {
        let result = parse_args(&args(&values)).expect_err("licenses beside a start");
        let cli = result.downcast_ref::<Error>().expect("typed usage error");
        assert_eq!(cli.kind(), ErrorKind::ArgumentConflict, "{values:?}");
        assert_eq!(cli.exit_code(), 2);
    }
}

/// Extra roots,
///  repeated file selection,
///  unknown options,
///  and empty paths are rejected.
#[test]
fn invalid_argument_shapes_are_usage_errors() {
    for values in [
        vec!["--unknown"],
        vec!["one", "two"],
        vec!["project", "--file"],
        vec!["project", "--file="],
        vec![""],
        vec!["project", "--file", "one", "--file", "two"],
    ] {
        let result = parse_args(&args(&values)).expect_err("invalid argv");
        let cli = result.downcast_ref::<Error>().expect("typed usage error");
        assert_eq!(cli.exit_code(), 2);
        assert!(cli.use_stderr());
    }
}
