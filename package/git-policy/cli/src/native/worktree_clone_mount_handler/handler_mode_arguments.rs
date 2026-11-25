//! What:
//!  The words one invocation of the root modes is read as:
//!  no argument at all for the clone-mount sweep ZED's own start runs,
//!  `boot-reconcile`,
//!  `start-probe`,
//!  `shutdown-unmount`,
//!  `holder-scan` with the dataset the scan reports on,
//!  `adopt-worktree-parent` with the replacement's device and inode,
//!  `record-checkout` with the checkout it records,
//!  `mount-path-descendant` with the nested dataset it mounts,
//!  and the two words that print what this build is.
//! Why:
//!  The design names the modes and no argument
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit",
//!  "The administrator's adoption"),
//!  and the plan's suggestion is that the default invocation,
//!  which is how ZED's own start calls it,
//!  runs the sweep
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Where the design is silent:
//! this plan's suggestions").
//!  This grammar is reached through the wrapper's own executable now, so every
//!  mode is named after the words `monochromatic-git-policy-cli zfs-root`
//!  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
//!  choice 2),
//!  and every refusal opens with those words:
//!  the family name is printed nowhere.
//!  A mode is a word,
//!  never a command line the caller composes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mode = parse(words); // { kind: 'attaching-sweep' } | { kind: 'boot-reconcile' } | ...
//! ```

use crate::start_probe::probe_source_name::check_dataset_name;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_mount_handler::handler_install_paths::{ROOT_MODE_INSTALL_PATH, ZEDLET_INSTALL_PATH, root_modes_name};
use crate::worktree_clone_mount_handler::root_mode_names::{ROOT_GROUP_WORD, ZEDLET_START_NAME};
use monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH;
use std::ffi::OsString;
use std::path::PathBuf;

/// The mode words this build accepts,
/// in the order the help lists them.
pub const MODE_WORDS: [&str; 7] = ["boot-reconcile", "start-probe", "shutdown-unmount", "holder-scan", "adopt-worktree-parent", "record-checkout", "mount-path-descendant"];

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

/// The option that names the dataset a holder scan reads.
pub const DATASET_OPTION: &str = "--dataset";

/// The option that carries the nonce one holder scan answers under.
pub const NONCE_OPTION: &str = "--nonce";

/// The option that replaces a recorded checkout's GUID with the one the kernel
/// shows now.
pub const REPLACE_OPTION: &str = "--replace";

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
///  The dataset one `holder-scan` invocation reads,
///  and the nonce it answers under when the caller names one.
/// Why:
///   The scan is about one dataset's mounts and nothing else:
///  it names every namespace that holds it and sets nothing aside,
///  so no path is read and none is guessed.
///  The nonce is the caller's when it names one,
///  because the wrapper asks for a fresh scan before each destroy and matches
///  the answer by the nonce it chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderScan = { dataset: 'zpcachyos/…/clones/one', nonce: '…' | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderScanInvocation {
    /// The dataset whose holders the scan reports.
    pub dataset: String,
    /// The nonce the answer is bound to,
    /// or nothing when the scan makes its own.
    pub nonce: Option<RequestNonce>,
}

/// What:
///  The checkout one `record-checkout` invocation records,
///  and the GUID it admits when a legitimate restore changed the record's.
/// Why:
///   The checkout is the folder whose dataset the route serves,
///  and the replacement is the kernel's own GUID and no other value,
///  so a command cannot admit a dataset without naming the number it read
///  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
///  choice 9).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordCheckout = { checkout: '/home/user/Monochromatic', replace: '9876543210987654321' | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordCheckoutRequest {
    /// The checkout whose route facts are recorded.
    pub checkout: PathBuf,
    /// The kernel's GUID when this run replaces a record a restore moved.
    pub replace: Option<String>,
}

/// What:
///  The nested dataset one `mount-path-descendant` invocation mounts.
/// Why:
///   The dataset names the work and the record names the checkout it is placed
///  under,
///  so the mode takes no path: a path a caller gave would be a mountpoint
///  nothing proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountPathDescendant = { dataset: 'zpcachyos/nosnap/vm-builder-output' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountPathDescendantRequest {
    /// The nested dataset's full name.
    pub dataset: String,
}

/// What:
///  What one invocation asks this build to do.
/// Why:
///   A mode is chosen by a leading word and nothing else,
///  and the only arguments a mode takes are the identities its proofs need:
///  the sweep reads the durable intent,
///  an adoption takes the replacement's own identity,
///  `record-checkout` takes the checkout and the GUID a replace admits,
///  and `mount-path-descendant` takes the dataset it places under the recorded
///  checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mode = { kind: 'sweep' } | { kind: 'boot-reconcile' } | { kind: 'shutdown-unmount' } | { kind: 'adopt'; request: Adoption } | { kind: 'help' } | { kind: 'version' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandlerMode {
    /// No argument:
    /// the clone-mount sweep ZED's own start runs.
    AttachingSweep,
    /// `boot-reconcile`:
    /// the boot unit's reconciliation.
    BootReconcile,
    /// `start-probe`:
    /// the root start probe's arms and its record.
    StartProbe,
    /// `shutdown-unmount`:
    /// the shutdown unit's verified unmount.
    ShutdownUnmount,
    /// `holder-scan`:
    /// the all-namespace holder scan of one dataset.
    HolderScan(HolderScanInvocation),
    /// `adopt-worktree-parent`:
    /// the administrator's decision to record a replacement parent folder.
    AdoptWorktreeParent(AdoptRequest),
    /// `record-checkout`:
    /// the record of the checkout's path, dataset, GUID, container and pool.
    RecordCheckout(RecordCheckoutRequest),
    /// `mount-path-descendant`:
    /// one nested dataset mounted at its own mountpoint beneath the recorded
    /// checkout.
    MountPathDescendant(MountPathDescendantRequest),
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
    /// The adoption's words lack an option it needs,
    /// or a holder scan's do.
    MissingOption {
        /// The option.
        option: &'static str,
    },
    /// The nonce is not one this route carries.
    Nonce {
        /// What the parse answered.
        reason: String,
    },
    /// The dataset a mode names is no dataset name this route reads.
    DatasetName {
        /// The mode that named it.
        mode: &'static str,
        /// What that mode's dataset is,
        /// as the phrase after the mode's name.
        what: &'static str,
        /// The name as typed.
        name: String,
        /// Why it is no dataset name.
        reason: String,
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
    /// naming the argument and every way forward,
    /// and opening with the words every root mode is reached through.
    pub fn message(&self) -> String {
        let named: String = root_modes_name();
        let prefix: String = format!("{named}: ");
        return match self {
            ArgumentRefusal::NotText { place } => format!(
                "{prefix}argument {place} is not text, and every argument the root modes read is UTF-8 text. Nothing was done. Ways forward: give the mode word and the adoption's numbers as text; a checkout path that is not valid UTF-8 cannot be named here, so name a checkout whose path is text."
            ),
            ArgumentRefusal::UnknownMode { word } => format!(
                "{prefix}`{word}` is not a mode. Nothing was done. The modes are {}, and running the command with no mode word after `{ROOT_GROUP_WORD}` is refused rather than run. Ways forward: run `{named} --help` for the modes this build accepts.",
                MODE_WORDS.join(", ")
            ),
            ArgumentRefusal::MissingValue { option } => format!(
                "{prefix}`{option}` is the last argument and has no value. Nothing was done. Ways forward: give `{option}` its value, as `{option} <value>`, and run the command again."
            ),
            ArgumentRefusal::RepeatedOption { option } => format!(
                "{prefix}`{option}` was given more than once, and two values would compete. Nothing was done. Ways forward: give `{option}` once, then run the command again."
            ),
            ArgumentRefusal::MissingOption { option } => format!(
                "{prefix}this invocation needs `{option}`, and it was not given. Nothing was done. Ways forward: run `{named} --help` for the options the mode takes and give every one of them; `adopt-worktree-parent` takes `{CHECKOUT_OPTION} <path> {DEVICE_OPTION} <count> {INODE_OPTION} <count>`, `holder-scan` takes `{DATASET_OPTION} <name>` and, only when the caller names one, `{NONCE_OPTION} <32 hex digits>`, `record-checkout` takes `{CHECKOUT_OPTION} <path>` and, only when it replaces a record a restore moved, `{REPLACE_OPTION} <the kernel's guid>`, and `mount-path-descendant` takes `{DATASET_OPTION} <name>`."
            ),
            ArgumentRefusal::Nonce { reason } => format!(
                "{prefix}the nonce is not one this route carries: {reason}. Nothing was done. Ways forward: give `{NONCE_OPTION}` exactly {} lowercase hexadecimal digits, as the wrapper's own request carries them, or leave the option out and the scan makes its own.", crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS
            ),
            ArgumentRefusal::DatasetName { mode, what, name, reason } => format!(
                "{prefix}`{name}` is not a dataset name this route reads: {reason}. Nothing was done. Ways forward: give `{mode}`'s `{DATASET_OPTION}` {what}."
            ),
            ArgumentRefusal::OptionNotACount { option, text } => format!(
                "{prefix}`{option} {text}` is not a decimal count. Nothing was done. Ways forward: give the number `stat --format=%d` (device) or `stat --format=%i` (inode) prints for the intended replacement, as decimal digits without a sign or a prefix."
            ),
            ArgumentRefusal::UnexpectedArgument { word } => format!(
                "{prefix}`{word}` is an argument this invocation does not take. Nothing was done. Ways forward: run `{named} --help` for the modes and their arguments; a mode takes only the words the help lists beside it."
            ),
        };
    }
}

/// What:
///  The word the usage prints where the root-owned file's path belongs.
/// Why:
///   The words select a mode on any file,
///  so the usage cannot name one path as the only spelling,
///  and the file root runs is the one plan step 19's setup installs,
///  which no release installs yet:
///  the placeholder says the command needs that file without claiming one exists.
pub const ROOT_MODE_FILE_PLACEHOLDER: &str = "<file>";

/// What:
///  The whole usage text,
///  naming the modes the words reach,
///  the root-owned file they are started as,
///  and the device the modes that make requests will open.
/// Why:
///   A person meeting the modes reads what they are and what this build refuses,
///  and every line names a spelling that works:
///  the words for every mode,
///  the ZED history event link handed no argument for the sweep,
///  the root-owned file with the step that installs it,
///  and the device a change of which is a change of one constant.
///  The rows below the first line hang under the words,
///  so each names a mode word and its options and nothing else;
///  the sweep has no word there,
///  because the words with no mode word after them are refused.
///  The first line names the file before the words because the words follow
///  whatever path starts the program,
///  so the words alone reach Git as an unknown command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const usage = `usage: <file> monochromatic-git-policy-cli zfs-root <mode>`; // one row per mode word, then the closing sentences
/// ```
pub fn usage() -> String {
    let named: String = root_modes_name();
    return [
        format!("usage: {ROOT_MODE_FILE_PLACEHOLDER} {named} <mode>"),
        String::from("  boot-reconcile       reconcile clone mounts after boot; the boot unit's ExecStart"),
        String::from("  start-probe          run the start probe's arms and write this boot's result"),
        String::from("  shutdown-unmount     unmount the verified clone mounts at shutdown; the boot unit's ExecStopPost"),
        String::from("  holder-scan --dataset <name> [--nonce <32 hex digits>]"),
        String::from("                       report which mount namespaces hold that clone, and every read the scan could not complete"),
        String::from("  adopt-worktree-parent --checkout <path> --device <count> --inode <count>"),
        String::from("                       record an administrator-identified replacement .claude/worktrees folder"),
        String::from("  record-checkout --checkout <path> [--replace <the kernel's guid, which the refusal prints>]"),
        String::from("                       record this checkout's path, dataset, GUID, container and pool for the route;"),
        String::from("                       `--replace`, given the GUID the kernel shows now, replaces a record whose GUID"),
        String::from("                       a legitimate restore changed"),
        String::from("  mount-path-descendant --dataset <name>"),
        String::from("                       mount one nested dataset at its own mountpoint, under the recorded checkout"),
        String::from("  --help               print this text"),
        String::from("  --version            print this build's version"),
        format!("Every mode with a word is reached through the words `{ROOT_MODE_FILE_PLACEHOLDER} {named} <mode>`,"),
        format!("with `{ROOT_MODE_FILE_PLACEHOLDER}` the root-owned file."),
        format!("Plan step 19's setup installs `{ROOT_MODE_FILE_PLACEHOLDER}` as {ROOT_MODE_INSTALL_PATH}"),
        format!("and links it as {ZEDLET_START_NAME} for ZED; no release installs it yet."),
        format!("ZED starts {ZEDLET_INSTALL_PATH} with no argument,"),
        String::from("and that start runs the clone-mount sweep, which has no mode word."),
        format!("The modes that make requests open {DEVICE_PATH} through this repository's binding, never a CDDL library and never the zfs program."),
    ].join("\n");
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
        "start-probe" => match words.len() {
            1 => Ok(HandlerMode::StartProbe),
            _ => Err(ArgumentRefusal::UnexpectedArgument { word: words[1].to_string() }),
        },
        "shutdown-unmount" => match words.len() {
            1 => Ok(HandlerMode::ShutdownUnmount),
            _ => Err(ArgumentRefusal::UnexpectedArgument { word: words[1].to_string() }),
        },
        "holder-scan" => parse_holder_scan(&words[1..]).map(HandlerMode::HolderScan),
        "adopt-worktree-parent" => parse_adoption(&words[1..]).map(HandlerMode::AdoptWorktreeParent),
        "record-checkout" => parse_record_checkout(&words[1..]).map(HandlerMode::RecordCheckout),
        "mount-path-descendant" => parse_mount_path_descendant(&words[1..]).map(HandlerMode::MountPathDescendant),
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
///  Read a holder scan's dataset and its optional nonce,
///  each option given once and in any order.
/// Why:
///   The dataset names the clone the scan reports on,
///  and the nonce names the request this answer belongs to,
///  so a repeated option would leave which one was meant to be read,
///  and a name the kernel would refuse or a nonce that is not 32 lowercase
///  hexadecimal digits is refused here rather than read as another value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invocation = parseHolderScan(['--dataset', 'zpcachyos/…/clones/one', '--nonce', digits]);
/// ```
fn parse_holder_scan(words: &[&str]) -> Result<HolderScanInvocation, ArgumentRefusal> {
    let mut dataset: Option<String> = None;
    let mut nonce: Option<RequestNonce> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = match words[index] {
            DATASET_OPTION => DATASET_OPTION,
            NONCE_OPTION => NONCE_OPTION,
            other => return Err(ArgumentRefusal::UnexpectedArgument { word: other.to_string() }),
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue { option });
        };
        if option == DATASET_OPTION {
            if dataset.is_some() {
                return Err(ArgumentRefusal::RepeatedOption { option });
            }
            check_dataset_name(value).map_err(|refusal| return ArgumentRefusal::DatasetName {
                mode: "holder-scan",
                what: "the clone's full dataset name, as `zfs list -H -o name` prints it, with no snapshot part",
                name: String::from(value),
                reason: refusal.message(),
            })?;
            dataset = Some(String::from(value));
        } else {
            if nonce.is_some() {
                return Err(ArgumentRefusal::RepeatedOption { option });
            }
            nonce = Some(RequestNonce::parse(value).map_err(|refusal| return ArgumentRefusal::Nonce { reason: refusal.message() })?);
        }
        index += 2;
    }
    let Some(given_dataset) = dataset else {
        return Err(ArgumentRefusal::MissingOption { option: DATASET_OPTION });
    };
    return Ok(HolderScanInvocation { dataset: given_dataset, nonce });
}

/// What:
///  Read a record mode's checkout and its optional replacement token,
///  each option given once and in any order.
/// Why:
///   The checkout is the folder whose dataset the route serves and the token is
///  the GUID the kernel shows now,
///  so the checkout must be an absolute path and a repeated option would leave
///  which one was meant;
///  the token is only text here because the mode compares it with the kernel's
///  own GUID and refuses `replace-token-differs` when they differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseRecordCheckout(['--checkout', '/home/user/Monochromatic', '--replace', digits]);
/// ```
fn parse_record_checkout(words: &[&str]) -> Result<RecordCheckoutRequest, ArgumentRefusal> {
    let mut checkout: Option<PathBuf> = None;
    let mut replace: Option<String> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = match words[index] {
            CHECKOUT_OPTION => CHECKOUT_OPTION,
            REPLACE_OPTION => REPLACE_OPTION,
            other => return Err(ArgumentRefusal::UnexpectedArgument { word: other.to_string() }),
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue { option });
        };
        if option == CHECKOUT_OPTION {
            if checkout.is_some() {
                return Err(ArgumentRefusal::RepeatedOption { option });
            }
            checkout = Some(PathBuf::from(value));
        } else {
            if replace.is_some() {
                return Err(ArgumentRefusal::RepeatedOption { option });
            }
            if value.is_empty() {
                return Err(ArgumentRefusal::MissingValue { option });
            }
            replace = Some(String::from(value));
        }
        index += 2;
    }
    let Some(given_checkout) = checkout else {
        return Err(ArgumentRefusal::MissingOption { option: CHECKOUT_OPTION });
    };
    return Ok(RecordCheckoutRequest { checkout: given_checkout, replace });
}

/// What:
///  Read a mount mode's dataset.
/// Why:
///   The dataset names the work and the record names the checkout it is placed
///  under,
///  so the mode takes no second option and no path,
///  and a name the kernel would refuse is refused here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = parseMountPathDescendant(['--dataset', 'zpcachyos/nosnap/vm-builder-output']);
/// ```
fn parse_mount_path_descendant(words: &[&str]) -> Result<MountPathDescendantRequest, ArgumentRefusal> {
    let mut dataset: Option<String> = None;
    let mut index: usize = 0;
    while index < words.len() {
        let option: &str = match words[index] {
            DATASET_OPTION => DATASET_OPTION,
            other => return Err(ArgumentRefusal::UnexpectedArgument { word: other.to_string() }),
        };
        let Some(value) = words.get(index + 1).copied() else {
            return Err(ArgumentRefusal::MissingValue { option });
        };
        if dataset.is_some() {
            return Err(ArgumentRefusal::RepeatedOption { option });
        }
        check_dataset_name(value).map_err(|refusal| return ArgumentRefusal::DatasetName {
            mode: "mount-path-descendant",
            what: "the nested dataset's full name, as `zfs list -H -o name` prints it, with no snapshot part",
            name: String::from(value),
            reason: refusal.message(),
        })?;
        dataset = Some(String::from(value));
        index += 2;
    }
    let Some(given_dataset) = dataset else {
        return Err(ArgumentRefusal::MissingOption { option: DATASET_OPTION });
    };
    return Ok(MountPathDescendantRequest { dataset: given_dataset });
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
