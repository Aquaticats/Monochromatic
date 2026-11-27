//! What:
//!  The text of one claim file and the record it reads back as:
//!  the header line,
//!  the released line,
//!  the owner's three fields and every refusal between them.
//! Why:
//!  A claim decides whether a command may work on one item,
//!  so the record these bytes give is the input of every liveness decision,
//!  and one decoder is the whole of what this command reads about an owner.
//!  The record and the decoder are public,
//!  so the fuzz sidecar
//!  (`package/git-policy/cli.fuzz/src/owner_claim_record.rs`) reads generated claim text through the same decoder a
//!  command does,
//!  instead of a copy of its rules.
//!
//! Files beside this one,
//!  and why the claim is not one file:
//!  `single_owner_claim_files.rs` holds the numbered files a claim is kept in,
//!  `single_owner_claim_liveness.rs` decides whether the owner a record names still runs,
//!  and `single_owner_claim.rs` takes the claim and gives it back.
//!  `rust/max-lines` splits the claim by role,
//!  and the words of one claim file are the role a reader can hold alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ClaimRecord = { kind: 'released' } | { kind: 'owned'; pid: number; birth: string; namespace?: string };
//! const decode = (text: string): ClaimRecord | undefined => { /* ... */ };
//! ```

/// The folder names a claim file's first line is spelled with,
///  the current release's first.
use crate::wrapper_names_old_and_new::DATA_FOLDER;

/// The first line of every claim file.
const CLAIM_HEADER: &str = "monochromatic-git-policy-cli owner-claim v1";

/// The line of a released claim.
const RELEASED_LINE: &str = "released";

/// What:
///  What a claim file says,
/// as [`decode`] reads it.
/// Why:
///   A claim decides whether a command may work on one item,
/// so the record it gives is the input of every liveness decision.
/// The record is public,
/// and so is [`decode`],
/// so the fuzz sidecar
///  (`package/git-policy/cli.fuzz/src/owner_claim_record.rs`) reads generated claim text through the same decoder a
///  command does,
/// instead of a copy of its rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimRecord {
    /// The owner gave the item back.
    Released,
    /// A process owns the item:
    /// its id,
    /// birth identity and PID namespace (`-` when unknown).
    Owned {
        /// The owner's process id,
        /// which a claim names no lower than one.
        pid: i64,
        /// The owner's birth identity,
        /// which a claim names non-empty.
        birth: String,
        /// The owner's PID namespace,
        /// absent when its own namespace is the reader's or cannot be read.
        namespace: Option<String>,
    },
}

/// The text of a claim file.
pub(crate) fn encode(record: &ClaimRecord) -> String {
    return match record {
        ClaimRecord::Released => format!("{CLAIM_HEADER}\n{RELEASED_LINE}\n"),
        ClaimRecord::Owned {
            pid,
            birth,
            namespace,
        } => format!(
            "{CLAIM_HEADER}\npid={pid}\nbirth={birth}\nnamespace={}\n",
            namespace.as_deref().unwrap_or("-")
        ),
    };
}

/// What:
///  The record a claim file's text holds,
/// or nothing when the text is not a claim of this command.
/// Why:
///   The two folder spellings,
/// the released line,
/// the three owner fields and every refusal between them are the whole of what
///  this command reads about an owner,
/// so a text that reads differently here is a liveness decision from what no command wrote.
/// Public so the fuzz sidecar reads generated claims through it.
pub fn decode(text: &str) -> Option<ClaimRecord> {
    let mut lines = text.lines();
    // The files are named as before the rename, so a claim of the earlier release is found, and read in its own spelling.
    let earlier_header: String = CLAIM_HEADER.replacen(DATA_FOLDER.current, DATA_FOLDER.earlier, 1);
    let first: &str = lines.next()?;
    if first != CLAIM_HEADER && first != earlier_header {
        return None;
    }
    let second: &str = lines.next()?;
    if second == RELEASED_LINE {
        return Some(ClaimRecord::Released);
    }
    let pid: i64 = second.strip_prefix("pid=")?.parse().ok()?;
    if pid <= 0 {
        return None;
    }
    let birth: String = lines.next()?.strip_prefix("birth=")?.to_string();
    let namespace: String = lines.next()?.strip_prefix("namespace=")?.to_string();
    if birth.is_empty() {
        return None;
    }
    return Some(ClaimRecord::Owned {
        pid,
        birth,
        namespace: if namespace == "-" {
            None
        } else {
            Some(namespace)
        },
    });
}
