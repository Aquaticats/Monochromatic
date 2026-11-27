//! What:
//!  Whether the commit Git builds in the scratch repository is signed:
//!  the configuration says so or the command line asks for it.
//! Why:
//!  The line a person reads while that commit runs names the signing program while one runs below it,
//!  so this question is asked once,
//!  of the configuration the commit is built under and of the person's own words,
//!  and never of what Git does with them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const signs = (snapshot, arguments) => configSigns(snapshot) || arguments.some(asksForGpgSign);
//! ```

use crate::config_snapshot::ConfigSnapshot;
use std::ffi::OsString;

/// Whether the commit is signed:
/// the configuration says so or the command line asks for it.
pub fn signs_commit(
    snapshot: Option<&ConfigSnapshot>,
    commit_arguments: &[OsString],
) -> bool {
    if crate::service_prepare::config_signs(snapshot) {
        return true;
    }
    let mut asked: bool = false;
    for argument in commit_arguments {
        let bytes: &[u8] = crate::path_bytes::argument_bytes(argument.as_os_str());
        if bytes == b"--" {
            break;
        }
        if bytes == b"--no-gpg-sign" {
            asked = false;
        } else if bytes.starts_with(b"--gpg-sign")
            || (bytes.starts_with(b"-S") && !bytes.starts_with(b"--"))
        {
            asked = true;
        }
    }
    return asked;
}
