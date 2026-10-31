# Vet the XML editor for the Rust file-enforcer `user write` command

Superseded 2026-10-29 before a verdict:
the user chose an in-repo XML edit crate instead of a third-party editor
("For xml editor, write one ourselves like what we did for jsonc."),
recorded in `doc/decision/file-enforcer-rust-rewrite.md`, "Consequences".
This halted draft is kept whole as knowledge ("Even if work is superseded, it should still become knowledge.", user)
and as input to that crate:
the incumbent's exact edits and synthetic fixture shapes,
the screened alternatives for comparison,
and the open question whether LSP4IJ and Harper accept trailing commas in the embedded JSON,
since `monochromatic-jsonc-edit`'s emitter writes them and the incumbent writes strict JSON.

Status: in progress, halted 2026-10-29 for handoff.
The owner will continue this vet later with another model.
No recommendation exists yet;
the lifecycle phase reached is "Screened" for metadata only, and no candidate is a hard-gate-confirmed finalist.

## Status and fingerprint

- Lifecycle phase:
   discovery executed for crates.io and GitHub, partial for the broader web, screening started from metadata only.
- Subject:
   file-enforcer XML editor.
- Scope:
   choose the XML parse-and-edit approach the Rust file-enforcer's `user write` command uses to apply the Harper
   LSP4IJ policy to `LanguageServersSettings.xml` and `UserDefinedLanguageServerSettings.xml`
   under the newest `~/.config/JetBrains/<IntelliJIdea|IdeaIC><version>/options/`,
   whose attribute values embed JSON.
- Started and last updated:
   2026-10-29.
- Owner:
   delegated coding-agent session (halted); the next agent takes over.
- Governing skill:
   `.agents/skills/choosing-technology/SKILL.md` at commit `37117e3639`,
   SHA-256 `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`
   (the `.claude/skills` copy is file-enforcer output with the same hash).
- Compatibility fingerprint,
   RFC 8785 canonical form, schema version 1, built by
   `/var/home/user/temp/agent/xmlvet/scripts/fingerprint.ts`:
   `19b5e6d4d87bf1d5b2d3c67b35e089db81625f1624164fe208c1c1255ac15eac`.
- Prior compatible report:
   none.
  `rg --files` over `doc/audit` for `file-enforcer` found only `file-enforcer.md` (unrelated) and,
   written concurrently by another session, a TOML editor vet;
   the September research (`doc/planning/monorepo-manager-route-research/rust-structured-edits.md`)
   is design research, not a vet report.
- Report lock:
   `/var/home/user/temp/agent/technology-vet-locks/var-home-user-Monochromatic-doc-audit-tech-file-enforcer-xml-editor-vet-2026-10-29.md.lock`
   (a directory holding `owner.json`) was created with create-new semantics at 23:44:51Z and is released at halt.
  The next agent re-acquires it before editing this file.

## Context (measured, not asked)

- Decision gate:
   `doc/decision/file-enforcer-rust-rewrite.md` (accepted 2026-10-29) lists
   "choosing-technology vets for `toml_edit`, `browserslist-rs`, `schemars`, and an XML editor" before implementation.
- Consumer:
   the `user write` and `user check` subcommands carry the JetBrains Harper LSP4IJ policy
   (`doc/decision/file-enforcer-rust-rewrite.md`, "Outputs file-enforcer writes";
   `doc/planning/file-enforcer-rust-rewrite.md`, "JetBrains policy: keep user write", user choice 2026-10-29).
- Incumbent:
   TypeScript, `package/file-enforcer/core/src/pipeline/xml.ts` (484 lines) using `@lezer/xml` 1.0.6
   (`package/file-enforcer/core/node_modules/@lezer/xml/package.json:3`), `xml-coding.ts` (225 lines),
   `package/file-enforcer/core/src/jetbrains/lsp4ij-apply.ts` (406 lines), `lsp4ij-entries.ts` (238 lines),
   `lsp4ij.ts` (192 lines), policy at `file-enforcer.config.ts:1396-1464`.
- Exact edits the incumbent makes (read from the sources above):
  - Locate the base server in `UserDefinedLanguageServerSettings.xml`:
     first `entry` whose `templateId` option equals `harper-ls`, or `serverName` option equals
     `Harper Language Server`, or `commandLine` option contains `harper-ls`, skipping the managed scoped ids.
  - In `LanguageServersSettings.xml` replace the base entry (same key) with a rebuilt block whose
     `configurationContent` option holds the merged config JSON (`JSON.stringify` with two-space indent),
     `configurationSchemaContent` holds the schema JSON with default properties added only where absent,
     and `experimentalContent` and `initializationOptionsContent` are `{}`.
  - Replace or insert (before the last `</map>`) a scoped entry `harper-ls-agents-claude-md` in both files;
     in the user-defined file it copies named options from the base block and adds `serverId`, `serverName`
     and a `mappings` option listing file names.
  - Whole-entry replacement is the incumbent behavior;
     it drops comments inside a managed entry.
     The design doc (below) already chose attribute-level edits of owned options instead.
  - Text goes into attributes through `escapeXmlAttribute`: `& " < >` plus `&#10;`, `&#13;`, `&#9;`
     (`pipeline/xml-coding.ts:177`).
  - The incumbent tolerates malformed XML (lezer error recovery, unknown entities kept verbatim);
     the owner requirement is that malformed managed XML fails with a diagnostic.
- Synthetic fixture shapes to copy (never the owner's real files):
   `LANGUAGE_XML` and `USER_DEFINED_XML` in `package/file-enforcer/core/src/jetbrains/lsp4ij.unit.test.ts:24-62`,
   `SAMPLE_MAP` in `package/file-enforcer/core/src/pipeline/xml.unit.test.ts:23-41`.
- Prior design research:
   `doc/planning/monorepo-manager-from-scratch-design.md` "Managed file editing" (about lines 2495-2518) and
   `doc/planning/monorepo-manager-route-research/rust-structured-edits.md` sections "XML" (probe results),
   "XML options" (X1 to X6) and "XML" ranking: X1 roxmltree plus byte-range splicing (9 of 9),
   X2 quick-xml plus the same splicing (9 of 9), `xot`, `xmltree`, `xml-rs` fail the cases.
  The September probe harness is gone (its scratchpad is wiped), so those numbers are unreproduced here.
- Verified troubleshooting evidence:
   `doc/troubleshooting/xml-attribute-whitespace-serialization.md` (xot 0.31.2, xml-rs 1.4.0, xmltree 0.12.0
   write a literal tab or newline into attribute values, which reparses as a space).
- Requirement tension to carry forward:
   the task brief says formatting outside the edited elements must be preserved;
   the owner's 2026-09-17 answer in the design doc says formatting outside edited nodes may change and line
   endings may be normalized.
  The fingerprint uses the stricter brief wording.
  A tree rewriter that normalizes the whole file passes only the relaxed reading, so report both outcomes.
- Supply-chain policy for a new Rust dependency (`rust-catalog.toml`):
   `minimumReleaseAge = 1440` minutes, `minimumReleaseAgeStrict = true`, `trustPolicy = "no-downgrade"`,
   `trustPolicyIgnoreAfter = 525600`.
  Neither `roxmltree`, `quick-xml` nor `xmlparser` is in the catalog today
   (`rg` over `rust-catalog.toml` found only `zbus_xml` and `memchr`); a new entry arrives through `cargo add` and the sync.
- JSON leg (given by the task): `package/jsonc-edit/core-rust` (`monochromatic-jsonc-edit` 0.1.1, LGPL-3.0-or-later,
   no dependencies, canonical emit).

## Hard constraints (frozen)

- Comments outside the edited elements survive.
- Formatting outside the edited elements survives.
- Embedded JSON in attribute values is edited correctly with `monochromatic-jsonc-edit`
   and written back through correct attribute encoding (tab, newline, carriage return, quotes, `& < >`).
- Malformed managed XML is rejected with a positioned diagnostic.
- License compatible with LGPL-3.0-or-later, inspectable source, known build provenance.
- Rust crate dependency taken from `rust-catalog.toml` with `workspace = true`.

## Base components and overlays

- Base category: inspectable open-source local technology (the XML crate, or a repository-owned splice editor on one).
- Overlays: incumbent dependency replacement (`@lezer/xml` plus repository splicing);
   multi-platform claim (Linux x64, macOS arm64, Windows x64 for `user write`).
- Not active: high-trust execution (no credentials, no network, per-user IDE files only);
   native, Wasm, or prebuilt overlay applies only if a C-backed candidate such as `tree-sitter-xml` or `libxml`
   survives screening (it then needs source-to-artifact mapping).
- Sensitive-data overlay: not applicable;
   the files hold IDE settings, no credentials.
- Replacement-parity notes:
   the incumbent consumer paths are `listXmlEntries`, `findXmlEntryByKey`, `getXmlOptionValue`,
   `replaceOrInsertXmlEntry`, and `escapeXmlAttribute` / `unescapeXmlAttribute`.
  "Keep the incumbent as a candidate" is marked not applicable as a standalone option because the accepted decision
   moves file-enforcer to Rust; a Node subprocess for one rule contradicts it.
  Record this once more when scoring.
- Custom implementation: a repository-owned XML scanner is eligible only if every existing crate fails a named
   hard constraint; the `jsonc-edit` no-dependency decision is not precedent here.

## Frozen criteria and weights (not yet rated)

Derived from requirements, default weight 1 each, frozen before any rating:

- Diagnostic ownership: the parser, not repository code, rejects malformed XML and reports line and column.
- Source auditability and dependency surface (lines, dependencies, `unsafe`).
- Maintenance evidence (releases, maintainer action, merge latency, concentration).
- Repository glue the candidate forces us to own (well-formedness checks, entity decoding, range handling).
- API stability across recent releases.
- Edge-case correctness on the probe cases and upstream tests (entities, DTD, CRLF, BOM, quote styles, attribute order).
- Binary size increase (low priority, the prior research measured +40 KB to +77 KB for the two leaders).

Ratings, confidence, ranges, arithmetic and the sensitivity matrix are pending.

## Query schedule and discovery ledger

Scripts and raw output:
`/var/home/user/temp/agent/xmlvet/scripts/crates-discovery.ts`, `gh-discovery.ts`, `disc-summary.ts`,
`/var/home/user/temp/agent/xmlvet/crates-discovery.jsonl` (every page: query, sort, page, total, count, new candidates,
new survivors), `gh-discovery.jsonl`, `crates-raw-*.json` (one file per page), `crates-seen.json`, `gh-seen.json`.

### Source class 1: crates.io registry

Provider: `https://crates.io/api/v1/crates?q=<query>&per_page=100&page=<n>&sort=<sort>`, accessed 2026-10-29.
Screening heuristic (records what it can hide): a crate is a survivor when name, description or keywords contain
"xml" and one of: pars, dom, tree, edit, lossless, round-trip, preserv, cst, syntax, span, offset, position, token,
format, document, writer, mutat, modif, splice, rewrite, serial.
It hides crates whose metadata omits those words and it over-admits (372 of 3568 candidates passed), so every
survivor list needs manual review, which is partly done below.

Initial schedule, relevance sort unless noted, stop rule: provider exhausted, or two consecutive pages without a new
survivor, or provider cap:

- `xml`: total 5395; pages 1 to 10 read (1000 candidates); page 11 refused with HTTP 400
   "Cannot page beyond the first 1000 results when sorting by relevance".
  The cap blocked the relevance enumeration; the replacement enumeration `sort=recent-downloads` has no cap:
   pages 1 and 2 added no candidate and no survivor, satisfying the two-page rule on that ordering.
  Limit: low-download crates deeper than page 2 on that ordering are only reachable through the specific queries below.
- `xml parser`: total 1714; pages 1 to 10 read, same cap at page 11; `recent-downloads` pages 1 and 2 added nothing.
- `xml dom`: total 340, exhausted at page 4.
- `xml edit`: total 1243, six pages, stopped by the two-page rule at page 6.
- `lossless xml`: total 189, exhausted at page 2.
- `xml cst`: total 23, exhausted.
- `xml round-trip`: total 483, exhausted at page 5.
- `xml span`: total 441, two-page rule at page 4.
- `xml tokenizer`: total 1251, two-page rule at page 4.
- `xml preserve comments`: total 270, exhausted at page 3.
- `xml tree`: total 1409, two-page rule at page 4.
- `xml formatter`: total 144, exhausted at page 2.
- `xml syntax tree`: total 355, two-page rule at page 3.
- `xml document`: total 3093, two-page rule at page 9.
- `roxmltree alternative`: total 11, exhausted.
- `quick-xml alternative`: total 50, exhausted.
- `xml rowan`: total 1, exhausted.
- `tree-sitter xml`: total 218, two-page rule at page 2.
- `intellij settings xml`: total 10, exhausted.
- `xml splice`: total 56, exhausted.
- `xml position`: total 816, two-page rule at page 7.

Expansion round (one de-duplicated round from taxonomy terms found: pull, streaming, lenient, forgiving, tolerant,
canonicalization, SAX, XHTML, markup, OOXML, attribute), then frozen:

- `pull parser xml` (433, exhausted p5), `streaming xml` (1381, two-page rule p5), `lenient xml` (73, exhausted),
   `forgiving xml` (7, exhausted), `tolerant xml parser` (173, exhausted p2), `xml canonicalization` (140, exhausted p2),
   `position preserving xml edit` (194, exhausted p2), `xml sax` (41, exhausted), `xhtml parser` (43, exhausted),
   `markup formatter` (94, exhausted), `ooxml xml tree` (38, exhausted), `xml attribute edit` (403, two-page rule p4),
   `xml lossless syntax tree rowan` (1, exhausted).
- Totals after all queries: 3568 distinct crates seen, 372 heuristic survivors.

Terminal result for this source class: saturated, with the stated limit on the two capped broad queries.

### Source class 2: GitHub repository search

Provider: `gh api -X GET search/repositories`, sort `stars`, 100 per page, accessed 2026-10-29
(provider cap is 1000 results per query; none reached).

- `topic:xml language:Rust`: total 209, three pages, exhausted.
- `topic:xml-parser language:Rust`: total 33, exhausted.
- `topic:xml-parsing language:Rust`: total 7, exhausted.
- `xml lossless language:Rust`: total 5, exhausted.
- `xml round-trip comments language:Rust`: total 0.
- `xml editor language:Rust`: total 11, exhausted.
- `xml dom language:Rust`: total 18, exhausted.
- `xml cst language:Rust`: total 1, exhausted.
- 245 distinct repositories seen. New names beyond the crates.io set worth a look:
   `aharjula/position_preserving_moodle_question_xml_edit`, `johnstonskj/rust-xml_dom`, `Ygg01/xml-air`,
   `sybila/biodivine-lib-xml-dom`, `fatihpense/rust_xml_oxide`, `meduketto/iksemel-rust`, `ArtemGr/vtd_xml.rs`,
   `Anton293/microdom`, `thetek42/nanoxml`, `gwenn/xmlreader`, `happenslol/novaxml`, `KWARC/rust-libxml`.
  Not yet screened.
- Not run: GitHub code search for `buffer_position` / `range_value` splice editors; releases and organization projects.

Terminal result for this source class: saturated for the repository-search queries above;
code, releases and organization-project searches remain.

### Source class 3: broader web

- `WebSearch` (standard) ran four queries:
   "Rust XML library edit file preserving comments and formatting lossless round trip crate";
   "roxmltree alternative Rust XML parser byte ranges spans attribute value";
   "edit XML attribute in place without reformatting Rust quick-xml buffer_position splice";
   "xmlparser crate StrSpan token Attribute value span RazrFalcon".
  They surfaced no new candidate beyond the registry set (`xmltok`, `xhtml_parser`, `oxml`, `aloe-xml`, `use-xml`
   appeared; the first four exist in the crates.io ledger; `aloe-xml` and `use-xml` are new and unscreened).
- `mcp__linkup__linkup-search` returned "Unauthorized action" twice; Linkup is unavailable this session.
  Not blocking: WebSearch worked.
- Remaining: `lib.rs` category browse (`https://lib.rs/parser-implementations`, `https://lib.rs/keywords/xml`) and
   a search for editors outside crates.io (for example Zed or Helix XML tooling, dprint XML plugins).

### Source class 4: this repository

Done so far: the incumbent, the design doc, `rust-structured-edits.md`, the troubleshooting doc,
`doc/decision/file-enforcer-rust-rewrite.md`, `doc/planning/file-enforcer-rust-rewrite.md`.
Not yet done: `rg` over every `Cargo.lock` and `Cargo.toml` for `roxmltree`, `quick-xml`, `xmlparser`, `xot`;
`rg` over `doc/audit` and `doc/decision` for other XML vets (a quick listing showed none named for XML);
check `package/test-fixture` for XML corpora.
The September research states that no tracked crate depends directly on `quick-xml` or `roxmltree`, and that
`package/music-player/desktop-app/Cargo.lock` locks both transitively.

## Candidate ledger (screening, metadata only)

Metadata fetched 2026-10-29 from the crates.io API by `scripts/meta.ts`, output in
`/var/home/user/temp/agent/xmlvet/meta-1.json`.
"Past year" counts non-yanked releases since 2025-10-29.
Every row below is metadata evidence only; no source has been read in this session and no clone exists yet.

### `roxmltree` 0.21.2

- Discovery: crates.io `xml`, `roxmltree alternative`; prior in-repo X1.
- Base: inspectable open-source local technology; multi-platform overlay.
- Metadata: MIT OR Apache-2.0; rust-version 1.60; edition 2021; one dependency (`memchr`); published
   2026-10-24 by `adamreichold`; 33.5 million recent downloads; 35 releases in total, one in the past year;
   repository `https://github.com/RazrFalcon/roxmltree`.
- Change since the September research: 0.21.2 is new (the research probed 0.21.1).
  The release is five days old, so it passes `minimumReleaseAge = 1440` minutes;
   read the 0.21.1 to 0.21.2 diff and the `range_value` defect noted below before trusting the September probe.
- Screening result: survivor, plausible fit, pending targeted evidence.
  September findings to re-verify: `Node::range` and `Attribute::range_value` give byte ranges;
   `range_value` is wrong for qualified names of `u16::MAX` bytes or more, or more than `u8::MAX` spaces around `=`
   (`roxmltree-0.21.1/src/lib.rs:620-621`); well-formedness, entities and DTD are the parser's job with line and
   column errors; `allow_dtd` must be on to accept a DOCTYPE.

### `quick-xml` 0.42.0

- Discovery: crates.io `xml`, `xml parser`; prior in-repo X2.
- Metadata: MIT; rust-version 1.86; edition 2024; one non-optional dependency (`memchr`); published 2026-08-22 by
   `dralley`; 154.7 million recent downloads; ten releases in the past year; repository `tafia/quick-xml`.
- Screening result: survivor, pending targeted evidence.
  September findings to re-verify: repository code must add unclosed-element, single-root, text-outside-root,
   unknown-entity (`Event::GeneralRef`) and DOCTYPE entity checks; attribute error positions are relative to the
   attribute value; the attribute API changed between 0.41 and 0.42; value byte offsets rely on borrowed slices from
   `Reader::from_str`.

### `xmlparser` 0.13.6

- Discovery: crates.io `xml parser` (roxmltree's tokenizer; new to the ledger, not in the September probe set).
- Metadata: MIT/Apache-2.0; zero dependencies; edition 2018; published 2023-09-30 by `RazrFalcon`;
   46.2 million recent downloads; no release in the past year.
- Screening result: survivor.
  It tokenizes with spans (`Token::Attribute` carries `prefix`, `local`, `value`, `span` as `StrSpan`, per the docs
   search result) but does not check well-formedness across tokens (a repository checker would own that);
   needs targeted evidence on the span offsets, entity handling and error positions.
  No release in the past year is low-signal for a stable tokenizer, not abandonment proof.

### `xot` 0.31.2, `xmltree` 0.12.0, `xml` 1.4.0 and `xml-rs` 0.8.29

- `xot`: MIT, six dependencies, published 2025-04-09, no release in the past year.
- `xmltree`: MIT, depends on `xml`, published 2025-11-10.
- `xml` 1.4.0 (published 2026-08-06) and `xml-rs` 0.8.29 (same repository `kornelski/xml-rs`, published the same day)
   are the renamed crate and its legacy name.
- Screening evidence so far:
   `doc/troubleshooting/xml-attribute-whitespace-serialization.md` (verified 2026-09-17 in the newest releases then)
   records literal tab or newline written into attribute values, whole-document rewriting (quote style, empty-element
   spelling, dropped whitespace text, attribute order), and for `xot` DOCTYPE rejection.
- Proposed result: hard-gate exit on the stated constraint "formatting outside the edited elements survives" and on
   correct attribute encoding, pending a re-check that no newer release changed this
   (`xot` has no newer release; `xml` and `xml-rs` released 2026-08-06, after the September probe, and need a diff of
   the writer's attribute escaping; `xmltree` 0.12.0 predates the troubleshooting doc).
  Do not finalize the exit until that re-check is recorded.

### `rxml` 0.14.0, `minidom` 0.19.0, `xml5ever` 0.40.0, `sxd-document` 0.3.2

- `rxml`: MIT, published 2026-02-21, strict XML 1.0 parser without DTD features, streaming; `minidom` builds a DOM on it
   (MPL-2.0).
- `xml5ever`: MIT OR Apache-2.0, depends on `markup5ever`; tree builders discard source layout.
- `sxd-document`: MIT, last release 2019.
- Screening result: category mismatch pending confirmation. Each is a parser or tree that reserializes; none is known
   to give attribute byte ranges.
  Check whether `rxml` exposes positions suited to splicing before exiting it.

### `uppsala` 0.12.0, `twig-doc` 7.0.1, `xmloxide` 0.5.0

- September status was "screened, not probed, Unverified"; Unverified is not a hard-gate failure, so these need targeted
   evidence before exit.
- `uppsala`: BSD-2-Clause, no dependencies, 18 releases (all in the past year), trusted-publishing data present
   (GitHub), 405 KB crate; a DOM with XPath and XSD validation, formatting fidelity unknown.
- `twig-doc`: MIT OR Apache-2.0, rust-version 1.88, 46 releases in the past year, depends on `twig-kit`; claims lossless
   round trips of documents in registered languages; whether an XML language exists is unknown.
- `xmloxide`: MIT, rust-version 1.81, 13 releases, optional `roxmltree`, `quick-xml` and `libxml` features,
   a reimplementation of libxml2; formatting fidelity unknown.

### Others screened from metadata

- `tree-sitter-xml` 0.7.0: MIT, C grammar through `tree-sitter-language`; error-tolerant, native overlay applies
   (source-to-artifact mapping, build scripts); contradicts the strict-rejection requirement unless error nodes are
   checked by repository code; not yet targeted.
- `xml-doc` 0.2.0 (2021), `edit-xml` 0.1.0 (2024, 4.1 MB crate): quick-xml-based trees; rewrite on save; pending.
- `draviavemal-xml_rs` 1.0.4-alpha.12: AGPL-3.0-only; license gate fails against LGPL-3.0-or-later distribution
   (needs the exact incompatibility reasoning recorded); alpha.
- `docxtpl-xml` 1.3.2: LGPL-2.1-only; "editable XML fragment layer with lenient parsing, preserving serialization and
   position tracking" per its description; the license is not "or later", so the combination with an
   LGPL-3.0-or-later work needs a license-compatibility check before any exit or promotion; seven releases, one author.
- `position_preserving_moodle_question_xml_edit` 0.1.2: MIT, built on `roxmltree`, domain-specific (Moodle question
   files), 37 recent downloads; useful as a prior-art reference for roxmltree-based splicing, not a general candidate.
- `oak-xml` 0.0.11 (MPL-2.0, pre-1.0 incremental parser in a language-framework workspace), `xmltok` 0.14.0 (fork of
   `xmlparser`, 390 recent downloads), `xtoken` 0.1.1 (2023), `xml-syntax-reader` 0.2.0 (callback tokenizer, 8 recent
   downloads): tokenizers with tiny adoption; screen after the main finalists, no gate evidence yet.
- Formatters (`markup_fmt`, `lax-markup`, `tidier`, `xmlformat`, `xml_magic`): category mismatch (they reformat whole
   documents), pending a one-line confirmation each.
- Domain crates from the survivor list (`openbim-*`, `docxml`, OOXML families, `rbx_xml`, `evtx`, RDF crates):
   category mismatch for a generic IDE-settings editor.
- C-backed `libxml` and `expat-sys`: native overlay; contradict the pure-Rust, no-C-toolchain preference recorded in
   the September research ("pure Rust and built statically without a C compiler"); pending a formal gate record.

## Findings so far that affect the report body

- The JSON leg needs a boundary check before the XML ranking is final.
  `package/jsonc-edit/core-rust/src/emit.rs` pushes a trailing comma after every array element
   (`out.push(',')` in the array branch of `emit_bare`); the record branch was only partly read, and the September
   research records that the TypeScript sibling emits trailing commas for records
   (`JE/stringify.unit.test.ts:100`, as cited in `rust-structured-edits.md`).
  The crate README describes the output as canonical rather than byte-preserving JSONC, not as strict JSON.
  The incumbent writes strict JSON (`JSON.stringify(value, null, 2)`, `pipeline/json.ts` `formatJsonObject`).
  Whether LSP4IJ's configuration reader and Harper accept trailing commas inside `configurationContent` is unverified.
  Next step: read the reader in `redhat-developer/lsp4ij` with `gh api` (EXT rule), and run `emit_jsonc_value` on a
   probe object to confirm the record branch.
  If trailing commas are rejected, the JSON leg needs a strict emit path (a mode on the crate, or `serde_json`
   which is in `rust-catalog.toml`), which does not change the XML ranking but is material for "edit the embedded
   JSON correctly".
- An embedded JSON string never contains a raw tab or newline (JSON requires escapes), so attribute-value
   normalization only affects whitespace between tokens in `configurationContent`; it does matter for
   `commandLine`, `serverName` and other copied options, and the `&#10;`, `&#13;`, `&#9;` encoding must stay.
- The incumbent's lenient unescape keeps unknown entities (`&nbsp;`) verbatim; a strict parser rejects them.
  Real IntelliJ output is not expected to contain them, which is a reason the strict requirement is low risk.
- Environment for validation: `localhost/monochromatic-lint-test:development` (Debian 13, rustc 1.97.0,
   cargo 1.97.0, x86_64) runs Rust; the digest has not been recorded yet.
  `quick-xml` 0.42.0 needs rust-version 1.86 and `twig-doc` 1.88, both below 1.97.
  The repository's BXD rule prefers a CachyOS base; `localhost/file-enforcer-perf:latest` and
   `docker.io/cachyos/cachyos:latest` lack Rust, so any probe image deviation (Debian) must be recorded.

## Execution manifests and validation

None executed.
Only read-only commands, the crates.io and GitHub APIs, and `podman run --rm --memory=512m --cpus=1` image probes
(`cargo --version`, OS release) ran; no third-party crate code was built or tested.

## Hard-gate exits, validation results, scoring, sensitivity, pros and cons, ranking, recommendation

Not reached.
Prior-research ranking, for reference only and not a result of this vet: X1 `roxmltree` > X2 `quick-xml` > X3 > `twig-doc`
 > `uppsala` > `xmloxide` > X4 `xot` > X5 `xml-rs` > X6 `xmltree`.

## Steps remaining, in the skill's order

1. Finish context and freeze: add the screening-gate list to the report and confirm the criteria (done in draft above);
    refreeze nothing after ratings begin.
2. Finish discovery:
   - `rg` this repository (`Cargo.lock`, `Cargo.toml`, `doc/audit`, `doc/decision`) for XML crates and XML vets.
   - GitHub code, release and organization queries; `lib.rs` keyword and category browse.
   - Manually review the 372 heuristic survivors (list: `node scripts/survivor-list.ts`, `node scripts/top-xml.ts <regex> <n>`
      in `/var/home/user/temp/agent/xmlvet`) and the unscreened GitHub names; record terminal discovery result.
3. Screening hard gates for every survivor with evidence: license (record the `docxtpl-xml` LGPL-2.1-only and
   `draviavemal-xml_rs` AGPL reasoning), category fit, pure-Rust and provenance, platform support, source inspectability.
   Re-check `xml`, `xml-rs`, `xmltree`, `xot` against the troubleshooting doc before exiting them.
4. Clone every serious alternative and finalist under `/var/home/user/temp/agent/<name>-2026-10-29` with
   `gh repo clone <repo> <path> -- --depth 1`; likely finalists: `roxmltree`, `quick-xml`, `xmlparser`, plus whichever of
   `uppsala`, `twig-doc`, `xmloxide`, `rxml`, `tree-sitter-xml` survive targeted evidence.
5. Targeted source audit per finalist: license text, manifest, transitive tree, `build.rs`, `unsafe`, range and position
   API, entity and DTD handling, error types, tests, CI matrix, fuzz and mutation search, maintenance sample (issues and
   pull requests updated in the last 12 months, all if 20 or fewer else the ten most recent; maintainer comments versus
   actions; merge latency; release cadence; maintainer concentration), human-auditability measurements.
6. Execution manifests, then run each finalist's upstream default `cargo test` and relevant non-default suites in
   `podman run --memory=2g --cpus=2 --rm` (fetch phase with network, then `--network=none --offline`), reading-only repository
   mount, private scratch volume; record image digest, commands, exit status, elapsed time.
7. Consumer-boundary probe (rebuild; the September harness is gone) under `/var/home/user/temp/agent`:
   one crate with each finalist behind the same splice editor and `monochromatic-jsonc-edit` as a path dependency.
   Fixtures: the TypeScript `LANGUAGE_XML`, `USER_DEFINED_XML`, `SAMPLE_MAP`, plus synthetic variants with comments inside
   and between entries, single-quoted attributes, attribute order swap, CRLF, a DOCTYPE with an internal entity, an unknown
   entity, an unclosed `</entry>`, and multi-line `configurationContent`.
   Cases: attribute-level set of owned options, whole-entry replace, insert before the last `</map>`, second-run idempotency,
   malformed rejection with position, output re-parse by the same parser, tab and newline encoded as `&#9;` and `&#10;`.
   Differential run against the built TypeScript incumbent
   (`package/file-enforcer/core/dist/final/node/index.mjs` exports `updatedLanguageSettingsXml` and friends) on the same
   synthetic strings, comparing decoded values.
   Platform checks: Linux x64 locally; try `cargo check --target aarch64-apple-darwin` and `x86_64-pc-windows-msvc` in the
   container after a separately inspected `rustup target add`; record the macOS and Windows runtime omission with upstream
   CI matrix evidence if not run.
8. Verify the JSON boundary (trailing commas, LSP4IJ reader) as described in the findings.
9. Score the validated finalists on the frozen criteria, run the full one-at-a-time sensitivity matrix, write pros and
   cons per finalist, the fully sorted ranking with adjacent reasons, evidence limits, and the recommendation or the
   terminal outcome; end with an open question only if a tie or the JSON boundary needs an owner decision.
10. Update this file atomically after each phase (lock, temp sibling, hash re-check, rename), release the lock, and do not
    commit until the owner reviews.

## Open question

None recorded yet.
Possible candidate once evidence exists: if LSP4IJ rejects trailing commas, whether to add a strict-JSON emit mode to
`monochromatic-jsonc-edit` or use `serde_json` for the embedded configuration JSON.

## Scratch paths created

- `/var/home/user/temp/agent/xmlvet/` with `scripts/` (`fingerprint.ts`, `crates-discovery.ts`, `gh-discovery.ts`,
  `disc-summary.ts`, `survivor-list.ts`, `top-xml.ts`, `meta.ts`), `crates-discovery.jsonl`, `gh-discovery.jsonl`,
  `crates-seen.json`, `crates-seen.json.surv`, `gh-seen.json`, `meta-1.json`, and 112 `crates-raw-*.json` page dumps.
- `/var/home/user/temp/agent/technology-vet-locks/` (created by this session; holds a sibling TOML vet lock owned by
  another session, which must be left alone).
- No clones, no probe crates, no containers left running.
