//! What:
//!  The object name hash a repository's configuration selects,
//!  SHA-1 or SHA-256,
//! and the name and length each one gives.
//! Why:
//!  Every reader of the object database needs the length of a name
//! and the text `rev-parse --show-object-format` prints,
//! and they must agree with the format the layout read found.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ObjectFormat = 'sha1' | 'sha256';
//! ```

/// The hash algorithms of object names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectFormat {
    /// 20-byte names,
    /// 40 hexadecimal digits.
    Sha1,
    /// 32-byte names,
    /// 64 hexadecimal digits.
    Sha256,
}

impl ObjectFormat {
    /// The name `rev-parse --show-object-format` prints.
    pub fn name(self) -> &'static str {
        return match self {
            ObjectFormat::Sha1 => "sha1",
            ObjectFormat::Sha256 => "sha256",
        };
    }

    /// The number of hexadecimal digits of one object name.
    pub fn hex_length(self) -> usize {
        return match self {
            ObjectFormat::Sha1 => 40,
            ObjectFormat::Sha256 => 64,
        };
    }
}
