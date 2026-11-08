# BUG-9 a dot-and-literal concatenation blows up compile in full / javascript mode

- Type:
   performance,
   compile-time denial of service.
- Phase:
   compile time (`Regex::with_options`),
   before any input is seen.
- Severity:
   a ~23-character pattern with no unbounded quantifier hangs compile
  for 40s+ (measured 71s for the full pattern),
   far past the engine's 10s
  "definite bug" bar,
   under `unicode(Full)` and `unicode(Javascript)`.
   Default,
  ascii,
   and hardened compile it in under 1ms.
- Affected:
   full and javascript unicode modes only.
- Discovery:
   the `compile` libFuzzer target (multiple timeout units of this
  shape).

## Reproducer

```rust
//! Reproduction of the compile-time hang of a dot followed by literals in the full Unicode mode.

/// The regex type,
/// its options and the Unicode mode that selects the full classes.
use resharp::{Regex, RegexOptions, UnicodeMode};
let opts = RegexOptions::default().unicode(UnicodeMode::Full);
let _ = Regex::with_options(".n.................  n.", opts); // hangs 40s+
```

The fuzzer's full unit is `.n.................  n...  n` (`.` `n`,
 then 17 `.`,
then two spaces,
 `n`,
 three `.`,
 two spaces,
 `n`);
 the 23-byte prefix
`.n.................  n.` already hangs.
 Hex of the prefix:
`2e6e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e20206e2e`.

```sh
repro --compile 2e6e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e20206e2e 2   # full: TIMEOUT > 40s
repro --compile 2e6e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e20206e2e 0   # default: 0.0003s
```

## Observed behaviour and isolation

```text
.n.................  n.  (and ...n...  n)   compile time by config:
  default  (cfg0): 0.0005s        full       (cfg2): > 40s  (71s for full pattern)
  ascii    (cfg1): 0.0005s        javascript (cfg3): > 40s
  hardened (cfg4): 0.0003s
```

It is the large `.` minterm of full / javascript mode (`.` = any codepoint
except line terminators,
 a wide multi-byte set) interacting with the specific
concatenation,
 not the dot count:
 `.` repeated 40 times is instant in full mode,
and structurally clean variants (`.................n.................n`,
`x.................x.................x`,
 `aa.....aa.....aa`) are all instant.
 The
blowup is fragile to the exact shape:
 bisecting the trigger by prefix length is
non-monotonic (length 22 = 16.8s,
 length 23 = TIMEOUT,
 length 24+ = 0.001s),
which is the signature of a derivative / minterm state-product crossing a size
threshold for one particular alignment of `.` and the literal `n`,
 rather than a
smooth per-character cost.

## Root cause (mechanism)

The derivative-based automaton construction expands the concatenation of the wide
full/javascript `.` minterm with the interspersed literal `n`,
 exploring the
product of "this position is `n` versus not" decisions across the `.` run.
 The
default and ascii `.` minterms are small enough that the product stays tiny;
 the
full/javascript `.` minterm is large enough that one specific length makes the
intermediate state or minterm set explode.
 The construction does not recognise
the fixed-width,
 quantifier-free structure that would let it avoid the product.

## Relationship

Distinct from bug-06 (full-mode `\w` bounded-repeat linear cost).
 bug-06 is a
smooth per-repeat cost of the `\w` class;
 bug-09 is a threshold blowup of the `.`
minterm in a quantifier-free concatenation.
 Both are full-mode compile DoS,
 but
different classes and different curves.
 Conceptually in the family of the 06-04
BUG-17 / BUG-23 minterm-expansion compile blowups,
 which the v0.6.12 work
addressed for the bracketed-perl-class case but not for the full `.` minterm.
