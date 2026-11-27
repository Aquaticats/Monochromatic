//! What:
//!  Every release call that replaces this process with another program puts back the action this process had for
//!  `SIGPIPE` when the replacement fails.
//! Why:
//!  `Command::exec` returns only when nothing was replaced,
//!  and before it tries it sets `SIGPIPE` to the kernel's default in the process that called it,
//!  which is `do_exec` in `library/std/src/sys/process/unix/unix.rs` at line 374.
//!  Every path of the wrapper that goes on after such a start writes its own words and ends with a status,
//!  and at the default the first of those writes to a pipe nobody reads ends the process by the signal instead
//!  The record is
//!  `doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md` for issue 6607.
//!  So the replacement puts the action back,
//!  and this scan judges each call that replaces the process:
//!  the call must be followed by the call that puts the action back before the next replacement of the same item.
//!  A file that makes one such call and puts the action back says nothing about a second call added beside it,
//!  and that is the shape a later round writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseSources) for (const item of items(code(file))) for (const call of replacements(item)) if (!restores(item).after(call).before(next(call))) finding(file, call);
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
///  one for each call of them that the call putting the action back does not follow.
/// Why:
///  The list of files is what shows the scan read something it guards,
///  so a tree where the replacement moved and the needle went stale fails here instead of passing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Scan { replacing: string[]; findings: string[] }
/// ```
struct Scan {
    /// The release files whose code replaces this process,
    /// as paths under the source root.
    replacing: Vec<String>,
    /// One finding for each replacement call that the call putting the action back does not follow.
    findings: Vec<String>,
}

/// What:
///  The line numbers where an item of a file ends:
///  every line that holds nothing but a closing brace,
///  which `rustfmt` writes at the start of the line for an item and never for a block inside one.
/// Why:
///  A replacement is judged inside the item that holds it,
///  and this is the boundary the text itself gives.
///  Two methods of one `impl` share an item, which only makes the check stricter:
///  a replacement with no restore after it is a finding either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function itemEnds(code: string): number[] { return lines(code).flatMap((l, i) => l.trimEnd() === '}' ? [i] : []); }
/// ```
fn item_ends(lines: &[&str]) -> Vec<usize> {
    let mut ends: Vec<usize> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.trim_end() == "}" {
            ends.push(index);
        }
    }
    return ends;
}

/// What:
///  The findings of one item of a file:
///  a call that replaces this process which no call putting the action back follows in that item.
/// Why:
///  Comments are cut before this is asked,
///  because the file that replaces the process names that call in prose beside the replacement,
///  and a prose mention is not the call.
///  The item is the stretch between two item ends,
///  and each replacement is judged up to the next replacement of the same item,
///  so a call with a put-back before and none after is a finding as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsInItem(path: string, item: string[], firstLine: number): string[] {
///   return replacements(item).filter((call, order) => !restores(item.slice(call + 1, next(order)))).map((call) => `${path}:${firstLine + call + 1}: ...`);
/// }
/// ```
fn findings_in_item(
    path: &str,
    item: &[&str],
    first_line_number: usize,
) -> Vec<String> {
    let calls: Vec<usize> = item
        .iter()
        .enumerate()
        .filter(|(_index, line)| {
            return THE_REPLACEMENTS
                .iter()
                .any(|needle| return line.contains(needle));
        })
        .map(|(index, _line)| return index)
        .collect();
    let mut findings: Vec<String> = Vec::new();
    for (order, call) in calls.iter().enumerate() {
        let end: usize = calls.get(order + 1).copied().unwrap_or(item.len());
        let followed: bool = item[call.saturating_add(1)..end]
            .iter()
            .any(|line| return line.contains(PUTS_THE_ACTION_BACK));
        if followed {
            continue;
        }
        findings.push(format!(
            "{path}:{}, a line of this file's code with its comments cut: replaces this process with another \
             program and the call `{PUTS_THE_ACTION_BACK}` does not follow it in the same item, so the \
             wrapper's own words end it by `SIGPIPE` after a failed start",
            first_line_number + call + 1
        ));
    }
    return findings;
}

/// What:
///  Every finding of one file's code:
///  each call that replaces this process which no put-back follows in its own item.
/// Why:
///  A file-level answer let a second call pass wherever one put-back stood in the file,
///  which is the shape a round adds beside the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsInCode(path: string, code: string): string[] { const lines = code.split('\n'); let first = 0; const found = []; for (const end of itemEnds(lines)) { found.push(...findingsInItem(path, lines.slice(first, end + 1), first)); first = end + 1; } if (first < lines.length) found.push(...findingsInItem(path, lines.slice(first), first)); return found; }
/// ```
fn findings_in_code(
    path: &str,
    code: &str,
) -> Vec<String> {
    let lines: Vec<&str> = code.lines().collect();
    let mut findings: Vec<String> = Vec::new();
    let mut first: usize = 0;
    for end in item_ends(lines.as_slice()) {
        findings.extend(findings_in_item(path, &lines[first..=end], first));
        first = end + 1;
    }
    if first < lines.len() {
        findings.extend(findings_in_item(path, &lines[first..], first));
    }
    return findings;
}

/// What:
///  Every finding of one file's text,
///  which is the code of it with its comments cut.
/// Why:
///  A control plants a text,
///  so the cut is made here once for it and for the scan below.
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
///  The scan of the release sources,
///  read as paths under `root`.
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

/// The words of one finding, so a control names the same sentence for each call it plants.
fn the_finding(
    path: &str,
    line_number: usize,
) -> String {
    return format!(
        "{path}:{line_number}, a line of this file's code with its comments cut: replaces this process with \
         another program and the call `put_the_action_back(libc::SIGPIPE, &before)` does not follow it in the same \
         item, so the wrapper's own words end it by `SIGPIPE` after a failed start"
    );
}

/// What:
///  The scan finds a replacement that does not put the action back,
///  whatever its spelling,
///  and says nothing about one that does,
///  nor about a comment that names the call.
/// Why:
///  A needle that stopped seeing one of the spellings would pass every tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const spelling of spellings) expect(findingsInText('planted.rs', spelling)).toEqual([theFinding('planted.rs', 2)]);
/// ```
#[test]
fn the_scan_finds_a_planted_replacement_that_does_not_put_the_signal_action_back() {
    for spelling in [
        "fn replace(command: &mut Command) -> Error {\n    return command.exec();\n}\n",
        "fn replace(command: &mut Command) -> Error {\n    let error: Error = std::os::unix::process::CommandExt::exec(command);\n    return error;\n}\n",
        "fn replace(command: &mut Command) -> Error {\n    let error: Error = CommandExt::exec(command);\n    return error;\n}\n",
    ] {
        assert_eq!(
            findings_in_text("planted.rs", spelling),
            vec![the_finding("planted.rs", 2)],
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
        vec![the_finding("planted_comment.rs", 2)],
        "a comment that only names a call is not a call, and the replacement beside it is still a finding"
    );
}

/// What:
///  A second call that replaces the process is a finding of its own beside the first,
///  where a put-back follows one call and none follows the other.
/// Why:
///  The file-level answer this scan gave before saw one put-back and passed the whole file,
///  so a round could add a call beside the one that restores and every run stayed green
///  (the alpha.49 behaviour read, answer 8).
///  A second call in the same item and one in another item are both the shape to catch,
///  and a put-back before a call is not a put-back after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findingsInText('two_functions.rs', twoFunctions)).toEqual([theFinding('two_functions.rs', 9)]);
/// expect(findingsInText('one_function.rs', oneFunction)).toEqual([theFinding('one_function.rs', 5)]);
/// ```
#[test]
fn a_second_replacement_of_the_process_is_a_finding_beside_the_one_that_puts_the_action_back() {
    let two_functions: &str = "\
fn replace(command: &mut Command) -> Error {
    let before: libc::sigaction = the_action_in_place(libc::SIGPIPE);
    let error: Error = command.exec();
    put_the_action_back(libc::SIGPIPE, &before);
    return error;
}

fn replace_again(command: &mut Command) -> Error {
    return command.exec();
}
";
    assert_eq!(
        findings_in_text("two_functions.rs", two_functions),
        vec![the_finding("two_functions.rs", 9)],
        "a second call in another function is a finding of its own"
    );
    let one_function: &str = "\
fn replace(command: &mut Command) -> Error {
    let before: libc::sigaction = the_action_in_place(libc::SIGPIPE);
    let error: Error = command.exec();
    put_the_action_back(libc::SIGPIPE, &before);
    let again: Error = other.exec();
    return again.or(error);
}
";
    assert_eq!(
        findings_in_text("one_function.rs", one_function),
        vec![the_finding("one_function.rs", 5)],
        "a second call in the same function, after the put-back the first one had, is a finding"
    );
}
