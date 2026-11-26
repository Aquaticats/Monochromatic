//! What:
//!  The fixture lifecycle both test targets run on:
//!  where a run's fixtures live,
//!  and how one fixture folder is made and removed.
//! Why:
//!  The library's unit tests and the `native_binary` target each kept a copy of these helpers,
//!  and the copies drifted apart,
//!  so a fix landed in one target and not the other
//!  (`doc/debt-ledger-2026-11-25.md`, item 11).
//!  Both targets include this one file by `#[path]`,
//!  as they already include `test_alive.rs`:
//!  `testing/test_support.rs` for the library's unit tests,
//!  and `binary/binary_tests.rs` for the binary controls.
//!  Each target keeps only what is its own:
//!  the unit tests' registry of fixture folders to remove,
//!  and the binary controls' `Fixture`,
//!  which links the built wrapper into the folder this file makes.
//!  The helper processes themselves are owned by `test_process_owner.rs`,
//!  a file of its own because the candidate consumer's crate takes it and keeps no fixtures.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // shared/fixture-support.ts
//! // fixtureRoot(); madeFixtureFolder(dir, prefix, name); settleFixture(root, keep);
//! // removePatiently(root, attempt, tries, pause);
//! ```

/// What:
///  `Path` is a borrowed file-system path (think TS's `string` when it names a file),
///  and `PathBuf` is the owned, growable form of one (think a `string` this code built and keeps).
/// Why:
///  A folder this file makes is handed back as the owned form the caller stores,
///  while the paths it is given are only looked at, so they stay borrowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // type Path = string   // borrowed view
/// // type PathBuf = string // owned copy
/// ```
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  `AtomicU64` is an unsigned 64-bit counter that several threads may change at once without a lock,
///  and `Ordering` says how much other memory traffic that change is allowed to move past it.
///  The siblings a reader may expect are `Cell<u64>` (single-threaded only) and `Mutex<u64>` (a lock).
/// Why:
///  Every test thread in one process calls the folder maker,
///  so the counter that names each folder has to be safe to use from all of them at once,
///  and an atomic needs no lock for one number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // let fixturesMade = 0n; const n = fixturesMade++; // the runtime makes the increment atomic enough for this
/// ```
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// What:
///  The environment variable that moves every fixture root,
///  for example onto a tmpfs for speed or onto a file system a control needs.
/// Why:
///  The container gate mounts a tmpfs and names it with this variable (`bin/native-gate-fixture-root.ts`),
///  so both test targets must read the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIXTURE_ROOT_VARIABLE = 'CLI_GIT_TEST_ROOT';
/// ```
pub(crate) const FIXTURE_ROOT_VARIABLE: &str = "CLI_GIT_TEST_ROOT";

/// What:
///  The environment variable that keeps every fixture on disk after its test ends,
///  for looking at a failure.
/// Why:
///  A person chasing a failing control sets it and reads the tree afterwards;
///  the removal helpers below return early when it is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEEP_FIXTURES_VARIABLE = 'CLI_GIT_KEEP_FIXTURES';
/// ```
pub(crate) const KEEP_FIXTURES_VARIABLE: &str = "CLI_GIT_KEEP_FIXTURES";

/// What:
///  Forbid core dumps for this test process and everything it starts,
///  once.
/// Why:
///   Tests make services abort or kill them on purpose,
///  and each dump costs the system's core-dump handler processor time and
///  disk writes on a disk other sessions share (several hundred dumps of `cli-git-service` from test runs on 2026-11-03).
///  A process limit is inherited across `fork` and `exec`,
///  and the hard limit stops a child from raising it;
///  this is the harness's limit,
///  and the product sets none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forbidCoreDumps(): void; // once: setrlimit(RLIMIT_CORE, { soft: 0, hard: 0 })
/// ```
pub(crate) fn forbid_core_dumps() {
    /// What:
    ///  The standard library's one-shot runner for a piece of work that must happen exactly once.
    /// Why:
    ///  Both targets call this at every fixture,
    ///  and a second `setrlimit` would be harmless but pointless work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // let done = false; if (done) return; done = true;
    /// ```
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // What: `rlimit` is the kernel's pair of numbers for one resource limit: the soft one in force and the hard one
        //  a process may raise the soft one to. Both zero here, so a dump is never written and no child may relax it.
        // Why: a dump is hundreds of megabytes the disk other sessions build on does not have to spare.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const none = { soft: 0, hard: 0 };
        // ```
        let none: libc::rlimit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // What: `setrlimit` asks the kernel to apply that pair; `unsafe` marks a call whose safety the compiler cannot
        //  check, and the `SAFETY` note says why this one is sound.
        // Why: apply the pair to this test process, from which every child inherits it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // setrlimit('core', none);
        // ```
        // SAFETY: `setrlimit` reads the one `rlimit` it is given and touches no memory of ours.
        let _set = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &none) };
    });
}

/// What:
///  The fixture root of one run:
///  the directory the variable `CLI_GIT_TEST_ROOT` names,
///  or `test-fixtures` beside the build output of the running test executable
/// (`<target>/debug/deps/<test>` has the target directory three levels up),
///  never `/tmp`;
///  and the parent of every `process-<pid>` folder a process's own fixtures are made in.
/// Why:
///   The one stored copy of the built executable that fixtures on another file system than the build's link stands in
///  this root (`binary_support_fixture_directories::fixture_in`),
///  so every process of one run links the one copy instead of copying the whole executable into a folder of its own:
///  the container gate's root is a 768 MiB tmpfs,
///  so a copy per test process filled it and the whole gate failed with `No space left on device` (issue 6488).
///  Everything one process writes for itself stays under its own `process-<pid>` folder (`fixture_root`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = env.CLI_GIT_TEST_ROOT ?? join(targetDir(), 'test-fixtures');
/// ```
pub(crate) fn run_fixture_root() -> PathBuf {
    // What: `map_or_else` answers from the first function when the variable is absent and from the second when it is
    //  present; `var_os` reads one environment variable as an operating-system string, the form a path may hold.
    //  The closure syntax `|| { ... }` is TS's `() => { ... }`.
    // Why: a run that names a root keeps its fixtures there and every other run keeps them beside the build output.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const base = env.CLI_GIT_TEST_ROOT !== undefined ? env.CLI_GIT_TEST_ROOT : targetDir();
    // ```
    let base: PathBuf = std::env::var_os(FIXTURE_ROOT_VARIABLE).map_or_else(
        || {
            // What: the target directory three levels above this test binary, read from the binary's own path.
            // Why: the build output is where the disk the build already uses is, and never the shared temporary one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const target = dirname(dirname(dirname(process.execPath)));
            // ```
            let target: Option<PathBuf> = this_test_binary().ok().and_then(|executable| {
                return executable.ancestors().nth(3).map(Path::to_path_buf);
            });
            // What: `unwrap_or_else` answers the found target, or the manifest directory's `target` when the test
            //  binary's own path could not be read.
            // Why: a build that runs the test binary from somewhere unexpected still gets a root inside the crate.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const root = join(target ?? join(manifestDir, 'target'), 'test-fixtures');
            // ```
            return target
                .unwrap_or_else(|| {
                    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
                })
                .join("test-fixtures");
        },
        PathBuf::from,
    );
    // What: `create_dir_all` makes the folder and any parent that is missing; `expect` ends the test with a message
    //  when it fails, and it then returns `()`.
    // Why: the wrapper store and every process's own folder are made under this one, so it must exist before the first
    //  fixture is made.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mkdirSync(base, { recursive: true });
    // ```
    std::fs::create_dir_all(&base).expect("run fixture root");
    // What: a tail expression with no `;` is the return value, here the folder every process of the run shares.
    // Why: the caller joins its own names under it, and the wrapper store stands in it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return base;
    // ```
    return base;
}

/// What:
///  The directory this process's fixtures are created in:
///  `process-<pid>` under the run's own fixture root (`run_fixture_root`), never `/tmp`.
/// Why:
///   The build output is shared by every checkout that builds into the same target directory,
///  so a bare `test-fixtures` itself is shared by every checkout and every test process of this host.
///  The names under it that are not the fixtures' own (the scratch home the product keeps
///  its per-user state in,
///  the marker `refused-push-ran`,
///  the repositories a control makes beside the fixtures)
///  were one fixed name for all of them,
///  so two runs at once shared them:
///  two checkouts started at the same moment reported three subvolume controls failing in
///  ten to twenty seconds while the same three passed alone.
///  A directory of the process's own makes every fixed name below it the process's,
///  and nothing else changes:
///  fixtures are still removed by their own test and kept for inspection on request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = join(runFixtureRoot(), `process-${process.pid}`);
/// ```
pub(crate) fn fixture_root() -> PathBuf {
    // What: the folder is per process, named after the process number that `id()` reads.
    // Why: two test processes of the same checkout, and two checkouts, then never share one fixed name below it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const root = join(base, `process-${process.pid}`);
    // ```
    let root: PathBuf = run_fixture_root().join(format!("process-{}", std::process::id()));
    // What: `create_dir_all` makes the folder and any parent that is missing; `expect` ends the test with a message
    //  when it fails, and it then returns `()`.
    // Why: every caller writes fixtures under this folder, so it must exist before the first one is made.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mkdirSync(root, { recursive: true });
    // ```
    std::fs::create_dir_all(&root).expect("fixture root");
    // What: a tail expression with no `;` is the return value, here the folder this run's fixtures are made in.
    // Why: the caller joins its own names under it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return root;
    // ```
    return root;
}

/// This test process's own executable,
/// for the helper processes a control starts from the test binary.
///  A test starts the test binary,
/// never the wrapper:
/// the ban on `std::env::current_exe` (`clippy.toml`) is about the
///  wrapper starting itself,
/// so this is the one place the lint is excused for tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thisTestBinary(): string | undefined; // process.execPath
/// ```
#[allow(
    clippy::disallowed_methods,
    reason = "identifies the running test binary so a control can start a helper copy of it; the wrapper is never started this way"
)]
pub(crate) fn this_test_binary() -> std::io::Result<PathBuf> {
    // What: `current_exe` asks the operating system for the path of the program running now.
    // Why: a control starts a second copy of this test binary to play a helper, and it must name the same file.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return process.execPath;
    // ```
    return std::env::current_exe();
}

/// What:
///  Every directory that is a system temporary directory:
///  `/tmp`,
///  `/var/tmp`,
///  and `TMPDIR` when the run sets it to one.
/// Why:
///   A fixture root there fills a shared tmpfs with the trees of tests that never removed them (issue 1809),
///  so the location control compares the root a run uses against this list.
///  Reading `TMPDIR` compares against it
///  without calling the temporary-directory function that the lint configuration bans.
///  An empty `TMPDIR` names no directory and is left out,
///  and an entry is what its path resolves to,
///  because a `TMPDIR` that is a link to the root is that root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function systemTemporaryDirectories(): string[]; // ['/tmp', '/var/tmp', env.TMPDIR].map(realpath).filter(Boolean)
/// ```
pub(crate) fn system_temporary_directories() -> Vec<PathBuf> {
    // What: `vec!` builds an owned growable array (TS's `[...]`), here the two directories every Unix system has.
    // Why: the fixed pair is always a temporary directory, whether or not the run names one more.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const listed = ['/tmp', '/var/tmp'];
    // ```
    let listed: Vec<PathBuf> = vec![PathBuf::from("/tmp"), PathBuf::from("/var/tmp")];
    // What: the run's own `TMPDIR`, kept only when it names something; `filter` keeps an element only where its
    //  closure answers `true`.
    // Why: an empty variable names no directory, and comparing against the empty path would be a false match.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const set = env.TMPDIR ? env.TMPDIR : undefined;
    // ```
    let set: Option<PathBuf> = std::env::var_os("TMPDIR")
        .map(PathBuf::from)
        .filter(|directory| {
            return !directory.as_os_str().is_empty();
        });
    // What: `chain` appends the optional entry after the listed pair, `map` turns each one into what its path
    //  resolves to, and `collect` builds the owned array the caller reads.
    // Why: a root that is a link into a listed directory is in that directory, so the comparison must use the
    //  resolved path; a path that cannot be resolved is compared as written.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [...listed, ...(set ? [set] : [])].map((d) => realpathSync(d) ?? d);
    // ```
    return listed
        .into_iter()
        .chain(set)
        .map(|directory| {
            return std::fs::canonicalize(&directory).unwrap_or(directory);
        })
        .collect();
}

/// How many fixture folders this process has made,
/// which names each call's folder its own.
/// Why:
///   A run of `cargo test` is many tests as threads of one process,
///  and two tests that pass one name took one folder,
///  whose removal each made while the other was writing in it
///  (`doc/troubleshooting/git-policy-cli-controls-that-name-one-fixture-folder.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let fixturesMade = 0n;
/// ```
static FIXTURES_MADE: AtomicU64 = AtomicU64::new(0);

/// The number this call's folder is named with:
/// one number for every call, in the order the process makes them.
/// Why:
///  The folder carries the process's number and this call's number,
///  so two calls of one process never name the same folder even when they pass the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theNumberOfThisFixture(): bigint; // return fixturesMade++;
/// ```
fn the_number_of_this_fixture() -> u64 {
    // What: `fetch_add` adds one and answers the value the counter held before the addition, in one indivisible step;
    //  `Relaxed` says no other memory traffic has to be ordered around it.
    // Why: two threads making a fixture at the same moment still get two different numbers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return fixturesMade++;
    // ```
    return FIXTURES_MADE.fetch_add(1, Ordering::Relaxed);
}

/// What:
///  Make one empty fixture folder for this call,
///  named `<prefix>-<pid>-<this call's number>-<name>`,
///  and answer the folder's resolved path.
/// Why:
///   Every call names a folder of its own,
///  so two tests that pass the same name never share one,
///  and neither removes the other's live folder:
///  the folder carries the process's number and the number this call took,
///  and the name the test writes.
///  The prefix keeps the folders of one target apart from the other's.
///  A folder of a run that was killed may carry this process's number and this call's number,
///  so one that exists is removed first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function madeFixtureFolder(directory: string, prefix: string, name: string): string;
/// ```
pub(crate) fn made_fixture_folder(
    directory: &Path,
    prefix: &str,
    name: &str,
) -> PathBuf {
    // What: `create_dir_all` makes the parent folder too, and the `if let Err` arm takes the failure value out of the
    //  `Result` the call answers (`Result` is Rust's "either a value or an error", TS's `T | Error`).
    // Why: a root that has gone makes every later failure a bare "no such file"; the report below names what became
    //  of the root, its parent and its grandparent instead.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { mkdirSync(directory, { recursive: true }); } catch (error) { throw new Error(`fixture directory root: ${error}; ${missingFixtureRootReport(directory)}`); }
    // ```
    if let Err(error) = std::fs::create_dir_all(directory) {
        // What: `panic!` ends the test at once with the message it is given.
        // Why: a fixture whose root cannot be made cannot host the control, and the report says what went.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // throw new Error(`fixture directory root: ${error}; ${missingFixtureRootReport(directory)}`);
        // ```
        panic!(
            "fixture directory root: {error}; {}",
            missing_fixture_root_report(directory)
        );
    }
    // What: the folder's own name, built with `format!` (TS's template string) from the prefix, the process number,
    //  this call's number and the name the test asked for.
    // Why: that combination is what makes two calls of one process, and two processes, name different folders.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const created = join(directory, `${prefix}-${process.pid}-${theNumberOfThisFixture()}-${name}`);
    // ```
    let created: PathBuf = directory.join(format!(
        "{prefix}-{}-{}-{name}",
        std::process::id(),
        the_number_of_this_fixture()
    ));
    // What: `exists` asks whether anything at all stands at that path.
    // Why: a folder a killed run left with this call's number is in the way of `create_dir` below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (existsSync(created)) { ... }
    // ```
    if created.exists() {
        // A folder an earlier run was killed in the middle of may hold a directory it closed, so the permission is given
        //  back before the removal is tried.
        restore_owner_access(created.as_path());
        // What: `remove_dir_all` removes the tree and everything below it; `expect` ends the test with a message when
        //  the removal fails.
        // Why: the folder must be empty and new before the fixture is written into it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // rmSync(created, { recursive: true, force: true });
        // ```
        std::fs::remove_dir_all(&created).expect("remove stale fixture");
    }
    // What: `create_dir` makes the one folder and refuses when its parent is missing, which the call above made sure
    //  of.
    // Why: an empty folder is what a fresh fixture is; the caller fills it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!mkdirSync(created)) throw new Error(`fresh fixture: ...; ${missingFixtureRootReport(directory)}`);
    // ```
    if let Err(error) = std::fs::create_dir(&created) {
        panic!(
            "fresh fixture: {error}; {}",
            missing_fixture_root_report(directory)
        );
    }
    // What: `canonicalize` answers the path with every link and `.`/`..` resolved, so comparisons with Git's own
    //  canonical output are exact.
    // Why: a fixture root that is a link would otherwise disagree with the paths Git prints.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return realpathSync(created);
    // ```
    return std::fs::canonicalize(&created).expect("canonical fixture");
}

/// How many entries of a folder the report below names before it says there are more.
/// Why:
///  A gate's fixture root once held thousands of entries, and a report that listed all of them would bury its first
///  line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REPORTED_ENTRIES = 40;
/// ```
const REPORTED_ENTRIES: usize = 40;

/// What:
///  What is known about a fixture root that could not be used,
///  so the failure names the folder that went:
///  whether the root exists,
///  and what its parent and its grandparent hold,
///  or that they are gone too.
/// Why:
///  A run of the gate once lost its fixture root between `create_dir_all` and the next `create_dir`,
///  and 217 later tests failed with a bare `NotFound`
///  (`doc/handover/repository-move-dangling-link-items.md`,
///  open harness item);
///  nothing in the log said what removed it or how far up the loss went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingFixtureRootReport(root: string): string; // names the root and what its parent and grandparent hold
/// ```
pub(crate) fn missing_fixture_root_report(root: &Path) -> String {
    // What: `parent` answers the folder containing this path, as an `Option` because a root path has none;
    //  `unwrap_or` substitutes the path itself in that case.
    // Why: the report must name two folders even when the failed path is as short as `/`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parent = dirname(root); const grandparent = dirname(parent);
    // ```
    let parent: &Path = root.parent().unwrap_or(root);
    let grandparent: &Path = parent.parent().unwrap_or(parent);
    // What: a `&str` here is a borrowed piece of text (TS's `string`, without a copy), chosen by the `if` expression.
    // Why: the first line of the report says whether the root itself is there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const state = existsSync(root) ? 'exists' : 'does not exist';
    // ```
    let state: &str = if root.exists() {
        "exists"
    } else {
        "does not exist"
    };
    // What: the tail expression is the report: the root with its state, then what each folder above it holds.
    // Why: a person reading a failure of 217 tests learns which folder went and how far up the loss reached.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `the fixture root ${root} ${state}; ${whatAFolderHolds('its parent', parent)}; ${whatAFolderHolds('its grandparent', grandparent)}`;
    // ```
    return format!(
        "the fixture root {} {state}; {}; {}",
        root.display(),
        what_a_folder_holds("its parent", parent),
        what_a_folder_holds("its grandparent", grandparent)
    );
}

/// The phrase that says what a folder holds,
///  or that it does not exist,
///  or why it cannot be listed.
/// Why:
///  The report above reads as one sentence per folder, so a person sees the first folder that went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatAFolderHolds(label: string, folder: string): string;
/// ```
fn what_a_folder_holds(
    label: &str,
    folder: &Path,
) -> String {
    // What: `exists` asks whether anything stands at the path.
    // Why: a folder that is gone is a different answer from a folder that is empty, and the report says which.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!existsSync(folder)) return `${label} ${folder} does not exist`;
    // ```
    if !folder.exists() {
        return format!("{label} {} does not exist", folder.display());
    }
    // What: the `match` reads the `Result` that `read_dir` answers: `Ok` holds the listing, `Err` the reason it
    //  could not be read. Both arms must be written, because the compiler checks that every case is handled.
    // Why: a folder that exists but cannot be read still has to be reported, and with the reason.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let listing; try { listing = readdirSync(folder); } catch (error) { return `${label} ${folder} cannot be listed: ${error}`; }
    // ```
    let listing = match std::fs::read_dir(folder) {
        Ok(listed) => listed,
        Err(error) => return format!("{label} {} cannot be listed: {error}", folder.display()),
    };
    // What: an owned array of the names, filled one entry at a time below.
    // Why: the names are sorted and cut to `REPORTED_ENTRIES`, which needs them all in hand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const names: string[] = [];
    // ```
    let mut names: Vec<String> = Vec::new();
    // What: `flatten` walks the listing and skips an entry whose reading failed; the `for` loop runs once per entry.
    // Why: one unreadable entry must not hide the names the report can give.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const entry of listing) names.push(entry.name);
    // ```
    for entry in listing.flatten() {
        // What: `to_string_lossy` turns the entry's operating-system name into text, replacing bytes that are not text
        //  with the replacement character; `into_owned` then copies it into an owned `String`.
        // Why: the report is text, and a name that is not text must still appear.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // names.push(entry.fileName.toString());
        // ```
        names.push(entry.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    // What: `saturating_sub` subtracts but stops at zero rather than wrapping round; the array's length minus the
    //  budget is how many names are not shown.
    // Why: the tail of the report must say how much it left out.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const more = Math.max(0, names.length - REPORTED_ENTRIES);
    // ```
    let more: usize = names.len().saturating_sub(REPORTED_ENTRIES);
    names.truncate(REPORTED_ENTRIES);
    // What: `String::new` is an empty owned string, and the `if` expression picks it or the count of the names left out.
    // Why: only a list that was cut has to say that it was.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const tail = more === 0 ? '' : ` and ${more} more`;
    // ```
    let tail: String = if more == 0 {
        String::new()
    } else {
        format!(" and {more} more")
    };
    // What: the tail expression joins the label, the folder, the shown names with `", "` between them, and the count
    //  of the names left out.
    // Why: one line per folder is what the failure report is built from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `${label} ${folder} holds: ${names.join(', ')}${tail}`;
    // ```
    return format!(
        "{label} {} holds: {}{tail}",
        folder.display(),
        names.join(", ")
    );
}

/// What:
///  Give this user back the read,
///  write and search permission on every directory of a tree that has lost it,
///  without following a symbolic link.
/// Why:
///   A control that writes a directory its owner may not write (`0500`,
///  `0555`,
///  `0000`) proves what the product does with one,
///  and removing that directory afterwards needs read to list it and write and search to unlink what it holds,
///  so a fixture left in that state outlived the test that made it
///  (issue 6282:
///  fixtures named `rollback-read-only`,
///  `remove-old-read-only`,
///  `migrate-unreadable-dir` and `sync-unreadable-dir`),
///  and a build-tree sweeper could not delete the tree they sat in.
///   Only directories are changed,
///  only ones this user owns,
///  and only the owner's three bits:
///  a file with no permissions of its own is unlinked from a writable directory,
///  and a group or other-user bit a control set stays as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreOwnerAccess(root: string): void // lstat each entry, chmod OWNER_RWX on the directories this user owns
/// ```
#[cfg(unix)]
pub(crate) fn restore_owner_access(root: &Path) {
    // What: `MetadataExt` and `PermissionsExt` add the Unix-only methods (`mode`, `uid`, `from_mode`) to the standard
    //  library's file metadata and permissions types; `as _` imports the traits without naming them.
    // Why: the owner number and the permission bits are what decide whether this call may widen a directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // import { lstatSync, chmodSync } from 'node:fs';
    // ```
    use std::os::unix::fs::{
        MetadataExt as _,
        PermissionsExt as _,
    };
    // What: `geteuid` answers the number of this process's user; `unsafe` because the compiler cannot check the call.
    // Why: only a folder this user owns may have its mode changed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const own = geteuid();
    // ```
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let own: u32 = unsafe { libc::geteuid() };
    // What: the read, write and search bits of the owner, as one number.
    // Why: a directory with all three can be listed and emptied by its owner.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ownerBits = 0o700;
    // ```
    let owner_bits: u32 = 0o700;
    // What: a work list of directories still to visit, starting with the root; `pop` takes the last one off.
    // Why: a stack walks the tree without recursion, however deep it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stack = [root];
    // ```
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    // What: `Some(directory)` is the present case of an `Option` (TS's `string | undefined`), and `let ... else`
    //  takes the value out or runs its `else` block, which here goes on to the next entry of the loop.
    // Why: a path that vanished while this walks must be skipped, not end the removal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (stack.length > 0) { const directory = stack.pop(); if (!directory) continue; ... }
    // ```
    while let Some(directory) = stack.pop() {
        // What: `symlink_metadata` reads the entry itself, never the file a link points at.
        // Why: a link out of the tree must not have its target's mode changed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const metadata = lstatSync(directory);
        // ```
        let Ok(metadata) = std::fs::symlink_metadata(&directory) else {
            continue;
        };
        // What: `is_dir` asks whether the entry is a directory.
        // Why: only directories are widened; a file is unlinked from a writable directory instead.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!metadata.isDirectory()) continue;
        // ```
        if !metadata.is_dir() {
            continue;
        }
        // What: the twelve permission bits of the entry, from its owner's three to the set-user-id bit.
        // Why: the widened mode keeps every bit it had and adds the owner's three.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const mode = metadata.mode & 0o7777;
        // ```
        let mode: u32 = metadata.mode() & 0o7777;
        // What: `mode & owner_bits != owner_bits` is true when at least one of the owner's read, write and search bits
        //  is missing.
        // Why: only a directory that lacks one of them needs the extra work of a `set_permissions` call.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (metadata.uid === own && (mode & ownerBits) !== ownerBits) { ... }
        // ```
        if metadata.uid() == own && mode & owner_bits != owner_bits {
            // What: `from_mode` builds the permission value and `set_permissions` applies it; the `|` adds the owner's
            //  three bits to the ones already there.
            // Why: the folder can be listed and emptied again after this.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // chmodSync(directory, (mode | ownerBits) & 0o7777);
            // ```
            let _widened = std::fs::set_permissions(
                &directory,
                std::fs::Permissions::from_mode(mode | owner_bits),
            );
        }
        // The directory may have been closed to this user until a moment ago, so it is listed after the mode is restored.
        //
        // What: `read_dir` lists the entries of one directory; `flatten` skips an entry whose reading failed.
        // Why: the directories below this one are visited only once this one can be listed again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const entry of readdirSync(directory)) { ... }
        // ```
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            // What: the entry's own metadata, again without following a link.
            // Why: only a real directory below this one is pushed.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const found = lstatSync(path);
            // ```
            let Ok(found) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            // What: `push` adds one directory to the end of the work list.
            // Why: it is visited on a later turn of the loop, after its own parent's mode is back.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (found.isDirectory()) stack.push(path);
            // ```
            if found.is_dir() {
                stack.push(path);
            }
        }
    }
}

/// What:
///  On a platform without Unix permissions there is nothing to give back.
/// Why:
///   The teardown calls this on every platform.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreOwnerAccess(root: string): void {} // no-op where permissions are not a Unix mode
/// ```
#[cfg(not(unix))]
pub(crate) fn restore_owner_access(_root: &Path) {}

/// How many times a removal that finds the directory still gaining entries is tried.
/// Why:
///  A child that a landed commit or a stopped service left finishing keeps adding entries for a moment; five seconds
///  of tries is far longer than that moment and still ends a wait that will never succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMOVAL_TRIES = 50;
/// ```
pub(crate) const REMOVAL_TRIES: usize = 50;

/// The pause between two tries of a removal.
/// Why:
///  The entries a finishing child adds arrive within milliseconds, so a pause this short spends the five seconds on
///  tries rather than on one sleep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMOVAL_PAUSE = 100; // milliseconds
/// ```
pub(crate) const REMOVAL_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// What:
///  Remove the tree at `root` with `attempt`,
///  trying again while the directory is not empty.
/// Why:
///   A landed commit or a command that stopped a service leaves a child finishing for a moment,
///  and removing a directory that
///  still gains entries fails with "not empty";
///  that is an event that ends by itself,
///  so the removal waits for it.
///  A tree that
///  is already gone is not a failure,
///  and any other error is returned at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removePatiently(root: string, attempt: (root: string) => void, tries: number, pause: number): Promise<void>;
/// ```
pub(crate) fn remove_patiently(
    root: &Path,
    attempt: &dyn Fn(&Path) -> std::io::Result<()>,
    tries: usize,
    pause: std::time::Duration,
) -> std::io::Result<()> {
    // What: the last failure seen, held in Rust's `Result` (either `Ok(())` or one `std::io::Error`).
    // Why: a loop that runs out of tries has to answer the failure that stopped it from removing the tree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let last: Error | undefined = undefined;
    // ```
    let mut last: std::io::Result<()> = Ok(());
    for _ in 0..tries {
        // What: `match` reads the `Result` the attempt answers: `Ok` is success, `Err(error)` a failure whose kind
        //  (`NotFound`, `DirectoryNotEmpty`, anything else) the guard arms below sort.
        // Why: a tree that is gone is a success, one that is still gaining entries is worth another try, and every
        //  other failure is the caller's to see at once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { attempt(root); return; } catch (error) { ... }
        // ```
        match attempt(root) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                last = Err(error);
                std::thread::sleep(pause);
            }
            Err(error) => return Err(error),
        }
    }
    // What: the tail expression hands back the last failure the loop saw.
    // Why: the caller decides whether a tree that never emptied is a failure of its test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // throw last;
    // ```
    return last;
}

/// What:
///  Remove a fixture tree unless the person asked to keep it,
///  giving back the owner's permission on every directory of it first.
/// Why:
///   A landed commit may leave a maintenance child finishing for a moment;
///  a tree already gone is the explicit removal having run first,
///  not a failure.
///   A control that made a directory read-only inside its fixture is answered here,
///  so the fixture goes however the control ended,
///  panic included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleFixture(root: string, keep: boolean): void; // best effort: never fails the test
/// ```
pub(crate) fn settle_fixture(
    root: &Path,
    keep: bool,
) {
    // What: leaving early when the person asked to keep the tree.
    // Why: a kept fixture is what a failure is looked at in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (keep) return;
    // ```
    if keep {
        return;
    }
    restore_owner_access(root);
    // What: `let _name` keeps the answer but marks that nothing reads it; the call is the point.
    // Why: this removal is best effort, and the caller has no way to act on a failure at teardown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { removePatiently(root, removeDirAll, REMOVAL_TRIES, REMOVAL_PAUSE); } catch {}
    // ```
    let _removed: std::io::Result<()> = remove_patiently(
        root,
        &|directory| {
            return std::fs::remove_dir_all(directory);
        },
        REMOVAL_TRIES,
        REMOVAL_PAUSE,
    );
}
