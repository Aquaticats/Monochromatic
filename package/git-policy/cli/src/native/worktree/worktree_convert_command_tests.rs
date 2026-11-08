//! The command over several worktrees:
//! `--all`,
//! a dry run that lists reasons,
//! exit statuses and the lines printed.

use super::*;
use crate::subvolume_test_support::{PlainPlatform, backdate, checkout_moved_behind_a_link, checkout_with_worktree};
use crate::test_support::{git, remove};

/// A checkout with two quiet linked worktrees,
/// `wt` and `second`,
/// both inside it.
fn two_worktrees(name: &str) -> (PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "second"]);
    let exclude: PathBuf = root.join(".git/info/exclude");
    let mut text: String = std::fs::read_to_string(&exclude).unwrap_or_default();
    text.push_str("/second/\n");
    std::fs::write(&exclude, text).expect("exclude");
    for worktree in ["wt", "second"] {
        backdate(root.join(worktree).as_path());
        git(root.join(worktree).as_path(), &["update-index", "--refresh"]);
    }
    return (parent, root);
}

/// Run the command and return its status and lines.
fn run(platform: &(dyn Platform + Sync), arguments: &ConvertArguments, cwd: &Path) -> (i32, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let code: i32 = execute(platform, arguments, cwd, &Interrupt::never(), &mut |text: &str| { lines.push(String::from(text)); });
    return (code, lines);
}

/// `--all --dry-run` lists every worktree with what it would do or why it is skipped,
/// and changes nothing;
/// the real run
///  then converts both,
/// and a second run reports them as subvolumes.
#[test]
fn all_worktrees_are_listed_then_converted() {
    let (parent, root) = two_worktrees("convert-command-all");
    let platform: PlainPlatform = PlainPlatform::new();
    let dry: ConvertArguments = ConvertArguments { dry_run: true, all: true, ..ConvertArguments::default() };
    std::fs::write(root.join(".git/worktrees/second/locked"), b"").expect("lock the second");
    let (code, lines) = run(&platform, &dry, root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    assert!(lines.iter().any(|line: &String| { return line.trim() == "would convert"; }), "{lines:#?}");
    assert!(lines.iter().any(|line: &String| { return line.contains("skipped:") && line.contains("locked"); }), "{lines:#?}");
    assert!(!platform.is_subvolume_root(root.join("wt").as_path()).expect("examined"), "a dry run converts nothing");
    std::fs::remove_file(root.join(".git/worktrees/second/locked")).expect("unlock");
    let real: ConvertArguments = ConvertArguments { all: true, ..ConvertArguments::default() };
    let (real_code, real_lines) = run(&platform, &real, root.as_path());
    assert_eq!(real_code, 0, "{real_lines:#?}");
    assert!(platform.is_subvolume_root(root.join("wt").as_path()).expect("examined"));
    assert!(platform.is_subvolume_root(root.join("second").as_path()).expect("examined"));
    let (again_code, again_lines) = run(&platform, &real, root.as_path());
    assert_eq!(again_code, 0);
    assert_eq!(again_lines.iter().filter(|line: &&String| { return line.contains("already a subvolume"); }).count(), 2, "{again_lines:#?}");
    remove(parent.as_path());
}

/// A named worktree that is refused ends the command with status 1 and a reason;
/// a path that is no worktree is refused too.
#[test]
fn a_refused_named_worktree_is_status_one_and_a_failure_is_status_two() {
    let (parent, root) = two_worktrees("convert-command-named");
    let platform: PlainPlatform = PlainPlatform::new();
    std::fs::write(root.join(".git/worktrees/wt/index.lock"), b"another git").expect("lock");
    let named: ConvertArguments = ConvertArguments { paths: vec![std::ffi::OsString::from("wt")], ..ConvertArguments::default() };
    let (code, lines) = run(&platform, &named, root.as_path());
    assert_eq!(code, REFUSED_EXIT_CODE, "{lines:#?}");
    assert!(lines.iter().any(|line: &String| { return line.contains("index.lock"); }), "{lines:#?}");
    let missing: ConvertArguments = ConvertArguments { paths: vec![std::ffi::OsString::from("nowhere")], ..ConvertArguments::default() };
    let (missing_code, missing_lines) = run(&platform, &missing, root.as_path());
    assert_eq!(missing_code, REFUSED_EXIT_CODE, "{missing_lines:#?}");
    std::fs::remove_file(root.join(".git/worktrees/wt/index.lock")).expect("unlock");
    let killed: i32 = execute(&platform, &named, root.as_path(), &Interrupt::at("created", 0), &mut |_text: &str| {});
    assert_eq!(killed, FAILED_EXIT_CODE);
    let (finished_code, finished_lines) = run(&platform, &named, root.as_path());
    assert_eq!(finished_code, 0, "{finished_lines:#?}");
    remove(parent.as_path());
}

/// Help is status 0 on standard output.
#[test]
fn help_is_printed() {
    let action: Action = convert_action(&ConvertArguments { help: true, ..ConvertArguments::default() }, Path::new("."), &[], None);
    assert!(matches!(action, Action::Exit { code: 0, ref stdout, .. } if stdout.contains("--abandon")));
}

/// A real run says the risk once,
/// before it touches the first worktree;
/// a dry run,
/// an abandon and an empty list say nothing.
#[test]
fn the_risk_is_said_once_before_anything_is_converted() {
    let (parent, root) = two_worktrees("convert-command-risk");
    let platform: PlainPlatform = PlainPlatform::new();
    let real: ConvertArguments = ConvertArguments { all: true, ..ConvertArguments::default() };
    let (code, lines) = run(&platform, &real, root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    assert_eq!(lines.iter().filter(|line: &&String| { return line.as_str() == RISK_NOTICE; }).count(), 1, "said once: {lines:#?}");
    assert_eq!(lines.first().map(String::as_str), Some(RISK_NOTICE), "before the first worktree is named");
    let dry: ConvertArguments = ConvertArguments { dry_run: true, all: true, ..ConvertArguments::default() };
    let (_dry_code, dry_lines) = run(&platform, &dry, root.as_path());
    assert!(!dry_lines.iter().any(|line: &String| { return line.as_str() == RISK_NOTICE; }), "a dry run removes nothing");
    remove(parent.as_path());
}

/// An attribute source chosen on the command line reaches the conversion:
/// it refuses to decide by content.
#[test]
fn a_command_line_attribute_source_stops_the_content_shortcut() {
    let (parent, root) = two_worktrees("convert-command-source");
    let platform: PlainPlatform = PlainPlatform::new();
    // A stale stat that only content can vouch for.
    let readme: PathBuf = root.join("wt/README.md");
    let bytes: Vec<u8> = std::fs::read(&readme).expect("read");
    std::fs::write(&readme, bytes).expect("rewrite");
    let named: ConvertArguments = ConvertArguments { dry_run: true, paths: vec![std::ffi::OsString::from("wt")], command_line_attributes: true, ..ConvertArguments::default() };
    let (_code, lines) = run(&platform, &named, root.as_path());
    assert!(lines.iter().any(|line: &String| { return line.contains("--attr-source") && line.contains("cannot be ruled out"); }), "{lines:#?}");
    let without: ConvertArguments = ConvertArguments { command_line_attributes: false, ..named };
    let (_plain_code, plain_lines) = run(&platform, &without, root.as_path());
    assert!(plain_lines.iter().any(|line: &String| { return line.trim() == "would convert"; }), "{plain_lines:#?}");
    remove(parent.as_path());
}

/// `--all` covers a worktree whose registration spells the repository's old name through a link,
/// and says how many registrations it left out and why.
#[test]
fn all_covers_a_link_spelled_registration_and_says_what_it_leaves_out() {
    let (parent, root, worktree) = checkout_moved_behind_a_link("convert-command-link");
    let platform: PlainPlatform = PlainPlatform::new();
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "../outside"]);
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "gone"]);
    std::fs::remove_dir_all(root.join("gone")).expect("the directory Git would prune");
    let dry: ConvertArguments = ConvertArguments { dry_run: true, all: true, ..ConvertArguments::default() };
    let (code, lines) = run(&platform, &dry, root.as_path());
    assert_eq!(code, 0, "{lines:#?}");
    let start: usize = lines.iter().position(|line: &String| { return line == &worktree.display().to_string(); }).unwrap_or_else(|| panic!("the link-spelled worktree is covered: {lines:#?}"));
    assert!(lines.get(start + 1).is_some_and(|line: &String| { return line.trim() == "would convert"; }), "the link-spelled worktree is covered: {lines:#?}");
    assert!(lines.iter().any(|line: &String| { return line.contains("left out") && line.contains("outside") && line.contains("outside the main checkout"); }), "{lines:#?}");
    assert!(lines.iter().any(|line: &String| { return line.contains("left out") && line.contains("gone") && line.contains("is gone"); }), "{lines:#?}");
    assert!(lines.last().is_some_and(|line: &String| { return line.contains("2 registered worktree(s) left out"); }), "{lines:#?}");
    let real: ConvertArguments = ConvertArguments { all: true, ..ConvertArguments::default() };
    let (real_code, real_lines) = run(&platform, &real, root.as_path());
    assert_eq!(real_code, 0, "{real_lines:#?}");
    assert!(platform.is_subvolume_root(worktree.as_path()).expect("examined"), "{real_lines:#?}");
    assert!(real_lines.last().is_some_and(|line: &String| { return line.contains("1 converted or completed") && line.contains("2 registered worktree(s) left out"); }), "{real_lines:#?}");
    remove(parent.as_path());
}
