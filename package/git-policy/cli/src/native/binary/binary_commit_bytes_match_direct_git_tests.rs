//! What:
//!  A table of commit shapes run twice over the same starting state,
//! once through the wrapper and once through real Git,
//!  comparing what each command ended with:
//! its exit status,
//! the exact bytes of the commit object it made,
//! the tree,
//!  the index entries and the worktree's status.
//! Why:
//!  Two audits of release 0.6.1-alpha.12 compared wrapped commits with direct ones byte for byte
//!  over about a hundred and twenty shapes.
//!  Every row here is one shape those audits found differing,
//!  or a spelling beside one of them that must not differ,
//! so a wrapper that
//!  keeps a rule of Git's as its own fails here the moment the two drift apart.
//!  Both arms make their base commit with real Git and the same fixed dates,
//!  so the bases are one object and only the command under test differs;
//!  a wrapped command starts its own service,
//!  as a person's does.
//!
//! The audits:
//!  `alpha-12-commit-landing-service-push-and-changeover` (`binary_audit_probe_commit_matrix_tests.rs`,
//!  `binary_audit_probe_non_utf8_tests.rs`)
//!  and `kimi-audit-alpha-12-commit-path` (`differential-commit-cases.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const shape of SHAPES) {
//!   prepareBoth(shape); const wrapped = runWrapped(shape); const direct = runDirect(shape); compare(wrapped, direct, shape.expectation);
//! }
//! ```

use super::commit::{keep_maintenance_in_the_foreground, wrapped_commit};
use super::support::{Fixture, Observed, direct, fixture, git, observe, remove, stderr_of};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Output;

/// The author's date every commit of this table is made with,
/// so the two arms' commits are the same object when nothing differs.
const AUTHOR_DATE: &str = "2001-02-03T04:05:06+0000";
/// The committer's date,
/// which is also the commit time.
const COMMITTER_DATE: &str = "2001-02-03T04:05:06+0000";

/// The name of an author or committer whose bytes are not UTF-8:
/// `Ren\xe9 \xff`, as a legacy Latin-1 `.gitconfig` holds it.
const RAW_NAME: &[u8] = b"Ren\xe9 \xff";

/// Files the two arms must both have as untracked files,
/// or not at all,
/// before a row is prepared.
///  A row that writes one of them leaves it behind for the next one otherwise.
const TRANSIENT_NAMES: [&str; 3] = ["d.txt", "list", "msg.txt"];

/// Whether a row's two arms must agree on everything the command made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Expectation {
    /// Both arms end the same way,
    /// with the same bytes.
    Same,
    /// Both commands make no commit and end with the same status,
    /// and the wrapper's display of what is left names a path an index mark hides,
    /// which Git's own display leaves out:
    /// the mark lives in the real index and the wrapper's own commit index does not carry it.
    /// `SPEC.md`, "Explicit-path commit", records the difference.
    RefusesAndNamesOnePathMore,
    /// The project records this difference on purpose:
    /// the wrapper commits the paths the command line names,
    /// and real Git refuses them.
    /// `doc/decision/cli-git-policies-platform.md`, "Transactional commit gate",
    /// names "selected deletion and untracked paths" among the commits it covers.
    WrapperCommitsWhatGitRefuses,
}

/// What a row adds to the index beyond writing files.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IndexMark {
    /// Nothing.
    None,
    /// The path's entry is marked skip-worktree,
    /// which hides the worktree's bytes from Git's own commit.
    SkipWorktree(&'static str),
    /// The path's entry is marked assume-unchanged.
    AssumeUnchanged(&'static str),
}

/// One commit shape and the state it starts from.
struct Shape {
    /// What the row covers, in words.
    label: &'static str,
    /// Files written after the base commit, name and content.
    edits: &'static [(&'static str, &'static [u8])],
    /// Files written and never committed, name and content.
    extra: &'static [(&'static str, &'static [u8])],
    /// Paths staged after the edits.
    stage: &'static [&'static str],
    /// The mark the row's preparation puts on an index entry.
    mark: IndexMark,
    /// The arguments after the program's name.
    argv: &'static [&'static str],
    /// Raw bytes for an environment variable,
    /// which a Rust string cannot carry.
    env: &'static [(&'static str, &'static [u8])],
    /// What each arm wrote to standard input.
    stdin: &'static [u8],
    /// Whether the two arms must agree on everything.
    expectation: Expectation,
}

/// A row with nothing set,
/// which every row starts from.
const EMPTY: Shape = Shape {
    label: "",
    edits: &[],
    extra: &[],
    stage: &[],
    mark: IndexMark::None,
    argv: &[],
    env: &[],
    stdin: b"",
    expectation: Expectation::Same,
};

/// What one arm ended with.
struct Ended {
    /// What the caller saw.
    observed: Observed,
    /// The raw bytes of the commit object `HEAD` names,
    /// or nothing when the branch has none.
    object: Option<Vec<u8>>,
    /// The tree `HEAD` names.
    tree: String,
    /// The index entries,
    /// as `git ls-files --stage` prints them.
    index: String,
    /// The worktree's status,
    /// untracked files included.
    status: String,
}

/// What:
///  A byte string as text a failure message can print,
/// with every byte that is not printable text escaped.
/// Why:
///   The point of the rows whose identity is not UTF-8 is which bytes the object holds,
/// so a failure must show them rather than a replacement character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = (bytes) => [...bytes].map((b) => b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : `\x${hex(b)}`).join('');
/// ```
fn escaped(bytes: &[u8]) -> String {
    let mut text: String = String::new();
    for byte in bytes {
        if (0x20..0x7f).contains(byte) {
            text.push(char::from(*byte));
        } else {
            text.push_str(format!("\\x{byte:02x}").as_str());
        }
    }
    return text;
}

/// What:
///  The command for one arm,
/// with the fixed dates and the raw-byte environment the row names.
/// Why:
///   Both arms must see the same clock and the same bytes,
/// so the objects they make compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const command = wrappedRepo ? wrappedCommit(...) : direct(...); command.env(dates); command.env(shape.env);
/// ```
fn command_for(fixture: &Fixture, wrapped: bool, shape: &Shape, repo: &Path) -> std::process::Command {
    let mut command: std::process::Command = if wrapped {
        // The repository's commit-only policy refuses a line of its own before this table's question is asked,
        // so the wrapper's policy flags are turned off for both arms; the policy's refusals have their own controls.
        let mut policy_off: Vec<&str> = shape.argv.to_vec();
        if let Some(at) = policy_off.iter().position(|word| { return *word == "commit"; }) {
            policy_off.insert(at + 1, "--no-enforce-only");
            policy_off.insert(at + 2, "--no-enforce-require-root");
        }
        wrapped_commit(fixture, repo, policy_off.as_slice())
    } else {
        let mut plain: std::process::Command = direct(fixture);
        plain.current_dir(repo).args(shape.argv);
        plain
    };
    command.env("GIT_AUTHOR_DATE", AUTHOR_DATE).env("GIT_COMMITTER_DATE", COMMITTER_DATE);
    for (name, bytes) in shape.env {
        command.env(name, OsStr::from_bytes(bytes));
    }
    return command;
}

/// What:
///  Everything one finished command left in its repository.
/// Why:
///   The commit's bytes are the requirement;
/// the tree,
/// the index and the status show what the command did to the person's
///  repository around them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = { observed, object: catFileCommitHead(repo), tree: revParseHeadTree(repo), index: lsFilesStage(repo), status: porcelain(repo) };
/// ```
fn ended(fixture: &Fixture, observed: Observed, repo: &Path) -> Ended {
    let object: Output = git_output(fixture, repo, &["cat-file", "commit", "HEAD"]);
    let tree: Output = git_output(fixture, repo, &["rev-parse", "HEAD^{tree}"]);
    let index: Output = git_output(fixture, repo, &["ls-files", "--stage"]);
    let status: Output = git_output(fixture, repo, &["status", "--porcelain=v1", "--untracked-files=all"]);
    return Ended {
        observed,
        object: object.status.success().then_some(object.stdout),
        tree: String::from_utf8_lossy(tree.stdout.as_slice()).trim().to_string(),
        index: String::from_utf8_lossy(index.stdout.as_slice()).into_owned(),
        status: String::from_utf8_lossy(status.stdout.as_slice()).into_owned(),
    };
}

/// Real Git's answer to a query,
/// which is not an assertion on its status.
fn git_output(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Output {
    return direct(fixture).current_dir(repo).args(arguments).output().expect("Git starts");
}

/// What:
///  Why one row's two arms differ,
/// or nothing when they agree as the row expects.
/// Why:
///   A failure names the field that differs and shows both arms' bytes,
/// so a rule of Git's that drifted is a diff and not a puzzle.
///  Standard error is compared only where Git refuses the line,
/// because that refusal is the answer the person must be given in Git's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const difference = (wrapped, direct, expectation) => string | undefined;
/// ```
fn difference(wrapped: &Ended, direct: &Ended, expectation: Expectation) -> Option<String> {
    if expectation == Expectation::WrapperCommitsWhatGitRefuses {
        if direct.observed.code != Some(0) && wrapped.observed.code == Some(0) {
            return None;
        }
        return Some(format!(
            "the wrapper must commit what real Git refuses: wrapped exit {:?}, direct exit {:?}",
            wrapped.observed.code, direct.observed.code
        ));
    }
    let mut differences: Vec<String> = Vec::new();
    if expectation == Expectation::RefusesAndNamesOnePathMore {
        if wrapped.observed.code != Some(0) && direct.observed.code != Some(0) && wrapped.tree == direct.tree && wrapped.index == direct.index {
            return None;
        }
        return Some(format!("the wrapper must refuse as real Git refuses: wrapped exit {:?}, direct exit {:?}", wrapped.observed.code, direct.observed.code));
    }
    if (wrapped.observed.code == Some(0)) != (direct.observed.code == Some(0)) {
        differences.push(format!("exit status: wrapped {:?}, direct {:?}", wrapped.observed.code, direct.observed.code));
    }
    let objects_agree: bool = match (wrapped.object.as_deref(), direct.object.as_deref()) {
        (Some(left), Some(right)) => left == right,
        (None, None) => true,
        _ => false,
    };
    if !objects_agree {
        differences.push(format!(
            "commit object bytes:\n  wrapped {}\n  direct  {}",
            wrapped.object.as_deref().map_or_else(|| String::from("none"), escaped),
            direct.object.as_deref().map_or_else(|| String::from("none"), escaped)
        ));
    }
    if wrapped.tree != direct.tree {
        differences.push(format!("tree: wrapped {}, direct {}", wrapped.tree, direct.tree));
    }
    if wrapped.index != direct.index {
        differences.push(format!("index:\n  wrapped {}\n  direct  {}", escaped(wrapped.index.as_bytes()), escaped(direct.index.as_bytes())));
    }
    if wrapped.status != direct.status {
        differences.push(format!("status:\n  wrapped {}\n  direct  {}", escaped(wrapped.status.as_bytes()), escaped(direct.status.as_bytes())));
    }
    // A command Git refuses must be refused in Git's own words on both streams: the person sees the same answer
    //  whichever program gave it.
    if direct.observed.code != Some(0) {
        let wrapped_words: String = stderr_of(&wrapped.observed);
        let direct_words: String = stderr_of(&direct.observed);
        if wrapped_words != direct_words {
            differences.push(format!("standard error:\n  wrapped {wrapped_words:?}\n  direct  {direct_words:?}"));
        }
        let wrapped_shown: String = String::from_utf8_lossy(&wrapped.observed.stdout).into_owned();
        let direct_shown: String = String::from_utf8_lossy(&direct.observed.stdout).into_owned();
        if wrapped_shown != direct_shown {
            differences.push(format!("standard output:\n  wrapped {wrapped_shown:?}\n  direct  {direct_shown:?}"));
        }
    }
    return (!differences.is_empty()).then(|| differences.join("\n"));
}

/// What:
///  Make one arm's base commit:
/// the files this table starts from,
/// committed by real Git with the fixed dates.
/// Why:
///   Both arms must start from the same bytes,
/// so a difference is the command's and never the setup's,
///  and the base object is one object on both sides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const base = commitBase({ fixture, repo, files: BASE_FILES });
/// ```
fn commit_base(fixture: &Fixture, repo: &Path) -> String {
    let names: Vec<&str> = BASE_FILES.iter().map(|(name, _content)| { return *name; }).collect();
    for (name, content) in BASE_FILES {
        std::fs::write(repo.join(name), content).expect("base file");
    }
    let mut staging: Vec<&str> = vec!["add", "--"];
    staging.extend(names.iter());
    git(fixture, repo, staging.as_slice());
    let mut command: std::process::Command = direct(fixture);
    command
        .current_dir(repo)
        .args(["commit", "--quiet", "--message=base"])
        .env("GIT_AUTHOR_DATE", AUTHOR_DATE)
        .env("GIT_COMMITTER_DATE", COMMITTER_DATE);
    let committed: Output = command.output().expect("base commit runs");
    assert!(committed.status.success(), "base commit: {}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let base: Output = git_output(fixture, repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(base.stdout.as_slice()).trim().to_string();
}

/// The files this table's base commit records.
const BASE_FILES: &[(&str, &[u8])] = &[("a.txt", b"one\n"), ("b.txt", b"one\n")];

/// What:
///  Return one arm's repository to the base commit and prepare the row on it.
/// Why:
///   A row that runs after another must not inherit its edits,
/// its staged bytes,
/// its index marks or the untracked files it wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// resetToBase({ fixture, repo, shape, base }); // reset --hard, clear marks, remove transients, write, add, mark
/// ```
fn prepare_row(fixture: &Fixture, repo: &Path, shape: &Shape, base: &str) {
    git(fixture, repo, &["reset", "--hard", "--quiet", base]);
    for name in TRANSIENT_NAMES {
        let _ = std::fs::remove_file(repo.join(name));
    }
    // One start per mark, because one `update-index` reads only the last of these flags.
    for (name, _content) in BASE_FILES {
        let _ = git_output(fixture, repo, &["update-index", "--no-skip-worktree", "--", name]);
        let _ = git_output(fixture, repo, &["update-index", "--no-assume-unchanged", "--", name]);
    }
    for (name, content) in shape.edits {
        std::fs::write(repo.join(name), content).expect("edited file");
    }
    for (name, content) in shape.extra {
        std::fs::write(repo.join(name), content).expect("extra file");
    }
    if !shape.stage.is_empty() {
        let mut staging: Vec<&str> = vec!["add", "--"];
        staging.extend_from_slice(shape.stage);
        git(fixture, repo, staging.as_slice());
    }
    let marking: Option<[&str; 3]> = match shape.mark {
        IndexMark::None => None,
        IndexMark::SkipWorktree(path) => Some(["update-index", "--skip-worktree", path]),
        IndexMark::AssumeUnchanged(path) => Some(["update-index", "--assume-unchanged", path]),
    };
    if let Some(arguments) = marking.as_ref() {
        git(fixture, repo, arguments.as_slice());
    }
    keep_maintenance_in_the_foreground(fixture, repo);
}

/// The two repositories one row compares.
struct Repos {
    /// The repository the wrapper works in.
    wrapped: PathBuf,
    /// The repository real Git works in.
    direct: PathBuf,
}

/// What:
///  One row: prepare both arms,
/// run each arm's command,
/// and compare what they ended with.
/// Why:
///   The comparison is the test;
/// a row that differs says which field did and what each arm held,
///  and every row is compared so one run names every shape that drifted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = compareRow({ fixture, shape, base, repos }); // undefined when the row agrees
/// ```
fn compare_row(fixture: &Fixture, shape: &Shape, base: &str, repos: &Repos) -> Option<String> {
    prepare_row(fixture, repos.wrapped.as_path(), shape, base);
    prepare_row(fixture, repos.direct.as_path(), shape, base);
    let mut wrapped_command: std::process::Command = command_for(fixture, true, shape, repos.wrapped.as_path());
    let wrapped: Ended = ended(fixture, observe(&mut wrapped_command, shape.stdin), repos.wrapped.as_path());
    let mut direct_command: std::process::Command = command_for(fixture, false, shape, repos.direct.as_path());
    let direct: Ended = ended(fixture, observe(&mut direct_command, shape.stdin), repos.direct.as_path());
    return difference(&wrapped, &direct, shape.expectation).map(|report| {
        return format!(
            "{} ({:?}) differs from real Git:\n{report}\n  wrapped standard output {:?}\n  wrapped standard error {:?}\n  direct standard output {:?}\n  direct standard error {:?}",
            shape.label,
            shape.argv,
            String::from_utf8_lossy(&wrapped.observed.stdout),
            stderr_of(&wrapped.observed),
            String::from_utf8_lossy(&direct.observed.stdout),
            stderr_of(&direct.observed),
        );
    });
}

/// What:
///  Run every row of the table over two repositories that hold the same bytes,
/// and fail on the first row whose wrapper answer differs from real Git's.
/// Why:
///  This is the comparison the two audits made by hand,
/// kept so that a later change to a rule of Git's the wrapper
///  keeps for itself is caught by the suite rather than by the next audit.
#[test]
fn every_shape_ends_as_the_same_bytes_real_git_makes() {
    let fixture: Fixture = fixture("commit-bytes-match-direct");
    let repos: Repos = Repos {
        wrapped: fixture.root.join("wrapped"),
        direct: fixture.root.join("direct"),
    };
    let mut bases: Vec<String> = Vec::new();
    for repo in [repos.wrapped.as_path(), repos.direct.as_path()] {
        std::fs::create_dir(repo).expect("repository directory");
        git(&fixture, repo, &["init", "--quiet", "--initial-branch=main"]);
        bases.push(commit_base(&fixture, repo));
    }
    assert_eq!(bases[0], bases[1], "the two arms' base commits must be the same object");
    let base: String = bases.remove(0);
    let mut differences: Vec<String> = Vec::new();
    for shape in SHAPES {
        if let Some(report) = compare_row(&fixture, shape, base.as_str(), &repos) {
            differences.push(report);
        }
    }
    remove(&fixture);
    assert!(differences.is_empty(), "{}", differences.join("\n\n"));
}

/// The rows:
/// the two audits' first rows,
/// the spellings beside them,
/// and the shapes whose rules the wrapper used to keep its own.
const SHAPES: &[Shape] = &[
    Shape {
        label: "an abbreviated --only",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        argv: &["commit", "--onl", "-m", "x", "b.txt"],
        ..EMPTY
    },
    Shape {
        label: "an abbreviated --include",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        argv: &["commit", "--incl", "-m", "x", "b.txt"],
        ..EMPTY
    },
    Shape {
        label: "the shortest spelling of --only",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        argv: &["commit", "--o", "-m", "x", "b.txt"],
        ..EMPTY
    },
    Shape {
        label: "--only inside a short cluster",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        argv: &["commit", "-om", "x", "b.txt"],
        ..EMPTY
    },
    Shape {
        label: "an abbreviated --pathspec-from-file",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        extra: &[("list", b"b.txt\n")],
        argv: &["commit", "--pathspec-fr", "list", "-m", "x"],
        ..EMPTY
    },
    Shape {
        label: "a no-break space at the end of the message",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "--quiet", "--message=subject\u{a0}", "--", "a.txt"],
        ..EMPTY
    },
    Shape {
        label: "a figure space at the end of the message",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "--quiet", "--message=subject\u{2007}", "--", "a.txt"],
        ..EMPTY
    },
    Shape {
        label: "a vertical tab at the end of the message",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "--quiet", "--message=subject\u{b}", "--", "a.txt"],
        ..EMPTY
    },
    Shape {
        label: "a committer name that is not UTF-8",
        edits: &[("a.txt", b"one\ntwo\n")],
        stage: &["a.txt"],
        argv: &["commit", "--quiet", "-m", "x"],
        env: &[("GIT_COMMITTER_NAME", RAW_NAME)],
        ..EMPTY
    },
    Shape {
        label: "an author name that is not UTF-8",
        edits: &[("a.txt", b"one\ntwo\n")],
        stage: &["a.txt"],
        argv: &["commit", "--quiet", "-m", "x"],
        env: &[("GIT_AUTHOR_NAME", RAW_NAME)],
        ..EMPTY
    },
    Shape {
        label: "-a beside an include selection",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        argv: &["commit", "-i", "-a", "-m", "x", "b.txt"],
        ..EMPTY
    },
    Shape {
        label: "-o beside -i",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "-o", "-i", "-m", "x", "--", "a.txt"],
        ..EMPTY
    },
    Shape {
        label: "--include with no path",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "--include", "-m", "x"],
        ..EMPTY
    },
    Shape {
        label: "--only with no path",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "--only", "-m", "x"],
        ..EMPTY
    },
    Shape {
        label: "-a with a named path",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        argv: &["commit", "-a", "-m", "x", "b.txt"],
        ..EMPTY
    },
    Shape {
        label: "--pathspec-file-nul without --pathspec-from-file",
        edits: &[("a.txt", b"one\ntwo\n")],
        argv: &["commit", "-m", "x", "--pathspec-file-nul"],
        ..EMPTY
    },
    Shape {
        label: "a named path a skip-worktree entry hides",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        mark: IndexMark::SkipWorktree("a.txt"),
        argv: &["commit", "--quiet", "-m", "skip", "--", "a.txt"],
        expectation: Expectation::RefusesAndNamesOnePathMore,
        ..EMPTY
    },
    Shape {
        label: "-a with a skip-worktree entry",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        mark: IndexMark::SkipWorktree("a.txt"),
        argv: &["commit", "--quiet", "-am", "skip"],
        ..EMPTY
    },
    Shape {
        label: "a named path an assume-unchanged entry marks",
        edits: &[("a.txt", b"one\ntwo\n"), ("b.txt", b"one\ntwo\n")],
        mark: IndexMark::AssumeUnchanged("a.txt"),
        argv: &["commit", "--quiet", "-m", "assume", "--", "a.txt"],
        ..EMPTY
    },
    Shape {
        label: "a named untracked path, which this project's commit-only policy commits",
        edits: &[("a.txt", b"one\ntwo\n")],
        extra: &[("d.txt", b"d\n")],
        argv: &["commit", "-m", "x", "d.txt"],
        expectation: Expectation::WrapperCommitsWhatGitRefuses,
        ..EMPTY
    },
];
