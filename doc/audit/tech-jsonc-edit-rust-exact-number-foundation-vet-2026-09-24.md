# Vet exact JSON number storage for the Rust jsonc-edit port

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

## Status and fingerprint

- Status:
   Superseded by [regex-gated report](tech-jsonc-edit-rust-exact-number-foundation-vet-2026-09-24-e8e0034a.md);
   original fingerprint is retained for audit history.
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
   `1fd8b19bb0a656cca8a01b1360a4e7225c8cc529f0888dcd46e530655c6570a4`.
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
   inspectable source and reproducible validation.
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
 The maximum for each is 4;
 no candidate has been scored.
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
   pending source proof of numeric equality semantics and JSON grammar.
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
  As a grammar validator and token holder combined with our exact comparator it remains eligible
   for targeted validation.
  Its `src/lib.rs:178-210` uses a checked scan followed by an unsafe cast;
   audit this boundary if used.
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
   `mise run test:number:control` deliberately dropped the exponent from identity and failed
    on huge positive versus negative exponents (`true !== false`,
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
 Its `LICENSE.TXT:1-3` identifies BSD 3-Clause despite registry `cargo info` reporting `license: unknown` (manifest uses
  `license-file`,
 not a license identifier).
 `fraction` 0.17.0 at `~/temp/agent/fraction-2026-09-24` uses `BigInt`-backed rationals but does not
  by itself preserve a raw JSON number token
  or provide unbounded exponents without magnitude-sized denominator/numerator expansion;
 a custom exponent layer would still be needed.
 The latter is an architectural inference,
 not a verified execution result.

## Pending evidence and validation

- Finish required discovery saturation or report a blocked terminal outcome;
   record exact expansion queries and pagination attempts.
- Clone every serious candidate under private `~/temp/agent/` using `gh repo clone`,
   disable clone push targets before any prototype commit,
   and record revisions.
- Inspect licenses,
   manifests,
   source parsing and equality paths,
   test/CI and maintenance;
   complete applicable hard gates and overlays.
   Do not run third-party build/test commands before inspecting execution trees and sandboxing.
- Validate every hard-gate survivor at equal depth,
   including original spelling,
   `1 = 1.0 = 1e0`,
   `-0`,
   integers past 2^53,
   large exponents,
   invalid grammar,
   and consumer use in a throwaway Rust crate.
- Freeze evidence-backed ratings,
   run scoring and sensitivity,
   state complete ranking with pros,
   cons and adjacent reasons,
   then present recommendation to the user for adoption.
   No dependency change or decision record before adoption.
