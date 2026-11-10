//! What:
//!  A stand-in Git that stops the word-level second pass at its own `git check-attr`
//!  and holds it there until the control that watches the pass says it may go on.
//! Why:
//!  The two controls of `binary_word_merge_tests.rs` act while the pass computes its merges,
//!  and they used to find the pass's own attribute query by walking `/proc` inside a fifteen-second window.
//!  That window is a clock nobody can meet under host load:
//!  at `c0bb93d036` one control of the pair failed at 16.06 s,
//!  and on another branch the other failed at 16.29 s,
//!  each passing alone,
//!  because the pass had not reached the query before the window closed
//!  (`doc/planning/git-policy-cli-full-tokio.md`,
//!  "Step T12,
//!  the leaf-first asynchronous Git and the ordered runner that awaits",
//!  which names this stand-in as the proper fix).
//!  The stand-in is the only real Git the run's `PATH` offers:
//!  at that one query it writes a marker file and a word on a pipe,
//!  and it reads the control's word on a second pipe before it runs real Git.
//!  The pass therefore cannot go past the query without the control,
//!  and the control waits on the pass instead of on a clock.
//!  A pipe held at both ends is how each side waits clockless:
//!  a pipe opened for one direction waits for the other end to open,
//!  and one held at both ends neither waits to be opened nor reads the end of its input.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gate = PassGate.prepare(fixture);
//! const [observed, control] = gate.heldRun(fixture, repo, ['merge', '--no-edit', 'topic'], () => writeFileSync(target, resolution));
//! expect(await control).toBe(true);  // it reached the pass, and the pass waited for it
//! ```

use super::supervisor::{IdleHold, hold_idle_judgment};
use super::support::{Fixture, Observed, REAL_GIT, TIME_BOUND_SECONDS, bounded_for, executable, without_hint_fields};
use std::ffi::{OsStr, OsString};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

/// The word the stand-in Git writes once the pass reaches its attribute query.
const REACHED_WORD: &str = "reached";

/// The word the control writes to let the pass go on.
const RELEASED_WORD: &str = "released";

/// The word the run's caller writes when the command ended without reaching the query,
///  so a control waiting for the pass is never left waiting.
const ENDED_WORD: &str = "ended";

/// What:
///  The stand-in Git of one control,
/// and the two pipes the pass and the control meet at.
/// Why:
///   One value per fixture holds the paths,
/// so a control names its gate once and the action it passes names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = { path: `${fixture.root}/bin:${fixture.root}/pass-gate/shim`, reached, release }
/// ```
pub(super) struct PassGate {
    /// The `PATH` of a gated run:
    ///  the wrapper's link directory,
    ///  then the stand-in,
    ///  and nothing else.
    ///  Real Git at a conventional location outranks every other candidate on `PATH`,
    ///  so the stand-in replaces real Git only where no conventional one is offered;
    ///  it starts real Git by absolute path,
    ///  and a merge starts no other program.
    path: OsString,
    /// The pipe the pass writes at its attribute query and the control reads.
    reached: PathBuf,
    /// The pipe the control writes and the pass reads before it goes on.
    release: PathBuf,
}

impl PassGate {
    /// What:
    ///  Make a gate for `fixture`:
    /// its two pipes,
    /// and a stand-in Git named `git` in a directory of its own.
    /// Why:
    ///   The stand-in must be Git to the wrapper,
    /// which takes the first executable file on `PATH` that is not itself,
    /// and it must not sit at a conventional location,
    /// which the wrapper promotes ahead of every other candidate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const root = join(fixture.root, 'pass-gate'); mkdirSync(join(root, 'shim'), { recursive: true });
    /// mkfifo(join(root, 'reached.pipe')); mkfifo(join(root, 'release.pipe'));
    /// writeExecutable(join(root, 'shim/git'), shimScript(root));
    /// ```
    pub(super) fn prepare(fixture: &Fixture) -> PassGate {
        let root: PathBuf = fixture.root.join("pass-gate");
        let shim: PathBuf = root.join("shim");
        std::fs::create_dir_all(shim.as_path()).expect("the stand-in's own directory");
        let reached: PathBuf = root.join("reached.pipe");
        let release: PathBuf = root.join("release.pipe");
        make_pipe(reached.as_path());
        make_pipe(release.as_path());
        executable(shim.join("git").as_path(), shim_script(root.as_path()).as_bytes());
        let mut path: OsString = fixture.root.join("bin").into_os_string();
        path.push(":");
        path.push(shim);
        return PassGate { path, reached, release };
    }

    /// What:
    ///  Run one wrapped command in `repo` with this gate on its `PATH`,
    ///  and let `when` act while the pass waits at its attribute query.
    ///  The control runs `when`,
    ///  releases the pass,
    ///  and says whether it got there at all.
    /// Why:
    ///   An edit a control makes must land after the merge left its conflicts and before the pass computes its merges,
    /// and the pass's attribute query is exactly that point.
    ///  The two words are the only handshake:
    ///  the pass waits for the control however long the control takes,
    ///  and the control waits for the pass however long the pass takes,
    ///  which is what removes the clock and its window.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const child = spawn(join(fixture.root, 'bin/git'), args, { env: { PATH: gate.path } });
    /// const control = watch(gate, when); const observed = await finish(child); wake(control); return [observed, control]
    /// ```
    pub(super) fn held_run(&self, fixture: &Fixture, repo: &Path, arguments: &[&str], when: impl FnOnce() + Send + 'static) -> (Observed, std::thread::JoinHandle<bool>) {
        let mut command: Command = bounded_for(fixture, fixture.root.join("bin/git").as_path(), self.path.as_os_str(), TIME_BOUND_SECONDS);
        command.current_dir(repo).args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        // Both pipes are held at both ends before the wrapper starts, so no open waits for its other end at any point of the run.
        let waiting: std::fs::File = both_ends(self.reached.as_path());
        let release_held: std::fs::File = both_ends(self.release.as_path());
        let child: Child = command.spawn().expect("the wrapper starts");
        let watched: u32 = child.id();
        let release: PathBuf = self.release.clone();
        let control: std::thread::JoinHandle<bool> = std::thread::spawn(move || {
            let mut line: String = String::new();
            let read: std::io::Result<usize> = BufReader::new(waiting).read_line(&mut line);
            if read.is_err() || line.trim_end() != REACHED_WORD {
                return false;
            }
            // The pass holds itself here for this control, so the tree is asleep on purpose: the idle rule would end it.
            let idle: IdleHold = hold_idle_judgment(watched);
            when();
            let mut pipe: std::fs::File = both_ends(release.as_path());
            let wrote: std::io::Result<()> = pipe.write_all(format!("{RELEASED_WORD}\n").as_bytes());
            drop(idle);
            return wrote.is_ok();
        });
        let output: Output = child.wait_with_output().expect("the wrapper finishes");
        // A control still waiting for a word means the pass never reached the query, so the command's own end is that word.
        let mut ended: std::fs::File = both_ends(self.reached.as_path());
        ended.write_all(format!("{ENDED_WORD}\n").as_bytes()).expect("the waiting control hears that the command ended");
        drop(release_held);
        let observed: Observed = Observed {
            code: output.status.code(),
            stdout: without_hint_fields(output.stdout.as_slice()),
            stderr: without_hint_fields(output.stderr.as_slice()),
        };
        return (observed, control);
    }
}

/// What:
///  `path` as one pipe,
///  in the fixture's own directory.
/// Why:
///   `mkfifo` is the only way to make one;
/// the path is handed over as the raw bytes the file system takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = Buffer.from(path); if (mkfifo(name, 0o600) !== 0) throw new Error(String(errno))
/// ```
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a path without a NUL byte");
    // SAFETY:
    // `name` is a live, NUL-terminated C string for the whole call,
    // and `mkfifo` reads it and nothing else;
    // its failure is inspected through the error it left.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "a pipe at {}: {}", path.display(), std::io::Error::last_os_error());
}

/// What:
///  The pipe at `path`,
/// held open for reading and writing at once.
/// Why:
///   A pipe opened for one direction waits for its other end to open,
/// which is the clock this gate removes,
///  and a pipe held at both ends reads its own writer's words when it asks for them and never reaches the end of its input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openSync(path, fs.constants.O_RDWR)
/// ```
fn both_ends(path: &Path) -> std::fs::File {
    return std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|error| panic!("the pipe at {} ({error})", path.display()));
}

/// What:
///  The stand-in Git of the gate at `root`:
/// a `/bin/sh` script that passes every start to real Git,
/// except the pass's one attribute query,
/// which it holds until the control writes its word.
/// Why:
///   The pass asks `git check-attr -z --stdin merge` once,
/// and a commit's content policy asks `check-attr` with its paths as arguments,
/// so the two flags tell the pass's own query apart from every other start.
///  The marker file makes the hold happen once,
/// so a later query of the same run goes straight through.
///  Only shell built-ins appear here,
///  because the gated `PATH` holds nothing the script could run but real Git,
///  and it starts that one by absolute path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `#!/bin/sh\n... if pass's check-attr and marker absent: write marker, write 'reached', read the release; exec ${REAL_GIT} "$@"`
/// ```
fn shim_script(root: &Path) -> String {
    let marker: String = shell_word(root.join("reached.marker").as_path());
    let reached: String = shell_word(root.join("reached.pipe").as_path());
    let release: String = shell_word(root.join("release.pipe").as_path());
    let real_git: String = shell_word(Path::new(REAL_GIT));
    return format!(
        "#!/bin/sh\n\
         # A stand-in for Git: the word-level second pass asks for the merge attribute once, and this file holds that\n\
         # one query until the control watching the pass says it may go on.\n\
         query=no\n\
         standard_input=no\n\
         for argument in \"$@\"; do\n\
         \x20   if [ \"$argument\" = \"check-attr\" ]; then query=yes; fi\n\
         \x20   if [ \"$argument\" = \"--stdin\" ]; then standard_input=yes; fi\n\
         done\n\
         if [ \"$query\" = yes ] && [ \"$standard_input\" = yes ] && [ ! -e {marker} ]; then\n\
         \x20   : > {marker}\n\
         \x20   printf '%s\\n' '{REACHED_WORD}' > {reached}\n\
         \x20   read released < {release}\n\
         fi\n\
         exec {real_git} \"$@\"\n"
    );
}

/// What:
///  `path` as one shell word.
/// Why:
///   The stand-in is started by its own interpreter with no quoting of ours,
/// so a directory named with a space or a quote must still name the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `'${path.replaceAll("'", "'\\''")}'`
/// ```
fn shell_word(path: &Path) -> String {
    let text: &OsStr = path.as_os_str();
    return format!("'{}'", text.to_string_lossy().replace('\'', "'\\''"));
}
