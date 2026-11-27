//! What:
//!  The `user-cache` pin's record:
//!  its two compared fields,
//!  the device `doctor` prints,
//!  the root-owned file it is recorded in,
//!  its bounded read,
//!  its atomic write,
//!  and the words a person checks a refusal by.
//! Why:
//!  The pin says which dataset must stand at the account's `.cache` and which
//!  mount root it must have,
//!  so a bind made before that dataset mounts cannot land on the home underlay
//!  (`doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "Decision").
//!  The record sits apart from the account's lookup and the mount checks,
//!  so a reader of the file's words does not read the mount calls as well.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pin = readUserCachePin('/var/lib/…/user-cache-pin');
//! ```

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
