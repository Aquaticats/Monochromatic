//! What:
//!  What a snapshot's leave-out made and moved,
//! with identities:
//! the trash directory it created and each directory it
//!  renamed into it,
//! as the text a work item carries and the handler reads back.
//! Why:
//!  A name of forty hexadecimal digits proves nothing about who made a directory.
//! Only what the leave-out itself created or
//!  moved may be deleted,
//! so the leave-out writes down the identity of every object it put in the trash and the disposal
//!  deletes an object only when it is still the one written down.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const manifest = { name, trash: id, entries: [id0, id1] }; // entry i is named `d${i}` in the trash
//! const text = encodeManifest(manifest); const same = decodeManifest(name, text);
//! ```

use crate::fs_beneath::ObjectId;

/// Separates the identities in the encoded manifest;
/// an encoded identity holds only digits,
/// colons and a minus sign.
const SEPARATOR: char = ';';

/// What:
///  The trash directory a leave-out made inside a new worktree and what it moved into it.
/// Why:
///   The disposal needs no scan of the directory it empties:
/// the manifest says which directory is the trash,
/// and which
///  entries of it are the leave-out's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrashManifest = { name: string; trash: ObjectId; entries: ObjectId[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrashManifest {
    /// The name of the trash directory below the trash parent:
    /// forty lowercase hexadecimal digits.
    pub name: String,
    /// The identity of the trash directory,
    /// taken when the leave-out created it.
    pub trash: ObjectId,
    /// The identity of each directory moved into the trash,
    /// in the order moved;
    /// the entry at index `i` is named by
    ///  `entry_name(i)`.
    pub entries: Vec<ObjectId>,
}

/// The name,
/// inside the trash directory,
/// of the entry moved `index`th.
pub fn entry_name(index: usize) -> String {
    return format!("d{index}");
}

/// Whether a name is one the leave-out gives a trash directory:
/// forty lowercase hexadecimal digits.
pub fn is_trash_name(name: &str) -> bool {
    return name.len() == 40 && name.bytes().all(is_lowercase_hexadecimal_digit);
}

/// What:
///  Whether a byte is a decimal digit or a lowercase letter from `a` to `f`.
/// Why:
///   A trash directory is named by forty such digits,
/// so any other byte means the name is not one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /[0-9a-f]/.test(char)
/// ```
fn is_lowercase_hexadecimal_digit(byte: u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
}

/// Methods over a manifest.
impl TrashManifest {
    /// The manifest as the text a work item carries:
    /// the identity of the trash directory,
    /// then one identity per entry.
    pub fn encode(&self) -> String {
        let mut parts: Vec<String> = vec![self.trash.encode()];
        parts.extend(self.entries.iter().map(ObjectId::encode));
        return parts.join(&SEPARATOR.to_string());
    }

    /// The manifest of the trash directory `name` from its encoded text;
    /// the reason is the error.
    pub fn decode(name: &str, text: &str) -> Result<TrashManifest, String> {
        if !is_trash_name(name) {
            return Err(format!("{name} is not the name of a trash directory"));
        }
        let mut parts = text.split(SEPARATOR);
        let Some(trash) = parts.next().and_then(ObjectId::decode) else {
            return Err(format!("the record of the trash {name} does not say which directory it is"));
        };
        let mut entries: Vec<ObjectId> = Vec::new();
        for part in parts {
            let Some(identity) = ObjectId::decode(part) else {
                return Err(format!("the record of the trash {name} holds an identity that cannot be read: {part}"));
            };
            entries.push(identity);
        }
        return Ok(TrashManifest { name: name.to_string(), trash, entries });
    }
}

/// Manifest controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_manifest_tests.rs"]
mod tests;
