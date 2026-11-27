//! What:
//!  The words a configuration value may be spelled with:
//!  the booleans Git reads as true and as false,
//! the names a per-driver and a per-remote key takes,
//!  the slots of `color.grep.<slot>`,
//! and the color and attribute words.
//! Why:
//!  Git accepts these spellings silently and fails on any other,
//! so the proof of a typed value needs the lists,
//!  and they are data that nothing else in the table decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const TRUE_WORDS = ['true', 'yes', 'on']; // and the other lists beside it
//! ```

/// The words Git reads as true.
pub(crate) const TRUE_WORDS: &[&str] = &["true", "yes", "on"];

/// The words Git reads as false.
pub(crate) const FALSE_WORDS: &[&str] = &["false", "no", "off"];

/// Per-driver keys (`diff.<driver>.<name>`) whose value is a boolean.
pub(crate) const DRIVER_BOOLEAN_NAMES: &[&str] = &["binary", "cachetextconv", "trustexitcode"];

/// Per-driver keys whose value is text.
pub(crate) const DRIVER_TEXT_NAMES: &[&str] = &["command", "textconv", "wordregex", "xfuncname"];

/// Per-remote keys (`remote.<name>.<key>`) whose value is a boolean.
pub(crate) const REMOTE_BOOLEAN_NAMES: &[&str] = &[
    "mirror",
    "promisor",
    "prune",
    "prunetags",
    "skipdefaultupdate",
    "skipfetchall",
];

/// Per-remote keys whose value is text.
pub(crate) const REMOTE_TEXT_NAMES: &[&str] = &[
    "negotiationinclude",
    "negotiationrestrict",
    "partialclonefilter",
    "proxy",
    "proxyauthmethod",
    "pushurl",
    "receivepack",
    "serveroption",
    "tagopt",
    "uploadpack",
    "url",
    "vcs",
];

/// The slots `color.grep.<slot>` knows;
/// any other slot is a failure in Git.
pub(crate) const GREP_COLOR_SLOTS: &[&str] = &[
    "column",
    "context",
    "filename",
    "function",
    "linenumber",
    "match",
    "matchcontext",
    "matchselected",
    "selected",
    "separator",
];

/// The color words Git reads without a `bright` prefix.
pub(crate) const PLAIN_COLOR_WORDS: &[&str] = &["normal", "default"];

/// The color words that also exist with a `bright` prefix.
pub(crate) const BRIGHT_COLOR_WORDS: &[&str] = &[
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];

/// The attribute words Git reads (each also with a `no` or `no-` prefix).
pub(crate) const ATTRIBUTE_WORDS: &[&str] =
    &["bold", "dim", "ul", "blink", "reverse", "italic", "strike"];
