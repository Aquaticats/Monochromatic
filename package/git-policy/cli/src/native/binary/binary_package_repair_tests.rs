//! What:
//!  The package-manager repair of a new worktree through the built executable:
//! after `git worktree add` the commit service starts pnpm and Cargo in the new worktree (fixture tools,
//! then the real ones),
//! never for `--no-checkout` or the opt-out,
//! and a
//!  tool that fails or is missing leaves the add successful,
//! with the add naming what it handed over and a missing tool's line naming the command to run.
//! Why:
//!  The owner's rule is that a new worktree works under every configuration of every package manager the repository uses.
//!  pnpm's links and Cargo's build state are wrong at a new location,
//! so the controls check the effect a person sees:
//! a
//!  `node_modules` entry that resolves,
//! a build that names the worktree's own paths,
//! and the source left as it was.
//!  Where no commit service answers,
//! the add runs the tools itself;
//! the lib controls of `repair` cover that arm,
//! because the fixture of these controls starts a service for every worktree it holds.
//!  Controls that need a real tool say so and stop with a loud line when it is missing,
//! as the btrfs controls do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // run(wrappedGit, ['worktree', 'add', ...]); expect(existsSync('../topic/node_modules/leaf/package.json')).toBe(true);
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::commit::remove_settled;
use super::support::{Fixture, Observed, SERVICE_WORK_PATIENCE, bounded, executable, fixture, git, link_real_git, observe, remove, repository, run_wrapped, settle_service_work, stderr_of, while_test_lives, wrapped};
use super::support::on_btrfs_or_acknowledged;
use super::worktree_snapshot::{commit_all, subvolume_repository};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The environment variable that acknowledges a real tool is absent on this machine,
/// so its controls may be skipped.
const ABSENT_ACKNOWLEDGED: &str = "CLI_GIT_REAL_TOOLS_ABSENT_ACKNOWLEDGED";

/// Whether a real tool is available;
/// where it is not,
/// the run must have said so,
/// and one loud line names the control.
fn available_or_acknowledged(found: Option<&Path>, tool: &str, control: &str) -> bool {
    if found.is_some() {
        return true;
    }
    let acknowledged: bool = std::env::var_os(ABSENT_ACKNOWLEDGED).is_some_and(|value| { return !value.is_empty(); });
    assert!(acknowledged, "the control {control} needs a real {tool}, which is not on PATH; install it, or set {ABSENT_ACKNOWLEDGED}=1");
    eprintln!("REAL {} CONTROL NOT RUN: {control} (no {tool} on PATH)", tool.to_uppercase());
    return false;
}

/// Words of the line that reports a repair that worked.
const REPAIRED_WORDS: &str = "work at its new location";

/// Words of the line that reports a repair that could not finish.
const PENDING_WORDS: &str = "still needs one step";

/// Where the fixture's tools are linked or written:
/// the first directory of the wrapped PATH.
fn bin(fixture: &Fixture) -> PathBuf {
    return fixture.root.join("bin");
}

/// A fixture `pnpm` that appends `arguments | directory` to a record and exits with `status`.
fn fake_pnpm(fixture: &Fixture, status: i32) -> PathBuf {
    let record: PathBuf = fixture.root.join("pnpm-record");
    executable(
        bin(fixture).join("pnpm").as_path(),
        format!("#!/bin/sh\necho \"$* | $(pwd -P)\" >> '{}'\nexit {status}\n", record.display()).as_bytes(),
    );
    return record;
}

/// A repository whose tracked files hold a lockfile and whose ignored directory holds a `node_modules` to copy.
fn repository_with_node_modules(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join(".gitignore"), b"node_modules/\n").expect("gitignore");
    std::fs::write(repo.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "pnpm-lock.yaml"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=lockfile"]);
    std::fs::create_dir_all(repo.join("node_modules/dep")).expect("node_modules");
    std::fs::write(repo.join("node_modules/dep/index.js"), b"module.exports = 1;\n").expect("dependency");
    return repo;
}

/// The lines of a record file,
/// empty when nothing was recorded.
/// The commit service runs the tools of a new worktree,
/// so a record read here is read after every observed wrapped command waited for the service's after-commit work to settle
/// (`supervised_wait`):
/// an add returns after the record and the wake,
/// and the line the tool wrote is there once that wait ends.
fn recorded(record: &Path) -> Vec<String> {
    return std::fs::read_to_string(record).map(|text: String| { return text.lines().map(str::to_owned).collect(); }).unwrap_or_default();
}

/// Add a worktree from the repository and require success.
fn add(fixture: &Fixture, repo: &Path, options: &[&str], branch: &str, destination: &str) -> Observed {
    let mut arguments: Vec<&str> = vec!["worktree", "add"];
    arguments.extend_from_slice(options);
    arguments.extend_from_slice(&["-b", branch, destination]);
    let created: Observed = run_wrapped(fixture, repo, arguments.as_slice());
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    return created;
}

/// pnpm is started in the new worktree,
/// and only there,
/// with the offline and frozen-lockfile switches;
/// the source is not a
///  directory it runs in.
/// The commit service starts it once the add has returned,
/// so the record below is read after this harness waited for that work to settle.
#[test]
fn pnpm_runs_in_the_new_worktree_only() {
    let fixture: Fixture = fixture("repair-pnpm-runs");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fake_pnpm(&fixture, 0);
    let created: Observed = add(&fixture, repo.as_path(), &[], "one", "../one");
    let lines: Vec<String> = recorded(record.as_path());
    assert_eq!(lines.len(), 1, "{lines:?} {}", stderr_of(&created));
    let one: PathBuf = fixture.root.join("one");
    assert!(lines[0].ends_with(format!("| {}", one.display()).as_str()), "{lines:?}");
    assert!(lines[0].contains("--offline") && lines[0].contains("--frozen-lockfile"), "{lines:?}");
    assert!(stderr_of(&created).contains(REPAIRED_WORDS), "{}", stderr_of(&created));
    remove(&fixture);
}

/// The same add through the snapshot rung,
/// which carries the ignored state itself,
/// still starts pnpm in the new worktree.
#[test]
fn pnpm_runs_after_the_snapshot_rung() {
    let fixture: Fixture = fixture("repair-pnpm-snapshot");
    if !on_btrfs_or_acknowledged(&fixture, "pnpm_runs_after_the_snapshot_rung") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = subvolume_repository(&fixture);
    std::fs::write(repo.join(".gitignore"), b"ignored/\nnode_modules/\n").expect("gitignore");
    std::fs::write(repo.join("pnpm-lock.yaml"), b"lockfileVersion: '9.0'\n").expect("lockfile");
    commit_all(&fixture, repo.as_path(), "lockfile");
    std::fs::create_dir_all(repo.join("node_modules/dep")).expect("node_modules");
    std::fs::write(repo.join("node_modules/dep/index.js"), b"module.exports = 1;\n").expect("dependency");
    let record: PathBuf = fake_pnpm(&fixture, 0);
    add(&fixture, repo.as_path(), &[], "one", "../one");
    let lines: Vec<String> = recorded(record.as_path());
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].ends_with(format!("| {}", fixture.root.join("one").display()).as_str()), "{lines:?}");
    assert!(fixture.root.join("one/node_modules/dep/index.js").exists(), "the snapshot carried the dependency");
    remove(&fixture);
}

/// `--no-checkout` and the copy opt-out start nothing and print nothing about a repair.
#[test]
fn no_checkout_and_the_opt_out_start_nothing() {
    let fixture: Fixture = fixture("repair-pnpm-skipped");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fake_pnpm(&fixture, 0);
    let bare: Observed = add(&fixture, repo.as_path(), &["--no-checkout"], "one", "../one");
    let opted_out: Observed = add(&fixture, repo.as_path(), &["--no-worktree-copy"], "two", "../two");
    assert!(recorded(record.as_path()).is_empty(), "a tool was started: {:?}", recorded(record.as_path()));
    for printed in [stderr_of(&bare), stderr_of(&opted_out)] {
        assert!(!printed.contains(REPAIRED_WORDS) && !printed.contains(PENDING_WORDS), "{printed}");
    }
    remove(&fixture);
}

/// `--quiet` silences the report of a repair that worked and still runs it.
#[test]
fn quiet_runs_the_repair_and_says_nothing_about_success() {
    let fixture: Fixture = fixture("repair-pnpm-quiet");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fake_pnpm(&fixture, 0);
    let created: Observed = add(&fixture, repo.as_path(), &["--quiet"], "one", "../one");
    assert_eq!(recorded(record.as_path()).len(), 1);
    assert!(!stderr_of(&created).contains(REPAIRED_WORDS), "{}", stderr_of(&created));
    remove(&fixture);
}

/// A pnpm that fails leaves the add successful,
/// and the add names the tool it handed over rather than the tool's end:
/// the add returns after the record and the wake,
/// and the commit service runs pnpm there
/// (`doc/planning/cli-git-worktree-add-package-repair.md`,
/// "The pnpm relink and the toolchain install are the service's work"),
/// so the switches and the worktree are read from the run the service made,
/// and a tool that fails is the service's line,
/// not the add's.
#[test]
fn a_failing_pnpm_never_fails_the_add() {
    let fixture: Fixture = fixture("repair-pnpm-fails");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fake_pnpm(&fixture, 1);
    let created: Observed = add(&fixture, repo.as_path(), &[], "one", "../one");
    let printed: String = stderr_of(&created);
    let line: &str = printed.lines().find(|line: &&str| { return line.contains(REPAIRED_WORDS); }).expect("one line about the repair");
    assert!(line.contains("pnpm"), "{line}");
    assert!(line.contains(fixture.root.join("one").to_str().expect("utf8")), "{line}");
    assert!(fixture.root.join("one/node_modules/dep/index.js").exists(), "the copy is kept");
    let ran: Vec<String> = recorded(record.as_path());
    assert_eq!(ran.len(), 1, "pnpm ran once, in the new worktree: {ran:?}");
    assert!(ran[0].contains("--offline") && ran[0].contains("--frozen-lockfile"), "{ran:?}");
    assert!(ran[0].ends_with(format!("| {}", fixture.root.join("one").display()).as_str()), "{ran:?}");
    remove_settled(&fixture);
}

/// A missing pnpm is the same:
/// the add succeeds and the line says what to run.
///  The run's PATH holds the wrapper's directory and a directory with only real Git,
/// never the system directories,
/// because
///  an image whose package manager installed a real pnpm into `/usr/bin` would otherwise not lack one.
#[test]
fn a_missing_pnpm_never_fails_the_add() {
    let fixture: Fixture = fixture("repair-pnpm-missing");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    link_real_git(&fixture);
    let path: String = format!("{}:{}", bin(&fixture).display(), fixture.root.join("real").display());
    let created: Observed = observe(
        bounded(&fixture, bin(&fixture).join("git").as_path(), OsStr::new(path.as_str()))
            .current_dir(&repo)
            .args(["worktree", "add", "-b", "one", "../one"]),
        b"",
    );
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    let printed: String = stderr_of(&created);
    assert!(printed.contains("pnpm was not found") && printed.contains("--frozen-lockfile"), "{printed}");
    remove(&fixture);
}

/// How long the held repair of the first add looks between two glances at its release,
/// in milliseconds;
/// the loop that waits ends at the release,
/// at the end of the fixture or of this control process,
/// never at a count of its own.
const HOLD_PAUSE_MILLISECONDS: u32 = 50;

/// How long the control waits between two glances at whether the held repair has started.
const START_PAUSE: std::time::Duration = std::time::Duration::from_millis(100);

/// While the commit service runs one add's repair,
/// another add in the same repository is not held by it:
/// the add returns after the record and the wake,
/// the tools run in the service,
/// and they touch only their own new worktree.
/// Both adds are waited for here rather than through `run_wrapped`,
/// because that wait is for the service's after-commit work to settle,
/// and the held repair is exactly the work that has not settled yet.
/// The wait for the held repair to start ends at that start,
/// at the end of the fixture or at this process,
/// and it is bounded by the wait every control gives the service's after-commit work,
/// so a repair that never starts fails with its own words instead of hanging the run.
/// The held repair itself ends when this control writes its release file,
/// at the end of its fixture,
/// or with this process,
/// so a loaded host cannot end the hold before the second add returns and make the control report a lock
///  that is not there;
/// every wait of the control ends at an event of the programs it started.
#[test]
fn a_running_repair_does_not_hold_another_add() {
    let fixture: Fixture = fixture("repair-lock-free");
    let repo: PathBuf = repository_with_node_modules(&fixture);
    let record: PathBuf = fixture.root.join("pnpm-record");
    let started: PathBuf = fixture.root.join("held-started");
    let release: PathBuf = fixture.root.join("held-release");
    let pause: String = format!("0.{HOLD_PAUSE_MILLISECONDS:03}");
    executable(
        bin(&fixture).join("pnpm").as_path(),
        format!(
            "#!/bin/sh\necho \"$* | $(pwd -P)\" >> '{record}'\ncase \"$(pwd -P)\" in\n  */one) : > '{started}'; while [ ! -e '{release}' ] && {lives}; do sleep {pause}; done;;\nesac\nexit 0\n",
            record = record.display(),
            started = started.display(),
            release = release.display(),
            lives = while_test_lives(fixture.root.as_path()),
            pause = pause.as_str(),
        )
        .as_bytes(),
    );
    let first: std::process::Output = wrapped(&fixture).current_dir(&repo).args(["worktree", "add", "--quiet", "-b", "one", "../one"]).output().expect("the first add runs");
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(first.stderr.as_slice()));
    let began: std::time::Instant = std::time::Instant::now();
    while !started.exists() {
        assert!(
            began.elapsed() < SERVICE_WORK_PATIENCE,
            "the service has not started the relink of the first add after {} s; the tools it ran wrote {:?}",
            SERVICE_WORK_PATIENCE.as_secs(),
            recorded(record.as_path())
        );
        std::thread::sleep(START_PAUSE);
    }
    assert!(!release.exists());
    let second: std::process::Output = wrapped(&fixture).current_dir(&repo).args(["worktree", "add", "--quiet", "-b", "two", "../two"]).output().expect("the second add runs");
    assert_eq!(second.status.code(), Some(0), "{}", String::from_utf8_lossy(second.stderr.as_slice()));
    assert!(!release.exists(), "the first repair ended while the second add ran, so the control held nothing");
    assert!(fixture.root.join("two/node_modules/dep/index.js").exists(), "the second worktree is whole");
    std::fs::write(&release, b"go").expect("release the first repair");
    settle_service_work(repo.as_path());
    assert_eq!(recorded(record.as_path()).len(), 2, "each add was repaired in its own worktree: {:?}", recorded(record.as_path()));
    remove_settled(&fixture);
}

/// A real `pnpm` found by the test process,
/// linked into the fixture's PATH directory.
fn link_real(fixture: &Fixture, name: &str, target: &Path) {
    std::os::unix::fs::symlink(target, bin(fixture).join(name)).expect("link a real tool");
}

/// The first executable named `name` on the test process's PATH.
fn on_path(name: &str) -> Option<PathBuf> {
    let search: std::ffi::OsString = std::env::var_os("PATH")?;
    return std::env::split_paths(&search).map(|directory: PathBuf| { return directory.join(name); }).find(|candidate: &PathBuf| { return candidate.is_file(); });
}

/// The first `pnpm` on the test process's PATH that is pnpm itself and not the repository's shadow of it,
/// which looks for a
///  real pnpm further along the PATH and finds none in the fixture's.
fn real_pnpm() -> Option<PathBuf> {
    let search: std::ffi::OsString = std::env::var_os("PATH")?;
    return std::env::split_paths(&search)
        .map(|directory: PathBuf| { return directory.join("pnpm"); })
        .filter(|candidate: &PathBuf| { return candidate.is_file(); })
        .find(|candidate: &PathBuf| {
            let version = Command::new(candidate).arg("--version").stdin(std::process::Stdio::null()).output();
            return version.is_ok_and(|output: std::process::Output| { return output.status.success() && !String::from_utf8_lossy(&output.stdout).contains("shadow"); });
        });
}

/// Run a linked tool in a directory with the hermetic environment the wrapped runs use,
/// and require success.
fn run_tool(fixture: &Fixture, program: &str, directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new(bin(fixture).join(program))
        .env_clear()
        .env("PATH", format!("{}:/usr/bin:/bin", bin(fixture).display()))
        .env("HOME", fixture.root.join("home"))
        .current_dir(directory)
        .args(arguments)
        .output()
        .expect("run the tool");
    assert!(output.status.success(), "{program} {arguments:?} failed in {}: {}", directory.display(), String::from_utf8_lossy(&output.stderr));
    return String::from_utf8_lossy(&output.stdout).into_owned();
}

/// A workspace whose one dependency is a local tarball,
/// installed with the global virtual store outside the tree:
/// the
///  layout whose links climb out of the tree by a fixed number of levels.
fn pnpm_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::create_dir_all(fixture.root.join("home")).expect("home");
    let packed: PathBuf = fixture.root.join("leaf-source");
    std::fs::create_dir_all(packed.join("package")).expect("package directory");
    std::fs::write(packed.join("package/package.json"), b"{\"name\":\"leaf\",\"version\":\"1.0.0\"}\n").expect("leaf manifest");
    std::fs::create_dir_all(repo.join("vendor")).expect("vendor");
    let tar = Command::new("/usr/bin/tar").arg("--create").arg("--gzip").arg("--file").arg(repo.join("vendor/leaf.tgz")).arg("--directory").arg(&packed).arg("package").status().expect("tar");
    assert!(tar.success());
    std::fs::write(repo.join("package.json"), b"{\"name\":\"root\",\"private\":true,\"dependencies\":{\"leaf\":\"file:./vendor/leaf.tgz\"}}\n").expect("manifest");
    std::fs::write(repo.join("pnpm-workspace.yaml"), format!("storeDir: {}\nvirtualStoreType: global\n", fixture.root.join("store").display())).expect("workspace");
    std::fs::write(repo.join(".gitignore"), b"node_modules/\n.claude/\n").expect("gitignore");
    run_tool(fixture, "pnpm", repo.as_path(), &["install"]);
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "package.json", "pnpm-workspace.yaml", "pnpm-lock.yaml", "vendor/leaf.tgz"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=workspace"]);
    return repo;
}

/// With real pnpm:
/// a copy of the install at another depth has a dead entry,
/// and after the add the entry resolves,
/// both in a
///  worktree under the source and in one outside it,
/// with the source left as it was.
#[test]
fn real_pnpm_relinks_a_global_virtual_store_install() {
    let fixture: Fixture = fixture("repair-real-pnpm");
    let real: Option<PathBuf> = real_pnpm();
    if !available_or_acknowledged(real.as_deref(), "pnpm", "real_pnpm_relinks_a_global_virtual_store_install") {
        remove(&fixture);
        return;
    }
    link_real(&fixture, "pnpm", real.as_deref().expect("checked above"));
    let repo: PathBuf = pnpm_repository(&fixture);
    let entry = |root: &Path| -> PathBuf { return root.join("node_modules/leaf/package.json"); };
    assert!(entry(repo.as_path()).exists(), "the source install works");
    let before: Vec<PathBuf> = std::fs::read_dir(repo.join("node_modules")).expect("source modules").flatten().map(|item| { return item.path(); }).collect();
    // Positive control: the same tree copied verbatim to another depth has a dead entry, so the fixture reproduces the hazard.
    std::fs::create_dir_all(fixture.root.join("deeper/more")).expect("deeper");
    let control: PathBuf = fixture.root.join("deeper/more/control");
    std::fs::create_dir_all(&control).expect("control");
    let copied = Command::new("/usr/bin/cp").arg("--archive").arg(repo.join("node_modules")).arg(&control).status().expect("cp");
    assert!(copied.success());
    assert!(!entry(control.as_path()).exists(), "the verbatim copy should hold a dead link");
    let outside: Observed = add(&fixture, repo.as_path(), &["--quiet"], "outside", "../deeper/more/outside");
    let nested: Observed = add(&fixture, repo.as_path(), &["--quiet"], "nested", ".claude/worktrees/nested");
    for (label, root) in [("outside", fixture.root.join("deeper/more/outside")), ("nested", repo.join(".claude/worktrees/nested"))] {
        assert!(entry(root.as_path()).exists(), "{label}: the entry is dead after the add: {} {}", stderr_of(&outside), stderr_of(&nested));
        let resolved: PathBuf = std::fs::canonicalize(entry(root.as_path())).expect("resolved");
        assert!(!resolved.starts_with(repo.join("node_modules")), "{label}: resolves into the source's modules: {}", resolved.display());
    }
    let after: Vec<PathBuf> = std::fs::read_dir(repo.join("node_modules")).expect("source modules").flatten().map(|item| { return item.path(); }).collect();
    assert_eq!(before.len(), after.len(), "the source's node_modules changed");
    assert!(entry(repo.as_path()).exists(), "the source install still works");
    remove(&fixture);
}

/// The registry crate the Cargo fixture depends on,
/// so a build shows what is reused.
const REGISTRY_CRATE: &str = "itoa-1.0.18";

/// The person's Cargo home when its offline cache holds `REGISTRY_CRATE`,
/// which lets a fixture depend on a registry crate
///  without the network.
fn cargo_home_with_registry_crate() -> Option<PathBuf> {
    let home: PathBuf = std::env::var_os("CARGO_HOME").map(PathBuf::from).or_else(|| { return std::env::var_os("HOME").map(|home: std::ffi::OsString| { return PathBuf::from(home).join(".cargo"); }); })?;
    let indexes = std::fs::read_dir(home.join("registry/cache")).ok()?;
    let found: bool = indexes.flatten().any(|index| { return index.path().join(format!("{REGISTRY_CRATE}.crate")).is_file(); });
    return found.then_some(home);
}

/// A Cargo workspace of two path-dependent crates,
/// built once in the source.
/// `external` keys the build directory by workspace
///  path outside the tree,
/// as the person's `~/.cargo/config.toml` does;
/// otherwise Cargo's default layout applies.
fn cargo_repository(fixture: &Fixture, external: bool) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::create_dir_all(fixture.root.join("home")).expect("home");
    for directory in ["lib/src", "app/src", ".cargo"] {
        std::fs::create_dir_all(repo.join(directory)).expect("directory");
    }
    std::fs::write(repo.join("Cargo.toml"), b"[workspace]\nmembers = [\"app\", \"lib\"]\nresolver = \"3\"\n").expect("workspace");
    let registry: bool = cargo_home_with_registry_crate().is_some();
    let dependency: &str = if registry { "\n[dependencies]\nitoa = \"=1.0.18\"\n" } else { "" };
    std::fs::write(repo.join("lib/Cargo.toml"), format!("[package]\nname = \"lib\"\nversion = \"0.1.0\"\nedition = \"2024\"\n{dependency}")).expect("lib manifest");
    let uses: &str = if registry { "pub fn render(number: u32) -> String { return itoa::Buffer::new().format(number).to_owned(); }\n" } else { "" };
    std::fs::write(repo.join("lib/src/lib.rs"), format!("pub const MANIFEST: &str = env!(\"CARGO_MANIFEST_DIR\");\n{uses}")).expect("lib source");
    std::fs::write(repo.join("app/Cargo.toml"), b"[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nlib = { path = \"../lib\" }\n").expect("app manifest");
    std::fs::write(repo.join("app/src/main.rs"), b"fn main() { println!(\"{}\", lib::MANIFEST); }\n").expect("app source");
    let layout: String = if external { format!("[build]\nbuild-dir = \"{}/{{workspace-path-hash}}\"\n", fixture.root.join("cargo-build").display()) } else { String::new() };
    std::fs::write(repo.join(".cargo/config.toml"), layout).expect("cargo config");
    std::fs::write(repo.join(".gitignore"), b"target/\n.claude/\n").expect("gitignore");
    run_tool(fixture, "cargo", repo.as_path(), &["build", "--offline"]);
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", ".cargo/config.toml", "Cargo.toml", "Cargo.lock", "lib", "app"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=workspace"]);
    return repo;
}

/// The crates Cargo compiled (not found fresh) in one build of a workspace.
fn compiled(fixture: &Fixture, root: &Path) -> Vec<String> {
    let output: String = run_tool(fixture, "cargo", root, &["build", "--offline", "--message-format=json"]);
    let mut names: Vec<String> = output
        .lines()
        .filter(|line: &&str| { return line.starts_with("{\"reason\":\"compiler-artifact\""); })
        .filter_map(|line: &str| { return serde_json::from_str::<serde_json::Value>(line).ok(); })
        .filter(|artifact: &serde_json::Value| { return artifact["fresh"] == false; })
        .filter_map(|artifact: serde_json::Value| { return artifact["target"]["name"].as_str().map(str::to_owned); })
        .collect();
    names.sort();
    return names;
}

/// Link the person's Rust toolchain into the fixture's PATH directory and share only the registry of their Cargo home,
/// or say
///  loudly why the controls cannot run.
fn prepare_cargo(fixture: &Fixture, name: &str) -> bool {
    let compiler: Option<PathBuf> = on_path("rustc");
    if !available_or_acknowledged(compiler.as_deref(), "cargo", name) {
        return false;
    }
    let sysroot: String = String::from_utf8_lossy(&Command::new("rustc").arg("--print").arg("sysroot").output().expect("rustc").stdout).trim().to_owned();
    for tool in ["cargo", "rustc", "rustdoc"] {
        link_real(fixture, tool, Path::new(sysroot.as_str()).join("bin").join(tool).as_path());
    }
    // The registry crate comes from the person's offline cache, reached through the fixture's home.
    if let Some(home) = cargo_home_with_registry_crate() {
        // Only the registry is shared: the person's Cargo configuration (which keys the build directory) must not apply.
        std::fs::create_dir_all(fixture.root.join("home/.cargo")).expect("fixture cargo home");
        std::os::unix::fs::symlink(home.join("registry"), fixture.root.join("home/.cargo/registry")).expect("link the registry");
    } else {
        eprintln!("REGISTRY REUSE NOT EXERCISED: {name} (no {REGISTRY_CRATE} in the offline Cargo cache)");
    }
    return true;
}

/// With real Cargo,
/// in each layout and in a worktree under the source and one outside it:
/// the worktree's crates rebuild with
///  the worktree's own paths,
/// a second build compiles nothing,
/// an edit in the worktree rebuilds,
/// and the source is untouched.
fn cargo_control(name: &str, external: bool) {
    let fixture: Fixture = fixture(name);
    if !prepare_cargo(&fixture, name) {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = cargo_repository(&fixture, external);
    let source_output: String = run_tool(&fixture, "cargo", repo.as_path(), &["run", "--offline", "--quiet"]);
    assert_eq!(source_output.trim(), repo.join("lib").to_str().expect("utf8"));
    std::fs::create_dir_all(fixture.root.join("deeper")).expect("deeper");
    let outside: Observed = add(&fixture, repo.as_path(), &["--quiet"], "outside", "../deeper/outside");
    let nested: Observed = add(&fixture, repo.as_path(), &["--quiet"], "nested", ".claude/worktrees/nested");
    for (label, root) in [("outside", fixture.root.join("deeper/outside")), ("nested", repo.join(".claude/worktrees/nested"))] {
        let reported: String = format!("{} {}", stderr_of(&outside), stderr_of(&nested));
        assert!(!reported.contains(PENDING_WORDS), "{label}: {reported}");
        // Dependencies from the registry are reused (not compiled again); the worktree's own crates rebuild.
        let first: Vec<String> = compiled(&fixture, root.as_path());
        assert_eq!(first, vec![String::from("app"), String::from("lib")], "{label}: only the worktree's own crates rebuild: {reported}");
        let ran: String = run_tool(&fixture, "cargo", root.as_path(), &["run", "--offline", "--quiet"]);
        assert_eq!(ran.trim(), root.join("lib").to_str().expect("utf8"), "{label}: the build names the worktree's own paths");
        assert!(compiled(&fixture, root.as_path()).is_empty(), "{label}: a second build compiles nothing");
        std::fs::write(root.join("lib/src/lib.rs"), b"pub const MANIFEST: &str = env!(\"CARGO_MANIFEST_DIR\");\npub const EDITED: u8 = 1;\n").expect("edit in the worktree");
        assert!(compiled(&fixture, root.as_path()).contains(&String::from("lib")), "{label}: an edit in the worktree rebuilds");
    }
    assert!(compiled(&fixture, repo.as_path()).is_empty(), "the source build is still fresh");
    let still: String = run_tool(&fixture, "cargo", repo.as_path(), &["run", "--offline", "--quiet"]);
    assert_eq!(still.trim(), repo.join("lib").to_str().expect("utf8"), "the source still names its own paths");
    remove(&fixture);
}

/// Cargo with the build directory keyed by workspace path outside the tree,
/// as the person's configuration has it.
#[test]
fn real_cargo_rebuilds_own_crates_with_an_external_build_directory() {
    cargo_control("repair-real-cargo-external", true);
}

/// Cargo's default layout,
/// where the build state lives in `target/` and arrives with the copy.
#[test]
fn real_cargo_rebuilds_own_crates_with_the_default_layout() {
    cargo_control("repair-real-cargo-default", false);
}

/// The workspace member the committed lock lacks,
/// which Cargo's rewrite of that lock adds.
const LATE_MEMBER: &str = "extra";

/// What:
///  The bytes kept for each `Cargo.lock` a repair's clean displaced,
/// under the repository's Git directory.
/// Why:
///   The copies sit under a directory named by the lock's own path in hexadecimal,
/// whose names are split to fit a file system,
///  so the control reads them by walking rather than by rebuilding that name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keptLockCopies = (repo) => walk(join(repo, '.git', DISPLACED_DIRECTORY)).map(readFile);
/// ```
fn kept_lock_copies(repo: &Path) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![repo.join(".git").join(git_policy_cli::package_repair_cargo_lock::DISPLACED_DIRECTORY)];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else { continue };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(bytes) = std::fs::read(path.as_path()) {
                found.push(bytes);
            }
        }
    }
    return found;
}

/// With real Cargo and a committed `Cargo.lock` that no longer matches a manifest (the state of several workspaces of this
///  repository),
/// the clean needs Cargo to resolve and Cargo would rewrite the lock;
/// the add must leave the tracked file as
///  checked out,
/// so `git status` of the new worktree stays clean,
/// must keep the bytes that rewrite displaced,
/// and must not report a failure.
#[test]
fn real_cargo_leaves_a_stale_lockfile_as_checked_out() {
    let fixture: Fixture = fixture("repair-real-cargo-stale-lock");
    if !prepare_cargo(&fixture, "real_cargo_leaves_a_stale_lockfile_as_checked_out") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = cargo_repository(&fixture, false);
    // A new member that `app` now depends on, committed without regenerating the lock.
    std::fs::create_dir_all(repo.join(LATE_MEMBER).join("src")).expect("the late member");
    std::fs::write(repo.join(LATE_MEMBER).join("Cargo.toml"), format!("[package]\nname = \"{LATE_MEMBER}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n")).expect("the late member's manifest");
    std::fs::write(repo.join(LATE_MEMBER).join("src/lib.rs"), format!("pub const {}: u8 = 1;\n", LATE_MEMBER.to_uppercase())).expect("the late member's source");
    std::fs::write(repo.join("Cargo.toml"), format!("[workspace]\nmembers = [\"app\", \"lib\", \"{LATE_MEMBER}\"]\nresolver = \"3\"\n")).expect("workspace");
    let app: String = std::fs::read_to_string(repo.join("app/Cargo.toml")).expect("app manifest");
    std::fs::write(repo.join("app/Cargo.toml"), format!("{app}{LATE_MEMBER} = {{ path = \"../{LATE_MEMBER}\" }}\n")).expect("app manifest with the late member");
    git(&fixture, repo.as_path(), &["add", "--", "Cargo.toml", "app/Cargo.toml", LATE_MEMBER]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=a member the lock lacks"]);
    let before: Vec<u8> = std::fs::read(repo.join("Cargo.lock")).expect("lock");
    std::fs::create_dir_all(fixture.root.join("deeper")).expect("deeper");
    let created: Observed = add(&fixture, repo.as_path(), &["--quiet"], "stale", "../deeper/stale");
    let worktree: PathBuf = fixture.root.join("deeper/stale");
    assert!(!stderr_of(&created).contains(PENDING_WORDS), "{}", stderr_of(&created));
    assert_eq!(std::fs::read(worktree.join("Cargo.lock")).expect("worktree lock"), before, "the worktree's lock was rewritten");
    let status = git(&fixture, worktree.as_path(), &["status", "--porcelain"]);
    assert!(status.stdout.is_empty(), "the new worktree is dirty: {}", String::from_utf8_lossy(&status.stdout));
    assert_eq!(std::fs::read(repo.join("Cargo.lock")).expect("source lock"), before, "the source's lock changed");
    // The bytes the clean displaced are kept rather than dropped: they are the only record of what stood in the worktree,
    // and nothing read after a clean can tell Cargo's rewrite from a newer write of the person's
    // (`package_repair_cargo_lock.rs`).
    let kept: Vec<Vec<u8>> = kept_lock_copies(repo.as_path());
    assert_eq!(kept.len(), 1, "the bytes the clean displaced were not kept: {kept:?}");
    let written: &[u8] = kept[0].as_slice();
    assert_ne!(written, before.as_slice(), "the kept copy is the checked-out lock itself, so no rewrite was kept");
    let late_member_line: String = format!("name = \"{LATE_MEMBER}\"");
    assert!(
        String::from_utf8_lossy(written).contains(late_member_line.as_str()),
        "the kept copy is not the lock the clean wrote: {}",
        String::from_utf8_lossy(written)
    );
    remove(&fixture);
}
