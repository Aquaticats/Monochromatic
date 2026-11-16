//! What:
//!  The caller's global pathspec mode,
//!  as an option and as an environment variable,
//!  crossed with every read the discarding guard makes of its own:
//!  a marked edit the status hides,
//!  an edit a `skip-worktree` mark hides,
//!  the destinations of `checkout-index --prefix`,
//!  the untracked and ignored entries a hard reset writes over,
//!  a glob a prefix shifts,
//!  and the case of a name.
//! Why:
//!  The guard asks Git questions of its own,
//!  and those starts inherit whatever the caller put before the subcommand or in the environment
//!  (`facts.global_prefix`).
//!  A mode that changes how a pathspec is read therefore changes what such a read answers:
//!  under `GIT_LITERAL_PATHSPECS` the guard's own `:(top)` magic is a file name that matches
//!  nothing, so an edit and a set of destinations went unseen and Git wrote over both
//!  (the ninth review's second and third findings).
//!  So every mode is run against every read,
//!  with real Git as the oracle:
//!  the native arm is the situation,
//!  and the wrapped arm must never lose an entry the person holds,
//!  whichever mode the caller chose.
//!  The row without a mode is the answer each mode is held against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const mode of modes) for (const scenario of scenarios) expect(wrappedLost(mode)).toEqual([])
//! ```

use super::support::{Fixture, Observed, direct, fixture, git, observe, remove, repository, stderr_of, wrapped};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The text every file a person holds begins with, so a row finds a loss by its content.
const USER_DATA: &str = "USER DATA";

/// Whether the wrapper itself rejected the command,
///  which is the only answer that leaves every entry where the person left it.
fn refused(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// Every entry below `root` as the person holds it:
///  a file's bytes marked `F`,
///  or a link's target marked `L`.
/// Why:
///  A mode may make Git write through a link or replace it,
///  and both are the person's entry lost,
///  so the row records the entry at the name and not only a file's bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if entry.file_name() == ".git" {
                continue;
            }
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            let relative: PathBuf = path.strip_prefix(root).unwrap_or(path.as_path()).to_path_buf();
            if kind.is_symlink() {
                if let Ok(target) = std::fs::read_link(path.as_path()) {
                    let mut held: Vec<u8> = b"L".to_vec();
                    held.extend_from_slice(target.as_os_str().as_encoded_bytes());
                    found.insert(relative, held);
                }
                continue;
            }
            let mut held: Vec<u8> = b"F".to_vec();
            held.extend_from_slice(std::fs::read(path.as_path()).unwrap_or_default().as_slice());
            found.insert(relative, held);
        }
    }
    return found;
}

/// The entries a person held that are no longer what they held.
fn lost(before: &BTreeMap<PathBuf, Vec<u8>>, after: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<String> {
    let mut gone: Vec<String> = Vec::<String>::new();
    for (path, held) in before {
        let theirs: bool = String::from_utf8_lossy(&held[1..]).starts_with(USER_DATA) || held.first() == Some(&b'L');
        if theirs && after.get(path) != Some(held) {
            gone.push(path.display().to_string());
        }
    }
    return gone;
}

/// One read of the guard, with the situation that makes it answer.
///  An `enum` is a closed set of named alternatives.
/// Why:
///  Each scenario names one question the guard asks Git itself,
///  so a mode that changes one read's answer fails a row of its own.
enum Scenario {
    /// The follow-up `ls-files -v --stage` for an `assume-unchanged` edit.
    MarkedEdit,
    /// The `ls-files -v` listing that finds an edit a `skip-worktree` mark hides.
    SkippedEdit,
    /// The index listing whose `:(top)` finds `checkout-index --prefix` destinations.
    PrefixDestinations,
    /// The whole-worktree listing of the untracked entries a hard reset writes over.
    ResetUntracked,
    /// The same listing for the entries the ignore rules cover.
    ResetIgnored,
    /// The index listing a glob pathspec reaches, shifted by a prefix.
    GlobAtAPrefix,
    /// The index listing whose answer a case-insensitive pathspec widens.
    EditUnderAnotherCase,
}

impl Scenario {
    /// Every scenario, in the order the rows are reported.
    const ALL: &'static [Scenario] = &[
        Scenario::MarkedEdit,
        Scenario::SkippedEdit,
        Scenario::PrefixDestinations,
        Scenario::ResetUntracked,
        Scenario::ResetIgnored,
        Scenario::GlobAtAPrefix,
        Scenario::EditUnderAnotherCase,
    ];

    /// The name a row is reported by.
    fn name(&self) -> &'static str {
        return match self {
            Scenario::MarkedEdit => "an assume-unchanged edit",
            Scenario::SkippedEdit => "an edit a skip-worktree mark hides",
            Scenario::PrefixDestinations => "checkout-index destinations below a prefix",
            Scenario::ResetUntracked => "an untracked entry a hard reset writes over",
            Scenario::ResetIgnored => "an ignored entry a hard reset writes over",
            Scenario::GlobAtAPrefix => "a glob pathspec shifted by a prefix",
            Scenario::EditUnderAnotherCase => "an edit another spelling of the name reaches",
        };
    }

    /// The words after the subcommand, as the person types them.
    fn arguments(&self) -> Vec<&'static str> {
        return match self {
            Scenario::MarkedEdit => vec!["restore", "--", "a.txt"],
            Scenario::SkippedEdit => vec!["restore", "--source=other", "--", "a.txt"],
            Scenario::PrefixDestinations => vec!["checkout-index", "-f", "--prefix=out/", "-a"],
            Scenario::ResetUntracked | Scenario::ResetIgnored => vec!["reset", "--hard", "other"],
            Scenario::GlobAtAPrefix => vec!["checkout-index", "-f", "--prefix=out/", "--", "a*"],
            Scenario::EditUnderAnotherCase => vec!["restore", "--", "a.txt"],
        };
    }

    /// The worktree one scenario runs in,
    ///  made fresh for every arm because the command writes.
    fn prepare(&self, fixture: &Fixture, name: &str) -> PathBuf {
        let repo: PathBuf = repository(fixture, OsStr::new(name));
        let linked: PathBuf = fixture.root.join(format!("{name}-linked"));
        std::fs::write(repo.join("a.txt"), b"main a\n").expect("a tracked file");
        std::fs::write(repo.join(".gitignore"), b"ignored-tree/\n").expect("ignore rules");
        git(fixture, repo.as_path(), &["add", "--", "a.txt", ".gitignore"]);
        git(fixture, repo.as_path(), &["commit", "--quiet", "--message=main"]);
        git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "other"]);
        std::fs::write(repo.join("a.txt"), b"other a\n").expect("the other tree's a");
        std::fs::write(repo.join("b.txt"), b"other b\n").expect("the other tree's b");
        git(fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
        git(fixture, repo.as_path(), &["commit", "--quiet", "--message=other"]);
        git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
        let linked_text: &str = linked.to_str().expect("a fixture path is text");
        git(fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked_text]);
        let precious = |path: PathBuf| {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("a directory on the way");
            }
            std::fs::write(path.as_path(), format!("{USER_DATA} {}\n", path.display())).expect("an entry a person holds");
        };
        match self {
            Scenario::MarkedEdit => {
                git(fixture, linked.as_path(), &["update-index", "--assume-unchanged", "a.txt"]);
                std::fs::write(linked.join("a.txt"), format!("{USER_DATA} the edit\n")).expect("the edit");
            }
            Scenario::SkippedEdit => {
                git(fixture, linked.as_path(), &["update-index", "--skip-worktree", "a.txt"]);
                std::fs::write(linked.join("a.txt"), format!("{USER_DATA} the edit\n")).expect("the edit");
            }
            Scenario::PrefixDestinations => {
                precious(linked.join("out/a.txt"));
                precious(linked.join("out/.gitignore"));
            }
            Scenario::ResetUntracked => {
                precious(linked.join("b.txt"));
            }
            Scenario::ResetIgnored => {
                // The ignore rule of `main`'s tree covers a name the other tree tracks, so a hard reset writes
                //  over an entry the untracked listing alone never names.
                std::fs::write(repo.join(".gitignore"), b"b.txt\n").expect("an ignore rule for the other tree's name");
                git(fixture, repo.as_path(), &["add", "--", ".gitignore"]);
                git(fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore-b"]);
                git(fixture, linked.as_path(), &["checkout", "--quiet", "--force", "side"]);
                precious(linked.join("b.txt"));
            }
            Scenario::GlobAtAPrefix => {
                precious(linked.join("out/a.txt"));
                std::fs::write(linked.join("ab.txt"), format!("{USER_DATA} beside it\n")).expect("another tracked name");
                git(fixture, linked.as_path(), &["add", "--", "ab.txt"]);
                git(fixture, linked.as_path(), &["commit", "--quiet", "--message=ab"]);
            }
            Scenario::EditUnderAnotherCase => {
                // The index holds `A.TXT`, and the person reaches it with the other case.
                std::fs::remove_file(linked.join("a.txt")).expect("the lower-case name goes");
                git(fixture, linked.as_path(), &["rm", "--quiet", "--cached", "--force", "--", "a.txt"]);
                std::fs::write(linked.join("A.TXT"), b"main upper\n").expect("the upper-case name");
                git(fixture, linked.as_path(), &["add", "--", "A.TXT"]);
                git(fixture, linked.as_path(), &["commit", "--quiet", "--message=upper"]);
                std::fs::write(linked.join("A.TXT"), format!("{USER_DATA} the edit\n")).expect("the edit");
            }
        }
        return linked;
    }
}

/// One mode, as the caller writes it:
///  a variable and its value, or a global option.
struct Mode {
    /// How the row names the mode.
    name: &'static str,
    /// The variable, when the caller sets one.
    variable: Option<&'static str>,
    /// The global option, when the caller writes one.
    option: Option<&'static str>,
}

/// Every global pathspec mode: the four variables and the four options.
const MODES: &[Mode] = &[
    Mode { name: "GIT_LITERAL_PATHSPECS", variable: Some("GIT_LITERAL_PATHSPECS"), option: None },
    Mode { name: "GIT_GLOB_PATHSPECS", variable: Some("GIT_GLOB_PATHSPECS"), option: None },
    Mode { name: "GIT_NOGLOB_PATHSPECS", variable: Some("GIT_NOGLOB_PATHSPECS"), option: None },
    Mode { name: "GIT_ICASE_PATHSPECS", variable: Some("GIT_ICASE_PATHSPECS"), option: None },
    Mode { name: "--literal-pathspecs", variable: None, option: Some("--literal-pathspecs") },
    Mode { name: "--glob-pathspecs", variable: None, option: Some("--glob-pathspecs") },
    Mode { name: "--noglob-pathspecs", variable: None, option: Some("--noglob-pathspecs") },
    Mode { name: "--icase-pathspecs", variable: None, option: Some("--icase-pathspecs") },
];

/// What one arm did:
///  whether the wrapper refused,
///  and which entries the person no longer holds.
struct Arm {
    /// Whether a policy refusal is what the caller saw.
    refused: bool,
    /// The entries lost, each as the worktree names it.
    lost: Vec<String>,
}

/// Run one scenario's words in one arm,
///  under one mode.
fn run(scenario: &Scenario, mode: Option<&Mode>, wrapped_arm: bool, label: &str) -> Arm {
    let fixture: Fixture = fixture(label);
    let directory: PathBuf = scenario.prepare(&fixture, "repo");
    let before: BTreeMap<PathBuf, Vec<u8>> = snapshot(directory.as_path());
    let mut words: Vec<String> = Vec::<String>::new();
    if let Some(option) = mode.and_then(|mode| return mode.option) {
        words.push(String::from(option));
    }
    for argument in scenario.arguments() {
        words.push(String::from(argument));
    }
    let borrowed: Vec<&str> = words.iter().map(|word| return word.as_str()).collect();
    let observed: Observed = if wrapped_arm {
        let mut command = wrapped(&fixture);
        command.current_dir(directory.as_path()).args(borrowed.as_slice());
        if let Some(variable) = mode.and_then(|mode| return mode.variable) {
            command.env(variable, "1");
        }
        observe(&mut command, b"")
    } else {
        let mut command = direct(&fixture);
        command.current_dir(directory.as_path()).args(borrowed.as_slice());
        if let Some(variable) = mode.and_then(|mode| return mode.variable) {
            command.env(variable, "1");
        }
        observe(&mut command, b"")
    };
    let after: BTreeMap<PathBuf, Vec<u8>> = snapshot(directory.as_path());
    let arm: Arm = Arm { refused: refused(&observed), lost: lost(&before, &after) };
    remove(&fixture);
    return arm;
}

/// What:
///  Every global pathspec mode, against every read the guard makes:
///  the wrapped arm never loses an entry the person holds,
///  and no arm is refused where Git alone writes nothing.
/// Why:
///  A mode is the caller's own words,
///  so the guard's reads must answer the same thing under it as without it,
///  and the only visible difference left is Git's own:
///  the native arm says whether the situation exists.
///  The row without a mode is the answer each mode is held against,
///  printed beside it so a difference can be read rather than guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const scenario of scenarios) {
///   const baseline = run(scenario, undefined, true);
///   for (const mode of modes) expect(run(scenario, mode, true).lost).toEqual([]);
/// }
/// ```
#[test]
fn every_read_of_the_guard_answers_the_same_under_every_global_pathspec_mode() {
    let mut rows: usize = 0;
    let mut failures: Vec<String> = Vec::<String>::new();
    for scenario in Scenario::ALL {
        let baseline: Arm = run(scenario, None, true, "pathspec-mode-baseline");
        println!("{}: without a mode the wrapper {} and the person lost {:?}", scenario.name(), if baseline.refused { "refused" } else { "allowed it" }, baseline.lost);
        rows += 1;
        for mode in MODES {
            let wrapped_arm: Arm = run(scenario, Some(mode), true, "pathspec-mode-wrapped");
            let native_arm: Arm = run(scenario, Some(mode), false, "pathspec-mode-native");
            rows += 1;
            println!(
                "{} under {}: Git alone lost {:?}; the wrapper {} and the person lost {:?}",
                scenario.name(),
                mode.name,
                native_arm.lost,
                if wrapped_arm.refused { "refused" } else { "allowed it" },
                wrapped_arm.lost
            );
            if !wrapped_arm.lost.is_empty() {
                failures.push(format!(
                    "{} under {}: the person lost {:?} (Git alone lost {:?})",
                    scenario.name(),
                    mode.name,
                    wrapped_arm.lost,
                    native_arm.lost
                ));
            }
            if wrapped_arm.refused && native_arm.lost.is_empty() && !native_arm.refused {
                failures.push(format!(
                    "{} under {}: the wrapper refused a command Git runs without losing an entry the person holds",
                    scenario.name(),
                    mode.name
                ));
            }
        }
    }
    println!("rows: {rows}; rows that lost an entry or refused a harmless command: {}", failures.len());
    assert!(failures.is_empty(), "{} rows:\n{}", rows, failures.join("\n"));
}
