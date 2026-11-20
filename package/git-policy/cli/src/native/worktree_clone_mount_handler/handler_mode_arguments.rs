//! What:
//!  The words one invocation of the handler is read as:
//!  no argument for the clone-mount sweep the installed ZED event starts,
//!  `boot-reconcile`,
//!  `shutdown-unmount`,
//!  `adopt-worktree-parent` with the replacement's device and inode,
//!  and the two words that print what this build is.
//! Why:
//!  The design names the binary and its modes and no argument
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit",
//!  "The administrator's adoption"),
//!  and the plan's suggestion is that the default invocation,
//!  which is how the installed ZED event calls it,
//!  runs the sweep
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Where the design is silent:
//! this plan's suggestions").
//!  Nothing here reads a path,
//!  a dataset name or a mount flag out of the request beyond the administrator's own
//!  identification of the replacement:
//!  a mode is a word,
//!  never a command line the caller composes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mode = parse(process.argv.slice(2)); // { kind: 'attaching-sweep' } | { kind: 'boot-reconcile' } | ...
//! ```

use crate::worktree_clone_mount_handler::handler_install_paths::{SERVICE_INSTALL_PATH, ZEDLET_INSTALL_PATH};
use monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH;
use std::ffi::OsString;
use std::path::PathBuf;

/// The mode words this build accepts,
/// in the order the help lists them.
pub const MODE_WORDS: [&str; 3] = ["boot-reconcile", "shutdown-unmount", "adopt-worktree-parent"];

/// The word that prints the help.
pub const HELP_WORD: &str = "--help";

/// The word that prints this build's version.
pub const VERSION_WORD: &str = "--version";

/// The option that names the checkout an adoption walks from.
pub const CHECKOUT_OPTION: &str = "--checkout";

/// The option that carries the replacement's device number.
pub const DEVICE_OPTION: &str = "--device";

/// The option that carries the replacement's inode number.
pub const INODE_OPTION: &str = "--inode";

/// What:
///  The administrator's identification of the replacement folder an adoption judges.
/// Why:
///   The design requires the device and inode to be identified independently,
///  from the restored checkout or the controlled recreation,
///  and the mode compares the held folder against them before anything is recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Adoption = { checkout: '/home/user/Monochromatic', device: 66306, inode: 1234567 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptRequest {
    /// The checkout whose `.claude/worktrees` folder is walked.
    pub checkout: PathBuf,
    /// The device the administrator read for the intended replacement.
    pub device: u64,
    /// The inode the administrator read for the intended replacement.
    pub inode: u64,
}

/// What:
///  What one invocation asks this build to do.
/// Why:
///   A mode is chosen by a leading word and nothing else,
///  and no argument names a folder to mount or a dataset to change:
///  the sweep reads the durable intent,
///  and an adoption takes only the replacement's own identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mode = { kind: 'sweep' } | { kind: 'boot-reconcile' } | { kind: 'shutdown-unmount' } | { kind: 'adopt'; request: Adoption } | { kind: 'help' } | { kind: 'version' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandlerMode {
    /// No argument:
    /// the clone-mount sweep the installed ZED event starts.
    AttachingSweep,
    /// `boot-reconcile`:
    /// the boot unit's reconciliation.
    BootReconcile,
    /// `shutdown-unmount`:
    /// the shutdown unit's verified unmount.
    ShutdownUnmount,
    /// `adopt-worktree-parent`:
    /// the administrator's decision to record a replacement parent folder.
    AdoptWorktreeParent(AdoptRequest),
    /// `--help`.
    Help,
    /// `--version`.
    Version,
}

/// What:
///  Why one invocation could not be read as a mode.
/// Why:
///   Each cause names the argument a person typed,
///  so the answer says what to change rather than only that the input was wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ArgumentRefusal = { kind: 'not-text' | 'unknown-mode' | 'missing-value' | ...; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgumentRefusal {
    /// An argument is not UTF-8 text.
    NotText {
        /// Its one-based place among the arguments,
        /// which is where a person sees it.
        place: usize,
    },
    /// The leading word is no mode of this build.
    UnknownMode {
        /// The word as typed.
        word: String,
    },
    /// An option came last and has no value.
    MissingValue {
        /// The option.
        option: &'static str,
    },
    /// An option was given more than once,
    /// so two identifications would compete.
    RepeatedOption {
        /// The option.
        option: &'static str,
    },
    /// The adoption's words lack an option it needs.
    MissingOption {
        /// The option.
        option: &'static str,
    },
    /// An option's value is not the decimal count it must be.
    OptionNotACount {
        /// The option.
        option: &'static str,
        /// The value as typed.
        text: String,
    },
    /// A word this invocation does not take,
    /// after the mode or after an option's value.
    UnexpectedArgument {
        /// The word as typed.
        word: String,
    },
}

impl ArgumentRefusal {
    /// The refusal in words,
    /// naming the argument and every way forward.
    pub fn message(&self) -> String {
        return match self {
            ArgumentRefusal::NotText { place } => format!(
                "argument {place} of mount-git-worktree-clone-datasets is not text, and every argument this program reads is UTF-8 text. Nothing was done. Ways forward: give the mode word and the adoption's numbers as text; a checkout path that is not valid UTF-8 cannot be named here, so name a checkout whose path is text."
            ),
            ArgumentRefusal::UnknownMode { word } => format!(
                "`{word}` is not a mode of mount-git-worktree-clone-datasets. Nothing was done. The modes are {}, and running the program with no argument at all runs the clone-mount sweep the installed ZED event starts. Ways forward: run `mount-git-worktree-clone-datasets --help` for the words this build accepts.",
                MODE_WORDS.join(", ")
            ),
            ArgumentRefusal::MissingValue { option } => format!(
                "`{option}` is the last argument and has no value. Nothing was done. Ways forward: give `{option}` its value, as `{option} <value>`, and run the command again."
            ),
            ArgumentRefusal::RepeatedOption { option } => format!(
                "`{option}` was given more than once, and two values would compete. Nothing was done. Ways forward: give `{option}` once, then run the command again."
            ),
            ArgumentRefusal::MissingOption { option } => format!(
                "adopt-worktree-parent needs `{option}`, and it was not given. Nothing was done. Ways forward: pass `--checkout <path> --device <count> --inode <count>`, all three, and run the command again; --help repeats the form."
            ),
            ArgumentRefusal::OptionNotACount { option, text } => format!(
                "`{option} {text}` is not a decimal count. Nothing was done. Ways forward: give the number `stat --format=%d` (device) or `stat --format=%i` (inode) prints for the intended replacement, as decimal digits without a sign or a prefix."
            ),
            ArgumentRefusal::UnexpectedArgument { word } => format!(
                "`{word}` is an argument this invocation does not take. Nothing was done. Ways forward: run `mount-git-worktree-clone-datasets --help` for the modes and their arguments; a mode takes only the words the help lists beside it."
            ),
        };
    }
}

/// What:
///  The whole usage text,
///  naming the modes,
///  the two installed paths,
///  and the device the modes that make requests will open.
/// Why:
///   A person meeting the program at either installed path reads what it is and what this build refuses;
///  naming the second installed path says the same build answers a ZED event,
///  and naming the device says a change of it is a change of one constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const usage = `usage: ...`; // one line per mode and one per installed path
/// ```
pub fn usage() -> String {
    return [
        String::from("usage: mount-git-worktree-clone-datasets [mode]"),
        String::from("  (no argument)        run the clone-mount sweep the installed ZED event starts"),
        String::from("  boot-reconcile       reconcile clone mounts after boot; the boot unit's ExecStart"),
        String::from("  shutdown-unmount     unmount the verified clone mounts at shutdown; the boot unit's ExecStopPost"),
        String::from("  adopt-worktree-parent --checkout <path> --device <count> --inode <count>"),
        String::from("                       record an administrator-identified replacement .claude/worktrees folder"),
        String::from("  --help               print this text"),
        String::from("  --version            print this build's version"),
        format!("The same build is installed at {SERVICE_INSTALL_PATH} for service calls and at {ZEDLET_INSTALL_PATH} for ZED events."),
        format!("The modes that make requests open {DEVICE_PATH} through this repository's binding, never a CDDL library and never the zfs program."),
    ].join("\n");
}

/// What:
///  The line `--version` prints:
///  the program's own name and the release it was built from.
/// Why:
///   An install that moved the archive asks this one command where the second executable is,
///  and the line names the release so a lockfile's row can be checked against it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const version = `${PROGRAM_NAME} ${packageVersion}`;
/// ```
pub fn version_line() -> String {
    return format!("{} {}", crate::worktree_clone_mount_handler::handler_install_paths::PROGRAM_NAME, env!("CARGO_PKG_VERSION"));
}

/// What:
///  Read the arguments as one mode.
/// Why:
///   The grammar is one leading word and,
///  for an adoption only,
///  three options with one value each;
///  every other word,
///  repetition and unreadable argument is a refusal rather than a guess,
///  because guessing a mode at root would start a sweep nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = parse(argv); // throws ArgumentRefusal
/// ```
///
/// # Errors
/// A word that is no mode,
/// a missing,
/// repeated or non-text option,
/// a value that is no decimal count,
/// or another word where the invocation takes none.
pub fn parse(arguments: &[OsString]) -> Result<HandlerMode, ArgumentRefusal> {
    if arguments.is_empty() {
        return Ok(HandlerMode::AttachingSweep);
    }
    let mut words: Vec<&str> = Vec::with_capacity(arguments.len());
    for (place, argument) in arguments.iter().enumerate() {
        let Some(word) = argument.to_str() else {
            return Err(ArgumentRefusal::NotText { place: place + 1 });
        };
        words.push(word);
    }
    let Some(first) = words.first().copied() else {
        return Ok(HandlerMode::AttachingSweep);
    };
    return match first {
        HELP_WORD => match words.len() {
            1 => Ok(HandlerMode::Help),
            _ => Err(ArgumentRefusal::UnexpectedArgument { word: words[1].to_string() }),
        },
        VERSION_WORD => match words.len() {
            1 => Ok(HandlerMode::Version),
            _ => Err(ArgumentRefusal::UnexpectedArgument { word: words[1].to_string() }),
        },
        "boot-reconcile" => match words.len() {
            1 => Ok(HandlerMode::BootReconcile),
            _ => Err(ArgumentRefusal::UnexpectedArgument { word: words[1].to_string() }),
        },
        "shutdown-unmount" => match words.len() {
            1 => Ok(HandlerMode::ShutdownUnmount),
            _ => Err(ArgumentRefusal::UnexpectedArgument { word: words[1].to_string() }),
        },
        "adopt-worktree-parent" => parse_adoption(&words[1..]).map(HandlerMode::AdoptWorktreeParent),
        other => Err(ArgumentRefusal::UnknownMode { word: other.to_string() }),
    };
}

/// What:
///  Which of an adoption's three options one word is.
/// Why:
///   The grammar reads each option by name and then falls through one exhaustive match,
///  so a new option is a compiler error rather than a word no branch handles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AdoptionOption = 'checkout' | 'device' | 'inode';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdoptionOption {
    /// `--checkout`.
    Checkout,
    /// `--device`.
    Device,
    /// `--inode`.
    Inode,
}

/// What:
///  Read an adoption's options,
///  each given once and in any order.
/// Why:
///   The three values together are the administrator's identification,
///  and a repeated option would leave which one was meant to be read;
///  the checkout is a path and the other two are decimal counts,
///  so a value the kernel would read as another number is refused here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseAdoption(['--checkout', '/repo', '--device', '66306', '--inode', '1234567']);
/// ```
fn parse_adoption(words: &[&str]) -> Result<AdoptRequest, ArgumentRefusal> {
    let mut given_checkout: Option<PathBuf> = None;
    let mut given_device: Option<u64> = None;
    let mut given_inode: Option<u64> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = words[index];
        let (known, name): (AdoptionOption, &'static str) = match option {
            CHECKOUT_OPTION => (AdoptionOption::Checkout, CHECKOUT_OPTION),
            DEVICE_OPTION => (AdoptionOption::Device, DEVICE_OPTION),
            INODE_OPTION => (AdoptionOption::Inode, INODE_OPTION),
            other => return Err(ArgumentRefusal::UnexpectedArgument { word: other.to_string() }),
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue { option: name });
        };
        match known {
            AdoptionOption::Checkout => {
                if given_checkout.is_some() {
                    return Err(ArgumentRefusal::RepeatedOption { option: name });
                }
                given_checkout = Some(PathBuf::from(value));
            }
            AdoptionOption::Device => {
                if given_device.is_some() {
                    return Err(ArgumentRefusal::RepeatedOption { option: name });
                }
                given_device = Some(decimal_count(name, value)?);
            }
            AdoptionOption::Inode => {
                if given_inode.is_some() {
                    return Err(ArgumentRefusal::RepeatedOption { option: name });
                }
                given_inode = Some(decimal_count(name, value)?);
            }
        }
        index += 2;
    }
    let Some(checkout) = given_checkout else {
        return Err(ArgumentRefusal::MissingOption { option: CHECKOUT_OPTION });
    };
    let Some(device) = given_device else {
        return Err(ArgumentRefusal::MissingOption { option: DEVICE_OPTION });
    };
    let Some(inode) = given_inode else {
        return Err(ArgumentRefusal::MissingOption { option: INODE_OPTION });
    };
    return Ok(AdoptRequest { checkout, device, inode });
}

/// What:
///  One option's value as the decimal count it must be,
///  or the refusal naming the option and what was typed.
/// Why:
///   A device or inode is an unsigned number in decimal;
///  a value with a sign,
///  a prefix or a letter is another number to `u64::from_str` or no number at all,
///  so the digits are checked before the parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = decimalCount('--device', '66306'); // throws on '0x10' or '-1'
/// ```
fn decimal_count(option: &'static str, text: &str) -> Result<u64, ArgumentRefusal> {
    if text.is_empty() || !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return Err(ArgumentRefusal::OptionNotACount { option, text: text.to_string() });
    }
    let Ok(count) = text.parse::<u64>() else {
        return Err(ArgumentRefusal::OptionNotACount { option, text: text.to_string() });
    };
    return Ok(count);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_mode_arguments_tests.rs"]
mod tests;
