//! What:
//!  The one writer of executable fixture files, shared by the unit tests and the binary-level controls.
//! Why:
//!  Both test crates need the same race-free write, and two copies could drift apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync('tee', [path], { input: content }); chmodSync(path, 0o755);
//! ```

/// Native process types used by the writer.
use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

/// What:
///  Write an executable file,
///  as a PATH candidate fixture,
///  through a child `tee` process.
/// Why:
///   Tests and controls run on several threads.
///  If this process opened the file for writing,
///  a
///       child forked by another test at that moment would inherit the open file until it
///       starts its own program,
///  and running the fixture in that window fails with
///       "Text file busy".
///  A file only ever opened for writing by `tee` cannot be
///       inherited by any other child of this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('tee', [path], { input: content }); chmodSync(path, 0o755);
/// ```
pub fn executable(path: &Path, content: &[u8]) {
    use std::os::unix::fs::PermissionsExt;
    // `Stdio::piped()` connects this process to `tee`'s input; its copy to standard output is discarded.
    let mut writer: Child = Command::new("tee")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("start fixture writer");
    // `.take()` moves the pipe out of the child record so it can be closed below.
    let mut input: ChildStdin = writer.stdin.take().expect("fixture writer input");
    input.write_all(content).expect("write candidate");
    // `drop` closes the pipe now, which is how `tee` learns the content is complete.
    drop(input);
    assert!(
        writer.wait().expect("fixture writer exit").success(),
        "fixture writer failed for {path:?}"
    );
    // Changing the mode names the file by path and never opens it.
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("mark candidate executable");
}
