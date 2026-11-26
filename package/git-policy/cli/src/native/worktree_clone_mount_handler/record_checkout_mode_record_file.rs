//! What:
//!  The checkout record as the file it is:
//!  the read that refuses a file or a folder another account could write,
//!  the write that stands under a name of its own and is renamed into place,
//!  the two folder proofs the write makes,
//!  and the three lines an agreeing run,
//!  a first run and a replace answer.
//! Why:
//!  `record_checkout_mode.rs` decides which facts to write and would go over the
//!  file-size limit carrying the file's own handling too,
//!  and that decision reads and writes the record through these functions and
//!  nothing else,
//!  so a reader holds the file's permissions and its atomicity apart from the
//!  proofs that choose the facts.
//!  Every refusal here is the record's own:
//!  a folder another account could write,
//!  a file that is not the record this route wrote,
//!  and a write that failed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorded = readRecord('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json');
//! ```

use super::record_checkout_mode_record::{
    RECORD_FOLDER_MODE,
    RECORD_MODE,
    RecordCheckoutFailure,
    RecordedCheckout,
    WRITABLE_BY_OTHERS,
    WRITING_NAME,
};
use crate::file_only_root_may_write::{
    FileControlRefusal,
    read_file_only_root_may_write,
};
use rustix::process::geteuid;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The record as the file holds it,
///  or nothing when the file is not there.
/// Why:
///   The read refuses a file another account could write,
///  and a file that is not the record this mode writes is a refusal rather than
///  a difference,
///  because a record nobody could read decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = readRecord('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json');
/// ```
///
/// # Errors
/// The file is not one root alone may write,
/// it could not be read,
/// or it is not the record's shape.
pub(crate) fn read_record(path: &Path) -> Result<Option<RecordedCheckout>, RecordCheckoutFailure> {
    // The folder a record stands in decides whether the record is the one this
    // route wrote, so it answers before the file is read:
    // a record another account could replace in its own folder is no record.
    match path.parent() {
        Some(folder) if folder.exists() => check_the_record_folder(folder)?,
        _ => {}
    }
    let text: Option<String> = match read_file_only_root_may_write(path) {
        Ok(text) => text,
        Err(
            refusal @ FileControlRefusal::Open {
                ..
            },
        ) => {
            return Err(RecordCheckoutFailure::RecordUnreadable {
                path: path.to_path_buf(),
                reason: refusal.reason(),
            });
        }
        Err(refusal) => {
            return Err(RecordCheckoutFailure::RecordUnreadable {
                path: path.to_path_buf(),
                reason: refusal.reason(),
            });
        }
    };
    let Some(record_text) = text else {
        return Ok(None);
    };
    let document: serde_json::Value =
        serde_json::from_str(record_text.as_str()).map_err(|error| {
            return RecordCheckoutFailure::RecordUnreadable {
                path: path.to_path_buf(),
                reason: format!("it is not JSON ({error})"),
            };
        })?;
    let Some(object) = document.as_object() else {
        return Err(RecordCheckoutFailure::RecordUnreadable {
            path: path.to_path_buf(),
            reason: String::from("it is not a JSON object"),
        });
    };
    let version: u64 = object
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            return RecordCheckoutFailure::RecordUnreadable {
                path: path.to_path_buf(),
                reason: String::from("it holds no `version` this mode wrote"),
            };
        })?;
    let text_field = |key: &str| -> Result<String, RecordCheckoutFailure> {
        return object
            .get(key)
            .and_then(serde_json::Value::as_str)
            .map(String::from)
            .ok_or_else(|| {
                return RecordCheckoutFailure::RecordUnreadable {
                    path: path.to_path_buf(),
                    reason: format!("it holds no text `{key}`"),
                };
            });
    };
    return Ok(Some(RecordedCheckout {
        version,
        checkout: PathBuf::from(text_field("checkout")?),
        dataset: text_field("dataset")?,
        guid: text_field("guid")?,
        container: text_field("container")?,
        pool: text_field("pool")?,
    }));
}

/// What:
///  Write the record,
///  making its folder when that is absent and refusing one another account
///  could write.
/// Why:
///   The record is root's,
///  so a folder or a file another account could rewrite would make every proof
///  that reads it advisory;
///  the write stands under a name of its own and is renamed into place,
///  so a reader sees the old record or the new one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeRecord('/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json', proved);
/// ```
///
/// # Errors
/// The folder could not be made or is not root's,
/// and the write itself.
pub(super) fn write_record(
    path: &Path,
    proved: &RecordedCheckout,
) -> Result<(), RecordCheckoutFailure> {
    let folder: PathBuf = path
        .parent()
        .map_or(PathBuf::from("/"), |parent| return parent.to_path_buf());
    make_the_record_folder(folder.as_path())?;
    #[allow(
        clippy::disallowed_methods,
        reason = "the JSON macro builds one object of every field; no fallible read is made here"
    )]
    let object: serde_json::Value = serde_json::json!({
        "version": proved.version,
        "checkout": proved.checkout.display().to_string(),
        "dataset": proved.dataset,
        "guid": proved.guid,
        "container": proved.container,
        "pool": proved.pool,
    });
    let document: String = format!("{object:#}\n");
    let held: std::fs::File = crate::foundation::entries_of_one_open_folder::open_folder(
        folder.as_path(),
    )
    .map_err(|error| {
        return RecordCheckoutFailure::RecordUnwritable {
            path: path.to_path_buf(),
            reason: error.to_string(),
        };
    })?;
    let writing: std::fs::File = match rustix::fs::openat(
        &held,
        WRITING_NAME,
        rustix::fs::OFlags::WRONLY
            | rustix::fs::OFlags::CREATE
            | rustix::fs::OFlags::EXCL
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::from_bits_truncate(RECORD_MODE),
    ) {
        Ok(opened) => std::fs::File::from(opened),
        Err(error) => {
            let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
            return Err(RecordCheckoutFailure::RecordUnwritable {
                path: path.to_path_buf(),
                reason: error.to_string(),
            });
        }
    };
    let mut writer: std::fs::File = writing;
    let written: Result<(), std::io::Error> = writer
        .write_all(document.as_bytes())
        .and_then(|()| return writer.write_all(b"\n"))
        .and_then(|()| return writer.sync_all());
    drop(writer);
    if let Err(error) = written {
        let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: path.to_path_buf(),
            reason: error.to_string(),
        });
    }
    let Some(name) = path.file_name() else {
        let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: path.to_path_buf(),
            reason: String::from("the record's path has no file name"),
        });
    };
    if let Err(error) = rustix::fs::renameat(&held, WRITING_NAME, &held, name) {
        let _removed = rustix::fs::unlinkat(&held, WRITING_NAME, rustix::fs::AtFlags::empty());
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: path.to_path_buf(),
            reason: error.to_string(),
        });
    }
    let _synced = held.sync_all();
    return Ok(());
}

/// What:
///  Make the record's folder when it is absent,
///  and refuse one that is not root's or that another account could write.
/// Why:
///  Step 19's setup runs the same `install -d` line,
///  so a host that never ran the mode still has the folder,
///  and a host whose folder was replaced answers which term failed rather
///  than writing into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// makeTheRecordFolder('/var/lib/monochromatic-git-policy-cli');
/// ```
///
/// # Errors
/// The folder could not be made,
/// or it is not root's or another account could write it.
fn make_the_record_folder(folder: &Path) -> Result<(), RecordCheckoutFailure> {
    if !folder.exists() {
        std::fs::create_dir_all(folder).map_err(|error| {
            return RecordCheckoutFailure::RecordUnwritable {
                path: folder.to_path_buf(),
                reason: error.to_string(),
            };
        })?;
        let _moded = rustix::fs::chmod(
            folder,
            rustix::fs::Mode::from_bits_truncate(RECORD_FOLDER_MODE),
        );
    }
    return check_the_record_folder(folder);
}

/// What:
///  Refuse the record's folder when it is not this program's account's or
///  when another account could write it.
/// Why:
///  The record decides which checkout the route places a mount under,
///  so the folder it stands in is part of the proof,
///  and a folder another account could add a file to would let that account
///  answer for the route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkTheRecordFolder('/var/lib/monochromatic-git-policy-cli');
/// ```
///
/// # Errors
/// The folder is not this account's,
/// or a group or other write bit is set.
fn check_the_record_folder(folder: &Path) -> Result<(), RecordCheckoutFailure> {
    let held: std::fs::File = crate::foundation::entries_of_one_open_folder::open_folder(folder)
        .map_err(|error| {
            return RecordCheckoutFailure::RecordUnwritable {
                path: folder.to_path_buf(),
                reason: error.to_string(),
            };
        })?;
    let metadata: std::fs::Metadata = held.metadata().map_err(|error| {
        return RecordCheckoutFailure::RecordUnwritable {
            path: folder.to_path_buf(),
            reason: error.to_string(),
        };
    })?;
    use std::os::unix::fs::MetadataExt;
    let this_account: u32 = geteuid().as_raw();
    if metadata.uid() != this_account {
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: folder.to_path_buf(),
            reason: format!(
                "its owner is user {} and this program runs as user {this_account}, so the account that owns it could rewrite the record this route acts on",
                metadata.uid()
            ),
        });
    }
    if metadata.mode() & WRITABLE_BY_OTHERS != 0 {
        return Err(RecordCheckoutFailure::RecordUnwritable {
            path: folder.to_path_buf(),
            reason: format!(
                "its permission bits are {:04o}, and {WRITABLE_BY_OTHERS:04o} would let another account write the record",
                metadata.mode() & 0o7777
            ),
        });
    }
    return Ok(());
}

/// The line a first record answers.
pub(super) fn recorded_line(
    proved: &RecordedCheckout,
    record: &Path,
) -> String {
    return format!(
        "recorded: {} is {} (GUID {}), in pool {}, beside container {}; the record is {}",
        proved.checkout.display(),
        proved.dataset,
        proved.guid,
        proved.pool,
        proved.container,
        record.display()
    );
}

/// The line a run that changed nothing answers.
pub(super) fn already_recorded_line(record: &Path) -> String {
    return format!(
        "already recorded: {}, and every field agrees with the kernel",
        record.display()
    );
}

/// The line a replace answers.
pub(super) fn replaced_line(
    proved: &RecordedCheckout,
    was: &str,
    record: &Path,
) -> String {
    return format!(
        "replaced: {} keeps its path, dataset, container and pool; its GUID was {was} and is now {}. The record is {}.",
        proved.checkout.display(),
        proved.guid,
        record.display()
    );
}
