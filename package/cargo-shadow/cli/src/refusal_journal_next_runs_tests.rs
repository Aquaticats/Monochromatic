//! What:
//! The journals a refusal keeps,
//! and what each kind of next run of the same workspace does with one:
//! an ordinary run that resolves dependencies,
//! `cargo metadata`,
//! `cargo update`,
//! `cargo-shadow install` and `cargo-shadow update`,
//! both `--dry-run` forms,
//! a command that resolves nothing,
//! a frozen run,
//! a frozen install,
//! a run outside every catalog repository,
//! and a run whose lockfile and remembered pass are fresh (the fast path).
//! Why:
//! The refusal of a run that could not be settled says what happens to the journal it kept,
//! and the sentence has to name the runs that settle it as well as the runs that leave it,
//! because only two call sites of this crate read a kept journal (`src/install.rs` and `src/update.rs`)
//! and the rest of the commands reach neither.
//! Every kind measured here was first seen on a kept journal a killed wrapper left
//! (`/home/user/temp/agent/cargo-shadow-alpha-15-release-review-2026-11-27/keep/evidence/10-next-runs`),
//! and each leg here starts from a refusal instead, so what a refusal leaves is measured rather than assumed.
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
/// The catalog commands themselves, so a leg can make the run a person types (`cargo-shadow install`, `update`).
/// Why: `install` and `update` are the two call sites that read a kept journal, and the ones the refusal names.
/// Gotcha: `std::process::Command` is the spawned process, so this one is named for the catalog grammar it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command as CatalogCommand } from './cli.ts';
/// ```
use crate::cli::Command as CatalogCommand;
/// The catalog marker file, so the leg that needs a directory outside every catalog repository can prove it has one.
/// Why: a directory that did have the marker above it would settle a journal and the leg would show nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CATALOG_MARKER } from './constants.ts';
/// ```
use crate::constants::CATALOG_MARKER;
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
/// A second crate of the same catalog repository, whose lockfile already satisfies its manifests.
/// Why: a run in a crate whose stamp and remembered pass are fresh takes the fast path without reading the journal.
const OTHER_CRATE_DIR: &str = "package/x/z";
/// A lockfile that does not satisfy its manifests.
const STALE: &str = "# stale\nversion = 4\n";
/// A lockfile that satisfies its manifests.
const SATISFIED: &str = "# satisfied\nversion = 4\n";
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
        // What:     `other_root` is the second crate root, `package/x/z`, whose lockfile already satisfies its manifests.
        // Why:      The fast path is reachable only beside a lockfile and a remembered policy pass that are both fresh,
        //           which a run in this crate builds; the refused crate's own stamp is never written.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const otherRoot = join(root.path(), 'package/x/z');
        // ```
        let other_root: PathBuf = root.path().join(OTHER_CRATE_DIR);
        fs::create_dir_all(other_root.join("src")).expect("make the second crate directory");
        fs::write(
            other_root.join("Cargo.toml"),
            "[package]\nname = \"z\"\nversion = \"0.1.0\"\n\n[workspace]\n",
        )
        .expect("write the second manifest");
        fs::write(other_root.join("Cargo.lock"), SATISFIED).expect("write the second lockfile");
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
    /// `fn start_in(&self, crate_dir: &str) -> PathBuf` is one crate root of the fixture's catalog repository.
    /// Why:
    /// A run on the fast path is a run in the other crate,
    /// so the leg has to name a working directory and not only a run kind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// startIn(crateDir: string): string { return join(this.root.path(), crateDir) }
    /// ```
    fn start_in(
        &self,
        crate_dir: &str,
    ) -> PathBuf {
        return self.root.path().join(crate_dir);
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
/// `fn run_one(fixture: &Fixture, name: &str, frozen: bool) -> Ran` makes one next run in the refused crate.
/// Why:
/// Most legs run where the refusal ran; only the fast-path leg runs in the other crate.
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
    return run_one_in(fixture, name, &fixture.start(), frozen);
}

/// What:
/// `fn run_one_in(fixture: &Fixture, name: &str, working: &Path, frozen: bool) -> Ran` makes one next run in a child of this
/// test executable,
/// in the working directory it is given.
/// Why:
/// A run started in this process would print into this process's standard error,
/// where no test can read it,
/// so each run is a child whose streams the parent reads back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runOneIn(fixture: Fixture, name: string, working: string, frozen: boolean): Ran
/// ```
fn run_one_in(
    fixture: &Fixture,
    name: &str,
    working: &Path,
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
        .current_dir(working)
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
        // `cargo metadata` resolves dependencies, so it enters the install step and reads the kept journal first.
        // What:     `metadata` is the argument list a cargo call for the workspace's package list carries.
        // Why:      It is the one resolving call that writes nothing, so its settling of the journal cannot be confused with a
        //           lockfile it changed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const metadata = ['metadata', '--format-version', '1'];
        // ```
        "metadata" => {
            let words: Vec<OsString> = ["metadata", "--format-version", "1"]
                .iter()
                .map(|word| return OsString::from(word))
                .collect();
            crate::shim::run_shim(words)
        }
        // `cargo update` is the catalog update command behind the shim, which reads the kept journal in its own recovery.
        "cargo_update" => crate::shim::run_shim(vec![OsString::from("update")]),
        // `cargo-shadow install`: the CLI command a person types, over every crate root of the catalog repository.
        "install" => crate::commands::run_command(CatalogCommand::Install {
            frozen_lockfile: false,
            no_frozen_lockfile: false,
            dry_run: false,
        }),
        // `cargo-shadow install --dry-run`: the same recovery, then a report that puts every lockfile back.
        "install_dry_run" => crate::commands::run_command(CatalogCommand::Install {
            frozen_lockfile: false,
            no_frozen_lockfile: false,
            dry_run: true,
        }),
        // `cargo-shadow install --frozen-lockfile`: a frozen install returns before the recovery, so it reads no journal.
        "install_frozen" => crate::commands::run_command(CatalogCommand::Install {
            frozen_lockfile: true,
            no_frozen_lockfile: false,
            dry_run: false,
        }),
        // `cargo-shadow update --dry-run`: the update's recovery, then a report that puts every lockfile back.
        "update_dry_run" => crate::commands::run_command(CatalogCommand::Update {
            dry_run: true,
        }),
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
    // What:     `fs::read_to_string(fixture.start().join("Cargo.lock"))` is the lockfile the refused run's cargo wrote and the refusal
    //           neither rolled back nor accepted.
    // Why:      The leftover is what every later run decides about, so a leg that found no leftover would be judging a different state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const leftover = readFileSync(join(start, 'Cargo.lock'), 'utf8');
    // ```
    let leftover: String =
        fs::read_to_string(fixture.start().join("Cargo.lock")).unwrap_or_default();
    assert_eq!(
        leftover, SATISFIED,
        "the refusal left no lockfile of cargo's for a later run to decide about: {}",
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
        !mentions_a_kept_journal(&next),
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
        !mentions_a_kept_journal(&next),
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

/// The words a run prints when it settles a kept journal, names one, or prints the command that acknowledges one.
/// Why:
/// The child re-enters this test binary,
/// so its own harness lines name `refusal_journal_next_runs_tests`;
/// a needle that was only the word `journal` would match them,
/// and a control that says "nothing was said about the journal" has to look for the words a person would read.
const JOURNAL_WORDS: [&str; 4] = [
    "journal of this run was kept",
    "kept journal",
    "cargo-shadow acknowledge",
    "was left by a run that",
];

/// What:
/// `fn mentions_a_kept_journal(next: &Ran) -> bool` says whether the run said anything about a kept journal.
/// Why:
/// A run that leaves a kept journal is promised to print nothing about it,
/// and the promise is about these words and not about any line that happens to hold the word "journal".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mentionsAKeptJournal(next: Ran): boolean
/// ```
fn mentions_a_kept_journal(next: &Ran) -> bool {
    return JOURNAL_WORDS
        .iter()
        .any(|needle| return next.prints(needle));
}

/// What:
/// `fn cargo_calls(fixture: &Fixture) -> Vec<String>` is every call the stand-in cargo was given so far.
/// Why:
/// A run on the fast path makes no cargo call at all,
/// so the log growing is what proves the leg took the slow path and read the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cargoCalls(fixture: Fixture): string[]
/// ```
fn cargo_calls(fixture: &Fixture) -> Vec<String> {
    // What:     `fs::read_to_string(&fixture.log)` is the whole log; `unwrap_or_default()` reads a log that does not exist yet as empty.
    // Why:      A leg that makes its first call creates the log, so a missing log is "no calls", not a failure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = readFileSync(log, 'utf8') ?? '';
    // ```
    let text: String = fs::read_to_string(&fixture.log).unwrap_or_default();
    // What:     `.lines()` walks the text line by line; `.map(str::to_string)` copies each into an owned `String`.
    // Why:      The count and the diff are what the assertions read, so the lines must outlive the borrowed text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lines = text.split('\n').filter((line) => line !== '');
    // ```
    return text.lines().map(str::to_string).collect();
}

/// What:
/// `fn outside_every_catalog_repository() -> tempfile::TempDir` is an empty directory whose ancestors hold no `rust-catalog.toml`.
/// Why:
/// A run outside any catalog repository leaves a kept journal alone because it never reads one,
/// and every directory under this build output has this repository's own `rust-catalog.toml` above it,
/// so the session's runtime directory is the only place that can stand for one without `/tmp`,
/// which this repository bans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outsideEveryCatalogRepository(): TempDir
/// ```
fn outside_every_catalog_repository() -> tempfile::TempDir {
    // What:     `XDG_RUNTIME_DIR` is the session's own private directory on a memory file system.
    // Why:      It is the one writable place this account owns whose ancestors are not inside a catalog repository.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = process.env.XDG_RUNTIME_DIR;
    // ```
    let runtime: PathBuf = match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(found) => PathBuf::from(found),
        None => panic!(
            "this host sets no XDG_RUNTIME_DIR, and every directory under the build output has this repository's rust-catalog.toml above it, so a run outside every catalog repository cannot be made here without /tmp, which this repository bans"
        ),
    };
    // What:     `tempfile::tempdir_in(&runtime)` makes an empty directory inside the runtime directory and removes it when the guard drops.
    // Why:      The leg must not leave a directory behind, however the test ends.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const made = mkdtempSync(join(runtime, 'dir-'));
    // ```
    let made: tempfile::TempDir =
        tempfile::tempdir_in(&runtime).expect("make a directory outside every catalog repository");
    // What:     `.ancestors()` walks the new directory and every directory above it; `.any(...)` is true when one holds the catalog marker.
    // Why:      This is the positive control of the leg: a directory that did have a catalog repository above it would settle a journal
    //           and the leg would show nothing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inside = ancestors(made.path()).some((dir) => existsSync(join(dir, 'rust-catalog.toml')));
    // ```
    let inside: bool = made
        .path()
        .ancestors()
        .any(|directory| return directory.join(CATALOG_MARKER).is_file());
    assert!(
        !inside,
        "{} has a catalog repository above it, so it is not outside every catalog repository: {:?}",
        made.path().display(),
        made.path().ancestors().collect::<Vec<&Path>>()
    );
    return made;
}

/// The control of a plain `cargo-shadow install`:
/// its recovery reads every kept journal of the catalog root before it checks anything,
/// so the journal is settled and the run says what became of the leftover lockfile.
#[test]
fn an_install_after_a_refusal_settles_the_kept_journal() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "install", false);

    assert_eq!(
        next.code, 0,
        "the install did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the install left the journal it should have settled: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the install did not say what became of the leftover lockfile: {}",
        next.stderr
    );
}

/// The control of `cargo-shadow install --dry-run`:
/// it settles a kept journal before it looks at anything,
/// and then puts back the lockfile it snapshotted,
/// which is the leftover the recovery kept.
#[test]
fn an_install_dry_run_after_a_refusal_settles_the_kept_journal_and_leaves_the_kept_leftover() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "install_dry_run", false);

    assert_eq!(
        next.code, 0,
        "the install --dry-run did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the install --dry-run left the journal it settled before it looked: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the install --dry-run did not say what became of the leftover lockfile: {}",
        next.stderr
    );
    // What:     `fs::read_to_string(...)` is the lockfile the dry run restored; the dry run's own restore puts back the text it snapshotted.
    // Why:      A person who runs a dry run to look expects nothing to change, and what the recovery kept is still there afterwards.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lock = readFileSync(join(start, 'Cargo.lock'), 'utf8');
    // ```
    let lock: String = fs::read_to_string(fixture.start().join("Cargo.lock")).unwrap_or_default();
    assert_eq!(
        lock, SATISFIED,
        "the install --dry-run did not leave the leftover the recovery kept: {lock:?}"
    );
}

/// The control of `cargo-shadow update --dry-run`:
/// the update's recovery settles the kept journal before it resolves anything.
#[test]
fn an_update_dry_run_after_a_refusal_settles_the_kept_journal() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "update_dry_run", false);

    assert_eq!(
        next.code, 0,
        "the update --dry-run did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the update --dry-run left the journal it settled before it looked: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the update --dry-run did not say what became of the leftover lockfile: {}",
        next.stderr
    );
}

/// The control of `cargo metadata`:
/// it resolves dependencies, so it enters the install step and settles the kept journal.
#[test]
fn a_cargo_metadata_run_after_a_refusal_settles_the_kept_journal() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "metadata", false);

    assert_eq!(
        next.code, 0,
        "the cargo metadata run did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the cargo metadata run left the journal it should have settled: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the cargo metadata run did not say what became of the leftover lockfile: {}",
        next.stderr
    );
}

/// The control of `cargo update`:
/// the catalog update command reads the kept journal in its own recovery.
#[test]
fn a_cargo_update_run_after_a_refusal_settles_the_kept_journal() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "cargo_update", false);

    assert_eq!(
        next.code, 0,
        "the cargo update run did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the cargo update run left the journal it should have settled: {:?}",
        fixture.journals()
    );
    assert!(
        next.prints("was left by a run that was killed"),
        "the cargo update run did not say what became of the leftover lockfile: {}",
        next.stderr
    );
}

/// The control of a frozen install:
/// `--frozen-lockfile` returns before the recovery,
/// so the journal stays and nothing is said about it.
#[test]
fn a_frozen_install_after_a_refusal_leaves_the_kept_journal_and_says_nothing() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);

    let next: Ran = run_one(&fixture, "install_frozen", false);

    assert_eq!(
        next.code, 0,
        "the frozen install did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    // What:     `cargo_calls` is the calls the stand-in cargo answered, which the frozen check made before it returned.
    // Why:      A frozen install that made no check could leave the journal for another reason than the one the control names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const calls = cargoCalls(fixture);
    // ```
    let calls: Vec<String> = cargo_calls(&fixture);
    assert!(
        !calls.is_empty(),
        "the frozen install asked cargo nothing, so this control shows nothing: {}\n{}",
        next.stdout,
        next.stderr
    );
    assert_eq!(
        fixture.journals().len(),
        1,
        "the frozen install settled the journal, so the refusal must not say that a frozen run leaves it: {:?}",
        fixture.journals()
    );
    assert!(
        !mentions_a_kept_journal(&next),
        "the frozen install said something about a journal it left: {}\n{}",
        next.stdout,
        next.stderr
    );
}

/// The control of a run outside every catalog repository:
/// it reads no journal of this catalog root,
/// so the journal stays and nothing is said about it.
#[test]
fn a_run_outside_every_catalog_repository_after_a_refusal_leaves_the_kept_journal_and_says_nothing()
{
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let _refused: Ran = staged_a_refusal(&fixture);
    let outside: tempfile::TempDir = outside_every_catalog_repository();

    let next: Ran = run_one_in(&fixture, "ordinary", outside.path(), false);

    assert_eq!(
        next.code, 0,
        "the run outside every catalog repository did not finish: {}\n{}",
        next.stdout, next.stderr
    );
    assert!(
        next.stdout.contains("fake-cargo-ran check"),
        "the run did not reach cargo, so this control shows nothing: {}\n{}",
        next.stdout,
        next.stderr
    );
    assert_eq!(
        fixture.journals().len(),
        1,
        "a run outside every catalog repository settled a journal it never read: {:?}",
        fixture.journals()
    );
    assert!(
        !mentions_a_kept_journal(&next),
        "the run outside every catalog repository said something about a journal it left: {}\n{}",
        next.stdout,
        next.stderr
    );
}

/// The control of the fast path:
/// a run in the other crate of the same repository,
/// whose stamp and remembered policy pass are fresh,
/// returns before the recovery and makes no cargo call,
/// so the journal stays and nothing is said about it.
#[test]
fn a_run_on_the_fast_path_after_a_refusal_leaves_the_kept_journal_and_says_nothing() {
    crate::cgroup_test_support::this_test_needs_a_delegated_scope();
    let fixture = Fixture::made();
    let other: PathBuf = fixture.start_in(OTHER_CRATE_DIR);
    // The stamp and the remembered pass come from a run of that crate before the refusal leaves a journal to read.
    let warmed: Ran = run_one_in(&fixture, "ordinary", &other, false);
    assert_eq!(
        warmed.code, 0,
        "the run that was to leave a fresh stamp did not finish: {}\n{}",
        warmed.stdout, warmed.stderr
    );
    assert!(
        fixture.journals().is_empty(),
        "the run that was to leave a fresh stamp left a journal of its own: {:?}",
        fixture.journals()
    );
    let _refused: Ran = staged_a_refusal(&fixture);
    let before: Vec<String> = cargo_calls(&fixture);

    let next: Ran = run_one_in(&fixture, "ordinary", &other, false);

    // What:     `cargo_calls` read again is the log after the run, and `added` is what the run appended to it.
    // Why:      The fast path makes none of the install step's calls (`update --locked`, `metadata`) and still runs the command the
    //           person typed, so the one call the log may gain is that command. Anything more is a slow run that read the journal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const after = cargoCalls(fixture);
    // const added = after.slice(before.length);
    // ```
    let after: Vec<String> = cargo_calls(&fixture);
    let added: &[String] = &after[before.len()..];
    assert!(
        added.len() == 1 && added[0] == "check --workspace -q",
        "the run made an install-step cargo call, so it did not take the fast path: {added:?}"
    );
    assert_eq!(
        fixture.journals().len(),
        1,
        "the run on the fast path settled the journal, so the refusal must not say that such a run leaves it: {:?}",
        fixture.journals()
    );
    assert!(
        !mentions_a_kept_journal(&next),
        "the run on the fast path said something about a journal it left: {}\n{}",
        next.stdout,
        next.stderr
    );
}
