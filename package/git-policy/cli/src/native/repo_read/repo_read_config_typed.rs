//! What:
//!  The proof that every configuration value Git types at startup, in every scope the readers look at, is a value
//!  Git accepts without a message.
//! Why:
//!  Git reads many keys for every command (`core.fileMode`, `core.compression`, `advice.*`, `diff.context`, and so
//!  on) and dies (`fatal: bad boolean config value 'maybe' for 'core.filemode'`), or warns, when a value does not
//!  fit the key's type.
//!  The readers never look at most of those keys, so without this proof they would answer where Git fails.
//!  The proof accepts only spellings it can show Git accepts (a conservative subset) and leaves every other
//!  spelling of a typed key to Git, which then words its own failure.
//!  The key table was built from the keys real Git 2.56.0 rejected or warned about when set to malformed values
//!  (`doc/planning/cli-git-in-process-reads.md`, "Typed configuration values"), and
//!  `every_documented_key_with_a_malformed_value_is_declined_or_equals_git` keeps it honest against the Git that
//!  runs the tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ok: boolean | undefined = everyTypedValueIsAccepted({ entries, scope }); // undefined: ask Git
//! ```

use crate::repo_read_config_scan::{ScopeEnvironment, include_target, is_include_key, read_following};
use crate::repo_read_config_text::{ConfigEntry, parse_config_text};
use std::ffi::OsString;
use std::path::Path;

/// How many nested includes are followed before the proof gives up; Git's own limit.
const INCLUDE_DEPTH_LIMIT: usize = 10;

/// How many environment-supplied entries are checked before the proof gives up.
const ENV_ENTRY_LIMIT: u64 = 1024;

/// The most digits of a number the proof accepts, so no accepted spelling can overflow the integer Git parses into.
const DIGIT_LIMIT: usize = 9;

/// The suffixes Git multiplies a number by, with their factors.
const UNIT_FACTORS: [(u8, i64); 3] = [(b'k', 1 << 10), (b'm', 1 << 20), (b'g', 1 << 30)];

/// The words Git reads as true.
const TRUE_WORDS: [&str; 3] = ["true", "yes", "on"];

/// The words Git reads as false.
const FALSE_WORDS: [&str; 3] = ["false", "no", "off"];

/// Keys whose value is a plain boolean.
const BOOLEAN_KEYS: [&str; 49] = [
    "color.advice",
    "color.pager",
    "commitgraph.readchangedpaths",
    "core.commitgraph",
    "core.filemode",
    "core.ignorecase",
    "core.ignorestat",
    "core.lockfilepid",
    "core.multipackindex",
    "core.precomposeunicode",
    "core.protecthfs",
    "core.protectntfs",
    "core.quotepath",
    "core.sparsecheckout",
    "core.sparsecheckoutcone",
    "core.symlinks",
    "core.trustctime",
    "core.usereplacerefs",
    "core.warnambiguousrefs",
    "diff.autorefreshindex",
    "diff.indentheuristic",
    "diff.mnemonicprefix",
    "diff.noprefix",
    "diff.relative",
    "diff.suppressblankempty",
    "diff.trustexitcode",
    "feature.experimental",
    "feature.manyfiles",
    "fetch.writecommitgraph",
    "format.encodeemailheaders",
    "fsmonitor.allowremote",
    "gc.writecommitgraph",
    "grep.column",
    "grep.extendedregexp",
    "grep.fullname",
    "grep.linenumber",
    "index.skiphash",
    "index.sparse",
    "log.abbrevcommit",
    "log.follow",
    "log.mailmap",
    "log.showroot",
    "log.showsignature",
    "pack.readreverseindex",
    "pack.usebitmapboundarytraversal",
    "pack.usepathwalk",
    "pack.usesparse",
    "sparse.expectfilesoutsideofpatterns",
    "user.useconfigonly",
];

/// Keys whose value is any text, but never a bare name.
const TEXT_KEYS: [&str; 21] = [
    "attr.tree",
    "author.email",
    "author.name",
    "committer.email",
    "committer.name",
    "core.askpass",
    "core.checkroundtripencoding",
    "core.editor",
    "core.untrackedcache",
    "core.whitespace",
    "diff.dstprefix",
    "diff.external",
    "diff.srcprefix",
    "diff.wordregex",
    "format.subjectprefix",
    "i18n.commitencoding",
    "i18n.logoutputencoding",
    "mailmap.blob",
    "remote.pushdefault",
    "user.email",
    "user.name",
];

/// Keys whose value is a path, which Git expands (`~/x`, and `~user/x`, which fails for an unknown user).
const PATHNAME_KEYS: [&str; 5] = ["core.attributesfile", "core.excludesfile", "core.hookspath", "fsmonitor.socketdir", "mailmap.file"];

/// Per-driver keys (`diff.<driver>.<name>`) whose value is a boolean.
const DRIVER_BOOLEAN_NAMES: [&str; 3] = ["binary", "cachetextconv", "trustexitcode"];

/// Per-driver keys whose value is text.
const DRIVER_TEXT_NAMES: [&str; 4] = ["command", "textconv", "wordregex", "xfuncname"];

/// Per-remote keys (`remote.<name>.<key>`) whose value is a boolean.
const REMOTE_BOOLEAN_NAMES: [&str; 6] = ["mirror", "promisor", "prune", "prunetags", "skipdefaultupdate", "skipfetchall"];

/// Per-remote keys whose value is text.
const REMOTE_TEXT_NAMES: [&str; 12] = [
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

/// The slots `color.grep.<slot>` knows; any other slot is a failure in Git.
const GREP_COLOR_SLOTS: [&str; 10] = [
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
const PLAIN_COLOR_WORDS: [&str; 2] = ["normal", "default"];

/// The color words that also exist with a `bright` prefix.
const BRIGHT_COLOR_WORDS: [&str; 8] = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];

/// The attribute words Git reads (each also with a `no` or `no-` prefix).
const ATTRIBUTE_WORDS: [&str; 7] = ["bold", "dim", "ul", "blink", "reverse", "italic", "strike"];

/// How Git reads the value of one key.
#[derive(Clone, Copy)]
enum Kind {
    /// `true yes on false no off`, a decimal number, or an empty value; a bare name is true.
    Boolean,
    /// A boolean, or one of the listed words.
    BooleanOr(&'static [&'static str]),
    /// A boolean or a decimal number; a bare name is true.
    IntegerOrBoolean,
    /// A decimal number inside the range, with an optional `k`, `m` or `g` suffix; never bare, never empty.
    Integer { low: i64, high: i64 },
    /// Any text; never bare.
    Text,
    /// Any text that is not a `~user` path; never bare.
    Pathname,
    /// Exactly one of the listed words, or an empty value when `empty` is set; never bare.
    OneOf { words: &'static [&'static str], empty: bool },
    /// A comma-separated list of the listed words, each with an optional `-` prefix when `negatable` is set.
    Words { words: &'static [&'static str], empty: bool, negatable: bool },
    /// A color specification of at most two colors and any attributes.
    Color,
    /// `core.abbrev`: `auto`, a false word, an empty value, or a number from the shortest abbreviation up.
    Abbreviation,
    /// `core.commentChar` and `core.commentString`: not empty, no line break, never bare.
    CommentText,
    /// A refspec `[+]refs/<name>:refs/<name>` whose names are plain and whose wildcards match; every other form is
    ///  left to Git, which checks a refspec against its full rules.
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
    /// A key Git warns about, or fails on, in spellings the proof does not model: its presence alone is declined.
    Refused,
}

/// The index of Git's shortest accepted `core.abbrev`.
const MINIMUM_ABBREVIATION: i64 = 4;

/// The longest `core.abbrev` the proof accepts (the longest object name in hexadecimal digits).
const MAXIMUM_ABBREVIATION: i64 = 64;

/// The compression levels of zlib, with the default marker `-1`.
const COMPRESSION: Kind = Kind::Integer { low: -1, high: 9 };

/// A number Git stores in an `int`, not below zero.
const NON_NEGATIVE: Kind = Kind::Integer { low: 0, high: i32::MAX as i64 };

/// A number Git stores in an `unsigned long`; the proof keeps to the range both widths hold.
const SIZE: Kind = Kind::Integer { low: 0, high: i32::MAX as i64 };

/// Split a canonical key into section, subsection and name; the section and name are lower case.
fn split_key(key: &str) -> Option<(&str, Option<&str>, &str)> {
    let (section, rest) = key.split_once('.')?;
    return Some(match rest.rsplit_once('.') {
        Some((subsection, name)) => (section, Some(subsection), name),
        None => (section, None, rest),
    });
}

/// How the key's value is read, or `None` when Git reads no type for the key at startup.
fn kind_of(key: &str) -> Option<Kind> {
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
        ("remote", "followremotehead") => Some(Kind::OneOf { words: &["create", "warn", "always", "never"], empty: false }),
        ("remote", "fetch" | "push") => Some(Kind::Refspec),
        ("url", "insteadof" | "pushinsteadof") => Some(Kind::Text),
        ("color", _) => color_kind(subsection, name),
        _ => None,
    };
}

/// The color keys the readers' commands load: `color.diff.<slot>`, `color.decorate.<slot>`, `color.advice.hint`, `color.grep.<slot>`.
fn color_kind(subsection: Option<&str>, name: &str) -> Option<Kind> {
    return match subsection {
        Some("diff" | "decorate" | "advice") => Some(Kind::Color),
        Some("grep") if GREP_COLOR_SLOTS.contains(&name) => Some(Kind::Color),
        Some("grep") => Some(Kind::Refused),
        _ => None,
    };
}

/// The keys of the form `section.name` whose type is not plain.
fn plain_kind(key: &str) -> Option<Kind> {
    return Some(match key {
        "branch.autosetupmerge" => Kind::BooleanOr(&["always", "inherit", "simple"]),
        "branch.autosetuprebase" => Kind::OneOf { words: &["never", "local", "remote", "always"], empty: false },
        "color.ui" | "color.diff" | "color.grep" => Kind::BooleanOr(&["always", "never", "auto"]),
        "commitgraph.changedpathsversion" | "commitgraph.generationversion" => NON_NEGATIVE,
        "core.abbrev" => Kind::Abbreviation,
        "core.autocrlf" => Kind::BooleanOr(&["input"]),
        "core.bigfilethreshold" | "core.deltabasecachelimit" | "core.packedgitlimit" | "core.packedgitwindowsize" => SIZE,
        "core.checkstat" => Kind::OneOf { words: &["default", "minimal"], empty: false },
        "core.commentchar" | "core.commentstring" => Kind::CommentText,
        "core.compression" | "core.loosecompression" | "pack.compression" => COMPRESSION,
        "core.createobject" => Kind::OneOf { words: &["rename", "link"], empty: false },
        "core.fsmonitor" => Kind::OffOrBuiltinDaemon,
        "trace2.normaltarget" | "trace2.eventtarget" | "trace2.perftarget" => Kind::TraceTarget,
        "core.fsync" => Kind::Words {
            words: &["loose-object", "pack", "pack-metadata", "commit-graph", "index", "objects", "reference", "derived-metadata", "committed", "added", "all", "none"],
            empty: false,
            negatable: true,
        },
        "core.fsyncmethod" => Kind::OneOf { words: &["fsync", "writeout-only", "batch"], empty: false },
        "core.fsyncobjectfiles" | "diff.dirstat" => Kind::Refused,
        "core.maxtreedepth" | "diff.context" | "diff.interhunkcontext" | "diff.renamelimit" | "diff.statgraphwidth" | "diff.statnamewidth" | "format.filenamemaxlength" => NON_NEGATIVE,
        "core.safecrlf" => Kind::BooleanOr(&["warn"]),
        "diff.algorithm" => Kind::OneOf { words: &["default", "myers", "minimal", "patience", "histogram"], empty: false },
        "diff.colormoved" => Kind::BooleanOr(&["default", "blocks", "zebra", "dimmed-zebra", "plain"]),
        "diff.colormovedws" => Kind::Words {
            words: &["no", "ignore-space-at-eol", "ignore-space-change", "ignore-all-space", "allow-indentation-change"],
            empty: false,
            negatable: false,
        },
        "diff.ignoresubmodules" => Kind::OneOf { words: &["none", "untracked", "dirty", "all"], empty: false },
        "diff.renames" => Kind::BooleanOr(&["copy", "copies"]),
        "diff.submodule" => Kind::OneOf { words: &["log", "short", "diff"], empty: false },
        "diff.wserrorhighlight" => Kind::Words { words: &["old", "new", "context", "all", "default", "none"], empty: true, negatable: false },
        "fetch.negotiationalgorithm" => Kind::OneOf { words: &["skipping", "noop", "consecutive", "default"], empty: false },
        "format.pretty" => Kind::OneOf { words: &["oneline", "short", "medium", "full", "fuller", "reference", "email", "raw", "mboxrd"], empty: true },
        "grep.patterntype" => Kind::OneOf { words: &["default", "basic", "extended", "fixed", "perl"], empty: false },
        "index.threads" => Kind::IntegerOrBoolean,
        "index.version" => Kind::Integer { low: 2, high: 4 },
        "log.date" => Kind::OneOf {
            words: &["relative", "local", "iso", "iso8601", "iso-strict", "iso8601-strict", "rfc", "rfc2822", "short", "raw", "human", "unix", "default"],
            empty: false,
        },
        "log.diffmerges" => Kind::OneOf {
            words: &["off", "none", "on", "m", "first-parent", "1", "separate", "combined", "c", "dense-combined", "cc", "remerge", "r"],
            empty: false,
        },
        "pack.packsizelimit" => SIZE,
        "push.default" => Kind::OneOf { words: &["nothing", "matching", "simple", "upstream", "tracking", "current"], empty: false },
        "transfer.credentialsinurl" => Kind::OneOf { words: &["allow", "warn", "die"], empty: false },
        _ => return None,
    });
}

/// The decimal number a spelling stands for, or `None` for a spelling the proof does not read as one.
///  A sign, 1 to 9 digits with no leading zero (Git reads a leading zero as octal), and one optional unit suffix.
fn number_of(text: &str) -> Option<i64> {
    let bytes: &[u8] = text.as_bytes();
    let (negative, rest): (bool, &[u8]) = match bytes.first() {
        Some(b'-') => (true, &bytes[1..]),
        Some(b'+') => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    let (digits, factor): (&[u8], i64) = match rest.split_last() {
        Some((last, head)) => match UNIT_FACTORS.iter().find(|(unit, _)| { return *unit == last.to_ascii_lowercase(); }) {
            Some((_, factor)) => (head, *factor),
            None => (rest, 1),
        },
        None => (rest, 1),
    };
    if digits.is_empty() || digits.len() > DIGIT_LIMIT || !digits.iter().all(u8::is_ascii_digit) || (digits.len() > 1 && digits[0] == b'0') {
        return None;
    }
    let magnitude: i64 = std::str::from_utf8(digits).ok()?.parse::<i64>().ok()?.checked_mul(factor)?;
    return Some(if negative { -magnitude } else { magnitude });
}

/// Whether `text` is a spelling of a boolean Git reads: a word, an empty value, or a number.
fn is_boolean_text(text: &str) -> bool {
    let lowered: String = text.to_ascii_lowercase();
    return text.is_empty()
        || TRUE_WORDS.contains(&lowered.as_str())
        || FALSE_WORDS.contains(&lowered.as_str())
        || number_of(text).is_some_and(|number| { return i32::try_from(number).is_ok(); });
}

/// Whether `text` is a false word or empty, the spellings Git reads as false without a number.
fn is_false_text(text: &str) -> bool {
    return text.is_empty() || FALSE_WORDS.contains(&text.to_ascii_lowercase().as_str()) || number_of(text) == Some(0);
}

/// Whether `lowered` is an attribute word, with or without a `no` or `no-` prefix.
fn is_attribute_word(lowered: &str) -> bool {
    let bare: &str = lowered.strip_prefix("no-").or_else(|| { return lowered.strip_prefix("no"); }).unwrap_or(lowered);
    return ATTRIBUTE_WORDS.contains(&lowered) || ATTRIBUTE_WORDS.contains(&bare);
}

/// Whether one color word, attribute word, number or `#rrggbb` spelling is one Git reads.
fn is_color_word(word: &str) -> bool {
    let lowered_text: String = word.to_ascii_lowercase();
    let lowered: &str = lowered_text.as_str();
    if PLAIN_COLOR_WORDS.contains(&lowered) || BRIGHT_COLOR_WORDS.contains(&lowered) || is_attribute_word(lowered) {
        return true;
    }
    if lowered.strip_prefix("bright").is_some_and(|base| { return BRIGHT_COLOR_WORDS.contains(&base); }) {
        return true;
    }
    if let Some(hex) = lowered.strip_prefix('#') {
        return (hex.len() == 6 || hex.len() == 3) && hex.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
    }
    return number_of(word).is_some_and(|number| { return (0..=255).contains(&number); });
}

/// Whether `text` is at most two colors and any number of attributes, each a word Git reads.
fn is_color_text(text: &str) -> bool {
    let words: Vec<&str> = text.split_ascii_whitespace().collect();
    let colors: usize = words.iter().filter(|word| { return !is_attribute_word(word.to_ascii_lowercase().as_str()); }).count();
    return colors <= 2 && words.iter().all(|word| { return is_color_word(word); });
}

/// Whether `name` is a ref name or pattern of plain components under `refs/`, with at most one wildcard component.
fn is_plain_ref_pattern(name: &str) -> Option<bool> {
    let rest: &str = name.strip_prefix("refs/")?;
    let mut wildcards: usize = 0;
    for component in rest.split('/') {
        if component == "*" {
            wildcards += 1;
            continue;
        }
        let plain: bool = !component.is_empty()
            && !component.starts_with('.')
            && !component.contains("..")
            && !component.ends_with(".lock")
            && !component.ends_with('.')
            && component.bytes().all(|byte| { return byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'); });
        if !plain {
            return None;
        }
    }
    return if wildcards > 1 { None } else { Some(wildcards == 1) };
}

/// Whether `text` is `[+]refs/...:refs/...` with plain names and a wildcard on both sides or on neither.
fn is_plain_refspec(text: &str) -> bool {
    let Some((source, destination)) = text.strip_prefix('+').unwrap_or(text).split_once(':') else {
        return false;
    };
    return match (is_plain_ref_pattern(source), is_plain_ref_pattern(destination)) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    };
}

/// Whether `text` is a comma-separated list of the listed words.
fn is_word_list(text: &str, words: &[&str], empty: bool, negatable: bool) -> bool {
    if text.is_empty() {
        return empty;
    }
    return text.split(',').all(|item| {
        let word: &str = if negatable { item.strip_prefix('-').unwrap_or(item) } else { item };
        return words.contains(&word);
    });
}

/// Whether `value` is a spelling of `kind` that Git accepts silently.
fn is_accepted(kind: Kind, value: Option<&[u8]>) -> bool {
    let Some(bytes) = value else {
        // A bare name is a true boolean and nothing else.
        return matches!(kind, Kind::Boolean | Kind::BooleanOr(_) | Kind::IntegerOrBoolean);
    };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    return match kind {
        Kind::Boolean => is_boolean_text(text),
        Kind::BooleanOr(words) => is_boolean_text(text) || words.contains(&text),
        Kind::IntegerOrBoolean => is_boolean_text(text) || number_of(text).is_some_and(|number| { return i32::try_from(number).is_ok(); }),
        Kind::Integer { low, high } => number_of(text).is_some_and(|number| { return (low..=high).contains(&number); }),
        Kind::Text => true,
        Kind::Pathname => !text.starts_with('~') || text.starts_with("~/"),
        Kind::OneOf { words, empty } => (text.is_empty() && empty) || words.contains(&text),
        Kind::Words { words, empty, negatable } => is_word_list(text, words, empty, negatable),
        Kind::Color => is_color_text(text),
        Kind::Abbreviation => text == "auto" || is_false_text(text) || number_of(text).is_some_and(|number| { return (MINIMUM_ABBREVIATION..=MAXIMUM_ABBREVIATION).contains(&number); }),
        Kind::CommentText => !text.is_empty() && !text.contains('\n'),
        Kind::TraceTarget => text.is_empty() || text == "0" || text.eq_ignore_ascii_case("false"),
        Kind::OffOrBuiltinDaemon => is_false_text(text) || TRUE_WORDS.contains(&text.to_ascii_lowercase().as_str()),
        Kind::Refspec => is_plain_refspec(text),
        Kind::Refused => false,
    };
}

/// What:
///  Whether Git accepts the value of one entry silently.
/// Why:
///   A key the table does not list is read by no startup code of the commands the readers replace, so it passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const accepted = kindOf(entry.key) === undefined || isAccepted(kindOf(entry.key), entry.value);
/// ```
pub fn entry_is_accepted(key: &str, value: Option<&[u8]>) -> bool {
    return kind_of(key).is_none_or(|kind| { return is_accepted(kind, value); });
}

/// Whether the entries parsed from `path`, and every file they include, hold only accepted values;
///  `None` when a file cannot be read or parsed.
pub fn entries_are_accepted(entries: &[ConfigEntry], path: &Path, home: Option<&Path>, depth: usize) -> Option<bool> {
    if depth > INCLUDE_DEPTH_LIMIT {
        return None;
    }
    for entry in entries {
        if !entry_is_accepted(entry.key.as_str(), entry.value.as_deref()) {
            return Some(false);
        }
        if is_include_key(entry.key.as_str()) {
            let target = include_target(entry.value.as_deref()?, path, home)?;
            if !file_is_accepted(target.as_path(), home, depth + 1)? {
                return Some(false);
            }
        }
    }
    return Some(true);
}

/// The same proof for the file at `path`; a missing file is an empty file to Git.
fn file_is_accepted(path: &Path, home: Option<&Path>, depth: usize) -> Option<bool> {
    let Some(text) = read_following(path)? else {
        return Some(true);
    };
    let entries: Vec<ConfigEntry> = parse_config_text(text.as_slice())?;
    return entries_are_accepted(entries.as_slice(), path, home, depth);
}

/// What:
///  Whether the system and global files, and everything they include, hold only accepted values.
/// Why:
///   These scopes feed the same startup reads as the repository's own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scopeFiles(env).every((file) => fileIsAccepted(file)); // undefined when unreadable
/// ```
pub fn outer_scopes_are_accepted(scope: &ScopeEnvironment) -> Option<bool> {
    if scope.unresolvable {
        return None;
    }
    for file in scope.files() {
        if !file_is_accepted(file.as_path(), scope.home.as_deref(), 0)? {
            return Some(false);
        }
    }
    return Some(true);
}

/// The key as Git canonicalizes it: lower-case section and name, subsection kept.
fn canonical(key: &str) -> Option<String> {
    let (section, rest) = key.split_once('.')?;
    return Some(match rest.rsplit_once('.') {
        Some((subsection, name)) => format!("{}.{subsection}.{}", section.to_ascii_lowercase(), name.to_ascii_lowercase()),
        None => format!("{}.{}", section.to_ascii_lowercase(), rest.to_ascii_lowercase()),
    });
}

/// What:
///  Whether every `GIT_CONFIG_COUNT` entry among `variables` holds an accepted value.
/// Why:
///   Environment entries reach the same startup reads; a malformed set of entries is not accepted either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// envEntries(variables).every(({ key, value }) => entryIsAccepted(key, value)); // false when malformed
/// ```
pub fn environment_is_accepted(variables: &[(OsString, OsString)]) -> bool {
    let find = |name: &str| -> Option<&OsString> {
        return variables.iter().rev().find(|(candidate, _)| { return candidate == name; }).map(|(_, value)| { return value; });
    };
    let Some(count_text) = find("GIT_CONFIG_COUNT") else {
        return true;
    };
    let Some(count) = crate::child_environment::parse_config_count(count_text.as_os_str()) else {
        return false;
    };
    if count > ENV_ENTRY_LIMIT {
        return false;
    }
    for index in 0..count {
        let (Some(key), Some(value)) = (find(format!("GIT_CONFIG_KEY_{index}").as_str()), find(format!("GIT_CONFIG_VALUE_{index}").as_str())) else {
            return false;
        };
        let Some(key_text) = key.to_str() else {
            return false;
        };
        let Some(canonical_key) = canonical(key_text) else {
            return false;
        };
        if !entry_is_accepted(canonical_key.as_str(), Some(value.as_encoded_bytes())) {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
#[path = "repo_read_config_typed_tests.rs"]
mod tests;
