//! What:
//!  What a repository holds after a command, as labelled text:
//!  its head,
//!  its references,
//!  the fields of its newest commit,
//!  the index and the two diffs,
//!  and the state files a merge, a cherry-pick or a revert leaves.
//! Why:
//!  A byte copy has the same history,
//!  so the twin harness compares fields as text rather than as a hash,
//!  and a difference then reads as the field it is in;
//!  reading a repository is one job,
//!  so it lives in a file of its own beside the harness that calls it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = leftBehind(fixture, repo); // [['head', '...'], ['stored %T', '...'], ...]
//! ```

use super::support::{
    Fixture,
    NATIVE_GIT_BOUND_SECONDS,
    native_git_bound,
    native_git_output,
};
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// The state files a merge,
/// a cherry-pick or a revert leaves in the Git directory,
/// which a conclusion removes.
const STATE_FILES: &[&str] = &[
    "MERGE_HEAD",
    "MERGE_MSG",
    "MERGE_MODE",
    "SQUASH_MSG",
    "AUTO_MERGE",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "MERGE_RR",
    "ORIG_HEAD",
];

/// A command for native Git in `repo` under `bound`,
/// without the wrapper and without the person's own configuration,
/// ready for the environment the caller adds.
pub(super) fn native_git_command(
    fixture: &Fixture,
    repo: &Path,
    arguments: &[&str],
    bound: &str,
) -> Command {
    let mut command: Command = native_git_bound(fixture, bound);
    command.args(arguments);
    command.current_dir(repo);
    return command;
}

/// The text of a Git query,
/// whether or not Git succeeded,
/// under `bound`,
/// as the bytes Git wrote them.
/// The output is kept in files rather than pipes:
/// `native_git_output` says why.
pub(super) fn text_of_within(
    fixture: &Fixture,
    repo: &Path,
    arguments: &[&str],
    bound: &str,
) -> String {
    let output: std::process::Output = native_git_output(
        fixture,
        &mut native_git_command(fixture, repo, arguments, bound),
        repo,
        bound,
    );
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned()
        + &String::from_utf8_lossy(output.stderr.as_slice());
}

/// The text of a Git query,
/// whether or not Git succeeded,
/// under the bound every ordinary read of this harness gets.
pub(super) fn text_of(
    fixture: &Fixture,
    repo: &Path,
    arguments: &[&str],
) -> String {
    return text_of_within(fixture, repo, arguments, NATIVE_GIT_BOUND_SECONDS);
}

/// What the repository holds that a commit decides,
/// as labelled text.
pub(super) fn left_behind(
    fixture: &Fixture,
    repo: &Path,
) -> Vec<(String, String)> {
    let mut left: Vec<(String, String)> = Vec::new();
    let mut add = |label: &str, value: String| {
        left.push((label.to_string(), value));
    };
    add(
        "head",
        text_of(fixture, repo, &["rev-parse", "--verify", "--quiet", "HEAD"]),
    );
    add(
        "symbolic head",
        text_of(fixture, repo, &["symbolic-ref", "--quiet", "HEAD"]),
    );
    add(
        "references",
        text_of(
            fixture,
            repo,
            &["for-each-ref", "--format=%(refname) %(objectname)"],
        ),
    );
    let formats: [&str; 9] = [
        "%T",
        "%B",
        "%an <%ae>",
        "%ad",
        "%cn <%ce>",
        "%cd",
        "%P",
        "%H",
        "%G?",
    ];
    for format in formats {
        add(
            format!("stored {format}").as_str(),
            text_of(
                fixture,
                repo,
                &[
                    "log",
                    "-1",
                    "--date=iso-strict",
                    format!("--format={format}").as_str(),
                ],
            ),
        );
    }
    add(
        "count",
        text_of(fixture, repo, &["rev-list", "--count", "--all"]),
    );
    add(
        "status",
        text_of(
            fixture,
            repo,
            &["status", "--short", "--untracked-files=all"],
        ),
    );
    add(
        "staged",
        text_of(fixture, repo, &["diff", "--cached", "--raw", "--no-abbrev"]),
    );
    add("index", text_of(fixture, repo, &["ls-files", "--stage"]));
    add(
        "unstaged",
        text_of(fixture, repo, &["diff", "--raw", "--no-abbrev"]),
    );
    let git_dir: PathBuf = repo.join(".git");
    for name in STATE_FILES {
        let state: String = std::fs::read(git_dir.join(name)).map_or_else(
            |_| {
                return String::from("(absent)");
            },
            |bytes| {
                return String::from_utf8_lossy(bytes.as_slice()).into_owned();
            },
        );
        add(format!("state {name}").as_str(), state);
    }
    let sequencer: String = std::fs::read_dir(git_dir.join("sequencer")).map_or_else(
        |_| {
            return String::from("(absent)");
        },
        |entries| {
            let mut names: Vec<String> = entries
                .flatten()
                .map(|entry| {
                    return format!(
                        "{}={}",
                        entry.file_name().to_string_lossy(),
                        std::fs::read_to_string(entry.path()).unwrap_or_default()
                    );
                })
                .collect();
            names.sort();
            return names.join("\n");
        },
    );
    add("state sequencer", sequencer);
    return left;
}
