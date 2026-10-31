//! What:
//!  Batched Git work for the seeded differential run of the strict reverse check:
//!  every case's base and landed commits come from one `git fast-import`,
//!  every patch from one `git diff-tree --stdin`,
//!  and the `git apply --reverse --check` oracle runs on a few worker threads.
//! Why:
//!  Starting Git roughly a dozen times per case made the run last minutes on a loaded host,
//!  and the watchdog of the unit-test run ended it.
//!  The patch text of each case is the text a repository of its own would print.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await Promise.all(workers.map(() => pullCases().map((job) => spawn('git', ['apply', '--reverse', '--check', job.patch]))));
//! ```

use crate::replay_patch::split_keeping_newlines;
use crate::test_support::{REAL_GIT, git, git_output, isolated_overlay};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

/// How many `git apply` checks run at the same time.
const WORKERS: usize = 4;

/// One seeded case: the text before the landed edit, after it, and the prepared text the strict check is asked about.
pub(super) struct ReverseCase {
    /// The case number in the seeded run, used in failure messages and directory names.
    pub(super) number: usize,
    /// The text before the landed edit.
    pub(super) base: Vec<u8>,
    /// The text after the landed edit.
    pub(super) landed: Vec<u8>,
    /// The prepared text whose reverse application is checked.
    pub(super) prepared: Vec<u8>,
}

/// Run real Git in a fixture with `input` on its standard input, and require success.
fn git_with_input(directory: &Path, arguments: &[&str], input: &[u8]) -> Output {
    let mut child: std::process::Child = Command::new(REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .envs(isolated_overlay())
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("native Git fixture command");
    let mut stdin: std::process::ChildStdin = child.stdin.take().expect("piped standard input");
    // A writer thread keeps a large input from blocking on a full pipe while Git writes its output.
    let (output, written): (Output, std::io::Result<()>) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || { return stdin.write_all(input); });
        let output: Output = child.wait_with_output().expect("Git output");
        return (output, writer.join().expect("input writer"));
    });
    assert!(output.status.success(), "git {arguments:?} failed: {}", String::from_utf8_lossy(&output.stderr));
    written.expect("input written to Git");
    return output;
}

/// The `git fast-import` stream that commits every pair as a base commit and a landed commit on its own branch,
///  with file `0` holding the text and the marks `2n + 1` (base) and `2n + 2` (landed) for pair `n`.
fn import_stream(pairs: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut stream: Vec<u8> = Vec::new();
    for (index, (base, landed)) in pairs.iter().enumerate() {
        for (step, (message, text)) in [("base", *base), ("landed", *landed)].into_iter().enumerate() {
            let mark: usize = 2 * index + 1 + step;
            stream.extend_from_slice(format!("commit refs/heads/case-{index}\nmark :{mark}\n").as_bytes());
            stream.extend_from_slice(format!("committer Fixture <fixture@example.invalid> {mark} +0000\n").as_bytes());
            stream.extend_from_slice(format!("data {}\n{message}\n", message.len()).as_bytes());
            if step == 1 {
                stream.extend_from_slice(format!("from :{}\n", mark - 1).as_bytes());
            }
            stream.extend_from_slice(format!("M 100644 inline 0\ndata {}\n", text.len()).as_bytes());
            stream.extend_from_slice(text);
            stream.push(b'\n');
        }
    }
    return stream;
}

/// The commit ids of the landed commits, in pair order, from the marks file `git fast-import` exported.
fn landed_commits(marks: &str, count: usize) -> Vec<String> {
    let all: std::collections::HashMap<usize, &str> = marks
        .lines()
        .filter_map(|line| {
            let (mark, id) = line.strip_prefix(':')?.split_once(' ')?;
            return Some((mark.parse::<usize>().ok()?, id));
        })
        .collect();
    return (0..count).map(|index| { return String::from(all[&(2 * index + 2)]); }).collect();
}

/// The raw patch `git diff-tree -p --text` prints for each pair between the trees of its base and landed commits,
///  each as the repository of the pair alone would print it.
/// One repository named `batch` is made in `directory`; the pairs are committed by one `git fast-import`
///  and diffed by one `git diff-tree --stdin`.
pub(super) fn patches_between_all(directory: &Path, pairs: &[(&[u8], &[u8])], context: u32) -> Vec<Vec<u8>> {
    let repo: PathBuf = directory.join("batch");
    std::fs::create_dir(&repo).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    git_with_input(
        repo.as_path(),
        &["fast-import", "--quiet", "--export-marks=marks.txt"],
        import_stream(pairs).as_slice(),
    );
    let marks: String = std::fs::read_to_string(repo.join("marks.txt")).expect("exported marks");
    let landed: Vec<String> = landed_commits(marks.as_str(), pairs.len());
    let unified: String = format!("--unified={context}");
    let listing: Vec<u8> = landed.iter().map(|id| { return format!("{id}\n"); }).collect::<String>().into_bytes();
    let output: Output = git_with_input(
        repo.as_path(),
        &[
            "diff-tree",
            "--stdin",
            "-p",
            "--text",
            "--no-renames",
            "--no-ext-diff",
            "--no-color",
            unified.as_str(),
            "--src-prefix=a/",
            "--dst-prefix=b/",
        ],
        listing.as_slice(),
    );
    // Each patch follows a line holding the commit id it belongs to; a patch line never looks like one,
    // because every patch line starts with a marker or a keyword.
    let mut patches: Vec<Vec<u8>> = Vec::new();
    for line in split_keeping_newlines(&output.stdout) {
        if landed.get(patches.len()).is_some_and(|id| { return line == format!("{id}\n").as_bytes(); }) {
            patches.push(Vec::new());
        } else {
            patches.last_mut().expect("a commit id line starts the output").extend_from_slice(&line);
        }
    }
    assert_eq!(patches.len(), pairs.len(), "one patch per pair");
    return patches;
}

/// Whether `git apply --reverse --check` accepts the patch of one case against its prepared text,
///  in a directory of its own under `root`.
/// A failure to prepare the directory panics with the case number.
fn git_accepts(root: &Path, case: &ReverseCase, patch: &[u8]) -> bool {
    let number: usize = case.number;
    let case_directory: PathBuf = root.join(format!("case-{number}"));
    std::fs::create_dir(&case_directory).unwrap_or_else(|error| { panic!("case {number}: case directory: {error}"); });
    std::fs::write(case_directory.join("0"), &case.prepared)
        .unwrap_or_else(|error| { panic!("case {number}: prepared entry: {error}"); });
    let patch_path: PathBuf = case_directory.join("landed.patch");
    std::fs::write(&patch_path, patch).unwrap_or_else(|error| { panic!("case {number}: patch file: {error}"); });
    let patch_argument: String = patch_path.to_string_lossy().into_owned();
    return git_output(
        case_directory.as_path(),
        &["apply", "--reverse", "--check", patch_argument.as_str()],
    )
    .status
    .success();
}

/// Whether Git accepts each case's patch reversed against its prepared text, in case order.
/// At most four checks run at once, each in its own directory under `root`.
/// A panic in a worker, which names its case, ends the run with the same message.
pub(super) fn git_verdicts(root: &Path, cases: &[ReverseCase], patches: &[Vec<u8>]) -> Vec<bool> {
    let next: AtomicUsize = AtomicUsize::new(0);
    // `mut` lets each finished worker fill in the verdicts it reached.
    let mut verdicts: Vec<Option<bool>> = vec![None; cases.len()];
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..WORKERS)
            .map(|_| {
                return scope.spawn(|| {
                    let mut reached: Vec<(usize, bool)> = Vec::new();
                    loop {
                        let index: usize = next.fetch_add(1, Ordering::Relaxed);
                        let Some(case) = cases.get(index) else {
                            return reached;
                        };
                        reached.push((index, git_accepts(root, case, patches[index].as_slice())));
                    }
                });
            })
            .collect();
        for worker in workers {
            match worker.join() {
                Ok(reached) => {
                    for (index, verdict) in reached {
                        verdicts[index] = Some(verdict);
                    }
                }
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
    });
    return verdicts.into_iter().map(|verdict| { return verdict.expect("every case reached"); }).collect();
}
