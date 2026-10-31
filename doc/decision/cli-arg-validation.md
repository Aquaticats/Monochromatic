# CLI argument validation

## Status

Accepted, 2026-05-18.
 Amended 2026-10-31:
 the Optique valueparser layer is deprecated across the workspace,
 and this decision now prescribes `node:util` `parseArgs`
 plus Valibot schemas directly.
 The 2026-05-18 Optique-era text and its adapter audit are preserved
 under `Historical: the Optique-era decision`.

## Context

When this decision first landed,
 Optique was the incumbent CLI parser
 and the question was only how far to take value-level validation.
 Since then every Optique import in the workspace carries
 `<!-- TODO: deprecate Optique -->`,
 new CLIs adopted `node:util` `parseArgs` instead
 (`package/crate-release/cli`,
 `package/open-code-review-issue/cli`,
 `package/mutation-test/cli-cargo-mutants`),
 the catalog strips `argparse` globally,
 and CAC was vetted and rejected for `cli-git`
 (`doc/audit/tech-cli-git-parser-migration-to-cac-vet-2026-08-14-a48b54e2.md`).
 The `search-fetch` CLI confirmed `parseArgs` as the parser for new work.

Valibot stays the workspace validation library
 (`PHILOSOPHY.tool-choices.md`,
 Standard Schema compatible),
 and the search-fetch tool specs are already Valibot schemas,
 so CLI inputs can validate against the exact same schemas the tools use.

## Decision

Parse with `node:util` `parseArgs`,
 strict mode for closed flag grammars,
 with a small hand-rolled dispatch where the grammar needs structure
 `parseArgs` does not provide
 (verb dispatch,
 preserving a `--` boundary,
 passthrough or exact-spelling contracts).

Decision rule for value validation:

- The value is "any string" or "any integer":
  no schema, accept the token as parsed.
- The value has a shape constraint
 (regex, range, picklist, URL, email, custom transform):
  validate with a Valibot pipe such as `v.pipe(v.string(), ...)`.
- A schema already exists for the input
 (the search-fetch tool specs are the example):
  reuse it rather than writing a second description of the same shape.
  A strict flag grammar feeding a tolerant tool schema is intended:
  the CLI rejects unknown flags,
  the tool layer still warns and strips unknown keys.
- The value needs a non-string output type (number, Date, parsed config):
  bridge with `v.transform()`; CLI arguments are always strings on entry.

Validation failures are usage errors
 (exit `2`, per `doc/decision/cli-output-and-exit-codes.md`),
 and the message lives in the thrown error (`PP6`),
 not in a preceding console line.

Async Valibot pipes (`v.pipeAsync`, `v.checkAsync`) run after argument
 parsing, never inside it;
 `parseArgs` itself is synchronous and stays that way.

Out of scope for this decision:

- Migrating the remaining Optique CLIs.
  Each migration is its own change.
- Reopening the parser choice for CLIs that hand-roll because their
  grammar is genuinely partial or passthrough
 (`package/terminal-exec/cli`,
 `package/mutation-test/cli`).

## Rejected alternatives

### Broad adoption: wrap every value in a Valibot pipe

Rejected.
 Parsers that accept "any string" (VM names, generic paths, freeform
 identifiers) gain nothing from `v.pipe(v.string())` beyond import
 churn. Reach for Valibot only where a shape constraint exists.

### Ad-hoc `map()` validation when shape checks are needed

Rejected.
 Hand-rolled validation duplicates what the workspace's chosen
 validation library does and forfeits Standard Schema interop,
 leaving two parallel validation idioms for the same constraint shapes.

### Use `zod` instead of Valibot for the same role

Rejected without separate vetting.
 The workspace chose Valibot at the catalog level;
 a second validation library for one subsystem splits contributor
 attention. It would need its own decision doc.

## Historical: the Optique-era decision

What follows is the 2026-05-18 record, kept for its research and
 reasoning. It describes a framework this workspace is deprecating;
 do not read it as current guidance.

The Optique-era decision rule was:
 use `@optique/valibot` selectively for shape-constrained value
 parsers, default to the plain `string()` / `integer()` primitives
 otherwise, and keep async validation out of the synchronous
 `ValueParser.parse()`.

The 2026-05-18 audit of `@optique/valibot`
 (via `gh api repos/dahlia/optique`, the npm registry, and npm
 download counts):
 MIT licensed,
 single maintainer (Hong Minhee, 3315 contributions against the second
 contributor's 2),
 1.1.0 at HEAD with 1.0.0 released 2026-04-14,
 3078 weekly downloads against 15832 for `@optique/core`,
 49267 bytes unpacked,
 peer dependency `valibot ^1.2.0`
 (the README's "0.42.0 and above" claim was stale),
 no open security advisories,
 zero open bug-labeled issues.
 Engines `node>=20.0.0`, `bun>=1.2.0`, `deno>=2.3.0`;
 dual ESM/CJS exports,
 with the Node ESM resolver picking the ESM entry.

## When to revisit

- If a non-CLI subsystem proposes a Standard Schema-compatible
  validation library other than Valibot,
  this decision renews alongside `PHILOSOPHY.tool-choices.md`.
- If async validation at parse time becomes a hard requirement
  (DNS resolution, remote registry lookups),
  it belongs in a post-parse step regardless of parser choice.
