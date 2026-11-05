//! What:
//!  The table of which configuration keys Git reads at startup and what kind of value each takes: the word lists, the key
//!  lists, the color words, the value kinds and the lookup from a key to its kind.
//! Why:
//!  `repo_read_config_typed.rs` checks whether a value is one Git accepts silently; the table it checks against is data and
//!  lookup that do not use the checking code, so they sit in a file of their own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Kind = { kind: 'boolean' } | { kind: 'integer'; low: number; high: number } | ...;
//! function kindOf(key: string): Kind | undefined;
//! ```


/// The words Git reads as true.
pub(crate) const TRUE_WORDS: [&str; 3] = ["true", "yes", "on"];

/// The words Git reads as false.
pub(crate) const FALSE_WORDS: [&str; 3] = ["false", "no", "off"];

/// Keys whose value is a plain boolean.
pub(crate) const BOOLEAN_KEYS: [&str; 49] = [
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
pub(crate) const TEXT_KEYS: [&str; 21] = [
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
pub(crate) const PATHNAME_KEYS: [&str; 5] = ["core.attributesfile", "core.excludesfile", "core.hookspath", "fsmonitor.socketdir", "mailmap.file"];

/// Per-driver keys (`diff.<driver>.<name>`) whose value is a boolean.
pub(crate) const DRIVER_BOOLEAN_NAMES: [&str; 3] = ["binary", "cachetextconv", "trustexitcode"];

/// Per-driver keys whose value is text.
pub(crate) const DRIVER_TEXT_NAMES: [&str; 4] = ["command", "textconv", "wordregex", "xfuncname"];

/// Per-remote keys (`remote.<name>.<key>`) whose value is a boolean.
pub(crate) const REMOTE_BOOLEAN_NAMES: [&str; 6] = ["mirror", "promisor", "prune", "prunetags", "skipdefaultupdate", "skipfetchall"];

/// Per-remote keys whose value is text.
pub(crate) const REMOTE_TEXT_NAMES: [&str; 12] = [
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
pub(crate) const GREP_COLOR_SLOTS: [&str; 10] = [
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
pub(crate) const PLAIN_COLOR_WORDS: [&str; 2] = ["normal", "default"];

/// The color words that also exist with a `bright` prefix.
pub(crate) const BRIGHT_COLOR_WORDS: [&str; 8] = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];

/// The attribute words Git reads (each also with a `no` or `no-` prefix).
pub(crate) const ATTRIBUTE_WORDS: [&str; 7] = ["bold", "dim", "ul", "blink", "reverse", "italic", "strike"];

/// How Git reads the value of one key.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
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
pub(crate) const MINIMUM_ABBREVIATION: i64 = 4;

/// The longest `core.abbrev` the proof accepts (the longest object name in hexadecimal digits).
pub(crate) const MAXIMUM_ABBREVIATION: i64 = 64;

/// The compression levels of zlib, with the default marker `-1`.
pub(crate) const COMPRESSION: Kind = Kind::Integer { low: -1, high: 9 };

/// A number Git stores in an `int`, not below zero.
pub(crate) const NON_NEGATIVE: Kind = Kind::Integer { low: 0, high: i32::MAX as i64 };

/// A number Git stores in an `unsigned long`; the proof keeps to the range both widths hold.
pub(crate) const SIZE: Kind = Kind::Integer { low: 0, high: i32::MAX as i64 };

/// Split a canonical key into section, subsection and name; the section and name are lower case.
pub(crate) fn split_key(key: &str) -> Option<(&str, Option<&str>, &str)> {
    let (section, rest) = key.split_once('.')?;
    return Some(match rest.rsplit_once('.') {
        Some((subsection, name)) => (section, Some(subsection), name),
        None => (section, None, rest),
    });
}

/// How the key's value is read, or `None` when Git reads no type for the key at startup.
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
        ("remote", "followremotehead") => Some(Kind::OneOf { words: &["create", "warn", "always", "never"], empty: false }),
        ("remote", "fetch" | "push") => Some(Kind::Refspec),
        ("url", "insteadof" | "pushinsteadof") => Some(Kind::Text),
        ("color", _) => color_kind(subsection, name),
        _ => None,
    };
}

/// The color keys the readers' commands load: `color.diff.<slot>`, `color.decorate.<slot>`, `color.advice.hint`, `color.grep.<slot>`.
pub(crate) fn color_kind(subsection: Option<&str>, name: &str) -> Option<Kind> {
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
