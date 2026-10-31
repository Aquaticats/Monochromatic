# CLI output and exit codes

## Status

Accepted,
2026-10-31.
 Recorded from conventions three CLIs already share
 (`package/crate-release/cli`,
 `package/env-local-json/core`,
 `package/git-policy/cli`);
 adopted by `package/search-fetch/cli`.

## Context

The exit-code and stream conventions lived only in package READMEs:
 `package/crate-release/cli/src/constants.ts` defines
 `failureExitCode = 1` and `usageExitCode = 2`,
 `package/env-local-json/core/README.md` documents
 "the sops exit status is passed through,
  malformed usage exits with status 2",
 and `package/git-policy/cli/README.md` documents
 "0 means clean or warnings only,
  1 means final error findings,
  2 means a usage,
configuration,
transaction or engine failure".
 `package/git-clone-size/cli/README.md` adds the stream rule:
 "Output is JSONL only ...
  Diagnostics go to stderr,
so stdout stays valid for `jq`".

Every new CLI re-derived these from its siblings,
 and the `search-fetch` CLI grilling surfaced the gap.
 This document is the single recorded home.

## Decision

Exit codes:

- `0`:
  success,
  and explicit `--help` or `--version` output.
- `1`:
  the operation failed:
  provider failure or deadline expiry.
- `2`:
  the caller or configuration is wrong:
  usage error,
  configuration error,
  or policy refusal such as a blocklist hit.

Streams:

- Result text is the only stdout writer on a data run.
  Machine output stays parseable,
  so pipes into `jq` and into agent transcripts stay clean.
- Explicit `--help` and `--version` print to stdout and exit `0`;
  a paged `--help` is a normal pipeline.
- Misuse prints usage text to stderr and exits `2`.
  A failing run leaves stdout empty.
- Diagnostics and warnings go to stderr through tagged loggers
 (per `TLG`),
  never interleaved with results.

Usage text is generated from the same constants the parser enforces,
 so the printed text and the enforced behavior cannot drift
 (the `package/crate-release/cli/src/usage.ts` pattern).

`process.exitCode` is set explicitly only for the non-standard code
 `2`,
per `PP6`;
 `0` and `1` come from normal return and thrown errors.

## Rejected alternatives

### sysexits-style codes (64 through 78)

Rejected.
 Consumers here are agents and small scripts;
 a code table they must learn buys nothing over
 "caller wrong,
operation failed,
fine",
 which is what `0`/`1`/`2` already encodes.
 The three incumbent CLIs agree on `0`/`1`/`2`,
 so a fourth scheme would split the fleet.

### Human text or logs on stdout

Rejected.
 `package/git-clone-size/cli/README.md` records the reason:
 stdout must stay valid for `jq`.
 Any prompt or notice that lands on stdout breaks machine consumers
 silently,
which is the failure class this rule exists to prevent.

### Usage text on stderr even for explicit `--help`

Rejected.
 An explicit `--help` is the result the caller asked for,
 and belongs in a pipeline (`search-fetch --help | less`).
 Only misuse keeps usage on stderr.

## Considered home: an `AGENTS.md` rule

The doc-gap rule prefers tightening `AGENTS.md` first.
 A new `AGENTS.md` rule was considered and not taken:
 the maintainer declined a new `AGENTS.md` rule on 2026-10-31
 (the `RNL` entry in `forbidden-strings.append.local.txt`),
 and the adjacent rules already cover their own ground without
 absorbing this topic (`TLG` owns raw console versus loggers,
 `OCG` owns option grammar,
 `PP6` owns error text and `process.exitCode`).
 This document is therefore the single home;
 package READMEs should point here instead of restating the rules.

## When to revisit

- If a CLI needs a fourth exit code,
  for example a distinct timeout code,
  record it here first so the fleet stays consistent.
- If `AGENTS.md` grows a CLI-contract rule,
  fold this content into it and delete this document
  only once no durable fact lacks a home.
