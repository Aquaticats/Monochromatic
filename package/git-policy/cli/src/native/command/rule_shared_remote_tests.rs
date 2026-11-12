//! What:
//!  Every spelling that writes the remote settings a repository's worktrees share,
//!  every spelling
//!       that only reads them,
//!  and the real Git 2.56.0 facts the decision rests on.
//! Why:
//!  A guard that misses one spelling is worse than none:
//!  the accident this rule exists for was a `remote
//!       set-url --push`,
//!  not the plain form.
//!  The scope questions are Git's own:
//!  a `--worktree` written after a key is a value,
//!  not a scope,
//!  so an argument-only reading has to be checked against the binary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(sharedRemoteWrite('config', ['--add', 'remote.origin.url', url])).toEqual({ kind: 'key', key: 'remote.origin.url' });
//! ```

/// The decision,
///  its result types and the tokenizer helpers the cases use.
use super::{SHARED_REMOTE_SETTINGS_CODE, SharedRemoteChange, SharedRemoteWrite, shared_remote_write};
use crate::command_test_support::{fixture, git, git_status, os_arguments, output_text, remove, repository};
/// The refusal text,
///  whose words the README has to carry too.
use crate::rule_shared_remote_messages::shared_remote_settings_message;
/// A linked worktree's shared configuration file is what the real-Git facts below read.
use std::path::{Path, PathBuf};

/// The remote URL every fixture repository starts with.
const ORIGIN_URL: &str = "https://example.invalid/origin.git";

/// The whole decision for a space-separated command line,
///  split at its first space into the word and the region.
fn decided_change(line: &str) -> Option<SharedRemoteChange> {
    let values: Vec<&str> = line.split(' ').collect();
    // `split_first` is `Some((first, rest))` for any non-empty list.
    let (word, region) = values.split_first().expect("a command word");
    return shared_remote_write(word.as_bytes(), os_arguments(region).as_slice());
}

/// The setting the decision for a space-separated command line names,
///  which is every case about a file the command does not name.
fn decided(line: &str) -> Option<SharedRemoteWrite> {
    return decided_change(line).map(|change| { return change.setting; });
}

/// Where one spelling of a key or section belongs,
///  as the decision names it.
fn key(name: &str) -> Option<SharedRemoteWrite> {
    return Some(SharedRemoteWrite::Key(String::from(name)));
}

/// Where one `git remote` subcommand belongs.
fn remote(name: &'static str) -> Option<SharedRemoteWrite> {
    return Some(SharedRemoteWrite::RemoteSubcommand(name));
}

/// Where one section belongs.
fn section(name: &str) -> Option<SharedRemoteWrite> {
    return Some(SharedRemoteWrite::Section(String::from(name)));
}

/// Every `git remote` subcommand that writes the shared configuration is refused,
///  in every position Git accepts its own options.
#[test]
fn every_remote_subcommand_that_writes_the_shared_configuration_is_refused() {
    for (line, expected) in [
        ("remote set-url origin https://example.invalid/two.git", remote("set-url")),
        ("remote set-url --push origin DISABLED", remote("set-url")),
        ("remote set-url --add origin https://example.invalid/three.git", remote("set-url")),
        ("remote set-url --delete origin https://example.invalid/three.git", remote("set-url")),
        ("remote add beta https://example.invalid/beta.git", remote("add")),
        ("remote remove beta", remote("remove")),
        ("remote rm beta", remote("rm")),
        ("remote rename origin two", remote("rename")),
        ("remote set-branches origin main", remote("set-branches")),
        ("remote set-head origin main", remote("set-head")),
        // The options Git reads before a subcommand hide nothing.
        ("remote -v set-url origin https://example.invalid/two.git", remote("set-url")),
        ("remote --verbose add beta https://example.invalid/beta.git", remote("add")),
        ("remote --verb add beta https://example.invalid/beta.git", remote("add")),
        ("remote -vv -v rm beta", remote("rm")),
        // The subcommand's own options are not the subcommand.
        ("remote add -t main beta https://example.invalid/beta.git", remote("add")),
        ("remote rename --progress origin two", remote("rename")),
    ] {
        assert_eq!(decided(line), expected, "{line}");
    }
}

/// The subcommands that read,
///  and the regions Git itself refuses,
///  are left alone.
#[test]
fn reads_and_regions_git_refuses_are_left_alone() {
    for line in [
        "remote",
        "remote -v",
        "remote --verbose",
        "remote get-url origin",
        "remote get-url --push origin",
        "remote show origin",
        "remote prune origin",
        "remote update",
        // Git refuses an abbreviated subcommand, so nothing is written.
        "remote set-u origin https://example.invalid/two.git",
        "remote -- add beta https://example.invalid/beta.git",
        "remote --bogus add beta https://example.invalid/beta.git",
        "status",
        "",
    ] {
        assert_eq!(decided(line), None, "{line}");
    }
}

/// Every writing form of `git config` on a key under `remote.` is refused,
///  in the legacy spelling and in the subcommand spelling.
#[test]
fn every_config_write_to_a_remote_key_is_refused() {
    for (line, expected) in [
        ("config remote.origin.url https://example.invalid/two.git", key("remote.origin.url")),
        ("config --local remote.origin.url https://example.invalid/two.git", key("remote.origin.url")),
        // Git's section names are case-insensitive, so this writes the same remote.
        ("config Remote.Origin.URL https://example.invalid/two.git", key("Remote.Origin.URL")),
        ("config remote..url https://example.invalid/two.git", key("remote..url")),
        ("config --add remote.origin.pushurl DISABLED", key("remote.origin.pushurl")),
        ("config --replace-all remote.origin.url https://example.invalid/two.git", key("remote.origin.url")),
        ("config --unset remote.origin.url", key("remote.origin.url")),
        ("config --unset-all remote.origin.url", key("remote.origin.url")),
        ("config set remote.origin.url https://example.invalid/two.git", key("remote.origin.url")),
        ("config set --append remote.origin.url https://example.invalid/two.git", key("remote.origin.url")),
        ("config set --all remote.origin.url https://example.invalid/two.git", key("remote.origin.url")),
        ("config unset remote.origin.url", key("remote.origin.url")),
        ("config unset --all remote.origin.url", key("remote.origin.url")),
        // A scope spelling after the key is a value Git stores in the shared file.
        ("config remote.origin.url --worktree", key("remote.origin.url")),
        ("config remote.origin.url --global", key("remote.origin.url")),
        ("config --remove-section remote.origin", section("remote.origin")),
        ("config --remove-section Remote.Origin", section("Remote.Origin")),
        ("config remove-section remote.origin", section("remote.origin")),
        // A rename creates remote settings just as removing them takes remote settings away.
        ("config --rename-section user.kept remote.created", section("remote.created")),
        ("config --rename-section remote.origin user.kept", section("remote.origin")),
        ("config rename-section remote.origin user.kept", section("remote.origin")),
        // Three arguments is Git's set-all, a write as well.
        ("config remote.origin.url one two", key("remote.origin.url")),
    ] {
        assert_eq!(decided(line), expected, "{line}");
    }
}

/// A read,
///  another section,
///  a blob and a region Git refuses name nothing this rule refuses.
#[test]
fn reads_other_sections_and_regions_git_refuses_are_left_alone() {
    for line in [
        "config remote.origin.url",
        "config --get remote.origin.url",
        "config get remote.origin.url",
        "config --get-all remote.origin.url",
        "config --get-regexp remote\\..*",
        "config list",
        "config --list",
        "config -l",
        // Another file is not the one every worktree shares.
        "config --global remote.origin.url https://example.invalid/two.git",
        "config --system remote.origin.url https://example.invalid/two.git",
        "config --worktree remote.origin.url https://example.invalid/two.git",
        "config --blob HEAD:.gitmodules remote.origin.url https://example.invalid/two.git",
        // An editor of another file changes no setting of this repository.
        "config --global --edit",
        "config --system --edit",
        "config --worktree --edit",
        // Git refuses the subcommand form with any argument ("wrong number of arguments, should be 0").
        "config edit user.name",
        // Another section.
        "config user.origin.url https://example.invalid/two.git",
        "config --unset user.origin.url",
        "config --remove-section user.origin",
        "config --rename-section user.kept user.created",
        // A region Git refuses runs nothing.
        "config remote.origin.url one two three",
        "config remote.origin.url one two three four",
        "config --edit --get remote.origin.url",
        "config --edit --remove-section remote.origin",
        "config",
    ] {
        assert_eq!(decided(line), None, "{line}");
    }
}

/// An editor session on the shared file is the same write with no key named,
///  so it is refused wherever it can reach a remote setting.
#[test]
fn an_editor_session_on_the_shared_file_is_refused() {
    for line in [
        "config --edit",
        "config -e",
        "config edit",
        // The legacy form runs the editor whatever follows it, as real Git does below.
        "config --edit user.name",
        "config --local --edit",
        "config edit --local",
    ] {
        assert_eq!(decided(line), Some(SharedRemoteWrite::Edit), "{line}");
    }
}

/// A write that names a file is this rule's only once the caller compares the path with the
///  shared file,
///  so the decision carries it.
#[test]
fn a_write_that_names_a_file_carries_its_path() {
    let origin_url: SharedRemoteWrite = SharedRemoteWrite::Key(String::from("remote.origin.url"));
    let origin_section: SharedRemoteWrite = SharedRemoteWrite::Section(String::from("remote.origin"));
    for (line, path, setting) in [
        (
            "config --file other.config remote.origin.url https://example.invalid/two.git",
            "other.config",
            origin_url.clone(),
        ),
        (
            "config --file=other.config remote.origin.url https://example.invalid/two.git",
            "other.config",
            origin_url.clone(),
        ),
        (
            "config -f other.config remote.origin.url https://example.invalid/two.git",
            "other.config",
            origin_url.clone(),
        ),
        (
            "config set --file other.config remote.origin.url https://example.invalid/two.git",
            "other.config",
            origin_url.clone(),
        ),
        (
            "config --file other.config --remove-section remote.origin",
            "other.config",
            origin_section,
        ),
        ("config --file other.config --edit", "other.config", SharedRemoteWrite::Edit),
        // The last use wins, and a relative path is kept exactly as written for the caller to resolve.
        (
            "config --file first.config --file ../other.config remote.origin.url x",
            "../other.config",
            origin_url,
        ),
    ] {
        let Some(change) = decided_change(line) else {
            panic!("{line}");
        };
        assert_eq!(change.named_file, Some(PathBuf::from(path)), "{line}");
        assert_eq!(change.setting, setting, "{line}");
    }
    // A read,
    //  a write to another section and a blob name no path this rule asks about.
    for line in [
        "config --file other.config remote.origin.url",
        "config --file other.config user.name x",
        "config --file other.config --remove-section user.origin",
        "config --blob HEAD:.gitmodules remote.origin.url https://example.invalid/two.git",
    ] {
        assert_eq!(decided_change(line), None, "{line}");
    }
}

/// The section every worktree shares is `$GIT_COMMON_DIR/config`,
///  so both a linked worktree inside and one outside the repository folder write it.
#[test]
fn a_linked_worktree_writes_the_configuration_file_the_main_checkout_owns() {
    let directory: PathBuf = fixture("shared-remote-worktree");
    let root: PathBuf = repository(directory.as_path(), "repository");
    git(root.as_path(), &["remote", "add", "origin", ORIGIN_URL]);
    let inside: PathBuf = root.join("linked-inside");
    let outside: PathBuf = directory.join("linked-outside");
    for place in [inside.as_path(), outside.as_path()] {
        git(
            root.as_path(),
            &["worktree", "add", "--detach", place.to_str().expect("text path"), "HEAD"],
        );
    }
    let shared: PathBuf = root.join(".git").join("config");
    let before: Vec<u8> = std::fs::read(shared.as_path()).expect("shared configuration");
    for place in [inside.as_path(), outside.as_path()] {
        git(place, &["remote", "set-url", "--push", "origin", "DISABLED"]);
        let after: String = String::from_utf8_lossy(&std::fs::read(shared.as_path()).expect("shared configuration")).into_owned();
        assert_ne!(after.as_bytes(), before.as_slice(), "{} wrote nothing shared", place.display());
        assert!(after.contains("DISABLED"), "{} wrote elsewhere: {after}", place.display());
        // Restore the file so the next worktree starts from the same bytes.
        std::fs::write(shared.as_path(), before.as_slice()).expect("shared configuration restored");
    }
    remove(directory.as_path());
}

/// The decisions follow Git's own reading of the three scope questions they rest on.
#[test]
fn the_scope_facts_match_what_git_writes() {
    let directory: PathBuf = fixture("shared-remote-scopes");
    let root: PathBuf = repository(directory.as_path(), "repository");
    let shared: PathBuf = root.join(".git").join("config");
    // A scope spelling after the key is a value, stored in the shared file.
    git(root.as_path(), &["config", "remote.origin.url", "--worktree"]);
    assert_eq!(output_text(&git(root.as_path(), &["config", "--local", "--get", "remote.origin.url"])), "--worktree");
    assert_eq!(decided("config remote.origin.url --worktree"), key("remote.origin.url"));
    // A section rename creates the section it names under the new spelling.
    git(root.as_path(), &["config", "user.kept.url", "one"]);
    git(root.as_path(), &["config", "--rename-section", "user.kept", "remote.created"]);
    assert!(output_text(&git(root.as_path(), &["config", "--get", "remote.created.url"])).contains("one"));
    assert_eq!(decided("config --rename-section user.kept remote.created"), section("remote.created"));
    // A per-worktree write needs the extension, and then it lands in no shared file.
    let linked: PathBuf = root.join("linked");
    git(
        root.as_path(),
        &["worktree", "add", "--detach", linked.to_str().expect("text path"), "HEAD"],
    );
    let no_extension: std::process::Output =
        git_status(linked.as_path(), &["config", "--worktree", "remote.origin.url", "own"]);
    assert!(!no_extension.status.success(), "Git accepted --worktree without the extension");
    git(root.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    let before: Vec<u8> = std::fs::read(shared.as_path()).expect("shared configuration");
    git(linked.as_path(), &["config", "--worktree", "remote.origin.url", "own"]);
    assert_eq!(
        std::fs::read(shared.as_path()).expect("shared configuration"),
        before,
        "a per-worktree write changed the shared file"
    );
    let own: PathBuf = root.join(".git").join("worktrees").join("linked").join("config.worktree");
    assert!(std::fs::read_to_string(own.as_path()).expect("per-worktree configuration").contains("own"));
    assert_eq!(decided("config --worktree remote.origin.url own"), None);
    // The negative control: the same Git refuses the abbreviated subcommand the decision leaves alone.
    let abbreviated: std::process::Output =
        git_status(root.as_path(), &["remote", "set-u", "origin", "x"]);
    assert!(!abbreviated.status.success(), "Git accepted an abbreviated subcommand");
    remove(directory.as_path());
}

/// The code stays a stable,
///  self-explaining finding code.
#[test]
fn the_finding_code_names_the_shared_setting() {
    assert_eq!(SHARED_REMOTE_SETTINGS_CODE, "shared-remote-settings");
}

/// The README documents the facts and the ways forward the refusal names,
///  as it documents every other rejection's alternatives.
#[test]
fn the_readme_documents_what_the_refusal_recommends() {
    let readme: &str = include_str!("../../../README.md");
    assert!(readme.contains("### The remote settings every worktree shares"));
    let message: String = shared_remote_settings_message(
        &SharedRemoteWrite::RemoteSubcommand("set-url"),
        Path::new("/repo/.git/config"),
        Path::new("/repo"),
    );
    let said: [&str; 3] = [
        "every worktree of this repository shares one Git configuration",
        "git clone --no-hardlinks",
        "--no-enforce-worktree",
    ];
    for phrase in said {
        assert!(message.contains(phrase), "the text lacks {phrase}");
    }
    let documented: [&str; 5] = [
        "Every worktree of a repository shares one `.git/config`",
        "git clone --no-hardlinks",
        "--no-enforce-worktree",
        "git remote --no-enforce-worktree set-url origin <url>",
        "git config --no-enforce-worktree remote.origin.url <value>",
    ];
    for phrase in documented {
        assert!(readme.contains(phrase), "the README lacks {phrase}");
    }
}

/// The guard decides the shared-remote write on the command Git will run,
///  so the refusal is one decision and a shell alias reaches it too.
#[test]
fn the_guard_decides_the_shared_remote_write() {
    use crate::rule_linked_worktree::{LinkedWorktreeDecision, decide_linked_worktree};
    let cases: [(&[&str], Option<SharedRemoteWrite>); 6] = [
        (
            &["remote", "set-url", "origin", "https://example.invalid/two.git"],
            remote("set-url"),
        ),
        (
            &["config", "--add", "remote.origin.url", "https://example.invalid/two.git"],
            key("remote.origin.url"),
        ),
        (
            &["config", "--remove-section", "remote.origin"],
            section("remote.origin"),
        ),
        // The subcommand's own options are not the guard's, and a read is no write.
        (&["remote", "-v"], None),
        (&["remote", "add", "-t", "main", "beta", "https://example.invalid/beta.git"], remote("add")),
        (&["status"], None),
    ];
    for (values, expected) in cases {
        let decided: LinkedWorktreeDecision = decide_linked_worktree(os_arguments(values).as_slice());
        let found: Option<SharedRemoteWrite> = match decided {
            LinkedWorktreeDecision::SharedRemote(change) => Some(change.setting),
            _ => None,
        };
        assert_eq!(found, expected, "{values:?}");
    }
}
