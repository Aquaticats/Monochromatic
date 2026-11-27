//! Desktop session persistence.
//!  Current JSON stores one `playback_mode`;
//!  former
//! `shuffle` and `repeat_track` fields exist only in the private read model used
//! during migration.
//! The former `page_control_style` key is ignored on read and no longer written.
//! The record shapes and their defaults live in the sibling file `session_record.rs`,
//! and the atomic replacement of the file in `session_file_replacement.rs`,
//! so that each file stays within the line budget;
//! `Session` is re-exported here,
//! so every caller still names `session::Session`.

/// Borrowed and owned filesystem paths: the session file and the disposable test home.
use std::path::{
    Path,
    PathBuf,
};

/// Current four-state playback model.
use crate::command::PlaybackMode;

/// Shared application identity used to resolve the config directory.
use crate::identity;

/// The four text sizes the Settings page changes,
/// and the rule that reads one stored field.
use crate::text_size::{
    TextElement,
    TextSizes,
    size_or_default,
};

/// What:
///      `#[path = "session_file_replacement.rs"] mod file_replacement;` declares a child module
///           whose code lives in the sibling file `session_file_replacement.rs`.
/// Why:
///       The atomic replacement of the session file fills a file of its own,
///  and this file keeps the one writer `save_at` that publishes through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fileReplacement from "./session_file_replacement";
/// ```
#[path = "session_file_replacement.rs"]
mod file_replacement;

/// What:
///      `#[path = "session_record.rs"] mod record;` declares a child module
///           whose code lives in the sibling file `session_record.rs`.
/// Why:
///       The record shapes and their defaults fill a file of their own,
///  and this file reads and writes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as record from "./session_record";
/// ```
#[path = "session_record.rs"]
mod record;

/// What:
///      `pub use record::Session;` re-exports the record at the `session` path.
/// Why:
///       Callers say `session::Session`, unaware of the submodule split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { Session } from "./session_record";
/// ```
pub use record::Session;

/// What:
///      `use file_replacement::replace_file;` brings the atomic replacement into this file.
/// Why:
///       Every save publishes through that one writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { replaceFile } from "./session_file_replacement";
/// ```
use file_replacement::replace_file;

/// What:
///      `use record::StoredSession;` brings the permissive read shape into this file.
/// Why:
///       Reading a stored file and migrating it both start from that shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { StoredSession } from "./session_record";
/// ```
use record::StoredSession;

/// Loads,
///  migrates,
///  and saves sessions.
impl Session {
    /// Converts permissive input into current state and reports whether to rewrite.
    fn from_stored(
        stored: StoredSession,
        has_current_mode: bool,
    ) -> (Session, bool) {
        let should_rewrite =
            !has_current_mode || stored.shuffle.is_some() || stored.repeat_track.is_some();
        let current_mode = stored.playback_mode.as_deref();
        let playback_mode = if current_mode == Some("repeat") {
            PlaybackMode::Repeat
        } else if current_mode == Some("shuffle_page") {
            PlaybackMode::ShufflePage
        } else if current_mode == Some("shuffle_all") {
            PlaybackMode::ShuffleAll
        } else if has_current_mode {
            PlaybackMode::InOrder
        } else if stored.repeat_track == Some(true) {
            PlaybackMode::Repeat
        } else if stored.shuffle.as_deref() == Some("WithinPage") {
            PlaybackMode::ShufflePage
        } else if stored.shuffle.as_deref() == Some("All") {
            PlaybackMode::ShuffleAll
        } else {
            PlaybackMode::InOrder
        };
        return (
            Session {
                source_root: stored.source_root,
                selected: stored.selected,
                position_secs: stored.position_secs,
                volume: stored.volume,
                playback_mode,
                track_rows_template: stored.track_rows_template,
                playing_template: stored.playing_template,
                auto_play_on_select: stored.auto_play_on_select,
                text_sizes: Session::text_sizes_from(&stored.text_sizes),
            },
            should_rewrite,
        );
    }

    /// What:
    /// `text_sizes_from` reads the four stored size fields,
    /// each falling back to its own default.
    /// Why:
    /// a corrupt or out-of-range field costs only that element its size (and is logged),
    /// never the other
    /// sizes and never the rest of the session.
    fn text_sizes_from(stored: &[Option<serde_json::Value>; 4]) -> TextSizes {
        let mut sizes = TextSizes::default();
        for (element, field) in TextElement::ALL.iter().zip(stored) {
            sizes = sizes.with(*element, size_or_default(*element, field.as_ref()));
        }
        return sizes;
    }

    /// Parses permissive input while preserving whether the current key was present.
    fn parse_stored(text: &str) -> Result<(StoredSession, bool), serde_json::Error> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        let has_current_mode =
            matches!(value.as_object(), Some(object) if object.contains_key("playback_mode"));
        let mut text_sizes: [Option<serde_json::Value>; 4] = Default::default();
        for (position, element) in TextElement::ALL.iter().enumerate() {
            text_sizes[position] = match value.as_object() {
                Some(object) => object.get(element.session_key()).cloned(),
                None => None,
            };
        }
        let mut stored: StoredSession = serde_json::from_value(value)?;
        stored.text_sizes = text_sizes;
        return Ok((stored, has_current_mode));
    }

    /// Parses current or former JSON without exposing migration fields.
    #[cfg(test)]
    fn from_json(text: &str) -> Result<Session, serde_json::Error> {
        let (stored, has_current_mode) = Session::parse_stored(text)?;
        return Ok(Session::from_stored(stored, has_current_mode).0);
    }

    /// Reads the session file,
    ///  returning defaults after any path,
    ///  I/O,
    ///  or parse failure.
    pub fn load() -> Session {
        let Some(path) = session_path() else {
            return Session::default();
        };
        return Session::load_at(&path);
    }

    /// What:
    /// `load_at` reads the session file at `path`,
    /// with the same fallbacks as `load`.
    /// Why:
    /// the persistence tests read a file in a temporary directory,
    /// never the user's own session file.
    ///
    /// A missing file is a first run and stays quiet.
    /// A file that exists but cannot be read,
    /// or whose text is not a
    /// session,
    /// resets the saved state to defaults,
    /// so each is logged with the error and the path.
    fn load_at(path: &Path) -> Session {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Session::default();
            }
            Err(error) => {
                tracing::warn!(?error, path = %path.display(), "failed to read the playback session; using defaults");
                return Session::default();
            }
        };
        let (stored, has_current_mode) = match Session::parse_stored(&text) {
            Ok(parsed) => parsed,
            Err(error) => {
                tracing::warn!(?error, path = %path.display(), "failed to parse the playback session; using defaults");
                return Session::default();
            }
        };
        let (session, should_rewrite) = Session::from_stored(stored, has_current_mode);
        if should_rewrite && let Err(error) = session.save_at(path) {
            tracing::warn!(?error, "failed to rewrite migrated playback session");
        }
        return session;
    }

    /// Writes only the current session shape to the app config directory.
    /// What:
    /// `load_templates` reads the two saved template texts,
    /// None where the user kept the default.
    /// Why:
    /// the editor and the track list start from the saved texts on launch.
    pub fn load_templates() -> (Option<String>, Option<String>) {
        let session = Session::load();
        return (session.track_rows_template, session.playing_template);
    }

    /// What:
    /// `save_templates` writes both template texts and keeps the playback fields as they are.
    /// Why:
    /// the template editor saves here,
    /// and the playback record is rewritten by the controller separately.
    pub fn save_templates(
        track_rows: Option<&str>,
        playing: Option<&str>,
    ) -> std::io::Result<()> {
        let mut session = Session::load();
        session.track_rows_template = track_rows.map(str::to_string);
        session.playing_template = playing.map(str::to_string);
        return session.save();
    }

    /// What:
    /// `load_auto_play_on_select` reads the Auto-play on select setting;
    /// on when the file has none.
    /// Why:
    /// the window starts from the saved choice (D129),
    /// and a session file written before the setting existed must
    /// keep loading,
    /// with the setting off.
    #[must_use]
    pub fn load_auto_play_on_select() -> bool {
        return Session::load().auto_play_on_select;
    }

    /// What:
    /// `save_auto_play_on_select` writes the setting and keeps every other field as it is.
    /// Why:
    /// the Settings switch saves here,
    /// and the playback record is rewritten by the controller separately.
    pub fn save_auto_play_on_select(enabled: bool) -> std::io::Result<()> {
        let Some(path) = session_path() else {
            return Ok(());
        };
        return Session::save_auto_play_on_select_at(&path, enabled);
    }

    /// What:
    /// `save_auto_play_on_select_at` is `save_auto_play_on_select` for the session file at `path`.
    /// Why:
    /// the persistence tests write a file in a temporary directory,
    /// never the user's own session file.
    fn save_auto_play_on_select_at(
        path: &Path,
        enabled: bool,
    ) -> std::io::Result<()> {
        let mut session = Session::load_at(path);
        session.auto_play_on_select = enabled;
        return session.save_at(path);
    }

    /// What:
    /// `load_text_sizes` reads the Text size setting;
    /// every element at its default when the file has none.
    /// Why:
    /// the window starts from the saved sizes,
    /// and a session file written before the setting existed must keep
    /// loading with the sizes the app always drew.
    #[must_use]
    pub fn load_text_sizes() -> TextSizes {
        return Session::load().text_sizes;
    }

    /// What:
    /// `save_text_sizes` writes the Text size setting and keeps every other field as it is.
    /// Why:
    /// the Settings steppers save here,
    /// and the playback record is rewritten by the controller separately.
    pub fn save_text_sizes(sizes: TextSizes) -> std::io::Result<()> {
        let Some(path) = session_path() else {
            return Ok(());
        };
        return Session::save_text_sizes_at(&path, sizes);
    }

    /// What:
    /// `save_text_sizes_at` is `save_text_sizes` for the session file at `path`.
    /// Why:
    /// the persistence tests write a file in a temporary directory,
    /// never the user's own session file.
    fn save_text_sizes_at(
        path: &Path,
        sizes: TextSizes,
    ) -> std::io::Result<()> {
        let mut session = Session::load_at(path);
        session.text_sizes = sizes;
        return session.save_at(path);
    }

    /// What:
    /// `save` writes the playback record as JSON to the session file.
    /// Why:
    /// the controller and the template editor both persist through this one file.
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = session_path() else {
            return Ok(());
        };
        return self.save_at(&path);
    }

    /// What:
    /// `save_at` writes the playback record as JSON to the file at `path`,
    /// creating its directory.
    /// Why:
    /// `save` and the persistence tests share one writer,
    /// and it replaces the file whole (see `replace_file`),
    /// so
    /// an interrupted save leaves the previous complete session.
    fn save_at(
        &self,
        path: &Path,
    ) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        return replace_file(path, json.as_bytes());
    }
}

/// Resolves the app-private session JSON path.
fn session_path() -> Option<PathBuf> {
    let directory = identity::config_dir()?;
    return Some(directory.join("session.json"));
}

/// What:
/// `use_disposable_config_home` points this process's config home at a new temporary directory and gives the
/// session file path inside it,
/// after checking that the path really is inside.
/// Why:
/// the entry points that read and write the user's own session file (`load`,
/// `save_auto_play_on_select`,
/// the
/// shutdown snapshot) can be tested without touching it;
/// the check runs before any write,
/// so a platform whose config
/// home ignores `XDG_CONFIG_HOME` fails the test instead of writing the user's file.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn use_disposable_config_home(test: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let home = crate::test_scratch::scratch_root().join(format!("mp_config_{test}_{nanos}"));
    std::fs::create_dir_all(&home).expect("the disposable config home is created");
    // SAFETY: nextest runs each test in its own process, and the test sets this before it starts any thread of its own,
    // so no other thread reads the environment while it changes.
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", &home);
    }
    let path = session_path().expect("the config home resolves");
    assert!(
        path.starts_with(&home),
        "the session file {path:?} is not inside the disposable home {home:?}"
    );
    return path;
}

/// Session persistence tests.
#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
