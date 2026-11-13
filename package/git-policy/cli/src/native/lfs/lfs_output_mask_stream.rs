//! What:
//!  The mask of a stream,
//! kept across the reads a relay makes of a child's output.
//! Why:
//!  A relay reads a stream in chunks,
//! and a chunk ends wherever the kernel had bytes:
//! a URL cut at a chunk's end can end before the `@` that ends its user information,
//! and a fragment read on its own reads as a host.
//! So the bytes after the last byte that ends a URL are held back,
//! everything before them is masked and handed over,
//! and the next chunk is masked with the held bytes in front of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class UrlMaskingStream { feed(bytes: Uint8Array): Uint8Array; finish(): Uint8Array }
//! ```

use super::lfs_output_mask_reading_a_url::{ends_url, next_separator};
use super::{mask_url_secrets, mask_url_secrets_cut};

/// Whether a byte ends a URL in running text,
/// as a reversed iterator hands it over.
fn is_a_url_end(byte: &u8) -> bool {
    return ends_url(*byte);
}

/// What:
///  How long a word with no end is held before it is hidden whole.
/// Why:
///   A stream that never ends a word would otherwise grow this value without bound.
pub const LONGEST_HELD_WORD: usize = 64 * 1024;

/// What:
///  The mask of a stream,
/// kept across the reads a relay makes of a child's output.
/// Why:
///   A held word that passes [`LONGEST_HELD_WORD`] is dealt with at once:
/// one that holds no `://` cannot be a URL and is handed over as it stands,
/// and one that holds a scheme is hidden whole (`mask_url_secrets_cut`),
/// with the rest of that word dropped when the mask hid something in it,
/// because a word whose middle was hidden may not be written unmasked afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UrlMaskingStream { feed(bytes: Uint8Array): Uint8Array; finish(): Uint8Array }
/// ```
#[derive(Debug, Default)]
pub struct UrlMaskingStream {
    /// The bytes of the word in progress,
    /// which may belong to a URL that goes on in the next chunk.
    held: Vec<u8>,
    /// Whether the bytes up to the next byte that ends a URL belong to a word already hidden whole and are dropped.
    swallowing: bool,
}

impl UrlMaskingStream {
    /// A stream that has read nothing.
    pub fn new() -> Self {
        return Self::default();
    }

    /// What:
    ///  Mask what this chunk lets be classified and return what may be written now.
    /// Why:
    ///   The held word is the only bytes this cannot classify yet,
    /// and it is what the next chunk starts with.
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut shown: Vec<u8> = Vec::new();
        let mut from: usize = 0;
        if self.swallowing {
            match bytes.iter().position(is_a_url_end) {
                Some(offset) => {
                    self.swallowing = false;
                    shown.push(bytes[offset]);
                    from = offset + 1;
                }
                None => return shown,
            }
        }
        // The held word comes first, so a URL that began in an earlier chunk is classified with this one.
        let mut text: Vec<u8> = std::mem::take(&mut self.held);
        text.extend_from_slice(&bytes[from..]);
        match text.iter().rposition(is_a_url_end) {
            Some(last) => {
                let (whole, tail): (&[u8], &[u8]) = text.split_at(last + 1);
                // The masked part is a text whose URLs all ended, so it is read as one.
                shown.extend_from_slice(mask_url_secrets(whole).as_slice());
                self.held = tail.to_vec();
            }
            None => self.held = text,
        }
        if self.held.len() > LONGEST_HELD_WORD {
            let long: Vec<u8> = std::mem::take(&mut self.held);
            if next_separator(long.as_slice(), 0).is_none() {
                // A word with no scheme carries no URL, so it is handed over as it stands.
                shown.extend_from_slice(long.as_slice());
            } else {
                let hidden: Vec<u8> = mask_url_secrets_cut(long.as_slice());
                // The rest of the word is dropped only when this mask hid something in it: a word that hid nothing may go
                //  on being read, and a word whose middle was hidden may not be written unmasked afterwards.
                self.swallowing = hidden.as_slice() != long.as_slice();
                shown.extend_from_slice(hidden.as_slice());
            }
        }
        return shown;
    }

    /// What:
    ///  The end of the stream:
    /// the held word is hidden the way the head of a cut text is,
    /// because the stream can end in the middle of a URL too.
    pub fn finish(mut self) -> Vec<u8> {
        if self.held.is_empty() {
            return Vec::new();
        }
        let held: Vec<u8> = std::mem::take(&mut self.held);
        return mask_url_secrets_cut(held.as_slice());
    }
}
