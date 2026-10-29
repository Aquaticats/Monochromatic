Status: in progress, halted 2026-10-29 for handoff

Superseded 2026-10-29 before a verdict:
the user approved `toml_edit` without a completed vet
("I'm approving toml_edit, browserslist-rs, schemars as-is."),
recorded in `doc/decision/file-enforcer-rust-rewrite.md`, "Consequences".
This halted draft is kept whole as knowledge ("Even if work is superseded, it should still become knowledge.", user):
its discovery, screening and candidate ledger remain evidence for any later revisit of the TOML editor.

# Vet the TOML editor for the Rust file-enforcer

The coordinator stopped this vet during discovery and screening, before any hard-gate confirmation, execution, scoring
or recommendation.
No recommendation is stated.
The owner will continue it later with another model; the "Steps remaining" section lists every skill step still open in
the skill's order.
Nothing was committed, per the coordinator's instruction, so the skill's "commit after each phase" rule is unmet.

## Status and fingerprint

- Status:
   in progress, halted 2026-10-29 for handoff.
- Lifecycle phase:
   between Discovered and Screened; no candidate is Hard-gate confirmed yet.
- Subject:
   file-enforcer TOML editor.
- Decision scope:
   choose the Rust library the Rust file-enforcer uses for format-preserving edits of per-crate `Cargo.toml` files
   and `rust-catalog.toml`, with parse and deserialize errors that carry source spans.
- Started and last updated:
   2026-10-29.
- Active audit owner:
   none after the halt.
   The session that wrote this report (id `73f0639e-153c-4b58-bb17-6162a418423e`) released its lock at the halt.
   The next owner acquires a new lock under `~/temp/agent/technology-vet-locks/`.
- Governing skill:
   `.agents/skills/choosing-technology/SKILL.md` at commit `37117e36397e30233062828581a5e38217da9458`
   (`git log --format=%H -1 -- .agents/skills/choosing-technology/SKILL.md`),
   SHA-256 `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.
   The untracked mirror `.claude/skills/choosing-technology/SKILL.md` has the same SHA-256.
- Compatibility fingerprint:
   `63568ccf3be5534007d79ec7edc3bfcc060e38f159df7a50da00759d0d9f219c`
   (schema version 1, RFC 8785 canonical JSON of 1516 UTF-8 bytes, SHA-256).
   The script is `/var/home/user/temp/agent/toml-vet-2026-10-29/fingerprint.ts`;
   it sorts keys recursively and has no numbers beyond the schema version, so `JSON.stringify` over sorted keys is the
   canonical form.
   The canonical input is stored in `/var/home/user/temp/agent/toml-vet-2026-10-29/fingerprint.out`.
   The hard-constraint list below is the fingerprint input; changing it changes the fingerprint.
- Prior compatible report:
   none.
   `rg --files doc/audit | rg -i 'toml-editor|toml-edit'` printed nothing before this report.
   Related but different subjects: `doc/audit/tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24.md`
   (JSONC) and the non-skill research in `doc/planning/monorepo-manager-route-research/rust-structured-edits.md`.

## Context

- Trigger:
   `doc/decision/file-enforcer-rust-rewrite.md` (accepted 2026-10-29), section "Consequences", lists
   "choosing-technology vets for `toml_edit`, `browserslist-rs`, `schemars`, and an XML editor" as a gate before
   implementation.
   This report covers the TOML editor only.
- Consumers (decision, section "Outputs file-enforcer writes"):
   per-crate `Cargo.toml` keys (edition, license, publish, lints, repository, homepage, readme, profiles,
   Dylint check-cfg) and `rust-catalog.toml` plus the generated `[workspace.dependencies]` tables
   (`doc/decision/rust-dependency-catalog.md`, section "Crate manifests": use sites say `workspace = true`).
- Required behavior from the request: preserve comments, key order and unowned keys; set keys only where a table
   exists; append a block if absent; deserialize errors with spans.
- Settled by the user and the design (recorded as inferences, no question needed):
  - Wrapper code around the chosen editor is accepted
     (`doc/planning/monorepo-manager-from-scratch-design.md`, section "Managed file editing", "Chosen editors").
  - Formatting outside edited nodes may change and line endings may be normalized
     ("'untouched bytes identical' rule and preserve CRLF per file - no need.", user, 2026-09-17).
  - Comments directly above a deleted node leave with it; JSONC and TOML files may be reformatted canonically on
     write, so files the editor cannot round-trip byte for byte are rewritten, not rejected.
  - The rewrite does not need to preserve the TypeScript tool ("Consider the original file-enforcer bad in every way",
     user, 2026-10-29).
- Prior evidence, none of it reproducible now (its scratchpad is gone; `ls` of the recorded `SE/` path failed on
   2026-10-29), so it can seed hypotheses but cannot satisfy an evidence record:
  - `toml_edit` 0.25.15 behind a repository wrapper passed 21 of 21 probe cases, as-is 12 of 21, `taplo` 0.14.0 6 of 21,
     the TypeScript `module-toml-edit` 15 of 21
     (`doc/planning/monorepo-manager-from-scratch-design.md`, "Chosen editors";
     `doc/planning/monorepo-manager-route-research/rust-structured-edits.md`, "Probe results by candidate", "TOML").
  - Quirk documentation: `doc/troubleshooting/toml-edit-comment-loss.md` (index assignment, `Array::remove`,
     `Table::remove` and `Table::insert` drop or move comments;
     "behavior unchanged at `toml-rs/toml` HEAD `3b81b06`" for 0.25.15) and
     `doc/troubleshooting/taplo-toml-1-1-inline-table-trailing-comma.md`.
  - `doc/research/file-enforcer-rust-rewrite-crates.md` section 5 names `toml_edit` "clearly" and cites
     `crates/toml_edit/src/de/error.rs:28` for `span() -> Option<Range<usize>>`.
    This session read the section but not the cited line.
- Measured repository facts (this session):
  - 26 tracked `Cargo.toml` files (`git ls-files 'package/**/Cargo.toml' | wc -l`), plus `rust-catalog.toml` at the root.
    These are the consumer-boundary corpus for validation.
  - `rust-catalog.toml` already lists `toml = ">=1.1"` in `[catalog]`.
  - `toml_edit` is already in two lockfiles as a transitive dependency:
     `package/ide/app/Cargo.lock` (line 5378) and `package/nested-wayland-session/cli/Cargo.lock` (line 2070).
    `cargo tree --invert` was not run to name what pulls it.
  - `rust-catalog.toml` supply-chain policy: `minimumReleaseAge = 1440`, `minimumReleaseAgeStrict = true`,
    `trustPolicy = "no-downgrade"`.
    A candidate version must satisfy these to enter the catalog.
  - The incumbent `@monochromatic-dev/toml-edit-core` is at `package/toml-edit/core/package.json` version 0.0.1,
    built on `toml-eslint-parser`; fixtures at `package/test-fixture/toml-edit/src/` (91 valid and 108 invalid
    `toml-test` fixtures per the research).

## Base components and overlays

- Base category:
   inspectable open-source local technology (a Rust library crate, source on a public host).
- Overlays:
  - Incumbent dependency replacement (the TypeScript `toml-edit-core` is the incumbent; a replacement parity inventory
     is required).
  - Multi-platform claim (Linux x64, macOS arm64, Windows x64).
  - Native, Wasm, prebuilt or generated-code boundary: applies as source-to-crate mapping only; the candidates are
     pure-Rust crates built from source, so the prebuilt-binary clauses are not applicable unless a candidate ships
     generated code or a build script (not yet checked per candidate).
- Not applicable:
   SaaS gates (no hosted component), high-trust execution overlay (no agent, hook or credential boundary; the tool
   parses repository files), sensitive-data overlay, browser baseline.
   Human-auditability measurements are applicable only as a scored-criterion input because the editor runs inside
   file-enforcer, which writes repository files; no credential or network boundary.

## Hard constraints (the fingerprint input)

- Edits preserve comments, key order and unowned keys.
- Keys are set only where the table already exists; an absent block is appended.
- Parse errors expose byte spans or line and column;
   typed deserialization errors with spans are reachable from the candidate or from an allowed companion crate
   (for example `toml` or `serde_spanned`, which the catalog already allows).
- Accepts the TOML 1.1 forms already present in repository manifests
   (for example inline-table trailing commas, which Cargo 1.98 accepted in the September research).
- Rust library embeddable in a Rust binary; no Node runtime.
- Inspectable source, licence compatible with an LGPL-3.0-or-later consumer.
- Resolvable through `rust-catalog.toml` as a registry crate or a `git` plus `rev` entry.
- Builds on Linux x64, macOS arm64 and Windows x64.

Frozen criteria and weights: not frozen.
A draft for the next agent is under "Steps remaining", step 6.

## Query schedule and discovery ledger

Provider notes:
crates.io searches were run with `https://crates.io/api/v1/crates?q=<q>&per_page=100&page=<n>&sort=relevance`,
a generic user agent, and 1100 ms spacing.
The script `/var/home/user/temp/agent/toml-vet-2026-10-29/discover-crates.ts` stopped a query after two consecutive pages
added no new "filter hit" or when a page returned fewer than 100 rows.
A filter hit means a crate whose name plus description matches `toml` and one of
`comment|preserv|lossless|format|edit|round.?trip|cst|syntax tree|mutat|modif|patch|rowan|document|dom|ast`.
What that filter can hide:
a relevant crate whose name and description lack those words, and any crate beyond the stopping page.
The crates.io relevance search is fuzzy (totals far exceed true matches), so the provider never reports exhaustion for
the broad queries;
saturation for those queries rests on the two-page rule and the keyword filter only.
Raw output: `crates-round1.json` and `crates-round2.json` in the scratch directory.

### Class 1: category package registry (crates.io)

Initial schedule, in order, with pages fetched and the provider's reported total:

- `toml edit`: 4 pages, total 10959; new filter hits 6, 2, 0, 0.
- `toml comment preserving`: 8 pages, total 1309; hits 20, 0, 1, 0, 1, 1, 0, 0.
- `toml format preserving`: 3 pages, total 4319; hits 1, 0, 0.
- `toml lossless`: 4 pages, total 963; hits 6, 2, 0, 0.
- `toml cst`: 2 pages, total 112 (exhausted); hits 2, 0.
- `toml parser`: 3 pages, total 7657; hits 2, 0, 0.
- `toml_edit alternative`: 2 pages, total 1602; hits 0, 0.
- `toml document mutable`: 2 pages, total 1037; hits 0, 0.

One expansion round from taxonomy terms in the ledger (span, rowan, syntax tree, formatter, cargo manifest, patch,
round-trip, error tolerant, tombi, cargo-edit):

- `cargo manifest edit`: 6 pages, total 3205; hits 4, 3, 0, 3, 0, 0.
- `toml rowan`: 1 page, total 34 (exhausted); hits 2.
- `toml syntax tree`: 2 pages, total 2173; hits 0, 0.
- `toml span`: 7 pages, total 3504; hits 1, 0, 1, 1, 1, 0, 0.
- `toml patch`: 3 pages, total 4397; hits 1, 0, 0.
- `toml roundtrip`: 3 pages, total 635; hits 1, 0, 0.
- `toml formatter`: 3 pages, total 1155; hits 13, 0, 0.
- `toml error tolerant`: 4 pages, total 1179; hits 0, 1, 0, 0.
- `tombi`: 1 page, total 36 (exhausted); hits 1.
- `cargo-edit`: 3 pages, total 260 (exhausted); hits 6, 1, 2.

The schedule is frozen after this round.
Later taxonomy terms (for example `tree-sitter-toml`, `toml_writer`, `serde_spanned`, `tomling`) are recorded here without
further queries: they were not queried.
Result: no filter hit appeared in the last two pages of any broad query, so the class meets the two-page rule under the
stated filter; the class is not saturated by provider exhaustion.

### Class 2: repository host (GitHub)

`gh api --method GET search/repositories -f q=<q> -f per_page=100 -f page=<n> -f sort=stars`,
script `discover-gh.ts`, raw output `gh-all.json`; every response reported `incomplete_results: false`; the 1000-result
cap was not reached.

- `toml comment preserving language:rust`: 1 page, total 5.
- `toml edit format-preserving language:rust`: 1 page, total 1.
- `toml lossless syntax tree language:rust`: total 0.
- `topic:toml language:rust`: 3 pages, total 256 (exhausted).
- `toml_edit alternative language:rust`: total 0.
- `toml span language:rust`: 1 page, total 6.
- `cargo manifest edit language:rust`: 1 page, total 1.
- `toml formatter language:rust`: 2 pages, total 108 (exhausted).

334 distinct repositories were returned; 56 matched a second keyword filter over name and description.
The GitHub class is saturated by provider exhaustion for the narrow queries and the topic query.
Limit:
`tombi-toml/tombi` and `tamasfe/taplo` were reached by direct clone, not by these searches, so the topic query does not
list everything relevant.
Releases, code search and organization-project enumeration were not run.

### Class 3: broader web (peer tools, comparison terms)

- Linkup search (`linkup-search`, query "Rust crate format-preserving TOML editor alternative to toml_edit comment
   preserving"): provider error "Unauthorized action".
   The provider is blocked for this session.
- `WebSearch`, "toml_edit alternative lossless TOML CST Rust library edit Cargo.toml preserving comments 2026":
   found no alternative; results were `toml_edit` docs and wrappers (Python `tomledit`, an R package).
   The summary quoted the README limitation that dotted-key order is not preserved.
- `WebSearch`, "tombi TOML toolkit Rust crate lossless parser edit library crates.io publish":
   described `tombi-rg-tree` as a rowan-inspired lossless tree; no evidence of crates.io publication.
- Class 3 is not saturated: two queries ran, no comparison-term or blog-post queries ran.

### Class 4: this repository

- Incumbent: `package/toml-edit/core` (TypeScript, `toml-eslint-parser`).
- Prior vet-like research: `doc/planning/monorepo-manager-route-research/rust-structured-edits.md` (T1 to T5 options).
- Decision records: `doc/decision/file-enforcer-rust-rewrite.md`, `doc/decision/rust-dependency-catalog.md`.
- Hand-rolled alternatives: T3 (Rust port of `module-toml-edit`) and a `tombi` or `taplo` plus splice editor (T2, T5).
- Existing lockfile presence of `toml_edit` (see "Context").

Terminal discovery result so far:
saturated for class 1 (under the filter) and class 2, not saturated for class 3, class 4 read but not
searched with literal queries.
Overall discovery is not saturated.

## Candidate ledger

Each entry: discovery source, base category, screening result.
"Pending" means no hard-gate failure is known and targeted evidence is still owed.
All crates.io facts were fetched 2026-10-29 from `https://crates.io/api/v1/crates/<name>` by `crate-meta.ts`
(the saved `crate-meta.json` holds only the last run).

### `toml_edit` 0.25.17+spec-1.1.0

- Discovery:
   class 1, first hit of `toml edit`; class 4 (research, lockfiles).
- Facts:
   published 2026-10-09; 22 non-yanked releases since 2025-10-29 out of 130 versions;
   962,763,195 downloads, 258,339,182 recent; `MIT OR Apache-2.0`; MSRV 1.85;
   owners `epage`, `ordian` and the team `github:toml-rs:maintainers`; `trustpub_data` null on this version.
- Dependencies (`Cargo.toml` of the clone):
   normal `indexmap ^2.13.0`, `toml_datetime ^1.1.2`;
   optional under default features `parse` and `display`: `toml_parser ^1.1.5`, `winnow ^1.0.0`, `toml_writer ^1.1.3`;
   optional `serde` feature: `serde_core`, `serde_spanned`.
- Source:
   shallow clone `/var/home/user/temp/agent/toml-rs-toml-2026-10-29` at tag `v0.25.17`,
   commit `ceac6e4412420d5a5c19e4f7293ad005cf1a31b2` (the tag `toml-v1.1.8` points at the same commit).
   The workspace has `LICENSE-MIT` and `LICENSE-APACHE`, `deny.toml`, `AI_POLICY.md` (unread).
- Declared tests in `crates/toml_edit/Cargo.toml`:
   `testsuite`, `compliance`, `decoder_compliance`, `encoder_compliance`, `serde`, example `visit`;
   dev dependencies `proptest`, `toml-test-harness`, `toml-test-data`, `snapbox`.
- Screening result:
   passes screening gates; comment-preservation behavior with plain calls is a known defect cluster
   (`doc/troubleshooting/toml-edit-comment-loss.md`), so the hard constraint holds only through a repository wrapper
   (repository code around the crate; the wrapper design is in the research section "T1").
   Whether a wrapper-plus-crate counts as an "existing tool" is addressed under "Open points for the owner".

### `toml_dom` 0.6.0

- Discovery:
   class 1 (`toml edit`, `toml comment preserving`), class 2 (`toml formatter` result `kathrinmotzkus/toml_dom`).
- Facts:
   published 2026-08-13; 7 releases since 2026-05-29 (all within the past year); 739 downloads, 252 recent;
   `MIT`; MSRV 1.70; single owner `kathrinmotzkus`; dependencies `chrono ^0.4`, `indexmap ^2`.
- Source:
   clone `/var/home/user/temp/agent/toml_dom-2026-10-29`, default-branch commit
   `5dec5fac98860e2fa39bacc2250323bd40bfd365` (not the 0.6.0 tag; re-clone at the tag before validation).
- README claims (read, not verified):
   TOML 1.1, format-preserving round trip, "exact preservation of unchanged source representation: guaranteed",
   structured errors with source positions, transactional editing.
- Screening result:
   pending; serious alternative if source and behavior hold up.

### `tomlini` 0.1.0

- Discovery:
   class 2 (GitHub `mcunha/tomlini`, query `toml comment preserving language:rust`), then crates.io by name.
- Facts:
   published 2026-06-05, one release; 28 downloads, 5 recent; `MIT OR Apache-2.0`; MSRV 1.85; single owner `mcunha`;
   zero normal dependencies.
- Source:
   clone `/var/home/user/temp/agent/tomlini-2026-10-29`, default-branch commit
   `837e73b93759d689eda63a7c70116323568cd300`.
- README claims (read, not verified):
   byte-range splice edits over a flat span index, "no decor model", builder API with `with_above_comment`,
   490 `toml-test` tests, lenient/relaxed/strict validation modes (strict mode is needed for the spec-compliance
   constraint).
- Screening result:
   pending; serious alternative on the strength of the design; maintenance and adoption evidence are thin and are
   scored concerns, not hard gates.

### `fig` 7.0.0 (with `fig-core` 7.0.0)

- Discovery:
   class 1 (`toml edit`, `toml comment preserving`).
- Facts:
   published 2026-10-15; 28 releases in about two weeks (5.1.1 to 7.0.0 between 2026-10-02 and 2026-10-15);
   4,646 downloads; `MIT OR Apache-2.0`; MSRV 1.88; single owner `adammharris`;
   `fig-core` depends on `fig-kit` only.
- Source:
   clone `/var/home/user/temp/agent/fig-2026-10-29`, default-branch commit
   `56dbf65e0b3d8bba09cf124ccff7320f6359be40`.
- README claims (read, not verified):
   lossless config editing across ten formats in the CLI; "The underlying library ... has no languages built in. Every
   language is registered the same way."
   Whether the library crate ships a TOML language module is not yet established; `rg -l -i toml rust` found hits in
   `fig-lang-export`, `fig-capi`, `fig-cli` and `fig-conform`, not in `fig-core`.
- Screening result:
   pending; the library may need a TOML module that is not part of the published `fig-core`, which would be a
   category or hard-constraint failure.

### `edikt-toml` 0.6.2

- Discovery:
   class 1 (`toml comment preserving`).
- Facts:
   published 2026-09-29; 11 releases since 2026-09-05; 418 downloads; `MIT OR Apache-2.0`; MSRV 1.88; single owner
   `jhheider`; depends on `edikt-core ^0.6.1`, `thiserror ^2` and `toml_edit ^0.22` (an older `toml_edit` line than the
   current 0.25).
- Source:
   workspace clone `/var/home/user/temp/agent/edikt-2026-10-29`, commit `c66c62c2de9e45c4ae324f952054ea4245b1a0d3`.
- Screening result:
   pending; it is a multi-format wrapper over `toml_edit`, so it adds a layer and a second `toml_edit` major line
   rather than a different TOML engine.

### `patchloom` 0.37.6

- Discovery:
   class 1 (`toml comment preserving`, `toml edit`), class 2.
- Facts:
   published 2026-10-24; 54 releases, all since 2026-10-08 at the earliest of the six latest listed (54 of 54 in the past year); 1,610 downloads;
   `MIT OR Apache-2.0`; MSRV 1.95; single owner `SebTardif`; depends on `toml_edit ^0.25.15` and a large surface
   (`reqwest` and `rmcp` as dev dependencies, `noyalib-serde-yaml`, `yaml-edit`, `similar`, `schemars`).
- Source:
   clone `/var/home/user/temp/agent/patchloom-2026-10-29`, commit `2d9e1bed5c1f4b7852f507085e6e2e79938a1417`;
   `Cargo.toml` shows a `[[bin]]` at line 148 and no `[lib]` section.
- Screening result:
   pending; likely a category mismatch (a CLI and MCP server for AI agents that delegates TOML to `toml_edit`).
   Confirm whether `src/lib.rs` exports a stable library API before exit.

### `toml-spanner` 1.0.3

- Discovery:
   class 1 (`toml span`, `toml format preserving`).
- Facts:
   published 2026-06-17; 10 releases since 2026-03-29; 61,053 downloads, 44,138 recent;
   `MIT OR Apache-2.0`; MSRV 1.90; single owner `exrok`; optional dependencies `serde`, `zmij`, `toml-spanner-macros`.
- Source:
   clone `/var/home/user/temp/agent/toml-spanner-2026-10-29`, commit `c2d94d98744d7dcde8a28fbe927bc92ec212ecc4`.
- README claims (read, not verified):
   "Format Preserving Serialization, even through mutation on your own data types"; span-preserving tree;
   passes the official `toml-test`; fuzzed under miri and memory sanitizers.
   It exposes `Formatting::preserved_from(&doc)` (README lines 123 to 134).
- Screening result:
   pending; it may support the span half of the requirement and a typed round trip, but whether it edits an arbitrary
   untyped document (set a key in an existing table, append a block) while keeping comments is unverified.
   It may also serve as the companion for spans instead of `toml`.

### `toml` 1.1.8+spec-1.1.0 and `toml-span` 0.7.2 (deserializer companions)

- Discovery:
   class 1.
- Facts:
   `toml` is already in `rust-catalog.toml` (`toml = ">=1.1"`), from the same repository as `toml_edit`
   (`toml-rs/toml`, tag `toml-v1.1.8`, same commit as `v0.25.17`);
   `toml-span` published 2026-10-02, `MIT OR Apache-2.0`, description "Toml parser and deserializer that preserves span
   information", owner `embark-studios`.
- Screening result:
   category mismatch as editors (read and deserialize, no document mutation per their descriptions;
   metadata only for `toml-span`; README unread).
   They remain eligible only as the span-bearing deserializer companion, which the "typed deserialization errors with
   spans" clause allows.
   `toml_edit`'s own `de::Error::span()` is cited by prior research but not read this session.

### `taplo` 0.14.0

- Discovery:
   class 1 (`toml parser`), class 4 (prior research T5).
- Facts:
   last release 2025-05-22; 0 releases in the past year (32 versions in total); 3,792,032 downloads, 730,975 recent;
   `MIT`; MSRV 1.74; owners `tamasfe`, `JounQin`; depends on `rowan ^0.15.3`, `logos ^0.12`, `time`.
- Source:
   clone `/var/home/user/temp/agent/taplo-2026-10-29`, commit `08f343be02ce1b20296470396a42f0fa47820449`.
- Screening result:
   pending hard-constraint exit.
   The September research recorded that it rejects `clap = { version = "3", features = [ "derive", ], }`
   ("expected value, trailing comma is not allowed") and fails the TOML 1.1 fixtures
   (`doc/troubleshooting/taplo-toml-1-1-inline-table-trailing-comma.md`).
   Not re-probed this session; re-probe in a container before recording the exit.
   It is also a parser and DOM with only value splices and key rename, so every other edit would be repository code.
- Related:
   `olpat` 0.14.0 (2026-07-08, "A TOML parser, analyzer and formatter library") appears to be a fork; unscreened.

### `tombi` family

- Discovery:
   class 1 (`tombi`, `toml rowan`), class 3, prior research T2.
- Facts:
   crates.io holds only placeholders: `tombi` 0.0.1 (2024-11-19, "Reserved package for tombi"), `tombi-cli` 0.0.1,
   `serde_tombi` 0.0.1, `tombi-lib` 0.0.0 (2026-10-12, "Placeholder for the tombi library crate. Name reserved."),
   `tombit` 0.0.0.
   The workspace crates (`tombi-parser`, `tombi-ast`, `tombi-document-tree`, `tombi-formatter`, `tombi-lexer`, ...) exist
   in the clone.
- Source:
   clone `/var/home/user/temp/agent/tombi-2026-10-29`, default-branch commit
   `3c49ebcffee605c14f77a53f57bc7e96230bdbae`.
- Screening result:
   a git dependency is allowed by `doc/decision/rust-dependency-catalog.md` (git entries carry `git` and `rev`), so the
   unpublished state is not a hard failure.
   The crates are parsers and formatters; no edit API was found (not exhaustively searched).
   If that holds, it is a custom-editor path (see "Open points for the owner").

### Parser-only or formatter-only crates (category mismatch for an editor)

- `tomlsmith` 0.4.1:
   README says "Parse once and query diagnostics, formatted output, highlights, and decoded values through the immutable
   `Document` API"; formatter and checker; no mutation API (clone `tomlsmith-2026-10-29`, commit
   `a667b2f75930d2e64ac8516f738d93a36810cdd6`).
   Exit candidate: category mismatch, README read only.
- `kosmetic-syntax` 0.1.0 (repository `GeneralKoski/kosmetic`):
   linter, formatter and language server with a lossless parser; 26 downloads; one release 2026-10-22;
   no public edit API seen (README read; clone `kosmetic-2026-10-29`, commit `36d05ec3bb5ac0c17d2096228c8a12f126e5e476`).
   Exit candidate: category mismatch.
- `toml_parser` 1.1.5:
   event parser used by `toml_edit`; a custom splice editor on it is the research's T3 path (custom).
- `oxc-toml` 0.15.1:
   "A TOML formatter library"; MSRV 1.95.0; `trustpub` data present; no normal dependencies;
   clone `oxc-toml-2026-10-29`, commit `394611978d6a6a3aecc515a0f0994ac8305a9dba`.
   README unread; exit candidate: category mismatch (formatter).

### Wrappers and command-line tools over `toml_edit` (not independent engines)

- `cargo-edit` 0.13.14 (CLI), `cargo_toml_edit` 0.0.3 (`toml_edit ^0.25`, hosted on Codeberg, `cfg-expr` dependency),
   `tomato-toml` 1.0.0 (`toml_edit ^0.23`, BlueOak-1.0.0, CLI), `toml-cli` 0.2.3 (`toml_edit ^0.15`, last release 2022),
   `aqc-cargo-toml-engine`, `dependency-check-updates-rust`, `verseconf-toml`.
   Metadata only; they sit on `toml_edit`, so they add no distinct TOML engine.
   Exit candidates: category mismatch or dependency on `toml_edit`.

### Licence or terms question

- `callisto-manifests` 0.16.0:
   crates.io licence field `FSL-1.1-MIT`; a release-engine product with 20 releases since 2026-10-23.
   Screening exit candidate: terms (a source-available licence that is not an OSI licence until its conversion date);
   the licence text was not read.

### Abandoned or pre-1.0 TOML crates

- `tomllib` 0.1.2 and `toml_document` 0.1.3 (both last published 2016, before TOML 1.0), `toml-parse` 0.2.11 (2020),
   `matklad/tom` (last push 2022-06-17), `LeopoldArkham/Molten` (last push 2018-04-14),
   `kgv/toml-lalrpop` (2020-06-08), `joelself/tomllib` (2021-07-10).
   Exit candidates on the hard constraint "accepts TOML 1.1 forms"; no source probe was run, so the exit is not yet
   evidenced beyond publication dates.

### Incumbent `@monochromatic-dev/toml-edit-core` 0.0.1

- Discovery:
   class 4.
- Screening result:
   exit on the hard constraint "Rust library embeddable in a Rust binary; no Node runtime";
   the Rust rewrite is accepted and the TypeScript tool is deleted at the cutover
   (`doc/decision/file-enforcer-rust-rewrite.md`, "Cutover": "deletes `package/file-enforcer/core`").
   It stays as the replacement-parity baseline: its consumed operations are `applyCargoPlan`
   (`package/file-enforcer/core/src/cargo/apply-plan.ts`, guarded sets, `[lints.clippy]` and `[workspace]` appends),
   `getTomlProperty`, and the exported but unused `editTomlKey` and `overwriteTomlKey`.
   The September research measured it at 15 of 21 on the shared cases (not reproducible now).

## Evidence records

Only the claims gathered this session are listed; every record is access date 2026-10-29.

- Claim:
   crates.io facts above.
   Status: pass or lead as labelled.
   Source: `https://crates.io/api/v1/crates/<name>`, `.../<version>/dependencies`, `.../owners`; commands in
   `crate-meta.ts`; Linux x64 host, Node from `.cache/toolchain-bin/node`, exit 0.
   Counterevidence: none checked.
- Claim:
   skill identity (commit, SHA-256).
   Command: `git log --format=%H -1 -- .agents/skills/choosing-technology/SKILL.md`; `sha256sum` of both copies.
- Claim:
   the `toml_edit` clone is the published release.
   Command: `gh repo clone toml-rs/toml /var/home/user/temp/agent/toml-rs-toml-2026-10-29 -- --depth 1 --branch v0.25.17`;
   `git describe --tags` printed `toml-v1.1.8`; `git rev-parse HEAD` printed
   `ceac6e4412420d5a5c19e4f7293ad005cf1a31b2`.
   The crate's published checksum was not compared with a source build; that comparison is owed.
- Claim:
   no code was executed in this session beyond the discovery scripts and `cargo --version`; no third-party command
   tree was run, so no execution manifest exists yet.

## Hard-gate outcomes so far

None confirmed.
Pending exits and the evidence each still needs are in the candidate ledger.
No candidate has been scored.

## Open points for the owner

- Custom editors versus existing tools.
  The skill says a custom implementation is eligible only when every existing tool fails a named hard constraint.
  `toml_edit` fails the comment-preservation constraint when called as its README shows (12 of 21 in the September
  probe) and passes only through a repository wrapper.
  The next agent should treat the wrapper-plus-crate as one candidate ("existing crate with a thin policy layer") and
  record that determination, then treat `tombi` or `taplo` or `toml_parser` plus a splice editor as custom paths that
  exit unless every existing editor fails a hard gate.
  `tomlini`, `toml_dom` and `fig` claim to be ready-to-use editors and therefore must get equal-depth validation if
  they survive screening.
- Question for the user: none is needed now.
  The context forks in the skill (primary delivery, self-hosted, price, geography, trust boundary, platform) are settled
  by the repository facts and the accepted decision.
  Ask only if the sensitivity run reverses a winner, or if two finalists tie exactly.
  If a question becomes necessary, the likely one is the weight of ready-made scale and maintenance history against
  an engine design with less repository code (for example `tomlini`): options (a) weight 5 on maintenance and adoption
  evidence, (b) weight 5 on the amount of repository wrapper code, (c) equal weights; recommended order
  (c) > (a) > (b), because the rest of the rubric already carries both concerns and unequal weights would pre-decide
  the ranking.
  This is a contingency, not an open question now.

## Steps remaining, in the skill's order

1. Finish discovery.
   - Class 3: run comparison-term and peer-tool queries; Linkup is unauthorized, use `WebSearch`, `agent-browser` or
      another provider.
   - Class 1 and 2: optional extra queries for taxonomy terms recorded above; GitHub code search for
      `DocumentMut` or `toml_edit` consumers is not required.
   - Class 4: run literal `rg` queries over `doc` and `package` with explicit paths and record counts;
      run `cargo tree --invert toml_edit --offline` in `package/ide/app` and `package/nested-wayland-session/cli`.
   - Record the terminal discovery result.
2. Freeze context and the rubric before rating any candidate.
   Draft criteria, default weight 1 each: comment-preserving edit fidelity on repository operations with the least
   wrapper code; parse and deserialize error spans; dependency and build surface (count and same-author crates);
   maintenance evidence (issue and PR sample per the skill, releases, maintainer concentration); upstream test depth
   (conformance, property, fuzz); platform CI matrix evidence; TOML 1.1 and spec compliance; API fit with
   `workspace = true` catalog operations.
   Human-auditability measurements apply as measurements feeding the dependency and source-size criteria.
3. Screening exits with evidence:
   re-probe `taplo` on a TOML 1.1 inline-table trailing comma in a container;
   read the `fig` TOML module status, the `patchloom` library surface, `oxc-toml` and `tomlsmith` README or API for
   mutation, the `callisto-manifests` licence text;
   probe the pre-2021 and old crates (`tomllib`, `toml_document`) for TOML 1.1.
4. Hard-gate confirmation for survivors:
   licence texts, manifests, lifecycle scripts (`build.rs`), transitive tree, native or generated code, security
   disclosure policy, release provenance (tags, CI release workflow, `trustpub` data for `toml_edit` and
   `oxc-toml`), supply-chain policy compatibility with `minimumReleaseAge` and `trustPolicy`.
5. Execution manifest per candidate before running any command tree, with the skill's default isolation:
   `podman run --memory=2g --cpus=2 --rm`, no ambient credentials, no real home mount, read-only repository mount,
   private scratch volume, network disabled after a separately inspected `cargo fetch` phase, image digest recorded.
   Quote the synthetic-load ban in any subagent brief.
6. Equal-depth finalist validation for every hard-gate-confirmed candidate:
   re-clone at the published release tag; read promised-behavior, error and serialization source with `path:line`
   excerpts; tests, CI and fuzzing or mutation evidence (absence reported inline);
   maintenance audit (issues and PRs updated in the last 12 months: all if at most 20, otherwise the 10 most recently
   updated; maintainer comments versus actions; merge latency; releases);
   run each upstream default CI-equivalent path and relevant non-default suites in the container;
   consumer-boundary probe, the same for every finalist: parse all 26 tracked `Cargo.toml` files and
   `rust-catalog.toml` from a read-only mount and compare round trips; run the operations the Rust file-enforcer needs
   (set inside an existing table, no creation when the table is absent, append a block, `workspace = true` forms,
   array append and remove, key delete with attached comments) and one deliberately malformed file to capture the
   error span; reuse the 91 valid and 108 invalid `toml-test` fixtures under `package/test-fixture/toml-edit/src/`;
   Linux x64 locally, cite upstream CI for macOS and Windows, state the limit.
7. Replacement parity: inventory the incumbent's consumed paths (see the incumbent entry) with a parity test for each.
8. Score validated finalists with the frozen rubric; run the full one-at-a-time sensitivity matrix; pros and cons per
   finalist; fully sorted ranking with the reason for each adjacent pair; confidence and evidence limits.
9. Synthesize, state the recommendation or the no-recommendation result, and end with the question only if the skill
   requires one.
10. Update this report after each phase, commit when the owner allows it, and release the lock.

## Scratch paths, clones and processes

- Scratch directory (mode 700): `/var/home/user/temp/agent/toml-vet-2026-10-29/` containing
  `discover-crates.ts`, `discover-gh.ts`, `crate-meta.ts`, `clone-all.ts`, `fingerprint.ts`, `fingerprint.out`,
  `crates-round1.json`, `crates-round2.json`, `gh-all.json`, `crate-meta.json` (last run only: `tomlini`, `fig-core`,
  `cargo_toml_edit`, `edikt-core`, `toml-cli`, `tomato-toml`; the earlier run was overwritten, its facts are in this
  report).
- Clones, all shallow (`gh repo clone <repo> <dir> -- --depth 1`) under `/var/home/user/temp/agent/`:
  - `toml-rs-toml-2026-10-29` at tag `v0.25.17`, commit `ceac6e4412420d5a5c19e4f7293ad005cf1a31b2`.
  - Default-branch HEADs (re-clone at release tags before validation):
     `taplo-2026-10-29` `08f343be02ce1b20296470396a42f0fa47820449`,
     `tombi-2026-10-29` `3c49ebcffee605c14f77a53f57bc7e96230bdbae`,
     `fig-2026-10-29` `56dbf65e0b3d8bba09cf124ccff7320f6359be40`,
     `toml_dom-2026-10-29` `5dec5fac98860e2fa39bacc2250323bd40bfd365`,
     `tomlsmith-2026-10-29` `a667b2f75930d2e64ac8516f738d93a36810cdd6`,
     `patchloom-2026-10-29` `2d9e1bed5c1f4b7852f507085e6e2e79938a1417`,
     `edikt-2026-10-29` `c66c62c2de9e45c4ae324f952054ea4245b1a0d3`,
     `toml-spanner-2026-10-29` `c2d94d98744d7dcde8a28fbe927bc92ec212ecc4`,
     `kosmetic-2026-10-29` `36d05ec3bb5ac0c17d2096228c8a12f126e5e476`,
     `oxc-toml-2026-10-29` `394611978d6a6a3aecc515a0f0994ac8305a9dba`,
     `tomlini-2026-10-29` `837e73b93759d689eda63a7c70116323568cd300`,
     `toml-span-2026-10-29` `f646aba035b5f6599c9d51f74017cc99a17281a6`.
  - A first `toml-rs-toml-2026-10-29` clone at default HEAD was deleted and replaced by the tag clone.
- No container, `systemd-run` unit or background process was started by this session.
  `ps` showed `podman` processes from other sessions (for example a node container for
  `/var/home/user/temp/agent/fe-rust/gates/p3`); they are not this session's and were left running.
- The vet lock directory
  `/var/home/user/temp/agent/technology-vet-locks/var-home-user-Monochromatic-doc-audit-tech-file-enforcer-toml-editor-vet-2026-10-29.md.lock`
  was created at the start and removed at the halt after this report was written.
