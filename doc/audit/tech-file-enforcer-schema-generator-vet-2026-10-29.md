# File-enforcer schema generator vet report

Superseded 2026-10-29 before a verdict:
the user approved `schemars` without a completed vet
("I'm approving toml_edit, browserslist-rs, schemars as-is."),
recorded in `doc/decision/file-enforcer-rust-rewrite.md`, "Consequences".
This halted draft is kept whole as knowledge ("Even if work is superseded, it should still become knowledge.", user).
Open facts for the implementation:
it is unproven that `JsonSchema` derive works on types with no serde `Deserialize` impl,
and how it represents `kind`-tagged rules and `{"file"}`, `{"var"}`, `{"concat"}` values (no probe was built);
and the golden test's corpus check needs a JSON Schema validator (for example the `jsonschema` crate),
a dependency the approval did not name.

Status:
 in progress, halted 2026-10-29 for handoff.
The coordinator stopped this audit during discovery and screening.
No recommendation exists, no candidate is rated, and the key requirement
(`JsonSchema` derive on types with no serde `Deserialize`, tagged enums, tagged value objects)
has not yet been proved or refuted by any build.
Nothing in this report is a verdict.

Lifecycle phase:
 discovery partly executed, screening partly executed.
Targeted evidence, execution manifest, finalist validation, scoring, sensitivity, and synthesis are not started.

Subject:
 file-enforcer schema generator.

Decision scope:
 choose how the Rust file-enforcer config JSON Schema (committed `file-enforcer.schema.json`,
 golden-tested against a hand-written decoder) is produced from Rust types.
The named candidate comes from the accepted design
([`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md), section "Schema"):
`schemars` 1.2.3 (published 2026-10-12).
That record also lists a choosing-technology vet for `schemars` as a gate before implementation.

Start date:
 2026-10-29.

Last updated:
 2026-10-29.

Governing skill commit:
`37117e36397e30233062828581a5e38217da9458`
(last commit touching `.agents/skills/choosing-technology/SKILL.md`).
The `.claude/skills/choosing-technology/SKILL.md` copy is an untracked mirror with the same hash.

Governing skill SHA-256:
`552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.

Compatibility fingerprint:
`396d088ee89f0cb3207504e363c2f5c663abea5d42102030c46c1ed05ecec1f7`.
Built from schema version 1, subject, decision scope, the ten hard constraints, deployment, trust boundary,
incumbent (null), base category, and overlays, canonicalized as sorted-key compact JSON
(RFC 8785 equivalent for this ASCII-only input) and hashed with SHA-256.
The exact canonical input is `/var/home/user/temp/agent/schemars-vet-2026-10-29/fingerprint-input.json`
and the generator is `/var/home/user/temp/agent/schemars-vet-2026-10-29/scripts/fingerprint.py`.

Active audit owner:
 none; the delegated session that started this audit was halted and the owner continues it later.

Prior compatible report:
 none.
`rg --files doc/audit` for the slug `tech-file-enforcer-schema-generator` found nothing.
A sibling session held a lock for `tech-file-enforcer-toml-editor-vet-2026-10-29.md` (different subject, different
fingerprint), which does not conflict.

## Hard constraints (frozen before candidate evidence was read)

- H01:
  derives the schema from Rust type definitions in the repository,
  with no hand-maintained second source of truth for field shapes.
- H02:
  derive works on types that implement neither serde `Deserialize` nor serde `Serialize`
  (the config is decoded by hand with the serde-free `monochromatic-jsonc-edit`).
- H03:
  represents an internally tagged enum whose tag field is named `kind` (rule kinds).
- H04:
  represents untagged unions of literals and single-key tagged objects
  (`{"file": ...}`, `{"var": ...}`, `{"concat": [...]}`), including recursion.
- H05:
  allows an optional string property named `$schema` and can reject unknown keys.
- H06:
  output is deterministic across runs, so a committed golden file compares byte for byte.
- H07:
  emits a dialect that VS Code and JetBrains both consume.
- H08:
  licence compatible with the repository (LGPL-3.0-or-later crates, permissive dependencies).
- H09:
  inspectable source whose published crate maps to a public tag, including the proc-macro.
- H10:
  builds on the repository Rust toolchain (`nightly-2026-10-29`, edition 2024) and the Linux x64 container,
  and satisfies the `rust-catalog.toml` release-age policy (`minimumReleaseAge = 1440`).

## Base components and overlays

- Base:
  inspectable open-source local technology (a Rust derive proc-macro crate and its runtime trait crate).
- Overlay, native or generated-code boundary:
  applies, because a proc-macro executes at compile time and a generated schema file is committed.
- Overlay, high-trust CI execution with human auditability:
  applied conservatively, because the proc-macro tree runs on developer machines and CI runners during `cargo build`
  and `cargo test`.
- Overlay, multi-platform:
  applies (Linux x64 run, macOS arm64 and Windows x64 as cross-check or claim review only).
- Overlay, incumbent dependency replacement:
  not applicable.
  The TypeScript tool has no config schema (`rg schema` over `package/file-enforcer/core` finds none),
  and the two sibling JSONC configs (`cli-git.config.jsonc`, `monochromatic-lint.config.jsonc`) have none.
- Overlay, sensitive data:
  not applicable; the generator handles only type definitions.
- No managed service is involved, so the SaaS gates are not applicable.

## Context facts read from the repository

- Accepted design:
  committed `file-enforcer.schema.json`, derived with `schemars`, cargo golden test that also checks the hand-written
  decoder against a coverage-complete sample corpus, mapped in `.idea/jsonSchemas.xml`, `"$schema"` kept for VS Code
  (`doc/decision/file-enforcer-rust-rewrite.md`, lines 149 to 155).
- The user chose "Generated, self-enforced" and then "Golden test + IDE mapping"
  (`doc/planning/file-enforcer-rust-rewrite.md`, lines 647 to 661 and 852 to 862).
  Whether to have a schema at all is therefore decided and out of scope here.
- An outside review (`doc/research/file-enforcer-rust-rewrite-review-sol.md`, lines 385 to 413) warns that
  hand-written decoding and schemars derivation are independent implementations, and that derivation alone does not
  establish agreement about tagged values, unknown fields, or the config's `$schema` member.
  It asks for paired acceptance and rejection cases against the decoder and the generated schema.
- JetBrains support for an instance-level `$schema` in `.jsonc` is unverified
  (`doc/research/file-enforcer-rust-rewrite-crates.md`, section 4; the planning doc repeats it).
- `rust-catalog.toml` lists `serde = ">=1"` and `serde_json = ">=1"` (lines 90 and 91) and no `schemars`.
  `rg` over `package` finds no Rust crate using `schemars`; the only hits are research and planning prose.
- `package/jsonc-edit/core-rust/Cargo.toml` states there are no runtime dependencies by decision and that anything added
  needs a fresh choosing-technology run.
  Its licence is LGPL-3.0-or-later.
- The repository toolchain is `nightly-2026-10-29` (`toolchain.lock.json`, entry `rust`).
  The only local container image with Rust that was found is `localhost/git-policy-native-mutation:w53`
  (Debian trixie, cargo 1.97.0 stable), created by another session; see "Execution manifest".

## Criteria for scoring (proposed, not yet frozen)

No candidate has been rated.
The owner must freeze these, with their weights, before any rating.
The user named no priority, so each defaults to weight 1.

- Shape fidelity for the config's value forms (internal tag, single-key objects, untagged unions, recursion).
- Independence of the config types from serde (coupling and dependency surface).
- Determinism and golden-test suitability (key order, readable diffs, dialect).
- Human auditability (non-test lines, runtime dependency count, proc-macro size).
- Maintenance depth.
- Editor consumption (dialect support in VS Code and JetBrains, descriptions from doc comments).

## Query schedule and discovery ledger

All crates.io API queries used `https://crates.io/api/v1/crates?...&per_page=100&sort=relevance`,
with no include or exclude filter, pages fetched until exhaustion or until the provider returned HTTP 400.
Raw page files are in `/var/home/user/temp/agent/schemars-vet-2026-10-29/discovery/` as `qNN-*.pN.json`.
The fetcher is `scripts/crates-query.ts`.

### Source class 1: crates.io registry

- q01, literal `q=json schema`:
  total 14112, fetched 1000, HTTP 400 at page 11.
  The provider caps results, so this query is **blocked**, not saturated.
- q02, `q=jsonschema`:
  total 982, fetched 982 over 10 pages, complete.
  Screened in full (see "Screening results").
- q03, `q=schemars`: total 943, fetched 943, complete.
  Not yet screened.
- q04, `keyword=json-schema`: total 184, fetched 184, complete.
  Not yet screened.
- q05, `keyword=jsonschema`: total 44, fetched 44, complete.
  Not yet screened.
- q06, `q=derive json schema`: total 5424, fetched 1000, HTTP 400 at page 11.
  Capped; **blocked**.
- q07, `q=schema generate from rust types`: total 8854, fetched 1000, HTTP 400 at page 11.
  Capped; **blocked**.
- q08, `keyword=schema-generation`: total 0.
- q09, `q=openapi schema derive` (expansion term for `utoipa`): total 1024, fetched 1000, HTTP 400 at page 11.
  Capped; **blocked**.
- q10, `q=type reflection schema export`: total 584, fetched 584, complete.
  Not yet screened.
- Independent enumeration for the capped queries:
  the crates.io database dump `https://static.crates.io/db-dump.tar.gz`
  (redirects to `cloudfront-static.crates.io`, 2,103,407,089 bytes, last modified 2026-10-29 02:11:22 GMT,
  snapshot directory `2026-10-29-020020`, data dated 2026-10-28 22:00).
  Extracted only `crates.csv`, `keywords.csv`, `crates_keywords.csv`, `categories.csv`, `crates_categories.csv`,
  `versions.csv`, `default_versions.csv` into `dbdump/` (treated as untrusted data; read with `python3 -I`).
  364,345 crates scanned.
  Filter (recorded because it can hide crates): a crate matches when `name + description + keywords` matches
  `json[ _-]?schema|jsonschema` (case-insensitive) or has keyword `json-schema`, `jsonschema`, or `schema`.
  1307 crates match (`discovery/dump-json-schema.json`, script `scripts/dump-enum.py`).
  What the filter hides: crates whose description and keywords never say "json schema" but whose derive emits one
  (for example `utoipa`, found only through q09 and the web search).
  A second pass, `scripts/dump-screen.py`, printed 107 crates whose description matches a generate or derive pattern
  (and does not match reverse-direction, client, validator, MCP, or LLM words) plus 75 crates matched only through
  their readme text; the output is `discovery/dump-screen.out`.
  This pass ran once, by eye, and is not a saturation proof.

### Source class 2: repository host (GitHub search API)

- g01, `topic:json-schema language:Rust`: total 124, fetched 124 (2 pages), complete.
  Not yet screened.
- g02, `topic:jsonschema language:Rust`: total 15, fetched 15, complete.
  Not yet screened.
- g03, `json schema generate from rust structs derive language:Rust in:name,description,readme`:
  total 2080, fetched 1000 (10 pages, provider cap), **blocked**; used for leads only.
  Not yet screened.
- Fetcher: `scripts/gh-query.ts`, sorted by stars; raw files in `discovery/g0N-*.pN.json`.

### Source class 3: broader web

- `rust generate JSON Schema from structs without serde Deserialize derive alternative to schemars`:
  returned only `schemars` documentation; no alternative named.
  A result summary (not a primary source) says schemars documents that serde-compatible attributes can be overridden
  with schemars-specific attributes, and that it works without serde.
  This must be confirmed in source or docs by the owner's run; it is a lead.
- `facet-json-schema OR utoipa ToSchema standalone JSON Schema 2020-12 rust crate compare schemars`:
  surfaced `facet-json-schema` and `facet-jsonschema` (derive from facet reflection, not serde) and confirmed
  `schemars` 1.2.2 shows a 2020-12 `$schema` URI in example output while 0.8.5 emits draft-07.
  It did not return `utoipa` results.

### Source class 4: this repository

- `rg` for `schemars|JsonSchema` over `package`, `doc`, `rust-catalog.toml`: prose only, no Rust code.
- Incumbent and hand-rolled alternatives: none.
  No in-repo schema generator exists, and no prior vet report covers a schema generator
  (`doc/audit/tech-*` was listed; the nearest are the `jsonc-edit` foundation vets, which cover parsing).

### Expansion round (not yet executed)

Taxonomy terms recorded so far, for one de-duplicated expansion round to be appended and frozen:
`openapi` (already used in q09), `typedef`, `reflection` (q10), `facet`, `specta`, `type export`, `serde-reflection`,
`okapi`, `aide`.

### Terminal discovery result

Not reached.
Four capped queries (q01, q06, q07, q09) and one capped GitHub query (g03) are replaced for enumeration purposes by the
database dump, but the dump filter was applied once and the two-page no-new-survivor rule has not been run over q03,
q04, q05, q10, g01, g02.
Discovery is therefore **not saturated**.

## Candidate ledger and screening results

Each entry gives base category, overlays, and the screening result reached so far.
All are inspectable open-source local technology unless noted.

### Survivors carried forward (serious alternatives, targeted evidence not yet collected)

- `schemars` 1.2.3 (crates.io `schemars` and `schemars_derive`):
  MIT, MSRV 1.74, created 2019-08-08, 98 releases, 593,835,289 total and 224,582,601 recent downloads,
  last published 2026-10-12, repository `https://github.com/GREsau/schemars`.
  Shallow clone `/var/home/user/temp/agent/schemars-2026-10-29` at commit
  `4791bb7352555d339e45658ca09c092ed9fa1f5d`; `git rev-parse v1.2.3` returns the same commit,
  so the tag equals the clone head.
  The crates.io package has not yet been compared with that tag (`.cargo_vcs_info.json` check pending).
  Crate readme mentions both `2020-12` and `draft-07`.
  The research note says default dialect is draft 2020-12 (`schemars/src/generate.rs:26 to 106` in the prior note),
  not re-verified here.
  Gate status:
  H01 expected pass, H02 to H06 **pending probe**, H07 pending editor evidence, H08 MIT pass on metadata,
  H09 tag matches clone head, H10 pending build.
- `facet-json-schema` 0.50.0-rc.7 (created 2025-12-31, updated 2026-08-20, 2,925 downloads, MIT OR Apache-2.0,
  MSRV 1.92, repository `facet-rs/facet-format`) and its older sibling `facet-jsonschema` 0.46.0
  (updated 2026-05-09, 17,357 downloads, 111 recent, MIT OR Apache-2.0, MSRV 1.90, repository
  `facet-rs/facet-jsonschema`).
  Derives from facet reflection (`#[derive(Facet)]`), so no serde is involved.
  Clones: `facet-format-2026-10-29` at `4279debff780ae1cd5b028201f446c26594b1120`,
  `facet-jsonschema-2026-10-29` at `b6a4203b1634d59c5bbb8077bf4117e18e9e7f7c`.
  Open: whether crates.io offers a non-release-candidate of `facet-json-schema` (the API `max_version` includes
  pre-releases; this was not checked), which bears on H10 and the repository's catalog policy.
  Gate status: H02 passes by construction (no serde), H03 to H07 **pending probe**.
- `utoipa` 6.0.0 (`utoipa-gen` 6.0.1; MIT OR Apache-2.0; MSRV 1.88; 58,453,493 downloads; updated 2026-09-22;
  repository `juhaku/utoipa`).
  Clone `utoipa-2026-10-29` at `c7fd8ade4400fba4b68da107a2f7b8dc75c36570`.
  Open question the probe must answer: whether a standalone JSON Schema document with resolvable `$ref` targets
  (`#/$defs/...`, not `#/components/schemas/...`) can be produced without post-processing.
  If not, it fails H01 or H07 with evidence.
  Gate status: **pending probe**.
- `specta-jsonschema` 0.0.5 (created 2026-03-02, updated 2026-10-21, 402 downloads, MIT; part of
  `specta-rs/specta`, which publishes `specta` 2.0.0-rc.27).
  Readme mentions drafts `2019-09`, `2020-12`, and `Draft 7`, and has `tag` mentions (6).
  Clone `specta-2026-10-29` at `954ea9063e63e734e9dd3bb2e161fca026163d06`.
  Pre-1.0 on both crates; gate status **pending probe**.

### Fork lineage of schemars (classified, not promoted)

`oxc-schemars` 0.9.1, `ckb_schemars` 0.8.22, `architect-schemars` 1.2.0, `cog_schemars` 0.1.0,
`apistos-schemars` 1.0.0-alpha.2, `schema_jsonrs` 0.1.0 (single release, 2025-01-12), `tamasfe-schemars`,
`dbt-yaml_schemars_derive`, `rovo-schemars-derive`, and the derive-only forks `yammer-derive`, `claudius-derive`,
`stigmergy-derive`.
These are forks or re-exports of the same technology for other serde backends or older schemars lines.
They are not independent alternatives and are not promoted.
This is a classification, not a hard-gate failure, so the owner may reopen it if upstream `schemars` fails a gate.

### Exits with evidence

- `rschema` 0.6.1 (last release 2022-04-23, last commit 2022-04-24; clone `rschema-2026-10-29`
  at `a431ddbd9ece89b4f50dd5919b81933c89d0f964`):
  fails H03 and H04.
  `rg -i "untagged|tag"` over `rschema-derive/src` and `rschema-core/src` finds only its own internal
  `#[serde(untagged)]` uses for schema keyword types, no tag attribute support for user enums.
  It does implement a `Draft` enum up to `Draft202012` (`rschema-core/src/draft.rs`).
- `json-schema-derive` 0.0.2 (two releases, 2025-04-07; clone `json-schema-derive-2026-10-29`
  at `a71bd19905002d6a2b2b67624e3dc67cf6176aa0`):
  fails H04.
  Its docs support `#[serde(tag = "...")]` internally tagged enums only behind the `serde-compat` feature
  (`src/lib.rs`, doc comment lines 25 to 40), and `rg "untagged|adjacent|content" src/lib.rs` finds nothing,
  so untagged unions are absent.
  It also needs serde attributes, which conflicts with the intent of H02.
- `enum2schema` 0.1.2 (clone `enum2schema-2026-10-29` at `74a9a92f13f8e69c7638113468758898fd56aa6f`):
  fails H03.
  Its readme (lines 18 to 20 and 53) states it models serde's default externally tagged enums plus a `string_enum`
  mode, and returns a schema fragment.
  `rg "untagged|tag ="` over the readme finds no internal tag attribute.
- `herosal-toschema` 0.0.0 (downloaded crate source `crates/herosal-toschema-0.0.0`):
  fails H03 and H04.
  `rg` over the crate finds no `untagged`, no tag attribute, and no `$ref` or `$defs`.
  Version 0.0.0, 77 downloads.
- `ix-schema-jsonschema` 0.2.1 (crate source `crates/ix-schema-jsonschema-0.2.1`, 37 downloads):
  fails H03 and H04 (no tag, untagged, or `$ref` hits in source); it converts an `ix-schema` manifest, not arbitrary
  Rust types.
- `omni-schema` 0.1.1 and `omni-schema-derive` 0.1.1 (71 downloads):
  fails H09 provisionally.
  `gh repo clone ddoemonn/omni-schema` returns "Could not resolve to a Repository", so the published crate has no
  reachable tag.
  Crate source is in `crates/omni-schema-0.1.1` and `crates/omni-schema-derive-0.1.1` if the owner wants to
  confirm that no other repository exists.
- `mongo_json_schema` 0.2.0:
  fails H07; it emits MongoDB `$jsonSchema` (readme names draft 4).
- `jtd-derive` 0.1.4:
  fails H07 and category fit; it emits JSON Type Definition (RFC 8927), not JSON Schema, and the crate source has no
  `$ref` or `$defs`.
- `eure-macros` 0.3.2:
  category mismatch; its description is "Derive macros for Eure schema generation", a different schema language.
  Evidence is the crates.io description and a 284-character readme; the source was downloaded but not read further.

### Category mismatches by description only (not source-verified)

- Schema to Rust types, the reverse direction:
  `typify`, `progenitor`, `kopium`, `json_schema`, `jsonschema_code_generator`, `json-schema-rs`.
- Validators:
  `jsonschema` 0.60.2 (112,011,296 downloads), `boon`, `jsonschema-valid`.
  Note that `jsonschema` is relevant elsewhere: the golden test's corpus check needs a validator, see
  "Open questions".
- Inference from sample data, not from types:
  `schema2000`, `genson-rs`, `genson-core`, `json-schema-generator`, `infers-jsonschema`.
- OpenAPI frameworks and LLM or MCP tool-schema macros that sit on `schemars` or `utoipa`
  (`okapi`, `aide`, `rocket_okapi`, `kube-derive`, `rmcp`, `wovyr-tool-macros`, `vcad-tool-derive`, many others).
- `serde-reflection` 0.6.0: describes serde formats and needs `Deserialize`; it was judged from its description only
  and not opened.

### Custom implementation: hand-written schema emission

Ledgered as a candidate.
Under the skill's "Existing tools before custom implementation" invariant it is eligible only if every existing tool
fails a named hard constraint.
Because `schemars` has not failed any hard constraint, it cannot win and stays in the ledger only so the skill's
question is answered:
alternatives need screening, and the custom option is screened out by the invariant until a failure is proven.

### Drop the schema

Out of scope.
The user already chose to generate and golden-test it (see "Context facts").

## Execution manifest (not yet approved or run)

No third-party code was executed.
The only commands run were `curl` and `gh api` metadata fetches, `tar -xz` of the crates.io database dump
(data, not code), shallow `gh repo clone` calls (no build, no scripts), `python3 -I` over extracted CSV, and
`node` over local TypeScript fetch scripts.
No container was started and no cargo command was run.

Planned manifest for the first probe (to be completed and approved by the next agent before running):

- Image:
  `localhost/git-policy-native-mutation:w53` (Debian 13 trixie, cargo 1.97.0, rustc 1.97.0), image id
  `85ed7feb86d0`.
  Recorded deviations: it is not the repository's CachyOS base (`BXD`), not `nightly-2026-10-29`, and it was
  created by another session, so the image id must be re-read and its full digest recorded before use.
  Schemars MSRV 1.74 is lower than 1.97, so the toolchain satisfies it.
- Limits:
  `podman run --memory=2g --cpus=2 --rm`; no home mount; no repository mount; a private scratch volume with
  `CARGO_HOME` inside it.
- Phase one (network on): `cargo fetch` only, then `cargo tree --edges normal,build` and `Cargo.lock` checksums
  recorded; build scripts and proc-macros in the tree must be inspected before phase two.
- Phase two: `cargo test` and `cargo run` with `--network=none` and `--offline`.
- No stress tooling or repeated full test runs (`NLG`).

## Probe design (agreed in advance, not run)

A minimal crate pinned to `schemars = "=1.2.3"`, with `serde_json` only for printing and no direct `serde`
dependency, containing:

- a `Config` struct with a `$schema` string field (renamed through schemars' own attribute namespace), a `rules`
  list, and unknown keys rejected;
- a `Rule` enum internally tagged with `kind`, using `#[schemars(tag = "kind")]` rather than `#[serde]`;
- a `Value` untagged union of a string literal and a `ValueObject` default externally tagged enum with `File`,
  `Var`, and `Concat(Vec<Value>)` (recursion, to test `$ref` and `$defs`);
- a `schema` cargo feature gating the derive (`cfg_attr(feature = "schema", derive(JsonSchema))`), so the shipped
  crate does not depend on schemars or serde;
- the printed schema run twice in separate processes and diffed for determinism;
- a corpus of accepted and rejected configs run through a JSON Schema validator, to list every convention a
  hand-written decoder must mirror: `Option` null versus absent, integer `format` and bounds, unknown keys,
  enum discriminant requirements, literal versus tagged object in the untagged union.

The same types are to be re-expressed with each other surviving candidate's attribute system, so every finalist
gets equal-depth validation.

## Steps remaining, in the skill's order

1. Finish discovery: screen q03, q04, q05, q10, g01, g02, g03; append and freeze one expansion round from the
   recorded taxonomy terms; apply the two-consecutive-page no-new-survivor rule; record the terminal discovery
   result (saturated with N survivors, or blocked with `recommend none`).
2. Finish screening hard gates for `facet-json-schema`, `facet-jsonschema`, `utoipa`, `specta-jsonschema`
   (and decide whether the fork lineage and the two facet crates are one candidate or two).
3. Context and rubric freeze: confirm hard constraints H01 to H10, freeze criteria and weights.
4. Targeted evidence for every survivor, with `path:line` excerpts:
   licence text, manifests, transitive tree, lifecycle scripts and `build.rs`, proc-macro source for attribute parsing
   and enum emission, `Option` and integer emission, `$schema` handling, `preserve_order` and key order,
   tests and CI inventory, fuzzing and mutation search, source quality, human-auditability measurements
   (non-test lines, file count, runtime dependency count, same-author crates), and the maintenance audit
   (issues and pull requests updated in the last 12 months, sample method in the skill).
   Compare each crates.io package with its tag.
5. Execution manifest approval, then the two-phase podman probe from "Probe design" for `schemars`,
   and the equivalent probe for each other survivor.
6. Finalist validation: inventory upstream tasks and CI jobs, run the default CI-equivalent path in a bounded
   container, run relevant non-default suites, exercise platform claims, then the consumer boundary.
7. Editor evidence for H07: VS Code and JetBrains handling of the emitted dialect and of `$schema` in a `.jsonc`
   instance (still unverified in prior research).
8. Scoring, arithmetic, and the full one-at-a-time sensitivity matrix (a single finalist makes sensitivity
   trivially stable; state that instead of skipping it).
9. Pros and cons for every finalist, fully sorted ranking with adjacent reasons, evidence limits, and the
   recommendation or terminal result.
10. Completion checklist; final report update; no product, dependency, configuration, or decision mutation.

## Open questions for the owner

No question blocks the next step.
Two judgement calls were noticed and left open:

- Should a validator crate such as `jsonschema` 0.60.2 (a dev-dependency of the golden test) receive its own vet?
  The decision's gate list names only `toml_edit`, `browserslist-rs`, `schemars`, and an XML editor, but the
  accepted golden test needs a validator to check the decoder against the schema.
  Options: vet it in the same pass (ranked first, because the golden test depends on it), vet it in a separate
  report, or compare schema-versus-decoder agreement with a hand-rolled check in the test.
- Should schemars forks and the two facet crates count as separate candidates?
  Proposed: forks are the same candidate (not promoted), the two facet crates are one candidate
  (`facet-json-schema` is the newer one).

## Scratch paths, clones, and fixtures for reuse

Under `/var/home/user/temp/agent/`:

- `schemars-vet-2026-10-29/scripts/`:
  `crates-query.ts`, `gh-query.ts`, `crate-meta.ts`, `screen.ts`, `dump-enum.py`, `dump-screen.py`,
  `readme-probe.py`, `fingerprint.py`, `publish-report.ts` (atomic report write with the lock procedure).
- `schemars-vet-2026-10-29/discovery/`: raw query pages, `dump-json-schema.json`, `dump-screen.out`, `readmes.json`.
- `schemars-vet-2026-10-29/dbdump/2026-10-29-020020/`: extracted crates.io database dump files (about 3.4 GB;
  delete when discovery is complete).
- `schemars-vet-2026-10-29/crates/`: extracted source of `herosal-toschema`, `ix-schema-jsonschema`, `omni-schema`,
  `omni-schema-derive`, `mongo_json_schema`, `jtd-derive`, `eure-macros`.
- `schemars-vet-2026-10-29/fingerprint-input.json` and `draft-report.md`.
- Shallow clones, each `<name>-2026-10-29`:
  `schemars`, `facet-format`, `facet-jsonschema`, `utoipa`, `specta`, `rschema`, `json-schema-derive`, `enum2schema`,
  `schema-json-rs`.
- `technology-vet-locks/`: holds a lock belonging to another session; this report's lock was released.
