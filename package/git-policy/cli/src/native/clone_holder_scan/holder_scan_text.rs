//! What:
//!  The text one holder scan's result is written as:
//!  a version line,
//!  the dataset's GUID,
//!  the scan's own nonce,
//!  the source,
//!  the reader's own namespace and its mounts,
//!  one line per holder namespace,
//!  every incomplete read,
//!  and the time,
//!  plus the bound that text is read back within.
//! Why:
//!  A later step carries this answer from the root handler to the account's
//!  wrapper,
//!  so the result is written once as data and read by exactly one grammar
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  whose receipts are written the same way).
//!  The GUID and the nonce travel in the file,
//!  never in its name,
//!  so a reader compares them with the request it made and refuses a stale or
//!  foreign answer.
//!  A path is escaped the way the kernel's own mount table escapes it
//!  (`\040` for a space and so on),
//!  and every control byte is escaped too,
//!  so a mount point an account names cannot carry a terminal escape sequence
//!  or a line break into the text a later step prints.
//!  A failure's sentence is written with every byte outside printable ASCII
//!  escaped,
//!  so one reason is one line.
//!  A path that is not UTF-8 is carried lossily,
//!  which can only make a mount point compare unequal to a slot and so report
//!  one holder more than another spelling would.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = scan.text(); // 'v1\nkind clone-holder-scan\ndataset 42\n…'
//! ```

use crate::clone_holder_scan::holder_scan::{
    HolderScan,
    MAXIMUM_DETAIL_LINES,
    MAXIMUM_MOUNT_NAMESPACES,
    MAXIMUM_MOUNTS_PER_NAMESPACE,
};
use crate::clone_holder_scan::incomplete_reads::IncompleteRead;
use crate::clone_holder_scan::namespace_holder::NamespaceHolder;
use crate::foundation::iso_time::instant_of_iso;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// The result's format version,
/// which every reader checks first.
pub const RESULT_VERSION: &str = "v1";

/// The version line's key,
/// which is the version itself and carries no value.
pub const VERSION_KEY: &str = "v1";

/// The result's kind,
/// which tells it from a receipt or a readiness record.
pub const RESULT_KIND: &str = "clone-holder-scan";

/// The word a field with nothing in it carries.
pub const ABSENT_WORD: &str = "none";

/// The keys before the findings,
/// in the order the file holds them.
pub const RESULT_KEYS: &[&str] = &[
    "kind",
    "dataset",
    "nonce",
    "source",
    "own-namespace",
    "namespaces-scanned",
    "own-mounts",
    "holder-namespaces",
    "incomplete-reads",
];

/// The key of the result's last line.
pub const TIME_KEY: &str = "time";

/// The key of one mount of the reader's own namespace.
pub const OWN_MOUNT_KEY: &str = "own-mount";

/// The key of one mount of a holder namespace.
pub const HOLDER_KEY: &str = "holder";

/// The key of one read the scan could not complete.
pub const INCOMPLETE_KEY: &str = "incomplete";

/// The digits a `u64` mount count takes,
/// which is `u64::MAX`'s own width.
pub const MAXIMUM_DECIMAL_DIGITS: usize = 20;

/// The bound one namespace identity is written within.
/// Why:
///  The kernel writes a mount namespace as `mnt:[<inode>]`,
///  and the bound leaves room for a number wider than any inode it holds.
pub const MAXIMUM_NAMESPACE_BYTES: usize = 32;

/// The bound one path is written within,
/// which is the kernel's `PATH_MAX` without its terminator.
pub const MAXIMUM_PATH_BYTES: usize = 4095;

/// The bound one dataset name is written within,
/// which is OpenZFS's own dataset-name bound.
pub const MAXIMUM_SOURCE_BYTES: usize = 255;

/// The bound one time is written within:
/// the date,
/// the second,
/// nine fraction digits and the offset.
pub const MAXIMUM_TIME_BYTES: usize = 35;

/// The bound one incomplete read's own sentence is written within,
/// which is the scan's own reason bound and its cut marker.
pub const MAXIMUM_REASON_BYTES: usize = 320;

/// The bound one cause's word is written within.
pub const MAXIMUM_KIND_BYTES: usize = 32;

/// How many bytes a key,
/// its separator and the longest value on its line take.
const fn line_bytes(
    key: &str,
    bound: usize,
) -> usize {
    return key.len() + 1 + bound;
}

/// The most bytes one mount's three fields take.
const fn mount_bytes() -> usize {
    return MAXIMUM_DECIMAL_DIGITS + 1 + MAXIMUM_PATH_BYTES + 1 + MAXIMUM_PATH_BYTES;
}

/// The most bytes one holder line takes:
/// its key,
/// the namespace,
/// the task and as many mounts as one namespace's line may carry.
const fn held_line_bytes() -> usize {
    return line_bytes(
        HOLDER_KEY,
        MAXIMUM_NAMESPACE_BYTES
            + 1
            + MAXIMUM_PATH_BYTES
            + MAXIMUM_MOUNTS_PER_NAMESPACE * (1 + mount_bytes()),
    );
}

/// The most bytes one own-mount line takes.
const fn own_mount_line_bytes() -> usize {
    return line_bytes(OWN_MOUNT_KEY, mount_bytes());
}

/// The most bytes one incomplete read's line takes.
const fn incomplete_line_bytes() -> usize {
    return line_bytes(
        INCOMPLETE_KEY,
        MAXIMUM_KIND_BYTES
            + 1
            + MAXIMUM_NAMESPACE_BYTES
            + 1
            + MAXIMUM_PATH_BYTES
            + 1
            + MAXIMUM_REASON_BYTES,
    );
}

/// The header's own bytes:
/// the version line and every key of `RESULT_KEYS` and the time.
const fn header_bytes() -> usize {
    return RESULT_VERSION.len()
        + 1
        + line_bytes(RESULT_KEYS[0], RESULT_KIND.len())
        + line_bytes(RESULT_KEYS[1], MAXIMUM_DECIMAL_DIGITS)
        + line_bytes(RESULT_KEYS[2], NONCE_HEX_DIGITS)
        + line_bytes(RESULT_KEYS[3], MAXIMUM_SOURCE_BYTES)
        + line_bytes(RESULT_KEYS[4], MAXIMUM_NAMESPACE_BYTES)
        + line_bytes(RESULT_KEYS[5], MAXIMUM_DECIMAL_DIGITS)
        + line_bytes(RESULT_KEYS[6], MAXIMUM_DECIMAL_DIGITS)
        + line_bytes(RESULT_KEYS[7], MAXIMUM_DECIMAL_DIGITS)
        + line_bytes(RESULT_KEYS[8], MAXIMUM_DECIMAL_DIGITS)
        + line_bytes(TIME_KEY, MAXIMUM_TIME_BYTES)
        + RESULT_KEYS.len()
        + 1;
}

/// What:
///  The largest result a reader accepts,
///  derived from the scan's own bounds and the bound of every field the writer
///  fills.
/// Why:
///   A file bound typed in beside the scan's own limits can fall below what the
///  scan writes,
///  and then a result the writer wrote is one the reader refuses.
///  Derived this way every result the writer builds fits,
///  and a control proves a result round-trips.
pub const MAXIMUM_RESULT_BYTES: usize = header_bytes()
    + MAXIMUM_DETAIL_LINES * held_line_bytes()
    + (MAXIMUM_DETAIL_LINES + MAXIMUM_MOUNT_NAMESPACES + 2) * incomplete_line_bytes()
    + 2 * own_mount_line_bytes();

/// What:
///  One path as a field with no spaces or control bytes in it,
///  escaped the way the kernel's own mount table escapes one.
/// Why:
///   One mount point is one field,
///  so a line's fields can be split on a space,
///  and a reader that wants the path back decodes it exactly as it decodes
///  `mountinfo`'s.
///  A control byte is escaped too,
///  because an account names its own mount points and a terminal reads what is
///  printed at it:
///  an escape sequence on the mode's output is the account's,
///  not the
///  handler's,
///  and never reaches the writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const field = escapePath('/a b'); // '/a\\040b'
/// ```
pub fn escape_path(path: &Path) -> String {
    return escape_field(path.as_os_str().as_bytes());
}

/// One namespace identity or dataset name as one field.
pub fn escape_text(text: &str) -> String {
    return escape_field(text.as_bytes());
}

/// One path as text,
/// decoded exactly as a mount table's field is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = unescapePath('/a\\040b'); // '/a b'
/// ```
pub fn unescape_path(text: &str) -> PathBuf {
    return PathBuf::from(OsStr::from_bytes(
        crate::fs_mount_options::unmangle_octal(text.as_bytes()).as_slice(),
    ));
}

/// One dataset name or namespace as text,
/// decoded exactly as a mount table's field is.
pub fn unescape_text(text: &str) -> String {
    return String::from_utf8_lossy(
        crate::fs_mount_options::unmangle_octal(text.as_bytes()).as_slice(),
    )
    .into_owned();
}

/// One byte string with every byte a line cannot carry escaped:
/// a space as `\040`,
/// a backslash as `\134`,
/// and every control byte,
/// the terminal's escape among them,
/// as its three-digit octal escape.
fn escape_field(bytes: &[u8]) -> String {
    let text: String = String::from_utf8_lossy(bytes).into_owned();
    let mut escaped: String = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b' ' => escaped.push_str("\\040"),
            b'\\' => escaped.push_str("\\134"),
            control if control < 0x20 || control == 0x7f => {
                escaped.push_str(format!("\\{control:03o}").as_str())
            }
            other => escaped.push(char::from(other)),
        }
    }
    return escaped;
}

/// One failure's sentence as the last field of its line.
/// Why:
///  A reason is the last field,
///  so a space in it is harmless,
///  while a newline or a control byte would end or corrupt the line;
///  every byte outside printable ASCII is written as its hexadecimal escape.
fn printable_reason(reason: &str) -> String {
    let mut shown: String = String::with_capacity(reason.len());
    for byte in reason.bytes() {
        if byte.is_ascii_graphic() || byte == b' ' {
            shown.push(char::from(byte));
            continue;
        }
        shown.push_str(format!("\\x{byte:02x}").as_str());
    }
    return shown;
}

/// One mount's three fields,
/// which every line that names a mount ends with.
fn mount_fields(mount: &crate::clone_holder_scan::held_mounts::HeldMount) -> String {
    return format!(
        "{} {} {}",
        mount.mount_id,
        escape_path(mount.root.as_path()),
        escape_path(mount.mount_point.as_path())
    );
}

/// One namespace's mounts as one line:
/// the namespace,
/// the thread its table came from and the mount's own three fields for each
/// mount it holds.
/// Why:
///  One namespace is one holder however many mounts of the clone it holds,
///  so one line per namespace keeps the count on the line above it the number
///  of lines that follow,
///  and the mount fields are three per mount with no spaces in them.
fn namespace_line(
    holder: &NamespaceHolder,
    key: &str,
) -> String {
    let namespace: String = escape_text(holder.namespace());
    let task: String = escape_path(holder.task.as_path());
    let mut line: String = format!("{key} {namespace} {task}");
    for mount in &holder.mounts {
        line.push(' ');
        line.push_str(mount_fields(mount).as_str());
    }
    return line;
}

/// One incomplete read as the result's line.
fn incomplete_line(read: &IncompleteRead) -> String {
    let kind: &str = read.kind();
    let namespace: String = read
        .namespace()
        .map_or(String::from(ABSENT_WORD), escape_text);
    let task: String = match read.task() {
        Some(path) => escape_path(path),
        None => String::from(ABSENT_WORD),
    };
    return match read {
        IncompleteRead::Listing {
            proc_root,
            reason,
        } => {
            format!(
                "{INCOMPLETE_KEY} {kind} {} {}",
                escape_path(proc_root.as_path()),
                printable_reason(reason.as_str())
            )
        }
        IncompleteRead::NamespaceLinkUnreadable {
            thread,
            reason,
        } => {
            format!(
                "{INCOMPLETE_KEY} {kind} {} {}",
                escape_path(thread.as_path()),
                printable_reason(reason.as_str())
            )
        }
        IncompleteRead::TaskEnded {
            reason,
            ..
        }
        | IncompleteRead::TableUnreadable {
            reason,
            ..
        } => {
            format!(
                "{INCOMPLETE_KEY} {kind} {namespace} {task} {}",
                printable_reason(reason.as_str())
            )
        }
        IncompleteRead::NoLiveMember {
            tasks_tried,
            reason,
            ..
        } => format!(
            "{INCOMPLETE_KEY} {kind} {namespace} {tasks_tried} {}",
            printable_reason(reason.as_str())
        ),
        IncompleteRead::NoTaskAtTheNamespaceRoot {
            ..
        } => format!("{INCOMPLETE_KEY} {kind} {namespace}"),
        IncompleteRead::NamespacesPastTheBound {
            bound,
        }
        | IncompleteRead::DetailsPastTheBound {
            bound,
        } => format!("{INCOMPLETE_KEY} {kind} {bound}"),
        IncompleteRead::MountsPastTheBound {
            bound,
            ..
        } => format!("{INCOMPLETE_KEY} {kind} {namespace} {bound}"),
    };
}

/// What:
///  Whether a time is one this result carries.
/// Why:
///  The reader checks the time rather than trusting it,
///  so a hand-written or foreign file is refused at that line.
pub fn is_a_result_time(text: &str) -> bool {
    return text.len() <= MAXIMUM_TIME_BYTES && instant_of_iso(text).is_some();
}

impl HolderScan {
    /// The file's whole text,
    /// its final newline included.
    pub fn text(&self) -> String {
        let own_namespace: String = self
            .own_namespace
            .as_deref()
            .map_or(String::from(ABSENT_WORD), escape_text);
        let mut lines: Vec<String> = vec![
            String::from(VERSION_KEY),
            format!("kind {RESULT_KIND}"),
            format!("dataset {}", self.dataset.file_name()),
            format!("nonce {}", self.nonce.as_str()),
            format!("source {}", escape_text(self.source.as_str())),
            format!("own-namespace {own_namespace}"),
            format!("namespaces-scanned {}", self.namespaces_scanned),
            format!("own-mounts {}", self.own_mounts.len()),
        ];
        for mount in &self.own_mounts {
            lines.push(format!("{OWN_MOUNT_KEY} {}", mount_fields(mount)));
        }
        lines.push(format!("holder-namespaces {}", self.holders.len()));
        for holder in &self.holders {
            lines.push(namespace_line(holder, HOLDER_KEY));
        }
        lines.push(format!("incomplete-reads {}", self.incomplete.len()));
        for read in &self.incomplete {
            lines.push(incomplete_line(read));
        }
        lines.push(format!("{TIME_KEY} {}", self.time));
        return format!("{}\n", lines.join("\n"));
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_text_tests.rs"]
mod tests;
