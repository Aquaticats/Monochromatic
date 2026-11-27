//! What:
//!  The kinds of value a configuration key takes,
//! and the lookup from a key to its kind:
//!  the plain key tables in `repo_read_config_key_kinds_table.rs`,
//! the accepted spellings in `repo_read_config_value_words.rs`,
//!  and the shapes a section has of its own.
//! Why:
//!  `repo_read_config_typed.rs` checks whether a value is one Git accepts silently;
//! the table it checks against is data and
//!  lookup that do not use the checking code,
//! so they sit in a file of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Kind = { kind: 'boolean' } | { kind: 'integer'; low: number; high: number } | ...;
//! function kindOf(key: string): Kind | undefined;
//! ```

use crate::repo_read_config_key_kinds_table::{
    BOOLEAN_KEYS,
    PATHNAME_KEYS,
    TEXT_KEYS,
};
pub(crate) use crate::repo_read_config_value_words::{
    ATTRIBUTE_WORDS,
    BRIGHT_COLOR_WORDS,
    FALSE_WORDS,
    PLAIN_COLOR_WORDS,
    TRUE_WORDS,
};
use crate::repo_read_config_value_words::{
    DRIVER_BOOLEAN_NAMES,
    DRIVER_TEXT_NAMES,
    GREP_COLOR_SLOTS,
    REMOTE_BOOLEAN_NAMES,
    REMOTE_TEXT_NAMES,
};

/// How Git reads the value of one key.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    /// `true yes on false no off`,
    /// a decimal number,
    /// or an empty value;
    /// a bare name is true.
    Boolean,
    /// A boolean,
    /// or one of the listed words.
    BooleanOr(&'static [&'static str]),
    /// A boolean or a decimal number;
    /// a bare name is true.
    IntegerOrBoolean,
    /// A decimal number inside the range,
    /// with an optional `k`,
    /// `m` or `g` suffix;
    /// never bare,
    /// never empty.
    /// A kind whose lower bound is not negative takes no minus sign,
    /// whether Git reads the key as a signed or an unsigned number,
    /// so a spelling that carries one is left to Git.
    Integer { low: i64, high: i64 },
    /// Any text;
    /// never bare.
    Text,
    /// Any text that is not a `~user` path;
    /// never bare.
    Pathname,
    /// Exactly one of the listed words,
    /// or an empty value when `empty` is set;
    /// never bare.
    OneOf {
        words: &'static [&'static str],
        empty: bool,
    },
    /// A comma-separated list of the listed words,
    /// each with an optional `-` prefix when `negatable` is set.
    Words {
        words: &'static [&'static str],
        empty: bool,
        negatable: bool,
    },
    /// A color specification of at most two colors and any attributes.
    Color,
    /// `core.abbrev`:
    /// `auto`,
    /// a false word,
    /// an empty value,
    /// or a number from the shortest abbreviation up.
    /// A spelling that stands for zero is a number out of range to Git, not a false word,
    /// so it is left to Git.
    Abbreviation,
    /// `core.commentChar` and `core.commentString`:
    /// not empty,
    /// no line break,
    /// never bare.
    CommentText,
    /// A refspec `[+]refs/<name>:refs/<name>` whose names are plain and whose wildcards match;
    /// every other form is
    ///  left to Git,
    /// which checks a refspec against its full rules.
    Refspec,
    /// `core.fsmonitor`:
    /// a false spelling,
    /// or one of the words `true`,
    /// `yes` and `on`,
    /// which turn the built-in monitor daemon off or on.
    ///  Git reads the bare name and the numbers other than zero as true,
    /// and any other text as the path of a hook program that it runs for commands that read the index.
    /// The subset is deliberately narrow:
    /// leaving those spellings to Git is conservative,
    /// and for a hook path it matters,
    /// because the hook can change what the commands print.
    OffOrBuiltinDaemon,
    /// `trace2.normalTarget`,
    /// `trace2.eventTarget` and `trace2.perfTarget`:
    /// only the spellings that turn the output off
    ///  (empty,
    /// `0` and `false`).
    /// Any other text is an active target (a file,
    /// a directory,
    /// a descriptor,
    /// `true` for standard
    ///  error),
    /// and Git then writes trace output the in-process readers do not (`trace2/tr2_dst.c`,
    /// `tr2_dst_get_trace_fd`).
    TraceTarget,
    /// A key Git warns about,
    /// or fails on,
    /// in spellings the proof does not model:
    /// its presence alone is declined.
    Refused,
}

/// The index of Git's shortest accepted `core.abbrev`.
pub(crate) const MINIMUM_ABBREVIATION: i64 = 4;

/// The longest `core.abbrev` the proof accepts (the longest object name in hexadecimal digits).
pub(crate) const MAXIMUM_ABBREVIATION: i64 = 64;

/// The compression levels of zlib,
/// with the default marker `-1`.
pub(crate) const COMPRESSION: Kind = Kind::Integer {
    low: -1,
    high: 9,
};

/// A number Git stores in an `int`,
/// not below zero.
pub(crate) const NON_NEGATIVE: Kind = Kind::Integer {
    low: 0,
    high: i32::MAX as i64,
};

/// A number Git stores in an `unsigned long`;
/// the proof keeps to the range both widths hold.
pub(crate) const SIZE: Kind = Kind::Integer {
    low: 0,
    high: i32::MAX as i64,
};

/// Split a canonical key into section,
/// subsection and name;
/// the section and name are lower case.
pub(crate) fn split_key(key: &str) -> Option<(&str, Option<&str>, &str)> {
    let (section, rest) = key.split_once('.')?;
    return Some(match rest.rsplit_once('.') {
        Some((subsection, name)) => (section, Some(subsection), name),
        None => (section, None, rest),
    });
}

/// How the key's value is read,
/// or `None` when Git reads no type for the key at startup.
pub(crate) fn kind_of(key: &str) -> Option<Kind> {
    let (section, subsection, name) = split_key(key)?;
    if section == "advice" && subsection.is_none() {
        return Some(Kind::Boolean);
    }
    if subsection.is_none() {
        if BOOLEAN_KEYS.contains(&key) {
            return Some(Kind::Boolean);
        }
        if TEXT_KEYS.contains(&key) {
            return Some(Kind::Text);
        }
        if PATHNAME_KEYS.contains(&key) {
            return Some(Kind::Pathname);
        }
        return plain_kind(key);
    }
    return match (section, name) {
        ("branch", "merge" | "pushremote" | "remote") => Some(Kind::Text),
        ("diff", _) if DRIVER_BOOLEAN_NAMES.contains(&name) => Some(Kind::Boolean),
        ("diff", _) if DRIVER_TEXT_NAMES.contains(&name) => Some(Kind::Text),
        ("remote", _) if REMOTE_BOOLEAN_NAMES.contains(&name) => Some(Kind::Boolean),
        ("remote", _) if REMOTE_TEXT_NAMES.contains(&name) => Some(Kind::Text),
        ("remote", "followremotehead") => Some(Kind::OneOf {
            words: &["create", "warn", "always", "never"],
            empty: false,
        }),
        ("remote", "fetch" | "push") => Some(Kind::Refspec),
        ("url", "insteadof" | "pushinsteadof") => Some(Kind::Text),
        ("color", _) => color_kind(subsection, name),
        _ => None,
    };
}

/// The color keys the readers' commands load:
/// `color.diff.<slot>`,
/// `color.decorate.<slot>`,
/// `color.advice.hint`,
/// `color.grep.<slot>`.
pub(crate) fn color_kind(
    subsection: Option<&str>,
    name: &str,
) -> Option<Kind> {
    return match subsection {
        Some("diff" | "decorate" | "advice") => Some(Kind::Color),
        Some("grep") if GREP_COLOR_SLOTS.contains(&name) => Some(Kind::Color),
        Some("grep") => Some(Kind::Refused),
        _ => None,
    };
}

/// The keys of the form `section.name` whose type is not plain.
pub(crate) fn plain_kind(key: &str) -> Option<Kind> {
    return Some(match key {
        "branch.autosetupmerge" => Kind::BooleanOr(&["always", "inherit", "simple"]),
        "branch.autosetuprebase" => Kind::OneOf {
            words: &["never", "local", "remote", "always"],
            empty: false,
        },
        "color.ui" | "color.diff" | "color.grep" => Kind::BooleanOr(&["always", "never", "auto"]),
        "commitgraph.changedpathsversion" | "commitgraph.generationversion" => NON_NEGATIVE,
        "core.abbrev" => Kind::Abbreviation,
        "core.autocrlf" => Kind::BooleanOr(&["input"]),
        "core.bigfilethreshold"
        | "core.deltabasecachelimit"
        | "core.packedgitlimit"
        | "core.packedgitwindowsize" => SIZE,
        "core.checkstat" => Kind::OneOf {
            words: &["default", "minimal"],
            empty: false,
        },
        "core.commentchar" | "core.commentstring" => Kind::CommentText,
        "core.compression" | "core.loosecompression" | "pack.compression" => COMPRESSION,
        "core.createobject" => Kind::OneOf {
            words: &["rename", "link"],
            empty: false,
        },
        "core.fsmonitor" => Kind::OffOrBuiltinDaemon,
        "trace2.normaltarget" | "trace2.eventtarget" | "trace2.perftarget" => Kind::TraceTarget,
        "core.fsync" => Kind::Words {
            words: &[
                "loose-object",
                "pack",
                "pack-metadata",
                "commit-graph",
                "index",
                "objects",
                "reference",
                "derived-metadata",
                "committed",
                "added",
                "all",
                "none",
            ],
            empty: false,
            negatable: true,
        },
        "core.fsyncmethod" => Kind::OneOf {
            words: &["fsync", "writeout-only", "batch"],
            empty: false,
        },
        "core.fsyncobjectfiles" | "diff.dirstat" => Kind::Refused,
        "core.maxtreedepth"
        | "diff.context"
        | "diff.interhunkcontext"
        | "diff.renamelimit"
        | "diff.statgraphwidth"
        | "diff.statnamewidth"
        | "format.filenamemaxlength" => NON_NEGATIVE,
        "core.safecrlf" => Kind::BooleanOr(&["warn"]),
        "diff.algorithm" => Kind::OneOf {
            words: &["default", "myers", "minimal", "patience", "histogram"],
            empty: false,
        },
        "diff.colormoved" => {
            Kind::BooleanOr(&["default", "blocks", "zebra", "dimmed-zebra", "plain"])
        }
        "diff.colormovedws" => Kind::Words {
            words: &[
                "no",
                "ignore-space-at-eol",
                "ignore-space-change",
                "ignore-all-space",
                "allow-indentation-change",
            ],
            empty: false,
            negatable: false,
        },
        "diff.ignoresubmodules" => Kind::OneOf {
            words: &["none", "untracked", "dirty", "all"],
            empty: false,
        },
        "diff.renames" => Kind::BooleanOr(&["copy", "copies"]),
        "diff.submodule" => Kind::OneOf {
            words: &["log", "short", "diff"],
            empty: false,
        },
        "diff.wserrorhighlight" => Kind::Words {
            words: &["old", "new", "context", "all", "default", "none"],
            empty: true,
            negatable: false,
        },
        "fetch.negotiationalgorithm" => Kind::OneOf {
            words: &["skipping", "noop", "consecutive", "default"],
            empty: false,
        },
        "format.pretty" => Kind::OneOf {
            words: &[
                "oneline",
                "short",
                "medium",
                "full",
                "fuller",
                "reference",
                "email",
                "raw",
                "mboxrd",
            ],
            empty: true,
        },
        "grep.patterntype" => Kind::OneOf {
            words: &["default", "basic", "extended", "fixed", "perl"],
            empty: false,
        },
        "index.maxpathlen" => NON_NEGATIVE,
        "index.threads" => Kind::IntegerOrBoolean,
        "index.version" => Kind::Integer {
            low: 2,
            high: 4,
        },
        "log.date" => Kind::OneOf {
            words: &[
                "relative",
                "local",
                "iso",
                "iso8601",
                "iso-strict",
                "iso8601-strict",
                "rfc",
                "rfc2822",
                "short",
                "raw",
                "human",
                "unix",
                "default",
            ],
            empty: false,
        },
        "log.diffmerges" => Kind::OneOf {
            words: &[
                "off",
                "none",
                "on",
                "m",
                "first-parent",
                "1",
                "separate",
                "combined",
                "c",
                "dense-combined",
                "cc",
                "remerge",
                "r",
            ],
            empty: false,
        },
        "pack.packsizelimit" => SIZE,
        "push.default" => Kind::OneOf {
            words: &[
                "nothing", "matching", "simple", "upstream", "tracking", "current",
            ],
            empty: false,
        },
        "transfer.credentialsinurl" => Kind::OneOf {
            words: &["allow", "warn", "die"],
            empty: false,
        },
        _ => return None,
    });
}
