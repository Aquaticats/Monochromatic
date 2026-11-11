//! What:
//!  Forms,
//!  file scopes and option positions of `git config` regions,
//!  with real Git
//!       2.56.0 controls for every table and for the positions the incumbent misread.
//! Why:
//!  `--global` after a variable name is a value Git stores in the repository;
//!  reading
//!      it as the scope option would exempt a command that needs the repository root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseConfigRegion(['user.name', '--global']).global).toBe(false);
//! ```

/// The parser,
///  its tables,
///  the oracles and the real-Git fixture helpers.
use super::ConfigForm::{Edit, Get, Legacy, List, RemoveSection, RenameSection, Set, Unset};
use super::ConfigFile::{Global, Named, Shared, System, Worktree};
use super::ConfigWrite::{Edit as EditAction, Key, Nothing};
use super::ConfigWrite::{RenameSection as RenameAction, RemoveSection as RemoveAction};
use super::{ConfigFile, ConfigForm, ConfigRegion, config_table, parse_config_region};
use crate::command_options::OptionErrorKind::{
    AmbiguousOption, HelpRequested, MissingValue, UnexpectedValue, UnknownOption,
};
use crate::command_options::{OptionError, WrapperOccurrence};
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, fixture, git, git_status, os_arguments, output_text, remove,
    repository,
};
use std::path::PathBuf;
use std::process::Output;

/// Parse a space-separated region Git accepts,
///  with no wrapper flags.
fn region(line: &str) -> ConfigRegion {
    let values: Vec<&str> = line.split_whitespace().collect();
    return parse_config_region(os_arguments(values.as_slice()).as_slice(), &[])
        .expect("valid region");
}

/// The facts of a region as `(form, global, system, lists)`.
fn facts(line: &str) -> (ConfigForm, bool, bool, bool) {
    let found: ConfigRegion = region(line);
    return (found.form, found.global, found.system, found.lists);
}

/// Only an exact first word selects a subcommand form;
///  anything else is the legacy form.
#[test]
fn picks_the_form_from_the_first_token() {
    for (line, form) in [
        ("list", List),
        ("get user.name", Get),
        ("set user.name x", Set),
        ("unset user.name", Unset),
        ("rename-section a b", RenameSection),
        ("remove-section a", RemoveSection),
        ("edit", Edit),
        ("", Legacy),
        ("user.name", Legacy),
        ("user.name list", Legacy),
        ("--list", Legacy),
        ("--global list", Legacy),
        ("-- list", Legacy),
        ("lis", Legacy),
        ("LIST", Legacy),
        ("--get user.name", Legacy),
        ("-", Legacy),
    ] {
        assert_eq!(region(line).form, form, "{line}");
    }
}

/// The scope options and the legacy listing action,
///  in every spelling Git accepts.
#[test]
fn reads_scope_and_listing_in_option_position() {
    for (line, expected) in [
        ("--global user.name", (Legacy, true, false, false)),
        ("--system user.name", (Legacy, false, true, false)),
        ("--glob user.name", (Legacy, true, false, false)),
        ("--sys --get user.name", (Legacy, false, true, false)),
        ("--list", (Legacy, false, false, true)),
        ("-l", (Legacy, false, false, true)),
        ("--li", (Legacy, false, false, true)),
        ("-lz", (Legacy, false, false, true)),
        ("-zl", (Legacy, false, false, true)),
        ("--global -l", (Legacy, true, false, true)),
        ("-- --global user.name", (Legacy, true, false, false)),
        ("list --global", (List, true, false, true)),
        ("list --show-origin --system", (List, false, true, true)),
        ("list", (List, false, false, true)),
        ("get --global user.name", (Get, true, false, false)),
        ("set --system user.name x", (Set, false, true, false)),
        ("unset --global user.name", (Unset, true, false, false)),
        (
            "rename-section --global a b",
            (RenameSection, true, false, false),
        ),
        (
            "remove-section --system a",
            (RemoveSection, false, true, false),
        ),
        ("edit --global", (Edit, true, false, false)),
        ("--local user.name", (Legacy, false, false, false)),
        ("--get user.name", (Legacy, false, false, false)),
    ] {
        assert_eq!(facts(line), expected, "{line}");
    }
}

/// Git applies options in order:
///  a later `--no-global` switches the scope back off.
#[test]
fn scope_is_the_final_state() {
    assert_eq!(
        facts("--global --no-global user.name"),
        (Legacy, false, false, false)
    );
    assert_eq!(
        facts("--no-global --global user.name"),
        (Legacy, true, false, false)
    );
    assert_eq!(
        facts("list --system --no-system"),
        (List, false, false, true)
    );
}

/// Divergence:
///  after the first name,
///  and in a value position,
///  the spellings are data.
#[test]
fn spellings_outside_option_position_are_data() {
    for line in [
        "user.name --global",
        "user.name --system",
        "user.name -l",
        "user.name --list",
        "set user.name --global",
        "set user.name -l",
        "get user.name --system",
        "unset user.name --global",
        "-f --global user.name",
        "--file --system user.name",
        "--blob --list user.name",
        "--default --global --get user.name",
        "--comment --system user.name x",
        "-t --global user.name",
        "get --value --global user.name",
        "get --url --system user.name",
        "--end-of-options --global",
        "list --end-of-options --global",
        "get -- --global",
    ] {
        let found: ConfigRegion = region(line);
        assert!(!found.global && !found.system, "{line}");
        assert_eq!(found.lists, found.form == List, "{line}");
    }
}

/// `list` and `edit` read options after a name too;
///  the other forms stop at the first name.
#[test]
fn only_list_and_edit_read_options_after_a_name() {
    assert!(region("list extra --global").global);
    assert!(region("edit extra --system").system);
    assert!(!region("get extra --global").global);
    assert!(!region("rename-section a --global b").global);
    assert!(!region("remove-section a --system").system);
}

/// Wrapper-only flags count in option position,
///  with region token indexes.
#[test]
fn reports_wrapper_flags_by_region_position() {
    let flags: [&[u8]; 1] = [b"--no-enforce-require-root"];
    for (values, expected) in [
        (vec!["--no-enforce-require-root", "--global", "a"], vec![0]),
        (
            vec!["set", "--no-enforce-require-root", "--global", "a", "b"],
            vec![1],
        ),
        (vec!["--", "--no-enforce-require-root", "a"], vec![1]),
        (vec!["list", "x", "--no-enforce-require-root"], vec![2]),
        (vec!["a", "--no-enforce-require-root"], vec![]),
        (vec!["set", "a", "--no-enforce-require-root"], vec![]),
        (vec!["-f", "--no-enforce-require-root", "a"], vec![]),
        // Leading flags are skipped before the form is picked.
        (
            vec![
                "--no-enforce-require-root",
                "--no-enforce-require-root",
                "list",
            ],
            vec![0, 1],
        ),
        (vec!["--no-enforce-require-root", "--", "a"], vec![0]),
    ] {
        let found: ConfigRegion =
            parse_config_region(os_arguments(values.as_slice()).as_slice(), &flags)
                .expect("valid region");
        let mut tokens: Vec<usize> = Vec::<usize>::new();
        for occurrence in &found.wrapper {
            assert_eq!(
                *occurrence,
                WrapperOccurrence {
                    flag: 0,
                    token: occurrence.token
                }
            );
            tokens.push(occurrence.token);
        }
        assert_eq!(tokens, expected, "{values:?}");
    }
    // The wrapper removes leading flags, so Git sees `config list --global` and
    // `config -- --global a`: the form and the scope are read after the flags.
    let keep_going: [&[u8]; 2] = [b"--no-enforce-require-root", b"--cli-git-keep-going"];
    let listed: ConfigRegion = parse_config_region(
        os_arguments(&["--cli-git-keep-going", "list", "--global"]).as_slice(),
        &keep_going,
    )
    .expect("valid region");
    assert_eq!(
        (listed.form, listed.global, listed.lists),
        (List, true, true)
    );
    assert_eq!(
        listed.wrapper,
        vec![WrapperOccurrence { flag: 1, token: 0 }]
    );
    let separated: ConfigRegion = parse_config_region(
        os_arguments(&["--cli-git-keep-going", "--", "--global", "a"]).as_slice(),
        &keep_going,
    )
    .expect("valid region");
    assert_eq!((separated.form, separated.global), (Legacy, true));
    // A refusal after leading flags names its region index.
    let refused: OptionError = parse_config_region(
        os_arguments(&["--cli-git-keep-going", "get", "-l"]).as_slice(),
        &keep_going,
    )
    .expect_err("refused");
    assert_eq!((refused.kind, refused.token), (UnknownOption, 2));
}

/// A region Git refuses is reported as refused,
///  at its region token index.
#[test]
fn reports_what_git_refuses() {
    for (line, kind, token) in [
        ("--bogus", UnknownOption, 0),
        ("--no-list", UnknownOption, 0),
        ("--no-bool", UnknownOption, 0),
        ("get -l", UnknownOption, 1),
        ("list --list", UnknownOption, 1),
        ("set --regexp a b", UnknownOption, 1),
        ("unset --append a", UnknownOption, 1),
        ("edit -z", UnknownOption, 1),
        ("-- --bogus", UnknownOption, 1),
        ("-f", MissingValue, 0),
        ("get --url", MissingValue, 1),
        ("--global=1 a", UnexpectedValue, 0),
        ("--l", AmbiguousOption, 0),
        ("--ge a", AmbiguousOption, 0),
        ("-h", HelpRequested, 0),
        ("list -h", HelpRequested, 1),
    ] {
        let values: Vec<&str> = line.split_whitespace().collect();
        let refused: OptionError =
            parse_config_region(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused");
        assert_eq!((refused.kind, refused.token), (kind, token), "{line}");
    }
    // `--l` is ambiguous only where `--list` exists beside `--local`.
    assert_eq!(facts("get --l user.name"), (Get, false, false, false));
}

/// Each assembled table matches what the binary prints for that form.
#[test]
fn tables_match_git() {
    let directory: PathBuf = fixture("config-tables");
    let root: PathBuf = repository(directory.as_path(), "repository");
    for (form, command) in [
        (List, vec!["config", "list"]),
        (Get, vec!["config", "get"]),
        (Set, vec!["config", "set"]),
        (Unset, vec!["config", "unset"]),
        (RenameSection, vec!["config", "rename-section"]),
        (RemoveSection, vec!["config", "remove-section"]),
        (Edit, vec!["config", "edit"]),
        // The leading `--` is dropped by Git's first look, so the legacy parser answers.
        (Legacy, vec!["config", "--"]),
    ] {
        let table: Vec<crate::command_options::OptionSpec> = config_table(form);
        assert_table_invariants(table.as_slice());
        assert_eq!(
            render_completion(table.as_slice()),
            git_completion(root.as_path(), command.as_slice()),
            "{form:?}"
        );
    }
    remove(directory.as_path());
}

/// Real Git stores,
///  reads and lists exactly where the parser says the scope applies.
///  The
/// fixture's per-user file does not exist,
///  so a per-user read prints nothing and fails.
#[test]
fn scope_positions_match_git() {
    let directory: PathBuf = fixture("config-scope");
    let root: PathBuf = repository(directory.as_path(), "repository");
    // After the name both spellings are values, stored in the repository's own file.
    git(root.as_path(), &["config", "user.name", "--global"]);
    git(root.as_path(), &["config", "set", "user.email", "-l"]);
    let local: Output = git(
        root.as_path(),
        &["config", "--local", "--get-regexp", "^user"],
    );
    assert_eq!(output_text(&local), "user.name --global\nuser.email -l");
    // In the value position of `-f` the spelling names a file.
    git(
        root.as_path(),
        &["config", "-f", "--global", "user.name", "x"],
    );
    assert!(root.join("--global").is_file());
    // In option position the scope applies: the per-user file has no such variable.
    for line in [
        "--global user.name",
        "--glob user.name",
        "-- --global user.name",
    ] {
        let values: Vec<&str> = line.split_whitespace().collect();
        let mut full: Vec<&str> = vec!["config"];
        full.extend_from_slice(values.as_slice());
        let read: Output = git_status(root.as_path(), full.as_slice());
        assert!(!read.status.success() && read.stdout.is_empty(), "{line}");
        assert!(region(line).global, "{line}");
    }
    // A later `--no-global` reads the repository's file again.
    let reread: Output = git(
        root.as_path(),
        &["config", "--global", "--no-global", "user.name"],
    );
    assert_eq!(output_text(&reread), "--global");
    // A clustered `-l` lists; `list` after an option is a variable name, not the subcommand.
    let listed: Output = git(root.as_path(), &["config", "-lz"]);
    assert!(String::from_utf8_lossy(&listed.stdout).contains("user.name\n--global\0"));
    let named: Output = git_status(root.as_path(), &["config", "--global", "list"]);
    assert!(!named.status.success());
    assert!(String::from_utf8_lossy(&named.stderr).contains("section"));
    remove(directory.as_path());
}

/// The file a region reads or writes and what it writes there,
///  in every spelling Git accepts.
#[test]
fn reports_the_file_and_the_write_of_a_region() {
    for (line, file, write) in [
        // No scope option is the repository's own file, which every worktree shares.
        ("remote.origin.url one", Shared, Key),
        ("--local remote.origin.url one", Shared, Key),
        ("--add remote.origin.url one", Shared, Key),
        ("--replace-all remote.origin.url one", Shared, Key),
        ("--unset remote.origin.url", Shared, Key),
        ("--unset-all remote.origin.url", Shared, Key),
        ("set remote.origin.url one", Shared, Key),
        ("set --append remote.origin.url one", Shared, Key),
        ("unset remote.origin.url", Shared, Key),
        ("unset --all remote.origin.url", Shared, Key),
        ("--remove-section remote.origin", Shared, RemoveAction),
        ("remove-section remote.origin", Shared, RemoveAction),
        ("--rename-section a.b c.d", Shared, RenameAction),
        ("rename-section a.b c.d", Shared, RenameAction),
        // Another file is named by the scope option.
        ("--global remote.origin.url one", Global, Key),
        ("--system remote.origin.url one", System, Key),
        ("--worktree remote.origin.url one", Worktree, Key),
        ("--file other remote.origin.url one", Named, Key),
        ("--blob HEAD:.gitmodules remote.origin.url one", Named, Key),
        // Two scope options at once is a region Git refuses.
        ("--global --local remote.origin.url one", ConfigFile::Contradictory, Key),
        // Reads and the editor write no key of their own.
        ("remote.origin.url", Shared, Nothing),
        ("--get remote.origin.url", Shared, Nothing),
        ("--get-all remote.origin.url", Shared, Nothing),
        ("--list", Shared, Nothing),
        ("list", Shared, Nothing),
        ("get remote.origin.url", Shared, Nothing),
        ("--edit", Shared, EditAction),
        ("edit", Shared, EditAction),
        // The subcommand form takes no argument ("wrong number of arguments, should be 0"),
        // while the legacy `--edit` runs the editor whatever follows it.
        ("edit user.name", Shared, Nothing),
        ("--edit user.name", Shared, EditAction),
        // Git's argument count decides the form without a subcommand word:
        //  one argument reads,
        //  two store,
        //  three replace the values a pattern selects,
        //  and every other count is refused.
        ("remote.origin.url one two", Shared, Key),
        ("remote.origin.url one two three", Shared, Nothing),
        // A spelling after the key is a value, so three arguments still write.
        ("remote.origin.url --get one", Shared, Key),
    ] {
        let found: ConfigRegion = region(line);
        assert_eq!((found.file, found.write), (file, write), "{line}");
    }
}

/// The path `--file` names is reported for the caller that compares it with the file every
///  worktree shares,
///  and `--blob` names no path at all.
#[test]
fn reports_the_file_a_region_names() {
    for (line, expected) in [
        ("--file other remote.origin.url one", Some("other")),
        ("--file=other remote.origin.url one", Some("other")),
        ("-f other remote.origin.url one", Some("other")),
        ("set --file other remote.origin.url one", Some("other")),
        // The last use wins, as Git's own variables do.
        ("--file first --file second remote.origin.url one", Some("second")),
        ("--file other --edit", Some("other")),
        // A read names the file it reads, and this reader reports it all the same.
        ("--file other remote.origin.url", Some("other")),
        // A blob is no path, and a region Git refuses names neither.
        ("--blob HEAD:.gitmodules remote.origin.url one", None),
        // Two scope options at once is a region Git refuses, and the path it names is still reported
        //  for the one file the region would have read.
        ("--file other --global remote.origin.url one", Some("other")),
        ("remote.origin.url one", None),
        ("--global remote.origin.url one", None),
    ] {
        assert_eq!(region(line).named_file, expected.map(PathBuf::from), "{line}");
    }
    // Real Git writes the file this reader names, leaves the shared one as it was,
    //  and refuses the subcommand form with an argument.
    let directory: PathBuf = fixture("config-named-file");
    let root: PathBuf = repository(directory.as_path(), "repository");
    let shared: PathBuf = root.join(".git").join("config");
    let before: Vec<u8> = std::fs::read(shared.as_path()).expect("shared configuration");
    let named: PathBuf = root.join("other");
    git(
        root.as_path(),
        &["config", "--file", "other", "remote.origin.url", "one"],
    );
    let written: String = std::fs::read_to_string(named.as_path()).expect("named configuration");
    assert!(written.contains("one"), "the named file was not written: {written}");
    assert_eq!(
        std::fs::read(shared.as_path()).expect("shared configuration"),
        before,
        "a write to a named file changed the shared configuration"
    );
    let refused: Output = git_status(root.as_path(), &["config", "edit", "user.name"]);
    assert!(!refused.status.success(), "Git accepted an argument after the edit subcommand");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("wrong number of arguments"),
        "Git refused the edit subcommand for another reason: {}",
        String::from_utf8_lossy(&refused.stderr)
    );
    remove(directory.as_path());
}

/// The tokens Git hands the command as arguments are reported by region position,
///  option values excluded.
#[test]
fn reports_the_arguments_git_reads() {
    for (line, expected) in [
        ("remote.origin.url one two", vec![0, 1, 2]),
        ("--add remote.origin.url one", vec![1, 2]),
        ("set remote.origin.url one", vec![1, 2]),
        // `--comment` takes a value, so the key follows it.
        ("--comment note remote.origin.url one", vec![2, 3]),
        ("--remove-section remote.origin", vec![1]),
        ("--comment note --add remote.origin.url one", vec![3, 4]),
        ("--get -- remote.origin.url", vec![2]),
        ("list", Vec::<usize>::new()),
    ] {
        assert_eq!(region(line).arguments, expected, "{line}");
    }
}

/// The scope the facts report is the file Git writes,
///  proved by where the value lands.
#[test]
fn the_scope_facts_match_where_git_writes() {
    let directory: PathBuf = fixture("config-write-scopes");
    let root: PathBuf = repository(directory.as_path(), "repository");
    // The default and `--local` write the repository's own file.
    git(root.as_path(), &["config", "remote.origin.url", "local-one"]);
    git(root.as_path(), &["config", "--local", "remote.origin.pushurl", "local-two"]);
    let shared: Output = git(root.as_path(), &["config", "--local", "--get-regexp", "^remote\\.origin\\."]);
    assert_eq!(output_text(&shared), "remote.origin.url local-one\nremote.origin.pushurl local-two");
    assert_eq!((region("remote.origin.url local-one").file, region("--local remote.origin.url local-one").file), (Shared, Shared));
    // A named file is not the repository's file.
    let named: PathBuf = root.join("named.config");
    git(root.as_path(), &["config", "--file", named.to_str().expect("text path"), "remote.origin.url", "named"]);
    assert!(std::fs::read_to_string(named.as_path()).expect("named configuration").contains("named"));
    assert_eq!(
        output_text(&git(root.as_path(), &["config", "--local", "--get", "remote.origin.url"])),
        "local-one"
    );
    assert_eq!(region("--file other remote.origin.url named").file, Named);
    // A per-worktree write needs the extension, and then the repository's own file keeps its values.
    let linked: PathBuf = root.join("linked");
    git(
        root.as_path(),
        &["worktree", "add", "--detach", linked.to_str().expect("text path"), "HEAD"],
    );
    let without: Output = git_status(linked.as_path(), &["config", "--worktree", "remote.origin.url", "own"]);
    assert!(!without.status.success(), "Git accepted --worktree without the extension");
    git(root.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    git(linked.as_path(), &["config", "--worktree", "remote.origin.url", "own"]);
    assert_eq!(
        output_text(&git(root.as_path(), &["config", "--local", "--get", "remote.origin.url"])),
        "local-one"
    );
    assert_eq!(
        output_text(&git(linked.as_path(), &["config", "--worktree", "--get", "remote.origin.url"])),
        "own"
    );
    assert_eq!(region("--worktree remote.origin.url own").file, Worktree);
    remove(directory.as_path());
}
