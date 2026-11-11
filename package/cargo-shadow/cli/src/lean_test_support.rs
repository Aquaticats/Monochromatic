//! Fixtures for the lean-folder tests:
//! disposable Cargo profile directories in both layouts,
//! with the compiler of each unit and its times set.
//! Why:
//! Every test of a removal needs a build folder that nothing real depends on.

/// Files,
/// times,
/// and the pipe a suite of controls needs.
use std::{fs, fs::File, path::{Path, PathBuf}, sync::mpsc, time::{Duration, UNIX_EPOCH}};

/// The pipe a name is given.
use rustix::fs::{CWD, FileType, Mode, mknodat};

/// The lock file name of a profile directory.
pub const BUILD_LOCK: &str = ".cargo-build-lock";

/// One hour in seconds.
pub const HOUR: i64 = 3_600;

/// One day in seconds.
pub const DAY: i64 = 86_400;

/// A moment well after the epoch so ages are positive.
pub const NOW: i64 = 1_800_000_000;

/// The longest a call that must answer at once may take in these tests:
/// a call that waits for the other end of a pipe never answers.
/// Ten seconds is longer than any answer on this host and short enough that a control that hangs fails the suite
/// rather than holding it.
pub const ANSWER_WITHIN: Duration = Duration::from_secs(10);

/// What:
/// `pub fn within<F, T>(work: F) -> T` is the work's answer,
/// and a failure when the work has not answered inside `ANSWER_WITHIN`.
/// Why:
/// The defect these controls guard is a call that never returns,
/// so the control cannot wait for it in the test's own thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function within<T>(work: () => T, limitMs: number): Promise<T>
/// ```
pub fn within<F, T>(work: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    return receiver.recv_timeout(ANSWER_WITHIN).expect("the call answered instead of waiting for the other end of a pipe");
}

/// What:
/// `pub fn make_pipe(path: &Path)` makes a named pipe with no reader and no writer.
/// Why:
/// A pipe at one of the names a pass opens is the shape that holds a blocking open forever,
/// and only a pipe tests what a symbolic link or another file cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const makePipe = (path: string): void => execFileSync('mkfifo', [path]);
/// ```
pub fn make_pipe(path: &Path) {
    mknodat(CWD, path, FileType::Fifo, Mode::from_raw_mode(0o600), 0).expect("a pipe at the name");
}

/// What:
/// `pub fn set_time(path: &Path, seconds: i64)` sets the modification time of a file or directory.
/// Why:
/// Age rules read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const setTime = (path: string, seconds: number): void => utimesSync(path, seconds, seconds);
/// ```
pub fn set_time(path: &Path, seconds: i64) {
    let file: File = File::open(path).expect("open to set the time");
    file.set_modified(UNIX_EPOCH + Duration::from_secs(u64::try_from(seconds).expect("positive time"))).expect("set the time");
}

/// What:
/// `pub fn make_profile(root: &Path, name: &str) -> PathBuf` makes an empty profile directory with its build lock.
/// Why:
/// The lock file marks a directory Cargo has built in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function makeProfile(root: string, name: string): string
/// ```
pub fn make_profile(root: &Path, name: &str) -> PathBuf {
    let profile: PathBuf = root.join(name);
    fs::create_dir_all(profile.join(".fingerprint")).expect("fingerprint directory");
    fs::create_dir_all(profile.join("deps")).expect("deps directory");
    fs::create_dir_all(profile.join("build")).expect("build directory");
    fs::create_dir_all(profile.join("incremental")).expect("incremental directory");
    fs::write(profile.join(BUILD_LOCK), b"").expect("lock file");
    return profile;
}

/// What:
/// `pub fn add_old_unit(profile: &Path, package: &str, hash: &str, compiler: u64, invoked: i64) -> Vec<PathBuf>` writes a unit in the old layout.
/// Why:
/// It has a fingerprint directory,
/// a `deps/` artifact and a `build/` directory,
/// all with one hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addOldUnit(profile: string, pkg: string, hash: string, compiler: bigint, invoked: number): string[]
/// ```
pub fn add_old_unit(profile: &Path, package: &str, hash: &str, compiler: u64, invoked: i64) -> Vec<PathBuf> {
    let fingerprint: PathBuf = profile.join(".fingerprint").join(format!("{package}-{hash}"));
    fs::create_dir_all(&fingerprint).expect("unit fingerprint");
    fs::write(fingerprint.join(format!("lib-{package}.json")), format!("{{\"rustc\":{compiler},\"features\":\"[]\"}}")).expect("fingerprint json");
    fs::write(fingerprint.join("invoked.timestamp"), b"").expect("invoked");
    set_time(&fingerprint.join("invoked.timestamp"), invoked);
    let artifact: PathBuf = profile.join("deps").join(format!("lib{package}-{hash}.rlib"));
    fs::write(&artifact, vec![7_u8; 4096]).expect("artifact");
    let build: PathBuf = profile.join("build").join(format!("{package}-{hash}"));
    fs::create_dir_all(&build).expect("build unit");
    fs::write(build.join("output"), b"x").expect("build output");
    return vec![fingerprint, build, artifact];
}

/// What:
/// `pub fn add_new_unit(profile: &Path, package: &str, hash: &str, compiler: u64, invoked: i64) -> PathBuf` writes a unit in the new layout.
/// Why:
/// `build/<package>/<hash>/` is the whole unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addNewUnit(profile: string, pkg: string, hash: string, compiler: bigint, invoked: number): string
/// ```
pub fn add_new_unit(profile: &Path, package: &str, hash: &str, compiler: u64, invoked: i64) -> PathBuf {
    let unit: PathBuf = profile.join("build").join(package).join(hash);
    fs::create_dir_all(unit.join("fingerprint")).expect("new unit fingerprint");
    fs::create_dir_all(unit.join("out")).expect("new unit out");
    fs::write(unit.join("fingerprint").join(format!("lib-{package}.json")), format!("{{\"rustc\":{compiler}}}")).expect("json");
    fs::write(unit.join("fingerprint").join("invoked.timestamp"), b"").expect("invoked");
    set_time(&unit.join("fingerprint").join("invoked.timestamp"), invoked);
    fs::write(unit.join("out").join("artifact"), vec![9_u8; 4096]).expect("artifact");
    return unit;
}

/// What:
/// `pub fn add_incremental(profile: &Path, name: &str, modified: i64) -> PathBuf` writes an incremental directory.
/// Why:
/// The sweep removes those last written before the newest compiler first built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addIncremental(profile: string, name: string, modified: number): string
/// ```
pub fn add_incremental(profile: &Path, name: &str, modified: i64) -> PathBuf {
    let directory: PathBuf = profile.join("incremental").join(name);
    fs::create_dir_all(&directory).expect("incremental directory");
    fs::write(directory.join("dep-graph.bin"), vec![1_u8; 4096]).expect("incremental file");
    set_time(&directory, modified);
    return directory;
}

/// A guard environment on the real `/proc` for a test of an idle fixture.
pub fn real_environment() -> crate::lean_guard::GuardEnvironment {
    return crate::lean_guard::GuardEnvironment::real();
}

/// What:
/// `pub fn age_everything(path: &Path, seconds: i64)` sets the modification time of a tree and everything in it.
/// Why:
/// Idleness is read from file times,
/// and a fixture is made old in one call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ageEverything(path: string, seconds: number): void
/// ```
pub fn age_everything(path: &Path, seconds: i64) {
    if fs::symlink_metadata(path).expect("metadata").is_dir() {
        for entry in fs::read_dir(path).expect("read the directory") {
            age_everything(&entry.expect("entry").path(), seconds);
        }
    }
    set_time(path, seconds);
}

/// What:
/// `pub fn make_tree(base: &Path, relative: &str, ledger: Option<&Path>) -> PathBuf` makes a build tree with one profile
/// directory holding one unit,
/// and a ledger naming the workspace when one is given.
/// Why:
/// The idle-tree tests start from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function makeTree(base: string, relative: string, ledger?: string): string
/// ```
pub fn make_tree(base: &Path, relative: &str, ledger: Option<&Path>) -> PathBuf {
    let tree: PathBuf = base.join(relative);
    let profile: PathBuf = make_profile(&tree, "debug");
    add_old_unit(&profile, "alpha", "00000000000000aa", 111, NOW - 100 * DAY);
    if let Some(root) = ledger {
        fs::write(tree.join(".cargo-shadow-workspace"), format!("{}\n", root.display())).expect("ledger");
    }
    return tree;
}
