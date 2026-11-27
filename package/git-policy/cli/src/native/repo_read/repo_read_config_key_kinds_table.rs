//! What:
//!  The configuration keys whose value Git types at startup as a plain boolean,
//!  as any text,
//! or as a path it expands.
//! Why:
//!  `kind_of` answers these three kinds from the key alone,
//! before it looks at the shapes a section has of its own,
//!  and the tables are data that nothing else in the file decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const BOOLEAN_KEYS = new Set(['color.advice', ...]); // and the other two
//! ```

/// Keys whose value is a plain boolean.
pub(crate) const BOOLEAN_KEYS: &[&str] = &[
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

/// Keys whose value is any text,
/// but never a bare name.
pub(crate) const TEXT_KEYS: &[&str] = &[
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

/// Keys whose value is a path,
/// which Git expands (`~/x`,
/// and `~user/x`,
/// which fails for an unknown user).
pub(crate) const PATHNAME_KEYS: &[&str] = &[
    "core.attributesfile",
    "core.excludesfile",
    "core.hookspath",
    "fsmonitor.socketdir",
    "mailmap.file",
];
