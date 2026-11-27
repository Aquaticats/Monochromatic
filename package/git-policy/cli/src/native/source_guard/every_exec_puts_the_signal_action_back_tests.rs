//! What:
//!  Every release call that replaces this process with another program puts back the action this process had for
//!  `SIGPIPE` when the replacement fails.
//! Why:
//!  `Command::exec` returns only when nothing was replaced,
//!  and before it tries it sets `SIGPIPE` to the kernel's default in the process that called it,
//!  which is `do_exec` in `library/std/src/sys/process/unix/unix.rs` at line 374.
//!  Every path of the wrapper that goes on after such a start writes its own words and ends with a status,
//!  and at the default the first of those writes to a pipe nobody reads ends the process by the signal instead
//!  (`doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md`, issue 6607).
//!  So the replacement puts the action back,
//!  and this scan fails a release file that replaces the process without it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseSources) if (code(file).includes('.exec()') && !code(file).includes(PUTS_IT_BACK)) finding(file);
//! ```

use std::path::{
    Path,
    PathBuf,
};

use super::release_sources_in;
use crate::source_scan_tests::{
    code_lines,
    real_roots,
    source_root,
};

/// What:
///  The calls that replace this process with another program:
///  the method a command has,
///  the trait's own call,
///  and the C calls a program replaces itself with.
/// Why:
///  A needle that names one spelling passes the others,
///  and the control below planted a `CommandExt::exec(` call the method's spelling did not see on 2026-11-27.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_REPLACEMENTS = ['.exec()', 'CommandExt::exec(', 'libc::execv'];
/// ```
const THE_REPLACEMENTS: &[&str] = &[".exec()", "CommandExt::exec(", "libc::execv"];

/// What:
///  The call beside the replacement that puts back the action this process had for `SIGPIPE`,
///  written with the two arguments the replacement passes it,
///  so a file that only defines that call and no longer makes it is still a finding.
/// Why:
///  A needle that names the function alone would pass a replacement whose call to it was removed,
///  which is the very change this scan exists to see (issue 6607).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PUTS_THE_ACTION_BACK = 'put_the_action_back(libc::SIGPIPE, &before)';
/// ```
const PUTS_THE_ACTION_BACK: &str = "put_the_action_back(libc::SIGPIPE, &before)";

/// What:
///  What the scan of the release sources found:
///  the files whose code replaces this process,
///  and the findings,
///  one for each of them that does not put the action back.
/// Why:
///  The list of files is what shows the scan read something it guards,
///  so a tree where the replacement moved and the needle went stale fails here instead of passing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Scan { replacing: string[]; findings: string[] }
/// ```
struct Scan {
    /// The release files whose code replaces this process, as paths under the source root.
    replacing: Vec<String>,
    /// One finding for each of them that does not put the action back.
    findings: Vec<String>,
}

/// What:
///  Every finding of one file's code:
///  a replacement of this process,
///  when the same file does not make the call that puts the action back.
/// Why:
///  Comments are cut before this is asked,
///  because the file that replaces the process names that call in prose beside the replacement,
///  and a prose mention is not the call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsInCode(path: string, code: string): string[] {
///   return code.includes(THE_REPLACEMENT) && !code.includes(PUTS_THE_ACTION_BACK) ? [`${path}: ...`] : [];
/// }
/// ```
fn findings_in_code(
    path: &str,
    code: &str,
) -> Vec<String> {
    let replaces: bool = THE_REPLACEMENTS.iter().any(|needle| {
        return code.contains(needle);
    });
    if !replaces {
        return Vec::new();
    }
    if code.contains(PUTS_THE_ACTION_BACK) {
        return Vec::new();
    }
    return vec![format!(
        "{path}: replaces this process with another program and does not make `{PUTS_THE_ACTION_BACK}`, so the \
         wrapper's own words end it by `SIGPIPE` after a failed start"
    )];
}

/// What:
///  Every finding of one file's text,
///  which is the code of it with its comments cut.
/// Why:
///  A control plants a text, so the cut is made here once for it and for the scan below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsInText(path: string, text: string): string[] { return findingsInCode(path, codeLines(text)); }
/// ```
fn findings_in_text(
    path: &str,
    text: &str,
) -> Vec<String> {
    return findings_in_code(path, code_lines(text).as_str());
}

/// What:
///  The scan of `sources`, read as paths under `root`.
/// Why:
///  One read of each file answers both questions the controls ask,
///  and a file that cannot be read stops the scan with its own name rather than passing as a file with no findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(root: string, sources: string[]): Scan
/// ```
fn scan(
    root: &Path,
    sources: &[String],
) -> Result<Scan, String> {
    let mut found: Scan = Scan {
        replacing: Vec::new(),
        findings: Vec::new(),
    };
    for path in sources {
        let text: String = std::fs::read_to_string(root.join(path.as_str()))
            .map_err(|error| return format!("cannot read {path}: {error}"))?;
        let code: String = code_lines(text.as_str());
        if THE_REPLACEMENTS.iter().any(|needle| {
            return code.contains(needle);
        }) {
            found.replacing.push(path.clone());
        }
        found
            .findings
            .extend(findings_in_code(path.as_str(), code.as_str()));
    }
    return Ok(found);
}

/// No release file replaces this process without putting the action back,
/// and the scan read the file that replaces it.
#[test]
fn every_release_replacement_of_the_process_puts_the_signal_action_back() {
    let root: PathBuf = source_root();
    let roots = real_roots();
    let sources: Vec<String> =
        release_sources_in(root.as_path(), roots.as_slice()).expect("the crate's release sources");
    let touch: Scan = scan(root.as_path(), sources.as_slice()).expect("the release sources' text");
    assert!(
        !touch.replacing.is_empty(),
        "no release file of the crate replaces this process any more, so this check guards nothing: the files \
         read are the {} the module tree reaches",
        sources.len()
    );
    assert!(
        touch.findings.is_empty(),
        "a release file replaces this process with another program and does not put back the action this process \
         had for `SIGPIPE`, so the wrapper's own words end it by the signal after a failed start (issue 6607); the \
         files that replace it are {:?}:\n{}",
        touch.replacing,
        touch.findings.join("\n")
    );
}

/// The scan finds a replacement that does not put the action back,
/// whatever its spelling,
/// and says nothing about one that does,
/// nor about a comment that names the call.
#[test]
fn the_scan_finds_a_planted_replacement_that_does_not_put_the_signal_action_back() {
    let expected: &str = "planted.rs: replaces this process with another program and does not make \
                          `put_the_action_back(libc::SIGPIPE, &before)`, so the wrapper's own words end it by \
                          `SIGPIPE` after a failed start";
    for spelling in [
        "fn replace(command: &mut Command) -> Error {\n    return command.exec();\n}\n",
        "fn replace(command: &mut Command) -> Error {\n    let error: Error = std::os::unix::process::CommandExt::exec(command);\n    return error;\n}\n",
        "fn replace(command: &mut Command) -> Error {\n    let error: Error = CommandExt::exec(command);\n    return error;\n}\n",
    ] {
        assert_eq!(
            findings_in_text("planted.rs", spelling),
            vec![String::from(expected)],
            "a planted replacement that puts nothing back was not one finding: {spelling}"
        );
    }
    let with: &str = "\
fn replace(command: &mut Command) -> Error {
    let before: libc::sigaction = the_action_in_place(libc::SIGPIPE);
    let error: Error = command.exec();
    put_the_action_back(libc::SIGPIPE, &before);
    return error;
}
";
    assert_eq!(
        findings_in_text("planted_with.rs", with),
        Vec::<String>::new(),
        "a replacement that puts the action back is no finding"
    );
    let comment_only: &str = "\
// A file whose comments name the call it no longer makes is the shape this scan reads.
fn replace(command: &mut Command) -> Error {
    return command.exec();
}
";
    assert_eq!(
        findings_in_text("planted_comment.rs", comment_only),
        vec![String::from(
            "planted_comment.rs: replaces this process with another program and does not make \
                          `put_the_action_back(libc::SIGPIPE, &before)`, so the wrapper's own words end it by \
                          `SIGPIPE` after a failed start"
        )],
        "a comment that only names a call is not a call, and the replacement beside it is still a finding"
    );
}
