//! The keyboard maps of the classifier and the error for an unknown map name.
//!
//! What:
//!  `KeyboardPlatform` names the two keyboard maps the classifier understands,
//!  with the name each is stored under,
//!  and `KeymapError` is the failure of reading a name that matches neither.
//! Why:
//!  The same key means different shortcuts on each map,
//!  so the map is a closed enum that the session file and the classifier share;
//!  it lives apart from `keymap.rs` so that file keeps to the classifier itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type KeyboardPlatform = "windows-linux" | "mac";
//! ```

/// What:
///      `pub enum KeyboardPlatform` names the two keyboard maps the classifier understands.
/// Why:
///      The same key means different shortcuts on each map,
///  so the map is a closed enum.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeyboardPlatform = "windows-linux" | "mac";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardPlatform {
    /// Windows and Linux map,
    ///  where Control is the primary modifier.
    WindowsLinux,
    /// macOS map,
    ///  where Command is the primary modifier.
    Mac,
}

/// Methods of `KeyboardPlatform`.
impl KeyboardPlatform {
    /// What:
    ///      `pub const ALL` lists both maps in declaration order.
    /// Why:
    ///      `from_name` searches this list,
    ///  so the accepted names live in one place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const ALL: readonly KeyboardPlatform[] = ["windows-linux", "mac"];
    /// ```
    pub const ALL: [KeyboardPlatform; 2] = [KeyboardPlatform::WindowsLinux, KeyboardPlatform::Mac];

    /// What:
    ///      `stored_name` returns the stable text for this map.
    /// Why:
    ///      The prototype and its tests refer to platforms by this exact text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function storedName(platform: KeyboardPlatform): string;
    /// ```
    pub fn stored_name(self) -> &'static str {
        return match self {
            Self::WindowsLinux => "windows-linux",
            Self::Mac => "mac",
        };
    }

    /// What:
    ///      `from_name` returns the map whose stored name equals `text`,
    ///  or an error for any other text.
    /// Why:
    ///      Unknown text must not silently choose a map,
    ///  and the match is exact and case-sensitive.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fromName(text: string): KeyboardPlatform {
    ///   if (text === "windows-linux") return "windows-linux";
    ///   if (text === "mac") return "mac";
    ///   throw new Error("Unknown keyboard-map platform.");
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `KeymapError::UnknownPlatform` when no stored name equals `text`.
    pub fn from_name(text: &str) -> Result<Self, KeymapError> {
        for platform in Self::ALL {
            if platform.stored_name() == text {
                return Ok(platform);
            }
        }
        return Err(KeymapError::UnknownPlatform);
    }
}

/// What:
///      `pub enum KeymapError` is the failure of `KeyboardPlatform::from_name`.
/// Why:
///      A typed error lets callers match on the cause,
///  and its `Display` keeps the prototype's exact message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KeymapError extends Error { /* "Unknown keyboard-map platform." */ }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeymapError {
    /// The text names neither keyboard map.
    UnknownPlatform,
}

/// Display text for `KeymapError`.
impl std::fmt::Display for KeymapError {
    /// What:
    ///      `fmt` writes the message the prototype throws for an unknown platform.
    /// Why:
    ///      Callers and the tests compare this exact text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return "Unknown keyboard-map platform."; }
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::UnknownPlatform => formatter.write_str("Unknown keyboard-map platform."),
        };
    }
}

/// Marks `KeymapError` as a standard error.
impl std::error::Error for KeymapError {}
