//! What:
//!  The scan that keeps every argument the discarding guard hands to a Git query in its own bytes.
//! Why:
//!  A path, a revision and a prefix are bytes, and Git holds any bytes in a name;
//!  a conversion through `String`, `to_string_lossy` or `String::from_utf8_lossy` replaces every byte
//!  that is not UTF-8 with U+FFFD,
//!  so the argument Git receives is another name.
//!  A lossy revision reached another branch and let a forced checkout write over a person's file:
//!  `--with-tree=<revision>` was built with `to_string_lossy`,
//!  the replacement-character ref pointed at `main`, the tree listed no `victim.txt`, and the command
//!  ran while Git wrote the file out of the raw-byte ref
//!  (the tenth review's second finding).
//!  The guard's modules are read line by line, and a line that builds a query argument out of a lossy
//!  conversion fails here rather than in a person's worktree.
//!  A lossy conversion in a refusal's sentence is not one of these: it names the bytes for a reader and
//!  is never handed to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const line of guardFiles) if (buildsAnArgument(line) && isLossy(line)) fail(line);
//! ```

use crate::source_scan_tests::{real_roots, source_root};
use std::path::PathBuf;

/// The modules of the discarding guard's own pass,
/// as `/`-separated paths relative to `src/native`.
/// Why:
///  These are the modules that build the arguments of every Git query the guard makes:
///  the written paths, the entries in the way, the edits the status hides, the dirty question,
///  the submodules, and the policy that asks them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GUARD_READERS = ['repo_read/repository_facts_written_paths.rs', /* ... */];
/// ```
const GUARD_READERS: &[&str] = &[
    "command/rule_worktree_dirty.rs",
    "policy/policy_linked_worktree.rs",
    "repo_read/repository_facts_edits_the_status_hides.rs",
    "repo_read/repository_facts_paths_in_the_way.rs",
    "repo_read/repository_facts_submodules.rs",
    "repo_read/repository_facts_written_paths.rs",
    "repo_read/repository_facts.rs",
];

/// The ways a lossy conversion appears in this crate.
/// Why:
///  Two spellings do the same thing, and a scan that named one would let the other through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOSSY = ['to_string_lossy', 'from_utf8_lossy'];
/// ```
const LOSSY: &[&str] = &["to_string_lossy", "from_utf8_lossy"];

/// The words that say a line builds an argument Git will read.
/// Why:
///  A query's arguments are pushed into a vector, formatted with a long option's own prefix,
///  or built as a pathspec;
///  a sentence for a person is none of these, and the guard's refusals name paths lossily on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BUILDS = ['arguments.push', 'listed.push', 'format!("--', 'spec_argument(', 'path_argument('];
/// ```
const BUILDS: &[&str] = &[
    "arguments.push",
    "listed.push",
    "arguments.extend",
    "spec_argument(",
    "path_argument(",
    "format!(\"--",
];

/// What:
///  Whether one line hands Git a value it converted lossily.
/// Why:
///  The two conditions are what the defect needs at once:
///  a lossy conversion, and a use of its result as a query argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lossyArgument = (line) => LOSSY.some((word) => line.includes(word)) && BUILDS.some((word) => line.includes(word));
/// ```
fn lossy_argument(line: &str) -> bool {
    let lossy: bool = LOSSY.iter().any(|word| return line.contains(word));
    let builds: bool = BUILDS.iter().any(|word| return line.contains(word));
    return lossy && builds;
}

/// What:
///  Every line of every guard reader, with the file it came from.
///  `&Path` borrows the crate's source root.
/// Why:
///  One pass over the files the scan names,
///  each line trimmed so a wrapped call is still read as one line's neighbour.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = (root) => GUARD_READERS.flatMap((file) => read(file).split('\n'));
/// ```
fn lines_of_the_guard_readers(root: &PathBuf) -> Vec<(&'static str, String)> {
    let mut found: Vec<(&'static str, String)> = Vec::<(&'static str, String)>::new();
    for file in GUARD_READERS {
        let text: String = std::fs::read_to_string(root.join(file)).expect("a guard reader the scan names");
        assert!(!text.is_empty(), "{file} is empty");
        for line in text.lines() {
            found.push((file, line.trim().to_owned()));
        }
    }
    return found;
}

/// What:
///  No argument the guard hands to Git is built out of a lossy conversion.
/// Why:
///  A lossy conversion is another name for the same bytes,
///  and Git reads the name it was given:
///  the tenth review reached another branch that way and let a forced checkout write over a person's
///  file.
///  The scan is generated from the guard's own modules, so a new query builder is read the moment it is
///  written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesOfTheGuardReaders(root).filter(lossyArgument)).toEqual([]);
/// ```
#[test]
fn no_query_argument_the_guard_hands_to_git_is_a_lossy_conversion() {
    let root: PathBuf = source_root();
    let offenders: Vec<String> = lines_of_the_guard_readers(&root)
        .into_iter()
        .filter(|(_file, line)| return lossy_argument(line))
        .map(|(file, line)| return format!("{file}: {line}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "every argument the guard gives Git keeps its own bytes (`OsStr::as_encoded_bytes` or \
         `path_from_bytes`), and a lossy conversion is only for a sentence a person reads:\n{}",
        offenders.join("\n")
    );
}

/// The scan reads files the compiler reads, so a renamed module fails here rather than being skipped.
#[test]
fn the_scan_reads_every_guard_reader() {
    let root: PathBuf = source_root();
    let sources: Vec<String> = crate::source_scan_tests::classified_sources(&root, &real_roots())
        .expect("the crate's sources")
        .into_iter()
        .map(|file| return file.path)
        .collect();
    for file in GUARD_READERS {
        assert!(
            sources.iter().any(|path| return path == file),
            "{file} is not a release source of this crate, so the list names a file the compiler never reads"
        );
    }
}
