//! PTY process management for the interactive terminal.

/// What:
///      `use std::{...};` imports standard-library modules.
///  `env` reads process
///           environment,
///  `Read` and `Write` are byte-stream traits,
///  `Sender` is a
///           channel sender,
///  and `thread` starts the background reader.
/// Why:
///       PTY output must be read off the UI thread and forwarded as byte chunks.
use std::{
    env,
    io::{Read, Write},
    sync::mpsc::Sender,
    thread,
};

/// What:
///      `use anyhow::Result;` imports `anyhow`'s application error result
///           alias.
///  Sibling typed results name the exact error type.
/// Why:
///       PTY setup can fail with several upstream error types,
///  and callers only
///           need one propagated diagnostic channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Result<T> = T; // failures throw Error objects
/// ```
use anyhow::Result;

/// What:
///      `use portable_pty::{...};` imports the selected PTY abstraction.
///           `PtySize` describes terminal dimensions,
///  `CommandBuilder` describes
///           the shell process,
///  and `MasterPty` owns the read/write side.
/// Why:
///       This crate avoids hand-written `forkpty` and `ioctl` setup.
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

/// What:
///      `use crate::engine::ViewportGeometry;` imports the shared terminal
///           size model from the engine module.
/// Why:
///       Ghostty and the PTY must receive the same rows,
///  cols,
///  and pixel size.
use crate::engine::ViewportGeometry;

/// What:
///      `pub enum PtyEvent` declares messages from the reader thread to the UI
///           thread.
///  The sibling shape would be one struct,
///  but this is a tagged
///           union with separate output and stopped cases.
/// Why:
///       The UI timer needs to distinguish shell bytes from reader lifecycle text.
#[derive(Debug)]
pub enum PtyEvent {
    /// What:
    ///      `Output(Vec<u8>)` carries one owned byte chunk from the PTY.
    /// Why:
    ///       The UI thread feeds these bytes into libghostty-vt.
    Output(
        /// What:
        ///      `Vec<u8>` is the single,
        ///  unnamed `.0` field of this tuple variant:
        ///           an owned,
        ///  growable,
        ///  heap-allocated array of bytes (`u8` is an
        ///           unsigned 8-bit integer).
        ///  Siblings the reader might expect:
        ///  `&[u8]`,
        ///           a borrowed read-only view,
        ///  and `[u8; N]`,
        ///  a fixed-size stack array.
        /// Why:
        ///       `Vec<u8>` (not the borrowed `&[u8]` nor a fixed `[u8; N]`) because
        ///           this value is moved through the `Sender` channel from the reader
        ///           thread to the UI thread:
        ///  it must own its bytes,
        ///  since a borrow
        ///           would dangle once the reader thread's buffer is reused,
        ///  and the
        ///           chunk length varies per read.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// bytes: Uint8Array;
        /// ```
        Vec<u8>,
    ),
    /// What:
    ///      `ReaderStopped(String)` carries human-readable reader shutdown text.
    /// Why:
    ///       The app can show EOF or read errors in the status line.
    ReaderStopped(
        /// What:
        ///      `String` is the single,
        ///  unnamed `.0` field of this tuple variant:
        ///           an owned,
        ///  heap-allocated,
        ///  growable UTF-8 text buffer holding the
        ///           reader-shutdown message.
        ///  Sibling `&str` is a borrowed view that
        ///           does not own its bytes.
        /// Why:
        ///       `String` (not `&str`) because this value is moved through the
        ///           `Sender` channel to the UI thread;
        ///  a borrowed `&str` would dangle
        ///           once the reader thread's stack frame returns,
        ///  so the message must
        ///           own its bytes.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// message: string;
        /// ```
        String,
    ),
}

/// What:
///      `pub struct PtySession` owns the live PTY process.
///  Boxed trait objects
///           hide platform-specific Unix or Windows implementations from callers.
/// Why:
///       Main only needs resize,
///  write,
///  and cleanup operations.
pub struct PtySession {
    /// What:
    ///      `master: Box<dyn MasterPty + Send>` owns the master side of the PTY.
    ///           `Box<dyn Trait>` is a heap-allocated trait object:
    ///  a value reached
    ///           through a pointer whose concrete type is hidden,
    ///  exposing only the
    ///           `MasterPty` interface (like a TS interface-typed value).
    ///  `+ Send` is a
    ///           marker promising the value is safe to move to another thread.
    ///  Siblings
    ///           the reader might expect:
    ///  `Rc<dyn MasterPty>` and `Arc<dyn MasterPty>`,
    ///           which add reference counting for shared ownership.
    /// Why:
    ///       `Box` (not `Rc`/`Arc`) because the session is the single owner of the
    ///           master;
    ///  trait-object boxing hides whether the platform built a Unix or
    ///           Windows PTY,
    ///  and the `+ Send` marker certifies the handle is thread-safe
    ///           to move,
    ///  which `portable_pty`'s API requires here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// master: MasterPty;
    /// ```
    master: Box<dyn MasterPty + Send>,
    /// What:
    ///      `writer: Box<dyn Write + Send>` owns the one writable handle to PTY
    ///           input.
    ///  `Box<dyn Write>` is a heap-allocated trait object exposing only
    ///           the `Write` byte-sink interface;
    ///  `+ Send` marks it safe to move across
    ///           threads.
    ///  Siblings `Rc<dyn Write>` / `Arc<dyn Write>` add shared,
    ///           reference-counted ownership.
    /// Why:
    ///       `Box` (not `Rc`/`Arc`) because the session uniquely owns the writer;
    ///           boxing the trait object hides the platform stream type,
    ///  and `+ Send`
    ///           keeps it movable between threads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writer: Writable;
    /// ```
    writer: Box<dyn Write + Send>,
    /// What:
    ///      `child: Box<dyn Child + Send + Sync>` owns the spawned shell process
    ///           handle.
    ///  `Box<dyn Child>` is a heap-allocated trait object exposing the
    ///           `Child` process interface;
    ///  `+ Send` marks it movable across threads and
    ///           `+ Sync` marks it safe to share by reference across threads.
    ///  Siblings
    ///           `Rc<dyn Child>` / `Arc<dyn Child>` would add reference counting.
    /// Why:
    ///       `Box` (not `Rc`/`Arc`) because the session is the sole owner of the
    ///           child and kills it on drop;
    ///  the `+ Send + Sync` markers certify the
    ///           handle is thread-safe to move and to share by reference,
    ///  which
    ///           `portable_pty`'s API requires here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// child: ChildProcess;
    /// ```
    child: Box<dyn Child + Send + Sync>,
}

/// What:
///      `impl PtySession` defines methods on the live PTY session.
/// Why:
///       Keep PTY creation and IO operations in one module.
impl PtySession {
    /// What:
    ///      `pub fn spawn_shell(...) -> Result<Self>` creates a shell PTY and
    ///           returns it or an `anyhow::Error`.
    ///  `Self` means `PtySession`.
    /// Why:
    ///       The binary should call one function to start an interactive shell.
    pub fn spawn_shell(
        geometry: ViewportGeometry,
        event_sender: Sender<PtyEvent>,
    ) -> Result<Self> {
        // What:     `let shell = default_shell_path()` reads the user's shell path.
        // Why:      A terminal should start the shell the user configured.
        let shell = default_shell_path();
        // What:     `let mut command = CommandBuilder::new(shell.as_str())` builds a
        //           mutable command with argv[0] set to the shell path.
        // Why:      PTY spawn needs a command object, not a raw string.
        let mut command = CommandBuilder::new(shell.as_str());
        // What:     `command.env(...)` overrides one environment variable inherited by
        //           the child shell.
        // Why:      Full-screen terminal apps query `TERM` to choose escape behavior.
        command.env("TERM", "xterm-256color");
        // What:     `command.env(...)` advertises true-color support to terminal apps.
        // Why:      libghostty-vt resolves RGB colors, so child apps may emit them.
        command.env("COLORTERM", "truecolor");
        // What:     `Self::spawn_command(...)` delegates to the testable command path.
        // Why:      Unit tests can spawn a deterministic command instead of a real shell.
        return Self::spawn_command(geometry, command, event_sender)
    }

    /// What:
    ///      `pub fn spawn_command(...) -> Result<Self>` creates a PTY around an
    ///           arbitrary command using the shared `anyhow` error channel.
    /// Why:
    ///       Tests and future command-launch options need the same PTY wiring.
    pub fn spawn_command(
        geometry: ViewportGeometry,
        command: CommandBuilder,
        event_sender: Sender<PtyEvent>,
    ) -> Result<Self> {
        // What:     `native_pty_system()` selects the platform PTY implementation.
        // Why:      Linux, macOS, and Windows PTYs differ behind this API.
        let pty_system = native_pty_system();
        // What:     `pty_system.openpty(...)` allocates a master/slave PTY pair.
        // Why:      The shell runs on the slave side while the app reads and writes master.
        let pair = pty_system.openpty(pty_size_from_geometry(geometry))?;
        // What:     `pair.slave.spawn_command(command)?` starts the child on the slave PTY.
        // Why:      The child then sees stdin, stdout, and stderr as a real terminal.
        let child = pair.slave.spawn_command(command)?;
        // What:     `pair.master.try_clone_reader()?` creates a readable handle to PTY output.
        // Why:      The reader moves to a background thread while the master stays owned here.
        let reader = pair.master.try_clone_reader()?;
        // What:     `pair.master.take_writer()?` creates the one writable handle to PTY input.
        // Why:      Keyboard bytes are written through this object.
        let writer = pair.master.take_writer()?;
        // What:     `spawn_reader_thread(reader, event_sender)` starts output forwarding.
        // Why:      Blocking reads must not stall Slint animations or input.
        spawn_reader_thread(reader, event_sender);
        // What:     `Ok(Self { ... })` wraps the live PTY handles in a session object.
        // Why:      The caller can resize, write, and kill the session by dropping it.
        return Ok(Self {
            master: pair.master,
            writer,
            child,
        })
    }

    /// What:
    ///      `pub fn resize(&self, geometry: ViewportGeometry) -> Result<()>`
    ///           sends updated terminal size to the PTY master through `anyhow`.
    /// Why:
    ///       Shells and full-screen apps need SIGWINCH and updated rows or cols.
    pub fn resize(&self, geometry: ViewportGeometry) -> Result<()> {
        // What:     `self.master.resize(...)` asks the OS PTY to update its winsize.
        // Why:      Child programs read this size and repaint to the new viewport.
        self.master.resize(pty_size_from_geometry(geometry))?;
        // What:     `Ok(())` returns success with no payload.
        // Why:      Resize completed.
        return Ok(())
    }

    /// What:
    ///      `pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()>`
    ///           writes borrowed bytes to the PTY input stream.
    /// Why:
    ///       Keyboard input reaches the shell through the master writer.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        // What:     `self.writer.write_all(bytes)?` writes the entire borrowed byte slice.
        // Why:      Partial writes would corrupt escape sequences and UTF-8 text.
        self.writer.write_all(bytes)?;
        // What:     `self.writer.flush()` asks the stream to push buffered bytes now.
        // Why:      Interactive input should reach the shell immediately.
        return self.writer.flush()
    }
}

/// What:
///      `impl Drop for PtySession` defines cleanup when the session is destroyed.
/// Why:
///       Closing the terminal window should terminate the child shell.
impl Drop for PtySession {
    /// What:
    ///      `fn drop(&mut self)` is Rust's destructor hook.
    /// Why:
    ///       It runs automatically when the last `PtySession` owner goes away.
    fn drop(&mut self) {
        // What:     `let _ = self.child.kill();` calls kill and intentionally ignores
        //           cleanup errors. `let _ =` is Rust's explicit discard pattern.
        // Why:      Destructors cannot return errors, and the child may already be gone.
        let _ = self.child.kill();
    }
}

/// What:
///      `fn default_shell_path() -> String` returns an owned shell path string.
/// Why:
///       `CommandBuilder::new` needs a stable string while building the command.
fn default_shell_path() -> String {
    // What:     `env::var("SHELL").ok()` reads `$SHELL` and converts `Result` to
    //           `Option`. `filter` rejects empty strings.
    // Why:      Desktop launches may have a configured shell; empty values are unusable.
    let shell = env::var("SHELL").ok().filter(|value| return !value.is_empty());
    // What:     `unwrap_or_else(...)` returns `$SHELL` or allocates `"/bin/sh"`.
    // Why:      POSIX systems should always have `/bin/sh` as a fallback shell.
    return shell.unwrap_or_else(|| return "/bin/sh".to_string())
}

/// What:
///      `fn pty_size_from_geometry(...) -> PtySize` converts engine geometry to
///           portable-pty's size struct.
/// Why:
///       Ghostty and the kernel PTY must agree on rows,
///  cols,
///  and pixel size.
fn pty_size_from_geometry(geometry: ViewportGeometry) -> PtySize {
    // What:     `PtySize { ... }` constructs a size record with rows, cols, and pixels.
    // Why:      PTY resize APIs need text dimensions and can also carry pixel hints.
    return PtySize {
        rows: geometry.rows,
        cols: geometry.cols,
        pixel_width: geometry.cell_width_px as u16 * geometry.cols,
        pixel_height: geometry.cell_height_px as u16 * geometry.rows,
    }
}

/// What:
///      `fn spawn_reader_thread(...)` takes ownership of a reader and sender,
///           then starts a background thread.
/// Why:
///       PTY reads block until the child writes output.
fn spawn_reader_thread(mut reader: Box<dyn Read + Send>, event_sender: Sender<PtyEvent>) {
    // What:     `thread::spawn(move || { ... })` starts a native thread and moves the
    //           reader plus sender into its closure.
    // Why:      The reader must live independently from the UI thread.
    thread::spawn(move || {
        // What:     `let mut buffer = [0u8; 8192]` creates a fixed-size byte buffer.
        // Why:      Reusing one buffer avoids allocating for every read call.
        let mut buffer = [0u8; 8192];
        // What:     `loop { ... }` repeats until EOF, read error, or UI shutdown.
        // Why:      The shell can produce output for the whole window lifetime.
        loop {
            // What:     `match reader.read(&mut buffer)` blocks for PTY output and branches
            //           on bytes read, EOF, or error.
            // Why:      Each branch needs a different event-loop message.
            match reader.read(&mut buffer) {
                // What:     `Ok(0)` means EOF from the PTY.
                // Why:      The shell exited or the PTY closed.
                Ok(0) => {
                    let _ = event_sender.send(PtyEvent::ReaderStopped("PTY closed".to_string()));
                    return;
                }
                // What:     `Ok(read_count)` means the buffer contains that many bytes.
                // Why:      Only the filled prefix should be forwarded.
                Ok(read_count) => {
                    let bytes = buffer[..read_count].to_vec();
                    if event_sender.send(PtyEvent::Output(bytes)).is_err() {
                        return;
                    }
                }
                // What:     `Err(error)` means the OS read call failed.
                // Why:      Surface the failure in the status line instead of silently dying.
                Err(error) => {
                    let message = format!("PTY read failed: {error}");
                    let _ = event_sender.send(PtyEvent::ReaderStopped(message));
                    return;
                }
            }
        }
    });
}

/// What:
///      `#[cfg(test)] #[path = "pty_tests.rs"] mod tests;`
///           declares a test-only submodule whose code lives in the sibling
///           file `pty_tests.rs`.
///  `#[cfg(test)]` gates it to test
///           builds only;
///  `#[path = "..."]` aims the module at a flat sibling
///           file instead of the default `pty/tests.rs`
///           subdirectory lookup.
///  The file stays the `tests` CHILD of
///           pty,
///  so its `use super::*` reaches the module items
///           (including private ones) unchanged.
/// Why:
///       Keep `pty.rs` to production code;
///  the tests live
///           beside it without inflating this file or its max-lines budget
///           (sibling `*_tests.rs` files are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // pty.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "pty_tests.rs"]
mod tests;
