//! What:
//!  The four commands a typed network command covers,
//!  and the first of one's words that is not an option,
//!  read with Git's own option table for that command.
//! Why:
//!  The remote decides whether the command is relayed,
//!  so it must be read the way Git reads its arguments:
//!  a value of `--depth` is not a remote,
//!  and a word Git's table does not know leaves the remote unknown,
//!  which keeps today's process replacement
//!  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type TypedNetworkCommand = 'clone' | 'fetch' | 'pull' | 'push';
//! // const remote = remoteOperand(command, region); // undefined when Git's table does not cover the words
//! ```

// What: `OsString` is owned operating-system text of raw bytes (its sibling `String` must be UTF-8).
// Why:  The region is the person's own words, passed to Git without being decoded.
//
// In TS you'd write (pseudocode):
// ```ts
// // process.argv entries, byte-preserving.
// ```
use std::ffi::OsString;
// What: The option grammar this reading walks with: one row per option, its arity, and the parse that reports the positions
//       of the words that are neither an option nor a value.
// Why:  Git's own table is what decides, and a second reading of the same words would disagree with it.
//
// In TS you'd write (pseudocode):
// ```ts
// type OptionSpec = { short?: string; long?: string; arity: 'none' | 'required' | 'optional' };
// ```
use crate::command_options::{
    Arity,
    DEFAULT_MODE,
    OptionSpec,
    ParsedOptions,
    parse_options,
};

/// What:
///  A typed network command whose remote may be carried by ssh.
/// Why:
///  These are the four the decision covers;
///  a command outside this list keeps the process replacement it has today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TypedNetworkCommand = 'clone' | 'fetch' | 'pull' | 'push';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedNetworkCommand {
    /// `git clone`.
    Clone,
    /// `git fetch`.
    Fetch,
    /// `git pull`.
    Pull,
    /// `git push`.
    Push,
}

/// What:
///  The remote operand the person wrote:
///  the first token that is not an option,
///  read with Git's own option table.
/// Why:
///   `git fetch --depth 1 origin` names `origin`,
///  and `1` is a value;
///   a table-less scan would take either.
///   A word Git's table does not know ends the reading with nothing,
///   because a value misread as a remote could relay a command whose transport is not ssh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remoteOperand(region: string[]): string | undefined {
///   const parsed = parseOptions(region, TABLE[command], {keepUnknown: false, stopAtNonOption: false}, []);
///   return parsed ? region[parsed.leading[0]] : undefined;
/// }
/// ```
pub fn remote_operand(
    command: TypedNetworkCommand,
    region: &[OsString],
) -> Option<&OsString> {
    // What: `ParsedOptions.leading` is the positions of the tokens that are not options and not an option's value.
    // Why:   The first of them is where Git reads the remote.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parsed: ParsedOptions = parseOptions(...);
    // ```
    let parsed: ParsedOptions = parse_options(region, table_of(command), DEFAULT_MODE, &[]).ok()?;
    let first: usize = *parsed.leading.first()?;
    return region.get(first);
}

/// What:
///  Git's own option table for one of the four commands.
/// Why:
///  The table decides which token is a value and which is the remote,
///  and a wrong guess relays a command whose remote is not
///  ssh;
///  the rows mirror `git <command> -h` of Git 2.56.1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TABLE = { clone: CLONE_TABLE, fetch: FETCH_TABLE, pull: PULL_TABLE, push: PUSH_TABLE };
/// ```
fn table_of(command: TypedNetworkCommand) -> &'static [OptionSpec] {
    return match command {
        TypedNetworkCommand::Clone => CLONE_TABLE,
        TypedNetworkCommand::Fetch => FETCH_TABLE,
        TypedNetworkCommand::Pull => PULL_TABLE,
        // The push table is the one the policy already reads, so one table serves both.
        TypedNetworkCommand::Push => crate::command_push::PUSH_TABLE,
    };
}

/// What:
///  One row of an option table:
///  no identifier is read here,
///  so every row says `UNREAD`.
/// Why:
///  The reading needs only each option's name and whether a value follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const row = (short, long, arity) => ({ id: 0, short, long, arity, negatable: true });
/// ```
const fn row(
    short: Option<u8>,
    long: Option<&'static str>,
    arity: Arity,
) -> OptionSpec {
    return OptionSpec {
        id: crate::command_options::UNREAD,
        short,
        long,
        arity,
        negatable: true,
    };
}

/// What:
///  `git clone`'s options,
///  from `git clone -h` of Git 2.56.1.
/// Why:
///  A clone carries its URL in its own words,
///  so finding the URL needs to know which words are values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLONE_TABLE = [row('v', 'verbose', 'none'), /* ... */];
/// ```
pub const CLONE_TABLE: &[OptionSpec] = &[
    row(Some(b'v'), Some("verbose"), Arity::None),
    row(Some(b'q'), Some("quiet"), Arity::None),
    row(None, Some("progress"), Arity::None),
    row(None, Some("reject-shallow"), Arity::None),
    row(Some(b'n'), Some("no-checkout"), Arity::None),
    row(None, Some("checkout"), Arity::None),
    row(None, Some("bare"), Arity::None),
    row(None, Some("mirror"), Arity::None),
    row(Some(b'l'), Some("local"), Arity::None),
    row(None, Some("no-hardlinks"), Arity::None),
    row(None, Some("hardlinks"), Arity::None),
    row(Some(b's'), Some("shared"), Arity::None),
    row(None, Some("recurse-submodules"), Arity::Optional),
    row(None, Some("recursive"), Arity::Optional),
    row(Some(b'j'), Some("jobs"), Arity::Required),
    row(None, Some("template"), Arity::Required),
    row(None, Some("reference"), Arity::Required),
    row(None, Some("reference-if-able"), Arity::Required),
    row(None, Some("dissociate"), Arity::None),
    row(Some(b'o'), Some("origin"), Arity::Required),
    row(Some(b'b'), Some("branch"), Arity::Required),
    row(None, Some("revision"), Arity::Required),
    row(Some(b'u'), Some("upload-pack"), Arity::Required),
    row(None, Some("depth"), Arity::Required),
    row(None, Some("shallow-since"), Arity::Required),
    row(None, Some("shallow-exclude"), Arity::Required),
    row(None, Some("single-branch"), Arity::None),
    row(None, Some("tags"), Arity::None),
    row(None, Some("shallow-submodules"), Arity::None),
    row(None, Some("separate-git-dir"), Arity::Required),
    row(None, Some("ref-format"), Arity::Required),
    row(Some(b'c'), Some("config"), Arity::Required),
    row(None, Some("server-option"), Arity::Required),
    row(Some(b'4'), Some("ipv4"), Arity::None),
    row(Some(b'6'), Some("ipv6"), Arity::None),
    row(None, Some("filter"), Arity::Required),
    row(None, Some("also-filter-submodules"), Arity::None),
    row(None, Some("remote-submodules"), Arity::None),
    row(None, Some("sparse"), Arity::None),
    row(None, Some("bundle-uri"), Arity::Required),
];

/// What:
///  `git fetch`'s options,
///  from `git fetch -h` of Git 2.56.1.
/// Why:
///  A fetch names its remote as its first positional word,
///  and its option values must not be taken for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FETCH_TABLE = [row('v', 'verbose', 'none'), /* ... */];
/// ```
pub const FETCH_TABLE: &[OptionSpec] = &[
    row(Some(b'v'), Some("verbose"), Arity::None),
    row(Some(b'q'), Some("quiet"), Arity::None),
    row(None, Some("all"), Arity::None),
    row(None, Some("set-upstream"), Arity::None),
    row(Some(b'a'), Some("append"), Arity::None),
    row(None, Some("atomic"), Arity::None),
    row(None, Some("upload-pack"), Arity::Required),
    row(Some(b'f'), Some("force"), Arity::None),
    row(Some(b'm'), Some("multiple"), Arity::None),
    row(Some(b't'), Some("tags"), Arity::None),
    row(Some(b'n'), Some("no-tags"), Arity::None),
    row(Some(b'j'), Some("jobs"), Arity::Required),
    row(None, Some("prefetch"), Arity::None),
    row(Some(b'p'), Some("prune"), Arity::None),
    row(Some(b'P'), Some("prune-tags"), Arity::None),
    row(None, Some("recurse-submodules"), Arity::Optional),
    row(None, Some("dry-run"), Arity::None),
    row(None, Some("porcelain"), Arity::None),
    row(None, Some("write-fetch-head"), Arity::None),
    row(Some(b'k'), Some("keep"), Arity::None),
    row(Some(b'u'), Some("update-head-ok"), Arity::None),
    row(None, Some("progress"), Arity::None),
    row(None, Some("depth"), Arity::Required),
    row(None, Some("shallow-since"), Arity::Required),
    row(None, Some("shallow-exclude"), Arity::Required),
    row(None, Some("deepen"), Arity::Required),
    row(None, Some("unshallow"), Arity::None),
    row(None, Some("refetch"), Arity::None),
    row(None, Some("update-shallow"), Arity::None),
    row(None, Some("refmap"), Arity::Required),
    row(Some(b'o'), Some("server-option"), Arity::Required),
    row(Some(b'4'), Some("ipv4"), Arity::None),
    row(Some(b'6'), Some("ipv6"), Arity::None),
    row(None, Some("negotiation-restrict"), Arity::Required),
    row(None, Some("negotiation-tip"), Arity::Required),
    row(None, Some("negotiation-include"), Arity::Required),
    row(None, Some("negotiate-only"), Arity::None),
    row(None, Some("filter"), Arity::Required),
    row(None, Some("auto-maintenance"), Arity::None),
    row(None, Some("auto-gc"), Arity::None),
    row(None, Some("show-forced-updates"), Arity::None),
    row(None, Some("write-commit-graph"), Arity::None),
    row(None, Some("stdin"), Arity::None),
];

/// What:
///  `git pull`'s options,
///  from `git pull -h` of Git 2.56.1.
/// Why:
///  A pull's remote is its first positional word,
///  and its option values are many.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PULL_TABLE = [row('v', 'verbose', 'none'), /* ... */];
/// ```
pub const PULL_TABLE: &[OptionSpec] = &[
    row(Some(b'v'), Some("verbose"), Arity::None),
    row(Some(b'q'), Some("quiet"), Arity::None),
    row(None, Some("progress"), Arity::None),
    row(None, Some("recurse-submodules"), Arity::Optional),
    row(Some(b'r'), Some("rebase"), Arity::Optional),
    row(Some(b'n'), Some("no-diffstat"), Arity::None),
    row(None, Some("stat"), Arity::None),
    row(None, Some("compact-summary"), Arity::None),
    row(None, Some("log"), Arity::Optional),
    row(None, Some("signoff"), Arity::Optional),
    row(None, Some("squash"), Arity::None),
    row(None, Some("commit"), Arity::None),
    row(None, Some("edit"), Arity::None),
    row(None, Some("cleanup"), Arity::Required),
    row(None, Some("ff"), Arity::None),
    row(None, Some("ff-only"), Arity::None),
    row(None, Some("verify"), Arity::None),
    row(None, Some("verify-signatures"), Arity::None),
    row(None, Some("autostash"), Arity::None),
    row(Some(b's'), Some("strategy"), Arity::Required),
    row(Some(b'X'), Some("strategy-option"), Arity::Required),
    row(Some(b'S'), Some("gpg-sign"), Arity::Optional),
    row(None, Some("allow-unrelated-histories"), Arity::None),
    row(None, Some("all"), Arity::None),
    row(Some(b'a'), Some("append"), Arity::None),
    row(None, Some("upload-pack"), Arity::Required),
    row(Some(b'f'), Some("force"), Arity::None),
    row(Some(b't'), Some("tags"), Arity::None),
    row(Some(b'p'), Some("prune"), Arity::None),
    row(Some(b'j'), Some("jobs"), Arity::Optional),
    row(None, Some("dry-run"), Arity::None),
    row(Some(b'k'), Some("keep"), Arity::None),
    row(None, Some("depth"), Arity::Required),
    row(None, Some("shallow-since"), Arity::Required),
    row(None, Some("shallow-exclude"), Arity::Required),
    row(None, Some("deepen"), Arity::Required),
    row(None, Some("unshallow"), Arity::None),
    row(None, Some("update-shallow"), Arity::None),
    row(None, Some("refmap"), Arity::Required),
    row(Some(b'o'), Some("server-option"), Arity::Required),
    row(Some(b'4'), Some("ipv4"), Arity::None),
    row(Some(b'6'), Some("ipv6"), Arity::None),
    row(None, Some("negotiation-restrict"), Arity::Required),
    row(None, Some("negotiation-tip"), Arity::Required),
    row(None, Some("negotiation-include"), Arity::Required),
    row(None, Some("show-forced-updates"), Arity::None),
    row(None, Some("set-upstream"), Arity::None),
];
