//! What:
//! Unit tests of the PATH lookup in `real_binary.rs`.
//! Why:
//! The lookup decides which program actually runs,
//! so a wrong answer would either loop the shadow into itself
//!           or run the wrong program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.unit.test.ts
//! describe('findInPath', () => { it('finds the first executable outside the skipped path', () => { ... }); });
//! ```

/// Imports the function under test from this crate.
use crate::real_binary::find_in_path;

/// Imports the scratch-directory helper (see `test_scratch.rs`).
use crate::test_scratch::scratch;

/// Imports the file-system module (writes fixture scripts,
/// sets permissions,
/// makes symlinks).
use std::fs;

/// Imports the trait that adds `from_mode` to permissions (marks a fixture executable).
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// What:
/// `fn write_tool(directory: &Path, name: &str, mode: u32)` declares a helper writing a one-line script named
///           `name` in `directory` with Unix permission bits `mode` (`u32`:
/// unsigned 32-bit integer,
/// siblings `i32`,
///           `u64`).
/// Why:
/// Each test needs runnable and non-runnable fixture files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeTool(directory: string, name: string, mode: number): void
/// ```
fn write_tool(directory: &Path, name: &str, mode: u32) {
    fs::write(directory.join(name), "#!/bin/sh\n").expect("write the fixture tool");
    fs::set_permissions(directory.join(name), fs::Permissions::from_mode(mode)).expect("set the fixture mode");
}

/// The first executable in PATH order is returned.
#[test]
fn finds_the_first_executable_in_path_order() {
    let first = scratch();
    let second = scratch();
    write_tool(first.path(), "mpm", 0o755);
    write_tool(second.path(), "mpm", 0o755);
    let path_variable: String = format!("{}:{}", first.path().display(), second.path().display());

    // `Some(first.path().join("mpm"))` is the present case of `Option` wrapping the expected path.
    assert_eq!(find_in_path("mpm", &path_variable, Path::new("/nowhere")), Some(first.path().join("mpm")));
}

/// A file without an execute bit is not a candidate.
#[test]
fn skips_files_that_are_not_executable() {
    let first = scratch();
    let second = scratch();
    write_tool(first.path(), "mpm", 0o644);
    write_tool(second.path(), "mpm", 0o755);
    let path_variable: String = format!("{}:{}", first.path().display(), second.path().display());

    assert_eq!(find_in_path("mpm", &path_variable, Path::new("/nowhere")), Some(second.path().join("mpm")));
}

/// Any candidate that resolves to the shadow itself is skipped,
/// however many links lead to it.
#[test]
fn skips_every_link_that_resolves_to_the_shadow() {
    let shadow_home = scratch();
    let first_links = scratch();
    let second_links = scratch();
    let real = scratch();
    write_tool(shadow_home.path(), "mpm-shadow", 0o755);
    write_tool(real.path(), "mpm", 0o755);
    std::os::unix::fs::symlink(shadow_home.path().join("mpm-shadow"), first_links.path().join("mpm")).expect("link");
    std::os::unix::fs::symlink(shadow_home.path().join("mpm-shadow"), second_links.path().join("mpm")).expect("link");
    let skip: PathBuf = fs::canonicalize(shadow_home.path().join("mpm-shadow")).expect("resolve the shadow");
    let path_variable: String =
        format!("{}:{}:{}", first_links.path().display(), second_links.path().display(), real.path().display());

    assert_eq!(find_in_path("mpm", &path_variable, &skip), Some(real.path().join("mpm")));
}

/// Nothing runnable on PATH gives `None`;
/// empty PATH entries are ignored.
#[test]
fn returns_none_when_nothing_is_found() {
    let empty = scratch();
    let path_variable: String = format!(":{}", empty.path().display());
    assert_eq!(find_in_path("mpm", &path_variable, Path::new("/nowhere")), None);
}

/// A launcher the way `install` makes one:
/// an executable,
/// a `.launch` record beside it naming the binary it starts,
/// and an `mpm` link to it in `links`.
fn write_launcher(directory: &Path, record_real: &Path, write_record_file: bool, record_text: Option<&str>) -> PathBuf {
    use crate::toolchain::launcher::{LaunchRecord, record_path, render_record};
    use crate::toolchain::scope::ScopeKind;
    let launchers: PathBuf = directory.join("launchers");
    fs::create_dir_all(&launchers).expect("make the launcher directory");
    write_tool(&launchers, "mpm-shadow", 0o755);
    let launcher: PathBuf = launchers.join("mpm-shadow");
    if write_record_file {
        let record = LaunchRecord {
            tool_id: "mpm-shadow".to_string(),
            lock_file: directory.join("toolchain.lock.json"),
            real: record_real.to_path_buf(),
            scope: ScopeKind::Repo,
        };
        let text: String = record_text.map(str::to_string).unwrap_or_else(|| return render_record(&record));
        fs::write(record_path(&launcher), text).expect("write the record");
    }
    let links: PathBuf = directory.join("links");
    fs::create_dir_all(&links).expect("make the link directory");
    std::os::unix::fs::symlink(&launcher, links.join("mpm")).expect("link the launcher as mpm");
    return links;
}

/// The running shadow,
/// a real `mpm`,
/// and a PATH with the launcher's link before the real `mpm`.
struct LauncherFixture {
    _directory: tempfile::TempDir,
    path_variable: String,
    running: PathBuf,
    real_mpm: PathBuf,
    links: PathBuf,
}

fn launcher_fixture(record_names_the_running_binary: bool, write_record_file: bool, record_text: Option<&str>) -> LauncherFixture {
    let directory = scratch();
    let running_dir: PathBuf = directory.path().join("running");
    fs::create_dir_all(&running_dir).expect("make the running directory");
    write_tool(&running_dir, "mpm-shadow", 0o755);
    let running: PathBuf = fs::canonicalize(running_dir.join("mpm-shadow")).expect("resolve the running binary");
    let other_dir: PathBuf = directory.path().join("other");
    fs::create_dir_all(&other_dir).expect("make the other directory");
    write_tool(&other_dir, "mpm-shadow", 0o755);
    let named: PathBuf = if record_names_the_running_binary { running.clone() } else { other_dir.join("mpm-shadow") };
    let links: PathBuf = write_launcher(directory.path(), &named, write_record_file, record_text);
    let real_dir: PathBuf = directory.path().join("real");
    fs::create_dir_all(&real_dir).expect("make the real mpm directory");
    write_tool(&real_dir, "mpm", 0o755);
    let path_variable: String = format!("{}:{}", links.display(), real_dir.display());
    return LauncherFixture { path_variable, running, real_mpm: real_dir.join("mpm"), links, _directory: directory };
}

#[test]
fn skips_the_launcher_of_the_running_binary_because_it_starts_this_shadow_again() {
    let fixture = launcher_fixture(true, true, None);
    assert_eq!(find_in_path("mpm", &fixture.path_variable, &fixture.running), Some(fixture.real_mpm.clone()));
}

#[test]
fn a_launcher_of_another_binary_is_not_skipped() {
    let fixture = launcher_fixture(false, true, None);
    assert_eq!(find_in_path("mpm", &fixture.path_variable, &fixture.running), Some(fixture.links.join("mpm")));
}

#[test]
fn an_executable_with_no_record_beside_it_is_not_a_launcher_and_is_not_skipped() {
    let fixture = launcher_fixture(true, false, None);
    assert_eq!(find_in_path("mpm", &fixture.path_variable, &fixture.running), Some(fixture.links.join("mpm")));
}

#[test]
fn an_unreadable_record_does_not_make_an_executable_a_launcher() {
    let fixture = launcher_fixture(true, true, Some("not json"));
    assert_eq!(find_in_path("mpm", &fixture.path_variable, &fixture.running), Some(fixture.links.join("mpm")));
}
