//! What:
//!  The account's side of the pin:
//!  the passwd record its home is read from,
//!  the home path itself,
//!  and the `.cache` descriptor every staging name is read and written under.
//! Why:
//!  The wrapper runs as the account,
//!  and the design pins the lookup to the UID's passwd home so a variable
//!  cannot steer the staging folder
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The staging target and who creates it").
//!  The one deliberate crossing into `.cache` is here,
//!  because every later read is relative to the descriptor this opens.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const home = passwdHomeOf(uid);
//! const cache = openTheCache(home);
//! ```

use super::user_cache_pin_record::PinRefusal;
use crate::fs_beneath::{
    open_directory_crossing_one_mount,
    open_root,
};
use std::path::{
    Path,
    PathBuf,
};

/// The buffer `getpwuid_r` fills,
/// large enough for one passwd entry on every account this route serves.
const PASSWD_BUFFER_BYTES: usize = 16_384;

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
