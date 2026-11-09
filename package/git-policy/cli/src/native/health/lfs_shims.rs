//! What:
//!  The Git LFS shims a repository may hold:
//! which files are exactly what git-lfs writes for its own name,
//!  and the explanation the report gives for them.
//! Why:
//!  A shim is expected and harmless,
//! because the wrapper does the LFS upload itself and starts Git with traditional hooks off,
//!  so it is explained apart from the unsupported hooks and leaves the exit status at 0.
//!  Recognition is by name and by the exact bytes of git-lfs 3.8.0,
//! so an edited,
//! misnamed or linked file stays an
//!  ordinary unsupported hook.
//!  Every call here reads the file system,
//! so each read is bounded (`work_awaited_within_a_ceiling.rs`):
//! a mount that
//!  stopped answering must not hold the report,
//! and a shim that could not be read is an error rather than a verdict,
//!  because silence would read as "no hooks here".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shims = found.filter(isLfsShim);      // byte for byte, by name
//! stderr.write(renderLfsShims(shims));        // '' when there are none
//! ```

/// `ErrorKind` tells a read that gave no answer from one that failed.
use std::io::ErrorKind;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// The ceiling of each file system read of the report.
use crate::wait_ceilings::REPORT_SCAN_CEILING;
/// The job of such a read,
/// run on a thread of its own.
use crate::work_awaited_within_a_ceiling::work_within;

/// The four hooks `git lfs install` writes.
const LFS_SHIM_NAMES: [&str; 4] = ["pre-push", "post-checkout", "post-commit", "post-merge"];

/// The longest shim file the comparison reads,
/// in bytes;
/// a longer file is not a shim.
const LFS_SHIM_LIMIT: u64 = 1024;

/// The text git-lfs 3.8.0 writes (`lfs/hook.go`,
/// `hookBaseContent` and its `write`,
/// which appends a line feed),
///  with the hook's own name where `{{Command}}` stands.
const LFS_SHIM_TEMPLATE: &str = r##"#!/bin/sh
command -v git-lfs >/dev/null 2>&1 || { printf >&2 "\n%s\n\n" "This repository is configured for Git LFS but 'git-lfs' was not found on your path. If you no longer wish to use Git LFS, remove this hook by deleting the '{{Command}}' file in the hooks directory (set by 'core.hookspath'; usually '.git/hooks')."; exit 2; }
git lfs {{Command}} "$@"
"##;

/// What:
///  The text git-lfs writes for the hook called `name`.
/// Why:
///   Recognition is by these exact bytes,
/// so an edited or foreign file is never taken for a shim.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// template.replaceAll('{{Command}}', name)
/// ```
pub fn lfs_shim_text(name: &str) -> String {
    return LFS_SHIM_TEMPLATE.replace("{{Command}}", name);
}

/// What:
///  Whether a hook file is exactly the git-lfs shim for its own name.
/// Why:
///   Only a regular file whose name is one of the four hooks and whose bytes equal that hook's shim counts;
/// a link,
///  a file that differs by one byte,
/// and a shim under another hook's name are ordinary unsupported hooks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isFile(path) && readFileSync(path).equals(lfsShimText(basename(path)))
/// ```
pub fn is_lfs_shim(path: &Path) -> std::io::Result<bool> {
    use std::io::Read;
    let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
        return Ok(false);
    };
    if !LFS_SHIM_NAMES.contains(&name) {
        return Ok(false);
    }
    let named: PathBuf = path.to_path_buf();
    let wanted: String = name.to_string();
    // The type check, the open and the read of the file all wait on the file system, so they run under the ceiling.
    let compared: Option<std::io::Result<bool>> = work_within(REPORT_SCAN_CEILING, move || {
        let Ok(metadata) = std::fs::symlink_metadata(named.as_path()) else {
            return Ok(false);
        };
        if !metadata.file_type().is_file() {
            return Ok(false);
        }
        let Ok(file) = std::fs::File::open(named.as_path()) else {
            return Ok(false);
        };
        let mut content: Vec<u8> = Vec::new();
        if file.take(LFS_SHIM_LIMIT).read_to_end(&mut content).is_err() {
            return Ok(false);
        }
        return Ok(content == lfs_shim_text(wanted.as_str()).as_bytes());
    });
    return match compared {
        Some(answer) => answer,
        None => Err(std::io::Error::new(
            ErrorKind::TimedOut,
            format!("the read of {} gave no answer within {REPORT_SCAN_CEILING:?}", path.display()),
        )),
    };
}

/// What:
///  The explanation of the Git LFS shims a repository holds.
/// Why:
///   They are listed apart from the unsupported hooks and say what is going on,
/// because the person who finds them
///  after deleting them needs to know they come back,
/// that this is normal,
/// and that nothing needs doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explanation = (shims: string[]) => '...';
/// ```
pub fn render_lfs_shims(shims: &[PathBuf]) -> String {
    let mut text: String = String::from(
        "git monochromatic-git-policy-cli doctor: the hook files below are the Git LFS shims, byte for byte, and nothing needs doing:\n",
    );
    for shim in shims {
        text.push_str("  ");
        text.push_str(shim.display().to_string().as_str());
        text.push('\n');
    }
    text.push_str(
        "git-lfs writes these files again whenever it cleans a file in a Git that monochromatic-git-policy-cli did not start (for example \
`git add` of an LFS file run as `/usr/bin/git` directly), so they can come back after you delete them; this is \
normal. monochromatic-git-policy-cli uploads the LFS objects itself before every push and starts Git with traditional hooks off, so these \
files never run through monochromatic-git-policy-cli. You may delete them or leave them. Only a script that starts Git without monochromatic-git-policy-cli, \
such as `/usr/bin/git` directly, would run them.\n",
    );
    return text;
}
