//! Library track value,
//!  the input every folder model groups.
//!
//! Ported from `core/Track.kt` in the Android app (`dev.monochromatic.musicplayer.Track`).
//! Only the two fields the pure folder models read are kept here.

/// What:
///      `pub struct Track { pub uri: String, pub display_path: String }`.
///  `uri` is the opaque playable address and `display_path` is the
///  folder-relative path such as `Artist/Album/01.flac`.
/// Why:
///       Folder grouping reads only `display_path`;
///  `uri` travels along so
///  playback can still address the track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Track = { uri: string; displayPath: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    /// Opaque playable address of the track.
    pub uri: String,
    /// Folder-relative display path,
    ///  with `/` between segments.
    pub display_path: String,
}
