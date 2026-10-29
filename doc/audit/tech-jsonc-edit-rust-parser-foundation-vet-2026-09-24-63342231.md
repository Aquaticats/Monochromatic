# Vet the parser foundation for a native Rust jsonc-edit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/jsonc-edit.conformance` is now `package/jsonc-edit/core-ts.conformance`;
> `package/rust-module/jsonc-edit` is now `package/jsonc-edit/core-rust`;
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

## Status and fingerprint

- Status:
   discovery saturated for the frozen schedule,
   both survivors validated at equal depth,
   scored with sensitivity,
   and a recommendation recorded.
   **Not adopted**:
   adoption is a separate user decision,
   and no product code,
   dependency,
   decision record or publication followed.
- Subject:
   jsonc-edit Rust parser foundation.
- Scope:
   Choose a parser and token source for the published native Rust comment-as-data JSONC editor.
- Started and last updated:
   2026-09-24.
- Owner:
   current coding-agent session.
- Governing skill:
   `.agents/skills/choosing-technology/SKILL.md` at `a05818ad7`,
   SHA-256 `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.
- Fingerprint,
   RFC 8785 canonicalized schema version 1:
   `63342231cc8d34fcdb2effc92f9c276181ddde482f704a397f3eb0b25fe8cf26`.
- No compatible prior parser-foundation report was found by a `rg --files doc/audit` search for JSONC or Rust parser.
- Category:
   inspectable open-source local technology,
   including a repository-owned translation as baseline.
   Overlays:
   native,
   multi-platform and untrusted parsing input.
   No browser,
   credential,
   or CI-execution overlay for the library itself.
   The maintained TypeScript library is a behavior reference,
   not removed.
- Hard constraints:
   strict JSONC-only container roots,
   original numeric and quoted-string lexemes,
   escaped unpaired UTF-16 surrogate support,
   separately queryable normalized comments on keys and values after adaptation,
   inspectable source and reproducible validation,
   compatible licensing,
   and no regex-backed production parsing or lexing in required dependencies.
- Deployment:
   native Rust crate on Linux,
   macOS and Windows,
   crates.io release.
- Criteria,
   frozen before candidate rating with default weight 1 each:
   source auditability,
   API clarity for the comment model,
   dependency and build surface,
   maintenance evidence,
   and measured parse performance.
   Ratings,
   totals,
   sensitivity checks and the full ranking are recorded in the scoring section.

## Discovery schedule and limits

The delegated research froze its literal initial queries before discovery:

- In-repo:
   `rg --files doc package | rg 'rust-structured-edits|jsonc|json-five|biome_json|edikt'` and
   `rg --line-number 'jsonc-parser|edikt-jsonc|json-five|biome_json_parser|attached.comment|surrogate' doc package`.
- Registry/web:
   `site:crates.io/crates/ jsonc-parser edikt-jsonc json-five biome_json_parser latest version repository` and
   `Rust JSONC parser lossless comments tokens raw string escapes exact numeric literals crates source` via Exa
   fast/Linkup fallback.
- Repository host:
   targeted official source linked from crates and repository decision records;
   broad GitHub query and registry pagination remain to be run.
   One de-duplicated expansion round must be frozen and recorded from taxonomy terms before further discovery.

Both local delegated `rg` queries were blocked by its security guardrail,
 so targeted reads were used instead.
 No class is saturated;
 counts and page cursors are still needed.
 The browser/page source findings here are leads,
 not hard-gate confirmations until pinned repository source is read and behavior probed.

## Candidate ledger

### Repository-owned parser translation

- Discovery:
   `package/module/jsonc-edit/src/parse.ts`,
   `src/scan.ts`,
   `src/parse-trivia.ts` and accepted `doc/decision/jsonc-edit-parser-foundation.md`.
- Category:
   inspectable repository-owned implementation,
   native and multi-platform overlays.
- Screening:
   plausible semantic fit.
   Rust UTF-16 values and exact numeric model differ from TypeScript,
   so direct translation needs design and verification;
   not automatically recommended.

### `jsonc-parser` 0.33.2

- Discovery:
   [versioned source and API](https://docs.rs/jsonc-parser/0.33.2/jsonc_parser/) and prior in-repo
   [Rust edit survey](../planning/monorepo-manager-route-research/rust-structured-edits.md).
- Category:
   inspectable open-source Rust parser with native and multi-platform overlays.
- Screening:
   source at private clone `~/temp/agent/jsonc-parser-2026-09-16`,
   revision `e6e3837`,
   `src/scanner.rs:175-201` decodes escaped strings during scanning;
   `src/string.rs:214-261` rejects unpaired high and low surrogates.
   It cannot parse the required accepted string domain as-is.
   An upstream fork modifying its string scanner would be a distinct custom candidate.
   It exposes comments and raw number tokens,
   but normalized key/value attachment and canonical emission would still be repository code.
- Adapter boundary:
   `src/scanner.rs:175-201` produces `Token::String(Cow<str>)` through the escape-decoding path;
   `src/tokens.rs:8-22` has no raw-string token variant,
   and `src/cst/mod.rs:1146-1160` calls `parse_to_ast` before CST construction.
   Thus the public AST and CST paths cannot merely bypass the rejecting decoder for a surrogate escape.
   A source-preserving surrogate prepass plus an owned UTF-16 decoder is a separate **unvalidated** adapter hypothesis,
   not a property of the upstream crate.
   `src/parse_to_ast.rs:285-309` recursively enters array and object parsing,
   and `src/cst/mod.rs:2541-2576` recursively builds CST containers;
   any adapter would also need a bounded accepted-depth lifecycle probe.

### `edikt-jsonc` 0.4.0

- Discovery:
   [crates.io entry](https://crates.io/crates/edikt-jsonc) and
   [lexer source](https://docs.rs/crate/edikt-jsonc/0.4.0/source/src/lexer.rs).
- Category:
   inspectable open-source Rust lexer/CST with native and multi-platform overlays.
- Screening:
   raw comments,
   strings,
   and numbers are available.
   [Comment projection](https://docs.rs/crate/edikt-jsonc/0.4.0/source/src/comments.rs) puts key-side and value-side
   comments together on the value,
   so an attachment layer is still needed.
   [Value projection](https://docs.rs/crate/edikt-jsonc/0.4.0/source/src/project.rs) uses `f64` for some numbers and
   Rust `String`;
   bypassing it may allow a compliant projection from raw tokens.
   JSON5 and malformed-input acceptance requires strict validation.
   Excluded at the regex hard gate:
   `lexer.rs:47-69` uses Logos `#[regex(...)]` patterns in production.
   Stop candidate validation.

### `json-five` 0.3.1

- Discovery:
   [round-trip token source](https://docs.rs/crate/json-five/0.3.1/source/src/rt/tokenize.rs) and
   [parser source](https://docs.rs/crate/json-five/0.3.1/source/src/rt/parser.rs).
- Category:
   inspectable open-source Rust parser with native and multi-platform overlays.
- Screening:
   raw scalar spelling and escaped unpaired-surrogate **source tokens** survive without a block comment,
   but JSON5 grammar and scalar roots need strict validation.
   The published round-trip tokenizer shortens every block-comment span by one byte
   (`src/tokenize.rs:529-532`),
   and the owned-token API shifts following lexemes (`src/rt/tokenize.rs:62-156`).
   An independent bounded [consumer control](../troubleshooting/json-five-unterminated-block-comment.md)
   confirmed both a clean no-block-comment round trip and corruption after inserting one multi-line block comment.
   A canonical emitter alone cannot repair incorrect comment ownership from this as-is token stream;
   a source fork or independently checked span repair would be a separate custom candidate.
   Decoded UTF-16 value behavior and 512-level lifecycle remain unverified.

### `biome_json_parser` 0.5.7

- Discovery:
   [lexer source](https://docs.rs/crate/biome_json_parser/0.5.7/source/src/lexer/mod.rs) and
   [parser source](https://docs.rs/crate/biome_json_parser/0.5.7/source/src/lib.rs).
- Category:
   inspectable open-source Rust parser with native and multi-platform overlays.
- Screening:
   lossless trivia and raw scalar ranges may support an adapter.
   [Prior repository research](../planning/monorepo-manager-route-research/rust-structured-edits.md) corrected its
   build pin count to three exact companion crates.
   Strict container-root validation and the attached-comment layer remain repository work.
   The published `src/lexer/mod.rs:494-607,635-683` scans an escaped `\uXXXX` as four hex digits without decoding it
   into a Rust `char`;
   `src/syntax.rs:151-230` parses nested sequences with an explicit `Vec` stack.
   `src/parser.rs:16-29` exposes comment and trailing-comma options.
   This source shape is a promising raw-syntax adapter,
   but runtime acceptance of lone surrogates,
   raw-node projection,
   deep tree lifecycle and full validation remain unproven.
   `cargo info biome_json_parser` resolved published 0.5.7 on 2026-09-24,
   while the shallow upstream checkout at `2f629c5` has `version = "0.7.0"` in root `Cargo.toml:7`.
   The [crates.io version record](https://crates.io/api/v1/crates/biome_json_parser) dates the latest published 0.5.7
   to 2024-03-12.
   The separate GitHub release `@biomejs/biome@2.5.14` was published 2026-09-16 according to the repository releases
   API;
   that application release is not a publication of this Rust crate.
   A GitHub commits API query filtered to `crates/biome_json_parser` since 2025-09-24 returned 18 path-touching commits,
   including `chore(tooling): publish crates` at `9dcc25547` on 2026-09-19.
   `RELEASES.md:33-39` in upstream checkout `2f629c5` explicitly says internal crates publish on demand,
   rather than alongside every application release.
   `CONTRIBUTING.md:542-547` describes manual workflow dispatch and an automated release PR;
   `.github/workflows/publish-crates.yml:1-19,72-152` publishes on merger of that PR.
   A repository PR query showed no open `automated-release/crates` branch at this check.
   The 2024 crates.io version alone therefore does not prove the parser source is abandoned,
   but the newer checked-out source is not the published dependency being evaluated.
   Maintainer response history and platform suites still need inspection.

### `jwc` 0.1.0

- Discovery:
   [AST source](https://docs.rs/crate/jwc/0.1.0/source/src/ast.rs) and
   [parser source](https://docs.rs/crate/jwc/0.1.0/source/src/single_pass_parser.rs).
- Category:
   inspectable open-source Rust parser with native and multi-platform overlays.
- Screening:
   key and value trivia are distinguished,
   but its parser rejects unpaired surrogate escapes according to source in the delegated research.
   It cannot parse the required domain as-is.
   Numeric values use approximate `f64` beyond bounded integers,
   so an exact adapter would also be needed.

## Biome maintenance snapshot

The package-specific registry record dates `biome_json_parser` 0.5.7 to 2024-03-12.
 The current upstream `RELEASES.md:33-39` says internal Rust crates publish on demand;
 active application releases do not imply a newer published library.
 A GitHub path-filtered commits query returned 18 changes to `crates/biome_json_parser`
 since 2025-09-24,
 including repository maintenance and an embedded-JSON parser feature.

The literal GitHub issue query `json parser` with repository and updated-since-2025-09-24 filters returned 16 issues.
 Their titles and opening body text were screened for parser relevance;
 relevant threads and their comments were then read in full.
 a separate title-only `JSONC` query returned two issues.
 Relevant threads show different boundaries:

- [#11504](https://github.com/biomejs/biome/issues/11504) concerned application configuration discovery,
   not direct parser value projection.
   A maintainer confirmed it on 2026-09-23,
   assigned it,
   and merged [#11910](https://github.com/biomejs/biome/pull/11910) the same day;
   that PR had a maintainer approval and CLI/service regression tests.
- [#8451](https://github.com/biomejs/biome/issues/8451) concerned JSONC settings-file configuration.
   A maintainer asked for a reduced reproduction,
   traced the final case to another tool's override configuration,
   and the reporter agreed.
- [#11426](https://github.com/biomejs/biome/issues/11426) combined JSON recovery diagnostics with editor refresh
- behavior.
   A maintainer responded,
   but the issue was automatically closed for lack of the requested reproduction;
   it does not prove a fix to the published parser.
- [#7728](https://github.com/biomejs/biome/issues/7728) remains open and confirmed for a malformed JSON file causing a
- Biome worker stack overflow.
   A contributor suggested recursive syntax-node destruction as a theory,
   not a verified root cause.
   The reported Biome application version and worker boundary differ from the published 0.5.7 parser probe;
   the bounded successful 512-depth cases do not resolve that incident.
- [#10684](https://github.com/biomejs/biome/issues/10684) reported a formatter worker overflow around 2000 nested
- arrays.
   The issue was closed with a contributor response rather than a source fix in that thread.
   This formatter boundary is distinct from the library parser and this port's emitter;
   do not transfer either the failure or the closure to those surfaces.

The [crate-publishing PR #11850](https://github.com/biomejs/biome/pull/11850) was approved and merged on 2026-09-19;
 its workflow opens a version-bump PR and publishes only after that PR merges.
 No open `automated-release/crates` PR appeared in the recorded query.
 The issue and PR samples establish active upstream code and some responsive investigation,
 but do not yet supply an equal-depth upstream CI,
 platform,
 or package-release validation for version 0.5.7.
 No soft maintenance rating is assigned yet.

### Malformed external fixture control

The public [issue #7728](https://github.com/biomejs/biome/issues/7728) reports a worker stack overflow on malformed
JSON in a Biome 2.2.5 application run.
 The reporter's fixture repository `gc/biome-crash` was cloned read-only at `bea9583`;
 `src/evil.json` is 665123 bytes of log-like invalid JSON,
 SHA-256 `8490581e9b29f474beedf09c72a9930d0103a694bc430a8408828c53a4340109`.
 The issue's recursive-drop explanation is a contributor hypothesis,
 not a confirmed source diagnosis.
 The revised `~/temp/agent/biome-execution-manifest.md` recorded the read-only fixture mount and stop conditions before
 execution.
 Bounded debug and optimized release runs of the published 0.5.7 parser library both reported syntax errors on the
 input,
 traversed 236856 syntax tokens,
 and dropped the syntax tree without an abort.
 These positive cleanup results are specific to the direct library path and that one fixture;
 they do not establish a cause or resolution for the newer Biome application CLI/worker incident.
 A second fixture test calls the scratch `parse_for_editor` consumer wrapper,
 which also formats diagnostics before returning an error;
 the execution manifest was updated before that test was run.
 Bounded debug and optimized release wrapper tests each returned an error and dropped the result without an abort.
 Neither direct nor wrapper result establishes a cause or resolution for the separate newer CLI/worker incident.

## Terminal discovery result

- **Registry class:
   finished.**
   Every frozen and expansion query was paginated to provider exhaustion (`next_page=null`) and validated for
   contiguous pages,
   matching totals and duplicate-free crate ids.
   Full-record metadata screens were displayed for `jsonc` (532 records) and
   `comment preserving json` (1214 records),
   plus the complete `jsonc lexer` (8) and `json-with-comments` (4) result sets.
   Limitation recorded rather than hidden:
   lead extraction used inclusive keyword predicates,
   so a crate whose name and description avoid both term families could have been passed over.
- **Repository-host class:
   finished.**
   The two frozen `gh search repos` queries returned no records.
   The comparable broadening ran three more literal queries,
   each returning fewer than its 100-result limit (provider exhaustion):
   `json5 parser language:Rust` (8 results:
   `json5-nodes`,
   a Git-only `json5_parser`,
   `JsonTape`,
   `jsonrepair-rs`,
   plus Lua,
   conversion and multi-format configuration projects),
   `json comments preserve language:Rust` (4 results:
   `google/json5format` at the regex gate,
   `AlexisTM/jwc` at the surrogate gate,
   `zoosky/yqr` YAML-only,
   `q5n/jsonsh` written in Go),
   and `lossless json editor language:Rust` (0 results).
   No new Rust JSONC parser candidate emerged from the broadening.
- **Broader-web class:
   frozen schedule finished,
   completeness not provable.**
   The recorded web queries returned source and documentation leads without totals or cursors,
   so that class cannot demonstrate enumeration the way the registry and host classes can.
- **In-repo class:
   finished.**
   The maintained TypeScript package,
   `doc/decision/jsonc-edit-parser-foundation.md` and the prior structured-edits research were read.
   Meow's selected editor stays out of scope:
   this port neither adopts nor replaces it.

Terminal outcome for the frozen schedule:
 **saturated with two survivors.**

1. The repository-owned parser and emitter translation,
   validated through the bounded suites,
   independent oracle controls,
   external consumer lifecycle,
   conformance corpus and workload-shape timings recorded in this report.
2. The `biome_json_parser` 0.5.7 syntax source plus the scratch projection adapter,
   validated at the same depth,
   with the preparse depth preflight,
   diagnostics gate and unsafe-surface findings recorded here.

Every other candidate either failed a hard gate as published
 (surrogate rejection,
 regex-defined lexing,
 depth ceilings,
 comment-model or formatter-only interfaces,
 memory-safety gaps)
 or remains an **unvalidated composition** that would still require writing this port's parser:
 `fjson` scanner-only,
 `hifijson` primitives,
 the Git-only `jsonc_lexer`,
 and the Serde byte/raw-token routes.
 Those compositions are not separately validated finalists;
 each would inherit the owned-parser work rather than replace it,
 and each adds an unaudited third-party surface.

Saturation here means the frozen schedule and page rule completed.
 It is not a claim that no suitable crate exists anywhere outside those queries.

## Current source checks, not a recommendation

- `jsonc-parser` 0.33.2,
   private clone `~/temp/agent/jsonc-parser-2026-09-24` at `e6e3837` (`Cargo.toml:3` names the version):
   `src/scanner.rs:175-201` calls the string decoder for escaped tokens,
   and `src/string.rs:214-261` explicitly returns `InvalidUnicodeEscapeSequence` for an unpaired high or low surrogate.
   The existing crate cannot parse `{"s":"\uD800"}` in its required JSONC grammar.
   Using its scanner would require an upstream fork,
   which is a distinct custom candidate.

  ```rust
  // src/scanner.rs:199-201
  crate::string::parse_string_with_char_provider(self)
    .map(Token::String)
    .map_err(|err| self.create_error_for_start(err.byte_index, ParseErrorKind::String(err.kind)))
  ```

- `edikt-jsonc` 0.4.0,
   private clone `~/temp/agent/edikt-2026-09-24` at `c87295c` (`crates/edikt-jsonc/Cargo.toml:3`):
   its `project.rs:122-131` converts a large integer to `f64`,
   and its `project.rs:140-185` unescapes `\u` via `char::from_u32`.
   An unpaired surrogate has no Rust `char` and falls through without appending a code unit.
   A compliant adapter must bypass that projection,
   read raw lexer tokens,
   decode to UTF-16,
   and implement the library's own comment attachment.
   `crates/edikt-jsonc/Cargo.toml:9-15` lists `edikt-core`,
   `edikt-syntax`,
   `logos`,
   `rowan` and `thiserror` as direct dependencies.
   License,
   CI,
   tests and full transitive tree remain to be checked.

  ```rust
  // crates/edikt-jsonc/src/project.rs:172-175
  let hex: String = chars.by_ref().take(4).collect();
  if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
      out.push(ch);
  }
  ```

- `json-five` 0.3.1,
   private clone `~/temp/agent/json-five-rs-2026-09-24` at `650be6a` (`Cargo.toml:3`):
   `src/rt/parser.rs:94-145` stores number and quoted-string lexemes verbatim;
   `src/rt/parser.rs:21` calls whitespace/comment context `Wsc = String`.
   It needs strict JSONC validation and a separate attached-comment parser and canonical emitter.
   Its `Cargo.toml:14-24` lists `unicode-general-category` and optional `serde` as production dependencies.
   Unpaired-surrogate acceptance in the round-trip parser remains to be probed.

`json-five` remains a serious composed candidate,
 not a validated finalist;
 `edikt-jsonc` is excluded by the regex hard gate.
 The repository-owned parser translation remains a baseline.
 `biome_json_parser` has source-level lexical and grammar evidence but still lacks a runtime surrogate and
 comment-boundary probe;
 other discovered candidates remain unscreened against those requirements;
 no elimination or ranking is justified yet.

## Parser expansion findings and registry pages

The frozen expansion searched crates.io using `jsonc comments` and `lossless json parser`.
For the first query,
 page 1 and page 2 each returned 100 records,
 and page 3 was empty (page 2 metadata reported total 200).
For the second,
 pages 1 and 2 each returned 100,
 and page 3 returned 86 with no next page (total 286).
These registry queries are exhausted,
 not evidence that every library is viable or that the broader discovery is complete.
The two frozen `gh search repos` queries returned no records;
 the comparable broadening queries and the resulting class status are recorded in the terminal discovery result.
Web searches discovered `jwc`,
 `hifijson`,
 `fjson`,
 `tokora`,
 and lexer-only `any-lexer` as leads;
 most lack evidence of the attached JSONC comment model.
 A delegated broader `jsonc` registry query exposed a first page but its 100-result web response was truncated;
 this is not a saturation result.
 The [crates.io data-access policy](https://crates.io/data-access) permits an identifying user-agent with at most one
 API request per second.
 A direct `curl --head` with this repository's identifying user-agent returned HTTP 200 for the `jsonc` alphabetical
 page.
 Before fetching the rest,
 `~/temp/agent/jsonc-registry-enumeration.mjs` fixed the query list,
 serial request delay,
 page ceiling,
 and full unfiltered private scratch output.
 The added broad query is an enumeration cross-check,
 not a recursive taxonomy expansion or an endorsement of any result.
 The identified API fetched and stored every broad `jsonc` alphabetical page:
 five pages of 100 and a final page of 32,
 with `meta.total=532` and `next_page=null` on page 6.
 The scratch `mise run audit:registry-validate` task checked contiguous pages,
 summed 532 records matching `meta.total`,
 and found 532 distinct crate ids with no duplicate.
 The numeric-query run initially stopped on a self-imposed page ceiling,
 then resumed from saved pages and finished without a provider cap.
 This completes the broad `jsonc` query's **pagination**,
 not screening of every record or saturation of every required discovery source.

### Delegated repository and broader-registry enumeration

A research-only agent searched GitHub repositories with `language:Rust`,
 best-match order,
 `per_page=100` and no explicit negative filter.
 Literal queries returned:
 `jsonc` 41,
 `json5` 39,
 `json lexer` 24,
 `lossless json` 12,
 `json syntax tree` 1,
 `jsonc lexer` 1,
 `json with comments` 12,
 `topic:jsonc` 6,
 `json cst` 2,
 and `json tokenizer` 157 across pages of 100 and 57.
 Each completed response reported `incomplete_results:false`;
 an extra narrow `json5 parser` call was blocked by its guardrail and is not counted.

Additional crates.io API terms yielded `jsonc lexer` eight results with no next page,
 `json-with-comments` four with no next page,
 and `comment preserving json` with a delegated-time reported total of 1212.
 The delegated request viewed its first three pages of ten;
 its metadata screen cannot establish outcomes on the rest.
 A later main-session alphabetical query with 100 results per page fetched all 13 pages under
 `~/temp/agent/jsonc-registry-full-pages/comment-preserving-json/`.
 The live index now reports 1214 records;
 `mise run audit:registry-validate` confirmed 1214 distinct IDs,
 contiguous pages,
 stable totals in the saved response and terminal `next_page=null`.
 An inclusive metadata lead display surfaced `zetch`,
 `jsonrepair-rs` and `zoko-parser` for source screening,
 but its keyword predicate can miss uninformative descriptions.
 This establishes secondary-query pagination,
 not full candidate screening or saturation.

Source leads for that screening include `jsonc_lexer` from
[richplastow/jsonc-lexer](https://github.com/richplastow/jsonc-lexer),
 [`fracturedjson` 0.1.1](https://crates.io/crates/fracturedjson/0.1.1),
 [`json-with-comments` 0.1.5](https://github.com/hayas1/json-with-comments),
 and [`json5format` 0.2.6](https://crates.io/crates/json5format/0.2.6).
 Versioned source checks have now classified all four leads:
 a Git-only lexer primitive,
 a published formatter-only interface,
 a Git-only parser with a proven surrogate-safety failure,
 and a published regex-backed parser.
 None has been adopted as this port's foundation.
 Candidate metadata alone is not a recommendation.
 The research transcript is retained at `~/temp/agent/` through its subagent output,
 while this report remains the owned audit artifact.

### Additional repository-host JSON5 parser query

A main-session `gh search repos 'json5 parser language:Rust' --limit 100 --json fullName,description,url,updatedAt`
 returned eight repositories after the delegated version of this query had been blocked.
 It surfaced `json5-nodes` and a separate Git-only `json5_parser` repository;
 `JsonTape` and `jsonrepair-rs` were already screened,
 while several other results advertise Lua integration,
 conversion,
 plain JSON or multi-format configuration rather than comment-bearing JSONC editing.
 This one completed host query does not settle the broader source class.

The published `json5_nodes` 2.0.2 `src/lib.rs:14-52,90-151` stores numbers as `i64` or `f64`
 and rejects escaped lone high/low surrogates when projecting a string.
 Its `src/json5.pest:4-5` makes comment grammar rules silent,
 so its public JSON5 AST has no comments on keys or values.
 `pest` is a PEG grammar dependency here,
 not by itself evidence of a regex-backed production lexer;
 this as-is exit is about the value and comment model.

The Git checkout `jessevanassen/json5_parser` at `45275c0d29438bcc5a7f25f22301170ef564d480`
 has `publish = false` in `json5_parser/Cargo.toml:1-8`.
 Its `src/parser.rs:18-25,105-114,120-151,280-329` returns Serde values,
 converts decimal numbers to `f64`,
 rejects escaped lone surrogates through `char::from_u32` and consumes comments as trivia.
 Its `json5_parser` package must not be confused with an unrelated published `json5-parser` 0.0.0.
 Adopting the Git-only source would require a separate fork or vendoring decision;
 no parser code was executed.

### Full broad `jsonc` metadata screen

The delegated subagent could not read the saved pages under its security guardrail and screened none of them.
 A direct main-session `mise run audit:registry-page` pass then displayed the name,
 version and first 240 description characters for **every** saved `jsonc` result on pages 1 through 6,
 with explicit truncation flags rather than a negative filter.
 The unabridged records remain in private scratch;
 clipped descriptions and metadata alone can still hide a candidate's actual behavior.

Besides already-ledgered parsers,
 metadata raises `fig` 4.1.0 (comment-preserving multi-format editor),
 `dprint-plugin-jsonc` 0.7.4,
 `prim-fmt` 0.9.1,
 `jcfmt` 0.1.2,
 `jsontape` 0.1.4,
 `nojson` 0.3.15,
 `babbel_json` 0.2.2,
 `jqf-codec-json` 0.1.1,
 and `serde_jsonc2` 0.1.2 for category/source checks.
 The later page also includes `subc-jsonc` 0.1.1,
 described as JSONC-to-JSON normalization rather than a raw comment owner.
 These are **leads**,
 not serious alternatives or hard-gate exits solely on registry text.
 Source and API screening remains necessary before assigning survivor counts or source-class saturation.
 An **as-is interface exit** is not automatically a foundation-composition exit:
 source spans or raw values can sometimes supply an adapter with information absent from the public editor value.
 Such compositions remain pending until information recovery and adapter cost are checked,
 using the same rule applied to the Biome projection.

### Secondary registry source leads

The published `zetch` 0.1.0 JSON path calls `fjson::ast::parse`
 (`src/read_write/langs/manager.rs:12-26`,
 `src/read_write/langs/json.rs:245`),
 and its reading path converts compact formatted data through `serde_json::Value`
 (`src/read_write/langs/json.rs:20-32`).
 Its YAML path uses `nondestructive`;
 that dependency is not an independent JSONC parser in this implementation.
 `zetch` therefore adds a mutable editor wrapper over the separately evaluated `fjson` parser,
 not a new parser foundation with a different accepted string/number/comment domain.

The published `jsonrepair-rs` 0.2.1 public API (`src/lib.rs:35-64,144-284`) returns repaired JSON text
 or an optional Serde value;
 strict mode rejects inputs requiring repair,
 including JSONC comments.
 Neither mode returns owned key/value comment data as-is.
 Its parser path merits no finalist promotion solely on a repair API.

The published `zoko-parser` 0.2.0 selects `regex` as a normal dependency (`Cargo.toml:55-57`);
 `src/lib.rs:14,263,464-479` invokes `Regex::new` from the parser's string-processing route.
 Exclude it at the production-regex gate without executing its parser.
 Zoko's own JSON-like grammar also admits non-JSONC string forms,
 but this does not change the earlier regex exit.
 These source screens do not settle the rest of the secondary query's metadata records.

### Secondary registry metadata screen

A direct `mise run audit:registry-page` pass displayed the name,
 version,
 first 240 description characters and clipping flag of **every** saved
 `comment preserving json` result across pages 1 through 13.
 The validated saved response has 1214 distinct records;
 this is an inspected metadata set,
 not evidence that each registry crate's source was audited.
 Most records advertise other domains,
 such as issue trackers,
 YAML-only editors and OpenAPI generators.
 The full-page pass uncovered `agent-first-data`,
 `patchloom` and `qubit-json`,
 which the earlier keyword lead display missed.
 All three received targeted source checks before treating the secondary query's visible metadata as screened.
 A description with no relevant terms is not proof of source behavior;
 full source-class saturation and surviving composition counts remain open.

- `agent-first-data` 0.34.0 has a byte-span JSON editor,
   but its JSON backend explicitly says that JSON has no comments
   (`rust/src/document/format/json.rs:5-42`),
   and `Parser::skip_ws` accepts only JSON whitespace (`:590-598`).
   Its comment-preserving backends are separate YAML/TOML paths.
   It cannot supply JSONC comment queries as-is;
   adding a comment lexer is a distinct composition.
- `patchloom` 0.36.0 strips JSONC comments and trailing commas for Serde JSON
   (`src/ops/doc/jsonc.rs:1-10,19-53`),
   rather than returning attached key/value comments.
   More decisively,
   its selected normal `regex` dependency is unconditional (`Cargo.toml:198-199`),
   and production search/replace code constructs regexes (`src/lib.rs:406-425`,
   `src/ops/replace.rs:7,54`).
   Exclude this published package at the user's selected-dependency regex gate,
   not merely because its JSONC edit path uses string scans.
- `qubit-json` 0.10.0 accepts `jsonc` only as a Markdown fence label:
   its documented fenced content must remain strict JSON without comments or trailing commas
   (`src/decode/markdown_fence_policy.rs:34-39`).
   Its syntax error model explicitly rejects unpaired Unicode surrogates
   (`src/decode/json_syntax_error_reason.rs:45-51`).
   That is a JSON processing component,
   not an as-is JSONC source foundation.

No candidate code was executed for these source screens.

### `fig` 4.1.0 as-is contract exit

The published `fig` 4.1.0 source in the local Cargo registry exposes an editor whose public comment operations take
`&mut self`
 (`src/editor.rs:342-423`) and a value model with `Int(i64)`,
 `Uint(u64)` and `Float(f64)` (`src/value.rs:85-116`).
 `src/value.rs:493-522` converts numeric literals that exceed its integer variants or are fractional into `f64`.
 This cannot be the required exact mathematical number value model as-is,
 and its edit API is not the requested immutable-state contract as-is.
 Its leading/trailing comment API is positional and restricts edits on one-line flow items,
 unlike independently queryable normalized comments on every key and value.

`fig` depends on `fig-sys` 4.1.0,
 whose `build.rs:18-40,104-148` links target-specific prebuilt native archives for default features or invokes Zig for
 source builds.
 The target packages and Zig build/source-to-artifact mapping would require a separate native safety and provenance
 audit for any proposed raw-token adapter.
 A targeted scan of its production Zig/Rust source found regex words in sample strings and comments,
 not an observed production regex lexer;
 regex is **not** the as-is exit reason.
 No candidate code or binary was executed for this screening.

### `dprint-plugin-jsonc` 0.7.4 as-is category exit

The published `Cargo.toml:28-42` depends on `jsonc-parser` in the 0.15 range.
 Its public `src/lib.rs:1-7` exports `format_text` and configuration,
 while its `parser` module is private.
 It is a formatter interface,
 not a public raw-token or independently queryable key/value-comment editor.
 Consuming its private parser would be a fork or another custom composition,
 not adoption of the published formatter as-is.
 The separately audited `jsonc-parser` 0.33.2 surrogate failure is **not** attributed to the older 0.15 dependency
 without a versioned source check.
 No upstream code was executed for this category screen.

### `prim-fmt` 0.9.1 as-is category exit

The published `prim-fmt` library exposes file classification and `format()` at `src/lib.rs:31-63`,
 not a comment-query or immutable edit-state API.
 Its JSONC implementation `src/json.rs:1-43` calls `dprint-plugin-json::format_text` and applies whitespace hygiene;
 `:59-65` tests that formatting removes a trailing comma.
 Preserving source comments while formatting is not the attached key/value comment model or canonical trailing-comma
 output of this port.
 Its private formatter parser would be a separate adapter/fork candidate rather than the published interface as-is.
 No candidate code was executed for this screen.

### `jcfmt` 0.1.2 category exit

The published `jcfmt` manifest declares only a `[[bin]]` target at `src/main.rs`,
 not a consumable Rust library (`Cargo.toml:18-35`).
 It is a JSONC formatting command,
 so calling it as a subprocess would not expose independently queryable key/value comments or a native immutable
 edit-state API.
 Its `nojson` dependency is a distinct potential parsing component and remains subject to separate source screening.
 No command was executed.

### `fracturedjson` 0.1.1 as-is formatter-interface exit

The published `src/lib.rs:123-139` keeps `tokenizer`,
 `parser`,
 and `model` modules private,
 exposing `Formatter`,
 error and configuration types rather than a consumable syntax tree.
 Its `src/formatter.rs:127,166,200,253` exposes text reformat/minify and Serde-value serialization methods,
 not a public parse-to-syntax method.
 Although internal scanning sees comment tokens,
 the public formatter cannot supply independently queryable comments on keys and values or immutable edit state.
 A fork exposing and adapting its internal tokenizer would be a separate custom candidate.
 A targeted search of its published `src/` and manifest found no direct production regex use;
 its as-is exit here is the public API boundary.
 No candidate code was executed.

### Git-only `jsonc_lexer` 1.0.0 primitive

The `richplastow/jsonc-lexer` checkout at `0d40eff67f985e5ecb7ba8c1565afb4f1206edc3`
 identifies `jsonc_lexer` 1.0.0 and `rlib`/`cdylib` in `wasm/jsonc_lexer/Cargo.toml:1-15`.
 Its `wasm/jsonc_lexer/src/lib.rs:42-60` exports native `jsonc_lexer_native(&str) -> TokenizeResult`;
 `src/types.rs:7-36` returns raw tokens for numbers,
 strings,
 comments and punctuation,
 not a parsed key/value ownership tree.
 Its line-comment scanner stops at CR or LF (`src/consume_line_comment.rs:13-28`),
 which is compatible with the required line termination at this source-screening level.
 The crate's own license file states MIT.
 Official crates.io exact-name lookups and package searches yielded no verified release under its manifest name;
 a Git version string is not a published registry artifact.
 A separately written parser and vendoring/fork packaging could use this lexer,
 but neither composition nor its selected normal/build dependencies have been audited.
 No production regex call was observed in its directly inspected lexer source,
 and no candidate code was executed.

### Git-only `json-with-comments` v0.1.5 safety exit

The tag `v0.1.5` at `6584e3495fecca3a1d73a14b8f0f11794faf5e0e` declares
 `json-with-comments` 0.1.5 in its manifest,
 but exact-name crates.io API and search found no verified published crate.
 `src/de.rs:61-65` invokes `StrTokenizer` from the public `from_str` entry;
 `src/de/token.rs:144-178` accepts four hexadecimal escape digits and passes their value to
 `unsafe { char::from_u32_unchecked(hex) }` without excluding `0xD800..=0xDFFF`.
 `from_str_raw` also reaches the shared escape parser (`src/de/token/raw.rs:23-28`).
 An escaped lone surrogate therefore reaches a source-proven undefined-behavior path;
 the original unsafe code was **not** run on this input.
 The bounded disposable [checked prototype](../troubleshooting/json-with-comments-surrogate-escape.md)
 passed 86 upstream unit tests,
 three new surrogate tests and 64 upstream integration tests,
 but it safely **rejects** unpaired surrogates,
 so it does not provide this port's accepted string domain.
 Exclude the upstream parser as-is on safety and semantic hard gates;
 a redesigned source fork is a separate custom candidate.

### `nojson` 0.3.15 as-is contract exit

The published Rust library does expose `RawJson::parse_jsonc` and comment byte ranges (`src/lib.rs:12-18`,
 `src/parse.rs:30-42,80-110`),
 with no declared runtime dependency or unsafe code in its advertised crate surface.
 Its parser sets `MAX_NESTING_DEPTH` to 128 (`src/parse.rs:7-24`),
 below the supported 512-container boundary.
 More decisively,
 `src/parse.rs:376-403` explicitly rejects an escaped unpaired high or low UTF-16 surrogate before a `RawJson` value
 exists.
 The comment-range formatter cannot recover that required accepted string domain from a rejected parse.
 The published parser therefore fails both hard constraints as-is;
 a fork or own string scanner/parser is a separate custom candidate.
 This exit is **not** a regex finding.
 No candidate code was executed.

### `jsontape` 0.1.4 as-is surrogate exit

The published `jsontape.rs:310-400` offers a JSONC preset with comments and trailing commas,
 and its owned-document option can preserve comment trivia (`:345-347,439-454`).
 It also exposes raw source spans and configurable depth,
 so registry metadata alone understated this candidate's behavior.
 However,
 the shared string-escape scanner at `jsontape.rs:2269-2287` returns `SyntaxKind::LoneSurrogate`
 for an escaped high surrogate without a following low half or for an isolated low half.
 Both owned and view parsing use that scanner,
 so the published crate cannot accept the required escaped UTF-16 input as-is.
 A fork changing the scanner or an independently validated token path would be a separate composition candidate;
 a raw/view adapter's escape-decoding requirements have not been established.
 Its manifest names `allocator-api2` as a normal dependency and does not identify a production regex lexer;
 this as-is exit is about the supported string domain.
 No candidate code was executed.

### `serde_jsonc` and `serde_jsonc2` as-is comment-model exits

The published `serde_jsonc` 1.0.108 and `serde_jsonc2` 0.1.2 parse line and block comments as whitespace
 (`src/de.rs:248-320` in both archives).
 Both published versions also reject a trailing array or object comma with `ErrorCode::TrailingComma`
 (`serde_jsonc/src/de.rs:1123-1151`,
 `serde_jsonc2/src/de.rs:1125-1152`).
 That fails the accepted JSONC grammar **before** the comment/value projection can be adapted as-is.
 Their `Value` projection follows Serde's JSON value model,
 without distinct comment-bearing object keys and values;
 consuming comments as whitespace cannot supply the required query/edit API as-is.
 `serde_jsonc2` offers `raw_value` and `arbitrary_precision` features,
 but those do not add normalized attached comments by themselves.
 Counterevidence against a blanket surrogate exit:
 both archives document `deserialize_bytes` accepting escaped lone surrogates as WTF-8 bytes
 (`serde_jsonc2/src/de.rs:1636-1699`,
 `serde_jsonc/src/de.rs:1634-1694`),
 even though the ordinary Rust `String` value route requires valid UTF-8.
 A custom Serde visitor with byte-string keys/values plus a separate comment owner might compose these parts,
 but its map-key route,
 exact numeric raw tokens and source-boundary ownership remain untested.
 The **as-is `Value` interface** fails comment queries;
 the composition is pending rather than excluded by that interface alone.
 No regex claim is inferred here,
 and no candidate code was executed.

### `babbel_json` 0.2.2 as-is grammar and comment-model exit

The published `src/parser/json5.rs:104-152,222-239` exposes a JSON5/JSONC parse entry that also admits JSON5-only syntax
 and fixes a private recursive-descent depth ceiling at 256 with no public setter.
 The maintained TypeScript structured parser accepts 512 containers (`package/module/jsonc-edit/src/parse.ts:21,54`).
 The selected JSON5 parser's `skip_whitespace_and_comments` consumes trivia without retaining it
 (`src/parser/json5.rs:176-220`);
 the separate `strip_comments` helper is **not** evidence that this parse entry calls it.
 The public `src/nodes/types.rs:10-29` projects to strings,
 numbers,
 arrays and unordered objects without attached key/value comments.
 As-is it therefore misses the strict JSONC-only grammar,
 accepted 512-depth domain and separate comment queries.
 An independently strict token/owner parser or a fork is a distinct composition candidate,
 not that public projection.
 A targeted source/manifest search found no production regex call in this JSON5 parser path;
 its exits here are semantic rather than regex-based.
 No candidate code was executed.

### `jqf-codec-json` 0.1.1 as-is comment-query exit

The published codec documents a JSONC grammar with comments and trailing commas,
 source-preserving edit splice and a shared exact-decimal number model (`src/jsonc/mod.rs:1-44`).
 The number representation and conversion path have not been independently verified against this port's exactness cases.
 Its documented `jsonc.comment@1` fact is **leading comments on the value node only**;
 the same module says inline,
 trailing and inner comments survive byte-wise edits but cannot be read through the comment query.
 That differs from this port's separately queryable normalized comment on **every key and value**,
 including trailing value comments.
 `src/edit.rs:1-19` describes splice verification by re-decoding;
 it does **not** prove a conflict with an immutable wrapper.
 However,
 the shared fast string path defers an escaped lone high surrogate to an error (`src/parse.rs:700-719,2480-2520`);
 a lone low surrogate becomes U+FFFD (`src/parse.rs:724-726`,
 `src/lex.rs:441-455`).
 The published parser therefore cannot supply this port's required decoded string value on that source text as-is.
 Retained authored bytes and spans may still help a fork or an independently validating preprocessing adapter,
 but neither composition has been shown to accept the exact string domain.
 A targeted direct-source search found no regex invocation in its JSONC path;
 transitive source and normal/build graph remain uninspected.
 No candidate code was executed.

### `subc-jsonc` 0.1.1 as-is span/normalizer exit

The published single-file `src/lib.rs:3-68` exports JSONC-to-JSON normalization by discarding comments
 and trailing commas.
 `src/lib.rs:115-200` also exports object/member byte-span queries for narrow edits,
 but the tokenizer and parser are private (`:221,341`) and no public key/value comment node or number value is returned.
 Source-preserving span cuts are not a complete canonical emitter with independently queryable comments.
 Treat it as a useful utility for a different edit model,
 not this port's parsing foundation as-is;
 building a comment-bearing editor on its private parser would be a separate custom candidate.
 Its inspected source has no direct regex call;
 no candidate code was executed.

### Other registry JSONC consumers and normalizers

`c4-config` 0.5.0 calls `jsonc_parser::parse_to_value` behind its JSONC feature,
 then converts raw numbers to `i64`,
 `u64` or `f64` and discards comments (`src/format/jsonc.rs:7-59`).
 `formatforge` 1.0.5 exposes conversion through `serde_json::Value` and calls
 `jsonc_parser::parse_to_serde_value` (`src/lib.rs:14-66`,
 `src/formats/jsonc.rs:1-14`).
 `lintel-validate` 0.0.12 likewise exposes JSONC schema validation through
 `jsonc_parser::parse_to_serde_value` (`src/parsers/jsonc.rs:1-38`).
 These wrappers introduce no distinct exposed comment-bearing parser foundation;
 do not transfer the separately audited 0.33.2 `jsonc-parser` surrogate verdict to an unverified older dependency
 version.

`jsonc-to-json` 0.1.1 exports a JSONC-to-JSON transformation and iterator
 (`src/lib.rs:151,207,238`),
 while `zpl_toolchain_jsonc_strip` 0.1.0 exports `strip_jsonc` (`src/lib.rs:13`).
 Their public transformations remove the comment data required for an editor as-is;
 independent re-tokenization would be a different composition.

`hana_rubric` 0.1.0 parses domain-specific keymaps through
 `serde_json_lenient::from_str::<WireDocument>` and a private source index
 (`src/keymap/document.rs:1-97`),
 not a public generic JSONC editor tree.
 Its underlying `serde_json_lenient` 0.2.4 is a distinct component lead:
 `src/de.rs:35-68,302-345` allows comments by default but consumes them as whitespace;
 `:1684-1754` documents escaped lone-surrogate bytes through `deserialize_bytes`.
 The default depth budget is 128;
 the optional unbounded setting (`:202-236`) does not by itself establish safe 512-depth lifecycle.
 More decisively,
 `src/de.rs:302-327` ends a `//` comment only on LF,
 although CR is otherwise recognized as whitespace.
 An independent bounded consuming crate using published 0.2.4 admitted LF and CRLF cases,
 but rejected a CR-only JSONC member with `EOF while parsing an object at line 1 column 18`.
 A fresh upstream [red/green prototype](../troubleshooting/serde-json-lenient-cr-line-comment.md)
 matched the published source at commit `111dd4522`;
 its CR tests failed before the scanner patch,
 passed afterward,
 and the patched upstream all-target suite passed.
 The published parser fails this port's CR-only comment contract as-is.
 Even a patched fork would still discard comment ownership and need an independently audited
 512-depth byte-key/value and exact-number adapter.
 The other wrapper source screens did not execute their candidate code.

### `fjson` 0.3.1

**Source correction:**
 the shallow clone initially pointed at `fe5a04a` despite its `Cargo.toml` still saying 0.3.1.
 Its scanner and AST SHA-256 hashes differed from the published 0.3.1 Cargo archive.
 After fetching `v0.3.1` (`e658f8a`),
 `src/scanner.rs` and `src/ast.rs` hashes match the published archive
 (`e44b66674c937f76909afb404b20c739ec16438772137b9c61b32e53c4c740be` and
 `d225e656d8b040a4eeac93661b86659980e7fc82cd18a37fbf2b94a5cf6ed62a`).
 All current source citations in this section target the tag.
 The newer checkout added a leading-zero check to the scanner;
 do not attribute that check to published 0.3.1.
 The published parser depth and whitespace observations were re-read at the release tag and still hold.

- Discovery:
   crates.io `jsonc comments` page 1,
   [registry](https://crates.io/crates/fjson/0.3.1),
   private clone `~/temp/agent/fjson-2026-09-24` at release tag `v0.3.1` (`e658f8a`),
   `Cargo.toml:3` confirms 0.3.1 and `:13-15` names `arrayvec` and `unicode-width` as runtime dependencies.
- Screening:
   `src/scanner.rs:143-173,175-213,214-264` returns raw JSON numbers,
   quoted strings (without outer quotes) and C-style comments.
   Its `src/ast.rs:12-60` holds same-line value comments but keeps most comments as separate `Metadata` objects,
   not distinct key/value comment fields.
   An attachment and merge layer,
   UTF-16 decoder and canonical writer are still needed.
   `src/ast.rs:64,83` caps parser recursion at 128 levels,
   while the TypeScript structured parser allows 512 (`package/module/jsonc-edit/src/parse.ts:21,54`).
   Its scanner also skips Unicode whitespace outside strings (`src/scanner.rs:326-330`),
   whereas strict JSONC uses JSON whitespace.
   These are source observations,
   not a completed upstream-behavior diagnosis or a validated workaround.

  ```rust
  // src/scanner.rs:326-330
  fn skip_whitespace(&mut self) {
      while let Some(c) = self.peek_char() {
          if c.1.is_whitespace() && c.1 != '\n' {
              self.skip_char();
  ```

- Source quality lead:
   `src/lib.rs:151` forbids unsafe code.
   `.github/workflows/test.yml` runs Cargo fmt,
   build,
   clippy and test,
   with no build script in this crate's manifest;
   transitive command trees are not yet audited.
   This is a serious lexer/AST candidate,
   not a finalist.

### Other registry leads

- `jsonc` 0.1.0 has no repository URL in registry metadata;
   its published `Cargo.toml:29-34` selects `jsonc-parser` 0.26.
   Its `JsoncDocument::parse` uses `CstRootNode::parse` and its public getter projects through
   `serde_json::Value` (`src/jsonc_document.rs:81-117`).
   The **versioned 0.26.0** CST parser calls `parse_to_ast`
   (`jsonc-parser/src/cst/mod.rs:1047-1060`),
   whose scanner calls the string decoder
   (`jsonc-parser/src/scanner.rs:148`,
   `src/string.rs:141-166`).
   That decoder rejects unpaired surrogate escapes through `char::from_u32`,
   so the wrapper's CST entry fails the required source domain as-is.
   This conclusion no longer borrows the separate 0.33.2 finding.
- `jsonc_tools` 0.0.1 published source (`src/lib.rs:1-18`,
   `src/parser/parser.rs:1-13`) is a scaffold with no JSONC parser implementation.
   Category mismatch.
- `jsontape` and the `tokora` JSON CST example have versioned as-is exits in their separate sections.
   `hifijson` 0.5.0 exposes a byte lexer whose caller supplies a peek function
   (`src/token.rs:27-60`);
   the default `ws_peek` skips only JSON whitespace,
   and attaching comments would require a custom peek/ownership parser.
   It remains a possible primitive composition,
   not an as-is comment-bearing editor.
   `purrdf-json` 2.0.2 parses RFC 8259 JSON,
   whose whitespace scanner omits comments (`src/parse.rs:4-58`);
   its RDF-oriented strict-JSON interface is a category mismatch as-is.
   `momoa` has a separate source-proven safety hard-gate failure recorded next.

### `momoa` 3.2.6 safety exit

The published Rust parser is excluded as-is on a safety hard gate,
 not on regex.
 In `rust/src/readers.rs:79-113`,
 four hex digits after `\u` are accepted without excluding surrogates;
 `rust/src/tokens.rs:119-140` produces a string token;
 and `rust/src/parse.rs:326-351` feeds the decoded `0xD800` to `unsafe { char::from_u32_unchecked(char_code) }`.
 The source in private clone `~/temp/agent/momoa-2026-09-24` at `8dfb563` matched the published 3.2.6 `parse.rs` and
 `readers.rs` SHA-256 hashes.
 [Rust documentation](https://doc.rust-lang.org/std/primitive.char.html#method.from_u32_unchecked) says a surrogate is
 not a valid `char`,
 and constructing it is undefined behavior.
 Full source trace,
 safe checked pre-fix control,
 bounded patched prototype,
 targeted public-entry checks,
 upstream Rust suite output,
 and reproducible patch are in
 [`momoa-rust-unpaired-surrogate.md`](../troubleshooting/momoa-rust-unpaired-surrogate.md).
 The original unsafe parser was never run on this input.
 The patched prototype rejects isolated surrogates,
 while this port explicitly requires retaining them as UTF-16 values;
 it is a safety workaround,
 not a semantic foundation for the port.

## Regex screening, 2026-09-24

The user requested early regex culling.
 Our operational screening interprets this as production parsing or lexing and required normal/build dependencies,
 not upstream dev-only usage.
 The original TypeScript scanner in `package/module/jsonc-edit/src/scan.ts` uses direct character scans.
 `edikt-jsonc` 0.4.0 is now **excluded**:
 `crates/edikt-jsonc/src/lexer.rs:47-69` uses repeated `#[regex(...)]` patterns through `logos::Logos` (`:23`),
 not merely a dev-only regex dependency.
 See [versioned lexer source](https://docs.rs/crate/edikt-jsonc/0.4.0/source/src/lexer.rs).
 Stop its audit at this hard gate.
 The `tokora` 0.11.0 [JSON CST example](https://docs.rs/crate/tokora/0.11.0/source/examples/json_cst.rs) likewise uses
 `logos` with `#[regex(...)]` for whitespace,
 number and string tokens;
 exclude that example as a proposed foundation,
 not the combinator crate in every possible configuration.
 The published `json5format` 0.2.6 `Cargo.toml:24-25` declares a production `regex` dependency,
 and `src/parser.rs:9,100-166` constructs `Regex` matchers for tokens,
 comments and quoted strings.
 Exclude that parser/formatter as-is at the user's production regex gate;
 no upstream code was executed.

`json-five` 0.3.1 lists `regex = "1"` only under `[dev-dependencies]` in `Cargo.toml:30-32`;
 its inspected production `src/` has no `regex::`,
 `Regex::`,
 `#[regex]`,
 or `logos` use.
 Do not cull on test-only dependencies.
 Direct source/manifests for `fjson`,
 `jsonc-parser`,
 `biome_json_parser`,
 `jwc`,
 and `hifijson` yielded no matching regex calls or macros under the same targeted search,
 and the resolved dependency graph was then checked for their selected configurations.
 Their broader behavior and source audits remain pending;
 this is not a claim of universal regex absence.

### Resolved production dependency graphs

A disposable set of exact-version manifests under `~/temp/agent/jsonc-regex-audit/` ran `mise run audit:regex-deps`,
 which invokes Cargo `tree --edges normal,build --target all --format {p}` without compiling upstream code.
 The individual graphs are saved as `tree.txt` in each case directory.
 `fjson` 0.3.1,
 `json-five` 0.3.1 with and without default features,
 `biome_json_parser` 0.5.7 with the exact companion pins,
 `hifijson` 0.5.0,
 `jsonc-parser` 0.33.2 with `cst`,
 and `jwc` 0.1.0 showed no package named `regex`,
 `logos`,
 `pest`,
 `regex-automata`,
 `regex-syntax`,
 `onig`,
 or `fancy-regex` in normal/build dependency graphs.
 Positive control `edikt-jsonc` 0.4.0 showed `logos`,
 `logos-codegen`,
 `regex`,
 `regex-automata`,
 and `regex-syntax`;
 the probe can reveal the unwanted dependencies.
 This verifies package names under the selected versions/features,
 not every line of transitive source or a future resolution.
 Keep source-level regex clearance pending where required.

### Selected Biome license and optional-feature check

`mise run audit:licenses` in the disposable Biome probe joined Cargo metadata to the **selected normal/build**
 `cargo tree --locked --offline --edges normal,build --target all` graph.
 It found 87 registry package versions,
 no missing license expression and no regex-named selected package.
 The selected expressions include MIT,
 Apache-2.0,
 0BSD,
 Unlicense,
 Zlib and one conjunction with Unicode-3.0.
 The published `biome_json_parser-0.5.7/LICENSE` is MIT;
 its `ROME_LICENSE` also carries an MIT notice.
 `unicode-ident-1.0.26/LICENSE-UNICODE` permits redistribution with its notice in copies or documentation,
 and `unicode-bom-2.0.3/LICENSE` contains Apache-2.0 terms.
 The [Apache Software Foundation compatibility note](https://www.apache.org/licenses/GPL-compatibility.html) states
 that Apache-2.0 software can be included in GPLv3 works.
 This source review found no license-expression exit for the prospective LGPL-3.0-or-later crate,
 but full product notice assembly and legal interpretation remain to be validated before publication.

An initial Cargo-metadata-only traversal appeared to select `regex-automata` through `similar` and `bstr`.
 That result was **not** a production graph:
 `similar-2.7.0/Cargo.toml:108-110,135-147` makes `bstr` optional behind its `bytes` feature,
 and the selected normal/build `cargo tree` omits `bstr` and `regex-automata`.
 Raw Cargo metadata includes unselected optional packages;
 do not use it alone to reject Biome at the regex gate.
 This does not replace source-level screening of all required selected dependencies.

### Biome bounded execution preparation

The published `biome_json_parser` 0.5.7 archive SHA-256
 `9c6d23fb9b683e6356c094b4a0cb38f8aa0acee60ce9c3ef24628d21a204de4d` matches its generated scratch Cargo.lock entry.
 `~/temp/agent/biome-execution-manifest.md` records the intended offline container command,
 source and checksum,
 build-script and proc-macro inventory,
 expected compiler and `emcc` probes,
 allowed read/write paths,
 resource ceilings,
 and stop conditions **before candidate execution**.
 The resolved graph inventory under `~/temp/agent/biome-execution-inventory.mjs` found 82 unique dependency names,
 16 build scripts and 9 procedural-macro packages across all target configurations;
 these are audit-surface measurements,
 not timing or proof of candidate behavior.
 The scratch test will probe surrogate and raw token acceptance,
 JSON5 rejection,
 and 512-level parse/syntax/drop only;
 the bounded first run completed 512-level array parsing,
 syntax-text access and destruction,
 and rejected JSON5-only forms.
 Its token-only comment assertion failed because comments are trivia attached to syntax tokens,
 not independent grammar tokens;
 the source-backed diagnosis and verified consumer correction are in
 [`biome-json-parser-token-trivia.md`](../troubleshooting/biome-json-parser-token-trivia.md).
 After reading trivia pieces,
 the same offline container passed all focused tests,
 including the escaped `\uD800` string and preserved `1e0` token.
 A further bounded test located a key-side comment on `{`'s trailing trivia,
 a value-side comment on `:`'s trailing trivia,
 and an after-comma inline comment on `,`'s trailing trivia;
 `biome_json_parser-0.5.7/src/token_source.rs:31-60` classifies trivia around newlines,
 while `biome_parser-0.5.7/src/tree_sink.rs:123-145` attaches it to syntax tokens.
 A consumer must still map grammar-delimiter trivia to the accepted key/value ownership policy.
 Bounded debug and optimized release `typed_member_comment_projection` tests reached the member name,
 colon,
 value and trailing separator via `biome_json_syntax-0.5.7/src/generated/nodes.rs:110-143`
 and `biome_rowan-0.5.7/src/ast/mod.rs:565-590`;
 it retrieved the key-side,
 value-side and after-comma pieces without a second syntax parse.
 This tests accessible source slots,
 not general ownership rules or immutable projection.
 Bounded debug and optimized release suites each passed 512-level array and record parse/syntax/drop,
 cleanup after a deep syntax error,
 and the adapter's 512-accepted/513-rejected depth controls.
 Upstream accepted a 513-level array without diagnostics;
 `biome_json_parser-0.5.7/src/syntax.rs:152-230` uses an explicit sequence stack without this editor's depth ceiling.
 The tested postparse guard rejects the extra opener and scalar roots;
 it does not cap upstream allocations before parsing.
 See [`biome-json-parser-nesting-limit.md`](../troubleshooting/biome-json-parser-nesting-limit.md).
 The lexer uses `from_utf8_unchecked` and `unreachable_unchecked` at `biome_json_parser-0.5.7/src/lexer/mod.rs:205-228`;
 the character-boundary and non-EOF invariants require a source and release-path safety audit.
 The presence of unsafe blocks alone does not prove a memory-safety failure.
 `src/lexer/mod.rs:67-90` gates token dispatch on a present byte,
 `:268-289,315-350` advances non-ASCII text by UTF-8 character length,
 and `biome_unicode_table-0.5.7/src/bytes.rs:103-125` declares a 256-entry dispatch table indexed by a byte.
 The scratch probe parsed and rejected selected multibyte strings and comments in bounded debug and optimized release
 runs.
 Additional bounded debug and optimized release tests exercised truncated escapes,
 multibyte invalid digits,
 unterminated strings and comments,
 and invalid Unicode placement through diagnostics,
 syntax-token and trivia-range traversal,
 and destruction without a crash.
 These checks support the inspected invariants but do not establish safety for every lexer and transitive-code path.
 Separately,
 `biome_json_parser-0.5.7/src/lexer/mod.rs:91-96` converts the source offset to `TextSize` with an `expect` for inputs
 beyond its representable range;
 `biome_text_size-0.5.8/src/size.rs:24-25,85-90` stores that offset as `u32`.
 The disposable adapter now checks source length before Biome runs.
 A synthetic boundary test accepted `u32::MAX - 1` and rejected `u32::MAX` in bounded debug and optimized release runs.
 No actual oversized document was allocated or passed through the upstream parser,
 so this verifies the guard's arithmetic and ordering in source,
 not a user-boundary result for a multi-gigabyte input.
 This validates a raw-syntax candidate,
 not key/value attachment,
 exact-value projection,
 canonical emission,
 immutable edits,
 or a consuming published crate.

### Syntax differential preparation

A separate scratch consumer `~/temp/agent/jsonc-foundation-diff/` depends on both disposable foundations.
 Its `Cargo.lock` was generated with `mise run lock:update` and SHA-256
 `33461c55d80be4ad511b16d593869e198fc9e9954b09b43aae536457ad15d550`.
 `~/temp/agent/jsonc-differential-execution-manifest.md` records the same pinned offline container,
 inspected Biome command tree,
 bounded source mutations,
 positive mismatch control,
 and stop conditions before execution.
 Bounded debug and optimized release runs passed named accepted/rejected JSONC controls and checked a deliberately
 mismatching BOM case **before** interpreting the generated comparison.
 The fixed single-character mutation corpus contained 5635 distinct inputs;
 both parser boundaries admitted 1577,
 and no syntax-admission difference was measured within that corpus in either build.
 This is a syntax-domain result under the stated generator,
 not a proof of full JSONC grammar equivalence,
 key/value comment parity,
 or value equality.
 Consumer-level comment projection remains unverified.
 The owned scratch crate now exposes its tested UTF-16 quoted-token decoder and ordered comment merger for a separate
 Biome-to-value projection experiment;
 `mise run lint:clippy` and its bounded debug suite passed after that extraction.
 This is prototype reuse,
 not a dependency or product API decision.

### Biome semantic projection prototype

A separate scratch package under `~/temp/agent/biome-projection-probe/` consumes Biome's typed syntax slots,
 the owned JSONC value model and canonical emitter,
 and the exact-number prototype.
 The package manager generated its Cargo.lock at SHA-256
 `9bc50be429ba207de2ce50e52195115a878af7c63a4d31a9a6d695e0cf377ac9`.
 `~/temp/agent/biome-projection-execution-manifest.md` records the inspected command tree,
 resource bounds,
 expected reads/writes,
 and semantic success/stop conditions before the first execution.
 Bounded debug and optimized release suites passed tests of distinct key/value comment owner,
 text and source kind;
 multiline value-comment ownership after canonical emission and reparse;
 after-comma,
 document and empty-container comments;
 escaped unpaired UTF-16 units on a key and value;
 exact equality of differently spelled numbers while retaining both raw literals;
 and 512-level array/record projection,
 emission,
 reparse,
 equality and cleanup with 513 rejected.
 These tests use explicit expected values rather than only the owned parser as oracle.
 The adapter shares the owned scratch emitter,
 comment merger,
 scalar decoder,
 and exact-number model;
 it does not establish independent correctness for those shared parts.
 Mixed-style comment kinds after emission,
 wider syntax and whitespace parity,
 package style lint and non-Linux targets remain unverified.

### Carriage-return line-comment divergence

A direct TypeScript bundle probe parsed a CRLF-separated object but returned a line-comment body ending in `\r`;
 the CR-only form threw `JsoncParseError: unterminated object (at offset 0)`.
 `package/module/jsonc-edit/src/scan.ts:340-359` searches only for `\n`.
 Separate isolated scratch Rust tests failed:
 the CRLF comment body was `" x\r"` rather than `" x"`,
 and the CR-only source returned `ParseError` at byte offset 18 before its next member.
 The scratch scanner also stops only on `\n` at `~/temp/agent/jsonc-parser-probe-2026-09-24/src/scan.rs:143-151`.
 Microsoft's JSONC scanner at `~/temp/agent/node-jsonc-parser-2026-09-24/src/impl/scanner.ts:251-261,399-401`
 (checkout `dba4356`) terminates a line comment at either CR or LF.
 The Biome adapter's bounded debug fixture accepted both CRLF and CR-only,
 returned a line-comment body `" x"` for each,
 and retained that value after canonical emission and reparse.
 Its optimized fixture remains unverified.
 The owned scratch scanner now stops at either CR or LF;
 bounded debug and optimized release suites passed both independent regressions and the remaining parser tests,
 with Clippy warnings denied.
 A separate scratch Rust consumer rebuilt the parser,
 exercised both line endings across parse,
 emit and reparse,
 and printed `JSONC consumer parse and emit passed`.
 The Biome projection fixture passed in both bounded debug and optimized release runs,
 retaining the same comment body and owner through canonical emission and reparse.
 TypeScript production behavior remains unchanged pending foundation adoption.

### Semantic differential preparation

The separate scratch differential consumer now imports the Biome-to-value adapter,
 with a package-manager-regenerated Cargo.lock SHA-256
 `6568ac62ea949cb295e2a472840da3b13cbb534b120446d2609c411434f26a87`.
 Its updated `~/temp/agent/jsonc-differential-execution-manifest.md` records the added read-only source mount,
 unchanged third-party build graph,
 and the new semantic comparison **before execution**.
 The same bounded mutation corpus will compare fully projected editor nodes when both grammars admit an input.
 A deliberately moved value comment is checked first so owner-blind comparisons cannot produce a trusted null result.
 Explicit expected values in the separate projection suite remain the independent control;
 comparing two prototypes with shared scalar/emitter helpers alone would not establish correctness.
 The bounded debug differential passed named semantic fixtures and both positive controls,
 but its generated corpus found one semantic difference among shared accepted inputs:
 for `{"a":1\n, //inline\n"b":2}`,
 the owned parser attaches `inline` to the following key `b`,
 while the Biome prototype placed it on the previous value `a`.
 Separate bounded named tests for object and array separators on a later line both failed the Biome adapter's owner
 assertion.
 The maintained TypeScript structured parser rejects the object source due its separately measured comma-after-trivia
 defect;
 its documented same-line trailing rule and the owned parser's continuation support the following-key reading.
 The prototype's previous `src/children.rs` routed every comma's trailing comment to the preceding value;
 Biome's raw syntax tree had retained the comment.
 The scratch adapter now checks for a line break between the value and comma
 (`src/children.rs:105-136`) and carries the comma's following comments to the next key or element when present.
 Both named owner regressions passed in bounded debug and optimized release suites.
 Reruns of the positive-controlled debug and optimized release differential each admitted 1577 of the 5635 generated
 inputs in both parsers
 and found no syntax or editor-node difference within that corpus.
 The intentionally mismatching BOM and moved-comment controls passed before interpreting either null result.
 This does not prove semantic equality outside the corpus.
 `package/module/jsonc-edit/src/emit-comment.ts:42-79` deliberately emits a trailing single-line comment as `//`;
 mixed-style comment kind may normalize after emission even when its body and owner survive.
 The named owner tests therefore assert body and owner after reparse,
 not an unsupported lexical-style identity promise.
 Full conformance,
 platform and upstream validation remain open.

### Biome recovery-to-editor-value boundary

Published `biome_json_parser` 0.5.7 builds an error-tolerant syntax tree
 (`src/lib.rs:19-47`);
 `JsonParse::has_errors` tests diagnostic error severity (`src/lib.rs:82-91`).
 The scratch editor wrapper checks `has_errors` **before** reading a root node or projecting values
 (`~/temp/agent/jsonc-regex-audit/biome/lib.rs:31-43`).
 A new positive-controlled test passed three malformed sources with a completed root or nested subtree:
 trailing non-JSON text,
 an invalid array child after a valid object,
 and a missing member separator.
 For each,
 the direct upstream syntax result had error diagnostics **and** an accessible typed root,
 while `parse_for_editor` returned an error rather than that apparently usable tree.
 Both bounded debug and optimized release wrapper suites passed 15 tests.
 This establishes the wrapper check for those recoverable fixtures,
 not a proof that every possible bogus node produces an error diagnostic.
 Duplicate-key behavior remains outside the user's supported contract;
 these tests do not claim a duplicate-key policy.

### Independent UTF-16 and comment-owner oracle controls

The two Rust projections share `jsonc_parser_probe::decode_quoted`,
 `jsonc_exact_number_probe::identity`,
 and `ValueKind`
 (`~/temp/agent/biome-projection-probe/src/lib.rs:30-66,82-109`).
 The after-emission comparison also uses the owned writer
 (`~/temp/agent/jsonc-foundation-diff/src/lib.rs:108-112`).
 Thus comparing their nodes alone cannot validate a shared string decoder or normalizer.
 A separate `mise run audit:utf16-oracle` Node script used `JSON.parse` plus `charCodeAt`
 to measure code units without either Rust parser:
 escaped lone high `[55296]`,
 escaped lone low `[56320]`,
 escaped U+FFFD `[65533]`,
 escaped non-BMP pair and literal `😀` both `[55357,56832]`,
 and literal or escaped `a` both `[97]`.
 The independent comparison at `~/temp/agent/jsonc-foundation-diff/src/lib.rs`
 now asserts those exact units **and distinct raw source spelling** for both parser candidates.
 Separate fixtures put `/*α😀*/` after a literal or escaped object key,
 verify that comment on the value rather than key,
 and assert ownership after shared canonical emission and reparse.
 A prior BOM admission mismatch and intentionally moved value comment remain positive controls.
 Bounded debug and optimized release `mise run test:isolated` suites each passed six tests,
 including the independent controls;
 this validates those measured examples,
 not all malformed inputs or the shared emitter's implementation in isolation.

### Shared TypeScript conformance corpus

Both candidates were run against the maintained corpus in
 `package/module/jsonc-edit.conformance/src/jsonc.conformance.unit.test.ts`
 (10 valid and 11 invalid sources),
 from the external differential consumer rather than from either parser crate.
 Every valid source was accepted by both,
 and every invalid source was rejected by both:
 JSON5-only forms (`{ 'a': 1 }`,
 `{ a: 1 }`,
 `{ "a": 0x1F }`),
 scalar roots (`42`,
 `"bare string"`),
 a missing comma,
 a doubled separator (`{ "a": 1, , }`),
 an unterminated object and an unterminated block comment.

Expectations were asserted **per candidate** rather than by mutual agreement:
 the number `-1.5e3` kept its spelling and compared mathematically equal to a separately parsed `-1500`;
 `"x\u0041"` decoded to units `[120, 65]` while retaining its raw spelling;
 boolean and null kinds matched;
 and comment ownership matched for a root trailing line comment,
 a root leading line comment,
 a key block comment and a same-line value comment.
 Bounded debug and optimized release suites each passed nine tests.

This corpus is the maintained TypeScript contract **as written today**.
 Where that contract has separately confirmed defects
 (a comma before trivia on a later line,
 multi-line value-comment emission,
 CR-only line comments,
 clean-input number spelling loss,
 and the clean fast-path depth bypass),
 matching it is not the acceptance criterion;
 those defects are tracked for correction in both implementations after foundation adoption.

### External consumer depth lifecycle and comment-style correction

A separate scratch project at `~/temp/agent/jsonc-foundation-diff/`
 now imports each parser candidate,
 parses 512-level arrays and records with an innermost `/*deep*/` value comment,
 drops the source string,
 emits,
 reparses through the same candidate,
 clones and compares the deep value,
 then drops every owned tree.
 An initial assertion comparing the whole pre-emission and post-emission tree **failed**:
 the canonical emitter writes this single-line block comment as a `//` trailing comment,
 so reparsing changes `CommentKind::Block` to `CommentKind::Line` while preserving its value owner and body
 (`~/temp/agent/jsonc-parser-probe-2026-09-24/src/emit.rs:95-97,130-132`,
 `src/comment.rs:114-115`).
 It was not evidence of value-to-key comment migration.
 The corrected test walks the full depth without recursion,
 asserts no container or key acquired the scalar comment,
 checks the innermost number and comment body separately,
 and checks clone equality before dropping.
 Both bounded debug and optimized release suites passed seven tests,
 including this external-consumer boundary case for **both** candidates.
 These results do not validate memory safety of Biome's unsafe upstream implementation by themselves.

### Malformed overdepth lifecycle

A common malformed source of 2048 array openers followed by an incomplete `{"a":`
 was tested within 2 GiB/2 CPU containers.
 The owned parser checks its frame count **before** opening container 513
 (`~/temp/agent/jsonc-parser-probe-2026-09-24/src/parse.rs:218-229`);
 its new test returned `JSONC nesting too deep` at byte offset 512.
 The owned bounded debug and optimized release suites each passed 23 tests.

The Biome wrapper at the time of that measurement called `biome_json_parser::parse_json`
 **before** scanning tokens for depth,
 so its guard was postparse only.
 The fixture produced upstream error diagnostics,
 preserved source text through syntax traversal,
 dropped the upstream parse and returned an editor error without a crash;
 bounded debug and optimized wrapper suites each passed 16 tests with stage markers
 before/after parse,
 drop and editor rejection.
 That result bounds cleanup for one 2048-container malformed source,
 not the work Biome performs before a postparse guard.

A dedicated bounded process then measured the unbounded case.
 With 8192 nested arrays and a scalar leaf,
 upstream parsing succeeded,
 and **dropping** that `JsonParse` aborted the test process with
 `thread 'tests::very_deep_before_guard_lifecycle' has overflowed its stack`
 (SIGABRT,
 exit 101) inside the 2 GiB/2 CPU debug container,
 before the wrapper's postparse token walk could run.
 The 2048-container malformed source and the 512/513 boundary cases did not overflow in the same bounds,
 so this is a depth-dependent destruction failure rather than a general parse failure,
 and it is a separate incident from the Biome CLI/worker reports in the maintenance snapshot.
 The source mechanism matches the measurement:
 green nodes own children inline
 (`biome_rowan-0.5.7/src/green/node.rs:35-49,63-82`),
 `Drop for ThinArc` rewraps into `Arc` (`src/arc.rs:445-453`),
 and `Arc::drop_slow` frees that allocation through `Box::from_raw` (`src/arc.rs:79-83`),
 so each nesting level adds a destructor frame even though parsing itself uses an explicit stack
 (`biome_json_parser-0.5.7/src/syntax.rs:152-230`).

The scratch wrapper now runs a **preparse** depth preflight
 (`~/temp/agent/jsonc-regex-audit/biome/depth.rs`) that scans quoted strings,
 both comment forms and container delimiters in one byte pass,
 rejecting a 513th opener,
 a mismatched closer,
 or an unterminated string or comment before `parse_json` is called
 (`lib.rs:40-46`).
 Controls passed in bounded debug and optimized runs:
 quoted and commented delimiter text does not consume budget,
 `[}` and an unterminated block comment are rejected,
 the previously fatal 8192-container source now returns `JSONC nesting too deep`
 from a dedicated bounded process in both profiles,
 and the wrapper suites passed 18 tests each
 (the one ignored test is that dedicated deep process).
 After the change the Biome projection suites passed 9 tests and the differential suites 8 tests in both profiles.
 This preflight bounds **container depth only**:
 it is not a general allocation,
 CPU or malformed-input budget,
 and it repeats trivia scanning the parser also performs.
 A product Biome adapter needs this gate,
 or an upstream depth policy,
 before it can be called stack-safe against adversarial nesting.

### Unterminated-string helper hazard behind the projection boundary

Published 0.5.7 lexes an unterminated quoted string as an ordinary `JSON_STRING_LITERAL`
 (`biome_json_parser-0.5.7/src/lexer/mod.rs:619-629`)
 and completes a well-typed `JSON_STRING_VALUE` from it (`src/syntax.rs:55-59`).
 The public helper `biome_json_syntax::inner_string_text` then strips one byte from each end
 (`biome_json_syntax-0.5.7/src/lib.rs:116-125`),
 so the one-byte source `"` produces an inverted range and panics in
 `TextRange::new` (`biome_text_size-0.5.8/src/range.rs:64-68`).
 Full trace,
 red/green consumer evidence and the minimal clamp prototype are in
 [`biome-json-syntax-unterminated-string-range.md`](../troubleshooting/biome-json-syntax-unterminated-string-range.md).

The scratch adapter is not exposed:
 its diagnostics gate (`~/temp/agent/jsonc-regex-audit/biome/lib.rs:46`) rejects error trees before any typed
 string helper runs,
 and a new bounded test asserts both that rejection and the raw helper's panic under `catch_unwind`.
 Upstream fixed the cause in its lexer ([PR 2621](https://github.com/biomejs/biome/pull/2621),
 closing [issue 2357](https://github.com/biomejs/biome/issues/2357)),
 but the latest published `biome_json_parser` is still 0.5.7,
 `biome_json_syntax` 0.7.0 keeps the same unguarded helper,
 and parser 0.5.7 requires the 0.5.7 syntax line
 (`biome_json_parser-0.5.7/Cargo.toml:44-51`).
 Any Biome composition therefore inherits a version-pinned hazard that only a diagnostics-first
 consumer discipline (or a fork) neutralizes.
 This is a panic,
 not demonstrated memory unsafety.

### Published Biome unsafe-path audit

`biome_json_syntax::JsonSyntaxKind` is `#[repr(u16)]` with consecutive variants ending at `__LAST`
 (`src/generated/kind.rs:6-49`),
 and its `From<u16>` asserts `d <= __LAST` before `transmute` (`src/lib.rs:15-20`),
 so that conversion cannot create an out-of-range discriminant;
 an invalid raw kind panics instead.

The JSON lexer's `current_char_unchecked` reads through `get_unchecked` and ends with
 `unreachable_unchecked` (`biome_json_parser-0.5.7/src/lexer/mod.rs:204-227`),
 guarded only by `debug_assert!` boundary checks (`:242-246`).
 Its advance discipline keeps the cursor on character boundaries:
 `advance_byte_or_char` moves one byte for ASCII and a whole character otherwise (`:273-279`),
 `advance_char_unchecked` adds `len_utf8` (`:287-290`),
 and the string and identifier loops advance by whole characters for non-ASCII bytes (`:581,693-706`).
 All seven `current_char_unchecked` call sites
 (`:287,322,344,545,654,693,700`) were read and each is reached with the cursor at a character start
 under that discipline;
 the malformed-escape and control-character arms change lexer state so the following iteration advances,
 which is why they do not spin.
 This is an inspection result on published 0.5.7,
 **not** a proof:
 release builds disable those assertions,
 and any future call site that advances by a single non-ASCII byte would break the invariant silently.

`biome_rowan` manages green nodes through refcounted raw pointers
 (`src/arc.rs:57-83,174-208,445-453`;
 `src/green/node.rs:35-49,63-82,293-296`),
 and its destruction recurses per nesting level,
 which the measured 8192-container overflow confirms at runtime.
 No use-after-free or aliasing violation was demonstrated in the inspected paths.

By contrast,
 a targeted search of the owned scratch parser and exact-number prototype sources
 (`~/temp/agent/jsonc-parser-probe-2026-09-24/src`,
 `~/temp/agent/jsonc-exact-number-probe/src`) found **no** `unsafe` block at all.
 The two finalists therefore differ in kind,
 not merely in degree,
 on the mandatory memory-safety constraint:
 one has no unsafe surface,
 and the other requires an argument about invariants that release builds do not check.
 That difference belongs in the scoring rubric and in any adoption rationale.

### Bounded parse-performance preparation

The scratch differential consumer now has `src/bin/bench.rs` and a `mise run bench:isolated` task,
 with the same inspected third-party command tree and a separate invocation recorded **before execution** in
 `~/temp/agent/jsonc-differential-execution-manifest.md`.
 Its fixed JSONC source has comments,
 nested records,
 an escaped unpaired surrogate and raw exact-number spellings.
 The binary compares the candidate editor nodes for equality before timing,
 warms both paths,
 measures repeated unchanged-build samples on the owned path,
 then Biome projection,
 then the owned path again to show timing drift.
 This measures complete parse-to-editor-value paths,
 not Biome syntax parsing alone.
 The first optimized container run measured one fixed 1065-byte JSONC document,
 with 256 parse-to-value calls per sample and seven samples per candidate.
 The owned baseline before Biome ranged from 22234 to 22409 nanoseconds per call;
 after Biome it ranged from 21851 to 26607.
 The Biome projection ranged from 98452 to 116553 nanoseconds per call.
 These measured bands do not overlap on this document,
 so the owned path had the lower measured parse-to-editor-value time for this run.
 A second unchanged-source optimized container run used the same document and sample counts.
 Its owned baseline ranged from 21578 to 22258 nanoseconds per call before Biome
 and 21296 to 22660 afterward;
 the Biome projection ranged from 98076 to 100724.
 The bands remained non-overlapping in both measured runs for this fixed parse-to-editor-value workload.
 This is not a claim about other document shapes,
 machine targets or the cost of future product integration.
 Performance ratings still await the remaining finalist and rubric evidence.

### Workload-shape performance measurement

The single 1065-byte document left the ranking sensitive to one shape,
 so `src/bin/bench.rs` now measures six fixed shapes chosen from source-level cost differences:
 the historical mixed baseline (comments,
 nesting,
 an escaped lone surrogate and exact-number spellings),
 64 nested records,
 a 512-element wide array,
 a comment-dense record with a key block comment and a same-line value comment on every member,
 32 long escaped strings,
 and 32 numbers with 40-digit exponents.
 Each shape first asserts that both candidates return **identical** editor values,
 then warms both paths,
 then samples the owned path,
 the Biome projection and the owned path again on one unchanged optimized binary
 (7 trials per sample set,
 64 to 256 parse-to-value calls per trial,
 inside the same 2 GiB/2 CPU offline container).

Measured nanoseconds per parse-to-editor-value call,
 as minimum to maximum of the seven trials:

- mixed baseline,
   1065 bytes:
   owned 16145 to 18559 before and 20950 to 21954 after;
   Biome projection 100092 to 104812.
- 64 nested records,
   385 bytes:
   owned 13119 to 15509;
   Biome 89029 to 95055.
- 512-element wide array,
   1939 bytes:
   owned 96153 to 107462;
   Biome 350888 to 379546.
- comment-dense 64 members,
   2395 bytes:
   owned 24303 to 26672;
   Biome 112099 to 114478.
- 32 long escaped strings,
   1837 bytes:
   owned 11763 to 12976;
   Biome 53115 to 54820.
- 32 numbers with 40-digit exponents,
   1667 bytes:
   owned 12739 to 13427;
   Biome 51336 to 54559.

The owned band is below the Biome band in every shape,
 with no overlap,
 at roughly 3.5 times (wide array) to 7 times (nested records) slower for the projection path.
 The baseline shape also shows why bracketing matters:
 the owned band after the Biome samples (20950 to 21954) does not overlap its own earlier band
 (16145 to 18559),
 a drift of about 25 percent on an unchanged binary,
 yet that drift is far smaller than the cross-candidate gap.

Scope limits:
 these are scratch prototypes rather than a product crate,
 they measure parse-to-editor-value only (not immutable edit operations,
 emission,
 or peak allocation),
 they ran on one Linux x86-64 host in a shared container,
 and the Biome path includes the adapter cost that a product composition would also pay.
 No memory-footprint measurement was taken.

## Existing-parser contract exits

These are outcomes for published implementations **as-is**,
 not proof that every possible fork,
 token adapter,
 or new parser implementation is impossible.

- `jsonc-parser` 0.33.2:
   `src/scanner.rs:175-201` calls `parse_string_with_char_provider` for escapes and `src/string.rs:214-261` rejects an
   unpaired high or low surrogate.
   This is a hard failure for the required accepted string domain.
   It also does not supply the normalized,
   attached-comment model as-is.
- `jwc` 0.1.0:
   the published `src/single_pass_parser.rs:201-222` explicitly errors on a low surrogate or a high surrogate not
   followed by a low one.
   It fails the string-domain hard gate as-is,
   regardless of its regex-free production dependency graph.
- `fjson` 0.3.1:
   `src/ast.rs:64,83` limits container nesting to 128.
   The TypeScript structured parser permits depth through 512 (`package/module/jsonc-edit/src/parse.ts:21,55`),
   and its comment-free fast path delegates to `JSON.parse`;
   the published `fjson` parser cannot preserve the supported depth domain as-is.
   `src/scanner.rs:326-330` also skips non-JSON Unicode whitespace.
   A scanner-only use paired with our own grammar/parser is a distinct composition,
   not an as-is editor.
- `json-five` 0.3.1:
   [prior corpus probes](../planning/monorepo-manager-route-research/rust-structured-edits.md) show JSON5-only syntax
   and scalar roots accepted,
   where the TypeScript JSONC conformance tests reject them;
   it is not a strict JSONC parser as-is.
   Its [block-comment tokenizer defect](../troubleshooting/json-five-unterminated-block-comment.md)
   occurs **before** the round-trip printer or an external emitter sees comments.
   The published raw-token API therefore loses a supported comment byte as-is;
   a source patch or independently verified span repair plus strict validator would be custom code.
- `hifijson` 0.5.0:
   [official crate documentation](https://docs.rs/hifijson/0.5.0/hifijson/) describes JSON token and value lexers,
   not JSONC comment tokens or separately attached key/value comments.
   It is not a ready-to-use implementation of this editor;
   coupling its scalar lexer to an owned comment scanner is a distinct composition.

The documented hard exits make the owned-parser translation eligible for serious evaluation under the
existing-tools-first rule;
 they do not establish it as a validated winner.
 `biome_json_parser` 0.5.7,
 `fjson` scanner-only,
 a patched or span-repaired `json-five` raw-token adapter,
 and a repository-owned parser remain different unvalidated compositions.
 Regex-free dependency screening does not imply semantic parity.

## Repository-owned parser and emitter prototype

A disposable Rust crate was written under `~/temp/agent/jsonc-parser-probe-2026-09-24/`,
 with a separate consumer under `~/temp/agent/jsonc-parser-consumer-2026-09-24/`.
 This code is outside the product repository and is **not** an adopted implementation.
 Its `src/scan.rs` uses direct byte/index scans,
 decodes escaped `\uXXXX` into owned UTF-16 units (including lone surrogates),
 and delegates JSON number identity to the local dependency-free scratch prototype.
 `src/parse.rs` attaches comments separately to object keys and values and rejects scalar roots/JSON5.
 `src/emit.rs` emits a canonical two-space tree with trailing commas and retains raw scalar spellings,
 including clean `1e0` input.
 No regex,
 generated lexer,
 external crate,
 or unsafe code was introduced by this prototype.

- `cd -- ~/temp/agent/jsonc-parser-probe-2026-09-24 && mise run test` passed the initial bounded parser suite over
- valid/invalid syntax,
   key and value comments,
   line-comment merging,
   unpaired/paired UTF-16 values,
   raw exact number text,
   and reparsable canonical output.
   `mise run lint:clippy` also passed with warnings denied after structural lint fixes.
- `cd -- ~/temp/agent/jsonc-parser-consumer-2026-09-24 && mise run test` imported the library by path and printed
- `JSONC consumer parse and emit passed`,
   checking attached key/value comments,
   unchanged `1e0`,
   decoded lone surrogate,
   and reparsed output.
- Positive controls in scratch:
   removing the root-container check made `invalid_documents_fail` fail on input `42`;
   suppressing key comment attachment made `comments_are_addressable` fail.
   Each source edit was restored,
   matching a saved pre-mutation SHA-256,
   and unit/Clippy tasks passed again.
   These controls cover only those guards,
   not all parser branches.
- A new nested-array test invalidated the initial pass:
   `mise run test:isolated` ran the added suite in an offline,
   read-only Podman container capped at 2 GiB memory,
   2 CPUs,
   128 PIDs and 600 seconds.
   The 513-level case aborted the test process with
   `thread 'tests::excessive_nesting_is_rejected' has overflowed its stack` (SIGABRT,
   exit 101) before returning the intended depth error.
   The other test names printed `ok`,
   but the suite failed and the owned parser candidate is **not validated**.
   Structural recursion needs an explicit work stack or another measured stack-safe shape before this depth contract
   can be claimed.
   Never reproduce this case outside a bounded container.

This is a partial parser/emitter probe with a demonstrated stack-safety failure,
 not full parity.
 No immutable edit/navigation/comment-set operations,
 full TypeScript conformance corpus,
 fuzzing,
 mutation campaign,
 cross-platform native check,
 release packaging,
 or performance comparison is provided.
 The comment attachment rule for blank-line-separated comments still needs the agreed correction and tests in both
 maintained implementations.
 Do not use this prototype as publication evidence.

### Accepted-depth positive control also failed

A separate filtered probe `mise run test:depth-512` generated a valid array document with 512 nested containers and a
scalar leaf.
 It ran solely inside the same 2 GiB/2 CPU,
 network-free container with a 600-second ceiling.
 The test process aborted with `thread 'tests::depth_512_is_accepted' has overflowed its stack` (SIGABRT,
 exit 101),
 before it could assert a successful parse.
 Thus a pre-scan that merely rejects nesting beyond 512 would not remediate the supported-depth case.
 The parser's recursive container-to-child calls must be replaced by an explicit work stack;
 once parsing succeeds,
 dropping and emitting a deeply nested tree must be measured separately because they can also recurse.
 Do not run these inputs on the host until the stack-safe design has passed the isolated positive control.

### Localized parse-stage stack overflow

The accepted-depth test was instrumented with unbuffered markers before parse,
 after parse,
 and before/after drop.
 `mise run test:depth-512` was rerun inside the 2 GiB/2 CPU bounded container with Cargo test output uncaptured.
 It printed `depth control: before parse` and then aborted with
 `thread 'tests::depth_512_is_accepted' has overflowed its stack` (SIGABRT,
 exit 101).
 Neither the after-parse nor drop markers appeared.
 This localizes the observed failure to parser execution,
 not successful tree destruction.
 Emission and destruction remain unverified separate boundaries.
 Replace recursive parsing with explicit container frames before any candidate recommendation.

### Separate TypeScript JSONC comma discrepancy

A direct call through the current TypeScript neutral bundle (`package/module/jsonc-edit/dist/final/neutral/index.mjs`)
measured the `JSON.parse` fast path and structured parser separately.
 Clean `[1\n,2]` parsed as a positive control.
 `[1\n,2,]` forced the structured path and threw `JsoncParseError: expected , or ] in array (at offset 3)`;
 `[1 /* c */\n,2]` also threw at offset 11.
 `{"a":1\n,"b":2,}` threw `JsoncParseError: expected , or } in object (at offset 7)`.
 The visible boundary differs by whether a comment or trailing comma requires the structured path.
 `package/module/jsonc-edit/src/parse-trivia.ts` ends same-line trailing capture at LF,
 and `src/parse.ts` demands a close when it saw no comma.
 This is independent of the Rust prototype's stack overflow.
 The accepted shared-behavior fixtures need an explicit comma-after-trivia case;
 TypeScript and Rust require a fix once the foundation is adopted.
 Do not treat a clean-only fast-path probe as evidence for the structured path.

### Iterative parser after-state, lifecycle checks still open

The scratch Rust `parse.rs` was rewritten to use an explicit `Vec<Frame>` with separate item-or-close,
 awaiting-child,
 and separator-or-close phases.
 The parser now rejects an opener when 512 containers are already open,
 rather than recursing between `parse_value` and container functions.
 `mise run lint:clippy` passed with warnings denied.
 `mise run test:isolated` then ran the full scratch unit suite in the same offline,
 read-only Podman image under 2 GiB memory,
 2 CPUs,
 128 PIDs and a 600-second cap.
 It exited 0:
 the valid 512-level array case,
 the rejected 513-level array case,
 and the other parser tests passed.
 The 512-level test explicitly drops the returned node,
 so this measured case no longer aborts during parse or basic drop.
 This is after-state evidence,
 not a prediction from the rewrite.

The result does **not** yet prove stack-safe emission,
 reparse,
 clone/equality,
 nested records,
 error cleanup after a deep completed subtree,
 or all comment ownership after canonical output.
 The parser remains a disposable incomplete candidate,
 not a selected foundation or product crate.
 The earlier isolated stack-overflow logs are retained as before-state evidence,
 not erased by the new pass.

### Separate multi-line value-comment ownership failure

The isolated scratch test `multiline_value_comment_stays_on_value` failed under the same bounded container while the
other parser tests passed.
 Source `{"k":/*value\ncomment*/1}` parsed with its block comment on the value;
 the scratch emitter wrote `/*value\ncomment*/` before the key,
 and reparse attached it to the key.
 The assertion at scratch `src/tests.rs:124` reported `value comment migrated to key` (exit 101).
 This is not the earlier stack overflow:
 both 512-level and 513-level depth tests completed under the explicit-frame parser in this run.

A separate direct probe of the built TypeScript library in `package/module/jsonc-edit/dist/final/neutral/index.mjs`
reproduced the same user-visible change.
 Before emission,
 `jsoncGetKeyComment({ path: ['k'] })` returned `COMMENT_ABSENT` and `jsoncGetComment` returned
 `{ type: 'block', text: 'value\ncomment' }`;
 `jsoncStringify` placed that block before `"k"`;
 after reparse the key carried that comment and the value returned `COMMENT_ABSENT`.
 The source `package/module/jsonc-edit/src/stringify.ts` emits `valLead` before the key.
 This is a confirmed ownership bug in both implementations,
 not a whitespace-only difference.
 The accepted shared-behavior contract requires the same regression fixture and a fix in both maintained packages after
 foundation adoption.
 The Rust prototype needs to emit multi-line value comments after the key's colon so the parser attaches them to the
 value;
 that remedy has not yet been applied or measured.

### After-state: Rust scratch value-comment owner retained

The scratch emitter was changed to write a multi-line value comment **after the object key's colon**,
 so the parser's between-colon-and-value trivia channel owns it.
 In the same 2 GiB/2 CPU offline container,
 `mise run test:isolated` exited 0 with the `multiline_value_comment_stays_on_value` regression passing alongside the
 rest of the bounded parser suite.
 `mise run lint:clippy` passed with warnings denied before the isolated run.
 This measures the Rust scratch fix at parse/emit/reparse,
 not merely a predicted effect from source.
 The maintained TypeScript bundle still demonstrates the original migration and needs the corresponding fix and shared
 fixture after foundation adoption.
 The scratch parser remains incomplete until deep emission,
 lifecycle and other required branches pass.

### Accepted-depth lifecycle after-state

The scratch parser's explicit frame stack passed the full `mise run test:isolated` suite in a pinned offline Rust
container with 2 GiB memory,
 2 CPUs,
 128 PIDs,
 and a 600-second ceiling.
 The run reported 17 passing unit tests and no failures.
 Explicit stage markers in separate tests confirmed a 512-container array and record parsed,
 emitted,
 reparsed,
 and dropped;
 a 512-array clone and equality check returned and both values dropped;
 malformed input after a deep completed child and after a deep root returned errors without an abort.
 A 513th opener was rejected with `JSONC nesting too deep` for scalar- and empty-container cases,
 while many shallow sibling containers remained accepted.
 These checks used the debug build;
 release and non-Linux targets remain unmeasured.

The same bounded suite covered valid comma-after-newline JSONC,
 key/value comment queries,
 empty-container comment order,
 exact raw numbers,
 and UTF-16 lone surrogate strings.
 A previous failing `multiline_value_comment_stays_on_value` test became passing only after the scratch emitter moved
 the multi-line value comment after its key's colon.
 The TypeScript production package remains unfixed on the independently measured comma and comment-owner incidents.
 This is still a feasibility prototype,
 not an immutable edit API,
 full conformance campaign,
 or publication-ready crate.

### Bounded optimized-build after-state

The disposable parser and exact-number crates were rebuilt in the same pinned,
 offline 2 GiB/2 CPU container using `mise run test:isolated:release` (`cargo test --release --locked --offline`).
 The optimized suite reported 17 passing unit tests and no failures,
 including 512-level array/record parse,
 canonical emission,
 reparse,
 clone/equality,
 success and error cleanup,
 and 513th-opener rejection.
 Stage markers printed after each deep operation.
 This complements the earlier debug run;
 it is not a timing comparison,
 a macOS/Windows result,
 or a consumer test of a published crate.
 After the iterative rewrite and value-comment correction,
 `mise run test` in the separate scratch consumer `~/temp/agent/jsonc-parser-consumer-2026-09-24/` rebuilt the library,
 called parse and emit,
 and printed `JSONC consumer parse and emit passed`.
 This is a local path dependency,
 not published-crate validation.

A later bounded debug run and optimized release run each reported 18 passing unit tests after adding
 the maintained TypeScript corpus's inline-comment-after-comma syntax,
 direct value-comment ownership checks,
 more escaped JSON string forms,
 and malformed-string cases.
 These are selected syntax and attachment checks,
 not a full semantic comparison of the TypeScript conformance or property corpus.
 Subsequent bounded debug and optimized release runs also passed adversarial edited comment bodies on both key and
 value owners,
 including block terminators,
 quotes,
 escapes,
 control characters,
 blank lines,
 carriage returns and non-ASCII text.
 It checked reparsability,
 retained text on the same owner,
 and a second-to-third emission fixpoint;
 it does not replace the generated TypeScript property corpus.

Remaining validation includes full TypeScript conformance and property cases,
 wider comment-placement and syntax-boundary coverage,
 mandatory package style lint,
 native target matrix,
 full immutable edit/navigation interface,
 and performance on stable-band measurements.
 The chosen foundation is still an unadopted scratch prototype.
 A later `deep_record_clone_and_equality` test added an independent 512-record clone,
 equality and drop lifecycle to the earlier array-only clone control.
 The complete scratch suite and explicit post-operation markers passed in both bounded debug and optimized release runs,
 with Clippy warnings denied before the debug run.
 This checks the current tree shape,
 not the public immutable edit API that has yet to be built.

## Maintained TypeScript depth fork and evaluation boundary

The user requested supported behavior rather than parser-algorithm imitation.
 A direct public `parseJsonc` probe in the disposable 2 GiB/2 CPU Node 26 container
 (`~/temp/agent/jsonc-ts-depth-envelope/probe.mjs`) measured:
 clean 512 and 513 nested arrays both returned a `plainJson` node;
 a commented 512 nested array returned structured `array`;
 a commented 513 nested array threw `JsoncParseError: nesting too deep (at offset 513)`.
 `package/module/jsonc-edit/src/parse-jsonc.ts:40-44,108-121` returns
 `JSON.parse` fast-path values without the structured parser's documented 512-depth check
 (`package/module/jsonc-edit/src/parse.ts:22-31,64-70`).
 This is a **distinct fast-path bypass**,
 not evidence that the user adopted unbounded nesting:
 the accepted `doc/decision/jsonc-edit-parser-foundation.md` says the fast path must not alter public behavior.
 The [port plan](../planning/jsonc-edit-rust-port.md) records the inference to align both maintained
 implementations at the explicit 512-container boundary after foundation adoption.
 The parser finalist check must reject depth 513 for both clean and commented documents
 and accept depth 512 with safe parse,
 emit,
 clone/equality and cleanup.
 Do not claim Rust parity solely from the TypeScript structured path's source guard.

## Cross-target compilation of both parser consumers

The current machine reports Linux x86_64 and only the
 `x86_64-unknown-linux-gnu` Rust target installed.
 Rather than equating Linux consumer tests with all native platforms,
 a secret-free 2 GiB/2 CPU Rust 1.97.1 container fetched the
 `x86_64-pc-windows-gnu` and `aarch64-apple-darwin` standard-library targets in separate runs.
 From `~/temp/agent/jsonc-foundation-diff/`,
 `mise run check:windows` and `mise run check:macos` both completed
 `cargo check --locked --target <triple>` with exit status 0.
 The checked graph included the owned parser,
 exact-number prototype,
 published Biome 0.5.7 parser and syntax dependencies,
 Biome projection,
 and the external differential consumer.
 Sources were mounted read-only;
 Cargo target data lived in container-local `/target` under a 1 GiB tmpfs.
 The local image ID was `a0635962c16d5f26400703edd4317175cf9531f3285d632613f9da227f9c71d1`.
 This verifies type/build compatibility for those targets,
 **not** executable runtime behavior on Windows or macOS,
 ABI safety in Biome's unsafe code,
 or packaging of an adopted product crate.

## Scoring, sensitivity and recommendation

Two survivors received equal-depth validation.
 Ratings use the frozen criteria at weight 1 each,
 on a 1 to 5 scale where 5 is best.
 Every rating cites evidence recorded in this report.

### Repository-owned parser and emitter translation

- Source auditability:
   **5**.
   Every line is repository code;
   a targeted search of both scratch crates found no `unsafe` block at all.
- API clarity for the comment model:
   **5**.
   Keys and values carry separate normalized comments in the value type,
   and the independent UTF-16,
   comment-owner and conformance controls passed.
- Dependency and build surface:
   **5**.
   No third-party runtime dependency beyond the owned exact-number prototype,
   so no graph to re-audit on a version bump.
- Maintenance evidence:
   **3**.
   Maintenance is internal and permanent,
   with no upstream abandonment risk but also no external reviewers,
   fuzzers or release engineering.
- Measured parse performance:
   **5**.
   Lower band on all six workload shapes,
   by roughly 3.5 to 7 times.
- Total:
   **23 of 25**.
- Pros:
   no unsafe surface;
   no dependency graph;
   conformance,
   CR and CRLF handling,
   comment ownership and the depth boundary are all fixable in one place;
   fastest measured path;
   smallest license and notice burden.
- Cons:
   this port must write and maintain a parser,
   emitter and their tests;
   no external ecosystem review;
   every future JSONC edge case is ours to find;
   the prototype is not yet a product crate
   (no public immutable edit API,
   no fuzzing,
   no packaging).

### `biome_json_parser` 0.5.7 with the scratch projection adapter

- Source auditability:
   **2**.
   Published source is inspectable,
   but the selected graph carries unsafe pointer ownership in `biome_rowan`,
   release-disabled lexer boundary assertions and 87 registry packages;
   the audit here covered the deciding paths,
   not the whole surface.
- API clarity for the comment model:
   **3**.
   Comments live as trivia on punctuation tokens;
   the adapter reconstructs key and value ownership and now passes the same controls,
   but two ownership defects had to be found and fixed during validation.
- Dependency and build surface:
   **1**.
   87 selected registry packages including proc-macros and build scripts;
   product notice assembly is still open.
- Maintenance evidence:
   **2**.
   The Biome project is active,
   but the published parser crate line is stale
   (0.5.7 predates the upstream fix for a known panic),
   internal crates publish on demand,
   and the newer workspace source is not what a consumer installs.
- Measured parse performance:
   **2**.
   Higher band on all six shapes.
- Total:
   **10 of 25**.
- Pros:
   lossless CST with byte-exact source retention;
   mature error recovery and diagnostics;
   substantial upstream engineering and test corpus;
   raw scalar tokens,
   including escaped lone surrogates,
   without a custom decoder.
- Cons:
   the mandatory safety and stack gates hold only because of adapter code this port would own permanently
   (diagnostics gate,
   preparse depth preflight,
   source-length guard);
   a measured stack overflow while dropping an 8192-deep tree;
   a published helper that panics on an unterminated string;
   3.5 to 7 times slower parse-to-value;
   a large dependency and notice surface;
   no published crate release carrying the upstream lexer fix.

### Ranking and sensitivity

Ranking:
 repository-owned translation first,
 Biome composition second.
 Adjacent-pair reason:
 the owned translation is rated at least as high on all five frozen criteria and strictly higher on four,
 so no non-negative weighting reverses the order.

Sensitivity:
 the owned candidate leads by 13 points.
 Reversing the order would require the Biome composition to gain 13 points relative to it;
 the largest swing any single criterion can produce on this scale is 4,
 so at least four criteria would have to flip to their maximum in Biome's favor.
 Two plausible re-ratings were computed explicitly:

- Equal maintenance evidence (3 and 3) gives totals of 23 and 11.
- Biome rated best on maintenance (5) and the owned candidate worst (1) gives totals of 21 and 13.

Neither changes the order,
 so the ranking is insensitive to any single-criterion disagreement and to any combination of up to three.

Hard-gate note:
 the Biome composition satisfies the mandatory memory-safety constraint **only** with consumer-side gates this port
 would own.
 That is a design obligation rather than an upstream property,
 and it is why its auditability rating is low instead of failing.

### Recommendation

Recommend the **repository-owned parser and emitter translation**,
 together with the repository-owned exact-number identity prototype recorded in the companion number report,
 as the native Rust JSONC foundation.

This is a recommendation for a **separate adoption decision**.
 Nothing here authorizes product code,
 a dependency change,
 a decision record or publication.
 If adoption is granted,
 the remaining work is the product crate,
 shared fixtures,
 the maintained TypeScript defect fixes and the publication route.
 If it is refused,
 the Biome composition is the only other validated path and carries the obligations listed above.

## Remaining validation before publication

- Product-crate boundary:
   the validated code is scratch prototypes,
   not `package/rust-module/jsonc-edit`.
   Packaging,
   the public immutable edit API,
   documentation and repository lint and type gates are unbuilt.
- Platform runtime:
   cross-target `cargo check` passed for Windows GNU and macOS ARM64,
   but no executable ran on either;
   only Linux x86-64 runtime behavior is measured.
- Fuzzing and property testing at product level.
   The differential corpus is bounded (5635 generated inputs) and is not a proof of semantic equivalence.
- Memory footprint and allocation behavior were not measured for either candidate.
- License notice assembly for the chosen foundation:
   trivial for the owned path,
   87 packages for the Biome path.
- Publication route:
   an authorized crates.io token path for the first release remains unverified,
   and name availability must be rechecked immediately before publishing.
