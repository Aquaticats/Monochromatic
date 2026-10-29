# Forbidden-strings migration to forbidden-regex

Date:
 2026-07-16.

Status:
 shared understanding confirmed by the user on 2026-07-16;
 not yet enacted;
 implementation starts on explicit request.

Migrate `package/forbidden-strings/cli` off resharp and the `regex` crate
onto the in-house engine at `package/forbidden-strings/core`.

## Motivation

- `doc/todo/forbidden-strings.1.0.md` item 24 flags `resharp`,
  `gix-hash`,
  and `gix-index` as pre-1.0 `0.x` dependency exposure;
  the engine swap removes the largest of the three.
- `package/forbidden-strings/core/README.md` states the engine
  "exists to power the `forbidden-strings` secret scanner";
  its `RegexSet` internally owns literal gating and prefiltering,
  duplicating what `scan.rs` builds today with aho-corasick shards.
- `package/forbidden-strings/core.bench` measures the engine ahead of the
  `regex` crate on the scanner's common case
  (mostly-non-matching,
   code-like lines).
- Open resharp issues (#158 upstream tracking,
   #240 pre-validator pinning)
  stop being liabilities when the dependency is gone.

## Settled decisions (user-confirmed)

### Engine scope: full replacement

`forbidden_regex::RegexSet` replaces resharp,
 the `regex` crate,
 and the aho-corasick gate.
Scanning becomes line-at-a-time.
Rejected alternatives:
 replacing resharp only (keeps two engines and two dialects permanently),
 and a dual-run transition period
 (a one-shot differential validation at cutover captures the safety value
 without carrying two stacks in the shipped binary).

### Rule porting: one-time port, strict loader

`data/builtin-rules.txt` and the append files are rewritten once into the
restricted dialect,
 scripted from the porter in
`package/forbidden-strings/core.bench/src/{port,normalize}.rs`
with a human-reviewed diff.
After migration the loader hard-errors on any rule outside the dialect
(fail-closed).
Rejected:
 runtime auto-normalization
 (silently weakens rules on every load,
 the worst failure mode for a security gate),
 and a shipped porting subcommand
 (permanent CLI surface for a one-time operation).

### File format: two-form format stays

Bare line = literal;
 `/PATTERN/` line = regex in the engine dialect.
The bench's `load_rules` already consumes exactly this format,
 which settled the question.
The loader escapes literal lines into the always-verbose dialect
(spaces,
`#`,
 and metacharacters).
Flags slot:
 `m` and `x` are accepted no-ops
 (multiline and verbose are always on in the engine);
 any other flag letter is a hard load error,
 because silently dropping `i` or `s` would change match semantics.
The 2 committed `/m` rules in `forbidden-strings.append.txt` port by dropping the flag.

### Output contract: columns dropped

Findings become `PATH:LINE rule=N`.
The engine reports per-line rule indices,
 not spans;
 the user confirmed losing column precision is expected.
One finding per (line,
 rule) pair.
Streams stay as today.
`package/git-policy/cli/src/optional/forbidden-strings/scanner-output.ts`
and `errors.ts` update in lockstep
 (they strictly parse the current column form).

### Rule 172: drop the curl anchor

The only cross-line builtin rule
(curl basic-auth matching up to 5 continuation lines)
is ported to flag the `(?:-u|--user)` plus `user:pass` credential shape
on any single line without requiring `curl` on that line.
The credential pair is the payload;
 `curl` is context,
 matching the porter's existing strip-context philosophy.
Rejected:
 same-line-only (silently loses the continuation-line case the rule existed for)
 and multi-line windowing (breaks the line model for one rule).

### Publishing: forbidden-regex goes to crates.io

Publish `forbidden-regex` 0.1.0
 (name verified unclaimed on the crates.io sparse index,
 2026-07-16),
extend `.github/workflows/cargo-publish.yml` to cover it,
and give `forbidden-strings` a path-plus-version dependency.
Publish ordering:
 engine before scanner.
Rejected:
 unpublishing the scanner (abandons the 1.0 roadmap)
 and vendoring the engine source (diverging copies of a living codebase).

Publish-readiness audit (measured 2026-07-16):

- Tests pass:
  221 run,
   221 passed,
   1 skipped
  (`mise run //package/forbidden-strings/core:test`).
- `lint:rust` (max-lines,
   require-rustdoc) passes.
- Clippy fails:
  367 errors,
  every one the crate's own `implicit_return = "deny"` gate
  (missing `return` statement);
  a mechanical sweep is a publish prerequisite (PKG).
- `cargo package --list` bundles README,
  `Cargo.lock`,
  and both `LICENSES/*.txt` files correctly.
- Metadata gaps:
  no `authors` or `documentation` fields;
  fill at publish time
  (same nit as 1.0 checklist item 11 for the scanner).

Verdict:
 not publish-ready today;
 the gap is the clippy sweep plus metadata,
 both bounded and mechanical.

### Scan path uses the engine's batch face

The scanner scans a file by handing every line to the set at once
instead of looping `is_match` per line.
`RegexSet::is_match_batch` is the plain per-line map;
the real throughput shape is the `#[doc(hidden)]` benchmark hook
`is_match_batch_concat`:
 one full-width SIMD prefilter sweep over a concatenated buffer,
 then per-line resolution
 (short lines starve Teddy at per-line width).
The scanner already owns the whole file buffer with newlines in place,
so the migration adds a public scanner-facing engine API
that productionizes the concat path without the copy:
 take the caller's buffer plus line-start offsets
 (the internal `sweep_candidates(&buf, &starts)` primitive),
 and return per-line rule indices
 (the batch hooks return only booleans today;
 findings need `rule=N` attribution).
This is an engine API work item that lands before the scanner rewrite.

## Adopted defaults (stated for veto, not asked)

- Version:
  `forbidden-strings` 0.1.9 becomes 0.2.0
  (breaking rule dialect and output format).
- Line mechanics:
  split on `\n`,
  strip one trailing `\r`,
  skip empty lines (the engine requires non-empty input);
  the 8 KiB binary tail cap and walker behavior are unchanged.
- Hardening kept:
  the scanner's `catch_unwind` fail-closed boundary,
  `panic = "unwind"`,
  and `overflow-checks = true` stay;
  the engine's own `Cargo.toml` documents that it expects the caller's
  `catch_unwind` boundary.
- Teardown:
  the resharp pre-validators
  (`src/rule/compile.rs`,
  `src/rule/nesting.rs`,
   shape checks)
  and engine-routing (`requires_resharp`) are deleted;
  the engine rejects bad shapes via `Result` at compile time.
- Redaction:
  the migration includes the sentinel redaction regression test
  (`doc/todo/forbidden-strings.1.0.md` item 4)
  over the new load path,
  closing #217;
  the engine's `CompileError` messages are static strings,
  with only single-metacharacter and repetition-count interpolations
  (audited 2026-07-16),
  which the sentinel test still covers.
- Fuzz:
  scanner-level targets in `package/forbidden-strings/cli.fuzz` are retargeted;
  gate/shard/dispatch targets die with the machinery they fuzz;
  the literal-roundtrip target grows adversarial verbose-mode escaping cases
  (literal-to-dialect escaping is a syntax-boundary transformer);
  engine-level fuzzing stays in `package/forbidden-strings/core.fuzz`.
- Cutover validation:
  a one-shot differential run of the 0.1.9 binary against the new binary
  over the repo corpus with the real rule files;
  every finding delta must be explained by a reviewed port semantic change.
- Performance:
  re-measure and update `PERF.md`;
  acceptance is staying within the README's pre-commit and pre-push budgets.
- Out-of-band rule copies:
  the cutover commit is atomic for committed files;
  a runbook covers porting the `FORBIDDEN_STRINGS_LIST` CI secret
  and each contributor's gitignored local file at merge time.
- Docs:
  scanner README rewritten to document the new dialect and engine;
  resharp troubleshooting and audit docs are kept
  (durable record of upstream bugs found and filed);
  `doc/planning/forbidden-strings-em-dash.md` gets annotated with dialect
  implications (unbounded `.*` complements must become bounded forms).
- Port tooling:
  the one-time port script lives as a temporary bin in the bench sidecar
  and is removed after migration.

## Open implementation questions (measure during implementation)

- RESOLVED (2026-07-16,
   measured during the port):
  startup compilation is not viable
  (worst faithful rule took 123 seconds to compile;
  49 rules exceed one second even after the leading-strip fix),
  so the loader embeds a precompiled serialized `RegexSet`
  (`to_bytes`/`from_bytes`) for the builtin ruleset at build time;
  the small local appendix still compiles at runtime.
- Whether any ported rule trips the engine's `EmptyMatchable` or
  state-cap rejections:
  surfaced mechanically by the strict compile during the port review.

## Measured facts (2026-07-16)

- `data/builtin-rules.txt`:
  861 lines;
  26 contain unbounded quantifiers;
  rule 172 is the only cross-line rule.
- `forbidden-strings.append.txt` (committed):
  18 lines;
  2 regex rules,
  both flagged `/m` only.
- `forbidden-strings.append.local.txt` (gitignored,
   feature counts only):
  54 lines;
  2 regex-form rules;
  no unbounded quantifiers;
  no flags.
- CORRECTION (2026-07-16,
   found during the port):
  the original claim here,
   that case-insensitivity was a non-issue,
  measured only trailing `/i` flags and missed pervasive inline `(?i)`
  groups:
   172 of 259 builtin regex rules carry them.
  The user's decision (corrected 2026-07-16,
   supersedes a brief
  blanket-strip reading):
  keyword literals under `(?i)` scope expand to a three-casing
  alternation (lowercase,
   Capitalized,
   UPPERCASE),
  classes under `(?i)` widen to both cases,
  mixed-case shapes like `AdOBe_` deliberately unmatched;
  the #387 differential is the empirical check.
  Companion standing preference:
  over-matching lossiness is wanted
  (a false positive inside a base64 blob is acceptable),
  which licenses aggressive context simplification in the port.
- `RegexSet::from_ruleset` is a delimiter-split convenience over `RegexSet::new`;
  the scanner's loader keeps owning the file format.
- The eleven load-path leak sites of #217 are
  `tracing::warn!(rule = ?src, ...)` calls in the resharp pre-validators
  in `src/rule/compile.rs`,
  all deleted by this migration.
- `cargo-publish.yml` currently publishes only `forbidden-strings`.
- The engine itself depends on `aho-corasick`,
  `memchr`,
  and `regex-automata` internally for its prefilters,
  so the scanner's direct aho-corasick dependency disappears
  but the machinery stays as a transitive dependency.

## Issue impacts

- #217 (redact load-path diagnostics):
  closed by the loader rewrite plus sentinel test.
- #226 (resharp fail-closed shape tests):
  closed as mooted;
  resharp is removed.
- #158 (track upstream resharp fix) and #240 (pre-validator pinning):
  closed as mooted by the engine swap.
- #224 (rule-loading foot-guns):
  unknown flags now hard-error instead of silently degrading to literals;
  bare `//` is rejected by the engine's `EmptyMatchable`;
  the BOM strip folds into the loader rewrite;
  remaining items (cwd-relative resolution,
  `--all` plus positionals) are
  unaffected and stay open.
- `doc/todo/forbidden-strings.1.0.md` item 24 reshapes:
  resharp exposure is replaced by own-engine maturity
  (fuzz coverage in `forbidden-regex.fuzz`,
   its own 0.x version discipline).

## Issue breakdown

Published 2026-07-16 as sixteen tracker issues,
sliced small enough for one bounded subagent context each
(the engine API and the scanner rewrite are staged into
independently compilable,
 independently verifiable sub-slices):

- #375 engine clippy sweep plus crate metadata (no blockers)
- #376 rule-file port,
   staged unwired (no blockers)
- #377 batch API contract plus reference implementation (after #375)
- #378 batch API single-sweep fast path (after #377)
- #379 batch API differential fuzz target (after #377)
- #380 batch API bench coverage and numbers (after #378)
- #381 seedless and line-start batch routing,
   bench-gated (after #380)
- #382 publish forbidden-regex 0.1.0 plus workflow lane (after #375,
   #377,
   #378)
- #383 scanner rule-compiler module,
   strict loader,
   redaction tests (after #377,
   #376)
- #384 scanner line-based scan path,
   columnless output (after #383)
- #385 scanner teardown,
   README,
   version 0.2.0 (after #384)
- #386 scanner fuzz retargeting (after #385)
- #387 differential cutover validation plus perf re-measure (after #376,
   #385,
   #380)
- #388 git-policy columnless output parser (after #384)
- #389 cutover,
   runbook,
   scanner publish;
   ready-for-human (after #382,
   #385,
   #386,
   #387,
   #388)
- #390 post-migration hygiene (after #389)

## Rollout sequence

 1. Publish-readiness sweep on `forbidden-regex`:
    fix the 367 clippy implicit-return errors,
    fill `authors` and `documentation` metadata.
 2. Add the public buffer-plus-line-starts batch API
    returning per-line rule indices.
 3. Publish `forbidden-regex` 0.1.0 (workflow extension first).
 4. Port rule files with the bench-sidecar script;
    review the semantic diff (quantifier bounds,
     rule 172,
    `/m` drops).
 5. Rewrite loader and scan path against the batch API;
    delete resharp machinery;
    add the sentinel redaction test;
    retarget scanner fuzz targets.
 6. Differential validation run,
     old binary against new.
 7. Re-measure `PERF.md`.
 8. Lockstep update of the git-policy scanner-output parser.
 9. Atomic cutover commit;
    execute the runbook for the CI secret and contributor local files.
10. Close and annotate issues;
    update the 1.0 checklist and the em-dash planning doc.
