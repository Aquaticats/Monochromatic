//! What:
//!  Controls that the built executable never starts itself as Git,
//! whatever stands between the wrapper and the file it
//!  selected:
//! a toolchain launcher,
//! a link,
//! a copy,
//! another release of the wrapper.
//! Why:
//!  Release 0.5.26 on a `PATH` whose only `git` was a symlink to the toolchain launcher of the wrapper started a chain of
//!  wrapper processes,
//! each one the Git of the one above it,
//! each command line longer than its parent's,
//! until the
//!  control group it ran in refused another process
//!  (`doc/planning/cli-git-no-self-invocation.md`,
//! section "The wrapper never starts itself as Git").
//!  Every control here runs the chain behind a counting stand-in that ends it after `START_CAP` starts,
//! so a regression
//!  shows as a start count and an exit code,
//! never as a host that runs out of processes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const starts = countLines(counterFile);
//! // expect(run(wrapper, { PATH: launcherDirectory }).code).toBe(2); expect(starts).toBe(0);
//! ```

/// The shared fixtures,
/// bounded runs and observation of what a run showed its caller.
use super::support::{Fixture, Observed, REAL_GIT, bounded, copy_executable, executable, fixture, git, observe, remove, repository};
/// The bound one chain of starts ends at,
/// which the relay control reads its refusal against.
use git_policy_cli::started_as_git::MOST_STARTS_IN_ONE_CHAIN;
/// Operating-system text,
/// for building the one `PATH` value of a run.
use std::ffi::{OsStr, OsString};
/// The paths of the fixture's files.
use std::path::{Path, PathBuf};

/// The most starts a counting stand-in allows before it ends the chain with `STAND_IN_STOP_CODE`.
///  The value is a fixture bound and not a limit of the wrapper:
/// it only has to be small,
/// because the chain this
///  control exists to catch grows by one process per start,
/// and larger than the one or two starts a correct run makes
///  so that a correct run never meets it.
const START_CAP: usize = 6;

/// The exit code of a stand-in that has reached `START_CAP`;
/// no wrapper exit code uses it.
const STAND_IN_STOP_CODE: i32 = 97;

/// What:
///  The deepest place in one chain a control's stand-in may reach before it ends the chain with `STAND_IN_STOP_CODE`
///  itself.
/// Why:
///   The wrapper's own bound on one chain of starts (`started_as_git::MOST_STARTS_IN_ONE_CHAIN`)
/// is reached first by a
///  correct run;
/// this is the fixture's backstop above it,
/// so a chain that ignores the bound fails as a level
///  and an exit code,
/// never as a host or a scope that runs out of processes.
const CHAIN_CAP: usize = 40;

/// The variable a control's stand-in passes its place in one chain down in,
/// so a level counts once however many starts it makes.
const CHAIN_LEVEL_VARIABLE: &str = "CLI_GIT_TEST_CHAIN_LEVEL";

/// The exit code of a wrapper that stops because it has no real Git to run.
const WRAPPER_STOP_CODE: i32 = 2;

/// The name of the toolchain launcher the repository's tool links point at,
/// and of its record's stem.
const LAUNCHER_NAME: &str = "monochromatic-git-policy-cli";

/// What:
///  Write a stand-in launcher:
/// a script that counts its start in `counter`,
/// ends the chain after `START_CAP` starts,
/// and
///  otherwise replaces itself with `target`.
/// Why:
///  The real launcher replaces itself with the wrapper's executable.
/// The stand-in does the same and also counts,
/// so a
///  control can assert how many times the wrapper reached the launcher.
/// It uses shell built-ins only,
/// because the
///  hermetic `PATH` of a control holds no `echo` or `wc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExecutable(path, `appendLine(counter); if (lines(counter) > START_CAP) process.exit(97); execve(target, args)`);
/// ```
fn counting_stand_in(path: &Path, counter: &Path, target: &Path) {
    let script: String = format!(
        "#!/bin/sh\n\
         echo started >> '{counter}'\n\
         started=0\n\
         while read -r line; do started=$((started + 1)); done < '{counter}'\n\
         if [ \"$started\" -gt {START_CAP} ]; then\n\
         echo 'stand-in: start cap reached' >&2\n\
         exit {STAND_IN_STOP_CODE}\n\
         fi\n\
         exec '{target}' \"$@\"\n",
        counter = counter.display(),
        target = target.display(),
    );
    executable(path, script.as_bytes());
}

/// What:
///  Write the record the toolchain's launcher keeps beside it,
/// naming this wrapper's tool and the file it replaces
///  itself with.
/// Why:
///  The wrapper recognises a launcher of itself by this record (`toolId` and `real`),
/// so the control needs the same
///  JSON the repository's `mpm-shadow` writes (`package/mpm-shadow/cli/src/toolchain/launcher.rs`,
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

/// How many starts a counter file records;
/// none when nothing started.
fn starts(counter: &Path) -> usize {
    return match std::fs::read_to_string(counter) {
        Ok(text) => text.lines().count(),
        Err(_) => 0,
    };
}

/// What:
///  Write a stand-in `git` that starts `target` as a **child** and waits,
/// recording its place in one chain in `counter` and
///  ending the chain itself past `cap`.
/// Why:
///  The covered failure is a script that names the wrapper without `exec`,
/// which puts a shell between the wrapper
///  that replaced itself with the script and the wrapper the script starts.
/// The place in the chain travels in `CHAIN_LEVEL_VARIABLE`,
/// so one level counts once however many starts
///  it makes;
/// the cap is the control's own backstop,
/// not a wrapper limit,
/// because without it a regression here
///  would grow the chain until the host or the test's supervisor refused another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExecutable(path, `const level = 1 + +(env[LEVEL] ?? 0); appendLine(counter, level); if (level > cap) process.exit(97); env[LEVEL] = level; ${target} "$@"`);
/// ```
fn counting_child_stand_in(path: &Path, counter: &Path, target: &Path, cap: usize) {
    let script: String = format!(
        "#!/bin/sh\n\
         level=$(( ${{{CHAIN_LEVEL_VARIABLE}:-0}} + 1 ))\n\
         echo \"$level\" >> '{counter}'\n\
         if [ \"$level\" -gt {cap} ]; then\n\
         echo 'stand-in: start cap reached' >&2\n\
         exit {STAND_IN_STOP_CODE}\n\
         fi\n\
         {CHAIN_LEVEL_VARIABLE}=$level\n\
         export {CHAIN_LEVEL_VARIABLE}\n\
         '{target}' \"$@\"\n",
        counter = counter.display(),
        target = target.display(),
    );
    executable(path, script.as_bytes());
}

/// What:
///  The deepest place in one chain a counter file records;
/// none when nothing started.
/// Why:
///   A level makes more than one start (its own reads then its forward),
/// so the place in the chain is the deepest number,
/// not
///  the number of lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deepest = Math.max(...lines(counter).map(Number));
/// ```
fn deepest_level(counter: &Path) -> usize {
    return std::fs::read_to_string(counter)
        .map_or(0, |text| { return text.lines().filter_map(|line| { return line.trim().parse::<usize>().ok(); }).max().unwrap_or(0); });
}

/// One directory under the fixture,
/// created now.
fn directory(fixture: &Fixture, name: &str) -> PathBuf {
    let path: PathBuf = fixture.root.join(name);
    std::fs::create_dir(&path).expect("fixture directory");
    return path;
}

/// What:
///  Run the wrapper as a program named `git` under a `PATH` of exactly `path_directories`,
/// in `repo`,
/// with a time bound.
/// Why:
///  The reported failure had no directory of real Git on `PATH` at all,
/// so the controls run with only the directories
///  they name,
/// and the wrapper's own `bin/git` link is the program that starts first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync(join(fixture, 'bin/git'), args, { cwd: repo, env: { PATH: directories.join(':') } });
/// ```
fn run_with_only(fixture: &Fixture, repo: &Path, path_directories: &[&Path], arguments: &[&str]) -> Observed {
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
            .args(arguments),
        b"",
    );
}

/// The stderr of a run as text,
/// for assertions on the words a person reads.
fn words(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// What:
///  The reported failure:
/// the only `git` on `PATH` is a symlink to the toolchain launcher of the wrapper.
/// Why:
///  The launcher is a different file from the running wrapper,
/// with different bytes,
/// so neither file identity nor content
///  recognised it;
/// it was started as Git,
/// replaced itself with the wrapper,
/// and the wrapper did the same again.
///  The wrapper must start nothing and say what it searched and how to fix it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: dirWithOnlyGitLinkedToLauncher }, ['rev-parse', 'HEAD'])).toMatchObject({ code: 2 }); expect(starts).toBe(0);
/// ```
#[test]
fn a_symlink_to_the_toolchain_launcher_alone_on_path_starts_nothing_and_says_what_was_searched() {
    let fixture: Fixture = fixture("launcher-symlink");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let launchers: PathBuf = directory(&fixture, "launchers");
    let counter: PathBuf = fixture.root.join("starts");
    let launcher: PathBuf = launchers.join(LAUNCHER_NAME);
    counting_stand_in(launcher.as_path(), counter.as_path(), fixture.wrapper.as_path());
    write_launch_record(launcher.as_path(), fixture.wrapper.as_path());
    let path_directory: PathBuf = directory(&fixture, "only");
    std::os::unix::fs::symlink(&launcher, path_directory.join("git")).expect("symlink to the launcher");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[path_directory.as_path()], &["rev-parse", "HEAD"]);
    let text: String = words(&observed);
    assert_eq!(starts(counter.as_path()), 0, "the launcher must never be started: {text}");
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{text}");
    assert!(observed.stdout.is_empty());
    assert!(text.contains(path_directory.join("git").to_string_lossy().as_ref()), "names the candidate it skipped: {text}");
    assert!(text.contains("PATH"), "names the input it searched: {text}");
    assert!(text.contains("install Git") || text.contains("Install Git"), "names the remedy: {text}");
    remove(&fixture);
}

/// What:
///  The same failure for `--version`,
/// which the reporter had run with a `PATH` that held real Git.
/// Why:
///  The version line proved nothing about the chain:
/// the banner names the Git the wrapper selected,
/// and which Git it
///  selects depends on `PATH`,
/// so under the launcher-only `PATH` the same command must stop too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: dirWithOnlyGitLinkedToLauncher }, ['--version'])).toMatchObject({ code: 2 });
/// ```
#[test]
fn a_toolchain_launcher_alone_on_path_answers_version_with_a_stop_and_not_a_chain() {
    let fixture: Fixture = fixture("launcher-version");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let launchers: PathBuf = directory(&fixture, "launchers");
    let counter: PathBuf = fixture.root.join("starts");
    let launcher: PathBuf = launchers.join(LAUNCHER_NAME);
    counting_stand_in(launcher.as_path(), counter.as_path(), fixture.wrapper.as_path());
    write_launch_record(launcher.as_path(), fixture.wrapper.as_path());
    let path_directory: PathBuf = directory(&fixture, "only");
    std::os::unix::fs::symlink(&launcher, path_directory.join("git")).expect("symlink to the launcher");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[path_directory.as_path()], &["--version"]);
    assert_eq!(starts(counter.as_path()), 0, "{}", words(&observed));
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{}", words(&observed));
    remove(&fixture);
}

/// What:
///  A launcher file that is itself named `git`,
/// with its record beside it,
/// and a chain of two symlinks that ends at a
///  launcher.
/// Why:
///  The record is found beside the candidate's own name and beside the file the links finally lead to,
/// so both
///  spellings of a launcher on `PATH` are recognised and skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const spelling of ['git + git.launch', 'git -> link -> launcher']) expect(starts).toBe(0);
/// ```
#[test]
fn a_launcher_named_git_and_a_symlink_chain_to_a_launcher_are_skipped() {
    let fixture: Fixture = fixture("launcher-named-git");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let counter: PathBuf = fixture.root.join("starts");
    let named: PathBuf = directory(&fixture, "named");
    counting_stand_in(named.join("git").as_path(), counter.as_path(), fixture.wrapper.as_path());
    write_launch_record(named.join("git").as_path(), fixture.wrapper.as_path());
    let linked: PathBuf = directory(&fixture, "linked");
    let launchers: PathBuf = directory(&fixture, "launchers");
    let launcher: PathBuf = launchers.join(LAUNCHER_NAME);
    counting_stand_in(launcher.as_path(), counter.as_path(), fixture.wrapper.as_path());
    write_launch_record(launcher.as_path(), fixture.wrapper.as_path());
    let middle: PathBuf = linked.join("middle");
    std::os::unix::fs::symlink(&launcher, &middle).expect("first link");
    std::os::unix::fs::symlink(&middle, linked.join("git")).expect("second link");
    for directory_under_test in [named.as_path(), linked.as_path()] {
        let observed: Observed = run_with_only(&fixture, repo.as_path(), &[directory_under_test], &["rev-parse", "HEAD"]);
        assert_eq!(starts(counter.as_path()), 0, "{}: {}", directory_under_test.display(), words(&observed));
        assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{}: {}", directory_under_test.display(), words(&observed));
    }
    remove(&fixture);
}

/// What:
///  Another release of the wrapper behind a launcher that keeps no record,
/// reached once by a child start and once by
///  process replacement.
/// Why:
///  Nothing on disk says that this launcher leads to a wrapper.
/// The wrapper that starts it has set its marker in the
///  environment of the start,
/// so the other release finds itself the process,
/// or the child,
/// of a wrapper that started it
///  as Git,
/// and stops at once.
/// The launcher is started once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const args of [['--version'], ['--exec-path']]) { expect(run(args).code).toBe(2); expect(starts).toBe(1); }
/// ```
#[test]
fn another_wrapper_release_behind_an_unrecorded_launcher_stops_on_arrival() {
    let fixture: Fixture = fixture("other-release-behind-launcher");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let other_directory: PathBuf = directory(&fixture, "other");
    let other: PathBuf = other_directory.join("monochromatic-git-policy-cli");
    copy_executable(fixture.wrapper.as_path(), other.as_path());
    // One trailing byte makes a different file with different bytes that still runs, as another release would be.
    let mut bytes: Vec<u8> = std::fs::read(&other).expect("copy bytes");
    bytes.push(0);
    executable(other.as_path(), bytes.as_slice());
    let path_directory: PathBuf = directory(&fixture, "only");
    let counter: PathBuf = fixture.root.join("starts");
    counting_stand_in(path_directory.join("git").as_path(), counter.as_path(), other.as_path());
    for arguments in [&["--version"][..], &["--exec-path"][..], &["rev-parse", "HEAD"][..]] {
        std::fs::remove_file(&counter).ok();
        let observed: Observed = run_with_only(&fixture, repo.as_path(), &[path_directory.as_path()], arguments);
        let text: String = words(&observed);
        assert!(starts(counter.as_path()) <= 1, "{arguments:?}: {text}");
        assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{arguments:?}: {text}");
        assert!(text.contains("started as Git"), "{arguments:?}: {text}");
        assert!(text.contains(path_directory.join("git").to_string_lossy().as_ref()) || text.contains("PATH"), "{arguments:?}: {text}");
    }
    remove(&fixture);
}

/// What:
///  A symlink,
/// a hard link and a byte copy of the wrapper,
/// each alone on `PATH`.
/// Why:
///  File identity and content recognise these forms of this very executable;
/// with nothing else on `PATH` the run must
///  end with the stop and the explanation,
/// not with a start of the wrapper by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const form of [symlink, hardLink, copy]) expect(run({ PATH: dirOf(form) }).code).toBe(2);
/// ```
#[test]
fn a_symlink_a_hard_link_and_a_copy_of_the_wrapper_alone_on_path_are_skipped() {
    let fixture: Fixture = fixture("wrapper-forms-alone");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let symlinked: PathBuf = directory(&fixture, "symlinked");
    std::os::unix::fs::symlink(&fixture.wrapper, symlinked.join("git")).expect("symlink");
    let hard: PathBuf = directory(&fixture, "hard");
    std::fs::hard_link(&fixture.wrapper, hard.join("git")).expect("hard link");
    let copied: PathBuf = directory(&fixture, "copied");
    copy_executable(fixture.wrapper.as_path(), copied.join("git").as_path());
    for form in [symlinked.as_path(), hard.as_path(), copied.as_path()] {
        let observed: Observed = run_with_only(&fixture, repo.as_path(), &[form], &["rev-parse", "HEAD"]);
        let text: String = words(&observed);
        assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{}: {text}", form.display());
        assert!(observed.stdout.is_empty(), "{}", form.display());
        assert!(text.contains(form.join("git").to_string_lossy().as_ref()), "{}: {text}", form.display());
    }
    remove(&fixture);
}

/// What:
///  A user script named `git` that starts the wrapper as a child,
/// with no `exec`,
/// anywhere on `PATH`.
/// Why:
///  A `replace` start keeps its process number,
/// so the arriving wrapper's parent is the process the marker
///  names,
/// not its own number;
/// the guard that compares only the number therefore matched nothing,
/// and each level
///  started another wrapper with nothing printed
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
///  "Reproductions" A).
/// The chain must end within the wrapper's own bound,
/// with the stop and the message that names the
///  `git` on `PATH` that led back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [binWithWrapper, dirWithScript] }).code).toBe(2); expect(starts).toBeLessThanOrEqual(bound + 1);
/// ```
#[test]
fn a_script_named_git_that_starts_the_wrapper_without_exec_ends_the_chain() {
    let fixture: Fixture = fixture("script-no-exec");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let scripts: PathBuf = directory(&fixture, "scripts");
    let counter: PathBuf = fixture.root.join("starts");
    let script: PathBuf = scripts.join("git");
    counting_child_stand_in(script.as_path(), counter.as_path(), fixture.wrapper.as_path(), CHAIN_CAP);
    let bin: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[bin.as_path(), scripts.as_path()], &["rev-parse", "HEAD"]);
    let text: String = words(&observed);
    let deepest: usize = deepest_level(counter.as_path());
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "the wrapper must end the chain itself, deepest level {deepest}: {text}");
    assert!(
        (2..CHAIN_CAP).contains(&deepest),
        "the chain must end after one step and before the stand-in's backstop at level {CHAIN_CAP}: deepest level {deepest}: {text}"
    );
    assert!(text.contains("started as Git"), "{text}");
    assert!(text.contains(script.to_string_lossy().as_ref()), "names the `git` on PATH that led back: {text}");
    assert!(observed.stdout.is_empty());
    remove(&fixture);
}

/// What:
///  Nine shell aliases that each run the next one through `git` on `PATH`,
/// the last of them running the command the
///  person asked for.
/// Why:
///  Every level is a real Git of its own that runs an alias,
/// and the wrapper of each level arrives as a child of a
///  shell rather than of the level above,
/// so each is a legitimate nesting;
/// the count of the marker ended
///  the chain at
/// `MOST_STARTS_IN_ONE_CHAIN`,
/// and nine aliases needed a tenth start
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 3 of the round-1 review).
/// The answer is the one real Git gives,
/// not a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [i, name] of ['a','b','c','d','e','f','g','h','i'].entries()) await git(['config', `alias.${name}`, `!git ${['b','c','d','e','f','g','h','i','rev-parse HEAD'][i]}`]);
/// expect(runWrapped(['a'])).toEqual(runGit(['rev-parse', 'HEAD']));
/// ```
#[test]
fn a_finite_chain_of_aliases_reaches_the_command_the_person_asked_for() {
    let fixture: Fixture = fixture("alias-chain");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let names: [&str; 9] = ["a", "b", "c", "d", "e", "f", "g", "h", "i"];
    for (index, name) in names.iter().enumerate() {
        let next: &str = names.get(index + 1).copied().unwrap_or("rev-parse");
        let body: String = if *name == "i" { String::from("!git rev-parse HEAD") } else { format!("!git {next}") };
        git(&fixture, repo.as_path(), &["config", format!("alias.{name}").as_str(), body.as_str()]);
    }
    let expected: std::process::Output = git(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let observed: Observed = super::support::run_wrapped(&fixture, repo.as_path(), &["a"]);
    let text: String = words(&observed);
    assert_eq!(observed.code, Some(0), "the chain of aliases must run: {text}");
    assert_eq!(observed.stdout, expected.stdout, "the answer is the one real Git gives: {text}");
    assert!(!text.contains("started as Git"), "a finite chain of aliases is no recursion: {text}");
    // The same chain where Git's own exec directory offers the wrapper, which is the install whose `git-core/git` leads
    //  back to it: every alias then runs the wrapper rather than the system Git.
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":/usr/bin:/bin");
    let through: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str())
            .env("GIT_EXEC_PATH", fixture.root.join("bin"))
            .current_dir(repo.as_path())
            .args(["a"]),
        b"",
    );
    let said: String = words(&through);
    assert_eq!(through.code, Some(0), "a finite chain of aliases is no recursion, whatever Git's exec path offers: {said}");
    assert_eq!(through.stdout, expected.stdout, "the answer is the one real Git gives: {said}");
    assert!(!said.contains("started as Git"), "{said}");
    remove(&fixture);
}


/// What:
///  Write a launcher named `git` that clears the environment and hands the invocation to two shells that only relay
///  it,
///  and a `relay` pair of scripts.
/// Why:
///  This is the shape a count of starts loses:
/// the launcher replaces itself with `env`,
/// which runs the first relay,
///  which
///  starts the second and waits,
/// and the second starts the wrapper and waits,
/// so two processes stand between the record the
///  wrapper writes and the wrapper that reads it.
/// Neither relay carries a marker,
/// because the launcher cleared the environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExecutable(launcher, `#!/bin/sh\nexec env --ignore-environment HOME=... PATH=... /bin/sh '${relayParent}' "$@"`);
/// writeExecutable(relayParent, `#!/bin/sh\n/bin/sh '${relayChild}' "$@"; echo relayed`);
/// writeExecutable(relayChild, `#!/bin/sh\n'${wrapper}' "$@"; echo relayed`);
/// ```
fn relays_behind_a_launcher_that_clears_the_environment(scripts: &Path, path_variable: &OsStr, wrapper: &Path, relays: usize) {
    let launcher: PathBuf = scripts.join("git");
    let first: PathBuf = scripts.join("relay-0");
    // A read of the wrapper's own is answered by the real Git, so the forwarding below is the chain this control is about.
    executable(
        launcher.as_path(),
        format!(
            "#!/bin/sh\ncase \"${{CLI_GIT_STARTED_AS_GIT}}\" in\n*query*) exec {real_git} \"$@\" ;;\nesac\nexec /usr/bin/env --ignore-environment HOME=\"$HOME\" PATH='{path}' /bin/sh '{first}' \"$@\"\n",
            path = path_variable.to_string_lossy(),
            first = first.display(),
            real_git = REAL_GIT
        )
        .as_bytes(),
    );
    // Each relay starts the next and waits, and the last starts the wrapper and waits, so every one of them stands
    //  between the wrapper that wrote the record and the wrapper that reads it.
    for level in 0..relays {
        let next: String = if level + 1 < relays {
            format!("/bin/sh '{}' \"$@\"", scripts.join(format!("relay-{}", level + 1)).display())
        } else {
            format!("'{}' \"$@\"", wrapper.display())
        };
        executable(
            scripts.join(format!("relay-{level}")).as_path(),
            format!("#!/bin/sh\n{next}\nstatus=$?\necho relayed >&2\nexit $status\n").as_bytes(),
        );
    }
}


/// What:
///  A chain of relays longer than the ancestors the record is looked for in still ends.
/// Why:
///  The walk read four processes above the arriving wrapper,
/// and a launcher that starts five relays puts the
///  start that wrote the record further above than that;
/// the record was then rejected and the count started again
///  at one at every generation,
/// so the wrapper kept starting itself and accumulating waiting shells until an
///  outside limit stopped it
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 1 of the round-4
///  review).
/// The record is now looked for in every process above this one,
/// so the length of the relay chain is not what
///  ends the count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [bin, scripts] }).code).toBe(2); expect(text).toContain('start 33 of one chain');
/// ```
#[test]
fn a_chain_of_five_relays_still_ends() {
    let fixture: Fixture = fixture("launcher-five-relays");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let scripts: PathBuf = directory(&fixture, "scripts");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(scripts.as_os_str());
    relays_behind_a_launcher_that_clears_the_environment(scripts.as_path(), path.as_os_str(), fixture.wrapper.as_path(), 5);
    let observed: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str()).current_dir(repo.as_path()).args(["rev-parse", "HEAD"]),
        b"",
    );
    let text: String = words(&observed);
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "the wrapper must end the chain itself, not a relay limit: {text}");
    assert!(text.contains(&format!("start {}", MOST_STARTS_IN_ONE_CHAIN + 1)), "it names the start that passed the bound: {text}");
    remove(&fixture);
}

/// What:
///  Two relays between the start that wrote the chain record and the wrapper that reads it still end the chain.
/// Why:
///  The rule that continued the record accepted only the reading process and the one that started it,
/// so two relays
///  reset the count to one at every generation,
/// every wrapper wrote a first-start marker,
/// and wrappers
///  and relays accumulated until an outside limit stopped them
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-3
///  review).
/// A record of a live process above is one of this chain,
/// however many relays it started between the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [bin, scripts] }).code).toBe(2); expect(deepestLevel(counter)).toBeLessThan(CHAIN_CAP);
/// ```
#[test]
fn a_chain_of_relays_between_two_starts_still_ends() {
    let fixture: Fixture = fixture("launcher-relays");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let scripts: PathBuf = directory(&fixture, "scripts");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(scripts.as_os_str());
    relays_behind_a_launcher_that_clears_the_environment(scripts.as_path(), path.as_os_str(), fixture.wrapper.as_path(), 2);
    let observed: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str()).current_dir(repo.as_path()).args(["rev-parse", "HEAD"]),
        b"",
    );
    let text: String = words(&observed);
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "the wrapper must end the chain itself, not a relay limit: {text}");
    assert!(text.contains("started as Git"), "{text}");
    assert!(text.contains(&format!("start {}", MOST_STARTS_IN_ONE_CHAIN + 1)), "it names the start that passed the bound: {text}");
    remove(&fixture);
}

/// What:
///  Write a launcher named `git` that runs the wrapper as a **child** of a cleared environment:
/// it counts its own level,
///  then runs `/usr/bin/env --ignore-environment` as a child with the same `path` and the wrapper,
/// and waits for it.
/// Why:
///  `env --ignore-environment` removes the marker variable from the environment of the wrapper it starts,
/// so neither
///  check that reads the marker sees anything there,
/// and a launcher that starts the wrapper again is a chain no count ends
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-1 review).
/// The launcher waits for its child,
/// as the reported chain's levels did,
/// and it is the process whose own environment still
///  holds the marker the wrapper wrote for it.
/// The level travels in the environment the launcher itself builds after the wipe,
/// so one level counts once;
/// the
///  cap is the control's own backstop and not a wrapper limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeExecutable(launcher, `#!/bin/sh\nlevel=$(( ${LEVEL:-0} + 1 )); echo $level >> counter; if [ $level -gt cap ]; then exit 97; fi\nenv --ignore-environment HOME=... PATH=... LEVEL=$level wrapper "$@"`);
/// ```
fn launcher_that_clears_the_environment(path: &Path, counter: &Path, path_variable: &OsStr, wrapper: &Path, cap: usize, replaces_itself: bool) {
    // A launcher that `exec`s leaves one process running the launcher, the clearing and the wrapper again.
    let start: &str = if replaces_itself { "exec /usr/bin/env" } else { "/usr/bin/env" };
    let script: String = format!(
        "#!/bin/sh\n\
         level=$(( ${{{CHAIN_LEVEL_VARIABLE}:-0}} + 1 ))\n\
         echo \"$level\" >> '{counter}'\n\
         if [ \"$level\" -gt {cap} ]; then\n\
         echo 'stand-in: start cap reached' >&2\n\
         exit {STAND_IN_STOP_CODE}\n\
         fi\n\
         {start} --ignore-environment HOME=\"$HOME\" PATH='{path_variable}' {CHAIN_LEVEL_VARIABLE}=$level '{wrapper}' \"$@\"\n",
        counter = counter.display(),
        path_variable = path_variable.to_string_lossy(),
        wrapper = wrapper.display(),
    );
    executable(path, script.as_bytes());
}

/// What:
///  The reported launcher that clears the environment **and replaces itself with the wrapper**,
/// so that one process runs the
///  launcher,
/// the clearing and the wrapper again.
/// Why:
///  No second process and no environment of its own are left,
/// so the environment marker and the environment of the process
///  above both say nothing,
/// and the wrapper started itself without end
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 1 of the round-2
///  review).
/// What survives an `exec` is the descriptor the chain record lives in,
/// and the pair of the start that
///  wrote it,
/// so the same process arriving as the wrapper again reads its own pair and stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [bin, launchers] }).code).toBe(2); expect(deepestLevel(counter)).toBe(1);
/// ```
#[test]
fn a_launcher_that_clears_the_environment_and_replaces_itself_ends_the_chain() {
    let fixture: Fixture = fixture("launcher-exec-clears-environment");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let launchers: PathBuf = directory(&fixture, "launchers");
    let counter: PathBuf = fixture.root.join("starts");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(launchers.as_os_str());
    launcher_that_clears_the_environment(launchers.join("git").as_path(), counter.as_path(), path.as_os_str(), fixture.wrapper.as_path(), CHAIN_CAP, true);
    let observed: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str()).current_dir(repo.as_path()).args(["rev-parse", "HEAD"]),
        b"",
    );
    let text: String = words(&observed);
    let deepest: usize = deepest_level(counter.as_path());
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "the wrapper must end the chain itself, deepest level {deepest}: {text}");
    assert_eq!(deepest, 1, "the launcher runs once and the wrapper it started stops: {text}");
    assert!(text.contains("started as Git"), "{text}");
    assert!(text.contains("replaced with this program"), "says how the start arrived: {text}");
    assert!(text.contains(launchers.join("git").to_string_lossy().as_ref()), "names the `git` on PATH that led back: {text}");
    remove(&fixture);
}

/// What:
///  The reported launcher that clears the environment:
/// the wrapper must end the chain itself,
/// with the stop and the message
///  that names the `git` on `PATH` which led back,
/// and not by growing until the stand-in's own cap.
/// Why:
///  Both recursion checks read a variable of the environment,
/// and the clearing launcher removes it,
/// so the
///  wrapper saw a fresh start at every level and started one more wrapper
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-1 review).
/// What survives the wipe is what the wrapper can read about the process that started it:
/// the environment
///  the launcher itself was started with,
/// which no child can clear.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [bin, launchers] }).code).toBe(2); expect(deepestLevel(counter)).toBeLessThan(CHAIN_CAP);
/// ```
#[test]
fn a_launcher_that_clears_the_environment_ends_the_chain() {
    let fixture: Fixture = fixture("launcher-clears-environment");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let launchers: PathBuf = directory(&fixture, "launchers");
    let counter: PathBuf = fixture.root.join("starts");
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(launchers.as_os_str());
    launcher_that_clears_the_environment(launchers.join("git").as_path(), counter.as_path(), path.as_os_str(), fixture.wrapper.as_path(), CHAIN_CAP, false);
    let observed: Observed = observe(
        bounded(&fixture, fixture.root.join("bin/git").as_path(), path.as_os_str()).current_dir(repo.as_path()).args(["rev-parse", "HEAD"]),
        b"",
    );
    let text: String = words(&observed);
    let deepest: usize = deepest_level(counter.as_path());
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "the wrapper must end the chain itself, deepest level {deepest}: {text}");
    assert!(
        (2..CHAIN_CAP).contains(&deepest),
        "the chain must end after one step and before the launcher's backstop at level {CHAIN_CAP}: deepest level {deepest}: {text}"
    );
    assert!(text.contains("started as Git"), "{text}");
    assert!(text.contains(launchers.join("git").to_string_lossy().as_ref()), "names the `git` on PATH that led back: {text}");
    remove(&fixture);
}

/// What:
///  The same script spelled with `exec`,
/// which keeps the wrapper in the process the marker names.
/// Why:
///  This spelling is the positive control of the control above:
/// it stopped at once before any bound existed,
///  so a bound that stops this one too would be stopping a chain that never ran away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [binWithWrapper, dirWithExecScript] }).code).toBe(2); expect(starts).toBe(1);
/// ```
#[test]
fn a_script_named_git_that_execs_the_wrapper_stops_on_arrival() {
    let fixture: Fixture = fixture("script-exec");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let scripts: PathBuf = directory(&fixture, "scripts");
    let counter: PathBuf = fixture.root.join("starts");
    counting_stand_in(scripts.join("git").as_path(), counter.as_path(), fixture.wrapper.as_path());
    let bin: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[bin.as_path(), scripts.as_path()], &["rev-parse", "HEAD"]);
    let text: String = words(&observed);
    assert_eq!(starts(counter.as_path()), 1, "{text}");
    assert_eq!(observed.code, Some(WRAPPER_STOP_CODE), "{text}");
    assert!(text.contains("started as Git"), "{text}");
    remove(&fixture);
}

/// What:
///  An ordinary user script named `git` that runs the real Git,
/// with no `exec`.
/// Why:
///  A bound on how many wrapper starts one chain may hold must not touch a script that leads to Git:
/// forwarding through it is what `PATH` says to do,
/// and the command must be Git's own,
/// with Git's output and status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run({ PATH: [binWithWrapper, dirWithScriptToGit] }, ['rev-parse', 'HEAD'])).toMatchObject({ code: 0, stdout: head });
/// ```
#[test]
fn a_script_named_git_that_runs_the_real_git_forwards_to_it() {
    let fixture: Fixture = fixture("script-real-git");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let scripts: PathBuf = directory(&fixture, "scripts");
    let counter: PathBuf = fixture.root.join("starts");
    counting_child_stand_in(scripts.join("git").as_path(), counter.as_path(), Path::new(REAL_GIT), CHAIN_CAP);
    let expected: std::process::Output = git(&fixture, &repo, &["rev-parse", "HEAD"]);
    let bin: PathBuf = fixture.root.join("bin");
    let observed: Observed = run_with_only(&fixture, repo.as_path(), &[bin.as_path(), scripts.as_path()], &["rev-parse", "HEAD"]);
    let text: String = words(&observed);
    assert_eq!(observed.code, Some(0), "{text}");
    assert_eq!(observed.stdout, expected.stdout, "Git's own answer through the script");
    assert_eq!(starts(counter.as_path()), 1, "the script is started once: {text}");
    remove(&fixture);
}
