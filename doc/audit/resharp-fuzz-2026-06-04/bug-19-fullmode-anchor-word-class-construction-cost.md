# BUG-19 an anchor adjacent to a full-mode word class costs seconds to match diverse input

## Classification

- Type:
   performance,
   fixed-cost match-time DFA construction blowup,
   no oracle
  needed.
- Phase:
   match time,
   the first scan over a diverse-byte input (the lazy DFA builds
  the expensive states then).
- Severity.
   A three-character pattern,
   `$?\w`,
   takes about 3 seconds to run
  `is_match` over 16 KB of varied bytes under `unicode(Full)`,
   with the size limits
  enabled.
   The 8700F should answer this in well under a second.
   The cost is the
  same at 2 KB and at 32 KB,
   so it is a one-time construction cost,
   not yet an
  input-scaling denial of service,
   but it already breaks the sub-second
  expectation on small inputs.

## Minimal reproducer

```rust
//! Reproduction of the construction cost of an anchor next to a word class in the full Unicode mode.

/// The regex type,
/// its options and the Unicode mode that selects the full classes.
use resharp::{Regex, RegexOptions, UnicodeMode};
let opts = RegexOptions::default().unicode(UnicodeMode::Full);
let re = Regex::with_options("$?\\w", opts).unwrap();
let hay: Vec<u8> = (0..16384u32).map(|i| (i % 256) as u8).collect();
let _ = re.is_match(&hay);   // ~3 s
```

Command line (`--benchcyc <hexpat> <N> <op> <cfgidx>` builds a hay of bytes
`0..256` cycling,
 length N,
 and times one op;
 full mode is cfgidx 2):

```sh
repro --benchcyc 243f5c77 16384 is_match 2   # $?\w  -> ~3.0
repro --benchcyc 245c77   16384 is_match 2   # $\w   -> ~1.0
```

## Observed behaviour and isolation

`is_match` over `cyc(16384)` (bytes 0..256 cycling),
 by config (0 default,
 1
ascii,
 2 full,
 3 js):

```text
$?\w   cfg0 0.0026   cfg1 0.0000   cfg2 3.05   cfg3 0.0000
$\w    cfg0 0.0009   cfg1 0.0000   cfg2 1.02   cfg3 0.0000
\w$    cfg0 0.0022   cfg1 0.0000   cfg2 0.11   cfg3 0.0000
\w     cfg0 0.0001   cfg1 0.0000   cfg2 0.0002 cfg3 0.0000
$?\W   cfg0 0.0044   cfg1 err      cfg2 1.98   cfg3 0.0001
$?\d   cfg0 0.0000   cfg1 0.0000   cfg2 0.0095 cfg3 0.0000
$?\s   cfg0 0.0000   cfg1 0.0000   cfg2 0.0002 cfg3 0.0000
```

Four conditions must all hold:

- Full unicode mode.
   Default,
   ascii,
   and javascript modes are all fast;
   only
  `unicode(Full)` pays the cost,
   because only there is `\w` the full multi-byte
  Unicode word class.
- A large perl class.
   `\w` and `\W` trigger it;
   `\d` and `\s` do not (they are
  small even in full mode).
   `\w` alone (no anchor) is also fast,
   so it is the
  combination,
   not the class by itself.
- An anchor adjacent to the class.
   `$\w` is 1 s,
   `$?\w` is 3 s (the optional `?`
  builds an `EPS | $` alternation,
   doubling the construction),
   `\w$` (anchor after
  the class) is only 0.1 s.
   The anchor in front of the class is the expensive
  placement.
- Diverse input.
   The same `$?\w` on 16 KB of a single byte (`'a'`) is 0.0000 s;
  the cost appears only when the input exercises many distinct byte classes,
   which
  is what forces the lazy DFA to build a state per class.

The cost is fixed in the input length:

```text
$?\w full is_match, cyc(N):  N=2048 3.22   4096 2.87   8192 3.03   16384 2.85   32768 3.23
```

## Root cause

`$` desugars to a positive lookahead (newline-or-end).
 Concatenated in front of a
class,
 the lazy DFA must,
 at each distinct input byte,
 take a derivative of the
lookahead-then-class term.
 In full mode `\w` is a large minterm set (the full
Unicode word class spanning multi-byte sequences),
 so the diverse input drives the
construction of a state for many of those minterms,
 each carrying the lookahead
context,
 and the one-time build is seconds of work.
 The anchor is what keeps the
class from collapsing to a single transition:
 `\w` alone in full mode shares one
class transition and stays fast,
 but the lookahead splits it.
 `\d`/`\s` stay fast
because their full-mode minterm sets are small,
 and ascii/js modes stay fast
because `\w` there is the small ascii class.

This is distinct from BUG-17 (which is the bracketed `[\w]` lowering,
 is
mode-independent,
 and is a compile-time cost) and from BUG-16 (which is a lookbehind
of a lookahead and is input-scaling O(n^3)).
 BUG-19 is mode-specific (full only),
diverse-input-triggered,
 and a fixed match-time construction cost.

## Affected configurations

`unicode(Full)` only,
 among the limits-enabled configs;
 `default`,
 `ascii`,
 and
`javascript` are all sub-millisecond,
 and `flags` and `hardened` were not the
trigger here.
 The `unbounded_size` config is out of scope (it disables the size
limits).
 Because full mode is the configuration a user picks precisely to get full
Unicode semantics,
 the cost lands on the realistic case,
 not an exotic one.

## Recommendation for ieviev

Share the lazy-DFA state across the minterms of a large class when an adjacent
anchor's lookahead has already been resolved,
 or resolve the anchor lookahead
before expanding the class transitions,
 so `$\w` in full mode builds the same small
automaton `\w` alone does.
 Until then,
 any anchor immediately in front of `\w` or
`\W` under full mode is a multi-second stall on the first match over real
(diverse-byte) text.
