//! Loads the selected track again when its file was replaced on disk, where it was.
//!
//! What: `impl Controller` methods that notice a replaced selected file and swap its decoder, peak and gain in place.
//! Why: D113 makes the library follow the disk. A playing file that was replaced used to keep the old audio, peak and gain
//! until the next load, and a paused one restarted from zero; now both follow the new file without a toast or a jump.

/// What: `use crate::command::Update;` brings the engine-to-window message enum into scope. `crate::` means "from the
/// root of this crate" (the package being compiled), and `command` is the sibling file `src/command.rs`. Sibling forms:
/// `super::` (the parent module) and `self::` (this module).
/// Why: the reload ends by telling the window the new length and position, and those are `Update` values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Update } from "./command";
/// ```
use crate::command::Update;
/// What: `use crate::controller::Controller;` imports the playback state machine type that the methods of this impl block attach to.
/// Why: Rust only allows `impl` blocks on a type that is named in the file, so the type must be imported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Controller } from "./controller";
/// ```
use crate::controller::Controller;
/// What: `use crate::decode::Source;` imports the trait (an interface) that every audio decoder implements.
/// Why: `Box<dyn Source>` in the method names the type of the freshly opened decoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Source } from "./decode";
/// ```
use crate::decode::Source;

/// What: `pub(crate) fn resume_position(position: f64, new_duration: f64, was_playing: bool) -> f64` says where the
/// replaced file starts: the old position when the new file reaches it, else the end (playing) or zero (paused).
/// `pub(crate)` makes the function visible inside this crate only (siblings: plain `pub` is visible to other crates,
/// and no keyword is private to the file). `f64` is a 64-bit floating point number, the same kind as TypeScript's
/// `number` (siblings: `f32` is half the width, `i32` and `u64` are integers). `bool` is TypeScript's `boolean`.
/// Why: a playing track keeps its place and, when the new file is shorter, ends as if it had played out. A paused track
/// has no end to reach, so a position the new file does not contain is meaningless and it starts again. An unknown new
/// length (zero) cannot clamp anything. `f64` is used, not `f32`, because the audio clock counts seconds over hours and
/// an `f32` would lose sub-millisecond places; a position is never an integer, so `u64` does not fit.
/// Gotcha: the function is pure (no `&mut`, no I/O), so unlike a TypeScript method it cannot read or change any
/// controller state; every input arrives as an argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resumePosition = (pos: number, dur: number, playing: boolean): number => dur <= 0 ? pos : pos < dur ? pos : playing ? dur : 0;
/// ```
pub(crate) fn resume_position(position: f64, new_duration: f64, was_playing: bool) -> f64 {
    // What:     `if new_duration <= 0.0 || position < new_duration { ... }` is an ordinary condition. Rust needs no
    //           parentheses around it, and the body must be braces. `0.0` is a float literal; writing `0` here would not
    //           compile because `f64` and integers never mix without a cast.
    // Why:      An unknown length (zero or less) or a position inside the new file both mean "keep the old position".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (newDuration <= 0 || position < newDuration) {
    // ```
    if new_duration <= 0.0 || position < new_duration {
        // What:     `return position.max(0.0);` leaves the function early with the larger of the position and zero.
        //           `.max(0.0)` is a method on `f64` (siblings: `.min`, `.clamp`). `return` with a trailing `;` is the
        //           explicit form of leaving early.
        // Why:      A negative position can never be where a track starts, so it is raised to zero.
        // Gotcha:   `f64::max` is not `Math.max`: if one side is `NaN` (not a number) it returns the other side instead
        //           of `NaN`, which is the behaviour wanted here.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return Math.max(position, 0);
        // ```
        return position.max(0.0);
    }
    // What:     `return if was_playing { new_duration } else { 0.0 };` returns the value of an `if` expression. In Rust
    //           `if` produces a value, so it plays the part of TypeScript's `cond ? a : b`; both branches must have the
    //           same type (`f64` here), and neither branch ends in `;`, because a trailing `;` would discard the value.
    // Why:      The position is past the new file's end: a playing track ends there, a paused one starts again.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return wasPlaying ? newDuration : 0;
    // ```
    return if was_playing { new_duration } else { 0.0 };
}

/// What: `impl Controller { ... }` opens a block of methods for the `Controller` struct. A struct holds data; the
/// methods that work on it are written in separate `impl` blocks, and one type may have many such blocks, even in
/// different files.
/// Why: the controller is large, so its methods are split over several files; this block holds the ones for a replaced
/// selected file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Like adding methods to a class in a second file (declaration merging):
/// interface Controller { reloadCurrentWhenItsFileChanged(): void }
/// ```
impl Controller {
    /// What: `pub(crate) fn reload_current_when_its_file_changed(&mut self)` opens the selected track's file again when
    /// its size or modified time differs from the file the decoder was opened on, and swaps it in at the same position,
    /// keeping the playing state, the page and the volume. `&mut self` means "this method borrows the controller and may
    /// change it", the way a TypeScript method may assign to `this`.
    /// Why: D113. The file is opened first and swapped only when it opens, so a replacement that is still being written
    /// leaves the old audio playing and is tried again at the next event; nothing is skipped and no toast is shown. The
    /// peak and gain are worked out again for the new file, and the seek bar takes the new length.
    /// Gotcha: `&mut self` is an exclusive borrow, so no other code can touch the controller while this method runs; the
    /// compiler enforces what TypeScript leaves to discipline.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reloadCurrentWhenItsFileChanged(): void {
    ///   const [path, key] = this.loadedFile ?? []; if (!path || path !== this.queue.currentPath()) return;
    ///   if (fingerprint(path) === key) return;
    ///   const source = tryOpen(path); if (!source) return;
    ///   const pos = this.audibleSecs(); const dur = this.installSourceSilently(source, path);
    ///   this.seek(resumePosition(pos, dur, this.playing)); this.emit({ kind: "duration", secs: dur });
    /// }
    /// ```
    pub(crate) fn reload_current_when_its_file_changed(&mut self) {
        // What:     `let Some(current) = self.queue.current_path().cloned() else { return; };` is a `let ... else`. The
        //           queue's `current_path()` answers `Option<&PathBuf>` (an optional borrowed path); `Option` is Rust's
        //           stand-in for `T | null`, with the variants `Some(value)` and `None`. `.cloned()` copies the borrowed
        //           path into an owned `PathBuf` (an owned, growable path; sibling: `&Path` is the borrowed view), and the
        //           `let Some(current) = ... else` pattern binds `current` when the value is `Some` and runs the `else`
        //           block (which must leave the function) when it is `None`.
        // Why:      With no selected track there is nothing to reload. The path is copied so that `current` does not keep
        //           the queue borrowed while the method changes other controller fields.
        // Gotcha:   `.cloned()` here is a real copy of the path text, not a second reference to the same string.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const current = this.queue.currentPath();
        // if (current == null) return;
        // ```
        let Some(current) = self.queue.current_path().cloned() else {
            // What:     `return;` leaves the method with no value (the method's return type is `()`, Rust's empty
            //           value, close to `void`).
            // Why:      No selected track means no work.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return;
            // ```
            return;
        };
        // What:     `let Some((loaded_path, loaded_key)) = self.loaded_file.clone() else { return; };` is the same
        //           `let ... else` for `self.loaded_file`, an `Option<(PathBuf, u64)>`: either nothing, or a pair of the
        //           path the decoder was opened on and its fingerprint. `(loaded_path, loaded_key)` splits the pair into
        //           two names, like TypeScript's array destructuring. `u64` is an unsigned 64-bit integer (siblings:
        //           `u32`, `i64`, `usize`). `.clone()` duplicates the pair so the controller keeps its own copy.
        // Why:      The saved pair says which file the decoder is playing and what that file looked like when it was
        //           opened; the fingerprint is a 64-bit hash, so `u64` is the natural width and `u32` would collide more
        //           often.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.loadedFile == null) return;
        // const [loadedPath, loadedKey] = this.loadedFile;
        // ```
        let Some((loaded_path, loaded_key)) = self.loaded_file.clone() else {
            return;
        };
        // What:     `if loaded_path != current || self.source.is_none() { return; }` compares two paths with `!=` (Rust
        //           compares the path contents, as TypeScript would compare two strings with `!==`) and asks whether the
        //           decoder slot is empty with `.is_none()` (the `Option` test for "is null").
        // Why:      A decoder opened on another file, or no decoder at all, means the saved pair is stale, so this
        //           selection is not the file that is loaded and there is nothing to swap.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (loadedPath !== current || this.source == null) return;
        // ```
        if loaded_path != current || self.source.is_none() {
            return;
        }
        // What:     `let Some(now) = crate::peakcache::fingerprint(&current) else { ... };` calls the fingerprint
        //           function with `&current`, a borrow: `&` lends the path read-only and keeps ownership here.
        //           The function answers `Option<u64>` and the `let ... else` unwraps it. The path is spelled from the
        //           crate root (`crate::peakcache::fingerprint`) because it was not imported by a `use`.
        // Why:      Taking the fingerprint of the file as it is now (a hash of path, size and modified time) is how a
        //           replacement is noticed without reading the audio; `None` means the file cannot be looked at (it is
        //           gone, or unreadable).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const now = fingerprint(current);
        // if (now == null) { /* log */ return; }
        // ```
        let Some(now) = crate::peakcache::fingerprint(&current) else {
            // What:     `tracing::debug!(path = %current.display(), "...");` writes one debug log line. `tracing::debug!`
            //           is a macro (code that expands at compile time; the `!` marks it). `path = %current.display()` adds
            //           a named field to the line; the `%` asks for the human-readable (`Display`) form, because a path
            //           has no built-in text form and `.display()` supplies one.
            // Why:      The repo logs every branch decision; this one explains why the old file stays loaded.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // logger.debug({ path: current }, "the selected file cannot be looked at; it stays as loaded");
            // ```
            tracing::debug!(path = %current.display(), "the selected file cannot be looked at; it stays as loaded");
            return;
        };
        // What:     `if now == loaded_key { return; }` compares the two fingerprints, both `u64`, with `==`.
        // Why:      The same fingerprint means the file did not change, so the decoder already plays the right audio.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (now === loadedKey) return;
        // ```
        if now == loaded_key {
            return;
        }
        // What:     `let source: Box<dyn Source> = match crate::decode::open(&current) { Ok(source) => source, Err(error) => {...} };`
        //           opens the changed file. `open` answers a `Result`, Rust's stand-in for a function that may throw:
        //           `Ok(value)` is success and `Err(error)` is failure. `match` takes the answer apart, one arm per
        //           variant; here the success arm hands the decoder out and the failure arm logs and leaves. The type
        //           `Box<dyn Source>` is an owning pointer to any decoder that implements the `Source` trait (siblings:
        //           `Rc<T>` and `Arc<T>` share ownership; a plain `T` needs its size known at compile time).
        // Why:      The new file is opened before anything is swapped, so a half-written replacement fails here and
        //           leaves the old audio untouched. `Box` is used, not `Rc` or `Arc`, because exactly one owner (the
        //           controller) will hold the decoder, and `dyn` is needed because Opus and the other formats are
        //           different concrete types.
        // Gotcha:   `match` must cover every variant of the `Result`; leaving out `Err` would not compile.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let source: Source;
        // try { source = open(current); } catch (error) { logger.warn(...); return; }
        // ```
        let source: Box<dyn Source> = match crate::decode::open(&current) {
            // What:     `Ok(source) => source,` is the success arm: it names the decoder `source` and the arm's value is
            //           that same decoder, which becomes the value of the whole `match`.
            // Why:      Hand the opened decoder to the `let` that is waiting for it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // the value of the try block: source
            // ```
            Ok(source) => source,
            // What:     `Err(error) => { ... }` is the failure arm: it names the failure `error`, logs it, and leaves.
            //           The arm ends in `return;`, so it never produces a value, which is allowed because leaving the
            //           function has no result to give.
            // Why:      A replacement that does not open yet is retried at the next watcher event.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // catch (error) { ...; return; }
            // ```
            Err(error) => {
                // What:     The `tracing::warn!` line is the warning-level log call. `error = %error` adds the failure
                //           text as a field with the `%` display form, like `path = %current.display()`.
                // Why:      The failure is expected while a file is still being written, but it is worth a line.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // logger.warn({ path: current, error }, "the replaced selected file does not open yet; the old audio stays");
                // ```
                tracing::warn!(path = %current.display(), error = %error, "the replaced selected file does not open yet; the old audio stays");
                return;
            }
        };
        // What:     `let was_playing: bool = self.playing;` reads the playing flag into a local. The `: bool` is an
        //           explicit type; `bool` is Rust's `boolean`, and a `bool` is copied on assignment (it is a `Copy`
        //           type), so later changes to `self.playing` do not change this local.
        // Why:      Installing the new decoder may stop playback, and the flag must be remembered from before.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const wasPlaying: boolean = this.playing;
        // ```
        let was_playing: bool = self.playing;
        // What:     `let position: f64 = self.audible_secs();` reads the audible playback position in seconds into a
        //           local `f64`. `audible_secs` takes `&self`, so it only reads the controller.
        // Why:      The position the listener hears now is where the new file must continue.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const position: number = this.audibleSecs();
        // ```
        let position: f64 = self.audible_secs();
        // What:     `let new_duration: f64 = self.install_source_silently(source, &current);` hands the new decoder to the
        //           controller and answers the new file's length in seconds. `source` is passed by value, which MOVES
        //           it: after this call the local `source` can no longer be used, because the controller now owns the
        //           decoder. `&current` lends the path read-only.
        // Why:      It swaps the decoder, peak and gain without a click or a toast ("silently") and reports the length
        //           the seek bar needs.
        // Gotcha:   A move is not a copy and not a shared reference; in TypeScript both names would still point at the
        //           decoder, but here the old name is dead.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const newDuration: number = this.installSourceSilently(source, current);
        // ```
        let new_duration: f64 = self.install_source_silently(source, &current);
        // What:     `let target: f64 = resume_position(position, new_duration, was_playing);` calls the pure function
        //           from the top of the file with three values that are all `Copy` types, so nothing is moved or borrowed.
        // Why:      It decides where the swapped file starts.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const target: number = resumePosition(position, newDuration, wasPlaying);
        // ```
        let target: f64 = resume_position(position, new_duration, was_playing);
        // What:     `tracing::info!(path = %current.display(), was_playing, old_position = position, new_duration, target, "...");`
        //           is the info-level log call. A field written as a bare name (`was_playing`, `new_duration`, `target`)
        //           is shorthand for `was_playing = was_playing`, taking the field name from the local's name.
        // Why:      One line records the whole decision of a reload, which is what a reader of the log needs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // logger.info({ path: current, wasPlaying, oldPosition: position, newDuration, target }, "the selected file was replaced; loaded again at the same place");
        // ```
        tracing::info!(
            path = %current.display(),
            was_playing,
            old_position = position,
            new_duration,
            target,
            "the selected file was replaced; loaded again at the same place"
        );
        // What:     `if target > 0.0 { self.seek(target); }` calls the controller's own `seek` method, which takes the
        //           position in seconds. `self.seek(...)` can mutate the controller because this method holds `&mut self`.
        // Why:      A start at zero needs no seek, because a new decoder already starts at the beginning.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (target > 0) this.seek(target);
        // ```
        if target > 0.0 {
            self.seek(target);
        }
        // What:     `self.emit(Update::Duration(new_duration));` sends a message to the window. `Update::Duration(x)` builds
        //           the `Duration` variant of the `Update` enum, a tagged union whose variants may carry data; `::` is
        //           the path separator between the enum's name and its variant.
        // Why:      The seek bar must take the new file's length.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emit({ kind: "duration", secs: newDuration });
        // ```
        self.emit(Update::Duration(new_duration));
        // What:     `self.emit(Update::Position(self.audible_secs()));` builds the `Position` variant around a freshly read
        //           position and sends it. This is the method's last statement and ends in `;`, so the method returns `()`.
        // Why:      The seek bar must show where the new file continues from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.emit({ kind: "position", secs: this.audibleSecs() });
        // ```
        self.emit(Update::Position(self.audible_secs()));
    }
}

/// What: `#[cfg(test)] #[path = "controller_reload_tests.rs"] mod tests;` declares the module `tests`, whose source is the
/// file `controller_reload_tests.rs`. `#[cfg(test)]` compiles it only for `cargo test`, and `#[path = "..."]` names the
/// file because it does not follow the default naming.
/// Why: the unit tests live in their own file to keep this one short, and stay out of the production build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: like a `foo.test.ts` file that the production bundle never imports.
/// ```
#[cfg(test)]
#[path = "controller_reload_tests.rs"]
mod tests;
