//! What:
//!  The words one invocation of the root modes is read as:
//!  the mode words,
//!  the options each mode takes,
//!  the two words that print what this build is,
//!  and the usage text naming every one of them.
//! Why:
//!  `handler_mode_arguments.rs` reads these words and answers a word it does
//!  not know and would go over the file-size limit carrying them too,
//!  and every one is a spelling a person types or reads,
//!  so a reader holds the spellings together rather than among the code that
//!  matches on them.
//!  Nothing here reads an argument:
//!  the usage is printed by the dispatcher and the words are matched by the
//!  parse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const usage = `usage: <file> monochromatic-git-policy-cli zfs-root <mode>`; // one row per mode word
//! ```

use crate::worktree_clone_mount_handler::handler_install_paths::{ROOT_MODE_INSTALL_PATH, ZEDLET_INSTALL_PATH, root_modes_name};
use crate::worktree_clone_mount_handler::root_mode_names::ZEDLET_START_NAME;
use monochromatic_zfs_request_to_openzfs_kernel_device::DEVICE_PATH;

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
