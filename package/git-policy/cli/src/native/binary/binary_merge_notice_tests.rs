//! What:
//!  The lines a merge prints about a path it kept whole,
//! and the commands inside them:
//! the person reads these,
//! and runs what they name.
//! Why:
//!  A merge that carries the person's work exits 0,
//! so the line is the whole account of where their bytes are
//! (`SPEC.md`,
//! "Merges and commits never fail";
//! the owner's "never discard user data").
//!  A path whose name holds a space,
//! a command that stages over a version the person staged,
//! and a line about a path that a save filled after the carry judged it
//! each cost the person the bytes the line was written to keep
//! (review `merge-notice-for-a-deleted-file` round 133,
//! findings 1 to 3).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ran = await wrapped(['merge', '--no-edit', 'feature']);
//! const ended = await sh(theLinesCommand(ran.stderr)); expect(ended.code).toBe(0);
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::held_lock_support::PATIENCE;
use super::merge_carries_edits::{assert_merged, diverged, run, said, text_at};
use super::support::{Fixture, Observed, bounded, fixture, git, observe, repository, wrapper_first_path};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The text of the first command a line puts between backticks,
/// with the program word the line names,
/// because a shell runs it as it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theCommandAsPrinted = (words) => words.slice(words.indexOf('`') + 1).split('`')[0];
/// ```
fn the_command_as_printed(words: &str) -> String {
    let opened: usize = words.find('`').expect("the line names a command between backticks") + 1;
    let rest: &str = &words[opened..];
    let closed: usize = rest.find('`').expect("the command's closing backtick is there");
    return String::from(&rest[..closed]);
}

/// What:
///  Run the first command a line prints,
/// through a shell that reads it exactly as the person's shell would.
/// Why:
///   A control that split the command on spaces itself would agree with a line a shell splits differently,
/// so the shell is what decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = observe(bounded(fixture, 'sh', wrapperFirstPath).args(['-c', theCommandAsPrinted(words)]));
/// ```
fn run_as_printed(fixture: &Fixture, repo: &Path, words: &str) -> Observed {
    let mut command: Command = bounded(fixture, Path::new("sh"), wrapper_first_path(fixture).as_os_str());
    command.current_dir(repo).args(["-c", the_command_as_printed(words).as_str()]);
    return observe(&mut command, b"");
}

/// What:
///  A repository whose `feature` deletes `gone with space.txt`,
/// and whose `main` changes another file.
/// Why:
///  The path is the finding's own:
/// a name a shell reads as one word only when it is quoted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// base({ 'gone with space.txt': 'base\n', 'main.txt': 'base\n' }); branch('feature', delete('gone with space.txt'));
/// ```
fn a_repository_that_deletes_a_path_with_a_space(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("gone with space.txt"), b"base\n").expect("the path with a space");
    std::fs::write(repo.join("main.txt"), b"base\n").expect("main.txt");
    git(fixture, repo.as_path(), &["add", "--", "gone with space.txt", "main.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    git(fixture, repo.as_path(), &["rm", "--quiet", "--", "gone with space.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("main.txt"), b"base\nmain\n").expect("main.txt");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    return repo;
}

/// What:
///  A path the merge deletes whose name holds a space,
/// with the person's own edit in it,
/// prints a command that a shell reads as that one path:
///  run as printed,
/// it takes the deletion of that path and of nothing else.
/// Why:
///  The line's command is advice the person pastes,
/// and an argument a shell splits into three
///  deletes three files the merge never named:
/// `rm -- gone with space.txt` removed `gone`,
/// `with` and `space.txt` and left the
///  intended file where it was
/// (review `merge-notice-for-a-deleted-file` round 133,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('gone with space.txt', person); write('gone', a); write('with', b); write('space.txt', c);
/// const ran = await wrapped(['merge', '--no-edit', 'feature']); expect(await sh(command(ran.stderr))).toSucceed();
/// expect(exists('gone')).toBe(true); expect(exists('gone with space.txt')).toBe(false);
/// ```
#[test]
fn a_path_with_a_space_is_one_word_in_the_command_the_line_prints() {
    let fixture: Fixture = fixture("merge-notice-path-with-space");
    let repo: PathBuf = a_repository_that_deletes_a_path_with_a_space(&fixture, "repo");
    let person: &str = "base\nthe person's own line\n";
    std::fs::write(repo.join("gone with space.txt"), person.as_bytes()).expect("the person's edit");
    let unrelated: [(&str, &str); 3] = [
        ("gone", "unrelated user data: gone\n"),
        ("with", "unrelated user data: with\n"),
        ("space.txt", "unrelated user data: space.txt\n"),
    ];
    for (name, text) in unrelated {
        std::fs::write(repo.join(name), text.as_bytes()).expect("a file the merge never names");
    }
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "path with a space");
    let words: String = said(&observed);
    let printed: String = the_command_as_printed(words.as_str());
    assert_eq!(printed, "rm -- 'gone with space.txt'", "the command names the path as one shell word: {words}");
    let taken: Observed = run_as_printed(&fixture, repo.as_path(), words.as_str());
    assert_eq!(taken.code, Some(0), "the command the line printed runs: {}", said(&taken));
    for (name, text) in unrelated {
        assert_eq!(text_at(repo.as_path(), name).unwrap_or_default(), text, "the command deleted {name}, which the merge never named");
    }
    assert_eq!(text_at(repo.as_path(), "gone with space.txt"), None, "the command takes the deletion of the path it names");
    remove_settled(&fixture);
}

/// What:
///  A path the person staged one version of and saved another one over,
/// when the merge deletes it:
/// both layers stand after the merge,
///  and no line tells the person to run `git add`,
/// which would put the checkout's version in the index
///  in place of the staged one.
/// Why:
///  The capture's ref goes when the merge lands,
/// so the staged version stands only in the index;
///  `git add` then left the index holding the later version,
/// and the staged version was gone
/// (review `merge-notice-for-a-deleted-file` round 133,
/// finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(staged); add(path); write(later); const ran = await wrapped(['merge', '--no-edit', 'feature']);
/// expect(indexText(path)).toBe(staged); expect(worktreeText(path)).toBe(later); expect(ran.stderr).not.toSay('git add -- gone.txt');
/// ```
#[test]
fn a_staged_version_and_a_later_save_both_stand_when_the_merge_deletes_the_path() {
    let fixture: Fixture = fixture("merge-notice-staged-layers");
    let repo: PathBuf = diverged(&fixture, "repo");
    let staged: &str = "gone base\nthe distinct staged version\n";
    let later: &str = "gone base\nthe later version in the checkout\n";
    std::fs::write(repo.join("gone.txt"), staged.as_bytes()).expect("the person's staged version");
    git(&fixture, repo.as_path(), &["add", "--", "gone.txt"]);
    std::fs::write(repo.join("gone.txt"), later.as_bytes()).expect("the person's later save");
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", said(&observed));
    assert_merged(&fixture, repo.as_path(), "staged layers the merge deletes");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", ":gone.txt"]), staged, "the staged version stands in the index");
    assert_eq!(text_at(repo.as_path(), "gone.txt").unwrap_or_default(), later, "the later save stands in the checkout");
    let words: String = said(&observed);
    assert!(words.contains("gone.txt"), "the line names the path: {words}");
    assert!(!words.contains("keeps your bytes in the index instead"), "no line tells the person to stage over the staged version: {words}");
    assert!(words.contains("would replace the version the index holds"), "the line says why `git add` is not the command to run: {words}");
    remove_settled(&fixture);
}

/// What:
///  A save that fills a path the merge deletes while the carry is writing leaves one line about the path,
/// and that line says the merge wrote nothing to it.
/// Why:
///  The carry judged the path empty and built its line from that judgement before it wrote anything,
/// so the save
///  left the line saying the path holds nothing of the person's and that a deletion of theirs stands,
/// beside the line saying
///  the save was kept (review `merge-notice-for-a-deleted-file` round 133,
/// finding 3):
/// two lines that disagree about
///  the checkout the command leaves,
/// and the first offers a `git restore` over the newer bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm('a.txt'); const paused = startMerge('merge-carry-index:pause'); await marker; write('a.txt', saved); release;
/// expect(read('a.txt')).toBe(saved); expect(stderr).toSay('changed while the merge ran'); expect(stderr).not.toSay('holds nothing of yours');
/// ```
#[test]
fn a_save_that_fills_a_kept_path_leaves_one_line_about_it() {
    let fixture: Fixture = fixture("merge-notice-save-while-carrying");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::remove_file(repo.join("a.txt")).expect("the person's deletion");
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    command.env(PHASE_SIGNAL_VARIABLE, format!("merge-carry-index:pause:{}", markers.display()));
    let child = command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("the merge starts");
    let began = std::time::Instant::now();
    while !markers.join("merge-carry-index.reached").exists() {
        assert!(began.elapsed() < PATIENCE, "the merge never reached the carry's index write");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let saved: &[u8] = b"a version saved while the carry was writing\n";
    std::fs::write(repo.join("a.txt"), saved).expect("the save");
    std::fs::write(markers.join("merge-carry-index.release"), b"").expect("release the merge");
    let output = child.wait_with_output().expect("the merge ends");
    let words: String = String::from_utf8_lossy(output.stderr.as_slice()).into_owned();
    assert_eq!(output.status.code(), Some(0), "{words}");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("a.txt"), saved, "the save stands");
    assert_merged(&fixture, repo.as_path(), "save while the carry writes");
    assert!(words.contains("a.txt changed while the merge ran"), "the line says the merge wrote nothing to it: {words}");
    assert!(!words.contains("holds nothing of yours"), "no line says the path is empty: {words}");
    assert!(!words.contains("a deletion of yours stands"), "no line says a deletion stands at the path: {words}");
    assert!(!words.contains("git restore --no-enforce-worktree"), "no line sends the person over the save: {words}");
    remove_settled(&fixture);
}
