# Vet exact JSON number storage for the Rust jsonc-edit port

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/jsonc-edit` is now `package/jsonc-edit/core-rust`;
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

## Status and fingerprint

- Status:
   discovery saturated for the frozen schedule,
   the single survivor validated and scored with sensitivity,
   and a recommendation recorded.
   **Not adopted**:
   adoption is a separate user decision,
   and no dependency,
   product code,
   decision record or publication followed.
- Subject:
   jsonc-edit Rust exact-number foundation.
- Scope:
   Choose exact JSON numeric value representation for the native published `monochromatic-jsonc-edit` crate.
- Started and last updated:
   2026-09-24.
- Owner:
   current coding-agent session.
- Governing skill:
   `.agents/skills/choosing-technology/SKILL.md` at `a05818ad7`,
   SHA-256 `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`.
- Fingerprint,
   RFC 8785 canonicalized schema version 1:
   `e8e0034aeb9c0f9b86aa17cd701ca3a0886ff81b49bc2c552004a2ec33138876`.
- No prior compatible report was found under `doc/audit/` by a `rg --files` search for JSONC,
   Rust parser,
   or structured edits.
- Base category:
   inspectable open-source local technology,
   including repository-owned implementation as the baseline.
- Overlays:
   native Rust library,
   multi-platform.
   No credential or CI execution in the candidate library;
   untrusted JSONC text crosses the parser boundary.
   No replacement of the TypeScript library,
   which stays maintained.
- Hard constraints:
   exact equality over admitted JSON numbers,
   original spelling for unedited literals,
   no silent rounding,
   LGPL-3.0-or-later compatible licensing,
   inspectable source and reproducible validation,
   and no regex-backed production number parsing in required dependencies.
- Deployment:
   native Rust on Linux,
   macOS and Windows,
   published to crates.io.
   Browser and Wasm not required.
- Preference weights:
   none beyond the confirmed requirements in `doc/planning/jsonc-edit-rust-port.md`.

## Frozen criteria before candidate rating

Every validated candidate receives the same weight of 1 for each relevant soft concern:
 API clarity for exact values,
 source auditability,
 release/maintenance evidence,
 dependency and build surface,
 and measured performance on the library's input corpus.
 The maximum for each is 4.
 Ratings,
 totals,
 sensitivity reruns and the ranking are recorded in the scoring section.
 Hard constraints are not scored.
 Sensitivity must rerun each weight from 1 through 5 and each uncertain rating endpoint before any recommendation.

## Query ledger and saturation

The literal numeric query schedule was frozen in `doc/planning/jsonc-edit-rust-port.md` before these calls.
 Required source classes are registry,
 repository host,
 broader web,
 and in-repo precedent.

- Registry:
   `cargo search 'exact decimal' --limit 100` returned a truncated result with 1139 additional hits.
   `cargo search bigint --limit 100` returned 932 more.
   `cargo search 'arbitrary precision json' --limit 100` returned 406 more.
   No negative filter,
   no sort control,
   first result page only;
   registry saturation is **blocked until a paginable enumeration route is proven**.
- Repository host:
   `gh search repos 'rust arbitrary precision decimal json' --limit 100 --json fullName,url,description,updatedAt`
   returned no matches;
   `gh search repos 'rust bigint' --limit 100 --json fullName,url,description,updatedAt` returned five.
   No negative filter or explicit sort.
   These queries are not proof that comparable projects are absent.
- Web:
   searches for `Rust exact JSON number decimal exponent arbitrary precision source crate`
   and `Rust JSON number lexeme exact comparison library` returned primary project documentation for `json-number`,
   `bigdecimal`,
   `serde_json`,
   and `jstrict`.
   No negative filter.
- In-repo:
   `package/module/jsonc-edit/src/value.ts`,
   `src/scan.ts`,
   and `doc/planning/monorepo-manager-route-research/rust-structured-edits.md` document JS-number decoding,
   raw literals on the structured path,
   and integer-equality pitfalls.
   The clean fast path loses `1e0` spelling;
   the reproducible bundle probe is in `doc/planning/jsonc-edit-rust-port.md`.
- One de-duplicated taxonomy expansion round may follow the initial schedule;
   record it before running.
   Registry and repository-host pagination remain open.

## Candidate ledger

### Repository-owned validated token and exact decimal equality

- Discovery:
   existing raw-token model and Rust native port requirements in `doc/planning/jsonc-edit-rust-port.md`.
- Category:
   inspectable repository-owned local implementation;
   same native and multi-platform overlays.
- Screening:
   pending proof of full JSON-number grammar,
   exact equality across arbitrary exponents,
   complexity bounds,
   and consumer API.
   It is not automatically eligible merely because external alternatives require vetting.

### `json-number` 0.4.10

- Discovery:
   broader search at [docs.rs Number](https://docs.rs/json-number/latest/json_number/struct.Number.html);
   `cargo info json-number` reports [repository](https://github.com/timothee-haudebourg/json-number),
   MIT/Apache-2.0,
   and optional canonical and serde features.
- Category:
   inspectable open-source local crate,
   native and multi-platform overlays.
- Screening:
   lexical equality is source-proven,
   and a separate [layout portability audit](../troubleshooting/json-number-unsized-layout.md)
   found no documented representation guarantee for its checked constructor's unsafe slice-to-wrapper conversion.
   The published crate fails this port's mandatory portable safety-proof gate as-is;
   a corrected fork is a separate unvetted candidate.
   Not recommended.

### `bigdecimal` 0.4.10

- Discovery:
   [docs.rs package](https://docs.rs/crate/bigdecimal/latest);
   `cargo info bigdecimal` reports [repository](https://github.com/akubera/bigdecimal-rs),
   MIT/Apache-2.0,
   and `num-bigint` dependency.
- Category:
   inspectable open-source local crate,
   native and multi-platform overlays.
- Screening:
   pending source proof for exponent range,
   formatting,
   exact equality,
   and consumer boundary.
   Not recommended.

### `serde_json` 1.0.151 with `arbitrary_precision`

- Discovery:
   `cargo info serde_json` reports the [upstream repository](https://github.com/serde-rs/json),
   MIT OR Apache-2.0,
   and `arbitrary_precision` feature.
   Previous repository research identifies false equality assumptions in its surrounding wrapper,
   not a verified limitation of this feature.
- Category:
   inspectable open-source local crate,
   native and multi-platform overlays.
- Screening:
   pending proof of number equality,
   raw-token preservation,
   and surrogate handling at the parser boundary.
   Not recommended.

### `jstrict` 0.15.0

- Discovery:
   [number API](https://docs.rs/jstrict/latest/jstrict/number/struct.Number.html) and `cargo info jstrict`,
   reporting [upstream repository](https://github.com/mskvarc/jstrict/),
   MIT OR Apache-2.0,
   and default `simdutf8` dependency.
- Category:
   inspectable open-source local crate,
   native and multi-platform overlays.
- Screening:
   pending source proof that number storage composes with JSONC token parsing.
   Not recommended.

### `rust_decimal` 1.43.0

- Discovery:
   `cargo search 'arbitrary precision json' --limit 100`
   and [upstream docs](https://docs.rs/rust_decimal/latest/rust_decimal/).
- Category:
   inspectable open-source local crate,
   native and multi-platform overlays.
- Screening:
   pending proof of full JSON-number exponent and coefficient domain.
   Not recommended.

## Targeted source screening, still incomplete

All clones sit under private `~/temp/agent/`;
 these reads did not execute upstream command trees.
A dependency plus repository-owned adapter remains distinct from the dependency used alone.
None has yet passed the full vet gates.

- `json-number` 0.4.10 at `283af83`,
   `src/lib.rs:110-120` describes lexical comparison and derives `PartialEq` and `Eq` on the raw bytes:

  ```rust
  /// All the comparison operations are done on this *lexical* representation,
  /// meaning that `1` is actually greater than `0.1e+80` for instance.
  #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct Number {
      data: [u8],
  }
  ```

  As a complete value model this fails `1 = 1.0 = 1e0`.
  The published 0.4.10 archive SHA-256 `479dfd2ad8e4b4ae076b031f72ef2f3791f65e2a0f51e5f3408dbf716c4c2f82`
   matches the generated scratch Cargo.lock checksum.
  Published `src/lib.rs` and private clone `283af83` both hash to
   `467186254f8695eba06e5065759056c48b6b832c89dbe39d43d52b73981deff0`.
  Its selected normal/build graph contains the `lexical` family but no regex-named package
  or build script in the inspected crate roots.
  The checked `Number::new` path calls `new_unchecked` and transmutes `&[u8]` to `&Number`
   (`src/lib.rs:188-210`),
   while `Number` has no `#[repr(transparent)]` (`:110-120`).
  The [Rust Reference](https://doc.rust-lang.org/reference/type-layout.html#the-rust-representation)
   gives no field-offset guarantee for default Rust representation;
   a successful build or consumer test cannot supply that missing cross-platform safety proof.
  The detailed [source audit](../troubleshooting/json-number-unsized-layout.md) does **not** claim observed undefined
  behavior.
  The proposed checked-token plus owned-identity composition was deliberately not executed
   because even its safe API reaches the unproved conversion.
  Exclude the published component as-is on the mandatory portable safety-proof gate.
  A fork with a documented transparent layout
  and pointer conversion would be a distinct candidate requiring its own safety and consumer validation.
- `serde_json` 1.0.151 at `afdf6fc`,
   `src/number.rs:20-25,72-73` derives equality on `Number` and sets its arbitrary-precision inner type to `String`:

  ```rust
  #[derive(Clone, PartialEq, Eq, Hash)]
  pub struct Number {
      n: N,
  }
  #[cfg(feature = "arbitrary_precision")]
  type N = String;
  ```

  That built-in equality is lexical,
   so the feature alone is not an exact semantic value model.
  A custom equality adapter remains a possible composition,
   not yet validated.
- `jstrict` 0.15.0 at `18700a1`,
   `src/number/mod.rs:105-119` explicitly documents lexical ordering and derives `PartialEq` on bytes.
  It is a strict JSON parser,
   not an attached-comment JSONC implementation.
- `reliakit-json` 1.0.0 at `7a7f574`,
   `crates/reliakit-json/src/number.rs:7-20` says equality is structural and `1`,
   `1.0`,
   `1e0` differ;
   it preserves raw spelling but fails exact numeric identity alone.
- `bigdecimal` 0.4.10 at tag `v0.4.10`,
   commit `ea0803e`,
   `src/impl_num.rs:47-55,92-104` parses exponents into `i128` then checks resulting scale fits `i64`:

  ```rust
  (base, i128::from_str(&e_exp[1..])?)
  // ...
  .and_then(|scale| scale.to_i64())
  .ok_or_else(|| ParseBigDecimalError::Other(format!("Exponent overflow when parsing '{}'", s)))?
  ```

  This cannot serve as the only representation for the full JSON number grammar,
   whose exponent has no specified digit cap.
- `scientific` 0.6.0 at `6e75628`,
   `scientific/src/types/scientific.rs:83` stores `exponent: isize`;
   `b10` 1.0.0 at `c039e3f`,
   `src/lib.rs:59-69` declares a base exponent constrained by an `i8` generic.
   Neither covers arbitrary exponent tokens as the only value representation.
- `ordecimal` 0.3.1 at `916c565`,
   `src/decimal.rs:113-130` parses `exp_str` into `i64`,
   so its advertised decimalInfinite encoding does not imply an unbounded input exponent.
  Its parser also accepts a leading plus and trims input (`src/decimal.rs:66-98`),
   so it would require a strict JSON grammar wrapper.
- `decimal-bytes` 0.6.0 at `677a792`,
   `src/encoding.rs:86-90,350-416` bounds the exponent to `MIN_EXPONENT` through `MAX_EXPONENT`,
   with `MAX_EXPONENT = 131_072`.
   It does not cover the admitted JSON-number grammar alone.

The grammar-valid raw-token representation with custom exact normalization remains a serious **candidate**,
 not an adoption.
Compare it with `json-number` plus that normalization after inspecting the latter's license,
 transitive dependencies,
 CI,
 issue history,
 and full test path.

## Registry pagination progress

The first pages of the registry searches returned 100 entries each and advertised more hits.
The crates.io API was read via `web_fetch` for pages 2 and 3 of the original three queries;
 each returned 100 entries.
The full metadata and a transparent keyword filter are saved in `~/temp/agent/jsonc-registry-pages.json`.
The keyword filter can hide numerically relevant crates whose name and description omit its keywords,
 so the full page data is retained.
Notable new leads include `fpdec`,
 `fraction`,
 `dashu-int`,
 `arbi`,
 `b10`,
 `scientific`,
 `ordecimal`,
 and `decimal-bytes`.
No saturation conclusion follows until each page's candidates are screened and the two-page condition holds.
 The [crates.io data-access policy](https://crates.io/data-access) documents an identifying user-agent
 and no more than one API request per second.
 The delegated query `exact decimal` reached alphabetical page 11 with 100 records,
 `meta.total=1239` and a next-page link;
 the earlier `cargo search` cap is not an API pagination limit.
 A direct identified `curl --head` request to a `jsonc` page returned HTTP 200 as a transport control.
 `~/temp/agent/jsonc-registry-enumeration.mjs` now fixes the original and expansion registry terms,
 alphabetical sort,
 serial delay,
 page ceiling,
 and private storage for **unfiltered** complete responses before requesting them.
 The identified API run stored unfiltered complete alphabetical pages for `exact decimal`
 (1240 records through page 13),
 `bigint` (1034 through page 11),
 and `arbitrary precision json` (509 through page 6),
 each ending with `next_page=null`.
 The first run stopped on a self-imposed 50-page ceiling during `json number`,
 not a provider cap.
 A bounded resume reused saved responses and finished `json number` at page 105 with 10460 records,
 then exhausted `decimal arbitrary exponent` on page 1 with 60.
 `mise run audit:registry-validate` checked all six query directories:
 contiguous pages,
 page totals equal to saved record counts,
 and no duplicate crate ids within each query.
 The full unfiltered page files under `~/temp/agent/jsonc-registry-full-pages/` are scratch evidence,
 not a curated candidate ledger.
 Registry **pagination** is complete for the frozen initial and expansion terms;
 the full-record numeric metadata screen is recorded in the next section,
 while saturation of the other required source classes remains open.

## Full numeric-registry metadata screen

`mise run audit:number-registry-leads` re-read every saved page of the five frozen numeric queries
 (`exact decimal`,
 `bigint`,
 `arbitrary precision json`,
 `json number`,
 `decimal arbitrary exponent`),
 scanned **13303** records,
 and applied one inclusive predicate:
 a record becomes a lead when its name or description matches a numeric term
 (`decimal`,
 `bigint`,
 `bignum`,
 `rational`,
 `exact`,
 `arbitrary precision`,
 `lossless`,
 `fixed point`,
 `num`/`number`,
 `numeric`)
 **and** a JSON or token term
 (`json`,
 `jsonc`,
 `token`,
 `lexeme`,
 `lexical`,
 `serial`,
 `parse`,
 `value`,
 `literal`,
 `spelling`).
 The run reported **87** leads with query,
 page and record index,
 saved at `~/temp/agent/jsonc-number-registry-leads.jsonl`.
 The predicate is deliberately over-inclusive,
 but a crate whose name and description avoid both term families could still be missed;
 this is a recorded metadata screen,
 not proof that every numeric crate was considered.

Source-verified exits among those leads:

- `bignumber` 0.1.1 wraps `dashu_float::DBig` with a compile-time default precision of 256 bits
   (`src/bignumber.rs:3-13,38`).
   A fixed-precision **binary** float rounds decimal fractions,
   so it cannot be the exact decimal identity for admitted JSON numbers.
- `postgres-jsonb-canonical` 0.1.0 bounds its numeric domain explicitly:
   `MAX_SCALE = 16_383` and `MAX_INTEGER_DIGITS = 131_072` (`src/number.rs:11,14`),
   with a dedicated error for numbers outside the PostgreSQL numeric domain (`src/lib.rs:112-113`).
   The JSON number grammar has no such bound.
- `number-general` 0.14.0 offers only `Bool`,
   `Complex`,
   `Float`,
   `Int` and `UInt` variants (`src/lib.rs:120-128`) with `f64` and complex-`f64` dtypes (`:92-94`).
   Fractional JSON literals land in binary floating point.
- `hypercast` 0.3.0 documents its exact decimal as a sign,
   a **96-bit** magnitude and a base-10 scale,
   states that precision is a range rather than a rounding opportunity,
   and refuses larger magnitudes (`src/decimal.rs:3-19,82-83,137-139`);
   its own test expects `OutOfRange` for a 23-digit integer (`src/lib.rs:192`).
- `dashu-ratio` 0.6.0 parses a decimal exponent by materializing `base.pow(abs_scale)` and multiplying it
   into the numerator or denominator (`src/parse.rs:163-170`).
   A literal such as `1e1000000000` would need a bigint of roughly a billion decimal digits,
   so this family fails on admitted input by resource exhaustion rather than by a clean rejection.
   That is the same class of exit already recorded for `bigdecimal`,
   `scientific` and `b10`,
   which bound the exponent instead of materializing it.
- `qubit-json` 0.10.0 reports integer and float values outside `i64`/`u64`/finite `f64` as errors
   (`src/decode/json_syntax_error_reason.rs:55-58`),
   which the parser-foundation report records separately as a strict-JSON component.

Metadata-level classifications,
 without pinned source confirmation:

- Fixed-scale or domain decimal models:
   `atomr-money`,
   `financial-ops`,
   `metering`,
   `ledgeline-core`,
   `price-parser`,
   `price-parser-rs`,
   `positive`,
   `wager-math`,
   `hardmoney`,
   `gnucobol-rs`,
   `emob-ocpp`,
   `tktax-serde`,
   `decimal_scaled_macros`.
- Other formats or unrelated domains:
   `ason`,
   `purrdf-json`,
   `purrdf-text`,
   `purrdf-geo`,
   `yaml-rt-core`,
   `tomljson`,
   `mp2json`,
   `pairl`,
   `tokn-codex-protocol`,
   `tokn-pi-protocol`,
   `spanned_json_parser`,
   `elicit_serde_json`,
   `enum2schema`,
   `serde_sated`,
   `dtype_variant`,
   `tagword`,
   `exatok`,
   `tokcost`,
   `pretty-bytes-enum`,
   `ecma-lex-cat`,
   `floravox-ssml`,
   `ferrodoc-ast`,
   `differential-engine`,
   `dig-rpc-protocol`,
   `dig-rpc-types`,
   `intl`,
   `laser-sdk`,
   `logicaffeine-base`,
   `okf-ingest`,
   `gcal-fetcher`,
   `bionamic-immunum`,
   `person-matcher`,
   `phonenumber-rs`,
   `pitlane-mcp`,
   `std-logger`,
   `ferrotherm`,
   `ferromorphic`,
   `torustcalcmcp`,
   `vastblue-uni`,
   `outram-park-fork-coolprop`,
   `fast-float`,
   `fast-float2`,
   `hexfloat2`.
- Source-verified additions to the bounded and materializing families:
   `puremp` 0.2.4 stores an arbitrary-precision coefficient with an **`i64`** exponent
   (`src/decimal.rs:1-7,41-45`),
   and `bigfixed` 0.0.0 stores a decimal digit vector with a radix position
   (`src/big_fixed.rs:26-30`),
   so a 40-digit JSON exponent is either unrepresentable or needs roughly a billion materialized digits.
- Arbitrary-precision integer families whose exponent handling was not individually read:
   `flexint`,
   `smallbigint`,
   `astra-num`,
   `near-bigint`,
   `num-rational-parse`,
   `dashu`,
   `dashu-int`.
   `dashu-ratio`'s verified exponent materialization is the family's decisive pattern,
   but each crate needs its own source check before an individual exit is asserted.
- `aequa` was not found on crates.io under that exact name by a direct registry lookup,
   so it remains a Git-only lead like the parser report's `jsonc_lexer`.
- `decimal-scaled-golden` 0.5.1 is a library-agnostic golden value set and comparison harness.
   It is a potential independent **oracle** rather than a value representation;
   the owned prototype already cross-checks against a separately computed bounded rational oracle,
   so no decision depends on adopting it.
- `json-number` 0.4.10 reappeared in this scan and keeps its recorded exclusion
   ([documented unsized-layout gap](../troubleshooting/json-number-unsized-layout.md)).

After this screen the numeric registry class has one **validated** representation
 (the repository-owned token plus identity prototype)
 and one unvalidated composition
 (`serde_json` `arbitrary_precision` behind a custom mathematical-equality adapter).
 That is a screening outcome over the frozen queries,
 not proof that no other crate could satisfy the constraints.
 The repository-host and broader-web classes are still incomplete,
 so discovery saturation remains unclaimable and no recommendation follows from this section alone.

## Repository-owned equality prototype and limits

A disposable JavaScript reference prototype at `~/temp/agent/jsonc-number-probe.mjs` validates a JSON number token,
 separates sign and significand digits,
 collapses zero,
 strips leading and trailing coefficient zeros,
 and compares the adjusted decimal exponent without materializing exponent-sized zero padding.
It retains the raw spelling separately in its call site.
 This is an algorithm probe,
 **not** a Rust library implementation or consumer validation.

- `cd -- ~/temp/agent && mise run test:number` passed,
   reporting `validated 1120 generated literals against a bounded BigInt oracle, plus edge cases`.
   The oracle cross-multiplies exact rational values only for bounded exponents;
   generated literals include signed coefficients,
   fractions and exponents.
   Targeted cases cover `1 = 1.0 = 1e0`,
   `100 = 1e2`,
   `0.00100 = 1e-3`,
   all zero spellings,
   adjacent integers beyond 2^53,
   distinct huge exponents and a compensating huge-exponent equality.
   Invalid forms include missing exponent digits,
   leading zero,
   invalid sign and Unicode digit.
- Positive control:
   `mise run test:number:control` deliberately dropped the exponent from identity
   and failed on huge positive versus negative exponents (`true !== false`,
   exit 1).
   The unmodified task passed again afterward.
   Thus an unchanged test result from the prototype is not an unproven null.
- Remaining failure risks:
   the prototype uses JavaScript `BigInt` to adjust exponents,
   while the Rust implementation needs a bounded-by-input,
   sign-aware decimal-exponent representation without machine-integer overflow.
   Exact equality and hashing must share normalized identity;
   `-0` equals `0` while retaining original spelling.
   The scratch probe cannot establish Rust ergonomics,
   linear time,
   parser integration,
   or published crate behavior.

Further source screening:
 `fpdec` 0.14.1,
 `~/temp/agent/fpdec-2026-09-24` at `b9ac6e5`,
 `src/lib.rs:98-131` stores `coeff: i128` and restricts fractional digits to `MAX_N_FRAC_DIGITS`;
 it cannot be the only representation for arbitrary JSON numbers.
 Its `LICENSE.TXT:1-3` identifies BSD 3-Clause despite registry `cargo info` reporting `license: unknown`
 (manifest uses `license-file`,
 not a license identifier).
 `fraction` 0.17.0 at `~/temp/agent/fraction-2026-09-24` uses `BigInt`-backed rationals
 but does not by itself preserve a raw JSON number token
 or provide unbounded exponents without magnitude-sized denominator/numerator expansion;
 a custom exponent layer would still be needed.
 The latter is an architectural inference,
 not a verified execution result.

## Regex screening, 2026-09-24

The user requested early regex culling.
 Our operational screening interprets this as production parsing or lexing and required normal/build dependencies,
 not upstream dev-only usage.
 The inspected direct source and manifest for `json-number` 0.4.10 at `~/temp/agent/json-number-2026-09-24` contains no
 `regex::`,
 `Regex::`,
 `#[regex]`,
 `logos`,
 or direct regex dependency;
 `src/lib.rs:117-195` validates number grammar with a state machine.
 Its exact-version normal/build Cargo dependency graph contains no regex-named package,
 while full transitive source screening remains pending.
 The repository-owned token/normalization prototype uses direct scans;
 it remains an incomplete JavaScript proof,
 not Rust validation.

### Resolved production dependency graph

`~/temp/agent/jsonc-regex-audit/json-number/tree.txt` records `mise run audit:regex-deps`,
 using Cargo `tree --edges normal,build --target all --format {p}` for `json-number = "=0.4.10"`.
 Neither `lexical` nor its selected `lexical-core` family brought a package named `regex`,
 `logos`,
 `pest`,
 `regex-automata`,
 `regex-syntax`,
 `onig`,
 or `fancy-regex`.
 The same script's `edikt-jsonc` positive control found all of `logos`,
 `regex`,
 `regex-automata` and `regex-syntax`;
 an empty result is meaningful for package-name screening.
 Source-level checks of transitive production code remain open,
 and the graph does not establish numeric correctness.

## Discovery expansion progress

The frozen expansion in `doc/planning/jsonc-edit-rust-port.md` ran `cargo search 'json number' --limit 100`,
 `cargo search 'decimal arbitrary exponent' --limit 100`,
 `gh search repos 'rust json number' --limit 100 --json fullName,url,description,updatedAt`,
 and a web query for `Rust lexical JSON number numeric equality arbitrary exponent crate`.
 The repository-host query returned no matches;
 its narrower wording cannot prove comparable projects absent.
 The web query returned source/docs leads for `json-number`,
 `json-syntax`,
 `reliakit-json`,
 and lexical parsing tools.

The crates.io API was paged through `web_fetch` for pages 2 and 3 of `json number` (100 records each),
 and page 2 of `decimal arbitrary exponent` (empty,
 with `next_page: null`).
 A name/description filter over the `json number` pages surfaced `rust_decimal` and `dashu` on page 2,
 neither newly discovered;
 page 3 surfaced none under that filter.
 `rust_decimal` is described by its [docs](https://docs.rs/rust_decimal/latest/rust_decimal/) as fixed precision;
 `dashu`'s [crate metadata](https://crates.io/crates/dashu) is an arithmetic family,
 not a lossless JSON number token and exact decimal-exponent identity on its own.
 These are category readings,
 not execution results.
 The full page records must be inspected before two-page saturation is claimed
 because filtering could hide a less obvious candidate.

Page 2 and page 3 of the initial `exact decimal`,
 `bigint`,
 and `arbitrary precision json` registry searches were also retrieved earlier.
 Their obvious leads include `fpdec`,
 `fraction`,
 `bigdecimal`,
 `ordecimal`,
 `decimal-bytes`,
 `arbi` and `dashu`;
 source screening recorded limitations for several.
 The three initial Cargo searches each advertised more results after their first 100,
 so their initial response alone was not saturation.
 No candidate is recommended from metadata pages alone.

### Delegated comparable-number leads

A research-only agent used paginable crates.io alphabetical searches for `unbounded decimal`
 (100 and 55 records,
 total 155) and `decimal exponent` (100 and 78,
 total 178),
 both ending at `next_page=null`.
 Its GitHub repository queries with `language:Rust`,
 default best-match,
 `per_page=100` and no negative filter returned 15 for `decimal arbitrary precision`,
 zero for `json number exact`,
 and zero for `decimal exponent`;
 all reported `incomplete_results:false`.
 Two broader-web queries each returned ten sources without a total or cursor,
 so web completeness is not established by those responses.
 These were additional evidence probes,
 not a second recursively expanding taxonomy schedule.

New source leads include [`aequa`](https://github.com/Xqhare/aequa),
 [`bignumber` 0.1.1](https://docs.rs/bignumber/0.1.1/bignumber/),
 [`puremp` 0.2.4](https://docs.rs/puremp/0.2.4/puremp/decimal/struct.Decimal.html),
 [`bigfixed` 0.0.0](https://docs.rs/bigfixed/0.0.0/bigfixed/),
 [`qubit-json` 0.10.0](https://docs.rs/qubit-json/0.10.0/qubit_json/),
 and [`postgres-jsonb-canonical` 0.1.0](https://docs.rs/postgres-jsonb-canonical/0.1.0/postgres_jsonb_canonical/).
 The delegated source reading suggests limited exponents,
 finite-precision rounding,
 binary rather than decimal scaling,
 or floating-point projection in several leads;
 those candidate-specific exits await pinned source confirmation.
 Full unfiltered registry pages now exist under `~/temp/agent/jsonc-registry-full-pages/`,
 and independent screening of that data remains in progress.
 No discovery saturation or recommendation follows from the lead list.

## Rust owned-number prototype, not an adopted foundation

A disposable,
 dependency-free Rust library at `~/temp/agent/jsonc-exact-number-probe/` validates JSON number grammar
 and computes numeric identity from sign,
 normalized nonzero coefficient digits,
 and a signed decimal exponent stored as text.
 It uses direct byte scans and decimal column addition/subtraction,
 with no regex or recursion.
 The original token remains the caller's responsibility;
 this prototype has no JSONC parser,
 comment model,
 or serializer.
 It is evaluation code outside the main worktree.

- `cd -- ~/temp/agent/jsonc-exact-number-probe && mise run test` passed the Rust unit suite,
   including mathematical equality across `1`,
   `1.0`,
   `1e0`,
   signed zero,
   compensated exponents longer than machine integers,
   adjacent large integers,
   invalid grammar,
   and a generated bounded-domain comparison with an independently computed `i128` rational oracle.
- Positive control:
   replacing only the adjusted-exponent calculation with `String::from("0")` caused `unequal_values`
   and `generated_against_rational_oracle` to fail.
   The original expression was restored,
   a SHA-256 comparison to the saved pre-mutation source matched,
   and `mise run test` passed again.
   Later additional equivalent/unequal cases passed.
- `mise run lint:clippy` passed with warnings denied
   after changing the scratch source's digit check to `is_ascii_digit` and its nonzero-start branch to a named local.
   The scratch manifest mirrors the repository Rust convention allowing `needless_return`
   while denying implicit returns.
   This is not a claim that the final crate passes its own linter.
- A separate disposable consumer at `~/temp/agent/jsonc-exact-number-consumer/` imported the library by a local path
   and ran `mise run test`,
   printing `consumer exact-number equality passed`.
   Its first run failed because the hand-written fixture compared exponents with different digit counts;
   that test input was measured and corrected before the passing run.
   No product code was touched.

The core operates on ASCII tokens of finite in-memory length,
 scans each character through grammar and coefficient normalization,
 and adds or subtracts exponent digit strings without expanding `10^E`.
 This suggests work bounded by input length,
 but the full crate's complexity and error contracts have not been measured.
 Candidate validation remains incomplete until its public value type retains raw spellings,
 edited inputs preserve exactness,
 and the port runs through the JSONC comment and serializer surfaces.

## Prototype review correction and expanded verification

An independent review found that the first Rust prototype exposed mutable-invariant `Identity` fields
and compared `Result` values directly,
 so some purportedly valid fixtures could have passed through matching parse errors.
 The scratch `Identity` fields are now private.
 Equality and inequality tests unwrap each valid input before comparing,
 and every generated source is validated before sampled pair comparisons.
 A `HashSet` test checks equal spellings hash and compare consistently,
 including signed zero.
 The bounded rational oracle remains independently computed rather than using the normalizer.

Additional targeted cases exercise an uppercase exponent marker and leading zeros,
 malformed number tokens,
 multi-digit fraction and trailing-zero adjustments,
 and signed carry/borrow over 256 exponent digits.
 `mise run lint:clippy` passed with warnings denied;
 `mise run test` passed the expanded Rust unit suite.
 The disposable consuming crate's `mise run test` also passed again after those changes.
 The earlier exponent-omission mutation demonstrated test sensitivity but was run before these additions;
 do not claim it killed every arithmetic fault.
 Public raw-token storage,
 serialization,
 full parser integration,
 runtime scaling measurements,
 and release consumer tests remain open.

## Terminal discovery result

- **Registry class:
   finished.**
   The five frozen numeric queries (`exact decimal`,
   `bigint`,
   `arbitrary precision json`,
   `json number`,
   `decimal arbitrary exponent`) were paginated to `next_page=null`,
   validated for contiguous pages,
   matching totals and duplicate-free crate ids,
   and all **13303** saved records were passed through the inclusive lead predicate recorded above.
   The two delegated comparable-term queries (`unbounded decimal`,
   `decimal exponent`) also ended at `next_page=null`.
   Limitation recorded rather than hidden:
   keyword lead extraction can pass over a crate whose name and description avoid both term families.
- **Repository-host class:
   finished.**
   The frozen queries `rust arbitrary precision decimal json`,
   `rust bigint` and the expansion `rust json number` returned no matches.
   The delegated comparable queries returned 15 for `decimal arbitrary precision` and zero for
   `json number exact` and `decimal exponent`,
   each reporting `incomplete_results:false`.
   Every lead they produced (`aequa`,
   `bignumber`,
   `puremp`,
   `bigfixed`,
   `qubit-json`,
   `postgres-jsonb-canonical`) is classified in the screen above.
- **Broader-web class:
   frozen schedule finished,
   completeness not provable.**
   The recorded queries returned source and documentation leads without totals or cursors.
- **In-repo class:
   finished.**
   The maintained TypeScript number behavior,
   the prior structured-edits research and the existing Cargo dependency set were read;
   the repository-owned prototype is the incumbent baseline.

Terminal outcome:
 **saturated with one survivor** for the exact-number value representation,
 namely the repository-owned raw-token plus mathematical-identity prototype.
 Every published alternative exited on a source-verified hard gate rather than on a score:

- Lexical equality as-is:
   `json-number` 0.4.10 (also excluded by the unsized-layout gate),
   `serde_json` 1.0.151 `arbitrary_precision`,
   `jstrict` 0.15.0,
   `reliakit-json` 1.0.0.
- Bounded exponent or scale:
   `bigdecimal` 0.4.10,
   `scientific` 0.6.0,
   `b10` 1.0.0,
   `ordecimal` 0.3.1,
   `decimal-bytes` 0.6.0,
   `fpdec` 0.14.1,
   `rust_decimal` 1.43.0 (a 96-bit coefficient with a scale field limited to 0 through 28,
   `src/decimal.rs:126-134` in the published archive),
   `hypercast` 0.3.0,
   `postgres-jsonb-canonical` 0.1.0,
   `puremp` 0.2.4.
- Magnitude materialization on admitted input:
   `dashu-ratio` 0.6.0,
   `bigfixed` 0.0.0,
   and `fraction` 0.17.0 by the same mechanism.
- Binary floating-point projection:
   `bignumber` 0.1.1,
   `number-general` 0.14.0.
- Integer or float range rejection:
   `qubit-json` 0.10.0.

Two further leads exited on registry metadata alone rather than source:
 `fast-float` 0.2.0 and `fast-float2` 0.2.4 describe themselves as floating-point number parsers,
 which is inexact by category.

Still unverified individually,
 and therefore not claimed as exits:
 `aequa` (Git-only,
 no crates.io record under that exact name),
 `arbi`,
 `flexint`,
 `smallbigint`,
 `astra-num`,
 `near-bigint`,
 `num-rational-parse`,
 `dashu` and `dashu-int`.
 Their family's decisive pattern is source-verified through `dashu-ratio`,
 but each needs its own check before an individual exit is asserted.
 None of them is a JSON-number token holder with unbounded decimal-exponent identity,
 which is what the survivor provides.

## Scoring, sensitivity and recommendation

The frozen rubric scores each relevant soft concern at weight 1 with a maximum of 4.
 Hard constraints are not scored.
 Only validated candidates are scored,
 and saturation produced one.

### Repository-owned raw token plus mathematical identity

- API clarity for exact values:
   **4**.
   `identity(raw)` returns a canonical identity with private fields,
   equality and hashing derive from that identity,
   and the caller keeps the original spelling separately;
   a separate consumer crate exercised it across a crate boundary.
- Source auditability:
   **4**.
   Repository-owned,
   and a targeted search of its source found no `unsafe` block.
- Release and maintenance evidence:
   **2**.
   No published crate,
   no CI history and no external users;
   maintenance is internal and unproven at product scale.
- Dependency and build surface:
   **4**.
   Standard library only,
   with no normal or build dependencies to audit or re-audit.
- Measured performance on the library's input corpus:
   **3**.
   Its cost is inside the measured parser shapes
   (the 40-digit-exponent shape ran 12739 to 13427 nanoseconds per parse-to-value call for the owned path),
   but no isolated microbenchmark of `identity` alone was taken.
- Total:
   **17 of 20**.
- Pros:
   exact equality across arbitrary exponent digit runs without materializing magnitude;
   original spelling retained for unedited literals;
   `-0` and `0` unified;
   no dependencies and no unsafe;
   directly fixable alongside the parser.
- Cons:
   this port owns the grammar validator,
   the normalizer and their tests forever;
   no external review,
   fuzzing or release engineering;
   performance evidence is shape-level rather than isolated.

### Strongest rejected alternative: `serde_json` `arbitrary_precision` plus a custom adapter

Not scored,
 because it is not an independent foundation.
 Its raw-token retention comes from `serde_json`'s `arbitrary_precision` `String` representation
 (`serde_json-1.0.151/src/number.rs:20-25,72-73`),
 whose derived equality is lexical,
 so mathematical identity still has to be supplied by the owned normalizer.
 The composition therefore adds a dependency and a foreign parse boundary
 without removing any owned code,
 and it inherits `serde_json`'s own string and number admission rules.
 Pros:
   a widely used parser and token store.
 Cons:
   lexical equality as-is,
   an extra dependency graph,
   and no reduction in the code this port must write and maintain.

### Sensitivity

With a single survivor,
 no weighting can change the order:
 there is no second scored candidate to overtake.
 The sensitivity question that does matter is whether re-rating could resurrect an excluded candidate,
 and it cannot,
 because every exclusion above is a hard-gate failure and the frozen rubric does not score hard constraints.

The two uncertain ratings were rerun at their endpoints.
 Maintenance evidence at 1 gives 16 of 20;
 at 3 it gives 18.
 Measured performance at 2 gives 16;
 at 4 it gives 18.
 The combined worst case is 15 of 20 and the best case 19 of 20.
 Rerunning each criterion's weight from 1 through 5 with the worst-case ratings leaves the survivor unchanged
 (the weighted worst case with performance at weight 5 is 24 of 36).
 No endpoint or weight overturns the outcome.

### Recommendation

Recommend the **repository-owned raw-token plus mathematical-identity representation** as the exact-number
 foundation for the native Rust crate,
 paired with the repository-owned parser recommendation in the companion parser report.

This is a recommendation for a **separate adoption decision**.
 It authorizes no dependency change,
 no product code,
 no decision record and no publication.
 If adoption is refused,
 the recorded alternative is not a different crate but the same owned normalizer behind
 `serde_json` `arbitrary_precision`,
 with the extra dependency and boundary noted above.

## Remaining validation before publication

- Product-crate boundary:
   the validated code is a scratch prototype,
   not `package/rust-module/jsonc-edit`;
   packaging,
   the public API shape,
   documentation and repository lint and type gates are unbuilt.
- Isolated performance measurement of the normalizer on an exponent-heavy corpus,
   if the product needs a number-level budget rather than a shape-level one.
- Fuzzing of the grammar validator and normalizer at product level;
   the current evidence is a bounded generated rational oracle plus curated edge cases.
- Individual source checks for the nine unverified integer-family leads if a later reviewer wants
   every registry lead closed rather than family-classified.
- Publication route:
   an authorized crates.io token path for the first release remains unverified,
   and name availability must be rechecked immediately before publishing.
