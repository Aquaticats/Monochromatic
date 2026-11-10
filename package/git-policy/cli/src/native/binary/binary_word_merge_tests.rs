//! What:
//!  The word-level second merge pass through the built executable:
//! after a merge,
//! rebase,
//! cherry-pick or revert that
//!  Git stops with a conflict,
//! the files whose every conflicted region resolves by word are written and staged,
//! each
//!  merged part is listed on standard error and in the journal,
//! and everything else stays as Git left it.
//! Why:
//!  The owner decided on 2026-11-03 that the wrapper re-merges only the conflicted parts word by word,
//! accepts a file only
//!  when every part resolves,
//! and lists each part (`doc/planning/cli-git-merge-auto-resolution.md`).
//! A person's own
//!  edits of a conflicted file,
//! and every file that does not resolve,
//! must be left exactly as Git wrote them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stopped = await wrapped(['merge', '--no-edit', 'topic']);
//! expect(await readFile('notes.md')).toEqual(expectedMerge); expect(stopped.stderr).toContain('word-level merge');
//! ```

use super::support::{Fixture, Observed, fixture, git, observe, remove, repository, run_direct, run_wrapped, stderr_of, wrapped};
use super::word_merge_pass_gate::PassGate;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The words every line about a merged part carries,
/// so a reader can find them in a long output.
pub(super) const MERGE_LINE_MARK: &str = "word-level merge";

/// The corpus input whose two sides edit a line and insert a line right after it:
/// Git conflicts,
/// the word pass resolves.
pub(super) const ADJACENT: &str = "md1b";

/// The corpus input whose two sides append different sections at one point:
/// the word pass leaves it a conflict.
pub(super) const BOTH_APPEND: &str = "md2";

/// The bytes of one stored corpus file.
pub(super) fn corpus(name: &str, extension: &str) -> Vec<u8> {
    return std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/merge/word_merge_corpus").join(format!("{name}.{extension}")))
        .expect("a stored corpus file");
}

/// The path a corpus input is tracked under.
pub(super) fn tracked(name: &str) -> String {
    return format!("{name}.md");
}

/// Which operation the control stops in.
#[derive(Clone, Copy, Debug)]
pub(super) enum Stop {
    /// `git merge --no-edit topic` on `main`.
    Merge,
    /// `git rebase main` on `topic`.
    Rebase,
    /// `git cherry-pick topic` on `main`.
    CherryPick,
    /// `git revert --no-edit HEAD~1` on `main`.
    Revert,
}

/// Every operation,
/// so a control covers all four.
pub(super) const STOPS: [Stop; 4] = [Stop::Merge, Stop::Rebase, Stop::CherryPick, Stop::Revert];

/// A repository with the inputs tracked on `main`,
/// ready for `stop`:
/// the sides of each input are committed on `main`
/// (ours) and `topic` (theirs),
/// except for a revert,
/// which reverts the edit made on `main` after the insertion landed.
pub(super) fn prepared(fixture: &Fixture, stop: Stop, names: &[&str]) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let write = |extension: &str| {
        for name in names {
            std::fs::write(repo.join(tracked(name)), corpus(name, extension)).expect("an input");
        }
    };
    let commit_all = |message: &str| {
        git(fixture, repo.as_path(), &["add", "--all"]);
        git(fixture, repo.as_path(), &["commit", "--quiet", format!("--message={message}").as_str()]);
    };
    write("base");
    commit_all("base");
    if matches!(stop, Stop::Revert) {
        write("ours");
        commit_all("the edit that is reverted");
        // The later commit holds the insertion right after the edited line.
        for name in names {
            std::fs::write(repo.join(tracked(name)), corpus(name, "word.out")).expect("the later state");
        }
        commit_all("the insertion after the edit");
        return repo;
    }
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    write("theirs");
    commit_all("theirs");
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    write("ours");
    commit_all("ours");
    if matches!(stop, Stop::Rebase) {
        git(fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
    }
    return repo;
}

/// The arguments of the operation.
pub(super) fn arguments_of(stop: Stop) -> Vec<&'static str> {
    return match stop {
        Stop::Merge => vec!["merge", "--no-edit", "topic"],
        Stop::Rebase => vec!["rebase", "main"],
        Stop::CherryPick => vec!["cherry-pick", "topic"],
        Stop::Revert => vec!["revert", "--no-edit", "HEAD~1"],
    };
}

/// What the resolved file holds:
/// the revert keeps the insertion and undoes the edit,
/// so it ends at the insertion side.
pub(super) fn expected_after(stop: Stop, name: &str) -> Vec<u8> {
    return match stop {
        Stop::Revert => corpus(name, "theirs"),
        _ => corpus(name, "word.out"),
    };
}

/// The paths Git lists as unmerged.
pub(super) fn unmerged(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let listed = git(fixture, repo, &["ls-files", "--unmerged", "-z"]);
    let mut paths: Vec<String> = listed
        .stdout
        .split(|byte| { return *byte == 0; })
        .filter_map(|entry| { return entry.split(|byte| { return *byte == b'\t'; }).nth(1); })
        .map(|path| { return String::from_utf8_lossy(path).into_owned(); })
        .collect();
    paths.dedup();
    return paths;
}

/// The lines of standard error about merged parts.
pub(super) fn merge_lines(observed: &Observed) -> Vec<String> {
    return stderr_of(observed).lines().filter(|line| { return line.contains(MERGE_LINE_MARK); }).map(String::from).collect();
}

/// The blob names a part line gives,
/// in the order it gives them.
fn blobs_named(line: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut words = line.split_whitespace();
    while let Some(word) = words.next() {
        if word == "blob"
            && let Some(oid) = words.next()
        {
            found.push(oid.trim_end_matches(',').to_owned());
        }
    }
    return found;
}

/// The entries that reached a journal stand-in,
/// as field maps.
fn journal_entries(receiver: &std::os::unix::net::UnixDatagram) -> Vec<std::collections::BTreeMap<String, String>> {
    let mut entries: Vec<std::collections::BTreeMap<String, String>> = Vec::new();
    let mut buffer: Vec<u8> = vec![0; 65_536];
    receiver.set_nonblocking(true).expect("non-blocking");
    while let Ok(length) = receiver.recv(buffer.as_mut_slice()) {
        entries.push(
            String::from_utf8_lossy(&buffer[..length])
                .lines()
                .filter_map(|line: &str| { return line.split_once('=').map(|(field, value): (&str, &str)| { return (field.to_string(), value.to_string()); }); })
                .collect(),
        );
    }
    return entries;
}

/// Run the operation through the wrapper with a journal stand-in;
/// the receiver is returned with what the caller saw.
fn run_with_journal(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> (Observed, std::os::unix::net::UnixDatagram) {
    use std::os::linux::net::SocketAddrExt;
    let name: String = format!("cli-git-test-journal-word-merge-{}-{}", std::process::id(), fixture.root.display().to_string().len());
    let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract address");
    let receiver = std::os::unix::net::UnixDatagram::bind_addr(&address).expect("bind the journal stand-in");
    let observed: Observed = observe(
        wrapped(fixture).env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{name}")).current_dir(repo).args(arguments),
        b"",
    );
    return (observed, receiver);
}

/// A command line with an option the finish does not read (an abbreviation of `--no-commit` here) leaves the operation stopped:
///  the file is staged as resolved,
/// Git's status stays,
/// and no merge commit is made.
#[test]
fn an_option_the_finish_does_not_know_leaves_the_operation_stopped_with_the_file_staged() {
    let fixture: Fixture = fixture("word-merge-unknown-option");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let head: Vec<u8> = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout;
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "--no-comm", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(1), "{said}");
    assert_eq!(unmerged(&fixture, repo.as_path()), Vec::<String>::new(), "{said}");
    assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == corpus(ADJACENT, "word.out"), "{said}");
    assert_eq!(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout, head, "no commit is made when an option is not read");
    assert!(merge_lines(&observed).iter().any(|line| { return line.contains(tracked(ADJACENT).as_str()); }), "{said}");
    remove(&fixture);
}

/// The journal receives one entry per merged file that names the file,
/// the part and the operation.
#[test]
fn the_journal_receives_each_merged_part() {
    let fixture: Fixture = fixture("word-merge-journal");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let (observed, receiver) = run_with_journal(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let entries = journal_entries(&receiver);
    let merged: Vec<_> = entries.iter().filter(|entry| { return entry.get("MESSAGE").is_some_and(|message| { return message.contains(MERGE_LINE_MARK); }); }).collect();
    assert_eq!(merged.len(), 1, "one entry for the one merged file: {entries:?} {}", stderr_of(&observed));
    let entry = merged[0];
    assert_eq!(entry.get("SYSLOG_IDENTIFIER").map(String::as_str), Some("monochromatic-git-policy-cli"), "{entry:?}");
    assert!(entry.get("MESSAGE").is_some_and(|message| { return message.contains(tracked(ADJACENT).as_str()); }), "{entry:?}");
    assert_eq!(entry.get("CLI_GIT_WORD_MERGE_FILE").map(String::as_str), Some(tracked(ADJACENT).as_str()), "{entry:?}");
    remove(&fixture);
}

/// A conflicted file that does not resolve stays exactly as Git wrote it,
/// beside a file that does.
#[test]
fn a_file_that_does_not_resolve_stays_conflicted_beside_one_that_does() {
    let fixture: Fixture = fixture("word-merge-beside");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT, BOTH_APPEND]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let said: String = stderr_of(&observed);
    assert_eq!(unmerged(&fixture, repo.as_path()), vec![tracked(BOTH_APPEND)], "{said}");
    let marked: String = String::from_utf8(std::fs::read(repo.join(tracked(BOTH_APPEND))).expect("the file")).expect("UTF-8");
    assert!(marked.contains("<<<<<<<") && marked.contains(">>>>>>>"), "Git's markers stay: {marked}");
    assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == corpus(ADJACENT, "word.out"), "{said}");
    assert!(!merge_lines(&observed).iter().any(|line| { return line.contains(tracked(BOTH_APPEND).as_str()); }), "an unresolved file is not listed as merged: {said}");
    remove(&fixture);
}

/// A file with one resolvable region and one that is not stays fully conflicted,
/// and nothing is staged for it.
#[test]
fn a_file_with_one_unresolvable_part_stays_fully_conflicted() {
    let fixture: Fixture = fixture("word-merge-partial");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let write = |text: &str| { std::fs::write(repo.join("partial.md"), text).expect("the file"); };
    write("alpha one\nkeep\nkeep again\nbeta two\n");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    write("ALPHA one\nkeep\nkeep again\nbeta zwei\n");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=theirs"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    write("alpha uno\nkeep\nkeep again\nbeta 2\n");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=ours"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let said: String = stderr_of(&observed);
    // Git merges the two edits of the first line itself only when they do not touch; here the second pair conflicts alone.
    let marked: String = String::from_utf8(std::fs::read(repo.join("partial.md")).expect("the file")).expect("UTF-8");
    assert!(marked.contains("<<<<<<<"), "{marked}");
    assert_eq!(unmerged(&fixture, repo.as_path()), vec![String::from("partial.md")], "{said}");
    assert!(merge_lines(&observed).is_empty(), "{said}");
    remove(&fixture);
}

/// A person's own edit of a conflicted file is never replaced:
/// a command that fails again on the same conflict leaves it.
#[test]
fn a_hand_edited_conflicted_file_is_left_alone() {
    let fixture: Fixture = fixture("word-merge-hand-edit");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let started: Observed = run_direct(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    assert_eq!(started.code, Some(1), "{}", stderr_of(&started));
    let hand_edited: &[u8] = b"my own half-resolved text\n<<<<<<< ours\nstill to decide\n=======\n>>>>>>> topic\n";
    std::fs::write(repo.join(tracked(ADJACENT)), hand_edited).expect("hand edit");
    let again: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--continue"]);
    assert_ne!(again.code, Some(0), "{}", stderr_of(&again));
    assert_eq!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file"), hand_edited, "{}", stderr_of(&again));
    assert_eq!(unmerged(&fixture, repo.as_path()), vec![tracked(ADJACENT)]);
    assert!(merge_lines(&again).is_empty(), "{}", stderr_of(&again));
    remove(&fixture);
}

/// A merge run in a checkout whose index holds another session's staged work is completed by the wrapper;
/// a conflict the
/// word pass resolves ends in a merge commit,
/// and the staged work stays staged.
#[test]
fn an_isolated_merge_with_an_adjacent_edit_is_completed_and_listed() {
    let fixture: Fixture = fixture("word-merge-isolated");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    std::fs::write(repo.join("staged.txt"), b"another session's work\n").expect("staged file");
    git(&fixture, repo.as_path(), &["add", "--", "staged.txt"]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let parents = git(&fixture, repo.as_path(), &["rev-list", "--parents", "--max-count=1", "HEAD"]);
    assert_eq!(String::from_utf8_lossy(&parents.stdout).split_whitespace().count(), 3, "a merge commit with two parents");
    assert!(std::fs::read(repo.join(tracked(ADJACENT))).expect("the file") == corpus(ADJACENT, "word.out"), "{said}");
    assert!(merge_lines(&observed).iter().any(|line| { return line.contains(tracked(ADJACENT).as_str()); }), "{said}");
    assert!(said.contains("git cat-file blob"), "a merge from objects says how to read an input after the fact: {said}");
    let status = git(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert!(String::from_utf8_lossy(&status.stdout).contains("A  staged.txt"), "the staged work stays staged");
    remove(&fixture);
}

/// The `merge=binary` line arrives in the side being merged,
/// so only the merge's own tree holds it:
/// the word pass must leave `data.lock` conflicted,
/// and the isolated merge is refused naming it,
/// never committing a text merge of a file its rule excludes.
#[test]
fn a_merge_rule_the_merge_itself_introduces_keeps_the_isolated_merge_from_merging_the_file() {
    let fixture: Fixture = fixture("word-merge-introduced-rule");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let guarded = |value: &str| -> Vec<u8> { return format!("data.lock merge={value}\n").into_bytes(); };
    std::fs::write(repo.join("data.lock"), b"port=1 host=a zone=z\n").expect("the base file");
    git(&fixture, repo.as_path(), &["add", "--", "data.lock"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join(".gitattributes"), guarded("binary")).expect("the rule");
    std::fs::write(repo.join("data.lock"), b"port=1 host=b zone=z\n").expect("their edit");
    git(&fixture, repo.as_path(), &["add", "--", ".gitattributes", "data.lock"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=theirs"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("data.lock"), b"port=9 host=a zone=z\n").expect("our edit");
    git(&fixture, repo.as_path(), &["add", "--", "data.lock"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ours"]);
    // Another session's staged work, which is what makes the wrapper merge from objects.
    std::fs::write(repo.join("staged.txt"), b"another session's work\n").expect("staged file");
    git(&fixture, repo.as_path(), &["add", "--", "staged.txt"]);
    let head: Vec<u8> = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout;
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(2), "{said}");
    assert!(said.contains("data.lock"), "{said}");
    assert!(merge_lines(&observed).is_empty(), "nothing was merged by word: {said}");
    assert_eq!(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout, head, "no merge commit is made");
    assert_eq!(std::fs::read(repo.join("data.lock")).expect("the file"), b"port=9 host=a zone=z\n".to_vec(), "{said}");
    remove(&fixture);
}

/// A part line names its three inputs as blobs,
/// which is what a person can still read once the pass has replaced the stages:
/// `git show :2:<path>` no longer answers,
/// and `git cat-file blob <oid>` answers each side's bytes.
#[test]
fn a_part_line_names_each_input_as_a_blob_that_survives_the_resolution() {
    let fixture: Fixture = fixture("word-merge-part-blobs");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT]);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let said: String = stderr_of(&observed);
    let lines: Vec<String> = merge_lines(&observed);
    let line: &String = lines.first().expect("the one merged file is listed");
    let named: Vec<String> = blobs_named(line);
    assert_eq!(named.len(), 3, "base, ours and theirs are named: {line}");
    assert!(said.contains("git cat-file blob"), "the output says how to read an input: {said}");
    assert_eq!(unmerged(&fixture, repo.as_path()), Vec::<String>::new(), "{said}");
    let staged: Observed = run_direct(&fixture, repo.as_path(), &["show", format!(":2:{}", tracked(ADJACENT)).as_str()]);
    assert_ne!(staged.code, Some(0), "the stage the list used to point at is gone");
    let sides: [Vec<u8>; 3] = [corpus(ADJACENT, "base"), corpus(ADJACENT, "ours"), corpus(ADJACENT, "theirs")];
    for (oid, bytes) in named.iter().zip(sides.iter()) {
        let blob: Vec<u8> = git(&fixture, repo.as_path(), &["cat-file", "blob", oid.as_str()]).stdout;
        assert_eq!(blob, *bytes, "the blob {oid} is one input of the part");
    }
    remove(&fixture);
}

/// A conflict the word pass cannot resolve still ends the isolated merge with the conflict finding,
/// and changes nothing.
#[test]
fn an_isolated_merge_with_a_both_append_conflict_is_still_refused() {
    let fixture: Fixture = fixture("word-merge-isolated-refused");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[BOTH_APPEND]);
    std::fs::write(repo.join("staged.txt"), b"another session's work\n").expect("staged file");
    git(&fixture, repo.as_path(), &["add", "--", "staged.txt"]);
    let head: Vec<u8> = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout;
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(2), "{said}");
    assert!(said.contains(tracked(BOTH_APPEND).as_str()), "{said}");
    assert_eq!(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout, head);
    remove(&fixture);
}

/// One file that resolves does not carry a second that does not:
/// the isolated merge is refused whole,
/// and nothing changes.
#[test]
fn an_isolated_merge_with_one_unresolvable_file_among_resolvable_ones_is_refused_whole() {
    let fixture: Fixture = fixture("word-merge-isolated-partial");
    let repo: PathBuf = prepared(&fixture, Stop::Merge, &[ADJACENT, BOTH_APPEND]);
    std::fs::write(repo.join("staged.txt"), b"another session's work\n").expect("staged file");
    git(&fixture, repo.as_path(), &["add", "--", "staged.txt"]);
    let head: Vec<u8> = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout;
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice());
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(2), "{said}");
    assert!(said.contains(tracked(BOTH_APPEND).as_str()) && said.contains(tracked(ADJACENT).as_str()), "both conflicted paths are named: {said}");
    assert_eq!(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout, head);
    assert!(merge_lines(&observed).is_empty(), "{said}");
    remove(&fixture);
}

/// What:
///  A `.gitattributes` the merge itself left conflicted names no rule,
/// so a file whose rule is inside that conflict keeps its conflict.
/// Why:
///   The worktree copy is Git's marker file,
/// and its last matching line is the second side's rule;
/// reading it would let a
///  rule the resolved file does not carry decide the merge
/// (cross-vendor review of this branch,
/// finding 2).
/// A rule file that conflicts is the same unknown as the one a merge from objects refuses,
/// so nothing is merged.
#[test]
fn a_gitattributes_the_merge_left_conflicted_names_no_rule_for_the_worktree_pass() {
    let fixture: Fixture = fixture("word-merge-worktree-conflicted-attributes");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let rule = |ours: &str| -> Vec<u8> { return format!("data.txt {ours}\n").into_bytes(); };
    std::fs::write(repo.join(".gitattributes"), rule("merge=text flag=old")).expect("the base rule");
    std::fs::write(repo.join("data.txt"), b"alpha = 0\nbeta = 0\n").expect("the base file");
    git(&fixture, repo.as_path(), &["add", "--", ".gitattributes", "data.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join(".gitattributes"), rule("merge=text flag=new")).expect("their rule");
    std::fs::write(repo.join("data.txt"), b"alpha = 0\nbeta = 2\n").expect("their edit");
    git(&fixture, repo.as_path(), &["add", "--", ".gitattributes", "data.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=theirs"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join(".gitattributes"), rule("merge=binary flag=old")).expect("our rule");
    std::fs::write(repo.join("data.txt"), b"alpha = 1\nbeta = 0\n").expect("our edit");
    git(&fixture, repo.as_path(), &["add", "--", ".gitattributes", "data.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ours"]);
    let head: Vec<u8> = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout;
    // What Git leaves in the file when it does not merge it by text: our side's bytes, with the stages kept.
    let by_git: Vec<u8> = std::fs::read(repo.join("data.txt")).expect("the file");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(1), "the merge stays stopped: {said}");
    let mut conflicted: Vec<String> = unmerged(&fixture, repo.as_path());
    conflicted.sort();
    assert_eq!(conflicted, vec![String::from(".gitattributes"), String::from("data.txt")], "both conflicts stay as Git left them: {said}");
    assert_eq!(std::fs::read(repo.join("data.txt")).expect("the file"), by_git, "the file Git would not merge by text keeps Git's bytes: {said}");
    assert!(merge_lines(&observed).is_empty(), "nothing is merged from a rule file that conflicts: {said}");
    assert_eq!(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout, head, "no commit is made");
    remove(&fixture);
}

/// What:
///  `git -c <key>=<tree> merge` reads the `merge` attribute from the tree the person named on the command line,
/// in any letter case the key allows.
/// Why:
///   Git puts a command-line configuration above every file,
/// so a tree that excludes the file must govern the pass
/// (cross-vendor review of this branch,
/// finding 3;
/// cross-vendor review round 2,
/// finding 2 for the letters).
/// The pass named the merge's own tree over it,
/// and the merge was committed although a rule the
///  person asked for forbids it.
fn a_command_line_tree_governs_the_isolated_merge(naming: &[&str], variable: Option<(&str, &str)>, name: &str) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("data.lock"), b"port=1 host=a zone=z\n").expect("the base file");
    git(&fixture, repo.as_path(), &["add", "--", "data.lock"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "rules"]);
    std::fs::write(repo.join(".gitattributes"), b"data.lock merge=binary\n").expect("the rules of the tree the person names");
    git(&fixture, repo.as_path(), &["add", "--", ".gitattributes"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=the rules"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join("data.lock"), b"port=1 host=b zone=z\n").expect("their edit");
    git(&fixture, repo.as_path(), &["add", "--", "data.lock"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=theirs"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("data.lock"), b"port=9 host=a zone=z\n").expect("our edit");
    git(&fixture, repo.as_path(), &["add", "--", "data.lock"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ours"]);
    // Another session's staged work, which is what makes the wrapper merge from objects.
    std::fs::write(repo.join("staged.txt"), b"another session's work\n").expect("staged file");
    git(&fixture, repo.as_path(), &["add", "--", "staged.txt"]);
    let head: Vec<u8> = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout;
    let mut command: Command = wrapped(&fixture);
    command.current_dir(repo.as_path()).args(naming).args(["merge", "--no-edit", "topic"]);
    if let Some((variable_name, value)) = variable {
        command.env(variable_name, value);
    }
    let observed: Observed = observe(&mut command, b"");
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(2), "the tree the person named forbids the merge ({naming:?}): {said}");
    assert!(said.contains("data.lock"), "the refused path is named: {said}");
    assert!(merge_lines(&observed).is_empty(), "nothing was merged by word: {said}");
    assert_eq!(git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).stdout, head, "no merge commit is made");
    assert_eq!(std::fs::read(repo.join("data.lock")).expect("the file"), b"port=9 host=a zone=z\n".to_vec(), "our file is untouched: {said}");
    remove(&fixture);
}

/// The tree the person names on the command line governs the isolated merge.
#[test]
fn a_configuration_option_on_the_command_line_names_the_tree_of_an_isolated_merge() {
    a_command_line_tree_governs_the_isolated_merge(&["-c", "attr.tree=rules"], None, "word-merge-command-line-tree");
}

/// The same key in the letters the person typed,
/// which Git reads as the same configuration key.
#[test]
fn a_configuration_option_in_any_letter_case_names_the_tree_of_an_isolated_merge() {
    a_command_line_tree_governs_the_isolated_merge(&["-c", "Attr.Tree=rules"], None, "word-merge-command-line-tree-case");
}

/// What:
///  The tree a person names through `--config-env` governs the isolated merge,
/// in both of its forms.
/// Why:
///   Git reads `--config-env=attr.tree=<variable>` as the same configuration value a `-c` gives,
/// and the pass read only
///  separate `-c` arguments,
/// so it named the tree the merge produced over the person's and word-merged a file their rule
///  excludes
/// (cross-vendor review round 3,
/// finding 3).
#[test]
fn a_configuration_environment_option_names_the_tree_of_an_isolated_merge() {
    a_command_line_tree_governs_the_isolated_merge(&["--config-env=attr.tree=RULES"], Some(("RULES", "rules")), "word-merge-config-env-tree");
    a_command_line_tree_governs_the_isolated_merge(&["--config-env", "attr.tree=RULES"], Some(("RULES", "rules")), "word-merge-config-env-tree-separate");
}

/// How many files a control conflicts.
/// Why:
///   The gate is what puts the control's edit inside the pass,
/// so the count buys no window;
/// it only gives the pass several files to merge,
/// so the edited one is neither the first nor the last of them.
const CONFLICTS_OF_A_HELD_PASS: usize = 4;

/// A repository whose merge of `topic` into `main` stops with every file conflicted and word-resolvable,
/// with the checked-out branch left on `main`.
fn many_conflicts(fixture: &Fixture, count: usize) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    // One file the word pass cannot resolve keeps the merge stopped, so the control never waits on a commit.
    let names: Vec<String> = (0..count).map(|number| { return format!("conflict{number:02}.md"); }).chain([format!("{BOTH_APPEND}.md")]).collect();
    let write = |extension: &str| {
        for name in &names {
            let input: &str = if name.starts_with(BOTH_APPEND) { BOTH_APPEND } else { ADJACENT };
            std::fs::write(repo.join(name), corpus(input, extension)).expect("an input");
        }
    };
    write("base");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    write("theirs");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=theirs"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    write("ours");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=ours"]);
    return repo;
}

/// What:
///  An edit another command makes while the pass merges its words is never replaced,
/// and the line names the path the pass left.
/// Why:
///   The baseline of a path was read after the merge was computed,
/// so an edit made during the merge became the baseline and `checkout-index --force` then wrote
///  the earlier merge over it
/// (cross-vendor review round 2,
/// finding 1).
///  The gate stops the pass at its own attribute query,
///  so the pass cannot compute a merge before this edit lands,
///  and no clock has to be beaten for that to happen.
#[test]
fn an_edit_made_while_the_pass_merges_is_not_replaced() {
    let fixture: Fixture = fixture("word-merge-edit-during-the-pass");
    let repo: PathBuf = many_conflicts(&fixture, CONFLICTS_OF_A_HELD_PASS);
    let edited: String = format!("conflict{:02}.md", CONFLICTS_OF_A_HELD_PASS / 2);
    let resolution: &[u8] = b"a resolution another command wrote\n";
    let target: PathBuf = repo.join(edited.as_str());
    let gate: PassGate = PassGate::prepare(&fixture);
    let (observed, control) = gate.held_run(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice(), move || {
        std::fs::write(target.as_path(), resolution).expect("the other command's write");
    });
    let said: String = stderr_of(&observed);
    assert!(control.join().expect("the control thread"), "the control reached the merge pass: {said}");
    assert_eq!(std::fs::read(repo.join(edited.as_str())).expect("the file"), resolution.to_vec(), "the other command's bytes survive: {said}");
    assert!(said.contains("changed while the second merge pass"), "the line names the path the pass left: {said}");
    assert!(said.contains(edited.as_str()), "and names it by path: {said}");
    remove(&fixture);
}

/// What:
///  A resolution of the same length whose modification time was put back is still not replaced.
/// Why:
///   The check compared the file's device,
/// inode,
/// length and modification time,
/// and a writer can restore the time it saved,
/// so different bytes passed it and `checkout-index --force` then wrote the merge over them
/// (cross-vendor review round 2,
/// finding 3).
/// These are conflicted files the pass has already read,
/// so the bytes are what it compares.
#[test]
fn a_same_length_resolution_with_a_restored_time_is_not_replaced() {
    use std::time::SystemTime;
    let fixture: Fixture = fixture("word-merge-same-length");
    let repo: PathBuf = many_conflicts(&fixture, CONFLICTS_OF_A_HELD_PASS);
    let edited: String = format!("conflict{:02}.md", CONFLICTS_OF_A_HELD_PASS / 2);
    let target: PathBuf = repo.join(edited.as_str());
    let gate: PassGate = PassGate::prepare(&fixture);
    let (observed, control) = gate.held_run(&fixture, repo.as_path(), arguments_of(Stop::Merge).as_slice(), move || {
        let before: std::fs::Metadata = std::fs::metadata(target.as_path()).expect("the file Git left");
        let saved: SystemTime = before.modified().expect("its modification time");
        let replacement: Vec<u8> = vec![b'R'; before.len() as usize];
        std::fs::write(target.as_path(), replacement.as_slice()).expect("the other command's write");
        let file: std::fs::File = std::fs::File::options().write(true).open(target.as_path()).expect("the file");
        file.set_times(std::fs::FileTimes::new().set_modified(saved)).expect("the time put back");
    });
    let said: String = stderr_of(&observed);
    assert!(control.join().expect("the control thread"), "the control reached the merge pass: {said}");
    let held: Vec<u8> = std::fs::read(repo.join(edited.as_str())).expect("the file");
    assert!(held.iter().all(|byte| { return *byte == b'R'; }), "the other command's bytes survive: {said}");
    assert!(said.contains("changed while the second merge pass"), "the line names the path the pass left: {said}");
    remove(&fixture);
}
