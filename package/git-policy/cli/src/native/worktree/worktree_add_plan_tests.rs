//! Controls for the `git worktree add` plan:
//! the accepted forms,
//! every refusal,
//! and a real Git 2.56.0 run of each
//!  accepted form that registers the destination the plan names.

use super::*;
use crate::command_test_support::os_arguments;
use crate::test_support::{fixture, git, remove, repository};
use std::process::Output;

/// The arguments of an invocation as the plan reads them:
/// the global prefix,
/// `worktree`,
/// `add` and the rest.
fn plan_of(values: &[&str], subcommand_index: usize, start: &Path) -> Option<AddPlan> {
    return parse_add_plan(os_arguments(values).as_slice(), subcommand_index, start);
}

/// The forms of the table name the destination and the place directly after `add` for the insertion.
#[test]
fn an_accepted_form_names_its_destination_and_the_insertion_point() {
    let root: PathBuf = fixture("add-plan-accepted");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    for (values, index, operand) in [
        (vec!["worktree", "add", "topic"], 0, "topic"),
        (vec!["worktree", "add", "-b", "feature", "topic"], 0, "topic"),
        (vec!["worktree", "add", "topic", "-b", "feature", "main"], 0, "topic"),
        (vec!["worktree", "add", "--detach", "--quiet", "topic", "main"], 0, "topic"),
        (vec!["worktree", "add", "-d", "-q", "--lock", "--reason", "why", "topic"], 0, "topic"),
        (vec!["worktree", "add", "--reason=why", "--checkout", "topic"], 0, "topic"),
        (vec!["worktree", "add", "--", "-odd"], 0, "-odd"),
        (vec!["--no-pager", "worktree", "add", "topic"], 1, "topic"),
    ] {
        let plan: AddPlan = plan_of(values.as_slice(), index, root.as_path()).unwrap_or_else(|| { panic!("{values:?} is accepted"); });
        assert_eq!(plan.destination, canonical.join(operand), "{values:?}");
        assert_eq!(plan.insert_at, index + 2, "{values:?}");
        assert_eq!(os_arguments(values.as_slice())[plan.insert_at - 1], "add", "{values:?}");
        let quiet: bool = values.contains(&"--quiet") || values.contains(&"-q");
        assert_eq!(plan.quiet, quiet, "{values:?}");
    }
    remove(root.as_path());
}

/// Every form outside the table is a plain Git run:
/// unknown and bundled options,
/// the options that change the
///  checkout,
/// a wrong operand count,
/// and a word other than `add`.
#[test]
fn a_form_outside_the_table_is_declined() {
    let root: PathBuf = fixture("add-plan-declined");
    for values in [
        vec!["worktree", "add"],
        vec!["worktree", "add", "a", "b", "c"],
        vec!["worktree", "add", "--no-checkout", "topic"],
        vec!["worktree", "add", "--force", "topic"],
        vec!["worktree", "add", "-f", "topic"],
        vec!["worktree", "add", "-B", "feature", "topic"],
        vec!["worktree", "add", "--orphan", "topic"],
        vec!["worktree", "add", "--track", "topic"],
        vec!["worktree", "add", "--detac", "topic"],
        vec!["worktree", "add", "-dq", "topic"],
        vec!["worktree", "add", "-bfeature", "topic"],
        vec!["worktree", "add", "-b"],
        vec!["worktree", "add", "topic", "-b"],
        vec!["worktree", "add", "-b", "feature"],
        vec!["worktree", "move", "topic", "other"],
        vec!["worktree", "ad", "topic"],
        vec!["worktree", "--force", "add", "topic"],
        vec!["worktree", "add", "."],
        vec!["worktree", "add", ".."],
        vec!["worktree", "add", "missing-parent/topic"],
        vec!["add", "topic"],
    ] {
        assert_eq!(plan_of(values.as_slice(), 0, root.as_path()), None, "{values:?}");
    }
    remove(root.as_path());
}

/// A global option that can change how the ladder's own Git runs read the repository declines the start directory;
///  `-C` moves it and a `..` component in it is declined.
#[test]
fn the_global_prefix_decides_the_start_directory() {
    let root: PathBuf = fixture("add-plan-prefix");
    std::fs::create_dir(root.join("inner")).expect("inner");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    let moved: PathBuf = start_directory(os_arguments(&["-C", "inner", "--no-pager"]).as_slice(), root.as_path()).expect("moved");
    let plan: AddPlan = parse_add_plan(os_arguments(&["worktree", "add", "topic"]).as_slice(), 0, moved.as_path()).expect("plan");
    assert_eq!(plan.destination, canonical.join("inner").join("topic"));
    let chained: PathBuf = start_directory(os_arguments(&["-C", "inner", "-C", "."]).as_slice(), root.as_path()).expect("chained");
    assert_eq!(chained, root.join("inner").join("."));
    for prefix in [
        vec!["-c", "core.autocrlf=true"],
        vec!["--git-dir", "x"],
        vec!["--work-tree", "x"],
        vec!["--config-env", "a=B"],
        vec!["-C", "inner/../inner"],
        vec!["-C"],
        vec!["--bare"],
    ] {
        assert_eq!(start_directory(os_arguments(prefix.as_slice()).as_slice(), root.as_path()), None, "{prefix:?}");
    }
    assert_eq!(start_directory(&[], root.as_path()), Some(root.clone()));
    remove(root.as_path());
}

/// The destination of a symbolic link parent is the link's target,
/// as Git records it.
#[test]
fn a_symbolic_link_parent_is_resolved_like_git_does() {
    let root: PathBuf = fixture("add-plan-link");
    std::fs::create_dir(root.join("real")).expect("real");
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).expect("link");
    let plan = plan_of(&["worktree", "add", "link/topic"], 0, root.as_path()).expect("plan");
    assert_eq!(plan.destination, std::fs::canonicalize(root.join("real")).expect("canonical").join("topic"));
    remove(root.as_path());
}

/// Positive control of the table against real Git 2.56.0:
/// each accepted form creates a worktree at the planned
///  destination,
/// and `worktree list` records the path the plan computed.
#[test]
fn real_git_registers_the_planned_destination_for_each_accepted_form() {
    let root: PathBuf = fixture("add-plan-real-git");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir(root.join("space dir")).expect("directory");
    std::os::unix::fs::symlink(root.join("space dir"), root.join("via-link")).expect("link");
    for (position, values) in [
        vec!["worktree", "add", "../one"],
        vec!["worktree", "add", "-b", "feature", "../two", "main"],
        vec!["worktree", "add", "--detach", "--quiet", "../three", "main"],
        vec!["worktree", "add", "../four", "-b", "other"],
        vec!["worktree", "add", "--lock", "--reason", "plan", "../five", "-b", "locked-one"],
        vec!["worktree", "add", "--", "../six"],
        vec!["worktree", "add", "../via-link/seven", "-b", "seventh"],
    ]
    .into_iter()
    .enumerate()
    {
        let plan: AddPlan = plan_of(values.as_slice(), 0, repo.as_path()).unwrap_or_else(|| { panic!("{values:?} is accepted"); });
        let before: String = String::from_utf8_lossy(&git(repo.as_path(), &["worktree", "list", "--porcelain"]).stdout).into_owned();
        assert!(!before.contains(plan.destination.to_string_lossy().as_ref()), "{position}: {values:?}");
        git(repo.as_path(), values.as_slice());
        let listing: Output = git(repo.as_path(), &["worktree", "list", "--porcelain"]);
        let text: String = String::from_utf8_lossy(&listing.stdout).into_owned();
        assert!(text.contains(format!("worktree {}\n", plan.destination.display()).as_str()), "{values:?}: {text}");
    }
    remove(root.as_path());
}

/// A destination that already is a symbolic link is declined,
/// because Git records the path the link resolves to while the
///  snapshot rung would swap the link itself;
/// a name that does not exist yet is served.
#[test]
fn a_destination_that_is_a_link_is_declined() {
    let root: PathBuf = fixture("add-plan-final-link");
    std::fs::create_dir(root.join("real")).expect("real");
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).expect("link");
    assert_eq!(plan_of(&["worktree", "add", "link"], 0, root.as_path()), None);
    assert_eq!(plan_of(&["worktree", "add", "link/"], 0, root.as_path()), None, "a trailing slash names the same link");
    assert!(plan_of(&["worktree", "add", "fresh"], 0, root.as_path()).is_some(), "positive control");
    assert!(plan_of(&["worktree", "add", "real"], 0, root.as_path()).is_some(), "a real directory is Git's to accept");
    remove(root.as_path());
}

/// The destination of a copy is read from every documented form of `git worktree add`,
/// with its options anywhere before or
/// after the operands,
/// bundled,
/// attached or in the next token.
#[test]
fn the_copy_destination_is_read_from_every_documented_form() {
    let root: PathBuf = fixture("copy-destination-forms");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    for (values, index) in [
        (vec!["worktree", "add", "topic"], 0),
        (vec!["worktree", "add", "-f", "topic"], 0),
        (vec!["worktree", "add", "--force", "--no-checkout", "topic", "main"], 0),
        (vec!["worktree", "add", "-B", "feature", "topic"], 0),
        (vec!["worktree", "add", "-bfeature", "topic"], 0),
        (vec!["worktree", "add", "-fb", "feature", "topic"], 0),
        (vec!["worktree", "add", "-dq", "topic"], 0),
        (vec!["worktree", "add", "--orphan", "-b", "feature", "topic"], 0),
        (vec!["worktree", "add", "--track", "--guess-remote", "--relative-paths", "topic"], 0),
        (vec!["worktree", "add", "--no-track", "--no-guess-remote", "topic"], 0),
        (vec!["worktree", "add", "--det", "topic"], 0),
        (vec!["worktree", "add", "--lock", "--reas", "why", "topic"], 0),
        (vec!["worktree", "add", "--lock", "--reas=why", "topic"], 0),
        (vec!["worktree", "add", "--no-che", "--qu", "topic"], 0),
        (vec!["worktree", "add", "--end-of-options", "-odd"], 0),
        (vec!["--git-dir=/g", "--work-tree=/w", "worktree", "add", "topic"], 2),
        (vec!["--git-dir", "/g", "--work-tree", "/w", "--namespace", "n", "worktree", "add", "topic"], 6),
        (vec!["--bare", "--attr-source=HEAD", "worktree", "add", "topic"], 2),
        (vec!["worktree", "add", "--lock", "--reason=why", "topic"], 0),
        (vec!["worktree", "add", "--no-lock", "--no-reason", "--quiet", "--no-quiet", "topic"], 0),
        (vec!["worktree", "add", "topic", "--detach", "main"], 0),
        (vec!["worktree", "add", "--", "-odd"], 0),
        (vec!["-c", "core.x=y", "worktree", "add", "topic"], 2),
        (vec!["--no-optional-locks", "--config-env", "a.b=C", "worktree", "add", "topic"], 3),
    ] {
        let found: PathBuf = parse_copy_destination(os_arguments(values.as_slice()).as_slice(), index, canonical.as_path())
            .unwrap_or_else(|| { panic!("{values:?} names a destination"); });
        let operand: &str = if values.contains(&"-odd") { "-odd" } else { "topic" };
        assert_eq!(found, canonical.join(operand), "{values:?}");
    }
    remove(root.as_path());
}

/// A form Git rejects,
/// or the reading does not know,
/// leaves the destination unknown:
/// an ambiguous or unknown option,
/// a value
/// on a flag,
/// a missing value,
/// a wrong operand count,
/// another word,
/// and an option between `worktree` and `add`.
#[test]
fn a_copy_destination_that_is_not_read_stays_unknown() {
    let root: PathBuf = fixture("copy-destination-unknown");
    for (values, index) in [
        (vec!["worktree", "add", "--detac", "--", "-x", "y", "z"], 0),
        (vec!["worktree", "add", "--re", "topic"], 0),
        (vec!["worktree", "add", "--n", "topic"], 0),
        (vec!["worktree", "add", "--no-", "topic"], 0),
        (vec!["worktree", "add", "--no-no-checkout", "topic"], 0),
        (vec!["worktree", "add", "--detach=yes", "topic"], 0),
        (vec!["worktree", "add", "--track=direct", "topic"], 0),
        (vec!["worktree", "add", "--no-reason=x", "topic"], 0),
        (vec!["worktree", "add", "--reason"], 0),
        (vec!["worktree", "add", "-d=x", "topic"], 0),
        (vec!["worktree", "add", "-h"], 0),
        (vec!["worktree", "add", "--help", "topic"], 0),
        (vec!["worktree", "add", "-b"], 0),
        (vec!["worktree", "add", "topic", "-b"], 0),
        (vec!["worktree", "add", "-bx"], 0),
        (vec!["worktree", "add", "-x", "topic"], 0),
        (vec!["worktree", "add"], 0),
        (vec!["worktree", "add", "a", "b", "c"], 0),
        (vec!["worktree", "move", "a", "b"], 0),
        (vec!["worktree", "--verbose", "add", "topic"], 0),
        (vec!["status", "add", "topic"], 0),
    ] {
        assert_eq!(parse_copy_destination(os_arguments(values.as_slice()).as_slice(), index, root.as_path()), None, "{values:?}");
    }
    remove(root.as_path());
}

/// The directory a destination is read against moves with `-C` and with nothing else;
/// `--git-dir` and `--work-tree` leave it
/// where it is,
/// as Git does,
/// and an option the reading does not know leaves it unknown.
#[test]
fn only_dash_c_capital_moves_the_start_of_a_copy_destination() {
    let start: &Path = Path::new("/work");
    assert_eq!(copy_start_directory(os_arguments(&["-C", "sub", "-C", "deeper"]).as_slice(), start), Some(PathBuf::from("/work/sub/deeper")));
    assert_eq!(copy_start_directory(os_arguments(&["-c", "a.b=c", "--no-pager", "-P"]).as_slice(), start), Some(PathBuf::from("/work")));
    assert_eq!(copy_start_directory(os_arguments(&["--config-env=a.b=C"]).as_slice(), start), Some(PathBuf::from("/work")));
    assert_eq!(copy_start_directory(os_arguments(&["--git-dir=/g", "--work-tree", "/w", "--bare", "--exec-path=/x"]).as_slice(), start), Some(PathBuf::from("/work")));
    assert_eq!(copy_start_directory(os_arguments(&["-C", ""]).as_slice(), start), Some(PathBuf::from("/work")));
    for refused in [vec!["--unknown"], vec!["--git-dir"], vec!["-C"], vec!["-c"], vec!["--exec-path"]] {
        assert_eq!(copy_start_directory(os_arguments(refused.as_slice()).as_slice(), start), None, "{refused:?}");
    }
}

/// Real Git 2.56.0 registers the destination the copy reading names for each form,
/// so the reading and Git agree.
#[test]
fn real_git_registers_the_destination_the_copy_reading_names() {
    let root: PathBuf = fixture("copy-destination-real");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    for (index, values) in [
        vec!["worktree", "add", "-fb", "one", "../one"],
        vec!["worktree", "add", "-bfeature", "../two"],
        vec!["worktree", "add", "--orphan", "-b", "three", "../three"],
        vec!["worktree", "add", "--detach", "--no-checkout", "../four", "main"],
    ].into_iter().enumerate() {
        let start: PathBuf = std::fs::canonicalize(repo.as_path()).expect("canonical repo");
        let named: PathBuf = parse_copy_destination(os_arguments(values.as_slice()).as_slice(), 0, start.as_path()).unwrap_or_else(|| { panic!("{values:?}"); });
        git(repo.as_path(), values.as_slice());
        assert!(named.join(".git").exists(), "{index}: Git made {}", named.display());
        assert_eq!(named.parent(), Some(canonical.as_path()));
    }
    remove(root.as_path());
}

/// Long options resolve as Git's `parse_long_opt` does:
/// an exact name wins over longer names,
/// a unique prefix is the option,
/// the `--no-` form negates,
/// and two candidates,
/// a prefix of `no-` and a double negation resolve to nothing.
#[test]
fn a_long_option_resolves_the_way_gits_parser_does() {
    let name = |text: &str| -> Option<(&'static str, bool, Option<String>)> {
        return resolve_long(text).map(|found| { return (found.option.long.unwrap_or("-"), found.unset, found.attached.map(String::from)); });
    };
    assert_eq!(name("detach"), Some(("detach", false, None)));
    assert_eq!(name("det"), Some(("detach", false, None)), "a unique prefix");
    assert_eq!(name("d"), Some(("detach", false, None)));
    assert_eq!(name("no-che"), Some(("checkout", true, None)), "a negated prefix");
    assert_eq!(name("no-checkout"), Some(("checkout", true, None)));
    assert_eq!(name("reas=why"), Some(("reason", false, Some(String::from("why")))), "a prefix with its value");
    assert_eq!(name("reason="), Some(("reason", false, Some(String::new()))), "an empty value is a value");
    assert_eq!(name("re"), None, "reason and relative-paths");
    assert_eq!(name("rel"), Some(("relative-paths", false, None)));
    assert_eq!(name("f"), Some(("force", false, None)));
    assert_eq!(name("g"), Some(("guess-remote", false, None)));
    assert_eq!(name("l"), Some(("lock", false, None)));
    assert_eq!(name("o"), Some(("orphan", false, None)));
    assert_eq!(name("t"), Some(("track", false, None)));
    assert_eq!(name("n"), None, "a prefix of no- abbreviates every negation");
    assert_eq!(name("no"), None);
    assert_eq!(name("no-"), None);
    assert_eq!(name("no-no-checkout"), None, "a double negation");
    assert_eq!(name("nonsense"), None);
    assert_eq!(name("detachx"), None, "a longer text than the name is not a prefix");
    assert_eq!(name("c"), Some(("checkout", false, None)));
}

/// A value on a flag or on a negated option,
/// and a missing value,
/// are errors in Git and leave the operands unread.
#[test]
fn values_are_taken_as_gits_parser_takes_them() {
    let region = |values: &[&str]| -> Option<usize> { return copy_add_operands(os_arguments(values).as_slice()).map(|operands| { return operands.len(); }); };
    assert_eq!(region(&["--reason", "why", "topic"]), Some(1), "a value in the next token");
    assert_eq!(region(&["--no-reason", "topic"]), Some(1), "a negated option takes none");
    assert_eq!(region(&["--no-reason=x", "topic"]), None);
    assert_eq!(region(&["--quiet=1", "topic"]), None, "a flag takes no value");
    assert_eq!(region(&["-qf", "topic"]), Some(1));
    assert_eq!(region(&["-fbname", "topic"]), Some(1), "the rest of a token is the value");
    assert_eq!(region(&["-fb", "name", "topic"]), Some(1), "the next token is the value");
    assert_eq!(region(&["-fb"]), None);
    assert_eq!(region(&["topic", "--detach", "main"]), Some(2), "options may follow operands");
    assert_eq!(region(&["--", "--detach", "x"]), Some(2), "after -- everything is an operand");
}

/// A destination whose parent directories do not exist yet is the real path of the nearest existing ancestor and the rest as
/// written;
/// a link,
/// a `..` below the missing part and a bare `.` are declined.
#[test]
fn a_destination_below_missing_directories_is_read_without_them() {
    let root: PathBuf = fixture("copy-destination-missing");
    let canonical: PathBuf = std::fs::canonicalize(root.as_path()).expect("canonical");
    let link: PathBuf = root.join("link");
    std::os::unix::fs::symlink(canonical.join("real"), link.as_path()).expect("link");
    std::fs::create_dir(canonical.join("real")).expect("real");
    let read = |operand: &str| -> Option<PathBuf> { return creating_destination(root.as_path(), OsStr::new(operand)); };
    assert_eq!(read(".claude/worktrees/topic"), Some(canonical.join(".claude/worktrees/topic")));
    assert_eq!(read("real/inner/topic"), Some(canonical.join("real/inner/topic")));
    assert_eq!(read("link/inner/topic"), Some(canonical.join("real/inner/topic")), "a link in the existing part resolves");
    assert_eq!(read("link"), None, "a destination that is a link is declined");
    assert_eq!(read("missing/../topic"), None);
    assert_eq!(read("missing/./topic"), Some(canonical.join("missing/topic")));
    assert_eq!(read("."), None);
    assert_eq!(read("a/.."), None);
    remove(root.as_path());
}

/// The table of options is the list `git worktree add -h` prints,
/// so it cannot drift from Git:
/// every option Git lists is in
/// the table with the same letter,
/// name and value,
/// and the table lists nothing Git does not.
#[test]
fn the_option_table_matches_what_git_prints_for_dash_h() {
    // `git worktree` sets up a repository before it reads `-h`, so it runs inside a fixture repository: outside one, as in
    // the container gate's working directory, Git stops with "not a git repository" and prints no usage.
    let root: PathBuf = fixture("add-plan-dash-h");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let output: Output = std::process::Command::new(crate::test_support::REAL_GIT)
        .args(["worktree", "add", "-h"])
        .current_dir(repo.as_path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("git runs");
    remove(root.as_path());
    // `-h` prints the usage on standard output with status 129, and the option lines follow a blank line.
    let text: String = String::from_utf8_lossy(output.stdout.as_slice()).into_owned() + String::from_utf8_lossy(output.stderr.as_slice()).as_ref();
    let listed: Vec<(Option<u8>, Option<String>, bool)> = text
        .lines()
        .skip_while(|line| { return !line.is_empty(); })
        .filter(|line| { return line.starts_with("    -"); })
        .map(|line| {
            // The spelling is dash words and `<placeholder>` words; the description starts at the first other word.
            let mut short: Option<u8> = None;
            let mut long: Option<String> = None;
            let mut takes: bool = false;
            for word in line.split_whitespace() {
                let head: &str = word.trim_end_matches(',');
                if let Some(rest) = head.strip_prefix("--") {
                    long = Some(rest.replace("[no-]", ""));
                } else if let Some(letter) = head.strip_prefix('-') {
                    short = letter.bytes().next();
                } else if head.starts_with('<') {
                    takes = true;
                } else {
                    break;
                }
            }
            return (short, long, takes);
        })
        .collect();
    let table: Vec<(Option<u8>, Option<String>, bool)> = ADD_OPTIONS
        .iter()
        .map(|option| { return (option.short, option.long.map(String::from), option.takes == Takes::Required); })
        .collect();
    assert!(!listed.is_empty(), "the usage lists options: {text}");
    for entry in &listed {
        assert!(table.contains(entry), "Git lists {entry:?}, which the table lacks: {text}");
    }
    for entry in &table {
        assert!(listed.contains(entry), "the table lists {entry:?}, which Git does not: {text}");
    }
}

/// An exact name wins over a longer name it is a prefix of,
/// even when the longer name comes first;
/// the real table has no such
/// pair,
/// so a table that has one shows the rule.
#[test]
fn an_exact_name_wins_over_the_names_it_is_a_prefix_of() {
    static TABLE: &[AddOption] = &[
        AddOption { short: None, long: Some("reason"), takes: Takes::Nothing },
        AddOption { short: None, long: Some("re"), takes: Takes::Required },
        AddOption { short: None, long: Some("reset"), takes: Takes::Nothing },
    ];
    let exact = resolve_in(TABLE, "re").expect("the exact name");
    assert_eq!(exact.option.long, Some("re"));
    assert!(resolve_in(TABLE, "res").is_some_and(|prefix| { return prefix.option.long == Some("reset"); }), "a unique prefix");
    assert!(resolve_in(TABLE, "rea").is_some_and(|prefix| { return prefix.option.long == Some("reason"); }));
    assert!(resolve_in(TABLE, "r").is_none(), "three names share the prefix");
    assert!(resolve_in(TABLE, "re=1").is_some_and(|valued| { return valued.attached == Some("1"); }));
}
