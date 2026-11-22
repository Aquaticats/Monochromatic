//! What:
//!  The two names the clone-mount sweep takes from the root-owned handler
//!  configuration:
//!  the pinned clone parent dataset whose immediate children it sweeps,
//!  and the checkout it walks from.
//! Why:
//!  The sweep attaches nothing without a parent to enumerate and a checkout to
//!  walk,
//!  and the configuration is the one place the route keeps them
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  which names
//!  `/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`).
//!  The project answered on 2026-11-21 that the start probe's two names sit
//!  inside one `startProbe` object,
//!  and this object is the sweep's own beside it:
//!  `cloneMounts.parent` and `cloneMounts.checkout`.
//!  A configuration that is absent,
//!  unreadable or names nothing the parse accepts is a sweep that attaches
//!  nothing and says which key to write,
//!  never a sweep that guesses a dataset or a checkout.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const configuration = readConfiguration('/etc/…/git-worktree-clone-mounts.jsonc');
//! ```

use crate::jsonc_member_text::{MemberReading, text_of_member};
use crate::start_probe::probe_source_name::{NameRefusal, check_dataset_name};
use std::path::{Path, PathBuf};

/// The object this step's two names sit in.
pub const CLONE_MOUNTS_KEY: &str = "cloneMounts";

/// The key naming the pinned clone parent dataset.
pub const PARENT_KEY: &str = "parent";

/// The key naming the checkout the walk starts from.
pub const CHECKOUT_KEY: &str = "checkout";

/// What:
///  Where the sweep's two names are,
///  once the configuration gives them.
/// Why:
///  The sweep reads one value and walks from the other,
///  so they travel together and a caller cannot pass the parent of one
///  checkout with another checkout's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepConfiguration = { parentDataset: string; checkout: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SweepConfiguration {
    /// The pinned clone parent dataset.
    pub parent_dataset: String,
    /// The checkout whose `.claude/worktrees` folder holds the slots.
    pub checkout: PathBuf,
}

/// What:
///  Why the configuration gave the sweep no two names.
/// Why:
///  A missing file,
///  a file that does not parse,
///  a key another part has not written,
///  a value of the wrong type and a name the kernel would refuse are five
///  different things for a person to look at,
///  and each names the key they hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigurationRefusal = { kind: 'missing' | 'unreadable' | 'not-jsonc' | 'absent' | 'not-text' | 'name-refused' | 'not-absolute'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigurationRefusal {
    /// The configuration file is not there.
    Missing {
        /// The path that was read.
        path: PathBuf,
    },
    /// The configuration file could not be read.
    Unreadable {
        /// The path that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The configuration is not one JSONC document.
    NotJsonc {
        /// The path that was read.
        path: PathBuf,
    },
    /// The object or the key is not there.
    Absent {
        /// The path that was read.
        path: PathBuf,
        /// The key a person must write.
        key: String,
    },
    /// The value is not text.
    NotText {
        /// The path that was read.
        path: PathBuf,
        /// The key whose value is not text.
        key: String,
    },
    /// The parent's text is not a dataset name the kernel takes.
    ParentNameRefused {
        /// The name as written.
        name: String,
        /// Why the name rule refused it.
        why: NameRefusal,
    },
    /// The checkout's text is not an absolute path.
    CheckoutNotAbsolute {
        /// The value as written.
        text: String,
    },
}

impl ConfigurationRefusal {
    /// The key one refusal is about,
    /// which is what a person has to write.
    pub fn key(&self) -> String {
        return match self {
            ConfigurationRefusal::Missing { .. } | ConfigurationRefusal::Unreadable { .. } | ConfigurationRefusal::NotJsonc { .. } => String::from(CLONE_MOUNTS_KEY),
            ConfigurationRefusal::Absent { key, .. } | ConfigurationRefusal::NotText { key, .. } => key.clone(),
            ConfigurationRefusal::ParentNameRefused { .. } => format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}"),
            ConfigurationRefusal::CheckoutNotAbsolute { .. } => format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}"),
        };
    }

    /// The refusal in words,
    /// naming the file,
    /// the key and what to write there.
    pub fn message(&self) -> String {
        let cause: String = match self {
            ConfigurationRefusal::Missing { path } => format!("the clone-mount configuration {} is not there", path.display()),
            ConfigurationRefusal::Unreadable { path, reason } => format!("the clone-mount configuration {} could not be read ({reason})", path.display()),
            ConfigurationRefusal::NotJsonc { path } => format!("the clone-mount configuration {} is not one JSONC document", path.display()),
            ConfigurationRefusal::Absent { path, key } => format!("the clone-mount configuration {} names no `{key}`", path.display()),
            ConfigurationRefusal::NotText { path, key } => format!("`{key}` in {} is not text", path.display()),
            ConfigurationRefusal::ParentNameRefused { name, why } => format!("`{name}` is not a dataset name the kernel takes: {}", why.message()),
            ConfigurationRefusal::CheckoutNotAbsolute { text } => format!("`{text}` is not an absolute checkout path, and a relative path would be resolved against the handler's own directory"),
        };
        return format!(
            "{cause}, so this sweep enumerated nothing, attached nothing and recorded nothing, and every clone's mount intent stands untouched. Ways forward: write `{key}` as text in the root-owned file, with `{CLONE_MOUNTS_KEY}.{PARENT_KEY}` naming the clone parent dataset such as `zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets` and `{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}` naming the checkout such as `/home/user/Monochromatic`, then run the sweep again; `doctor` reports this key once plan step 18 adds it.",
            key = self.key()
        );
    }
}

/// What:
///  One text member the configuration must carry.
/// Why:
///  Both keys are read the same way,
///  so a missing one and a value of the wrong type are worded once rather than
///  twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = requiredText(document, 'checkout', path);
/// ```
fn required_text(document: &str, key: &str, path: &Path) -> Result<String, ConfigurationRefusal> {
    return match text_of_member(document, CLONE_MOUNTS_KEY, key) {
        MemberReading::Text(text) => Ok(text),
        MemberReading::DocumentNotJsonc => Err(ConfigurationRefusal::NotJsonc { path: path.to_path_buf() }),
        MemberReading::Absent => Err(ConfigurationRefusal::Absent { path: path.to_path_buf(), key: format!("{CLONE_MOUNTS_KEY}.{key}") }),
        MemberReading::NotText => Err(ConfigurationRefusal::NotText { path: path.to_path_buf(), key: format!("{CLONE_MOUNTS_KEY}.{key}") }),
    };
}

/// What:
///  Read the sweep's two names out of one configuration document.
/// Why:
///  The document is parsed once and both keys come from the same bytes,
///  so a file changed between two reads cannot make them disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = parseConfiguration(text, '/etc/…');
/// ```
///
/// # Errors
/// The document is not JSONC,
/// either key is absent or is not text,
/// the parent is not a dataset name,
/// or the checkout is not an absolute path.
pub fn parse_configuration(document: &str, path: &Path) -> Result<SweepConfiguration, ConfigurationRefusal> {
    let parent_dataset: String = required_text(document, PARENT_KEY, path)?;
    let checkout_text: String = required_text(document, CHECKOUT_KEY, path)?;
    let checkout: PathBuf = PathBuf::from(checkout_text.as_str());
    if !checkout.is_absolute() {
        return Err(ConfigurationRefusal::CheckoutNotAbsolute { text: checkout_text });
    }
    check_dataset_name(parent_dataset.as_str()).map_err(|why| return ConfigurationRefusal::ParentNameRefused { name: parent_dataset.clone(), why })?;
    return Ok(SweepConfiguration { parent_dataset, checkout });
}

/// What:
///  Read the configuration file and answer the two names it holds.
/// Why:
///  The release path takes the file the install put in place,
///  and a control hands in a fixture file,
///  so nothing here names the installed path itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const configuration = readConfiguration(path);
/// ```
///
/// # Errors
/// Everything `parse_configuration` refuses,
/// and a file that is absent or unreadable.
pub fn read_configuration(path: &Path) -> Result<SweepConfiguration, ConfigurationRefusal> {
    let document: String = std::fs::read_to_string(path).map_err(|error| {
        return match error.kind() {
            std::io::ErrorKind::NotFound => ConfigurationRefusal::Missing { path: path.to_path_buf() },
            _ => ConfigurationRefusal::Unreadable { path: path.to_path_buf(), reason: error.to_string() },
        };
    })?;
    return parse_configuration(document.as_str(), path);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_configuration_tests.rs"]
mod tests;
