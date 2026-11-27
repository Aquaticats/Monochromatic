//! The files whose one-shot Git questions go through the captured run,
//! and the one path of them that signals no process by a number it no longer holds:
//! pins over another module's source text,
//! which is a subject of its own beside the run's own behavioural controls.

use crate::source_guard::source_scan_tests::{
    classified_sources,
    real_roots,
    source_root,
};
use std::path::PathBuf;

/// What:
///  `QUESTION_MODULES` names the two folders under `src/native/` whose files ask Git a one-shot question:
///  each folder plus its `/`,
///  which is how a source's identity is written.
/// Why:
///  The call the control below requires is found across a module's files rather than in one named file,
///  because a split moves such a call into a sibling file and leaves behind the file it left;
///  the round for issue 6540 moved `output_of_within_bound` out of `repo_read/git_system_config.rs`
///  into `repo_read/git_system_config_asking.rs` on 2026-11-27 (`e58254244d`),
///  and a control that named the file it left asserted a line that file no longer held (issue 6573).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const QUESTION_MODULES = ['repo_read/', 'invocation/'];
/// ```
const QUESTION_MODULES: &[&str] = &["repo_read/", "invocation/"];

/// What:
///  `THE_CAPTURED_RUN` is the call a one-shot Git question of those modules is asked through:
///  a child awaited to its own end,
///  with its output read from files rather than through a pipe.
/// Why:
///  A question asked any other way waits for its child with no bound,
///  which is the failure both modules' questions were changed to lose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_CAPTURED_RUN = 'output_of_within_bound';
/// ```
const THE_CAPTURED_RUN: &str = "output_of_within_bound";

/// What:
///  `BLOCKING_ON_OUTPUT` is the standard library's call that waits for a child with no bound
///  and collects its whole output in memory.
/// Why:
///  Naming it once keeps both assertions of the control below about one text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BLOCKING_ON_OUTPUT = '.output()';
/// ```
const BLOCKING_ON_OUTPUT: &str = ".output()";

/// What:
///  `fn code_lines(text: &str) -> String` is one source file's code:
///  every line that is not a comment,
///  joined with newlines.
/// Why:
///  A comment may name a call the file does not make,
///  and the header of a split is exactly where the name of the call that moved out survives.
///  This crate writes every comment as whole lines,
///  so cutting at a leading `//` loses no code.
/// Gotcha:
///  The cut is textual:
///  a call written inside a string literal is kept and would be read as a call,
///  which is why the words a file says about itself are checked by its own module,
///  not here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function codeLines(text: string): string {
///   return text.split('\n').filter((line) => !line.trimStart().startsWith('//')).join('\n');
/// }
/// ```
fn code_lines(text: &str) -> String {
    // What: `filter` keeps the lines that do not start with a comment marker, and `collect` gathers them.
    // Why:  The join below needs the whole list, and the newlines it puts back are the only ones lost.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kept = text.split('\n').filter((line) => !line.trimStart().startsWith('//'));
    // ```
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| {
            return !line.trim_start().starts_with("//");
        })
        .collect();
    // What: `join` puts one newline between the lines it kept.
    // Why:  A needle that spans two lines of code then reads as it does in the file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return kept.join('\n');
    // ```
    return kept.join("\n");
}

/// The one-shot Git questions of the two modules are asked through the captured run,
/// and no release file of either module blocks a thread on `Command::output`.
///
/// What:
///  The call is required of a module's files rather than of one named file of it,
///  and the failure names every file that was read.
/// Why:
///  A split moves a call into a sibling file:
///  `repo_read/git_system_config.rs` lost
///  `output_of_within_bound` to `repo_read/git_system_config_asking.rs` on 2026-11-27,
///  so a control that named the file it left failed on `main` for a move that changed nothing
///  about the question it guards (issue 6573).
///  A module is what the claim is about:
///  the question is still asked in `repo_read/`,
///  and a caller that goes back to `Command::output` in any file of either module still fails here.
/// Gotcha:
///  The scan reads the tree's module declarations,
///  so a file no `mod` declaration reaches is an error
///  rather than a file this control silently skips.
#[test]
fn the_one_shot_git_questions_no_longer_block_a_thread_on_command_output() {
    // What: `source_root()` is this crate's `src/native`, and `real_roots()` is what `Cargo.toml` starts
    //       the build from; `classified_sources` follows those roots' declarations to every file.
    // Why:  The list is the build's own tree, so a module that moved into a subfolder is still read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const root = sourceRoot();
    // const sources = classifiedSources(root, realRoots());
    // ```
    let root: PathBuf = source_root();
    let roots = real_roots();
    let sources =
        classified_sources(root.as_path(), roots.as_slice()).expect("the crate's own sources");
    for module in QUESTION_MODULES {
        // What: `read` is every file of the module that was looked at, so a failure can name them all.
        // Why:  A module whose files could not be read would otherwise pass by having nothing to check.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const read: string[] = [];
        // ```
        let mut read: Vec<&str> = Vec::new();
        // What: `asking` is the files of the module that reach the captured run, and `blocking` is the
        //       files that call `Command::output`.
        // Why:  The two lists are the two halves of the claim, and each failure names its own list.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const asking: string[] = [];
        // const blocking: string[] = [];
        // ```
        let mut asking: Vec<&str> = Vec::new();
        let mut blocking: Vec<&str> = Vec::new();
        for file in sources.iter().filter(|file| {
            return !file.test_only && file.path.starts_with(module);
        }) {
            let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
                .expect("a source file of this crate");
            let code: String = code_lines(text.as_str());
            if code.contains(THE_CAPTURED_RUN) {
                asking.push(file.path.as_str());
            }
            if code.contains(BLOCKING_ON_OUTPUT) {
                blocking.push(file.path.as_str());
            }
            read.push(file.path.as_str());
        }
        assert!(
            !read.is_empty(),
            "no release file under {module} was read, so this control guards nothing there"
        );
        assert!(
            blocking.is_empty(),
            "{module} blocks a thread on `Command::output`, which no ceiling ends, in: {blocking:?}"
        );
        assert!(
            !asking.is_empty(),
            "no release file under {module} asks Git through `{THE_CAPTURED_RUN}` any more, so the \
             question that module asks is waited for with no bound; the files read are: {read:?}"
        );
    }
}

/// What:
///  Neither query path signals a process by its number.
/// Why:
///  A number is a safe address only while its process is this process's own unreaped child,
/// and the thread that waits for
///  a query reaps it as soon as it ends,
/// so the number read at the start may name another process by the time a ceiling
///  is reached.
/// Nothing on these paths signals a number at all:
/// the one-shot query is waited for,
/// and ended,
/// by the thread that
///  owns it (`wait_within`),
/// and the batch query is read by the runtime,
/// which owns its child for the same reason
///  (`read_until_the_child_ends`).
#[test]
fn neither_query_path_signals_by_number() {
    let sources: [(&str, &str); 2] = [
        (
            "process/captured_child_run_to_its_end.rs",
            include_str!("captured_child_run_to_its_end.rs"),
        ),
        (
            "push/manual_push_range.rs",
            include_str!("../push/manual_push_range.rs"),
        ),
    ];
    for (name, source) in sources {
        let code: String = code_lines(source);
        assert!(
            !code.contains("libc::kill"),
            "{name} still signals a process by its number"
        );
        assert!(
            code.contains("wait_within") || code.contains("read_until_the_child_ends"),
            "{name} does not wait for its child with a ceiling that its own thread keeps"
        );
    }
}
