//! Unmodified variable fonts share stable identities across shaping contexts and glyph caches.

/// Fontique's immutable font blob is already used by Parley and Slint.
use parley::fontique::Blob;
/// LazyLock constructs each font identity once;
///  Arc retains immutable embedded bytes across threads.
use std::sync::{Arc, LazyLock};

/// JetBrains Mono variable roman,
///  with its original wght axis and OpenType features.
pub const CODE_ROMAN: &[u8] = include_bytes!("../asset/font/JetBrainsMono-Variable.ttf");
/// Genuine JetBrains Mono italic design,
///  not a skew transform of the roman face.
pub const CODE_ITALIC: &[u8] = include_bytes!("../asset/font/JetBrainsMono-VariableItalic.ttf");
/// Inter's variable text/display optical-size and weight design.
pub const UI_ROMAN: &[u8] = include_bytes!("../asset/font/InterVariable.ttf");
/// Genuine Inter italic with its own optical-size and weight axes.
pub const UI_ITALIC: &[u8] = include_bytes!("../asset/font/InterVariable-Italic.ttf");

/// Build the shared blob of the roman code face.
fn code_roman_blob() -> Blob<u8> {
    return Blob::new(Arc::new(CODE_ROMAN));
}

/// Build the shared blob of the italic code face.
fn code_italic_blob() -> Blob<u8> {
    return Blob::new(Arc::new(CODE_ITALIC));
}

/// Build the shared blob of the roman interface face.
fn ui_roman_blob() -> Blob<u8> {
    return Blob::new(Arc::new(UI_ROMAN));
}

/// Shared roman identity lets cache tests distinguish variation coordinates rather than new font IDs.
static CODE_ROMAN_BLOB: LazyLock<Blob<u8>> = LazyLock::new(code_roman_blob);
/// The italic face has a distinct identity while sharing its bytes across all views.
static CODE_ITALIC_BLOB: LazyLock<Blob<u8>> = LazyLock::new(code_italic_blob);

/// The roman interface face has one identity for every shaper,
///  so cached glyph masks are shared between frames.
static UI_ROMAN_BLOB: LazyLock<Blob<u8>> = LazyLock::new(ui_roman_blob);

/// Return shared handles for both real code faces;
///  cloning retains each blob's ID.
pub(crate) fn code_faces() -> [Blob<u8>; 2] {
    return [CODE_ROMAN_BLOB.clone(), CODE_ITALIC_BLOB.clone()];
}

/// Return the shared handle of the roman interface face,
///  which sets the text of virtual rows:
/// inlay hints and diagnostic messages above a code row.
pub(crate) fn row_face() -> Blob<u8> {
    return UI_ROMAN_BLOB.clone();
}
