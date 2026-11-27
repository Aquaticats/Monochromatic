//! What:
//!  The durable work-item record itself:
//! its version,
//! the directories a queue lives in,
//! the fields one item carries,
//! the one JSON line it is written as,
//! the reader of that line,
//! and the claimed item a consumer works on.
//! Why:
//!  The steps that take an item (`work_item_claim_taking.rs`)
//! and settle it (`work_item_settling.rs`),
//! every producer that queues work,
//! and every consumer that drains it
//! read and write this record,
//! so it sits in a file that uses none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = item.encode(); const back = WorkItem.decode(line); const id = back.id();
//! ```

use crate::path_bytes_in_records as path_bytes;
use crate::work_item_kinds_and_their_names::WorkKind;
use serde_json::{
    Map,
    Value,
};
use std::path::PathBuf;

/// The record format version.
pub const VERSION: u64 = 1;

/// Directory under the worktree's own Git directory (`WorkItem::git_dir`) that holds the state directories.
pub const WORK_DIRECTORY: &str = "monochromatic-git-policy-cli/work";

/// What:
///  One work item:
/// what to do,
/// to which landed object,
/// and where.
/// Why:
///   The consumer needs only the object name and the places to find the repository;
/// it reads the patch itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkItem = { kind; repository: string; gitDir: string; ref: string; oid: string; parent: string; created: string; tries: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkItem {
    /// What to do.
    pub kind: WorkKind,
    /// The worktree root.
    pub repository: PathBuf,
    /// The Git directory of the worktree that made the commit and whose service instance runs the item:
    /// the common directory
    ///  for the main worktree,
    /// `<common>/worktrees/<name>` for a linked one.
    /// The work records are under it,
    /// and
    ///  `is_acceptable` refuses an item whose value is not the directory it is read from.
    pub git_dir: PathBuf,
    /// The branch reference that received the commit.
    pub branch_ref: String,
    /// The landed commit.
    pub oid: String,
    /// Its first parent,
    /// or empty for a root commit.
    pub parent: String,
    /// `toISOString` text of the enqueue time.
    pub created: String,
    /// Failed or abandoned attempts so far.
    pub tries: u32,
    /// What:
    ///  Whether the command that asked for this work chose the transport its push speaks through.
    /// Why:
    ///   `GIT_SSH`,
    /// `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` are held only in the memory of the service that runs the push
    /// (`service_push_programs_held_in_memory`),
    /// so a service that restarted has none of them;
    /// the record is then the only thing left that says the push must not run through the manager's transport instead,
    /// and a word can say so to the person (`push_without_its_transport`).
    /// It is a flag and never a value:
    /// a command line that can hold a password reaches no record.
    pub transport: bool,
}

/// Methods over an item.
impl WorkItem {
    /// What:
    ///  The id and file name:
    /// the kind,
    /// the landed object,
    /// and the one field of the record that tells two of them apart,
    /// so the same work always yields the same id.
    /// Why:
    ///   A push names its branch,
    /// because the same commit on two branches is two pushes to two remotes,
    /// and a subvolume delete names the directory that held it,
    /// because a
    ///  subvolume id is unique on one filesystem only.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const id = (item) =>
    ///   item.kind === 'push'
    ///     ? `${item.kind}-${item.oid}-${branchDigest(item.ref)}`
    ///     : item.kind === 'subvolume-delete'
    ///       ? `${item.kind}-${item.oid}-${parentDigest(item.parent)}`
    ///       : `${item.kind}-${item.oid}`
    /// ```
    pub fn id(&self) -> String {
        if self.kind == WorkKind::Push {
            return format!(
                "{}-{}-{}",
                self.kind.name(),
                self.oid,
                crate::work_item_ids_and_names::branch_digest(self.branch_ref.as_str())
            );
        }
        if self.kind == WorkKind::SubvolumeDelete {
            return format!(
                "{}-{}-{}",
                self.kind.name(),
                self.oid,
                crate::work_item_ids_and_names::parent_digest(self.parent.as_str())
            );
        }
        return format!("{}-{}", self.kind.name(), self.oid);
    }

    /// What:
    ///  The id a release that keyed a record by its kind and object alone wrote for this item.
    /// Why:
    ///   Such a record is on disk in a repository this release took over,
    /// and it is still this item's:
    /// the record's
    ///  name must not decide that a push,
    /// or a subvolume delete,
    /// left over from the release before this one is other
    ///  work.
    /// The
    ///  current id is what this release writes,
    /// so the two are asked for together wherever a name is matched against a record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `${kind}-${oid}`
    /// ```
    pub fn id_before_the_branch(&self) -> String {
        return format!("{}-{}", self.kind.name(), self.oid);
    }

    /// The record's one JSON line.
    pub fn encode(&self) -> String {
        let mut fields: Map<String, Value> = Map::new();
        fields.insert(String::from("v"), Value::from(VERSION));
        fields.insert(String::from("tries"), Value::from(self.tries));
        // A directory goes in as plain text and as its bytes, so a reader of either form finds the one the wrapper named.
        path_bytes::write_field(&mut fields, "repository", self.repository.as_path());
        path_bytes::write_field(&mut fields, "gitDir", self.git_dir.as_path());
        for (name, text) in [
            ("kind", self.kind.name()),
            ("id", self.id().as_str()),
            ("ref", self.branch_ref.as_str()),
            ("oid", self.oid.as_str()),
            ("parent", self.parent.as_str()),
            ("created", self.created.as_str()),
        ] {
            fields.insert(String::from(name), Value::from(text));
        }
        // A record of a push whose command chose no transport says nothing, so every item of a repository that never set one
        //  encodes exactly as it did before this field existed.
        if self.transport {
            fields.insert(String::from("transport"), Value::from(true));
        }
        return format!("{}\n", Value::Object(fields));
    }

    /// Read a record;
    /// any other version,
    /// an unknown kind,
    /// a missing field or an id that is not derived from the kind and
    /// object is refused with a message.
    pub fn decode(text: &str) -> Result<WorkItem, String> {
        let value: Value = match serde_json::from_str(text) {
            Ok(parsed) => parsed,
            Err(error) => return Err(format!("the record is not JSON: {error}")),
        };
        if value.get("v").and_then(Value::as_u64) != Some(VERSION) {
            return Err(String::from("the record has an unsupported version"));
        }
        let kind: WorkKind = match WorkKind::from_name(text_field(&value, "kind")?.as_str()) {
            Some(known) => known,
            None => return Err(String::from("the record has an unknown kind")),
        };
        // A count that is missing, or too large for a count of tries, is no count.
        let Some(count) = value.get("tries").and_then(Value::as_u64) else {
            return Err(String::from("the record has no tries"));
        };
        let Ok(tries) = u32::try_from(count) else {
            return Err(String::from("the record has no tries"));
        };
        let item: WorkItem = WorkItem {
            kind,
            repository: path_bytes::read_field(&value, "repository")?,
            git_dir: path_bytes::read_field(&value, "gitDir")?,
            branch_ref: text_field(&value, "ref")?,
            oid: text_field(&value, "oid")?,
            parent: text_field(&value, "parent")?,
            created: text_field(&value, "created")?,
            tries,
            // A record of an earlier release, or of a push whose command chose no transport, says nothing.
            transport: value
                .get("transport")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        };
        let named: String = text_field(&value, "id")?;
        if named != item.id() && named != item.id_before_the_branch() {
            return Err(String::from(
                "the record's id is not derived from its kind, object and branch",
            ));
        }
        return Ok(item);
    }
}

/// What:
/// What:
///  Whether this text is a record written by a release later than this one:
///  a version above this build's,
///  or a kind this build does not know.
/// Why:
///  A queue outlives the program that writes to it,
///  and every release of this program shares one queue per Git directory.
///  A record a later release queued must be left for that release,
///  and a service that settled it in `failed` would take away work nothing ever reads again
///  (`take_entry`).
///  Only the two fields that say how to read the record are asked,
///  so a record of a kind this build knows and a field it does not is still this build's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const later = (text) => { const value = JSON.parse(text); return value.v > VERSION || !WorkKind.fromName(value.kind) };
/// ```
pub(crate) fn record_of_a_later_release(text: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    if let Some(version) = value.get("v").and_then(Value::as_u64)
        && version > VERSION
    {
        return true;
    }
    return match value.get("kind").and_then(Value::as_str) {
        Some(name) => WorkKind::from_name(name).is_none(),
        None => false,
    };
}

///  A named field of a work record as text.
/// Why:
///  `WorkItem::decode` reads every text field of the record,
/// and a field that is missing or not text refuses the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const textField = (value: object, name: string): string => { if (typeof value[name] !== 'string') throw new Error(`the record has no text field ${name}`); return value[name]; };
/// ```
fn text_field(
    value: &Value,
    name: &str,
) -> Result<String, String> {
    return match value.get(name).and_then(Value::as_str) {
        Some(text) => Ok(String::from(text)),
        None => Err(format!("the record has no text field {name}")),
    };
}

/// What:
///  A held item:
/// the decoded record,
/// its id,
/// and the name of the file that holds it.
/// Why:
///   Finishing needs the claim's file name,
/// not the id:
/// the name carries the generation of the claim,
/// so a settlement or a
///  renewal reaches only the claim it was made for.
/// The consumer needs the item.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Claimed = { id: string; name: string; item: WorkItem };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    /// File name in every state directory but `claimed` and `recovering`,
    /// where the file name is `name`.
    pub id: String,
    /// File name in `claimed` as of the claim or the last renewal:
    /// the id,
    /// the claim's generation and the clock reading.
    pub name: String,
    /// The decoded record.
    pub item: WorkItem,
}
