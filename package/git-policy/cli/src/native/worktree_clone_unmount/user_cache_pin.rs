//! What:
//!  The pin of the `user-cache` mount,
//!  the account's passwd home it is reached from,
//!  and the check that runs before the wrapper creates a staging folder and
//!  before the handler binds one.
//! Why:
//!  A bind made before `zpcachyos/nosnap/user-cache` mounts lands on the home
//!  underlay and is hidden when the dataset mounts later,
//!  so the pin's source carries the ordering check,
//!  and its mount-root inode catches a mount at `~/.cache` whose root is not the
//!  dataset's root
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision";
//!  `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  M5).
//!  The pin never compares the device:
//!  that is ZFS's anonymous per-mount minor,
//!  which a reboot may change,
//!  so it is recorded for `doctor` and gated on by nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pin = readUserCachePin('/var/lib/…/user-cache-pin');
//! const cache = openTheCache(accountHome(uid));
//! checkUserCachePin(cache, pin);
//! ```

use crate::fs_beneath::{
    open_directory_crossing_one_mount,
    open_root,
};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::mount_facts_of;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// The root-owned file the pin is recorded in.
///
/// Why:
///  Step 19's setup writes it and `doctor` prints it;
///  a pin read from a file the account could write would be no pin at all.
pub const USER_CACHE_PIN_PATH: &str = "/var/lib/monochromatic-git-policy-cli/user-cache-pin";

/// The pin record's version.
pub const PIN_VERSION: &str = "v1";

/// The pin record's kind.
pub const PIN_KIND: &str = "user-cache-pin";

/// The largest pin file this reader accepts.
pub const MAXIMUM_PIN_BYTES: u64 = 4096;

/// The buffer `getpwuid_r` fills,
/// large enough for one passwd entry on every account this route serves.
const PASSWD_BUFFER_BYTES: usize = 16_384;

/// What:
///  The two fields every check compares,
///  and the device `doctor` prints.
/// Why:
///  The source says whether the cache dataset is mounted at all,
///  the inode says whether the mount at `.cache` is that dataset's root,
///  and the device is recorded but never compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UserCachePin = { source: string; rootInode: bigint; device: bigint };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserCachePin {
    /// The dataset's own name,
    /// as the mount table prints it.
    pub source: String,
    /// The inode of the mount root under that dataset.
    pub root_inode: u64,
    /// The device the boot that wrote the pin read,
    /// for `doctor` only.
    pub device: u64,
}

/// What:
///  Why the pin or the cache folder it pins could not be used.
/// Why:
///  A missing pin,
///  a pin of another dataset,
///  a cache mount whose root is not the dataset's,
///  and a cache folder that is not a directory are distinct things a person
///  checks,
///  and each refusal names what it compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PinRefusal = { kind: 'missing' | 'unreadable' | 'malformed' | 'source' | 'inode' | 'home' | 'cache'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PinRefusal {
    /// The pin file is absent.
    Missing {
        /// The path that was read.
        path: PathBuf,
    },
    /// The pin file or the cache folder could not be read.
    Unreadable {
        /// The path that was read.
        path: String,
        /// What the read answered.
        reason: String,
    },
    /// The pin file is not one this route writes.
    Malformed {
        /// What about it is wrong.
        said: String,
    },
    /// The mount's source is not the pinned dataset.
    Source {
        /// The source the pin records.
        pinned: String,
        /// The source the mount reports.
        found: String,
    },
    /// The mount's root inode is not the pinned one.
    Inode {
        /// The inode the pin records.
        pinned: u64,
        /// The inode the mount root reports.
        found: u64,
    },
    /// The account's passwd entry gives no home,
    /// or no such account.
    Home {
        /// The account whose home was asked for.
        uid: u32,
        /// What the lookup answered.
        said: String,
    },
}

impl PinRefusal {
    /// The refusal in words.
    pub fn message(&self) -> String {
        return match self {
            PinRefusal::Missing {
                path,
            } => format!(
                "the `user-cache` pin is not at {}. Nothing was created or bound, because a bind made before the cache dataset mounts lands on the home underlay. Ways forward: run the one-time setup that records the pin, then run the removal again.",
                path.display()
            ),
            PinRefusal::Unreadable {
                path,
                reason,
            } => format!(
                "{path} could not be read: {reason}. Nothing was created or bound on a mount that could not be checked."
            ),
            PinRefusal::Malformed {
                said,
            } => format!(
                "the `user-cache` pin is not one this route writes: {said}. Nothing was created or bound on a pin that cannot be read, and `doctor` prints this state."
            ),
            PinRefusal::Source {
                pinned,
                found,
            } => format!(
                "the mount at the account's `.cache` has source `{found}`, and the pin records `{pinned}`. A bind made here would land on the wrong mount. Nothing was created or bound; ways forward: check that the pinned dataset is mounted at the account's `.cache`, or record the pin again for the dataset the machine mounts there."
            ),
            PinRefusal::Inode {
                pinned,
                found,
            } => format!(
                "the mount at the account's `.cache` has root inode {found}, and the pin records {pinned}. The mount at that name is not the dataset's root. Nothing was created or bound; ways forward: check for a bind or another mount at the account's `.cache` and remove it, then run the removal again."
            ),
            PinRefusal::Home {
                uid,
                said,
            } => format!(
                "the home of account {uid} could not be read from the passwd database: {said}. The staging folder lives under that home, so nothing was created or bound."
            ),
        };
    }
}

/// The pin record as the file holds it.
impl UserCachePin {
    /// The record's text.
    pub fn encode(&self) -> String {
        #[allow(
            clippy::disallowed_methods,
            reason = "the JSON macro builds one object of every field; no fallible read is made here"
        )]
        let object: serde_json::Value = serde_json::json!({
            "version": PIN_VERSION,
            "kind": PIN_KIND,
            "source": self.source,
            "root-inode": self.root_inode.to_string(),
            "device": self.device.to_string(),
        });
        return format!("{object}\n");
    }
}

/// One decimal field of a pin object.
fn pin_number(
    object: &serde_json::Map<String, serde_json::Value>,
    key: &'static str,
) -> Result<u64, PinRefusal> {
    let Some(text) = crate::json_record::string_field(object, key) else {
        return Err(PinRefusal::Malformed {
            said: format!("`{key}` is absent or is not text"),
        });
    };
    return text.parse::<u64>().map_err(|_error| {
        return PinRefusal::Malformed {
            said: format!("`{key}` is not a decimal number"),
        };
    });
}

/// What:
///  Read one pin record text.
/// Why:
///  The source is the one field a person can mistype,
///  so it is checked as non-empty text and every number as a decimal number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = userCachePinFromText(text);
/// ```
pub fn pin_from_text(text: &str) -> Result<UserCachePin, PinRefusal> {
    let object: serde_json::Map<String, serde_json::Value> = crate::json_record::parse_object(text)
        .ok_or_else(|| {
            return PinRefusal::Malformed {
                said: String::from("the text is not one JSON object"),
            };
        })?;
    let version: &str = crate::json_record::string_field(&object, "version").unwrap_or("");
    let kind: &str = crate::json_record::string_field(&object, "kind").unwrap_or("");
    if version != PIN_VERSION || kind != PIN_KIND {
        return Err(PinRefusal::Malformed {
            said: format!(
                "its version is `{version}` and its kind `{kind}`, and this route reads `{PIN_VERSION}` `{PIN_KIND}`"
            ),
        });
    }
    let source: &str = crate::json_record::string_field(&object, "source").unwrap_or("");
    if source.is_empty() {
        return Err(PinRefusal::Malformed {
            said: String::from("`source` is absent or empty"),
        });
    }
    return Ok(UserCachePin {
        source: String::from(source),
        root_inode: pin_number(&object, "root-inode")?,
        device: pin_number(&object, "device")?,
    });
}

/// What:
///  Read the pin from its root-owned file.
/// Why:
///  The release path reads the file step 19's setup writes;
///  a missing file is a refusal rather than a pin derived from whatever is
///  mounted,
///  because such a pin would agree with the mount it is meant to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pin = readUserCachePin('/var/lib/…/user-cache-pin');
/// ```
pub fn read_pin(path: &Path) -> Result<UserCachePin, PinRefusal> {
    let bytes: Vec<u8> = match crate::plain_file::read_plain_file(path, MAXIMUM_PIN_BYTES) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(PinRefusal::Missing {
                path: path.to_path_buf(),
            });
        }
        Err(error) => {
            return Err(PinRefusal::Unreadable {
                path: format!("the `user-cache` pin at {}", path.display()),
                reason: error.to_string(),
            });
        }
    };
    let text: &str = crate::json_record::decode_fatal(bytes.as_slice()).ok_or_else(|| {
        return PinRefusal::Malformed {
            said: String::from("the file is not UTF-8 text"),
        };
    })?;
    return pin_from_text(text);
}

/// What:
///  Write one pin record for step 19's setup and the guest's fixture.
/// Why:
///  The pin is root's record,
///  so the writer takes the directory it is written in and the pin,
///  and the guest arm writes the fixture's dataset name there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeUserCachePin(directory, pin);
/// ```
pub fn write_pin(
    directory: &std::fs::File,
    pin: &UserCachePin,
) -> Result<(), PinRefusal> {
    use std::io::Write;
    let temporary: std::ffi::OsString = std::ffi::OsString::from("user-cache-pin.tmp");
    let final_name: std::ffi::OsString = std::ffi::OsString::from("user-cache-pin");
    let _stale = crate::fs_beneath::remove_at(directory, temporary.as_os_str(), false);
    let mut file: std::fs::File =
        crate::fs_beneath::create_file_at(directory, temporary.as_os_str(), 0o644).map_err(
            |error| {
                return PinRefusal::Unreadable {
                    path: String::from("the pin's temporary name"),
                    reason: error.to_string(),
                };
            },
        )?;
    file.write_all(pin.encode().as_bytes()).map_err(|error| {
        return PinRefusal::Unreadable {
            path: String::from("the pin's temporary name"),
            reason: error.to_string(),
        };
    })?;
    file.sync_all().map_err(|error| {
        return PinRefusal::Unreadable {
            path: String::from("the pin's temporary name"),
            reason: error.to_string(),
        };
    })?;
    drop(file);
    crate::fs_beneath::rename_at(
        directory,
        temporary.as_os_str(),
        directory,
        final_name.as_os_str(),
        true,
    )
    .map_err(|error| {
        return PinRefusal::Unreadable {
            path: String::from("the pin"),
            reason: error.to_string(),
        };
    })?;
    return directory.sync_all().map_err(|error| {
        return PinRefusal::Unreadable {
            path: String::from("the pin's folder"),
            reason: error.to_string(),
        };
    });
}

/// What:
///  The home of one account,
///  read from the passwd database rather than from this process's environment.
/// Why:
///  The wrapper runs as the account,
///  and the design pins the lookup to the UID's passwd home so a variable cannot
///  steer the staging folder
///  ("Proposals the owner settled").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const home = passwdHomeOf(uid);
/// ```
pub fn passwd_home_of(uid: u32) -> Result<PathBuf, PinRefusal> {
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut buffer: Vec<libc::c_char> = vec![0; PASSWD_BUFFER_BYTES];
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    // SAFETY: the entry and the buffer are this call's own and outlive it, and `found` points into the buffer.
    let status: libc::c_int = unsafe {
        libc::getpwuid_r(
            uid,
            &mut entry,
            buffer.as_mut_ptr(),
            buffer.len(),
            &mut found,
        )
    };
    if status != 0 {
        return Err(PinRefusal::Home {
            uid,
            said: std::io::Error::from_raw_os_error(status).to_string(),
        });
    }
    if found.is_null() || entry.pw_dir.is_null() {
        return Err(PinRefusal::Home {
            uid,
            said: String::from("the account has no passwd entry with a home"),
        });
    }
    // SAFETY: `pw_dir` points into `buffer`, which is alive here and is not freed by this read.
    let text: &std::ffi::CStr = unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) };
    let path: PathBuf = PathBuf::from(text.to_string_lossy().into_owned());
    return Ok(path);
}

/// What:
///  The account's `.cache` folder,
///  held by a descriptor that crossed the one mount into it.
/// Why:
///  Every staging name is read and written relative to this descriptor,
///  and the one crossing is the deliberate hop the design names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cache = openTheCache(home);
/// ```
pub fn open_cache(home: &Path) -> Result<std::fs::File, PinRefusal> {
    let root: std::fs::File = open_root(home).map_err(|error| {
        return PinRefusal::Unreadable {
            path: home.display().to_string(),
            reason: error.to_string(),
        };
    })?;
    return open_directory_crossing_one_mount(&root, std::ffi::OsStr::new(".cache")).map_err(
        |error| {
            return PinRefusal::Unreadable {
                path: format!("{}/.cache", home.display()),
                reason: error.to_string(),
            };
        },
    );
}

/// What:
///  Check the cache's own mount against the pin,
///  then check that the mount's root is the folder the descriptor holds.
/// Why:
///  The first check is the ordering check,
///  and the second rejects a mount at `~/.cache` whose root is not the
///  dataset's root,
///  such as a bind of a subdirectory of the same dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkUserCacheMount(cache, pin, facts);
/// ```
pub fn check_pin(
    cache: &std::fs::File,
    pin: &UserCachePin,
    facts: &MountFacts,
) -> Result<(), PinRefusal> {
    if facts.source != pin.source {
        return Err(PinRefusal::Source {
            pinned: pin.source.clone(),
            found: facts.source.clone(),
        });
    }
    let inode: u64 = cache
        .metadata()
        .map_err(|error| {
            return PinRefusal::Unreadable {
                path: String::from("the account's `.cache`"),
                reason: error.to_string(),
            };
        })?
        .ino();
    if inode != pin.root_inode {
        return Err(PinRefusal::Inode {
            pinned: pin.root_inode,
            found: inode,
        });
    }
    return Ok(());
}

/// What:
///  The mount facts of the mount a held cache descriptor is on.
/// Why:
///  The check compares the mount the descriptor is on,
///  never a path resolved again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = cacheMountFacts(cache);
/// ```
pub fn cache_mount_facts(cache: &std::fs::File) -> Result<MountFacts, PinRefusal> {
    let mount_id: u64 = mount_id_of(cache).map_err(|error| {
        return PinRefusal::Unreadable {
            path: String::from("the account's `.cache`"),
            reason: error.to_string(),
        };
    })?;
    return mount_facts_of(mount_id).map_err(|refusal| {
        return PinRefusal::Unreadable {
            path: String::from(crate::fs_mount_options::MOUNTINFO_PATH),
            reason: refusal.message(),
        };
    });
}

/// What:
///  The live check both the wrapper and the handler run:
///  the cache's mount against the pin and its root inode.
/// Why:
///  One body keeps the two sides from comparing different fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkTheLivePin(cache, pin);
/// ```
pub fn check_live_pin(
    cache: &std::fs::File,
    pin: &UserCachePin,
) -> Result<(), PinRefusal> {
    let facts: MountFacts = cache_mount_facts(cache)?;
    return check_pin(cache, pin, &facts);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "user_cache_pin_tests.rs"]
mod tests;
