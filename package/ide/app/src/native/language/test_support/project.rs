//! The disposable project a test opens,
//!  the directories beside it,
//!  and the scripted server its language definition names.

/// Name of the scripted server definition,
///  which the definition written here refers to.
use super::SERVER;
/// A project in memory when the machine has one,
///  beside the other window tests' fixtures.
use crate::native::find_tests::memory_project;
/// Fixture files,
///  and the paths the definition and the server addresses are built from.
use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  A disposable project:
///  the directory guard,
///  the canonical root,
///  and a scratch directory
///       beside the project for outside-project files.
/// Why:
///  Dropping the guard removes everything;
///  servers report canonical paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Project = { guard: TempDir; root: string; outside: string };
/// ```
pub(super) struct Project {
    /// Removes the directory when dropped.
    _guard: tempfile::TempDir,
    /// Canonical project root.
    pub(super) root: PathBuf,
    /// Canonical directory beside the project,
    ///  outside it.
    pub(super) outside: PathBuf,
}

/// Create a disposable project with `files` (relative path and text) and an outside directory.
pub(super) fn project(files: &[(&str, &str)]) -> Project {
    // The project lives in memory when `/dev/shm` exists (see `memory_project`): on btrfs, the first
    // read of a fresh file and the creation of files wait for filesystem transactions, which took
    // longer than the waits of these tests while the machine flushed.
    let guard = crate::native::find_tests::memory_project("ide-native-language-");
    let base = guard.path().canonicalize().expect("canonical base");
    let root = base.join("project");
    let outside = base.join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    for (relative, text) in files {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("fixture directories");
        fs::write(&path, text).expect("fixture file");
    }
    return Project {
        _guard: guard,
        root,
        outside,
    };
}

/// The `file` address of a canonical path,
///  as a server names it.
pub(super) fn address(path: &Path) -> String {
    return format!("file://{}", path.display());
}

/// A JSON location for the scripted server:
///  an address and a zero-based line and column range.
pub(super) fn location(
    uri: &str,
    line: u32,
    start: u32,
    end: u32,
) -> String {
    return format!(
        r#"{{"uri":"{uri}","range":{{"start":{{"line":{line},"character":{start}}},"end":{{"line":{line},"character":{end}}}}}}}"#
    );
}

/// What:
///  Language definitions for the scripted server with extra `IDE_SCRIPTED_*` variables.
///       `command` replaces the server program,
///  for the missing-program case.
/// Why:
///  Each test names exactly the answers it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function definitions(variables: [string, string][], command?: string): string
/// ```
pub(super) fn definitions(
    variables: &[(&str, &str)],
    command: Option<&str>,
) -> String {
    // The scripted server is built beside the test binary's `deps` directory.
    let program = std::env::current_exe()
        .expect("test executable")
        .parent()
        .and_then(Path::parent)
        .expect("target directory")
        .join("ide-scripted-lsp");
    assert!(
        command.is_some() || program.is_file(),
        "build ide-scripted-lsp before the native tests: {}",
        program.display()
    );
    let program_text = program.display().to_string();
    let chosen = command.unwrap_or(&program_text);
    // What: `Vec<String>` is a growable list of owned strings (sibling: a fixed array `[String; N]`);
    //       `iter().any(...)` asks whether one pair names `PUSH`; `*name` reads the borrowed name.
    // Why: The server pushes a diagnostic on every publish unless told not to. Tests that do not
    //      ask for it turn it off, and a test that names `PUSH` decides alone: an inline TOML
    //      table refuses the same key twice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entries: string[] = variables.some(([name]) => name === 'PUSH') ? [] : ["IDE_SCRIPTED_PUSH = '0'"];
    // ```
    let mut entries: Vec<String> = Vec::new();
    let mut names_push = false;
    for (name, _value) in variables {
        if *name == "PUSH" {
            names_push = true;
            break;
        }
    }
    if !names_push {
        entries.push(String::from("IDE_SCRIPTED_PUSH = '0'"));
    }
    for (name, value) in variables {
        entries.push(format!("IDE_SCRIPTED_{name} = '{value}'"));
    }
    let environment = entries.join(", ");
    return format!(
        r#"
[language-server.{SERVER}]
command = '{chosen}'
timeout = 5
environment = {{ {environment} }}

[[language]]
name = "scripted"
scope = "source.scripted"
file-types = ["scripted"]
roots = []
language-servers = ["{SERVER}"]
"#
    );
}
