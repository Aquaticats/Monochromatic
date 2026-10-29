//! What:
//!  The complete `git push` option table of Git 2.56.0 and the facts read from it.
//! Why:
//!  The atomic-push transform must know whether the caller already chose atomicity,
//!      and the manual-push gate whether the push is a dry run;
//!  both in every spelling Git
//!      accepts (`--at`,
//!  `--no-at`,
//!  `-nf`),
//!  which a list of exact tokens misses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parsePushRegion(postSubcommandArgs): PushRegion  (throws OptionError)
//! ```

/// What:
///  Bring the tokenizer,
///  its table builder and its questions into this file.
/// Why:
///   This module only declares Git's table and interprets what the tokenizer found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions, row } from './command_options.ts';
/// ```
use super::command_options::{
    Arity, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, UNREAD, WrapperOccurrence,
    parse_options, row,
};
use super::command_options_query::{is_enabled, is_stated, positional_tokens};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// `--atomic`:
///  request an atomic transaction on the remote side.
pub const ATOMIC: u16 = 1;
/// `-n`,
///  `--dry-run`:
///  do everything except send the updates.
pub const DRY_RUN: u16 = 2;
/// `-f`,
///  `--force`:
///  update a ref that is not a descendant.
pub const FORCE: u16 = 4;
/// `--force-with-lease`:
///  force when the remote ref is still what the caller expected.
pub const FORCE_WITH_LEASE: u16 = 8;
/// `--force-if-includes`:
///  a safety check that only means something with a forced push.
pub const FORCE_IF_INCLUDES: u16 = 16;
/// `--mirror`:
///  make the remote's refs equal the local ones.
pub const MIRROR: u16 = 32;
/// `-d`,
///  `--delete`:
///  remove remote refs.
pub const DELETE: u16 = 64;
/// `--all` and its alias `--branches`:
///  every local branch.
pub const ALL: u16 = 128;
/// `--tags`:
///  every local tag.
pub const TAGS: u16 = 256;
/// `--prune`:
///  remove remote refs that have no local counterpart.
pub const PRUNE: u16 = 512;
/// `--follow-tags`:
///  add the annotated tags that point into the pushed history.
pub const FOLLOW_TAGS: u16 = 1024;
/// `--repo`:
///  the destination is an option and not the first positional argument.
pub const REPO: u16 = 2048;
/// `--porcelain`:
///  machine-readable output that a script parses.
pub const PORCELAIN: u16 = 4096;
/// `-u`,
///  `--set-upstream`:
///  record the pushed branch as the local branch's upstream.
pub const SET_UPSTREAM: u16 = 8192;

/// What:
///  Rows of `options[]` in `cmd_push` (builtin/push.c:707-743),
///  in source order.
///       `&[OptionSpec]` is a borrowed table baked into the program.
/// Why:
///   `--branches` is an `OPT_ALIAS` of `--all`;
///  the two names share no prefix,
///  so the
///       alias needs no special ambiguity handling and is a row of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const PUSH_TABLE = [row(UNREAD, 'v', 'verbose', 'none', true), /* ... */] as const;
/// ```
pub const PUSH_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(REPO, None, Some("repo"), Arity::Required, true),
    row(ALL, None, Some("all"), Arity::None, true),
    row(ALL, None, Some("branches"), Arity::None, true),
    row(MIRROR, None, Some("mirror"), Arity::None, true),
    row(DELETE, Some(b'd'), Some("delete"), Arity::None, true),
    row(TAGS, None, Some("tags"), Arity::None, true),
    row(DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(PORCELAIN, None, Some("porcelain"), Arity::None, true),
    row(FORCE, Some(b'f'), Some("force"), Arity::None, true),
    // `PARSE_OPT_OPTARG`: only `--force-with-lease=<refname>:<expect>` carries a value.
    row(
        FORCE_WITH_LEASE,
        None,
        Some("force-with-lease"),
        Arity::Optional,
        true,
    ),
    row(FORCE_IF_INCLUDES, None, Some("force-if-includes"), Arity::None, true),
    row(
        UNREAD,
        None,
        Some("recurse-submodules"),
        Arity::Required,
        true,
    ),
    row(UNREAD, None, Some("thin"), Arity::None, true),
    row(UNREAD, None, Some("receive-pack"), Arity::Required, true),
    row(UNREAD, None, Some("exec"), Arity::Required, true),
    row(SET_UPSTREAM, Some(b'u'), Some("set-upstream"), Arity::None, true),
    row(UNREAD, None, Some("progress"), Arity::None, true),
    row(PRUNE, None, Some("prune"), Arity::None, true),
    row(UNREAD, None, Some("no-verify"), Arity::None, true),
    row(FOLLOW_TAGS, None, Some("follow-tags"), Arity::None, true),
    // `PARSE_OPT_OPTARG`: only `--signed=<mode>` carries a value.
    row(UNREAD, None, Some("signed"), Arity::Optional, true),
    row(ATOMIC, None, Some("atomic"), Arity::None, true),
    row(
        UNREAD,
        Some(b'o'),
        Some("push-option"),
        Arity::Required,
        true,
    ),
    // `OPT_IPVERSION` rows are `PARSE_OPT_NONEG` (parse-options.h:636-640).
    row(UNREAD, Some(b'4'), Some("ipv4"), Arity::None, false),
    row(UNREAD, Some(b'6'), Some("ipv6"), Arity::None, false),
];

/// What:
///  Final-state facts of one `git push` region.
///  `bool` is true or false;
///       `Vec<WrapperOccurrence>` is an owned list of wrapper-flag positions.
/// Why:
///   `atomic_stated` is "the caller chose",
///  in either direction;
///  `dry_run` is the state
///       after the last `-n`,
///  `--dry-run` or `--no-dry-run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushRegion = { hasAtomicChoice: boolean; isDryRun: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushRegion {
    /// `--atomic` or `--no-atomic` was written,
    ///  in any accepted abbreviation.
    pub atomic_stated: bool,
    /// The push sends no updates.
    pub dry_run: bool,
    /// Wrapper-only flags in option position;
    ///  `flag` indexes the caller's list.
    pub wrapper: Vec<WrapperOccurrence>,
}

/// What:
///  Parse the region after `push`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal,
///  never a guessed fact set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePushRegion(region: string[], wrapperFlags: string[]): PushRegion; // throws
/// ```
pub fn parse_push_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<PushRegion, OptionError> {
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, PUSH_TABLE, DEFAULT_MODE, wrapper_flags)?;
    // `Ok(x)` is the success case of `Result`.
    return Ok(PushRegion {
        atomic_stated: is_stated(&parsed, ATOMIC),
        dry_run: is_enabled(&parsed, DRY_RUN),
        wrapper: parsed.wrapper,
    });
}

/// Options after which a rejected push is never integrated:
///  each either rewrites or removes remote history, sends more than one kind of ref, changes what the caller
///  reads back, or names its destination in a way the stacked push does not reproduce.
const NOT_STACKABLE: &[u16] = &[
    DRY_RUN,
    FORCE,
    FORCE_WITH_LEASE,
    FORCE_IF_INCLUDES,
    MIRROR,
    DELETE,
    ALL,
    TAGS,
    PRUNE,
    FOLLOW_TAGS,
    REPO,
    PORCELAIN,
    SET_UPSTREAM,
];

/// What:
///  The option tokens of a push that may be integrated after a rejection, or nothing when it may not.
///  `Option<Vec<OsString>>` is "the tokens, or no answer".
/// Why:
///   Only a plain update of branches keeps its meaning when the wrapper merges what landed on the remote and pushes the
///  merge; the stacked push repeats these tokens and replaces the remote and refspec arguments,
///  so the caller's `--no-verify`, `--push-option` and the like still apply.
///   A region the table refuses, a forced refspec (`+src:dst`) and every option in `NOT_STACKABLE` give nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackableOptionTokens(region: string[]): string[] | undefined;
/// ```
pub fn stackable_option_tokens(region: &[OsString]) -> Option<Vec<OsString>> {
    let parsed: ParsedOptions = parse_options(region, PUSH_TABLE, DEFAULT_MODE, &[]).ok()?;
    if NOT_STACKABLE.iter().any(|id| is_enabled(&parsed, *id)) {
        return None;
    }
    let positionals: Vec<usize> = positional_tokens(&parsed, region.len());
    if positionals
        .iter()
        .any(|index| region[*index].as_encoded_bytes().first() == Some(&b'+'))
    {
        return None;
    }
    return Some(
        region
            .iter()
            .enumerate()
            .filter(|(index, _)| !positionals.contains(index))
            .map(|(_, token)| token.clone())
            .collect(),
    );
}

/// Facts in every accepted spelling,
///  and real-Git controls of the table and its abbreviations.
#[cfg(test)]
#[path = "command_push_tests.rs"]
mod tests;
