//! What:
//!  A `git checkout` or `git restore` that would replace a conflicted file during a real merge,
//! rebase (both backends),
//! cherry-pick and revert,
//! through the
//!  built executable:
//! the command is refused,
//! the refusal names every file it would replace and the opt-out,
//! the opt-out runs the command as real Git runs it,
//! and nothing is saved.
//! Why:
//!  The resolution of a conflict is where the person's own work stands during an operation,
//! and no copy of it is kept.
//! These controls make a real conflict with real Git,
//! edit the conflicted file by hand
//!  as a person resolving it would,
//! run the wrapper,
//! compare what it said and what is left on disk,
//! and run the same command through real Git alone for the answer the opt-out must reach.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await makeConflict('merge'); edit('a.rs'); const out = run(['checkout', '--theirs', '--', 'a.rs']);
//! expect(out.stderr).toContain('--no-enforce-worktree'); expect(readFileSync('a.rs')).toEqual(handEdited);
//! ```

use super::support::{Fixture, Observed, fixture, git, remove, repository, run_direct, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What a person had in the conflicted file when they ran the command:
/// not any side's content,
/// not valid UTF-8.
const HAND_EDITED: &[u8] = b"<<<<<<< ours\nmine \xff\n=======\ntheirs\n>>>>>>> topic\nand my own line\n";

/// Which operation a control stops in.
#[derive(Clone, Copy, Debug)]
enum Stop {
    /// `git merge topic`.
    Merge,
    /// `git rebase main` on `topic`,
    /// with the merge backend.
    RebaseMerge,
    /// `git rebase --apply main` on `topic`,
    /// with the apply backend.
    RebaseApply,
    /// `git cherry-pick topic`.
    CherryPick,
    /// `git revert` of the commit that changed `a.rs` on `main`,
    /// after `a.rs` changed again.
    Revert,
}

/// The word the refusal uses for the operation `stop` leaves waiting.
fn operation_word(stop: Stop) -> &'static str {
    return match stop {
        Stop::Merge => "merge",
        Stop::RebaseMerge | Stop::RebaseApply => "rebase",
        Stop::CherryPick => "cherry-pick",
        Stop::Revert => "revert",
    };
}

/// A repository stopped by `stop` with `a.rs` conflicted and `b.rs` and `c.rs` clean,
/// and `a.rs` hand-edited.
fn conflicted(fixture: &Fixture, stop: Stop) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    for (name, text) in [("a.rs", "base\n"), ("b.rs", "base\n"), ("c.rs", "keep\n")] {
        std::fs::write(repo.join(name), text).expect("file");
    }
    git(fixture, repo.as_path(), &["add", "--", "a.rs", "b.rs", "c.rs"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "topic"]);
    std::fs::write(repo.join("a.rs"), "topic\n").expect("topic side");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic side"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("a.rs"), "main\n").expect("main side");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main side"]);
    let started: Observed = match stop {
        Stop::Merge => run_direct(fixture, repo.as_path(), &["merge", "--no-edit", "topic"]),
        Stop::CherryPick => run_direct(fixture, repo.as_path(), &["cherry-pick", "topic"]),
        Stop::RebaseMerge | Stop::RebaseApply => {
            git(fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
            let arguments: Vec<&str> = if matches!(stop, Stop::RebaseApply) { vec!["rebase", "--apply", "main"] } else { vec!["rebase", "main"] };
            run_direct(fixture, repo.as_path(), arguments.as_slice())
        }
        Stop::Revert => {
            std::fs::write(repo.join("a.rs"), "later\n").expect("later change");
            git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=later change"]);
            run_direct(fixture, repo.as_path(), &["revert", "--no-edit", "HEAD~1"])
        }
    };
    assert_eq!(started.code, Some(1), "the operation stops with a conflict: {}{}", String::from_utf8_lossy(&started.stdout), stderr_of(&started));
    assert!(
        git(fixture, repo.as_path(), &["ls-files", "--unmerged"]).stdout.windows(4).any(|window| { return window == b"a.rs"; }),
        "a.rs is unmerged"
    );
    std::fs::write(repo.join("a.rs"), HAND_EDITED).expect("hand edit");
    return repo;
}

/// Whether the wrapper itself rejected the command.
fn rejected_by_policy(observed: &Observed) -> bool {
    return observed.code == Some(1) && stderr_of(observed).contains("\"policyId\":\"linked-worktree-only\"");
}

/// The directory of the copies below the common Git directory of `repo`.
fn saved_root(repo: &Path) -> PathBuf {
    return repo.join(".git/monochromatic-git-policy-cli/discarded");
}

/// Every audit line of the repository.
fn audit_lines(repo: &Path) -> Vec<String> {
    return std::fs::read_to_string(repo.join(".git/monochromatic-git-policy-cli-audit.jsonl")).unwrap_or_default().lines().map(String::from).collect();
}

/// The wrapper-only opt-out every refusal of this policy names.
const OPT_OUT: &str = "--no-enforce-worktree";

/// Whether the refusal says what the command would replace:
/// the file,
/// that the index marks it as unmerged,
/// the operation that waits,
/// and the opt-out.
fn names_the_conflicted_file(said: &str, stop: Stop) -> bool {
    return said.contains("`a.rs`")
        && said.contains("unmerged")
        && said.contains(operation_word(stop))
        && said.contains(OPT_OUT)
        && said.contains("keeps no copy");
}

/// No copy of the person's work is anywhere below the Git directory.
fn nothing_was_saved(repo: &Path, said: &str) {
    assert!(!saved_root(repo).exists(), "no copy was made: {said}");
    assert!(audit_lines(repo).iter().all(|line| { return !line.contains("saved-conflict"); }), "no saved-conflict line: {said}");
}

/// Every form the reading covers is refused during the operation it settles:
/// the refusal names the file it would replace and the opt-out,
/// the file is as the person wrote it,
/// and nothing is saved.
#[test]
fn a_conflict_side_is_refused_naming_the_file_and_the_opt_out() {
    let cases: [(Stop, &[&[&str]]); 5] = [
        (
            Stop::Merge,
            &[
                &["checkout", "--theirs", "--", "a.rs"],
                &["checkout", "--ours", "--", "a.rs"],
                &["checkout", "topic", "--", "a.rs"],
                &["checkout", "--merge", "--", "a.rs"],
                &["restore", "--theirs", "--", "a.rs"],
                &["restore", "--ours", "--", "a.rs"],
                &["restore", "--source=topic", "--", "a.rs"],
            ],
        ),
        (Stop::RebaseMerge, &[&["checkout", "--theirs", "--", "a.rs"], &["restore", "--ours", "--", "a.rs"]]),
        (Stop::RebaseApply, &[&["checkout", "--theirs", "--", "a.rs"]]),
        (Stop::CherryPick, &[&["checkout", "--theirs", "--", "a.rs"], &["restore", "--theirs", "--", "a.rs"]]),
        (Stop::Revert, &[&["checkout", "--ours", "--", "a.rs"]]),
    ];
    for (stop, forms) in cases {
        for (round, arguments) in forms.iter().enumerate() {
            let fixture: Fixture = fixture(format!("side-refused-{stop:?}-{round}").to_lowercase().as_str());
            let repo: PathBuf = conflicted(&fixture, stop);
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments);
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(names_the_conflicted_file(said.as_str(), stop), "{stop:?} {arguments:?}: the refusal names the file and the opt-out: {said}");
            assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "{stop:?} {arguments:?}: the file is as the person had it");
            nothing_was_saved(repo.as_path(), said.as_str());
            remove(&fixture);
        }
    }
}

/// The same holds in a linked worktree,
/// where a copy used to go to the common Git directory.
#[test]
fn a_linked_worktree_refuses_the_conflict_side_and_saves_nothing() {
    let fixture: Fixture = fixture("side-refused-linked");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.rs"), "base\n").expect("a.rs");
    git(&fixture, repo.as_path(), &["add", "--", "a.rs"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["branch", "topic"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    std::fs::write(linked.join("a.rs"), "side\n").expect("side edit");
    git(&fixture, linked.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "topic"]);
    std::fs::write(repo.join("a.rs"), "topic\n").expect("topic edit");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=topic"]);
    let merged: Observed = run_direct(&fixture, linked.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(merged.code, Some(1), "{}", stderr_of(&merged));
    std::fs::write(linked.join("a.rs"), HAND_EDITED).expect("hand edit");
    let observed: Observed = run_wrapped(&fixture, linked.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert!(rejected_by_policy(&observed), "{said}");
    assert!(names_the_conflicted_file(said.as_str(), Stop::Merge), "{said}");
    assert_eq!(std::fs::read(linked.join("a.rs")).expect("a.rs"), HAND_EDITED);
    nothing_was_saved(repo.as_path(), said.as_str());
    remove(&fixture);
}

/// The opt-out runs the command as real Git runs it:
/// the file holds the side the command names,
/// no copy is kept,
/// and real Git does exactly the same in the same state.
#[test]
fn the_opt_out_replaces_the_conflicted_file_as_real_git_does() {
    let direct: Fixture = fixture("side-refused-hatch-direct");
    let direct_repo: PathBuf = conflicted(&direct, Stop::Merge);
    let git_ran: Observed = run_direct(&direct, direct_repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(git_ran.code, Some(0), "real Git takes the side: {}", stderr_of(&git_ran));
    let fixture: Fixture = fixture("side-refused-hatch");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", OPT_OUT, "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    assert_eq!(
        std::fs::read(repo.join("a.rs")).expect("a.rs"),
        std::fs::read(direct_repo.join("a.rs")).expect("real Git's a.rs"),
        "the opt-out leaves what real Git leaves"
    );
    assert!(!saved_root(repo.as_path()).exists(), "the opt-out saves nothing: {said}");
    assert!(audit_lines(repo.as_path()).iter().any(|line| { return line.contains("\"outcome\":\"hatch\""); }), "the audit line names the opt-out");
    remove(&fixture);
    remove(&direct);
}

/// A path that is not conflicted is not named,
/// and the line that names it beside a conflicted one is refused whole:
/// the refusal claims only the file the index marks as unmerged,
/// and nothing is copied or changed.
#[test]
fn a_clean_path_is_not_named_as_a_conflict_and_the_line_is_refused() {
    for stop in [Stop::Merge, Stop::RebaseMerge, Stop::CherryPick] {
        let fixture: Fixture = fixture(format!("side-refused-clean-{stop:?}").to_lowercase().as_str());
        let repo: PathBuf = conflicted(&fixture, stop);
        for arguments in [
            vec!["checkout", "--theirs", "--", "c.rs"],
            vec!["checkout", "topic", "--", "b.rs"],
            vec!["restore", "--theirs", "--", "c.rs"],
            vec!["checkout", "--theirs", "--", "."],
            vec!["checkout", "--theirs", "--", "*.rs"],
        ] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(!said.contains("keeps no copy"), "{stop:?} {arguments:?}: no file of this line is unmerged, so no file is claimed: {said}");
        }
        for arguments in [vec!["checkout", "--theirs", "--", "a.rs", "c.rs"], vec!["restore", "--ours", "--", "c.rs", "a.rs"]] {
            let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
            let said: String = stderr_of(&observed);
            assert!(rejected_by_policy(&observed), "{stop:?} {arguments:?}: {said}");
            assert!(said.contains("`a.rs`") && said.contains("keeps no copy"), "{stop:?} {arguments:?} names the unmerged file: {said}");
            assert!(!said.contains("`c.rs`"), "{stop:?} {arguments:?}: the clean path is not called a conflict: {said}");
        }
        assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "the hand edit is untouched");
        nothing_was_saved(repo.as_path(), "");
        remove(&fixture);
    }
}

/// The forms the reading does not cover keep their rejection even for a conflicted path.
#[test]
fn forms_outside_the_reading_keep_the_rejection() {
    let fixture: Fixture = fixture("side-refused-uncovered");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    for arguments in [
        vec!["checkout", "-f", "--", "a.rs"],
        vec!["checkout", "--theirs", "a.rs"],
        vec!["checkout", "--patch", "--", "a.rs"],
        vec!["restore", "--staged", "--theirs", "--", "a.rs"],
        vec!["restore", "--theirs", "a.rs"],
        vec!["checkout", "--theirs", "--", "sub/"],
        vec!["reset", "--hard"],
        vec!["merge", "--abort"],
    ] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        assert!(rejected_by_policy(&observed), "{arguments:?}: {}", stderr_of(&observed));
    }
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED);
    nothing_was_saved(repo.as_path(), "");
    remove(&fixture);
}

/// With no operation in progress a conflict-shaped line is an ordinary discarding form and is rejected,
/// and it names no conflict,
/// even for a path that
///  still has unmerged entries (what a conflicting `git stash pop` leaves).
#[test]
fn without_an_operation_in_progress_no_conflict_is_named() {
    let fixture: Fixture = fixture("side-refused-no-operation");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    std::fs::remove_file(repo.join(".git/MERGE_HEAD")).expect("end the merge's state, leaving the unmerged entries");
    for arguments in [vec!["checkout", "--theirs", "--", "a.rs"], vec!["restore", "--source=topic", "--", "a.rs"]] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), arguments.as_slice());
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{arguments:?}: {said}");
        assert!(!said.contains("keeps no copy"), "{arguments:?}: no operation waits, so no conflict is claimed: {said}");
    }
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED);
    nothing_was_saved(repo.as_path(), "");
    remove(&fixture);
}

/// What:
///  The commands a sentence of a refusal gives,
/// in order,
///  with the path placeholders filled in for one file.
/// Why:
///   The control runs the words the person is given and not a paraphrase of them,
///  so a message and the behaviour it promises cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commands = backtickedSpans(sentence).filter((span) => span.includes('<')).map((span) => span.replaceAll('<path>', path));
/// ```
fn backticked_commands(sentence: &str, path: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest: &str = sentence;
    while let Some(opened) = rest.find('`') {
        let after: &str = &rest[opened + 1..];
        let Some(closed) = after.find('`') else { break };
        let span: &str = &after[..closed];
        if span.contains('<') {
            found.push(span.replace("<paths>", path).replace("<path>", path));
        }
        rest = &after[closed + 1..];
    }
    return found;
}

/// What:
///  The sentence of a refusal that offers the opt-out,
///  which is the sentence whose commands are the person's own way through.
/// Why:
///   Reading it from the message keeps the control tied to what the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const advice = message.split('. ').find((one) => one.includes('on purpose'));
/// ```
fn the_opt_out_sentence(said: &str) -> String {
    for sentence in said.split(". ") {
        if sentence.contains("on purpose") {
            return String::from(sentence);
        }
    }
    return String::new();
}

/// What:
///  Run a shell command the way the person would,
///  in the fixture's own environment,
///  with the wrapper first on `PATH`.
/// Why:
///   A refusal names commands for a shell,
///  and the control has to run exactly those,
///  with the same `HOME` and runtime directory as the wrapped command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const observed = await sh(repo, adviceCommand);
/// ```
fn run_advice(fixture: &Fixture, directory: &Path, command: &str) -> Observed {
    let mut shell: std::process::Command = std::process::Command::new("/bin/sh");
    shell.arg("-c").arg(command);
    let path: std::ffi::OsString = super::support::wrapper_first_path(fixture);
    super::support::hermetic(&mut shell, fixture, path.as_os_str());
    // `hermetic` starts every command in the fixture root; the person runs these in the worktree.
    shell.current_dir(directory);
    if let Some(runtime) = super::support::service_runtime(fixture) {
        shell.env("XDG_RUNTIME_DIR", runtime);
    }
    return super::support::observe(&mut shell, b"");
}

/// What:
///  The refusal's own advice,
/// run to its end on a real conflict:
///  the backup is made,
///  the opt-out replaces the file,
///  and the backup puts the person's bytes back.
/// Why:
///  A refusal that protects the work and then tells the person to make a backup that cannot be restored
///  loses the very resolution it refused to replace
///  (a cross-vendor review of this change,
/// its first finding:
/// `git diff -- <path>` on an unmerged path is a combined diff,
/// which `git apply` refuses).
/// So the bytes are compared here,
/// and the message must name the command that puts them back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const advice = theOptOutSentence(refusal); const [save, restore] = backtickedCommands(advice, 'a.rs');
/// await sh(save); await hatch(); await sh(restore ?? 'git apply backup.patch'); expect(read('a.rs')).toEqual(whatThePersonWrote);
/// ```
#[test]
fn the_refusals_own_backup_advice_restores_the_resolution_after_the_opt_out() {
    let fixture: Fixture = fixture("side-advice-backup");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let refusal: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&refusal);
    assert!(rejected_by_policy(&refusal), "{said}");
    let advice: String = the_opt_out_sentence(said.as_str());
    assert!(!advice.is_empty(), "the refusal offers the opt-out in its own sentence: {said}");
    let commands: Vec<String> = backticked_commands(advice.as_str(), "a.rs");
    assert!(!commands.is_empty(), "the opt-out sentence names a command that keeps the bytes: {advice}");
    let saved: Observed = run_advice(&fixture, repo.as_path(), commands[0].as_str());
    assert_eq!(saved.code, Some(0), "the advised backup runs: {}\n{}", commands[0], stderr_of(&saved));
    let hatch: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--no-enforce-worktree", "--theirs", "--", "a.rs"]);
    assert_eq!(hatch.code, Some(0), "the opt-out runs: {}", stderr_of(&hatch));
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), b"topic\n", "the opt-out really replaced the file");
    let restore: Option<&String> = commands.get(1);
    let restored: Observed = match restore {
        Some(command) => run_advice(&fixture, repo.as_path(), command.as_str()),
        // A message that names no way back is judged by the only way back a patch offers.
        None => run_advice(&fixture, repo.as_path(), "git apply backup.patch"),
    };
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "the advised backup brings the resolution back ({commands:?}, restore exit {:?}: {})", restored.code, stderr_of(&restored));
    assert!(restore.is_some(), "the refusal names the command that puts the file back: {advice}");
    remove(&fixture);
}

/// What:
///  Every command a refusal names,
/// in the order it names them,
///  with the path placeholders filled in for one file.
/// Why:
///   The controls run the words the person is given and not a paraphrase of them,
///  so a message and the behaviour it promises cannot drift apart.
///  Every command a refusal names is in backticks,
/// which is also how the person sees it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commands = backtickedSpans(message).filter((span) => span.includes('<')).map((span) => span.replaceAll('<path>', path));
/// ```
fn commands_of_the_message(said: &str, path: &str, message: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest: &str = said;
    while let Some(opened) = rest.find('`') {
        let after: &str = &rest[opened + 1..];
        let Some(closed) = after.find('`') else { break };
        let span: &str = &after[..closed];
        if span.contains('<') {
            found.push(span.replace("<paths>", path).replace("<path>", path).replace("<message>", message));
        }
        rest = &after[closed + 1..];
    }
    return found;
}

/// What:
///  The commands of a message that start with `prefix`,
/// in the order named.
/// Why:
///   A refusal names reads,
/// a way to conclude,
/// and a way to back the file up;
///  a control runs the ones of its own kind and leaves the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reads = commands.filter((one) => one.startsWith('git show'));
/// ```
fn commands_starting_with(commands: &[String], prefix: &str) -> Vec<String> {
    return commands.iter().filter(|one| { return one.starts_with(prefix); }).cloned().collect();
}

/// What:
///  The other pieces of advice the refusal gives,
#[test]
fn every_other_piece_of_advice_in_the_refusal_works_on_its_own_state() {
    let fixture: Fixture = fixture("side-advice-sides");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    let refusal: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&refusal);
    assert!(rejected_by_policy(&refusal), "{said}");
    // One word, so the shell-splitting of the control carries the whole message as one argument.
    let commands: Vec<String> = commands_of_the_message(said.as_str(), "a.rs", "resolve-the-conflict");
    // The three sides the message points at: stage 1 is the base, 2 is ours, 3 is theirs.
    let sides: Vec<String> = commands_starting_with(commands.as_slice(), "git show");
    assert_eq!(sides.len(), 3, "the refusal names the three sides: {commands:?}");
    for (command, expected) in sides.iter().zip([b"base\n" as &[u8], b"main\n", b"topic\n"]) {
        let shown: Observed = run_advice(&fixture, repo.as_path(), command.as_str());
        assert_eq!(shown.code, Some(0), "`{command}` prints a side: {}", stderr_of(&shown));
        assert_eq!(shown.stdout.as_slice(), expected, "`{command}` prints the side it names");
    }
    // `git diff` prints the file as it stands: the resolution the person wrote is in it.
    let diffs: Vec<String> = commands_starting_with(commands.as_slice(), "git diff");
    assert_eq!(diffs.len(), 1, "the refusal names one way to see the file as it stands: {commands:?}");
    let shown: Observed = run_advice(&fixture, repo.as_path(), diffs[0].as_str());
    let text: String = String::from_utf8_lossy(shown.stdout.as_slice()).into_owned();
    for line in String::from_utf8_lossy(HAND_EDITED).lines() {
        assert!(text.contains(line), "`{}` prints the line the person wrote ({line:?}): {text}", diffs[0]);
    }
    remove(&fixture);
}

/// What:
///  The refusal's advice for concluding the merge,
/// run to its end through the wrapper,
///  with the fixture's own commit service:
///  the resolution is staged,
///  the commit is made,
///  and the merge is over.
/// Why:
///  The advice used to be `git commit --message <message> -- <paths>`,
///  which stages nothing and,
///  in this wrapper,
///  commits the named path while the merge stays in progress
///  (the wrapper says so itself:
/// "the merge is still in progress,
/// because this commit took only the paths you named").
/// Real Git refuses that form outright (`fatal: cannot do a partial commit during a merge`).
/// So the control runs the message's own `git add` and `git commit` commands,
/// in its order,
/// and asks for the end state:
/// no `MERGE_HEAD`,
/// one ordinary commit,
/// and the person's bytes in the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [stage, commit] = commandsOfTheMessage(refusal); await runCommit(stage); await runCommit(commit);
/// expect(existsSync('.git/MERGE_HEAD')).toBe(false); expect(read('a.rs')).toEqual(theResolution);
/// ```
#[test]
fn the_advised_way_to_conclude_the_merge_concludes_it() {
    let fixture: Fixture = fixture("side-advice-conclude");
    let repo: PathBuf = conflicted(&fixture, Stop::Merge);
    // A resolution a person would write, with no conflict markers left in it.
    std::fs::write(repo.join("a.rs"), b"both sides, as I settled them\n").expect("a resolution");
    let refusal: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    let said: String = stderr_of(&refusal);
    assert!(rejected_by_policy(&refusal), "{said}");
    // One word, so the shell-splitting of the control carries the whole message as one argument.
    let commands: Vec<String> = commands_of_the_message(said.as_str(), "a.rs", "resolve-the-conflict");
    let staging: Vec<String> = commands_starting_with(commands.as_slice(), "git add");
    let committing: Vec<String> = commands_starting_with(commands.as_slice(), "git commit");
    assert_eq!(staging.len(), 1, "the refusal names the way to stage the resolution: {commands:?}");
    assert_eq!(committing.len(), 1, "the refusal names the way to commit it: {commands:?}");
    assert!(committing[0].contains("--message resolve-the-conflict"), "the commit names the message the refusal gives: {commands:?}");
    let staging_words: Vec<String> = shell_words(staging[0].as_str());
    let staged: Observed = super::commit::run_commit(&fixture, repo.as_path(), staging_words.iter().map(String::as_str).collect::<Vec<&str>>().as_slice());
    let staged_said: String = stderr_of(&staged);
    assert_eq!(staged.code, Some(0), "`{}` runs: {staged_said}", staging[0]);
    let committing_words: Vec<String> = shell_words(committing[0].as_str());
    let concluded: Observed = super::commit::run_commit(&fixture, repo.as_path(), committing_words.iter().map(String::as_str).collect::<Vec<&str>>().as_slice());
    let concluded_said: String = stderr_of(&concluded);
    assert_eq!(concluded.code, Some(0), "`{}` runs: {concluded_said}", committing[0]);
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "the merge is over after the advised commands: {concluded_said}");
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), b"both sides, as I settled them\n", "the commit holds the resolution");
    assert_eq!(
        git(&fixture, repo.as_path(), &["log", "--format=%s", "-1"]).stdout.as_slice(),
        b"resolve-the-conflict\n",
        "one ordinary commit is what the advisee makes"
    );
    super::commit::remove_settled(&fixture);
}

/// What:
///  The words of a command the way a shell splits them:
///  one argument per space-separated word,
///  which is all the controls' commands need (none carries a space or a quote).
/// Why:
///  The wrapped commands are run as an argument vector,
///  and the refusal writes them for a shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = command.split(' ').filter((one) => one.length > 0);
/// ```
fn shell_words(command: &str) -> Vec<String> {
    let words: Vec<String> = command.split(' ').filter(|word| { return !word.is_empty(); }).map(String::from).collect();
    // A refusal writes the whole command, starting at `git`; the wrapper is the `git` the control runs.
    if words.first().is_some_and(|first| { return first == "git"; }) {
        return words[1..].to_vec();
    }
    return words;
}

/// Command lines that look like one conflicted file to the guard and are more to Git,
/// Command lines that look like one conflicted file to the guard and are more to Git,
/// found by a cross-vendor review.
#[path = "binary_conflict_side_hardening_tests.rs"]
mod hardening;
