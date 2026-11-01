# Handover: forbidden-strings runtime rule precompilation

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/rust-module/forbidden-regex` is now `package/forbidden-strings/core`.

## Status

Implementation and verification are complete.
Commit `37fd7136a` ships the hybrid runtime cache and bumps the scanner to `0.4.0`.
Its shell-constructed final commit paragraph contained literal `\\n` text,
so GitHub did not recognize the closing keyword there.
Follow-up commit `3b2ff2594` carries the proper closing line and GitHub closed issue #456 at
2026-08-25 17:59:54 UTC.

Clean-tree `cargo package` passed after 560 seconds.
The independent forbidden-strings workflow and Scorecard workflow passed on the release commit.
Crates.io published `forbidden-strings 0.4.0` with checksum
`99e6095140f9154a557470ad1fddee37bd4fe35982efcc16e8a0a0262014f2f6`
and trusted-publishing provenance bound to commit `37fd7136a`.
The cargo-publish workflow completed successfully.
GitHub release [`forbidden-strings-v0.4.0`][release] published at 2026-08-25 18:03:10 UTC with all seven target
archives.

[release]: https://github.com/Aquaticat/Monochromatic/releases/tag/forbidden-strings-v0.4.0

This handover is final.
The user explicitly said no `AGENTS.md` edit is needed for the communication correction.

## User goal

Resolve issue #456 completely:

- Make runtime rule precompilation a supported capability of the published scanner.
- Wire it automatically into this monorepo.
- Preserve text rules as the sole policy authority.
- Keep repeated commit scans within the scanner's documented pre-commit budget.
- Preserve redaction,
   rule identity,
   and fail-closed behavior.
- Commit the implementation with `Closes #456` in the commit body.

## Refresher

`forbidden-strings` is a Rust deny-list scanner under
`package/cli/forbidden-strings`.
It reads literal and restricted-regular-expression rules,
compiles them into a `forbidden_regex::RegexSet`,
and scans candidate files line by line.
Findings identify only path,
 line,
 and an opaque rule token.
Rule text,
 matching content,
 and surrounding source text stay out of findings.

Two rule sources participate in this repository:

- The built-in baseline lives in
  `package/cli/forbidden-strings/data/builtin-rules.txt`.
  `package/cli/forbidden-strings/build.rs` compiles and serializes it during the Rust build.
  Runtime loading uses `load_precompiled` and does not recompile it.
- Runtime appendices live in `forbidden-strings.append.txt` and
  `forbidden-strings.append.local.txt`.
  `generateForbiddenStringsRules` in `file-enforcer.config.ts` concatenates them into
  `.cache/forbidden-strings.rules.txt`.
  `FORBIDDEN_STRINGS_RULES` in generated `mise.toml` points the scanner at that text.
  `package/cli/forbidden-strings/src/frx_load.rs` compiles it on every invocation.

The local cli-git policy in `cli-git.config.ts` spawns the release scanner.
The canonical adapter is
`package/git-policy/forbidden-strings/src/scan-candidates.ts`.
It inherits scanner environment variables and parses scanner stderr strictly through
`package/git-policy/forbidden-strings/src/scanner-output.ts`.
Every nonempty unrecognized stderr line currently fails the policy as malformed output.

## Issue evidence

Issue #456 reports that runtime compilation dominates large-list scans.
Its corrected measurement used the same 10,206-rule file and the same two targets twice:

```text
run 1   139.81s
run 2   139.26s
```

The issue also reports a positive control:
a planted literal was detected while a neighboring clean file produced no finding.
The performance problem is repeated compilation,
not failure to enforce the rules.

These numbers come from the issue author.
They have not yet been reproduced in this work session.
Before making final performance claims,
measure an unchanged-build run-to-run band and use a positive control.

## Repository facts already verified

- `RegexSet::to_bytes` and `RegexSet::from_bytes` use raw bincode serialization in
  `package/rust-module/forbidden-regex/src/regex.rs`.
- Raw engine bytes carry no explicit artifact or engine format version.
- `RegexSet::from_bytes` validates decoded structures and rebuilds runtime prefilters.
- Runtime tail-format rules carry section names outside `RegexSet` in `CompiledRules.names`.
- A runtime artifact containing only raw engine bytes would lose named finding identities.
- File-enforcer's `overwrite` path compares content and skips unchanged files.
- File-enforcer writes changed text through a durable same-directory temporary file and atomic rename.
- Root preparation can run `node file-enforcer.config.ts` before Rust package builds.
  The scanner binary may therefore be absent during fresh setup.
- The release workflow builds Linux GNU,
   Linux musl,
   macOS,
   and Windows binaries.
- `.cache/` and `forbidden-strings.append.local.txt` are gitignored.
- At the time of inspection,
  `forbidden-strings.append.local.txt` and `.cache/forbidden-strings.rules.txt`
  both had Unix mode `0644`.
- The XDG Base Directory Specification 0.8 assigns nonessential user cache data to
  `$XDG_CACHE_HOME`,
   requires absolute XDG paths,
   and defaults Unix cache storage to
  `$HOME/.cache` when the variable is absent or empty.

## Settled decisions

### Product contract

The user selected all recommended options in grill round 1.

- **Q1,
   public scope**:
  add a supported scanner capability and wire it automatically in this monorepo.
  Q13 later established that external consumers receive read-write caching by default.
- **Q2,
   authority**:
  text is always authoritative.
  Compiled data is disposable and may be used only after proving it corresponds to current text.
- **Q3,
   cost placement**:
  eager compilation belongs at rule-generation time rather than on an ordinary commit scan.
- **Q4,
   confidentiality**:
  treat compiled artifacts as sensitive as their source.
  Binary encoding is not a confidentiality boundary.
- **Q5,
   completion evidence**:
  require semantic parity and measured large-list cached performance against the documented
  sub-100 ms pre-commit objective.

### Cache mechanics

- **Q6,
   cache location**:
  derive a per-user cache location rather than accepting a separate artifact path or using a
  repository-adjacent sibling convention.
- **Q7,
   artifact shape**:
  use one self-validating envelope containing a magic marker,
  schema and compatibility data,
  source-content SHA-256,
  rule identities,
  and serialized `RegexSet` bytes.
- **Q8,
   invalid artifact**:
  emit a redacted warning,
  compile authoritative text,
  and continue with a correct scan.
  An unusable cache must never produce a false-clean scan.
- **Q9,
   compilation interface**:
  add a dedicated subcommand:

  ```text
  forbidden-strings compile-rules --rules <PATH>
  ```

  The command derives the user-cache destination.
  It does not scan files or add the built-in baseline.
- **Q10,
   eager plus recovery**:
  file-enforcer invokes `compile-rules` eagerly when it generates this repository's runtime text.
  Scan mode also writes a repaired cache after compiling a missing or invalid artifact.
- **Q11,
   permissions**:
  enforce owner-only permissions for sensitive local representations where the platform supports them.
  The intended Unix modes are `0600` for
  `forbidden-strings.append.local.txt`,
  `.cache/forbidden-strings.rules.txt`,
  and the compiled artifact.
  The committed shared appendix remains `0644`.
- **Q12,
   concurrent writers**:
  use source recheck plus same-directory temporary-file creation,
  flush,
  and atomic replacement.
  Do not add a persistent artifact lock.
  Concurrent writers may duplicate compilation,
  but readers must see only complete self-validating artifacts.
- **Q13,
   cache activation**:
  read-write caching is the default and has no opt-out.
  Every runtime-rules scan checks the derived cache and attempts repair after a miss or rejection.
  `compile-rules` remains the eager explicit operation.
- **Q14,
   cross-platform cache root**:
  use an absolute `FORBIDDEN_STRINGS_CACHE_DIR` override when present.
  Otherwise use XDG cache resolution on Linux and other XDG-oriented Unix,
  `$HOME/Library/Caches` on macOS,
  and `%LOCALAPPDATA%` on Windows.
- **Q15,
   source slot identity**:
  key the artifact slot by SHA-256 of the exact current rules content.
  Identical content may share one artifact across repositories within the same version and platform partition.
  Every source edit selects a different slot.
- **Q16,
   warning protocol**:
  emit JSON cache-warning records on stderr alongside plain-text findings.
  Cli-git must parse the exact warning schema and continue rejecting unknown stderr records.
- **Q17,
   compatibility partition**:
  partition cache storage by exact scanner version and operating-system/architecture family.
  Repeat compatibility data inside the envelope before decoding raw engine bytes.
- **Q18,
   JSON warning schema**:
  use closed discriminated JSON objects with exact keys,
  schema version,
  a closed reason union,
  and a closed recovery union.
  Include no paths,
  digests,
  rule data,
  arbitrary operating-system errors,
  or extra properties.
- **Q19,
   retention**:
  perform no automatic cache deletion.
  Document that removing the application cache directory is always safe.
- **Q20,
   GitHub Actions cache root**:
  let one-shot secret-backed scans use the hosted runner account's native cache directory.
  Do not add a `$RUNNER_TEMP` override or a separate eager `compile-rules` step.
- **Q21,
   missing scanner during file-enforcer**:
  emit a non-sensitive notice and continue after generating authoritative text and applying permissions.
  A later scan repairs the cache through default read-write behavior.
- **Q22,
   compilation command output**:
  print nothing on successful `compile-rules` creation or reuse.
  Exit status 0 is the complete success signal.

## Current proposed behavior

### Eager generation

The monorepo integration is one-way:
file-enforcer calls the public scanner command,
while the published scanner has no dependency on file-enforcer.

```text
forbidden-strings.append.txt
            +
forbidden-strings.append.local.txt
            |
            v
file-enforcer writes authoritative runtime text
            |
            v
forbidden-strings compile-rules --rules <runtime-text>
            |
            v
versioned artifact under the per-user cache root
```

A missing scanner during fresh setup must not destroy or invalidate authoritative text.
File-enforcer emits a non-sensitive notice and continues.
The first later scan uses the default read-write recovery path.

### Scan behavior

```text
resolve authoritative runtime rules path
            |
            v
read and hash authoritative rules bytes
            |
            v
derive versioned platform partition and content-digest slot
            |
            +--> valid compatible artifact: load matcher and names
            |
            +--> cache miss or rejection:
                    emit closed redacted JSON warning
                    compile authoritative text in memory
                    scan correctly
                    attempt atomic cache repair
```

The scan must never trust modification time as source identity.
It must never run an artifact whose embedded source digest disagrees with current text.

### Compilation behavior

`compile-rules` should:

- Require an explicit `--rules` path.
- Read and hash one source snapshot.
- Derive the cache slot from the exact source-content digest.
- Reuse an already valid compatible artifact without recompiling.
- Compile through the same `compile_rules` path used by scanning.
- Preserve every optional rule name in order.
- Re-read and re-hash the source before publishing.
- Discard its result if the source changed during compilation.
- Write a complete artifact through a same-directory temporary file.
- Flush the temporary file before atomic replacement.
- Apply owner-only file permissions where supported.
- Keep all diagnostics free of rule text and matched content.
- Print nothing on successful creation or valid-artifact reuse.

## Settled grill round 3 and 4 consequences

The selected path shape is provisionally:

```text
<platform-cache-root>/forbidden-strings/
  <exact-scanner-version>/
    <operating-system>-<architecture>/
      <source-content-sha256>/
        rules.bin
```

The exact component spelling remains open.
The envelope repeats the scanner version,
platform family,
source digest,
rule identities,
and artifact schema before its engine bytes.

Default read-write behavior intentionally changes every runtime-rules scan from read-only to potentially state-mutating.
There is no cache-mode flag or environment opt-out to design.
An unavailable cache root or failed artifact write must therefore degrade to a correct in-memory text compilation.
The scanner emits a closed JSON warning and continues with compiled in-memory rules.

Content-addressed slots make old artifacts unreachable after every rules edit.
They also permit identical rules content to share one compiled artifact across source paths.
No automatic cleanup removes old content,
scanner-version,
or platform partitions.

JSON warning records become part of the scanner-to-cli-git protocol.
The selected object is one compact JSON value per line with exact keys and no extras.
The provisional shape is:

```json
{"type":"forbidden-strings/cache-warning","schemaVersion":1,"reason":"missing","recovery":"compile-from-text"}
```

The closed reason candidates are `missing`,
`cache-root-unavailable`,
`unreadable`,
`source-mismatch`,
`incompatible`,
`invalid`,
and `write-failed`.
The closed recovery candidates are `compile-from-text` and `continue-with-compiled-rules`.
The exact names remain open until the artifact failure taxonomy is finalized.
Cli-git parses only a complete object matching the schema.
A non-JSON line continues through the finding parser;
unknown or malformed complete JSON fails closed.

An expected first cache miss emits the selected JSON warning because Q8 explicitly included missing artifacts.
A successful `compile-rules` invocation emits no stdout status.

GitHub Actions keeps its native hosted-runner cache resolution.
The current one-shot workflow performs no eager compilation step and gains no same-job cache reuse.
The derived artifact receives owner-only permissions and follows the hosted runner account's lifecycle.

## Delegated implementation details

After Q22,
the user said the remaining frontier consisted of implementation questions and ended grilling.
The implementer owns these choices and should use the strongest design consistent with the settled contract.
The current implementation defaults are:

- Use an explicit fixed-width scanner-owned envelope around existing raw engine bytes.
- Apply hostile-length ceilings before allocation,
  reject overflow and trailing bytes,
  and confirm rule-name count against decoded engine rule count.
- Use the detailed closed warning reasons:
  `missing`,
  `cache-root-unavailable`,
  `unreadable`,
  `source-mismatch`,
  `incompatible`,
  `invalid`,
  and `write-failed`.
- Use hierarchical lowercase cache components:
  `forbidden-strings/v<scanner-version>/<os>-<architecture>/<content-sha256>/rules.bin`.
- Bump the public scanner to `0.4.0` because default filesystem mutation and stderr protocol are material
  pre-1.0 behavior changes.
- File-enforcer tolerates only an absent scanner executable.
  Once `compile-rules` starts,
  any nonzero or interrupted result fails generation.
- Keep the built-in baseline on its existing embedded raw-byte and name-sidecar path.
- Document manual deletion of the application cache directory;
  add no automatic cleanup or cache-clear command for issue #456.

These are implementation defaults,
not additional preference gates.
Change one only when code or measured evidence proves it cannot satisfy the settled product contract.

## Implementation progress

Implementation started after the user ended grilling.
Current commits:

- `38f01052f` `feat(forbidden-strings): cache compiled runtime rules`
  adds content hashing,
  cross-platform cache-root resolution,
  scanner-owned envelope encoding,
  private atomic publication,
  default scan-time repair,
  `compile-rules`,
  integration tests,
  and a cache-envelope fuzz target.
- `9e16a2e7b` `feat(forbidden-strings): parse cache warning records`
  adds strict JSON cache-warning parsing to the canonical git-policy package,
  mirrors it into cli-git,
  and rejects unknown keys,
  reasons,
  recoveries,
  and schema versions.
- `4bee062e2` `feat(file-enforcer): precompile forbidden-string rules`
  makes file-enforcer apply owner-only modes,
  invoke the release scanner eagerly,
  and tolerate only an absent executable.
- Follow-up fixes through `e242ca0c7` satisfy warning-protocol TypeScript lint,
  Rust documentation lint,
  Clippy,
  and built-artifact test imports.
- `2a1493ed2` replaces open-ended JSON parsing with a byte-exact allow-list of valid compact warning records.
- `fcec4611d` and `58b70f7d5` exercise mixed real-scanner cache-warning and finding output through git-policy.
- `be7117fca` adds scanner integration coverage for invalid rules,
  corrupt artifacts,
  content changes,
  write failures,
  and unavailable native cache roots.
- `ca331e7a5` rejects noncanonical inner engine bytes and removes invalid warning-pair construction.
- `8720834a9` preserves missing-versus-unreadable read races and separates publication errors from envelope errors.
- `2203dc6a9` verifies that cli-git accepts a clean exit carrying a first-miss cache warning.
- `b800bd9f5` applies mode `0600` to CI's secret-bearing runtime text.

The first implementation serialized one `RegexSet` engine per rule.
Actual-fixture evidence proved that shape insufficient:
146.320 seconds to compile,
a 162,668,054-byte artifact,
and 1,428.5 ms warm median under the bounded 2-GiB/2-CPU container.
The positive and clean controls passed,
but the sub-100 ms objective failed by more than an order of magnitude.
`3f197a33f` records the rejected result in `PERF.md`.

The implementation is pivoting to one hybrid runtime matcher:

- Bare literals retain exact bytes and build one Aho-Corasick automaton.
- Explicit and multiline regex rules alone enter a precompiled `RegexSet`.
- Both subset-local ids map back to original runtime ids before finding attribution.
- Cache schema 2 stores compact literal groups,
  regex-id mappings,
  and optional regex-only engine bytes.

The cache implementation uses focused modules under
`package/cli/forbidden-strings/src/runtime_cache/`:

- `path.rs` owns exact-source SHA-256 and platform cache locations.
- `envelope.rs` owns fixed-width framing and validation.
- `publish.rs` owns complete reads,
  private modes,
  flush,
  and atomic replacement.
- `warning.rs` owns the closed JSON protocol.
- `mod.rs` owns load,
  fallback compilation,
  scan-time repair,
  and eager command orchestration.
- `runtime_matcher.rs` owns exact-literal de-duplication,
  Aho-Corasick construction,
  short-literal boundaries,
  regex subset mapping,
  and globally ordered batch matches.

Verification is in progress.

- Scanner package `cargo check` passed after 503 seconds,
  including its build-time baseline compile.
- Scanner Rust linter passed after documenting the digest tuple field and Unix permission imports.
- Scanner Clippy passed after restricting test-only warning accessors and renaming the configured command factory.
- The `0.4.0` scanner test run passed all 154 tests in 21.189 seconds after compilation.
  A post-review rerun is pending after the final read-race and canonical-envelope fixes.
- Git-policy build,
  `lint:types`,
  Oxlint with zero findings,
  and unit tests pass.
- Real git-policy integration tests parse a cache-miss JSON warning with a runtime finding and accept the warning on a
  clean scanner exit.
- A first attempt to scope root Oxlint by appending `file-enforcer.config.ts` to `mise run lint:oxlint` failed in mise's
  inline task parser;
  it did not execute Oxlint and is not a source-code verdict.
- README and `PERF.md` now document runtime caching and pending performance acceptance.
- The `0.4.0` version bump and CLI help update are in the working tree.
  Package `cargo check` is regenerating the lock and rebuilding with that version.

The first `lint:fuzzing` run exposed a feature-gated missing import in `frx_load.rs` after 509 seconds of baseline
compilation.
`8720834a9` restores the conditional import;
the rerun passed.
Post-review package check and Rust linter also pass.
Final Clippy is currently rebuilding the baseline.

The hybrid source and tests are in the working tree but not committed because the same final change carries the
`0.4.0` release-triggering manifest update.

The hybrid actual-fixture result under identical bounds is accepted:

```text
first compile + publish      108 ms
artifact size          1,232,050 bytes
warm minimum                  85 ms
warm median                   88.0 ms
warm p95                      92 ms
warm maximum                  93 ms
samples                       30
```

The planted rule matched,
the clean control exited 0,
and the artifact had mode `0600`.
Compared with the rejected full-engine artifact,
the hybrid artifact is 99.24% smaller,
first compilation is 1,354.8 times faster,
and warm median is 16.2 times faster.
`ba0ed3816` records the accepted evidence in `PERF.md`.

Final verification now passes:

- Release build and `forbidden-strings 0.4.0` CLI help/version checks.
- Package `cargo check` and feature-gated `cargo check`.
- Clippy with warnings denied.
- Rust linter with max-lines and required-rustdoc enforcement.
- All 162 scanner tests in 19.952 seconds after compilation.
- Cache-envelope fuzz target build plus all-target 128-second smoke campaign.
- Forbidden-regex bench package check and dependent lock refresh.
- File-enforcer eager hybrid artifact generation with source/text/artifact modes `0600` and cache directories `0700`.
- Git-policy build,
  type lint,
  zero-finding package Oxlint,
  unit tests,
  and real scanner warning integration.
- Bundled cli-git build,
  type lint,
  and unit tests.

Repo-wide Oxlint still exits nonzero on its measured baseline of 1,385 errors and 3,910 warnings;
its second run reports no diagnostic at the new file-enforcer cache helper lines.
Repo-wide dprint still lists existing unrelated files,
but no longer lists `package/cli/forbidden-strings/Cargo.toml`.
`cargo package` correctly refused the dirty working tree and must rerun immediately after the final commit.

Only the release-triggering closing commit,
clean-tree `cargo package`,
and GitHub closure/release observation remain.

## Candidate implementation scope

The final design may still adjust this list.

Scanner changes:

- `package/cli/forbidden-strings/src/cli.rs`
- `package/cli/forbidden-strings/src/lib.rs`
- `package/cli/forbidden-strings/src/frx_load.rs`
- New focused Rust modules for cache-root resolution,
  artifact encoding,
  and atomic publication.
- `package/cli/forbidden-strings/tests/integration.rs`
- Focused Rust unit-test sidecars.
- `package/cli/forbidden-strings/README.md`
- `package/cli/forbidden-strings/PERF.md`
- Possibly `package/cli/forbidden-strings/Cargo.toml`,
  through the repository's Cargo-manifest enforcement source if dependencies change.

Monorepo integration changes:

- `file-enforcer.config.ts`
- Generated `mise.toml`
- `package/git-policy/forbidden-strings/src/scanner-output.ts`
- Its generated mirror under
  `package/git-policy/cli/src/optional/forbidden-strings/scanner-output.ts`
- Canonical and integration tests for warning parsing.

## Required verification

### Artifact correctness

Cover every path separately:

- Named tail-format rules round-trip with names unchanged.
- Unnamed legacy rules round-trip with numeric identity unchanged.
- Valid artifact produces findings identical to text compilation.
- Source-content change selects a new content-addressed slot and cannot reuse the old artifact.
- Scanner-version or platform mismatch rejects old artifact.
- Bad magic,
  truncated fields,
  oversized lengths,
  count mismatch,
  malformed names,
  and invalid engine bytes all fall back safely.
- No cache diagnostic contains seeded sensitive rule text.
- A source change during compilation prevents stale publication.
- Concurrent writers publish only complete artifacts.

### Integration behavior

- `compile-rules` creates or reuses the derived artifact.
- Scan-time recovery creates a valid artifact when authorized.
- A cache-write failure still runs a correct text-compiled scan.
- Cli-git accepts only exact-key schema-version-1 cache-warning JSON and still maps plain-text findings.
- Cli-git rejects extra JSON properties and every unknown type,
  reason,
  recovery,
  or schema version.
- Cli-git rejects unknown scanner stderr.
- File-enforcer generates text before invoking compilation.
- Fresh setup emits a non-sensitive notice and remains usable when the scanner binary is absent.
- `compile-rules` success keeps stdout empty for both creation and reuse.
- GitHub Actions uses native runner cache resolution for its one-shot secret-backed scan.
- Unix permission checks prove owner-only sensitive files.
- Linux,
  macOS,
  and Windows cache-root resolution use injected disposable environments.

### Performance evidence

Use a disposable fixture and release binary.
Do not mutate real rules or cache state for verification.

- Establish run-to-run timing variation on one unchanged build.
- Run a positive-control rules change that must invalidate the cache.
- Measure first compile,
  valid cached load,
  and repeated cached scans separately.
- Use a rule count representative of issue #456 rather than only the current repository appendix.
- Verify matching output while timing;
  a fast null result without a known match is not sufficient.
- Exercise the final CLI boundary,
  not only library functions.

Run package tasks through `mise run`.
Read root and package `mise.toml` before selecting the exact task set.
Rust changes require package build,
test,
Clippy,
Rust linter,
and end-user CLI execution.
TypeScript changes require the package `lint:types` task in addition to tests and lint.

## Risks to keep visible

- A serialized matcher can retain sensitive structure.
  Keep cache data private and disposable.
- Cache correctness is security correctness.
  A stale artifact must cause text compilation,
  never a clean result.
- Warning output is part of the cli-git protocol.
  Broadly ignoring stderr would weaken fail-closed behavior.
- Raw bincode compatibility is not a public stable format.
  The envelope and cache partition must reject unsupported producers before decoding.
- Scan-time repair introduces default state mutation with no opt-out.
  Write failures must preserve a correct scan and emit the closed JSON diagnostic contract.
- Content-addressed slots retain one artifact per observed rules content and scanner/platform partition.
  No automatic retention bound exists.
- A content digest is a stable fingerprint of sensitive source.
  Treat cache paths and artifacts as sensitive even when rule text is absent.
- File-enforcer is only this monorepo's eager integration point.
  Do not make the published scanner depend on it.
- The repository supports Windows and macOS releases.
  Unix-only cache and permission assumptions are incomplete.

## Next action

No open action remains for issue #456.
