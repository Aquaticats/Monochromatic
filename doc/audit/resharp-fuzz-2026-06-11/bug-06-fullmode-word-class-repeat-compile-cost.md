# BUG-6 full-unicode word-class bounded repeat has a large per-repeat compile cost

- Type:
   performance,
   compile-time denial of service.
- Phase:
   compile time (`Regex::with_options`),
   before any input is seen.
- Severity:
   a single small pattern exceeds the engine's own "limits enabled =>
  nothing should take >= 1s" bar,
   and scales linearly to a >= 10s hang within the
  `{0,500}` repeat cap.
   `\w{8}` already costs ~1.1s in full mode.
- Affected:
   `unicode(Full)` only.
   Default and ascii are instant.
- Discovery:
   the `compile` libFuzzer target (timeouts dominated by full-mode
  `\w` / `\b\w` patterns) and directed scaling measurement.

## Reproducer

```rust
//! Reproduction of the construction cost of a bounded word class repeat in the full Unicode mode.

/// The regex type,
/// its options and the Unicode mode that selects the full classes.
use resharp::{Regex, RegexOptions, UnicodeMode};
let opts = RegexOptions::default().unicode(UnicodeMode::Full);
let _ = Regex::with_options(r"\w{24}", opts); // ~3.3s
```

Scaling (full mode,
 config 2,
 compile only):

```text
\w{8}  full: 1.14s      \w{16} full: 2.26s      \w{24} full: 3.32s
\w{12} full: 1.79s      \w{20} full: 2.82s
\w{16} default: 0.02s   \w{16} ascii: 0.0003s
```

The curve is linear in the repeat count with a slope of ~0.14s per `\w` and a
~0.5s intercept.
 It stays linear (the 06-04 BUG-23 super-linear `\w{16}` = 15.6s
is gone),
 but the constant is large enough that every `\w{n}` with `n >= 8` in
full mode crosses 1s,
 and the `{0,500}` repeat cap admits `\w{500}` at roughly
70s.

```sh
repro --compile "$(printf '\\w{24}' | xxd -p | tr -d '\n')" 2   # 3.32|ok=true
```

## Observed versus expected

Expected:
 a bounded repeat of a single class compiles in milliseconds,
 as it does
for default and ascii.
 In full mode the large multi-byte `\w` minterm set is
re-expanded per repeat instead of compiled once and counted,
 so the cost is
linear in `n` rather than near-constant.
 `\b\w` patterns share the cost (the
`compile` target's timeout units are `\b\b\wU`-shaped under the Full config).

## Source pointer

The full-unicode `\w` lowering and the `mk_repeat` bounded-repeat unroll (the
06-04 BUG-17 / BUG-23 path:
 `resharp-parser/src/lib.rs` perl-class lowering into
`resharp-algebra/src/lib.rs` `mk_repeat`).
 The fix direction is the same as the
06-04 recommendation:
 use the native counted representation for the repeat rather
than unrolling a large-minterm class `n` times.

## Relationship

The performance face of the 06-04 BUG-23 (full-unicode word class bounded repeat).
The super-linear blowup is fixed;
 the linear-with-large-constant cost remains
above the limits-enabled threshold and is the dominant `compile`-target timeout
family in this campaign.
