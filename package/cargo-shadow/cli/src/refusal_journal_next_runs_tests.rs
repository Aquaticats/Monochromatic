//! What:
//! The journals a refusal keeps,
//! and what each kind of next run of the same workspace does with one:
//! an ordinary run that resolves dependencies,
//! a command that resolves nothing,
//! a frozen run,
//! and `cargo-shadow update`.
//! Why:
//! The refusal of a run that could not be settled says what happens to the journal it kept,
//! and the sentence has to name the runs that settle it as well as the runs that leave it,
//! because only two call sites of this crate read a kept journal (`src/install.rs` and `src/update.rs`)
//! and the rest of the commands reach neither.
//! Every leg here starts from a real refusal:
//! the stop is made to report that the scope vanished between the last `populated` read and the listing
//! (`ContainmentStopStuck` and `ContainmentScopeGoneAtListing`),
//! which is the arm of the refusal whose sentence names the journal.
//! Each next run is a child of this test executable,
//! so what it prints is read from its own standard output and error and not from a call inside this process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the next run after a refusal', () => { it('a pass-through leaves the journal', ...) })
//! ```

/// Holds the count of started runs the helper watches, and the callback that reads it.
use std::cell::Cell;
/// Holds the counters the fakes of the update leg keep, and the count of the starts the helper watches.
use std::cell::RefCell;
/// Holds the versions the fake registry answers with.
use std::collections::BTreeMap;
/// Copies text into an owned `String`.
use std::ffi::OsString;
/// Reads and writes the fixture's files, the stand-ins, and the child's exit code.
use std::fs;
/// Makes the stand-in cargo executable.
use std::os::unix::fs::PermissionsExt;
/// Borrows and owns the fixture's paths.
use std::path::{
    Path,
    PathBuf,
};
/// Takes a child's standard output and status.
use std::process::{
    Command,
    Output,
};
/// Shares one counter between the helper and the hook that reads it.
use std::rc::Rc;

/// The cargo runner the install step and the update call.
use crate::cargo_run::{
    CargoOutcome,
    CargoRunner,
};
/// The waits the control-group tests share.
use crate::cgroup_test_support::{
    scope_directory_of,
    wait_until,
};
/// The one decision the refuse-on test arms: the stop's report and the listing of a scope that went away.
use crate::failpoints::{
    Point,
    arm,
};
/// The error a registry answers with.
use crate::http::RegistryError;
/// The journal directories of a fixture.
use crate::journal_support::journal_dirs;
/// One release of one crate, as a registry answers it.
use crate::policy::VersionRecord;
/// The registry the update asks about versions.
use crate::registry::Registry;
/// The tests' own scratch directory, beside the build output.
use crate::test_scratch::scratch;
/// Renders the catalog into the manifests, which the update calls after it resolves.
use crate::tighten::Syncer;
/// The update's own entry point and the two values it is called with.
use crate::update::{
    UpdateKind,
    UpdateRequest,
    update_rust_lockfiles,
};
/// What a fake cargo answers and how it is asked, for the install step and the update.
use monochromatic_dependency_catalog::Settings;
/// The trust policy the fixture's catalog states.
use monochromatic_dependency_catalog::TrustPolicy;

/// The variable that carries the fixture's root to the child.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_FIXTURE_ROOT";
/// The variable that names the next run the child makes.
const NEXT_RUN: &str = "CARGO_SHADOW_NEXT_RUN";
/// The crate root inside the fixture's catalog repository.
const CRATE_DIR: &str = "package/x/y";
/// A lockfile that does not satisfy its manifests.
const STALE: &str = "# stale\nversion = 4\n";
/// The stand-in cargo:
/// it logs every call,
/// answers `update --locked` by whether the lock starts `# satisfied`,
/// writes a satisfied lock on a plain `update`,
/// answers `metadata` with an empty package list of this workspace,
/// and prints a marker for anything else.
/// A stand-in: no claim about real cargo rests on it.
const FAKE_CARGO: &str = "#!/bin/sh\n\
echo \"$@\" >> \"$FAKE_CARGO_LOG\"\n\
if [ \"$1\" = update ]; then\n\
  case \" $* \" in\n\
    *\" --locked \"*) head -n 1 Cargo.lock | grep -q satisfied; exit $? ;;\n\
  esac\n\
  printf '# satisfied\\nversion = 4\\n' > Cargo.lock\n\
  exit 0\n\
fi\n\
if [ \"$1\" = metadata ]; then\n\
  printf '{\"workspace_root\":\"%s\",\"packages\":[]}' \"$PWD\"\n\
  exit 0\n\
fi\n\
echo \"fake-cargo-ran $@\"\n\
exit 0\n";

/// The fixture's settings, which match the catalog the fixture writes.
fn settings() -> Settings {
    // What:     `Settings { .. }` is the catalog's parsed settings, built field by field here.
    // Why:      The fixture writes a catalog file; the update and the install step are handed the settings the same way a
    //           command hands them over, and this is what that file parses to.
    return Settings {
        catalog_prune: false,
        minimum_release_age: 0,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: false,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

/// A fake cargo for the update leg:
/// a plain `generate-lockfile` or `update` writes the satisfied lock,
/// `update --locked` answers by the lock's first line,
/// `metadata` lists the one crate of the workspace,
/// and every call is recorded.
struct FakeCargo {
    /// The calls the update made.
    calls: RefCell<Vec<String>>,
}

/// What:
/// `impl CargoRunner for FakeCargo` answers what a real cargo would answer for this fixture.
/// Why:
/// The update's recovery runs before any resolution,
/// so the runner only has to let the update get past its own work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class FakeCargo implements CargoRunner { run(cwd, args): CargoOutcome }
/// ```
impl CargoRunner for FakeCargo {
    fn run(
        &self,
        cwd: &Path,
        args: &[String],
    ) -> Result<CargoOutcome, String> {
        self.calls.borrow_mut().push(args.join(" "));
        // What:     `lock` is the lockfile beside the directory cargo is run in.
        // Why:      The resolution writes it, exactly as a real `cargo generate-lockfile` would.
        let lock: PathBuf = cwd.join("Cargo.lock");
        if args.iter().any(|argument| {
            return argument == "metadata";
        }) {
            return Ok(CargoOutcome {
                ok: true,
                output: format!(
                    "{{\"workspace_root\":\"{}\",\"packages\":[]}}",
                    cwd.display()
                ),
                stdout: String::new(),
            });
        }
        if args[0] == "update"
            && args.iter().any(|argument| {
                return argument == "--locked";
            })
        {
            let satisfied: bool = fs::read_to_string(&lock)
                .unwrap_or_default()
                .starts_with("# satisfied");
            return Ok(CargoOutcome {
                ok: satisfied,
                output: "needs update".to_string(),
                stdout: String::new(),
            });
        }
        fs::write(&lock, "# satisfied\nversion = 4\n").map_err(|error| {
            return error.to_string();
        })?;
        return Ok(CargoOutcome {
            ok: true,
            output: String::new(),
            stdout: String::new(),
        });
    }
}

/// A registry with no versions of anything, which is every crate this fixture's lock names.
struct EmptyRegistry;

/// What:
/// `impl Registry for EmptyRegistry` answers nothing about any crate.
/// Why:
/// The fixture's lockfile names no dependency,
/// so the update asks about none, and a registry that answers "none" says so if one is ever asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class EmptyRegistry implements Registry { pubtimes(): Map; yanked(): Set; history(): []; owners(): Set }
/// ```
impl Registry for EmptyRegistry {
    fn pubtimes(
        &self,
        _name: &str,
    ) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(
        &self,
        _name: &str,
    ) -> Result<std::collections::BTreeSet<String>, RegistryError> {
        return Ok(std::collections::BTreeSet::new());
    }
    fn history(
        &self,
        _name: &str,
        _versions: &[String],
    ) -> Result<Vec<VersionRecord>, RegistryError> {
        return Ok(Vec::new());
    }
    fn owners(
        &self,
        _name: &str,
    ) -> Result<std::collections::BTreeSet<String>, RegistryError> {
        return Ok(std::collections::BTreeSet::new());
    }
}

/// A catalog sync that renders nothing, which the update calls after it resolves.
struct NoSync;

/// What:
/// `impl Syncer for NoSync` answers that the catalog was rendered.
/// Why:
/// The fixture's catalog is written by hand and no manifest states a catalog entry,
/// so the sync has nothing to do and the update only needs its answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NoSync implements Syncer { sync(root): 'synced' }
/// ```
impl Syncer for NoSync {
    fn sync(
        &self,
        _root: &Path,
    ) -> Result<String, String> {
        return Ok("synced".to_string());
    }
}

/// What:
/// `struct Fixture` is one catalog repository whose lockfile is stale, and the stand-in cargo beside it.
/// Why:
/// Every leg of this file needs the same world,
/// and the refusal leaves the journal in it.
struct Fixture {
    /// The scratch directory that holds the repository and the stand-ins.
    root: tempfile::TempDir,
    /// The directory the fake cargo stands in, first on the child's `PATH`.
    programs: PathBuf,
    /// The log the fake cargo appends its calls to.
    log: PathBuf,
}

/// What:
/// `impl Fixture` builds the world and answers the paths a run is started in.
/// Why:
/// The repository must look like a catalog repository with one crate whose lockfile is stale,
/// or the install step skips and there is no journal to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Fixture { static made(): Fixture; start: string }
/// ```
impl Fixture {
    /// What:
    /// `fn made() -> Fixture` writes the catalog, the crate, its stale lockfile and the stand-in cargo.
    /// Why:
    /// A fixture that is not a catalog repository would make the install step skip,
    /// and the test would show nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static made(): Fixture
    /// ```
    fn made() -> Fixture {
        let root = scratch();
        fs::write(
            root.path().join("rust-catalog.toml"),
            "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n",
        )
        .expect("write the catalog");
        let crate_root: PathBuf = root.path().join(CRATE_DIR);
        fs::create_dir_all(crate_root.join("src")).expect("make the crate directory");
        fs::write(
            crate_root.join("Cargo.toml"),
            "[package]\nname = \"y\"\nversion = \"0.1.0\"\n\n[workspace]\n",
        )
        .expect("write the manifest");
        fs::write(crate_root.join("Cargo.lock"), STALE).expect("write the lockfile");
        // What:     `programs` is the directory the stand-in cargo is written into.
        // Why:      The child's `PATH` starts with it, so the shadow finds this cargo and no real one.
        let programs: PathBuf = root.path().join("programs");
        fs::create_dir(&programs).expect("make the programs directory");
        let cargo: PathBuf = programs.join("cargo");
        fs::write(&cargo, FAKE_CARGO).expect("write the stand-in cargo");
        fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755))
            .expect("make the stand-in cargo executable");
        // What:     `log` is the file the stand-in cargo appends every call to.
        // Why:      A leg that must not resolve has to show that cargo was not asked at all.
        let log: PathBuf = root.path().join("fake-cargo.log");
        return Fixture {
            root,
            programs,
            log,
        };
    }

    /// What:
    /// `fn start(&self) -> PathBuf` is the crate root a run is started in.
    /// Why:
    /// The shadow finds the catalog above the working directory and runs the install step over that crate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get start(): string { return join(this.root.path(), 'package/x/y') }
    /// ```
    fn start(&self) -> PathBuf {
        return self.root.path().join(CRATE_DIR);
    }

    /// What:
    /// `fn journals(&self) -> Vec<PathBuf>` is every journal directory the fixture holds now.
    /// Why:
    /// Whether the journal is settled is the claim every leg is judged on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// journals(): string[] { return journalDirs(this.root.path()) }
    /// ```
    fn journals(&self) -> Vec<PathBuf> {
        return journal_dirs(self.root.path());
    }
}

/// What:
/// `struct Ran` is what one child run of this test executable printed and answered.
/// Why:
/// The claim about a next run is about what it prints and what it leaves,
/// and a child's own standard streams carry the first.
struct Ran {
    /// What the child wrote to standard output.
    stdout: String,
    /// What the child wrote to standard error.
    stderr: String,
    /// The status the child's run answered, or the harness's status where the run answered none.
    code: i32,
}

/// What:
/// `impl Ran` reads the child's text the way a person reads it.
/// Why:
/// Every assertion is about a line of that text, never about the whole of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Ran { prints(line: string): boolean }
/// ```
impl Ran {
    /// What:
    /// `fn prints(&self, needle: &str) -> bool` says whether either stream holds this text.
    /// Why:
    /// A warning may be printed by the run or by a step it calls,
    /// and both land on the standard error of the child.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prints(needle: string): boolean { return this.stdout.includes(needle) || this.stderr.includes(needle) }
    /// ```
    fn prints(
        &self,
        needle: &str,
    ) -> bool {
        return self.stdout.contains(needle) || self.stderr.contains(needle);
    }
}

/// What:
/// `fn run_one(fixture: &Fixture, name: &str, frozen: bool) -> Ran` makes one next run in a child of this test executable.
/// Why:
/// A run started in this process would print into this process's standard error,
/// where no test can read it,
/// so each run is a child whose streams the parent reads back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runOne(fixture: Fixture, name: string, frozen: boolean): Ran
/// ```
fn run_one(
    fixture: &Fixture,
    name: &str,
    frozen: bool,
) -> Ran {
    // What:     `current_exe()` is this test executable's own path.
    // Why:      The child re-enters this same binary and runs the one helper below by name.
    let executable: PathBuf = std::env::current_exe().expect("this test executable's own path");
    // What:     `path` is the child's `PATH`: the stand-in cargo's directory first, then this process's own.
    // Why:      The shadow picks the first `cargo` it finds, and the rest of `PATH` keeps `systemd-run` and `systemctl`.
    let path: String = format!(
        "{}:{}",
        fixture.programs.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let code_file: PathBuf = fixture.root.path().join("next-run-exit-code");
    let _ = fs::remove_file(&code_file);
    let mut command: Command = Command::new(executable);
    command
        .args([
            "--ignored",
            "--exact",
            "refusal_journal_next_runs_tests::one_next_run_fixture",
            "--nocapture",
        ])
        .current_dir(fixture.start())
        .env(FIXTURE_ROOT, fixture.root.path())
        .env(NEXT_RUN, name)
        .env("FAKE_CARGO_LOG", &fixture.log)
        .env("PATH", path);
    if frozen {
        command.env("CI", "true");
    }
    let output: Output = command.output().expect("the child run starts");
    // What:     `wrote` is the status the helper wrote when it still had a process of its own to write it from.
    // Why:      An ordinary command hands the child's process over to cargo (`command.exec`), so the helper never returns and
    //           the status of the child process itself is the run's; a refusal and an update return, and their status is the
    //           file.
    let code: i32 = match fs::read_to_string(&code_file) {
        Ok(found) => found
            .trim()
            .parse::<i32>()
            .expect("the child run's status is a number"),
        Err(_) => output.status.code().unwrap_or(1),
    };
    return Ran {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        code,
    };
}

/// The helper process:
/// it makes the one run its parent names, in the fixture its parent built,
/// and writes the status it answered.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of the next-run controls"]
fn one_next_run_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let name: String = std::env::var(NEXT_RUN).expect("the run this child makes");
    let root: PathBuf = PathBuf::from(root_text);
    let check: Vec<OsString> = ["check", "--workspace", "-q"]
        .iter()
        .map(|word| {
            return OsString::from(word);
        })
        .collect();
    let code: i32 = match name.as_str() {
        // The refusal: the stop reports at once and the listing finds the scope's directory gone, which is the arm whose
        // sentence names the kept journal.
        // The fault is armed at the second start of a contained program, so the first one (the install step's `--locked`
        // staleness check) runs its own stop and the second one (the update that writes the lockfile) is the refused run:
        // the lockfile cargo wrote is then the leftover the next run decides about.
        "refusal" => {
            let starts: Rc<Cell<u32>> = Rc::new(Cell::new(0));
            let seen: Rc<Cell<u32>> = Rc::clone(&starts);
            crate::bounded_run::on_start_for_test(Some(Rc::new(move || {
                let now: u32 = seen.get() + 1;
                seen.set(now);
                if now == 2 {
                    arm(Point::ContainmentStopStuck);
                    arm(Point::ContainmentScopeGoneAtListing);
                }
            })));
            crate::shim::run_shim(check)
        }
        "ordinary" => crate::shim::run_shim(check),
        // A command that resolves nothing: `cargo fmt` never enters the install step.
        "pass_through" => {
            crate::shim::run_shim(vec![OsString::from("fmt"), OsString::from("--check")])
        }
        "frozen" => crate::shim::run_shim(check),
        "update" => {
            let catalog: PathBuf = root.join("rust-catalog.toml");
            let crate_dirs: Vec<String> = vec![CRATE_DIR.to_string()];
            let runner = FakeCargo {
                calls: RefCell::new(Vec::new()),
            };
            // What:     `request` is the update's own request: the fixture's root, the one crate, and the fakes.
            // Why:      `update_rust_lockfiles` settles a kept journal before it resolves anything, which is the claim here.
            let request: UpdateRequest = UpdateRequest {
                root: root.as_path(),
                crate_dirs: &crate_dirs,
                dry_run: false,
                now: 0,
                settings: &settings(),
                catalog_path: &catalog,
                runner: &runner,
                registry: &EmptyRegistry,
                forward: &[],
                syncer: &NoSync,
                prune_catalog: false,
                lock_base: None,
            };
            match update_rust_lockfiles(&request, &UpdateKind::Full) {
                Ok(_report) => 0,
                Err(error) => {
                    eprintln!("cargo-shadow: the update of this fixture did not finish: {error}");
                    1
                }
            }
        }
        other => panic!("the parent asked for a next run this helper does not make: {other}"),
    };
    fs::write(root.join("next-run-exit-code"), code.to_string()).expect("write the run's status");
}

/// What:
/// `fn staged_a_refusal(fixture: &Fixture) -> Ran` makes the refusal and leaves its journal in the fixture.
/// Why:
/// Every leg starts from the same state,
/// and the refusal is the one this file's sentences are about.
fn staged_a_refusal(fixture: &Fixture) -> Ran {
    let refused: Ran = run_one(fixture, "refusal", false);
    assert_eq!(
        fixture.journals().len(),
        1,
        "the refusal kept no journal, so this control shows nothing: {}",
        refused.stderr
    );
    assert!(
        refused.prints("could not be stopped"),
        "the run was not refused for a scope it could not show empty: {}",
        refused.stderr
    );
    assert!(
        refused.prints("its directory was gone when its members were read"),
        "the refusal did not come from the arm whose sentence names the journal, so this control shows nothing: {}",
        refused.stderr
    );
    // The writer the stand-in cargo wrote leaves the scope by itself; the next run reads the recorded scope as gone.
    let scope: PathBuf =
        scope_directory_of(&fixture.journals()[0].join("groups").display().to_string());
    wait_until("the refusal's scope to be gone", &|| {
        return !scope.exists()
            || fs::read_to_string(scope.join("cgroup.events")).is_ok_and(|text| {
                return text.contains("populated 0");
            });
    });
    return refused;
}

/// The control of the ordinary next run:
/// a command that resolves dependencies settles the kept journal,
/// keeps the leftover lockfile its new versions pass the policy for,
/// and says both.
#[test]
fn an_ordinary_run_after_a_refusal_settles_the_kept_journal_and_says_what_it_kept() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "ordinary", false);

    assert_eq!(
        next.code, 0,
        "the next run did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the next run left the journal it should have settled: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the next run did not say what became of the leftover lockfile: {}",
        next.stderr
    );
}

/// The control of a command that resolves nothing:
/// `cargo fmt --check` never enters the install step,
/// so the journal stays and nothing is said about it.
#[test]
fn a_pass_through_after_a_refusal_leaves_the_kept_journal_and_says_nothing() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "pass_through", false);

    assert_eq!(
        next.code, 0,
        "the pass-through did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        next.stdout.contains("fake-cargo-ran fmt --check"),
        "the command did not reach cargo, so this control shows nothing: {}\n{}",
        next.stdout,
        next.stderr
    );
    assert_eq!(
        fixture.journals().len(),
        1,
        "the pass-through settled the journal, so the refusal must not say that a run which resolves nothing leaves it: \
         {:?}",
        fixture.journals()
    );
    assert!(
        !next.prints("journal"),
        "the pass-through said something about a journal it left: {}\n{}",
        next.stdout,
        next.stderr
    );
}

/// The control of a frozen run:
/// `CI=true` makes the install step return before it reads a journal,
/// so the journal stays and nothing is said about it.
#[test]
fn a_frozen_run_after_a_refusal_leaves_the_kept_journal_and_says_nothing() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "frozen", true);

    assert_eq!(
        next.code, 0,
        "the frozen run did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        next.stdout.contains("fake-cargo-ran check --workspace -q"),
        "the frozen run did not reach cargo, so this control shows nothing: {}\n{}",
        next.stdout,
        next.stderr
    );
    assert_eq!(
        fixture.journals().len(),
        1,
        "the frozen run settled the journal, so the refusal must not say that a frozen run leaves it: {:?}",
        fixture.journals()
    );
    assert!(
        !next.prints("journal"),
        "the frozen run said something about a journal it left: {}\n{}",
        next.stdout,
        next.stderr
    );
}

/// The control of `cargo-shadow update`:
/// its own recovery reads the kept journal before it resolves anything,
/// so the journal is settled and the run says what became of the kept lockfile.
#[test]
fn an_update_after_a_refusal_settles_the_kept_journal() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "update", false);

    assert_eq!(
        next.code, 0,
        "the update did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the update left the journal it should have settled: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the update did not say what became of the leftover lockfile: {}",
        next.stderr
    );
}
