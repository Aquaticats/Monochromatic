//! What:
//! The two controls the `rust` row's `bins` needs:
//! an install of this repository's own `rust` row ends with no unresolved bin,
//! and a bin no route provides is still named.
//! Why:
//! The row's `bins` is what the exposure farm links and what the report's
//! `unresolved bins` line is made of,
//! and this file reads that list from the repository's own `toolchain.lock.json`,
//! so a program the `rustup-toolchain` route cannot put in the farm fails here.
//! Each install runs against a disposable home whose toolchain holds the programs a
//! toolchain of this machine holds,
//! and the fake runner answers every `rustup` call,
//! so no real toolchain and no manager on this machine is read or started.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the-repository-rust-row-names-no-unresolved-bin.test.ts
//! ```

/// Imports `fs` from `std`,
/// the reader of the repository's lock file.
use std::fs;
/// Imports the borrowed path type and the owned one,
/// because the repository root is borrowed from the manifest path and the lock path is built.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the rig the fakes and a disposable scope are built into,
/// and the write of the stand-in programs a toolchain holds.
use super::fakes::{
    Rig,
    write_executable,
};
/// Imports the reader of the lock whose row is under test.
use super::lock::parse_lock;
/// Imports the locked tool the row is returned as.
use super::model::LockedTool;
/// Imports the lock file's name,
/// so the test names the file the scope names.
use super::scope::LOCK_FILE_NAME;
/// Imports the install of the tools a lock holds.
use super::sync::sync_tools;
/// Imports the text an install prints,
/// whose last lines the controls read.
use super::sync_report_text::format_sync_report;

/// What:
///     `const CRATE_DIRECTORIES_BELOW_THE_ROOT: usize = 3` is how many directory levels
///     this crate's folder sits below the repository root
///     (`package/mpm-shadow/cli`).
///     The siblings a reader might expect are `u32` and `u64` and `i32` and `i64`.
/// Why:
///     `Path::ancestors` counts a number of levels,
///     so `usize` is the width its `nth` takes and no cast is needed at the call
///     (not `u32` or `u64`, which would need one).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const crateDirectoriesBelowTheRoot = 3;
/// ```
const CRATE_DIRECTORIES_BELOW_THE_ROOT: usize = 3;

/// What:
///     `const THE_FIXTURE_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu"` is the second half
///     of the directory name `rustup` gives an installed channel
///     (`nightly-2026-11-16-x86_64-unknown-linux-gnu`).
///     Its sibling `&String` would be an owned buffer this constant does not need.
/// Why:
///     The farm reaches a toolchain's programs through
///     `~/.rustup/toolchains/<channel>-<triple>/bin`, and the fixture needs the same shape,
///     so the name is one `&str` built and used in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theFixtureHostTriple = 'x86_64-unknown-linux-gnu';
/// ```
const THE_FIXTURE_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";

/// What:
///     `const THE_TOOLCHAIN_PROGRAMS: [&str; 10]` is the set of programs the toolchain
///     `nightly-2026-11-16-x86_64-unknown-linux-gnu` holds on this machine:
///     `cargo`, `cargo-clippy`, `cargo-fmt`, `clippy-driver`, `rustc`, `rustdoc`, `rustfmt`,
///     `rust-gdb`, `rust-gdbgui` and `rust-lldb`.
///     `[&str; 10]` is a fixed-size array of borrowed strings; its siblings are
///     `Vec<&str>` (growable, allocated) and `&[&str]` (a borrowed view).
/// Why:
///     The fixture stands for that machine, so the list is that machine's toolchain bin
///     directory and not a shape invented here:
///     it is the output of
///     `ls --almost-all ~/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/bin`,
///     run on 2026-11-27T17:12:19-05:00.
///     It is a fixed-size array because the list is written here and never grows
///     (not `Vec`, which would allocate for nothing).
/// Gotcha:
///     `cargo-fmt` and `rustfmt` are in this list because this machine's two toolchains had
///     the `rustfmt` component added to them on 2026-11-25T21:21 -05:00 (their file times),
///     and no row of the repository's plan declares that component:
///     a machine that follows the plan's `rustComponents` alone has neither program,
///     and an install there names both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theToolchainPrograms = ['cargo', 'cargo-clippy', 'cargo-fmt', 'clippy-driver', 'rustc', 'rustdoc', 'rustfmt', 'rust-gdb', 'rust-gdbgui', 'rust-lldb'];
/// ```
const THE_TOOLCHAIN_PROGRAMS: [&str; 10] = [
    "cargo",
    "cargo-clippy",
    "cargo-fmt",
    "clippy-driver",
    "rust-gdb",
    "rust-gdbgui",
    "rust-lldb",
    "rustc",
    "rustdoc",
    "rustfmt",
];

/// What:
///     `fn repository_root() -> PathBuf` is the repository's own folder, found from the
///     folder this crate was compiled from.
/// Why:
///     The row under test is this repository's declaration,
///     so the control reads the committed lock instead of a copy that could drift from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repositoryRoot(): string
/// ```
fn repository_root() -> PathBuf {
    // What:     `env!("CARGO_MANIFEST_DIR")` is the crate's folder, pasted in at compile
    //           time. `Path::new` borrows it as a path, and `ancestors()` yields that folder
    //           and then each parent, so `.nth(3)` is the repository root.
    // Why:      The lock is three levels above the crate, and the path is derived from a
    //           constant cargo supplies rather than written as a relative path that would
    //           depend on where the test runs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const manifest = import.meta.dirname;
    // const root = dirname(dirname(dirname(manifest)));
    // ```
    let ancestor: Option<&Path> = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(CRATE_DIRECTORIES_BELOW_THE_ROOT);
    // What:     `.expect(...)` unwraps the present variant of `Option` and ends the test
    //           with this sentence when it is `None`.
    // Why:      A layout change that moves the crate should say so here, not fail later
    //           with a missing file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (ancestor === undefined) throw new Error('...');
    // ```
    let root: &Path = ancestor.expect("the crate sits three levels below the repository root");
    return root.to_path_buf();
}

/// What:
///     `fn the_repository_rust_row() -> LockedTool` is the lock's row for this repository's
///     Rust toolchain, decoded from `toolchain.lock.json` at the repository root.
/// Why:
///     This is the exact list `unresolvable_bins` reads after an install,
///     so the control fails when the row declares a program the route cannot provide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theRepositoryRustRow(): LockedTool
/// ```
fn the_repository_rust_row() -> LockedTool {
    // What:     `join(...)` builds the lock's path from the repository root, and the name
    //           is the constant the scope uses.
    // Why:      The file read is the one `install` reads, not a copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lockFile = pathJoin(repositoryRoot(), LOCK_FILE_NAME);
    // ```
    let lock_file: PathBuf = repository_root().join(LOCK_FILE_NAME);
    // What:     `fs::read_to_string` returns `Result<String, io::Error>`, and
    //           `.expect(...)` unwraps its success value.
    // Why:      A missing lock is a broken checkout, and the test says which file it wanted.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = readFileSync(lockFile, 'utf8');
    // ```
    let text: String =
        fs::read_to_string(&lock_file).expect("the repository's toolchain.lock.json is readable");
    // What:     `display()` renders the path for a message, and `.to_string()` copies that
    //           text into an owned `String` the decoder can keep.
    // Why:      A decode error names its file, and the label must outlive the borrow of the
    //           path expression.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const label = lockFile;
    // ```
    let label: String = lock_file.display().to_string();
    // What:     `parse_lock` decodes the lock's text and `.expect(...)` unwraps the
    //           `Result`, so a lock this crate cannot read fails here with the decoder's own
    //           sentence.
    // Why:      The row is only trustworthy if the committed file decodes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parsed = parseLock(text, label);
    // ```
    let parsed = parse_lock(&text, &label).expect("the repository's lock decodes");
    // What:     the loop walks the lock's rows in written order, and `return tool;` hands the
    //           first row whose id is `rust` back to the caller.
    // Why:      The row is found by its id, so a lock that reorders its rows still answers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const tool of parsed.lock.tools) { if (tool.declaration.id === 'rust') return tool; }
    // ```
    for tool in parsed.lock.tools {
        if tool.declaration.id == "rust" {
            return tool;
        }
    }
    // What:     `panic!(...)` ends the test with this sentence.
    // Why:      The repository's plan and lock both hold a `rust` row; a lock without one
    //           means the control no longer has a subject.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // throw new Error(`${label} has no rust row`);
    // ```
    panic!("{label} has no rust row");
}

/// What:
///     `fn rig_of_the_repository_rust_row() -> Rig` is a disposable scope whose lock holds
///     only this repository's `rust` row, in a home whose toolchain holds the programs
///     `THE_TOOLCHAIN_PROGRAMS` names.
/// Why:
///     The install then runs the route against a machine of the shape this one has,
///     without reading or writing this machine's toolchain,
///     rustup home or exposure folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rigOfTheRepositoryRustRow(): Rig
/// ```
fn rig_of_the_repository_rust_row() -> Rig {
    // What:     `Rig::new()` builds the fakes and a scope under the crate's own scratch
    //           directory.
    // Why:      The install needs a scope, a fake runner and a fake network, and nothing in
    //           it may reach the real machine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rig = Rig.new();
    // ```
    let mut rig: Rig = Rig::new();
    // What:     the row is this repository's declaration with its locked version.
    // Why:      The control's subject is the committed row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const row = theRepositoryRustRow();
    // ```
    let row: LockedTool = the_repository_rust_row();
    // What:     `format!(...)` builds the toolchain's directory name from the row's version
    //           and the host triple, and the joins walk from the fixture home to its `bin`
    //           directory.
    // Why:      `bin_search_dirs` looks for a `rustup-toolchain` row's programs in
    //           `<home>/.rustup/toolchains` and below, so the fixture has to sit exactly
    //           there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const toolchainBin = pathJoin(rig.scope.home, '.rustup', 'toolchains', `${row.version}-${theFixtureHostTriple}`, 'bin');
    // ```
    let toolchain_bin: PathBuf = rig
        .scope
        .home
        .join(".rustup")
        .join("toolchains")
        .join(format!("{}-{}", row.version, THE_FIXTURE_HOST_TRIPLE))
        .join("bin");
    // What:     the loop writes one stand-in program per name, each a shell script the
    //           executable check accepts.
    // Why:      The farm links a program it can find and reports the names it cannot, so the
    //           fixture's toolchain is what decides which of the row's bins resolve.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const name of theToolchainPrograms) writeExecutable(pathJoin(toolchainBin, name));
    // ```
    for name in THE_TOOLCHAIN_PROGRAMS {
        write_executable(&toolchain_bin.join(name));
    }
    // What:     the scope's lock becomes one row, the Rust toolchain's.
    // Why:      The sample lock's other tools would install archives and crates the fixture
    //           does not hold, and the control is about this row alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.lock.tools = [row];
    // ```
    rig.lock.tools = vec![row];
    return rig;
}

/// What:
///     `#[test] fn an_install_of_the_repository_rust_row_names_no_unresolved_bin()` installs
///     the repository's `rust` row and reads the last lines of the report it prints.
/// Why:
///     Issue 6626 is that line: an unchanged toolchain named `rustup`, a program no
///     toolchain holds, every time. The control fails when the row declares a program the
///     route cannot put in the farm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('an install of the repository rust row names no unresolved bin', () => { ... });
/// ```
#[test]
fn an_install_of_the_repository_rust_row_names_no_unresolved_bin() {
    // What:     the rig whose home holds the toolchain's programs.
    // Why:      The install writes nothing to this machine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rig = rigOfTheRepositoryRustRow();
    // ```
    let rig: Rig = rig_of_the_repository_rust_row();
    // What:     `sync_tools` installs every tool the lock holds, and `None` says "not a
    //           subset"; its report lists what was installed, what was left alone and the
    //           bins no farm link resolves.
    // Why:      This is the run whose printed line the issue reports.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const report = syncTools(rig.context(), undefined);
    // ```
    let report = sync_tools(&rig.context(), None);
    // What:     `format_sync_report` builds the text `install` prints from the report.
    // Why:      The issue is about the last line of that text, not about the report record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = formatSyncReport(report);
    // ```
    let text: String = format_sync_report(&report);
    // What:     the assertion compares the report's unresolved bins with an empty list, and
    //           `Vec::<String>::new()` is that empty list.
    // Why:      An empty list is the whole of the issue's "names no unresolved bin".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.missingBins).toEqual([]);
    // ```
    assert_eq!(report.missing_bins, Vec::<String>::new(), "{text}");
    // What:     `lines().last()` is the report's final line, and the assertion says the
    //           text never holds the unresolved-bins sentence.
    // Why:      The line is printed last, and a person reads the last lines; the assertion
    //           names the text so a failure shows what was printed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(text).not.toContain('unresolved bins');
    // ```
    let last: &str = text.lines().last().expect("the report prints at least one line");
    assert!(
        !last.contains("unresolved bins") && !text.contains("unresolved bins"),
        "{text}"
    );
}

/// What:
///     `#[test] fn a_bin_no_route_provides_is_still_named()` installs the same row with one
///     more bin and reads the report's last line.
/// Why:
///     The line must keep its meaning: a program nothing provides is still reported, so an
///     install that prints nothing says the machine is complete rather than that the report
///     stopped reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a bin no route provides is still named', () => { ... });
/// ```
#[test]
fn a_bin_no_route_provides_is_still_named() {
    // What:     the rig whose home holds the toolchain's programs.
    // Why:      The extra name must be the only one the farm cannot resolve.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rig = rigOfTheRepositoryRustRow();
    // ```
    let mut rig: Rig = rig_of_the_repository_rust_row();
    // What:     `cargo-miri` is a real program of the `miri` component, which no row of the
    //           plan declares, and this machine's toolchain holds no such file (the listing
    //           `THE_TOOLCHAIN_PROGRAMS` is copied from).
    //           `.push(...)` appends the name to the row's declared bins, and
    //           `.to_string()` copies the borrowed literal into the owned `String` the list
    //           holds.
    // Why:      The row has to declare a program the route cannot provide, without touching
    //           the repository's own row.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // rig.lock.tools[0].declaration.bins.push('cargo-miri');
    // ```
    rig.lock.tools[0]
        .declaration
        .bins
        .push("cargo-miri".to_string());
    // What:     the install of that row.
    // Why:      The report is what the person reads after it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const report = syncTools(rig.context(), undefined);
    // ```
    let report = sync_tools(&rig.context(), None);
    // What:     the printed text.
    // Why:      The assertion is about the line, not the record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = formatSyncReport(report);
    // ```
    let text: String = format_sync_report(&report);
    // What:     the report's last unresolved bin, which is the name appended last, compared
    //           with `Some(&"cargo-miri".to_string())` - the present variant of `Option`
    //           holding that name.
    // Why:      The name is declared last, so it is the last entry of the list, whatever else
    //           the row declares.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(report.missingBins.at(-1)).toBe('cargo-miri');
    // ```
    assert_eq!(
        report.missing_bins.last(),
        Some(&"cargo-miri".to_string()),
        "{text}"
    );
    // What:     the report's final line, and the assertion that it is the unresolved-bins
    //           sentence naming the program.
    // Why:      A person acting on the line has to find the name there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(text.trimEnd().split('\n').at(-1)).toContain('unresolved bins:');
    // ```
    let last: &str = text
        .trim_end()
        .lines()
        .last()
        .expect("the report prints at least one line");
    assert!(
        last.starts_with("toolchain install: unresolved bins: ") && last.contains("cargo-miri"),
        "{last}"
    );
}
