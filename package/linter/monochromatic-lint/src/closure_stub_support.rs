//! What:
//!  Compile the stub `cargo` once per test process and describe a scenario for it.
//! Why:
//!  A stub written as a compiled Rust program needs no shell,
//! runs the same on every platform the tests run on,
//!  and is built from a tracked source file,
//! `closure_stub_cargo.rs`.
//!  The tests that use it run in the bounded test container,
//! where `rustc` is on `PATH`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stub: Path = compileStubOnce();
//! function scenario(files: [string, string][]): Scenario; // a directory with a copy of the stub
//! ```

/// Import native paths,
///  the process type that runs the compiler and the once-per-process cell.
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

/// What:
///  Compile `closure_stub_cargo.rs` into a temporary directory and return the program's path.
/// Why:
///  The compile takes under a second and is shared by every test of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stubCargo(): Path;
/// ```
pub(crate) fn stub_cargo() -> &'static Path {
    /// The compiled stub,
    ///  shared by every test of the process.
    static STUB: OnceLock<PathBuf> = OnceLock::new();
    return STUB.get_or_init(compile_stub).as_path();
}

/// What:
///  The one compile that `stub_cargo` caches.
/// Why:
///  A named function is the initializer that `OnceLock` takes without a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compileStub(): Path;
/// ```
fn compile_stub() -> PathBuf {
    let output: PathBuf = crate::test_scratch::scratch_root().join(format!(
        "monochromatic-lint-stub-cargo-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&output).expect("the stub directory is created");
    let program: PathBuf = output.join(if cfg!(windows) { "cargo.exe" } else { "cargo" });
    let source: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/closure_stub_cargo.rs");
    let status = Command::new("rustc")
        .args(["--edition=2024", "--crate-name=closure_stub_cargo"])
        .arg(&source)
        .arg("-o")
        .arg(&program)
        .status()
        .expect("rustc starts");
    assert!(status.success(), "the stub cargo compiles");
    return program;
}

/// What:
///  A directory that holds a copy of the stub `cargo` and the answer files of one scenario.
/// Why:
///  The stub reads its answers from the directory it runs from,
///  so a scenario needs no environment variable and tests of one process cannot disturb each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scenario = { directory: Path; cargo: Path }; // removed when dropped
/// ```
pub(crate) struct Scenario {
    /// The scenario directory.
    pub(crate) directory: PathBuf,
    /// The copy of the stub that tests pass as the `cargo` program.
    pub(crate) cargo: PathBuf,
}

/// A process-wide sequence for unique scenario directory names.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// What:
///  Create a scenario directory containing the stub and the given answer files.
/// Why:
///  The directory name contains a space,
///  so every test also proves that a path with a space reaches the process arguments intact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scenario(files: [string, string][]): Scenario;
/// ```
pub(crate) fn scenario(files: &[(&str, &str)]) -> Scenario {
    let sequence: u64 = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let directory: PathBuf = crate::test_scratch::scratch_root().join(format!(
        "monochromatic-lint scenario {}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("the scenario directory is created");
    let cargo: PathBuf = directory.join(stub_cargo().file_name().expect("the stub has a name"));
    // A hard link, not a copy: writing an executable while a parallel test forks can fail with "text file busy".
    // The program locates its scenario by the path it was started through, which a hard link keeps distinct.
    if std::fs::hard_link(stub_cargo(), &cargo).is_err() {
        std::fs::copy(stub_cargo(), &cargo).expect("the stub is copied");
    }
    for (name, text) in files {
        std::fs::write(directory.join(name), text).expect("an answer file is written");
    }
    return Scenario { directory, cargo };
}

/// Read what the stub logged,
///  or nothing when it was never called.
impl Scenario {
    /// The lines of `calls.log`.
    pub(crate) fn calls(&self) -> Vec<String> {
        return match std::fs::read_to_string(self.directory.join("calls.log")) {
            Ok(text) => text.lines().map(String::from).collect::<Vec<String>>(),
            Err(_) => Vec::<String>::new(),
        };
    }
}

/// Remove the scenario directory,
///  including after a failed assertion.
impl Drop for Scenario {
    /// Delete only the directory this scenario created.
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(&self.directory);
    }
}
