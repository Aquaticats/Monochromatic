//! Controls for the filesystem section of the doctor:
//! the wording for every case on built facts,
//! the kernel command
//!  line parsing,
//! the facts read from a real fixture,
//! and the section's place in the doctor's output and exit status.

use super::*;
use crate::action::Action;
use crate::doctor::{doctor_action, doctor_action_with_section};
use crate::test_support::{executable, fixture, git, git_context, remove, repository};
use std::time::Duration;
use crate::transaction_git::GitContext;
use std::ffi::OsStr;
use std::os::unix::fs::MetadataExt;

/// The checkout path the built facts use.
const ROOT: &str = "/work/checkout";
/// The mount point the built facts use.
const MOUNT: &str = "/work";

/// The source the built mount and the built root mount share,
/// unless a control separates them.
const SOURCE: &str = "/dev/mapper/root";

/// The production filesystem section,
/// read for the checkout Git named for the command.
fn section_of_the_named_checkout(viewpoint: &crate::doctor::CheckoutViewpoint<'_>) -> String {
    return match viewpoint.checkout {
        Some(checkout) => checkout_section(checkout, viewpoint.layout),
        None => String::new(),
    };
}

/// The doctor's action with a filesystem section,
/// its report built by a task of the runtime,
/// because every Git question of the report is awaited there.
fn with_section(
    context: &GitContext,
    cwd: &Path,
    home: Option<&Path>,
    section: &dyn Fn(&crate::doctor::CheckoutViewpoint<'_>) -> String,
) -> (i32, String, String) {
    let action: Action = crate::lazy_runtime::run_on_runtime(doctor_action_with_section(context, cwd, home, section)).expect("the runtime");
    return match action {
        Action::Exit { code, stdout, stderr } => (code, stdout, stderr),
        Action::ExitBytes { code, stdout, stderr } => (
            code,
            String::from_utf8(stdout).expect("a report of text paths"),
            String::from_utf8(stderr).expect("an explanation of text paths"),
        ),
        other => panic!("an exit action, not {other:?}"),
    };
}

/// Facts for a btrfs checkout on an ostree host with one `rootflags=` word.
fn facts(kind: FilesystemKind, root_is_subvolume: bool, has_removal_option: Option<bool>) -> FilesystemFacts {
    return FilesystemFacts {
        root: PathBuf::from(ROOT),
        layout: CheckoutLayout::Main,
        kind,
        root_is_subvolume,
        mount: has_removal_option.map(|active: bool| { return MountSummary { mount_point: PathBuf::from(MOUNT), source: String::from(SOURCE), has_removal_option: active }; }),
        root_source: Some(String::from(SOURCE)),
        ostree_booted: true,
        kernel_rootflags: vec![String::from("subvol=root,compress=zstd:1")],
    };
}

/// Every line of a rendered section ends the section text with a line feed and the first carries the marker.
fn assert_shape(text: &str) {
    assert!(text.starts_with(SECTION_MARKER) && text.ends_with('\n'), "{text}");
    assert_eq!(text.matches(SECTION_MARKER).count(), 1, "{text}");
}

/// A checkout on btrfs that is an ordinary directory:
/// the owner's wording about performance,
/// the migrate commands in
///  order (dry run first),
/// and the plain statement that worktrees still use Git's checkout.
#[test]
fn an_ordinary_directory_on_btrfs_is_told_a_subvolume_can_help_with_performance() {
    let text: String = render_section(&facts(FilesystemKind::Btrfs, false, Some(true)));
    assert_shape(text.as_str());
    assert!(text.contains("ordinary directory, not a subvolume") && text.contains("can help with performance"), "{text}");
    let dry: usize = text.find("git monochromatic-git-policy-cli migrate-to-subvolume --dry-run").expect("the dry run command");
    let real: usize = text.rfind("git monochromatic-git-policy-cli migrate-to-subvolume\n").expect("the migrate command");
    assert!(dry < real, "{text}");
    assert!(text.contains("from a btrfs snapshot of the worktree it is run from") && text.contains("Until then `git worktree add` clones the tracked files"), "{text}");
    assert!(text.contains("has `user_subvol_rm_allowed`") && text.contains("`git worktree remove` deletes a clean subvolume worktree in one call"), "{text}");
    assert!(!text.contains("does not create worktrees from snapshots yet"), "the wrapper creates them now: {text}");
    assert!(!text.contains("sudo "), "an active option needs no command: {text}");
}

/// A checkout that is a subvolume gets no migration advice.
#[test]
fn a_subvolume_checkout_is_not_told_to_migrate() {
    let text: String = render_section(&facts(FilesystemKind::Btrfs, true, Some(true)));
    assert_shape(text.as_str());
    assert!(text.contains("is a btrfs subvolume.") && !text.contains("migrate-to-subvolume"), "{text}");
    assert!(text.contains("from a snapshot of the worktree it is run from when that worktree is clean") && text.contains("every new worktree a subvolume"), "{text}");
}

/// Without the option,
/// the runtime command names the mount point,
/// and an ostree host gets delete plus append with the
///  value of the running kernel command line;
/// `--replace` is explained,
/// never printed as a command.
#[test]
fn the_missing_option_comes_with_the_runtime_command_and_the_ostree_delete_and_append() {
    let text: String = render_section(&facts(FilesystemKind::Btrfs, true, Some(false)));
    assert!(text.contains(format!("sudo mount --options remount,{REMOVAL_OPTION} {MOUNT}").as_str()), "{text}");
    let old: &str = "subvol=root,compress=zstd:1";
    assert!(text.contains(format!("sudo rpm-ostree kargs --delete=rootflags={old} --append=rootflags={old},{REMOVAL_OPTION}").as_str()), "{text}");
    assert!(text.contains("not `--replace`") && !text.contains("sudo rpm-ostree kargs --replace"), "{text}");
    assert!(!text.contains("/etc/fstab"), "an ostree host is not told to edit fstab: {text}");
}

/// A filesystem other than the root one (an extra disk) is not told to change the kernel command line,
/// which does not
///  configure it;
/// it is pointed at its own mount configuration.
#[test]
fn another_filesystem_than_the_root_one_gets_no_kernel_argument() {
    let mut elsewhere: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    elsewhere.root_source = Some(String::from("/dev/mapper/other"));
    let text: String = render_section(&elsewhere);
    assert!(text.contains("is not the root filesystem") && text.contains("/etc/fstab") && !text.contains("rpm-ostree kargs"), "{text}");
    assert!(text.contains(format!("sudo mount --options remount,{REMOVAL_OPTION} {MOUNT}").as_str()), "{text}");
    elsewhere.root_source = None;
    assert!(render_section(&elsewhere).contains("is not the root filesystem"), "an unknown root mount cannot be matched");
}

/// A kernel command line that already lists the option gets no command that would list it twice.
#[test]
fn an_option_already_on_the_kernel_command_line_is_not_appended_again() {
    let mut listed: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    listed.kernel_rootflags = vec![format!("subvol=root,{REMOVAL_OPTION}")];
    let text: String = render_section(&listed);
    assert!(text.contains("already lists") && text.contains("reboot") && !text.contains("sudo rpm-ostree"), "{text}");
}

/// A conventional host is told to edit the filesystem table;
/// it gets no `rpm-ostree` command.
#[test]
fn a_conventional_host_is_told_to_edit_fstab() {
    let mut conventional: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    conventional.ostree_booted = false;
    let text: String = render_section(&conventional);
    assert!(text.contains("/etc/fstab") && !text.contains("rpm-ostree"), "{text}");
    assert!(text.contains(format!("sudo mount --options remount,{REMOVAL_OPTION} {MOUNT}").as_str()), "{text}");
}

/// An ostree host with no or several `rootflags=` words gets no single command to run.
#[test]
fn an_ostree_host_without_exactly_one_rootflags_word_gets_no_kargs_command() {
    let mut none: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    none.kernel_rootflags.clear();
    let text: String = render_section(&none);
    assert!(text.contains("has no `rootflags=` word") && !text.contains("sudo rpm-ostree"), "{text}");
    let mut several: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    several.kernel_rootflags.push(String::from("subvol=other"));
    let crowded: String = render_section(&several);
    assert!(crowded.contains("several `rootflags=` words") && !crowded.contains("sudo rpm-ostree kargs --delete"), "{crowded}");
}

/// An unreadable mount table is said,
/// not guessed.
#[test]
fn an_unreadable_mount_table_is_stated() {
    let text: String = render_section(&facts(FilesystemKind::Btrfs, true, None));
    assert!(text.contains("mount table could not be read") && !text.contains("sudo "), "{text}");
}

/// Overlayfs:
/// the detection,
/// what works inside it and what does not apply,
/// and no migration or mount advice.
#[test]
fn overlayfs_is_described_with_what_works_and_what_does_not() {
    let text: String = render_section(&facts(FilesystemKind::Overlayfs, false, Some(false)));
    assert_shape(text.as_str());
    assert!(text.contains("is on overlayfs") && text.contains("clones there") && text.contains("a snapshot does not apply"), "{text}");
    assert!(text.contains("clones the tracked files of a clean worktree"), "{text}");
    assert!(!text.contains("migrate-to-subvolume") && !text.contains("sudo "), "{text}");
}

/// Any other filesystem:
/// one line that Git's checkout is used and nothing faster applies.
#[test]
fn another_filesystem_gets_one_line() {
    let text: String = render_section(&facts(FilesystemKind::Other, false, Some(false)));
    assert_shape(text.as_str());
    assert_eq!(text.lines().count(), 1, "{text}");
    assert!(text.contains("no faster method applies") && text.contains("XFS made with reflink"), "{text}");
}

/// The value of a `rootflags=` word is everything after the first `=`,
/// for every word,
/// and other words are ignored.
#[test]
fn rootflags_values_keep_their_equal_signs() {
    let line: &str = "BOOT_IMAGE=(hd0,gpt2)/vmlinuz root=UUID=x rootflags=subvol=root,compress=zstd:1 rw norootflags=1 rootflags=a";
    assert_eq!(parse_rootflags(line), vec![String::from("subvol=root,compress=zstd:1"), String::from("a")]);
    assert_eq!(parse_rootflags("quiet splash"), Vec::<String>::new());
    assert_eq!(parse_rootflags(""), Vec::<String>::new());
}

/// The facts of a fixture directory agree with what the kernel reports for it,
/// whatever the host's filesystem is;
/// a
///  directory that is not there has no facts.
#[test]
fn the_facts_of_a_real_directory_agree_with_the_kernel() {
    let parent: PathBuf = fixture("doctor-filesystem-facts");
    let found: FilesystemFacts = gather_facts(parent.as_path(), CheckoutLayout::Main).expect("facts");
    assert_eq!(found.root, parent);
    assert_eq!(found.kind, filesystem_kind(parent.as_path()).expect("kind"));
    let inode_256: bool = std::fs::metadata(parent.as_path()).expect("metadata").ino() == 256;
    assert_eq!(found.root_is_subvolume, found.kind == FilesystemKind::Btrfs && inode_256);
    assert!(found.mount.is_some(), "the fixture is under some mount");
    assert!(found.root_source.is_some(), "the mount table lists the filesystem the kernel command line configures");
    assert_eq!(gather_facts(parent.join("absent").as_path(), CheckoutLayout::Main), None);
    remove(parent.as_path());
}

/// What:
///  The section names the checkout Git reported and starts with the marker,
/// and a directory whose file system cannot be identified gives nothing.
/// Why:
///  The section's subject is Git's answer and no longer a directory name,
/// so a relative name and a directory that is
///  not there are simply facts that cannot be read.
#[test]
fn the_section_names_the_checkout_git_reported() {
    let parent: PathBuf = fixture("doctor-filesystem-host");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let section: String = checkout_section(repo.as_path(), CheckoutLayout::Main);
    assert_shape(section.as_str());
    assert!(section.contains(repo.display().to_string().as_str()), "{section}");
    assert_eq!(checkout_section(parent.join("absent").as_path(), CheckoutLayout::Main), "");
    remove(parent.as_path());
}

/// The section is the last thing on standard error,
/// changes neither the status nor standard output,
/// and a run that
///  could not examine the repository prints none.
#[test]
fn the_section_is_informational_and_last() {
    let parent: PathBuf = fixture("doctor-filesystem-order");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let section_text: &str = "git monochromatic-git-policy-cli doctor: filesystem: informational test text\n";
    let section = |_viewpoint: &crate::doctor::CheckoutViewpoint<'_>| -> String { return section_text.to_string(); };
    let split = |action: Action| -> (i32, String, String) {
        return match action {
            Action::Exit { code, stdout, stderr } => (code, stdout, stderr),
            Action::ExitBytes { code, stdout, stderr } => (
                code,
                String::from_utf8(stdout).expect("a report of text paths"),
                String::from_utf8(stderr).expect("an explanation of text paths"),
            ),
            other => panic!("an exit action, not {other:?}"),
        };
    };
    let clean: (i32, String, String) = split(doctor_action(&git_context(), repo.as_path()));
    let with: (i32, String, String) = with_section(&git_context(), repo.as_path(), None, &section);
    assert_eq!((with.0, with.1.as_str(), with.2.as_str()), (clean.0, clean.1.as_str(), format!("{}{section_text}", clean.2).as_str()));
    executable(repo.join(".git/hooks/pre-commit").as_path(), b"#!/bin/sh\n");
    let found: (i32, String, String) = with_section(&git_context(), repo.as_path(), None, &section);
    assert_eq!(found.0, 1);
    assert!(found.2.ends_with(section_text) && found.2.contains("never run through monochromatic-git-policy-cli"), "{}", found.2);
    let failed: (i32, String, String) = with_section(&git_context(), parent.as_path(), None, &section);
    assert_eq!(failed.0, 2);
    assert!(!failed.2.contains(SECTION_MARKER), "{}", failed.2);
    remove(parent.as_path());
}

/// A mount point with a shell separator,
/// spaces or a substitution is one quoted word in the printed command,
/// so pasting
///  the command cannot run anything else.
#[test]
fn a_mount_point_is_quoted_in_the_printed_command() {
    for mount in ["/mnt/work; sudo reboot", "/mnt/with space", "/mnt/$(id)", "/mnt/it's"] {
        let mut hostile: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
        hostile.mount = Some(MountSummary { mount_point: PathBuf::from(mount), source: String::from(SOURCE), has_removal_option: false });
        let text: String = render_section(&hostile);
        let line: &str = text.lines().find(|candidate: &&str| { return candidate.contains("sudo mount --options"); }).expect("the mount command");
        let word: &str = line.split_once(format!("remount,{REMOVAL_OPTION} ").as_str()).expect("the argument").1;
        assert_eq!(word, crate::shell_quote::shell_quote(mount.as_bytes()), "{mount}: {line}");
        assert!(word.starts_with('\''), "{mount}: the word is single-quoted: {word}");
    }
}

/// The delete and append arguments of the kernel argument command are single words even when the value has spaces,
///  quotes,
/// `$` or backticks.
#[test]
fn the_kernel_argument_command_quotes_its_values() {
    let mut hostile: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    hostile.kernel_rootflags = vec![String::from("subvol=$(touch /tmp/x),compress=zstd")];
    let text: String = render_section(&hostile);
    assert!(
        text.contains("sudo rpm-ostree kargs '--delete=rootflags=subvol=$(touch /tmp/x),compress=zstd' '--append=rootflags=subvol=$(touch /tmp/x),compress=zstd,user_subvol_rm_allowed'"),
        "{text}"
    );
}

/// A quoted value of the kernel command line keeps its quotes in the delete argument (it must equal the stored one) and
///  gets the option inside them in the append argument;
/// a quoted value that lists the option already gets no command.
#[test]
fn a_quoted_kernel_value_is_deleted_as_it_is_and_appended_inside_its_quotes() {
    let mut quoted: FilesystemFacts = facts(FilesystemKind::Btrfs, true, Some(false));
    quoted.kernel_rootflags = vec![String::from("\"subvol=@my root,compress=zstd\"")];
    let text: String = render_section(&quoted);
    assert!(
        text.contains("sudo rpm-ostree kargs '--delete=rootflags=\"subvol=@my root,compress=zstd\"' '--append=rootflags=\"subvol=@my root,compress=zstd,user_subvol_rm_allowed\"'"),
        "{text}"
    );
    quoted.kernel_rootflags = vec![String::from("\"subvol=root,user_subvol_rm_allowed\"")];
    let listed: String = render_section(&quoted);
    assert!(listed.contains("already lists") && !listed.contains("sudo rpm-ostree"), "{listed}");
}

/// The command line is split the way the kernel splits it:
/// a double quote keeps spaces in a word,
/// a quoted value is read
///  without its quotes for the option check but kept whole for the delete argument,
/// and a word quoted as a whole is
///  unwrapped.
#[test]
fn the_kernel_command_line_is_split_at_unquoted_spaces() {
    let line: &str = "quiet rootflags=\"subvol=@my root,compress=zstd\" ro \"rootflags=whole word\" rootflags=a\"b c\"d";
    assert_eq!(
        parse_rootflags(line),
        vec![String::from("\"subvol=@my root,compress=zstd\""), String::from("whole word"), String::from("a\"b c\"d")]
    );
    assert_eq!(kernel_value("\"subvol=@my root\""), "subvol=@my root");
    assert_eq!(kernel_value("a\"b c\"d"), "a\"b c\"d", "quotes inside a value stay");
    assert_eq!(kernel_value("\"unterminated"), "unterminated");
    assert_eq!(parse_rootflags("rootflags=\"never closed and more words"), vec![String::from("\"never closed and more words")]);
}

/// A read of the host's facts that never answers,
/// as one on a mount whose server stopped answering does not.
///
/// The thread parks,
/// so it spends no processor time while it waits:
/// this is a stall,
/// not a load.
fn stalled_host_facts(_checkout: &Path, _layout: CheckoutLayout) -> String {
    loop {
        std::thread::park();
    }
}

/// What:
///  A read of the host facts that never answers ends at its ceiling with one line that says so.
/// Why:
///  `statfs` and the mount table wait in the kernel on a mount whose server stopped answering,
/// and the owner's rule is
///  "nothing should ever wait forever or indefinitely".
/// The line matters as much as the bound:
/// a section silently
///  left out would read as "nothing to say about this file system".
#[test]
fn a_stalled_host_facts_read_gives_up_at_its_ceiling() {
    let parent: PathBuf = fixture("doctor-filesystem-stalled");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    let named: PathBuf = repo.join(".git");
    let (done, finished) = std::sync::mpsc::channel::<String>();
    let worker = std::thread::spawn(move || {
        let section: String = checkout_section_with(stalled_host_facts, Duration::from_millis(50), named.as_path(), CheckoutLayout::Main);
        let _sent = done.send(section);
    });
    // The worker is left behind on purpose: a read that never answers has no end, and the test process ends it.
    let answered: Result<String, _> = finished.recv_timeout(Duration::from_secs(10));
    let _abandoned = worker;
    let section: String = answered.unwrap_or_else(|_| { panic!("the host facts never returned, so a mount that stopped answering holds the report forever") });
    assert!(section.starts_with(SECTION_MARKER), "{section}");
    assert!(section.contains("could not be read within"), "{section}");
    assert!(section.ends_with('\n'), "{section}");
    remove(parent.as_path());
}

/// What:
///  A repository whose Git directory is not beside its worktree is reported on its worktree,
/// and never on the directory that holds the metadata.
/// Why:
///  The section's subject was taken from the common Git directory's name,
/// so a separate Git directory's parent,
///  a directory of metadata and not a checkout,
/// was measured and named as the place to migrate.
#[test]
fn a_separate_git_directory_is_reported_on_its_worktree() {
    let parent: PathBuf = fixture("doctor-filesystem-separate");
    let worktree: PathBuf = parent.join("real-worktree");
    let metadata: PathBuf = parent.join("metadata");
    std::fs::create_dir(worktree.as_path()).expect("worktree directory");
    std::fs::create_dir(metadata.as_path()).expect("metadata directory");
    git(
        worktree.as_path(),
        &[OsStr::new("init"), OsStr::new("--quiet"), OsStr::new(format!("--separate-git-dir={}", metadata.join(".git").display()).as_str())],
    );
    let section: String = with_section(&git_context(), worktree.as_path(), None, &section_of_the_named_checkout).2;
    assert!(!section.contains(metadata.display().to_string().as_str()), "the metadata directory is not a checkout: {section}");
    assert!(section.is_empty() || section.contains(worktree.display().to_string().as_str()), "the worktree the command runs in is the subject: {section}");
    remove(parent.as_path());
}

/// What:
///  A bare repository has no checkout,
/// so the section says nothing about a file system.
/// Why:
///  Its common Git directory may be named `\u200b.git`,
/// and the directory above it is an ordinary parent directory,
/// not a worktree;
/// the
///  section measured that parent and printed the migration command for it.
#[test]
fn a_bare_repository_gets_no_filesystem_section() {
    let parent: PathBuf = fixture("doctor-filesystem-bare");
    let bare: PathBuf = parent.join("bare-parent");
    std::fs::create_dir_all(bare.join(".git")).expect("bare Git directory");
    git(bare.join(".git").as_path(), &["init", "--quiet", "--bare", "--initial-branch=main"]);
    let section: String = with_section(&git_context(), bare.join(".git").as_path(), None, &section_of_the_named_checkout).2;
    assert!(!section.contains(SECTION_MARKER), "a bare repository has no checkout to report on: {section}");
    remove(parent.as_path());
}

/// What:
///  A linked worktree is advised the command that converts it,
/// and never the one that refuses it.
/// Why:
///  `migrate-to-subvolume` converts a repository's main checkout;
/// a linked worktree is converted with
///  `convert-worktree-to-subvolume`,
/// so printing the first would name a command that refuses the directory it just described.
#[test]
fn a_linked_checkout_is_advised_its_own_conversion_command() {
    let mut linked: FilesystemFacts = facts(FilesystemKind::Btrfs, false, Some(false));
    linked.layout = CheckoutLayout::Linked;
    let text: String = render_section(&linked);
    assert!(text.contains("convert-worktree-to-subvolume"), "{text}");
    assert!(!text.contains("migrate-to-subvolume"), "{text}");
    let main: String = render_section(&facts(FilesystemKind::Btrfs, false, Some(false)));
    assert!(main.contains("migrate-to-subvolume"), "{main}");
    assert!(!main.contains("convert-worktree-to-subvolume"), "{main}");
}
