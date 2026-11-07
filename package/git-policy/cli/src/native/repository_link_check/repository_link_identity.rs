//! What:
//!  The identity file `~/.config/monochromatic-dev/repository-link.json` and its reader.
//! Why:
//!  The cut-over step records the link path,
//! its exact text,
//! the approved canonical root,
//! the 860's mount point and filesystem UUID,
//! the first commit of `main` and the phase.
//! The check reads that file and never writes it.
//! The reader is lenient the way the TypeScript reader is:
//! a field that is missing or not text is the empty text,
//! and anything but the phase `after` is the phase `before`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = await readIdentity(join(home, '.config/monochromatic-dev/repository-link.json'));
//! ```

use serde_json::Value;
use std::path::{Path, PathBuf};

/// The identity file below the home directory.
pub const IDENTITY_RELATIVE_PATH: &str = ".config/monochromatic-dev/repository-link.json";

/// What:
///  The phase the identity describes.
/// Why:
///  `Before` while the repository is still a real folder on the NVMe,
/// `After` once the old path is a link to the 860.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkPhase = 'after' | 'before';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkPhase {
    /// The repository is a real folder on the NVMe.
    Before,
    /// The old path is a link to the 860.
    After,
}

/// What:
///  What the cut-over recorded.
/// Why:
///  The check enforces exactly this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkIdentity = { linkPath; linkText; canonicalRoot; mountPoint; driveUuid; nvmeUuid; rootCommit; phase };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkIdentity {
    /// The old repository path.
    pub link_path: String,
    /// The exact text of the link;
    /// empty in phase `before`.
    pub link_text: String,
    /// The approved canonical root the link must resolve to.
    pub canonical_root: String,
    /// The 860's mount point.
    pub mount_point: String,
    /// Filesystem UUID of the 860.
    pub drive_uuid: String,
    /// Filesystem UUID of the NVMe that holds the repository before the move.
    pub nvme_uuid: String,
    /// First commit hash of `main`.
    pub root_commit: String,
    /// The phase the identity describes.
    pub phase: LinkPhase,
}

/// What:
///  What reading the identity file found.
/// Why:
///  An absent file is the usual case and must cost nothing more than the failed open;
/// an unreadable one is told apart so the gate can say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IdentityRead = { found: false } | { found: true; identity: LinkIdentity } | { error: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityRead {
    /// There is no identity file.
    Absent,
    /// The file was read.
    Found(LinkIdentity),
    /// The file exists but cannot be read or is not JSON;
    /// the text says why.
    Unreadable(String),
}

/// What:
///  The path of the identity file for a home directory.
/// Why:
///  The home comes from the invocation's environment,
/// never a fixed user.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityPathFor(home)
/// ```
pub fn identity_path_in(home: &Path) -> PathBuf {
    return home.join(IDENTITY_RELATIVE_PATH);
}

/// The text of one field,
/// or the empty text when it is missing or not text.
fn text_of(parsed: &Value, key: &str) -> String {
    return match parsed.get(key) {
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    };
}

/// What:
///  The identity of parsed JSON text.
/// Why:
///  Mirrors `identityFrom` of the TypeScript reader:
/// a field that is missing or not text becomes the empty text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityFrom(JSON.parse(text))
/// ```
pub fn identity_from_text(text: &str) -> Result<LinkIdentity, String> {
    let parsed: Value = serde_json::from_str(text).map_err(|error| {
        return error.to_string();
    })?;
    return Ok(LinkIdentity {
        link_path: text_of(&parsed, "linkPath"),
        link_text: text_of(&parsed, "linkText"),
        canonical_root: text_of(&parsed, "canonicalRoot"),
        mount_point: text_of(&parsed, "mountPoint"),
        drive_uuid: text_of(&parsed, "driveUuid"),
        nvme_uuid: text_of(&parsed, "nvmeUuid"),
        root_commit: text_of(&parsed, "rootCommit"),
        phase: if text_of(&parsed, "phase") == "after" { LinkPhase::After } else { LinkPhase::Before },
    });
}

/// What:
///  Reads the identity file below `home`.
/// Why:
///  One `read` call is the whole cost when the file is absent:
/// the gate runs on every Git command,
/// so an absent file is a single failed open and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readIdentity(identityPathFor(home))
/// ```
pub fn read_identity(home: &Path) -> IdentityRead {
    let path: PathBuf = identity_path_in(home);
    let bytes: Vec<u8> = match std::fs::read(path.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return IdentityRead::Absent,
        Err(error) => return IdentityRead::Unreadable(format!("cannot read {}: {error}", path.display())),
    };
    return match identity_from_text(String::from_utf8_lossy(bytes.as_slice()).as_ref()) {
        Ok(identity) => IdentityRead::Found(identity),
        Err(why) => IdentityRead::Unreadable(format!("{} is not valid JSON: {why}", path.display())),
    };
}
