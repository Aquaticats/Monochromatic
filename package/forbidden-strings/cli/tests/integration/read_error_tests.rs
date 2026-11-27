//! What:
//!  The read-error half of the binary boundary:
//!  a file the scanner cannot
//!  open must surface as a finding on stderr and a non-zero exit,
//!  never as a silent pass.
//! Why:
//!  A secret-scanning gate that exits clean on a file it could not inspect
//!  reports a false-clean result;
//!  CI would merge on it.
//!  Unix expresses an unreadable file with mode bits,
//!  Windows with an exclusive sharing mode,
//!  so each target keeps the fixture its own permission model can build.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // read-error.test.ts: chmod 000 on POSIX, an exclusive open on Windows
//! ```

use std::fs;

use crate::{
    BIN,
    Command,
    unique_tmp,
};

// What:     `#[test] fn read_error_surfaces_as_hit_and_nonzero_exit()`.
//           BUG 4 regression test. Creates an unreadable file (chmod
//           000) containing a deny-listed literal. Pre-fix, the scanner
//           silently substituted empty content and exited 0. Post-fix
//           the scanner emits a `<path>: read error: ...` line on
//           stderr and exits 1.
// Why:      Secret-scanning CI must NOT pass on files it could not
//           inspect; a permission error is a signal, not a silent
//           success.
//
// In TS you'd write (pseudocode):
// ```ts
// test("read error surfaces as hit and non-zero exit", () => { ... });
// ```
//
// What:     `#[cfg(unix)]` compiles this test only for Unix targets.
// Why:      Mode bits are a Unix permission model; Windows has no
//           `PermissionsExt`, and its equivalent unreadable file is the
//           sharing-violation case in `windows_locked_file_read_error_surfaces_as_hit`.
#[test]
#[cfg(unix)]
fn read_error_surfaces_as_hit_and_nonzero_exit() {
    let dir = unique_tmp("bug4");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("unreadable.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");
    // What:     `fs::set_permissions(&target, fs::Permissions::from_mode(0))`
    //           chmod 000 the target. Use the Unix-specific extension
    //           trait to set bare-bits permissions; the Windows
    //           equivalent is a separate test because Windows has no
    //           mode bits to clear.
    // Why:      Force an io::ErrorKind::PermissionDenied at read time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.chmodSync(target, 0);
    // ```
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&target, fs::Permissions::from_mode(0o000)).expect("chmod 000");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    // What:     Restore permissions so cleanup can remove the file.
    // Why:      Tempdir cleanup on a chmod-000 file would itself fail
    //           with EACCES, leaving the fixture behind for the next
    //           run.
    let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o644));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "BUG 4: unreadable file must produce non-zero exit; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    assert!(
        stderr.contains("read error"),
        "BUG 4: stderr must contain `read error`; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     `#[test] #[cfg(windows)] fn windows_locked_file_read_error_surfaces_as_hit()`.
//           The Windows equivalent of the BUG 4 regression test. This
//           test process opens the target with share mode 0, which
//           tells Windows that no other open of the file may proceed
//           while this handle lives. The spawned scanner's open then
//           fails with a sharing violation, a real Windows read error.
// Why:      Secret-scanning CI must not pass on a file it could not
//           open on Windows either; mode bits cannot express that there.
//
// In TS you'd write (pseudocode):
// ```ts
// test("locked file read error surfaces as hit", () => { ... });
// ```
#[test]
#[cfg(windows)]
fn windows_locked_file_read_error_surfaces_as_hit() {
    // What:     `use std::os::windows::fs::OpenOptionsExt;` brings the
    //           Windows-only `share_mode` builder method into scope.
    //           Rust adds platform methods to a shared type through such
    //           extension traits, which must be imported to be callable.
    // Why:      `share_mode(0)` is how a std file open denies every other
    //           open of the same file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // import { openExclusive } from 'windows-fs';
    // ```
    use std::os::windows::fs::OpenOptionsExt;

    let dir = unique_tmp("bug4-windows-lock");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("locked.txt");
    fs::write(&target, "SECRET_NEEDLE_XYZ_LONG_ENOUGH\n").expect("write target");
    // What:     `let lock: fs::File = fs::OpenOptions::new().read(true).share_mode(0).open(&target)`
    //           opens the target for reading and denies shared read,
    //           write and delete access to every other opener. `&target`
    //           lends the path without giving it away. `.expect(...)`
    //           unwraps the `Result`, stopping the test if the open fails.
    // Why:      The handle must stay alive while the scanner runs, so it
    //           is bound to a name that lives until the explicit `drop`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lock = openExclusive(target);
    // ```
    let lock: fs::File = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&target)
        .expect("open target exclusively");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    // What:     `drop(lock)` closes the exclusive handle now.
    // Why:      Cleanup cannot delete a file that is still open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // lock.close();
    // ```
    drop(lock);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "locked file must produce non-zero exit; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    assert!(
        stderr.contains("locked.txt: read error"),
        "stderr must report the locked file's read error; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}
