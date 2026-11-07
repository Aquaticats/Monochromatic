//! The lookups and constructors that `main` runs before the window opens:
//! the music folder to start in,
//! the color scheme override,
//! and the two system notifiers.
//!
//! What:
//!  Each function answers one startup question from the environment:
//!  where the user's music folder is,
//!  which color scheme the environment forces,
//!  and the app identity and notifiers that the engine reports through.
//! Why:
//!  `main` stays the one place that wires the pieces together,
//!  and the environment lookups sit in a file of their own where each can be read without the window code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export { musicDir, colorSchemeFromEnv, startAnalysisNotifier, startUnplayableNotifier };
//! ```

/// Imports the path type that the music folder lookups return.
use std::path::PathBuf;

/// Imports the path type that compares the user-dirs answer with the home folder,
/// which only the Unix lookup does.
#[cfg(unix)]
use std::path::Path;

/// Imports the color scheme environment variable,
/// its follow-system value and the override parser.
use music_player::color_scheme::{COLOR_SCHEME_ENV, FOLLOW_SYSTEM, scheme_override};

/// Imports the analysis indicator's handle,
/// the identity that names the app to the desktop and the configuration that starts it.
use music_player::analysis_notification::{AnalysisNotifier, Identity, NotifierConfig};

/// Imports the pause gate that the indicator's Pause button drives.
use music_player::sweep_gate::SweepGate;

/// Imports the unplayable-file notifier and the configuration that starts it.
use music_player::unplayable_notification::{UnplayableConfig, UnplayableNotifier};

/// What:
///      `fn xdg_user_dir_music() -> Option<PathBuf>`.
///  Last-resort lookup:
///  shell
///           out to the `xdg-user-dir MUSIC` command and use its printed path.
///  The
///           return is `Option<PathBuf>`:
///  `Some(path)` on success,
///  `None` otherwise
///           (Rust has no `null`;
///  absence is modeled by the `Option` enum).
/// Why:
///       Some setups (and some `directories` parsing gaps) leave the music dir
///           discoverable only through the official `xdg-user-dir` tool;
///  this is the
///           fallback when the env var and the user-dirs file both come up empty.
/// What:
///      `#[cfg(unix)]` compiles this Unix version of the helper only on Unix
///           targets (Linux/macOS/BSD);
///  the `#[cfg(not(unix))]` stub just below
///           replaces it on Windows.
///  `unix` is a built-in cfg covering the whole
///           Unix family;
///  siblings:
///  `windows`,
///  `target_os = "linux"`.
/// Why:
///       `xdg-user-dir` is a freedesktop CLI tool that exists only on Unix
///           desktops;
///  on Windows the spawn would always fail,
///  so gate it out and
///           let the stub return `None` instead of wasting a process spawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function xdgUserDirMusic(): string | null { ... } // Unix-only implementation
/// ```
#[cfg(unix)]
pub(crate) fn xdg_user_dir_music() -> Option<PathBuf> {
    // What:     `let output = std::process::Command::new("xdg-user-dir").arg("MUSIC").output().ok()?;`.
    //           `Command::new(name)` starts a process builder; `.arg("MUSIC")` adds an
    //           argument; `.output()` runs it to completion and returns
    //           `io::Result<Output>` (stdout/stderr/status). `.ok()` converts that
    //           `Result` into an `Option` (DROPPING the error); the `?` then unwraps
    //           the `Option`, RETURNING `None` from this whole function if the command
    //           could not run (e.g. the tool is not installed, as in the container).
    // Why:      Ask the OS where the music directory is; treat "could not run" as
    //           "no answer" rather than an error to propagate.
    // Gotcha:   `?` here is EARLY RETURN, not optional chaining: on `None` it exits
    //           the function returning `None`. Combined with `.ok()` it also silently
    //           discards the underlying I/O error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let output;
    // try { output = spawnSync("xdg-user-dir", ["MUSIC"]); } catch { return null; }
    // ```
    let output = std::process::Command::new("xdg-user-dir")
        .arg("MUSIC")
        .output()
        .ok()?;
    // What:     `if !output.status.success() { return None; }`. `status.success()` is
    //           `true` only on exit code 0; `!` negates it; an early `return None`
    //           constructs the empty `Option`.
    // Why:      A failed command yields no usable path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (output.status !== 0) return null;
    // ```
    if !output.status.success() {
        return None;
    }
    // What:     `let text = String::from_utf8(output.stdout).ok()?;`. `output.stdout`
    //           is the raw bytes (`Vec<u8>`); `String::from_utf8` validates them as
    //           UTF-8 returning `Result<String, _>`; `.ok()?` yields the `String` or
    //           early-returns `None` on invalid UTF-8.
    // Why:      We need the printed path as text.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = output.stdout.toString("utf8");
    // ```
    let text = String::from_utf8(output.stdout).ok()?;
    // What:     `let trimmed = text.trim();`. `.trim()` returns a BORROWED `&str`
    //           view of `text` with leading/trailing whitespace (including the
    //           trailing newline) removed; no new allocation.
    // Why:      Command output ends in a newline we must strip.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trimmed = text.trim();
    // ```
    let trimmed = text.trim();
    // What:     `if trimmed.is_empty() { return None; }`. `.is_empty()` is `true` for
    //           a zero-length string; early-return the empty `Option`.
    // Why:      No path was printed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!trimmed) return null;
    // ```
    if trimmed.is_empty() {
        return None;
    }
    // What:     `if let Some(home) = std::env::var_os("HOME") && Path::new(&home) == Path::new(trimmed) { return None; }`.
    //           A LET-CHAIN `if`: it runs the body only if BOTH `var_os("HOME")` is
    //           `Some(home)` (the env var is set) AND the two paths are equal.
    //           `Path::new(x)` wraps a borrowed `&OsStr`/`&str` as a `&Path` with no
    //           allocation (unlike `PathBuf::from`, which would allocate); `&Path ==
    //           &Path` compares by path components. `&home` borrows the `OsString`.
    // Why:      `xdg-user-dir MUSIC` prints `$HOME` when no music dir is configured;
    //           reject that case to avoid auto-loading the entire home directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const home = process.env.HOME;
    // if (home !== undefined && home === trimmed) return null;
    // ```
    if let Some(home) = std::env::var_os("HOME")
        && Path::new(&home) == Path::new(trimmed) {
            return None;
        }
    // What:     `Some(PathBuf::from(trimmed))`. `PathBuf::from(trimmed)` allocates an
    //           owned path from the borrowed `&str`; `Some(...)` wraps it as the
    //           present case of `Option`. Tail expression -> return value.
    // Why:      Hand back the discovered music directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return trimmed;
    // ```
    return Some(PathBuf::from(trimmed))
}

/// What:
///      `#[cfg(not(unix))] fn xdg_user_dir_music() -> Option<PathBuf>`.
///  The
///           non-Unix stub (Windows):
///  same signature as the Unix version above,
///           compiled only when NOT a Unix target.
///  `not(unix)` inverts the `unix`
///           cfg predicate.
/// Why:
///       Windows has no `xdg-user-dir` tool,
///  and `music_dir()` already resolves
///           the Windows Music known-folder via the `directories` crate one step
///           earlier,
///  so this fallback has nothing to do;
///  keep the call site
///           platform-agnostic by returning `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function xdgUserDirMusic(): string | null { return null; } // non-Unix stub
/// ```
#[cfg(not(unix))]
pub(crate) fn xdg_user_dir_music() -> Option<PathBuf> {
    // What:     `None`. The empty case of `Option<PathBuf>` (no path); sibling
    //           `Some(p)` would carry a path. Bare tail expression -> return value.
    // Why:      Signal "this source found no music dir" on Windows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return null;
    // ```
    None
}

/// What:
///      `fn music_dir() -> Option<PathBuf>`.
///  Find the user's music directory:
///           the `XDG_MUSIC_DIR` environment variable first,
///  then the XDG user-dirs
///           file via the `directories` crate,
///  then the `xdg-user-dir MUSIC`
///           command,
///  then `Music` inside the home folder (D95:
/// a library is always open).
///  Returns `None` unless one yields an existing directory.
/// Why:
///       The containerized `run` task bind-mounts the host music folder and
///           exports `XDG_MUSIC_DIR` as its in-container path;
///  a native run has no
///           such env,
///  so we fall back to the user-dirs file and finally the
///           `xdg-user-dir` tool.
///  The `directories` crate reads only the file,
///  never
///           the env var,
///  so the env lookup must be explicit here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function musicDir(): string | null { ... }
/// ```
pub(crate) fn music_dir() -> Option<PathBuf> {
    // What:     the places to look, in order: `XDG_MUSIC_DIR`, the user-dirs file, the `xdg-user-dir` tool, then `Music`
    //           inside the home folder. Each later place is computed only if the earlier ones gave no folder.
    // Why:      D95: a library is always open. A home folder with a `Music` folder but no user-dirs entry used to show
    //           the "No folder is open" screen (EM7). The first place that is an existing folder wins, so a variable
    //           pointing at a missing folder falls through instead of ending the search.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [process.env.XDG_MUSIC_DIR, userDirs()?.audioDir, xdgUserDirMusic(), join(homedir(), "Music")].find(isDir) ?? null;
    // ```
    let from_environment = std::iter::once(std::env::var_os("XDG_MUSIC_DIR").map(PathBuf::from));
    let from_user_dirs = std::iter::once_with(|| {
        return directories::UserDirs::new().and_then(|dirs| return dirs.audio_dir().map(|p| return p.to_path_buf()));
    });
    let from_tool = std::iter::once_with(xdg_user_dir_music);
    let from_home = std::iter::once_with(|| {
        return directories::BaseDirs::new().map(|dirs| return music_player::default_library::home_music_folder(dirs.home_dir()));
    });
    return music_player::default_library::first_existing_dir(
        from_environment.chain(from_user_dirs).chain(from_tool).chain(from_home),
    );
}

/// What:
///      `fn color_scheme_from_env() -> i32`.
///  The `Theme.scheme-override` value that `MUSIC_PLAYER_COLOR_SCHEME` selects.
/// Why:
///       A mistyped value is logged and the window follows the system,
/// so the app still opens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function colorSchemeFromEnv(): number {
///   try { return schemeOverride(process.env.MUSIC_PLAYER_COLOR_SCHEME); } catch (error) { warn(error); return FOLLOW_SYSTEM; }
/// }
/// ```
pub(crate) fn color_scheme_from_env() -> i32 {
    // What:     `scheme_override(value.as_deref())`. Map the variable's text to the override.
    // Why:      An absent variable is `None`, which follows the system.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = process.env.MUSIC_PLAYER_COLOR_SCHEME;
    // ```
    let value = std::env::var(COLOR_SCHEME_ENV).ok();
    return match scheme_override(value.as_deref()) {
        Ok(scheme) => scheme,
        Err(error) => {
            tracing::warn!(error = %error, "ignoring the color scheme override");
            FOLLOW_SYSTEM
        }
    };
}

/// What:
/// `start_analysis_notifier` starts the system indicator for the true-peak analysis (D107).
/// Why:
/// its Pause and Resume go to the sweep gate,
/// which parks the sweep workers before their next claim;
/// the callback
/// only flips the gate,
/// so it returns at once.
/// The name and icon are those of the shipped desktop file,
/// so a run
/// without the installed file still gets a named header.
pub(crate) fn start_analysis_notifier() -> AnalysisNotifier {
    return AnalysisNotifier::spawn(NotifierConfig::session(
        app_identity(),
        true,
        Box::new(|paused| SweepGate::shared().set_paused(paused)),
    ))
}

/// What:
/// `app_identity` names the app to the desktop for every system notification it posts.
/// Why:
/// the analysis indicator (D107) and the "cannot be played" notification (D122) show the same name,
/// icon and
/// desktop entry,
/// so they are built from one value.
pub(crate) fn app_identity() -> Identity {
    return Identity {
        desktop_entry: music_player::identity::APP_ID.to_string(),
        app_name: "Music Player".to_string(),
        app_icon: "multimedia-player".to_string(),
    };
}

/// What:
/// `start_unplayable_notifier` starts the system notification for files that cannot be played (D122).
/// Why:
/// the engine thread hands it the paths;
/// it posts them on its own thread,
/// so neither the engine nor the window
/// waits on the bus.
/// The window itself shows only the row mark.
pub(crate) fn start_unplayable_notifier() -> UnplayableNotifier {
    return UnplayableNotifier::spawn(UnplayableConfig::session(app_identity()));
}
