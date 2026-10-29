//! What:
//!  One long-lived `git cat-file --batch` process that answers every object read of an invocation.
//! Why:
//!  Starting Git costs far more than reading one object,
//!  so candidate bytes are read
//!      through a single process whose count does not grow with the number of files.
//!      Blob bytes are remembered by object name:
//!  a name always denotes the same bytes,
//!      so a remembered blob can never become stale.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const reader = startObjectReader(git); const bytes = await reader.blob(oid);
//! ```

/// Import the pure reply reader and the values it produces.
use super::candidate_batch::{BatchReply, ObjectKind, read_batch_reply};
/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// Import the validated object name.
use super::candidate_object::ObjectId;
/// Import the shared child-command builder.
use super::forwarding::git_command;
/// The watch that announces a request that takes long and stops the process at the bound.
use super::child_watch::{ChildWatch, WatchTerms, watched_timeout_text};
/// What:
///  `HashMap<K, V>` is a key-to-value table (siblings:
///  `BTreeMap`,
///  which keeps keys
///       sorted,
///  and `Vec` of pairs).
/// Why:
///   Remembered blobs are looked up by object name and never listed in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blobs = new Map<ObjectId, Uint8Array>();
/// ```
use std::collections::HashMap;
/// `OsString` is owned operating-system text of raw OS bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `BufReader` adds a read buffer to a pipe;
///  `Write` is the trait that gives pipes `.write_all(..)`.
use std::io::{BufReader, Write};
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// The child process record,
///  its two pipes,
///  and the stream selector.
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
/// What:
///  `Rc<T>` is a shared,
///  read-only handle to one heap value,
///  counted so the value
///       lives until its last handle is gone (siblings:
///  `Box<T>`,
///  a single owner,
///  and
///       `Arc<T>`,
///  the thread-safe form).
/// Why:
///   The reader keeps a blob while callers also hold it;
///  copying large files for
///       each holder would double their memory.
///  Nothing here crosses threads,
///  so `Arc`
///       would add cost for no benefit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Every object reference in TS already behaves like this.
/// ```
use std::rc::Rc;

/// What:
///  The request and reply pipes of a running reader.
/// Why:
///   Both are closed together:
///  after any reply failure the stream can no longer be
///       trusted,
///  and closing both ends lets the process stop whatever it was doing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReaderPipes = { input: Writable; output: Readable };
/// ```
struct ReaderPipes {
    /// Where request lines are written.
    input: ChildStdin,
    /// Where replies are read,
    ///  through a buffer so header lines do not cost one read per byte.
    output: BufReader<ChildStdout>,
}

/// What:
///  A running object reader and the blobs it has already read.
///       `Option<ReaderPipes>` is "open pipes or nothing";
///  nothing means closed.
///       `Rc<[u8]>` is a shared read-only byte list.
/// Why:
///   One value owns the process for its whole life,
///  so the process is always
///       stopped and collected when the value is dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ObjectReader { #child: ChildProcess; #pipes?: ReaderPipes; #blobs: Map<ObjectId, Uint8Array> }
/// ```
pub struct ObjectReader {
    /// The `git cat-file --batch` process.
    child: Child,
    /// Open pipes,
    ///  or nothing once the reader was closed after a failure.
    pipes: Option<ReaderPipes>,
    /// Blob bytes already read,
    ///  by object name.
    blobs: HashMap<ObjectId, Rc<[u8]>>,
    /// Announces a request that takes long and stops the process when one crosses the bound.
    watch: ChildWatch,
}

/// What:
///  Build the failure for a reader that can no longer answer.
/// Why:
///   The message tells the person how to see Git's own explanation,
///  because the
///       reader's error stream is discarded (an unread pipe could stall Git).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readerEnded(detail: string): CandidateError;
/// ```
fn reader_ended(detail: &str) -> CandidateError {
    return CandidateError::new(
        CandidateFailure::ReaderEnded,
        format!(
            "cli-git could not read a Git object: the git cat-file --batch process {detail}. \
             Run `git cat-file --batch-check` in this repository to see Git's own diagnostic."
        )
        .as_str(),
    );
}

/// What:
///  Start the reader for the repository the caller's global options select.
///       `&[OsString]` borrows the arguments before the subcommand,
///  unchanged;
///       `&[(OsString, OsString)]` borrows added environment pairs.
///       `Result<T, E>` is "a value or a failure".
/// Why:
///   Replaying the global prefix makes the reader open the same repository as the
///       forwarded command.
///  The overlay may point it at a private object directory,
///  so
///       one reader is bound to one overlay for its whole life.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startObjectReader(realGit, globalPrefix, overlay): ObjectReader;
/// ```
pub fn start_object_reader(
    real_git: &Path,
    global_prefix: &[OsString],
    overlay: &[(OsString, OsString)],
) -> Result<ObjectReader, CandidateError> {
    return start_object_reader_with(real_git, global_prefix, overlay, WatchTerms::from_environment());
}

/// What:
///  `start_object_reader` with the notice cadence, the bound and the notice sink of its requests chosen by the caller.
/// Why:
///   The production entry reads them from the environment;
///  tests choose short ones and read the notices from a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startObjectReader(realGit, globalPrefix, overlay, terms): ObjectReader;
/// ```
pub fn start_object_reader_with(
    real_git: &Path,
    global_prefix: &[OsString],
    overlay: &[(OsString, OsString)],
    terms: WatchTerms,
) -> Result<ObjectReader, CandidateError> {
    // `.to_vec()` copies the borrowed prefix into an owned, growable argument list.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    arguments.push(OsString::from("cat-file"));
    arguments.push(OsString::from("--batch"));
    // What: `Stdio::piped()` connects a stream to this process; `Stdio::null()` discards it.
    //       `.spawn()` starts the child without waiting; `match` reads its `Result`.
    // Why:  Requests and replies travel over the two pipes. Standard error is discarded
    //       because nothing reads it while replies are awaited, and a full unread pipe
    //       would block Git forever.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const child = spawn(realGit, args, { stdio: ['pipe', 'pipe', 'ignore'] });
    // ```
    let started: std::io::Result<Child> = git_command(real_git, arguments.as_slice(), overlay)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn_retrying();
    let mut child: Child = match started {
        Ok(spawned) => spawned,
        Err(error) => {
            // `Err(...)` is the failure variant.
            return Err(CandidateError::new(
                CandidateFailure::GitNotStarted,
                format!(
                    "cli-git could not start git cat-file --batch to read Git objects: {error}."
                )
                .as_str(),
            ));
        }
    };
    // What: `.take()` moves each pipe out of the child record, leaving nothing behind.
    //       `let (Some(a), Some(b)) = (x, y) else { ... };` unwraps both or exits.
    // Why:  The reader owns the pipes so it can close them independently of the process record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const { stdin, stdout } = child;
    // ```
    let (Some(input), Some(output)) = (child.stdin.take(), child.stdout.take()) else {
        // Both streams were requested as pipes, so this is not expected; the child is still collected.
        let _ = child.kill();
        let _ = child.wait();
        return Err(reader_ended("started without its request and reply pipes"));
    };
    // `Ok(...)` is the success variant; `Some(...)` is the "present" variant.
    let watch: ChildWatch = ChildWatch::start("git cat-file --batch", child.id(), terms);
    return Ok(ObjectReader {
        child,
        pipes: Some(ReaderPipes {
            input,
            output: BufReader::new(output),
        }),
        blobs: HashMap::new(),
        watch,
    });
}

/// What:
///  `impl ObjectReader { ... }` attaches the read operations,
///  like class methods.
/// Why:
///   Only complete object names and the fixed word `HEAD` are ever written as
///       requests,
///  so no caller text can inject a second request line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ObjectReader { blob(oid) {} headCommit() {} }
/// ```
impl ObjectReader {
    /// What:
    ///  Send one request line and read its reply.
    ///  `&mut self` borrows the reader
    ///       for changing:
    ///  its pipes advance and may be closed.
    /// Why:
    ///   Strict alternation (one request,
    ///  then its whole reply) can never fill a
    ///       pipe in both directions at once.
    ///  Any failure closes the pipes,
    ///  because the
    ///       rest of the stream could be object content that looks like a reply.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #request(name: Buffer): Promise<BatchReply>
    /// ```
    fn request(&mut self, name: &[u8]) -> Result<BatchReply, CandidateError> {
        // The watch announces a request that takes long and stops the process when one crosses the bound.
        self.watch.begin();
        let reply: Result<BatchReply, CandidateError> = self.exchange(name);
        self.watch.end();
        if reply.is_err() && self.watch.expired() {
            self.pipes = None;
            return Err(CandidateError::new(
                CandidateFailure::ReaderEnded,
                format!(
                    "cli-git could not read a Git object: {}",
                    watched_timeout_text("git cat-file --batch", self.watch.limit())
                )
                .as_str(),
            ));
        }
        return reply;
    }

    /// What:
    ///  One request line and its reply, without the watch.
    /// Why:
    ///   `request` marks the wait around it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async #exchange(name: Buffer): Promise<BatchReply>
    /// ```
    fn exchange(&mut self, name: &[u8]) -> Result<BatchReply, CandidateError> {
        // `.as_mut()` lends the open pipes for changing without taking them out.
        let Some(pipes) = self.pipes.as_mut() else {
            return Err(reader_ended("was closed after an earlier failure"));
        };
        let mut line: Vec<u8> = name.to_vec();
        line.push(b'\n');
        // `if let Err(error) = ...` runs the block only when the write failed.
        if let Err(error) = pipes.input.write_all(line.as_slice()) {
            // `None` is the "absent" variant: assigning it drops, and so closes, both pipes.
            self.pipes = None;
            return Err(reader_ended(
                format!("stopped accepting requests ({error})").as_str(),
            ));
        }
        match read_batch_reply(&mut pipes.output, name) {
            Ok(reply) => return Ok(reply),
            Err(error) => {
                self.pipes = None;
                return Err(error);
            }
        }
    }

    /// What:
    ///  The exact bytes of one blob,
    ///  read once and shared afterwards.
    ///       `Rc::clone(..)` makes another handle to the same bytes without copying them.
    /// Why:
    ///   The same blob often backs several paths and several policies;
    ///  each further
    ///       use costs neither a request nor a copy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async blob(oid: ObjectId): Promise<Uint8Array>
    /// ```
    pub fn blob(&mut self, object: &ObjectId) -> Result<Rc<[u8]>, CandidateError> {
        // `.get(object)` is `Option<&Rc<[u8]>>`: the remembered bytes, if any.
        if let Some(bytes) = self.blobs.get(object) {
            return Ok(Rc::clone(bytes));
        }
        // A trailing `?` returns the failure to our caller, or unwraps the reply.
        let reply: BatchReply = self.request(object.as_str().as_bytes())?;
        // What: `match` on the reply; `{ kind, bytes, .. }` binds two fields and ignores the rest.
        // Why:  Only a blob is file content; a missing or differently typed object is a
        //       failure the caller must not read as empty content.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (reply.kind === 'missing') throw ...; if (reply.type !== 'blob') throw ...;
        // ```
        match reply {
            BatchReply::Missing => {
                return Err(CandidateError::new(
                    CandidateFailure::ObjectMissing,
                    format!(
                        "cli-git could not read candidate content: Git has no object {}. \
                         Run `git fsck` to check the object store.",
                        object.as_str()
                    )
                    .as_str(),
                ));
            }
            BatchReply::Found { kind, bytes, .. } => {
                if kind != ObjectKind::Blob {
                    return Err(CandidateError::new(
                        CandidateFailure::ObjectKindUnexpected,
                        format!(
                            "cli-git could not read candidate content: object {} is not a blob.",
                            object.as_str()
                        )
                        .as_str(),
                    ));
                }
                // `Rc::from(bytes)` moves the owned bytes into shared storage.
                let shared: Rc<[u8]> = Rc::from(bytes);
                // `.clone()` copies the name so the table owns its key.
                self.blobs.insert(object.clone(), Rc::clone(&shared));
                return Ok(shared);
            }
        }
    }

    /// What:
    ///  The commit `HEAD` names,
    ///  or nothing when the branch has no commit yet.
    ///       `Option<ObjectId>` is "a commit name or nothing".
    /// Why:
    ///   The staged listing compares against this commit.
    ///  Asking the running
    ///       reader costs no further process.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async headCommit(): Promise<ObjectId | undefined>
    /// ```
    pub fn head_commit(&mut self) -> Result<Option<ObjectId>, CandidateError> {
        match self.request(b"HEAD")? {
            BatchReply::Missing => return Ok(None),
            BatchReply::Found { object, kind, .. } => {
                if kind != ObjectKind::Commit {
                    return Err(CandidateError::new(
                        CandidateFailure::ObjectKindUnexpected,
                        "cli-git could not list candidate files: HEAD does not name a commit.",
                    ));
                }
                return Ok(Some(object));
            }
        }
    }
}

/// What:
///  `impl Drop` runs when the value goes out of scope,
///  like a `finally` block
///       attached to the value itself.
/// Why:
///   A child that is never waited for stays in the process table as a zombie,
///       which counts against the process limit of long sessions and containers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose]() { this.#pipes = undefined; this.#child.kill(); }
/// ```
impl Drop for ObjectReader {
    /// Close both pipes,
    ///  which ends the process,
    ///  then collect it.
    fn drop(&mut self) {
        self.pipes = None;
        // `let _ =` discards the exit status: every reply was already checked on its own.
        let _ = self.child.wait();
    }
}

/// Process controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_reader_tests.rs"]
mod tests;

/// Stand-in programs that cut replies short,
///  mismatch them,
///  or never answer.
#[cfg(test)]
#[path = "candidate_reader_failure_tests.rs"]
mod failure_tests;
