//! What:
//!  Controls that the built executable falls back to the real Git its commit service was set up with when no real Git
//!  follows it on `PATH`,
//! and keeps its stop when that record cannot be used.
//! Why:
//!  A `PATH` that holds the wrapper's `git` but no real Git (the incident of release 0.5.26 had no `/usr/bin`) used to
//!  end in a stop with exit status 2.
//! The coordinator picked,
//! under the owner's delegation and open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`,
//! section "Falling back to the Git the commit service was set up
//!  with"),
//! that the wrapper then uses the Git recorded in the unit file of its commit service,
//! with a notice on
//!  standard error.
//! A record is used only when it passes the checks that every `PATH` candidate passes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writeUnit(runtimeDir, { git: recordedGit });
//! // const run = spawnSync(join(fixture, 'bin/git'), ['--version'], { env: { PATH: wrapperDirectoryOnly, XDG_RUNTIME_DIR: runtimeDir } });
//! // expect(run.status).toBe(0); expect(run.stderr).toContain('no real Git follows this wrapper on PATH');
//! ```

/// The shared fixtures,
///  bounded runs and observation of what a run showed its caller.
use super::support::{
    Fixture, Observed, bounded, copy_executable, executable, fixture, link_real_git, observe, remove,
    repository,
};
/// The writer of the unit text the wrapper itself provisions,
///  so the control records a path exactly as the product does.
use git_policy_cli::service_provision::{ProvisionHost, SERVICE_UNIT_NAME, runtime_unit_dir, unit_texts};
/// Operating-system text,
///  for the one `PATH` value of a run.
use std::ffi::{OsStr, OsString};
/// The paths of the fixture's files.
use std::path::{Path, PathBuf};

/// The exit code of a wrapper that stops because it has no real Git to run.
const WRAPPER_STOP_CODE: i32 = 2;

/// The words every notice of this fallback starts with;
///  a person and a log search read them.
const NOTICE_LEAD: &str = "monochromatic-git-policy-cli: no real Git follows this wrapper on PATH";

/// The version line the stand-in Git prints.
const STAND_IN_VERSION: &str = "git version 2.56.0";

/// One directory under the fixture,
///  created now.
fn directory(fixture: &Fixture, name: &str) -> PathBuf {
    let path: PathBuf = fixture.root.join(name);
    std::fs::create_dir(&path).expect("fixture directory");
    return path;
}

/// What:
///  Write a stand-in for real Git:
///  a script that counts its start in `counter` and prints a version line.
/// Why:
///  The control must show how many times the wrapper started the recorded file;
///  shell built-ins only,
/// because the hermetic `PATH` of a run holds no `echo` or `wc` program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExecutable(path, `appendLine(counter, 'started'); console.log('git version 2.56.0')`);
/// ```
fn counting_git(path: &Path, counter: &Path) {
    let script: String = format!(
        "#!/bin/sh\necho started >> '{counter}'\necho '{STAND_IN_VERSION}'\n",
        counter = counter.display(),
    );
    executable(path, script.as_bytes());
}

/// How many starts a counter file records;
///  none when nothing started.
fn starts(counter: &Path) -> usize {
    return match std::fs::read_to_string(counter) {
        Ok(text) => text.lines().count(),
        Err(_) => 0,
    };
}

/// What:
///  Write the unit file the wrapper provisions for its commit service,
///  naming `git` as the Git the service starts,
///  below the runtime directory.
/// Why:
///  The control uses the product's own `unit_texts`,
/// so the record has the exact form the product writes and a change of
///  that form breaks this control and not the person's fallback.
///  The runtime directory is the fixture's own:
/// no control touches the real systemd user directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(join(runtime, 'systemd/user'), { recursive: true }); writeFileSync(join(runtime, 'systemd/user/cli-git@.service'), unitTexts({ git })[1]);
/// ```
fn write_record(runtime: &Path, git: &Path) {
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: runtime.to_path_buf(),
        user_unit_dir: None,
        bus: PathBuf::from("/nonexistent/bus"),
        service_binary: PathBuf::from("/nonexistent/monochromatic-git-policy-cli"),
        git: git.to_path_buf(),
        instance: String::from("fixture"),
        retirement_wait: std::time::Duration::ZERO,
    };
    let units = unit_texts(&host);
    let service_text: String = units
        .into_iter()
        .find(|(name, _)| {
            return *name == SERVICE_UNIT_NAME;
        })
        .map(|(_, text)| {
            return text;
        })
        .expect("the service unit text");
    let unit_directory: PathBuf = runtime_unit_dir(runtime);
    std::fs::create_dir_all(&unit_directory).expect("unit directory");
    std::fs::write(unit_directory.join(SERVICE_UNIT_NAME), service_text).expect("unit file");
}

/// What:
///  Write the record a toolchain launcher keeps beside it,
///  naming this wrapper's tool.
/// Why:
///  The wrapper recognises a launcher of itself by this record
///  (`package/mpm-shadow/cli/src/toolchain/launcher.rs`,
/// `render_record`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeFileSync(`${launcher}.launch`, JSON.stringify({ toolId, lockFile, real, scope: 'repo' }));
/// ```
fn write_launch_record(launcher: &Path, real: &Path) {
    let mut record_name: OsString = launcher.as_os_str().to_os_string();
    record_name.push(".launch");
    let record: String = format!(
        "{{\"toolId\":\"cargo:monochromatic-git-policy-cli\",\"lockFile\":\"/nonexistent/toolchain.lock.json\",\"real\":\"{}\",\"scope\":\"repo\"}}",
        real.display()
    );
    std::fs::write(PathBuf::from(record_name), record).expect("launcher record");
}

/// What:
///  Run the wrapper as the program named `git` with a `PATH` of exactly `path_directories` and the runtime directory
///  `runtime`,
///  in `repo`.
/// Why:
///  The reported failure had no directory of real Git on `PATH`,
///  and the record is found through the runtime directory variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync(join(fixture, 'bin/git'), args, { cwd: repo, env: { PATH: directories.join(':'), XDG_RUNTIME_DIR: runtime } });
/// ```
fn run_with_only(
    fixture: &Fixture,
    repo: &Path,
    path_directories: &[&Path],
    runtime: &Path,
    arguments: &[&str],
) -> Observed {
    let mut path: OsString = OsString::new();
    for entry in path_directories {
        if !path.is_empty() {
            path.push(":");
        }
        path.push(entry);
    }
    return observe(
        bounded(fixture, fixture.root.join("bin/git").as_path(), path.as_os_str())
            .current_dir(repo)
            .env("XDG_RUNTIME_DIR", runtime)
            .args(arguments),
        b"",
    );
}

/// The stderr of a run as text.
fn words(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// The stdout of a run as text.
fn output(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stdout.as_slice()).into_owned();
}

/// What:
///  The reported case with a record:
///  the only directory on `PATH` holds the wrapper's `git`,
///  and the commit service's unit names a real Git elsewhere.
/// Why:
///  The command must run once through the recorded Git,
///  and the person must be told,
/// in one line on standard error,
///  that `PATH` had no real Git,
/// which file was used and how to silence the notice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run(['--version'], { PATH: wrapperDirectory })).toMatchObject({ code: 0 }); expect(starts).toBe(1); expect(stderr).toContain(notice);
/// ```
#[test]
fn a_path_with_only_the_wrapper_and_a_recorded_real_git_runs_the_command_once_through_it_with_the_notice() {
    let fixture: Fixture = fixture("record-fallback-once");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let recorded_directory: PathBuf = directory(&fixture, "recorded");
    let recorded: PathBuf = recorded_directory.join("git");
    let counter: PathBuf = fixture.root.join("starts");
    counting_git(recorded.as_path(), counter.as_path());
    let runtime: PathBuf = directory(&fixture, "run");
    write_record(runtime.as_path(), recorded.as_path());
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[wrapper_directory.as_path()], runtime.as_path(), &["--version"]);
    let text: String = words(&observed);
    assert_eq!(observed.code, Some(0), "{text}");
    assert_eq!(starts(counter.as_path()), 1, "the recorded Git is started once: {text}");
    assert!(output(&observed).contains(STAND_IN_VERSION), "Git's own answer reaches the caller: {}", output(&observed));
    assert!(text.starts_with(NOTICE_LEAD), "the notice leads standard error: {text}");
    assert_eq!(text.lines().count(), 1, "the notice is one line: {text}");
    assert!(text.contains(wrapper_directory.to_string_lossy().as_ref()), "names the PATH it read: {text}");
    assert!(text.contains(recorded.to_string_lossy().as_ref()), "names the file it used: {text}");
    assert!(
        text.contains(&format!("add {} to the PATH", recorded_directory.display())),
        "names the way to silence it: {text}"
    );
    remove(&fixture);
}

/// What:
///  The same fallback with the real Git itself and a command that reads the repository.
/// Why:
///  A stand-in proves the count;
///  this proves that Git's real answer and exit code come through and that the notice does not enter standard output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run(['rev-parse', 'HEAD'], { PATH: wrapperDirectory }).stdout).toBe(directGit(['rev-parse', 'HEAD']).stdout);
/// ```
#[test]
fn the_recorded_real_git_answers_a_command_that_reads_the_repository_and_the_notice_stays_off_standard_output() {
    let fixture: Fixture = fixture("record-fallback-real");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    link_real_git(&fixture);
    let recorded: PathBuf = fixture.root.join("real/git");
    let runtime: PathBuf = directory(&fixture, "run");
    write_record(runtime.as_path(), recorded.as_path());
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[wrapper_directory.as_path()], runtime.as_path(), &["rev-parse", "HEAD"]);
    let expected: Observed = observe(
        super::support::direct(&fixture).current_dir(&repo).args(["rev-parse", "HEAD"]),
        b"",
    );
    let text: String = words(&observed);
    assert_eq!(observed.code, Some(0), "{text}");
    assert_eq!(observed.stdout, expected.stdout);
    assert!(text.starts_with(NOTICE_LEAD), "{text}");
    assert!(!output(&observed).contains("monochromatic-git-policy-cli:"), "standard output carries Git's answer only");
    remove(&fixture);
}

/// What:
///  A `PATH` that holds a real Git while the record names another file.
/// Why:
///  The record is a fallback and never an override:
///  the Git `PATH` offers is used,
/// the recorded file is not started and no notice is printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run(['--version'], { PATH: `${wrapperDirectory}:${realDirectory}` }).stderr).toBe('');
/// ```
#[test]
fn a_real_git_on_path_is_used_and_the_record_is_neither_started_nor_announced() {
    let fixture: Fixture = fixture("record-not-needed");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    link_real_git(&fixture);
    let recorded_directory: PathBuf = directory(&fixture, "recorded");
    let recorded: PathBuf = recorded_directory.join("git");
    let counter: PathBuf = fixture.root.join("starts");
    counting_git(recorded.as_path(), counter.as_path());
    let runtime: PathBuf = directory(&fixture, "run");
    write_record(runtime.as_path(), recorded.as_path());
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    let real_directory: PathBuf = fixture.root.join("real");
    let observed: Observed = run_with_only(
        &fixture,
        repo.as_path(),
        &[wrapper_directory.as_path(), real_directory.as_path()],
        runtime.as_path(),
        &["--version"],
    );
    assert_eq!(observed.code, Some(0), "{}", words(&observed));
    assert_eq!(starts(counter.as_path()), 0, "the recorded file is not started");
    assert!(!words(&observed).contains(NOTICE_LEAD), "no notice: {}", words(&observed));
    remove(&fixture);
}

/// What:
///  Assert the stop that no usable record leaves in place:
///  exit status 2,
///  nothing on standard output,
///  no notice,
///  the `PATH` and the way out named,
///  and no start of `counter`.
/// Why:
///  Four controls share these assertions;
///  the case that differs is what the record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(observed).toMatchObject({ code: 2, stdout: '' }); expect(stderr).not.toContain(notice); expect(starts).toBe(0);
/// ```
fn assert_stopped(observed: &Observed, counter: &Path, path_directory: &Path) {
    let text: String = words(observed);
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{text}");
    assert!(observed.stdout.is_empty(), "{}", output(observed));
    assert!(!text.contains(NOTICE_LEAD), "no fallback happened: {text}");
    assert_eq!(starts(counter), 0, "nothing was started as Git: {text}");
    assert!(text.contains(path_directory.to_string_lossy().as_ref()), "names the PATH it read: {text}");
    assert!(text.contains("install Git") || text.contains("Install Git"), "names the remedy: {text}");
}

/// What:
///  No unit file at all,
///  and a runtime directory that is not set.
/// Why:
///  No record means no recorded path:
///  the wrapper keeps its stop and does not look for Git anywhere `PATH` does not show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run(['--version'], { PATH: wrapperDirectory }).code).toBe(2);
/// ```
#[test]
fn no_record_keeps_the_stop() {
    let fixture: Fixture = fixture("record-absent");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let counter: PathBuf = fixture.root.join("starts");
    let runtime: PathBuf = directory(&fixture, "run");
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[wrapper_directory.as_path()], runtime.as_path(), &["--version"]);
    assert_stopped(&observed, counter.as_path(), wrapper_directory.as_path());
    let unset: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), wrapper_directory.as_os_str())
            .current_dir(&repo)
            .args(["--version"]),
        b"",
    );
    assert_stopped(&unset, counter.as_path(), wrapper_directory.as_path());
    remove(&fixture);
}

/// What:
///  A record that names this wrapper:
///  once through the file the wrapper runs as,
///  once through a byte copy of it.
/// Why:
///  A recorded path is checked like a `PATH` candidate:
///  the wrapper never starts itself as Git,
/// so both are refused and the stop remains,
///  with the record named so the person can see why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const named of [wrapperItself, wrapperCopy]) { writeUnit({ git: named }); expect(run(['--version']).code).toBe(2); }
/// ```
#[test]
fn a_record_naming_the_wrapper_or_a_copy_of_it_is_refused_and_keeps_the_stop() {
    let fixture: Fixture = fixture("record-names-wrapper");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let counter: PathBuf = fixture.root.join("starts");
    let copied_directory: PathBuf = directory(&fixture, "copied");
    let copy: PathBuf = copied_directory.join("git");
    copy_executable(fixture.wrapper.as_path(), copy.as_path());
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    for (index, named) in [fixture.wrapper.clone(), fixture.root.join("bin/git"), copy].into_iter().enumerate() {
        let runtime: PathBuf = directory(&fixture, format!("run-{index}").as_str());
        write_record(runtime.as_path(), named.as_path());
        let observed: Observed = run_with_only(&fixture, repo.as_path(), &[wrapper_directory.as_path()], runtime.as_path(), &["--version"]);
        assert_stopped(&observed, counter.as_path(), wrapper_directory.as_path());
        assert!(words(&observed).contains(named.to_string_lossy().as_ref()), "names the recorded file: {}", words(&observed));
    }
    remove(&fixture);
}

/// What:
///  A record that names a toolchain launcher of this wrapper,
///  a file with a `.launch` record beside it,
///  behind a counting stand-in.
/// Why:
///  A unit written while the 0.5.26 flaw stood can hold such a path.
///  Starting it would start the wrapper again,
/// so it is refused without a start and the stop remains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeUnit({ git: launcher }); expect(run(['--version']).code).toBe(2); expect(starts).toBe(0);
/// ```
#[test]
fn a_record_naming_a_launcher_of_the_wrapper_is_refused_without_a_start_and_keeps_the_stop() {
    let fixture: Fixture = fixture("record-names-launcher");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let counter: PathBuf = fixture.root.join("starts");
    let launchers: PathBuf = directory(&fixture, "launchers");
    let launcher: PathBuf = launchers.join("monochromatic-git-policy-cli");
    counting_git(launcher.as_path(), counter.as_path());
    write_launch_record(launcher.as_path(), fixture.wrapper.as_path());
    let runtime: PathBuf = directory(&fixture, "run");
    write_record(runtime.as_path(), launcher.as_path());
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[wrapper_directory.as_path()], runtime.as_path(), &["--version"]);
    assert_stopped(&observed, counter.as_path(), wrapper_directory.as_path());
    assert!(words(&observed).contains(launcher.to_string_lossy().as_ref()), "names the recorded file: {}", words(&observed));
    remove(&fixture);
}

/// What:
///  A record that names a file which does not exist,
///  and one that names a file which is not executable.
/// Why:
///  A recorded path survives an uninstall of the Git it named;
///  the wrapper must not start what is not there,
/// so the stop remains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const named of [missing, notExecutable]) { writeUnit({ git: named }); expect(run(['--version']).code).toBe(2); }
/// ```
#[test]
fn a_record_naming_a_missing_or_non_executable_file_is_refused_and_keeps_the_stop() {
    let fixture: Fixture = fixture("record-names-missing");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let counter: PathBuf = fixture.root.join("starts");
    let missing: PathBuf = fixture.root.join("gone/git");
    let plain_directory: PathBuf = directory(&fixture, "plain");
    let plain: PathBuf = plain_directory.join("git");
    std::fs::write(&plain, b"#!/bin/sh\n").expect("a file without the executable bit");
    let wrapper_directory: PathBuf = fixture.root.join("bin");
    for (index, named) in [missing, plain].into_iter().enumerate() {
        let runtime: PathBuf = directory(&fixture, format!("run-{index}").as_str());
        write_record(runtime.as_path(), named.as_path());
        let observed: Observed = run_with_only(&fixture, repo.as_path(), &[wrapper_directory.as_path()], runtime.as_path(), &["--version"]);
        assert_stopped(&observed, counter.as_path(), wrapper_directory.as_path());
        assert!(words(&observed).contains(named.to_string_lossy().as_ref()), "names the recorded file: {}", words(&observed));
    }
    remove(&fixture);
}
