//! What:
//!  The filesystem section of `git monochromatic-git-policy-cli doctor`:
//! what the checkout Git named for the command sits on (btrfs subvolume,
//!  btrfs directory,
//! overlayfs,
//! other),
//! whether the mount allows a regular user to delete a subvolume,
//! and the exact
//!  commands that change what can be changed.
//! Why:
//!  Cheap worktrees need a btrfs subvolume,
//! and deleting a worktree that is a subvolume in one step needs the mount
//!  option `user_subvol_rm_allowed`.
//! Neither is visible to the person until something is slow,
//! so the doctor says what
//!  holds and prints the command,
//! never runs it.
//! Everything is informational:
//! the section goes to standard error after
//!  the hook report and never changes the exit status.
//! It reads `statfs`,
//! one inode number,
//! `/proc/self/mountinfo`,
//!  `/run/ostree-booted` and `/proc/cmdline`,
//! and starts nothing.
//! The facts are gathered apart from the wording
//!  (`FilesystemFacts` in,
//! text out),
//! so the wording is tested on built facts and the exact-output controls of the
//!  doctor and the management command stay independent of the host they run on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const section = renderSection(await gatherFacts(mainCheckoutRoot)); // '' when no checkout was found
//! ```

use crate::checkout_layout::{CheckoutLayout, conversion_command};
use crate::fs_kind::{FilesystemKind, filesystem_kind, is_subvolume_root};
use crate::fs_mount_options::read_mount_facts;
use crate::shell_quote::shell_quote;
/// The ceiling of a read of the host's facts.
use crate::wait_ceilings::REPORT_SCAN_CEILING;
/// The job of such a read,
/// run on a thread of its own.
use crate::work_awaited_within_a_ceiling::work_within;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The first words of the first line of the section,
/// which a caller can cut the section off at.
pub const SECTION_MARKER: &str = "git monochromatic-git-policy-cli doctor: filesystem:";

/// The mount option that lets a regular user delete a subvolume in one step.
pub const REMOVAL_OPTION: &str = "user_subvol_rm_allowed";

/// The marker file that systemd creates on an ostree or bootc host.
const OSTREE_BOOTED_PATH: &str = "/run/ostree-booted";

/// Where an ostree or bootc host mounts the filesystem that the kernel command line's `root=` and `rootflags=` name;
/// `/`
///  there is a composefs overlay.
const SYSROOT_PATH: &str = "/sysroot";

/// The kernel command line of the running system.
const CMDLINE_PATH: &str = "/proc/cmdline";

/// What:
///  The mount that holds the checkout,
/// reduced to what the section needs.
/// Why:
///   The remount command names the mount point,
/// and the option check reads both option lists of the mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountSummary = { mountPoint: string; source: string; hasRemovalOption: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountSummary {
    /// Where the filesystem is mounted.
    pub mount_point: PathBuf,
    /// The device or source the kernel lists for it.
    pub source: String,
    /// Whether `user_subvol_rm_allowed` is among the mount options or the superblock options.
    pub has_removal_option: bool,
}

/// What:
///  Everything the section is written from.
///  `Vec<String>` is an owned list of owned text.
/// Why:
///   One value holds the host facts,
/// so the wording is a pure function of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FilesystemFacts = { root: string; kind: FilesystemKind; rootIsSubvolume: boolean; mount?: MountSummary; rootSource?: string; ostreeBooted: boolean; kernelRootflags: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesystemFacts {
    /// The checkout the section reports on.
    pub root: PathBuf,
    /// How that checkout is laid out,
    /// which decides the command the advice names.
    pub layout: CheckoutLayout,
    /// The filesystem kind holding it.
    pub kind: FilesystemKind,
    /// Whether the checkout is a subvolume root (inode 256 on btrfs).
    pub root_is_subvolume: bool,
    /// The mount holding it,
    /// when the mount table could be read.
    pub mount: Option<MountSummary>,
    /// The source of the filesystem the kernel command line's `rootflags=` configures:
    /// the mount at `/sysroot` on an
    /// ostree host,
    /// else the mount at `/`.
    pub root_source: Option<String>,
    /// Whether the host is an ostree or bootc deployment (`/run/ostree-booted` exists).
    pub ostree_booted: bool,
    /// The values of every `rootflags=` word on the running kernel command line,
    /// as written there (quotes included).
    pub kernel_rootflags: Vec<String>,
}

/// What:
///  The words of a kernel command line,
/// split as the kernel splits it:
/// at whitespace outside double quotes.
/// Why:
///   A double quote toggles quoting wherever it stands in a word,
/// so `rootflags="subvol=@my root"` is one word;
/// the
///  kernel's `next_arg` (`lib/cmdline.c` and `kernel/params.c`) counts quotes the same way and strips them afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = (cmdline: string) => splitOutsideQuotes(cmdline, /\s/);
/// ```
fn command_line_words(command_line: &str) -> Vec<&str> {
    let mut words: Vec<&str> = Vec::new();
    let mut start: Option<usize> = None;
    let mut in_quote: bool = false;
    for (index, character) in command_line.char_indices() {
        if character.is_whitespace() && !in_quote {
            if let Some(began) = start.take() {
                words.push(&command_line[began..index]);
            }
            continue;
        }
        if start.is_none() {
            start = Some(index);
        }
        if character == '"' {
            in_quote = !in_quote;
        }
    }
    if let Some(began) = start {
        words.push(&command_line[began..]);
    }
    return words;
}

/// What:
///  A kernel argument value without the quote that opens it and the quote that closes it.
/// Why:
///   The kernel strips a leading quote from a value and a trailing one when the value had a leading one;
/// quotes inside a
///  value stay (they only toggled the splitting).
/// The option check reads this form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kernelValue = (raw: string) => raw.startsWith('"') ? raw.slice(1).replace(/"$/, '') : raw;
/// ```
pub fn kernel_value(raw: &str) -> String {
    let Some(rest) = raw.strip_prefix('"') else { return raw.to_string() };
    return rest.strip_suffix('"').unwrap_or(rest).to_string();
}

/// What:
///  The values of the `rootflags=` words of a kernel command line,
/// as written there (a quoted value keeps its quotes).
/// Why:
///   The persistent command on an ostree host replaces that exact value,
/// so it is read from the running system and
///  never guessed.
/// The value is everything after the first `=`,
/// because the value itself contains `=`.
/// A word quoted as
///  a whole (`"rootflags=a b"`) is unwrapped first,
/// as the kernel does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rootflags = (cmdline: string) => words(cmdline).map(unwrap).filter((w) => w.startsWith('rootflags=')).map((w) => w.slice(10));
/// ```
pub fn parse_rootflags(command_line: &str) -> Vec<String> {
    let mut flags: Vec<String> = Vec::new();
    for word in command_line_words(command_line) {
        // A quoted word is read without its quotes, unless nothing would be left of it.
        let unwrapped: &str = match word.strip_prefix('"') {
            Some(inner) => match inner.strip_suffix('"') {
                Some(unquoted) if !unquoted.is_empty() => unquoted,
                _ => word,
            },
            None => word,
        };
        if let Some(flag) = unwrapped.strip_prefix("rootflags=") {
            flags.push(flag.to_string());
        }
    }
    return flags;
}

/// What:
///  Read the facts for a checkout from the running system.
/// Why:
///   A fact that cannot be read becomes the plain answer (`None`,
/// empty list,
/// `false`),
/// and the wording states less.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = await gatherFacts(root);
/// ```
pub fn gather_facts(root: &Path, layout: CheckoutLayout) -> Option<FilesystemFacts> {
    let kind: FilesystemKind = filesystem_kind(root).ok()?;
    let root_is_subvolume: bool = kind == FilesystemKind::Btrfs && is_subvolume_root(root).unwrap_or(false);
    let mount: Option<MountSummary> = match read_mount_facts(root).ok().flatten() {
        Some(facts) => Some(MountSummary { has_removal_option: facts.has_option(REMOVAL_OPTION), source: facts.source.clone(), mount_point: facts.mount_point }),
        None => None,
    };
    let ostree_booted: bool = Path::new(OSTREE_BOOTED_PATH).exists();
    let root_source: Option<String> = match read_mount_facts(Path::new(if ostree_booted { SYSROOT_PATH } else { "/" })).ok().flatten() {
        Some(facts) => Some(facts.source),
        None => None,
    };
    let command_line: String = std::fs::read_to_string(CMDLINE_PATH).unwrap_or_default();
    return Some(FilesystemFacts {
        root: root.to_path_buf(),
        layout,
        kind,
        root_is_subvolume,
        mount,
        root_source,
        ostree_booted,
        kernel_rootflags: parse_rootflags(command_line.as_str()),
    });
}

/// The wording for a filesystem that is neither btrfs nor overlayfs.
fn other_lines(facts: &FilesystemFacts) -> Vec<String> {
    return vec![format!(
        "{} {} is on a filesystem other than btrfs or overlayfs. `git worktree add` uses Git's own checkout there, and no faster method applies (XFS made with reflink support clones the tracked files of a clean worktree instead).",
        SECTION_MARKER,
        facts.root.display()
    )];
}

/// The wording for overlayfs.
fn overlay_lines(facts: &FilesystemFacts) -> Vec<String> {
    return vec![
        format!("{} {} is on overlayfs.", SECTION_MARKER, facts.root.display()),
        String::from("  Cloning a file's data into another file inside the same overlay works, so the wrapper's copy of ignored files uses clones there, and `git worktree add` clones the tracked files of a clean worktree instead of letting Git check them out; a snapshot does not apply, because snapshots need a btrfs subvolume and the btrfs under an overlay cannot be addressed from inside it."),
        String::from("  Git's own checkout, which the wrapper falls back to when the worktree it is run from is not clean, was several times slower on an overlay than on btrfs in one measurement (doc/planning/cli-git-btrfs-worktree.md, section Overlayfs)."),
    ];
}

/// The value with the option added:
/// inside the closing quote of a quoted value,
/// at the end of any other.
fn with_removal_option(raw: &str) -> String {
    if raw.starts_with('"') && raw.len() > 1 && let Some(inner) = raw.strip_suffix('"') {
        return format!("{inner},{REMOVAL_OPTION}\"");
    }
    return format!("{raw},{REMOVAL_OPTION}");
}

/// Whether a comma-separated mount option list names the removal option as one of its words.
fn lists_removal_option(options: &str) -> bool {
    for option in options.split(',') {
        if option == REMOVAL_OPTION {
            return true;
        }
    }
    return false;
}

/// The persistent way to set the option on an ostree host,
/// for the filesystem the kernel command line configures.
fn kernel_argument_lines(facts: &FilesystemFacts) -> Vec<String> {
    return match facts.kernel_rootflags.as_slice() {
        [old] if lists_removal_option(&kernel_value(old)) => vec![format!(
            "  Kept across boots: the running kernel command line already lists `{REMOVAL_OPTION}` in `rootflags=`, so a mount that lacks it was made before that; reboot, and compare with the output of `rpm-ostree kargs`."
        )],
        [old] => vec![
            String::from("  Kept across boots on this ostree host, then reboot (the value is the `rootflags=` of the running kernel command line; compare it with the output of `rpm-ostree kargs` before running):"),
            format!(
                "    sudo rpm-ostree kargs {} {}",
                shell_quote(format!("--delete=rootflags={old}").as_bytes()),
                shell_quote(format!("--append=rootflags={}", with_removal_option(old)).as_bytes())
            ),
            String::from("  Use delete and append, not `--replace`: the value contains `=`, and libostree splits each argument at the first `=`, so `--replace=rootflags=...` reads the old value as only the part before the next `=`."),
        ],
        [] => vec![String::from("  Kept across boots: this ostree host's kernel command line has no `rootflags=` word, so the mount options come from somewhere else (for example `/etc/fstab`); add the option there.")],
        _ => vec![String::from("  Kept across boots: this host's kernel command line has several `rootflags=` words, so no single command is printed; edit them with `rpm-ostree kargs --editor`.")],
    };
}

/// The commands that make the removal option active,
/// for the host the facts describe.
fn removal_option_lines(facts: &FilesystemFacts, mount: &MountSummary) -> Vec<String> {
    let mount_point: &Path = mount.mount_point.as_path();
    let mut lines: Vec<String> = vec![
        format!("  The mount {} lacks `{REMOVAL_OPTION}`. Without it, `git worktree remove` of a worktree that is a subvolume works but unlinks its files one by one; with it, the wrapper deletes a clean worktree's subvolume with one call.", mount_point.display()),
        format!("  Active until the next boot or unmount: sudo mount --options remount,{REMOVAL_OPTION} {}", shell_quote(mount_point.as_os_str().as_bytes())),
    ];
    let on_root_filesystem: bool = facts.root_source.as_deref() == Some(mount.source.as_str());
    if facts.ostree_booted && on_root_filesystem {
        lines.extend(kernel_argument_lines(facts));
    } else if facts.ostree_booted {
        lines.push(format!("  Kept across boots: {} is not the root filesystem, so the kernel command line's `rootflags=` does not apply to it; add the option to its `/etc/fstab` entry or its systemd mount unit, then reboot.", mount_point.display()));
    } else {
        lines.push(format!("  Kept across boots on a conventional host: add `{REMOVAL_OPTION}` to the options of this filesystem's entry in /etc/fstab (every btrfs entry that shares its filesystem), then reboot."));
    }
    return lines;
}

/// The wording for btrfs.
fn btrfs_lines(facts: &FilesystemFacts) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if facts.root_is_subvolume {
        lines.push(format!("{} {} is a btrfs subvolume.", SECTION_MARKER, facts.root.display()));
        lines.push(String::from("  `git worktree add` makes the new worktree from a snapshot of the worktree it is run from when that worktree is clean, and makes every new worktree a subvolume, so the next one can be a snapshot too. A worktree that is not clean, has a sparse checkout or worktree-specific configuration, or has a nested worktree that is an ordinary directory is cloned file by file instead, or checked out by Git."));
    } else {
        lines.push(format!("{} {} is on btrfs but is an ordinary directory, not a subvolume.", SECTION_MARKER, facts.root.display()));
        lines.push(String::from("  Making it a subvolume can help with performance: `git worktree add` then makes the new worktree from a btrfs snapshot of the worktree it is run from, which took a median of 0.35 s to 1.09 s in the design measurements, against 2.1 s to 2.7 s for Git's checkout and 28.35 s with an ignored tree of 72 thousand entries, and shares every data block with its source (doc/planning/cli-git-btrfs-worktree.md, section Measurements)."));
        lines.push(String::from("  Until then `git worktree add` clones the tracked files of a clean worktree, which shares disk but takes about as long as Git's checkout, and makes every new worktree a subvolume that later ones can be snapshotted from; worktrees that exist already as ordinary directories stay ordinary. The conversion keeps the original tree until you delete it."));
        lines.push(String::from("  Stop agent sessions and editors that have their working directory inside the checkout first, then:"));
        lines.push(format!("    git monochromatic-git-policy-cli {} --dry-run", conversion_command(facts.layout)));
        lines.push(format!("    git monochromatic-git-policy-cli {}", conversion_command(facts.layout)));
    }
    match &facts.mount {
        Some(mount) if mount.has_removal_option => {
            lines.push(format!("  The mount {} has `{REMOVAL_OPTION}`: `git worktree remove` deletes a clean subvolume worktree in one call.", mount.mount_point.display()));
        }
        Some(mount) => lines.extend(removal_option_lines(facts, mount)),
        None => lines.push(String::from("  The mount table could not be read, so `user_subvol_rm_allowed` was not checked.")),
    }
    return lines;
}

/// What:
///  The section text for the facts:
/// lines ending in a line feed,
/// the first beginning with `SECTION_MARKER`.
/// Why:
///   Pure wording,
/// so every case is tested without the host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = renderSection(facts);
/// ```
pub fn render_section(facts: &FilesystemFacts) -> String {
    let lines: Vec<String> = match facts.kind {
        FilesystemKind::Btrfs => btrfs_lines(facts),
        FilesystemKind::Overlayfs => overlay_lines(facts),
        FilesystemKind::Other => other_lines(facts),
    };
    let mut text: String = lines.join("\n");
    text.push('\n');
    return text;
}

/// What:
///  The section for the checkout Git named for the command,
/// read from the running system;
/// empty when its file system cannot be
///  identified.
/// Why:
///   This is what the command appends to the doctor's standard error.
/// The checkout is the one Git reported for
///  where the command runs,
/// so a worktree,
/// `-C`,
/// `--git-dir` and a repository whose Git directory lives elsewhere all lead to a
///  directory that is a checkout,
/// and a repository without one (a bare repository) gets no line.
///  A basename is not a layout proof:
/// reading the directory above the common Git directory measured the metadata
///  directory of a separate Git
/// directory and the parent directory of a bare repository,
/// and told the person to migrate
///  those.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const section = checkoutSection(checkoutGitNamed, layout);
/// ```
pub fn checkout_section(checkout: &Path, layout: CheckoutLayout) -> String {
    return checkout_section_with(section_of_the_checkout, REPORT_SCAN_CEILING, checkout, layout);
}

/// What:
///  What one read of the host's facts answers,
/// as a function pointer,
/// so a control can stand a read that never answers where the file system stands.
///  `CheckoutLayout` is how the checkout it is asked about is laid out.
/// Why:
///  `statfs`,
/// the subvolume check and the mount table wait in the kernel on a mount whose server stopped
///  answering,
/// so the read is a value the section is given rather than a call it makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HostFactsRead = (checkout: string, layout: CheckoutLayout) => string;
/// ```
pub type HostFactsRead = fn(&Path, CheckoutLayout) -> String;

/// What:
///  The section's own facts read:
/// the checkout's file system kind,
/// its subvolume state,
/// its mount and the kernel command
///  line,
/// or nothing when its file system cannot be identified.
/// Why:
///   This is the read the ceiling is placed on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sectionOfTheCheckout = (checkout, layout) => { ... };
/// ```
fn section_of_the_checkout(checkout: &Path, layout: CheckoutLayout) -> String {
    return match gather_facts(checkout, layout) {
        Some(facts) => render_section(&facts),
        None => String::new(),
    };
}

/// What:
///  `checkout_section` with the facts read and the ceiling given.
/// Why:
///   A read that gives no answer within the ceiling becomes one line that says so,
/// because a report that
///  silently left the section out would read as "nothing to say about this file system".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkoutSectionWith(sectionOfTheCheckout, REPORT_SCAN_CEILING, checkout, layout);
/// ```
pub fn checkout_section_with(reader: HostFactsRead, ceiling: std::time::Duration, checkout: &Path, layout: CheckoutLayout) -> String {
    let named: PathBuf = checkout.to_path_buf();
    let read: Option<String> = work_within(ceiling, move || { return reader(named.as_path(), layout); });
    return match read {
        Some(text) => text,
        None => format!(
            "{SECTION_MARKER} the host facts of {} could not be read within {ceiling:?}, so nothing is said about its file system\n",
            checkout.display()
        ),
    };
}

/// Filesystem section controls stay out of the release executable.
#[cfg(test)]
#[path = "doctor_filesystem_tests.rs"]
mod tests;
