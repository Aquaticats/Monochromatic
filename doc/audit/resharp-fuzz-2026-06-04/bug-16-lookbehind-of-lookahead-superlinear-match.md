# BUG-16 lookbehind of a positive lookahead is super-linear at match time

## Classification

- Type:
   performance,
   algorithmic-complexity blowup (ReDoS class),
   no oracle
  needed.
   A pattern that compiles must not take seconds to match a kilobyte.
- Phase:
   match time (the lazy DFA never reaches a fixpoint,
   so a fresh state is
  built at every input offset).
- Severity:
   denial of service.
   A six-character pattern matches a 512-byte input
  in 13 seconds and a 1 KB input in over two minutes,
   with the size limits
  enabled.
   This violates ieviev's own invariant that nothing should take 10
  seconds or more when the limits are not disabled;
   the threshold is crossed at
  roughly 512 bytes of input.

## Minimal reproducer

```rust
//! Reproduction of the superlinear match:
//! a lookbehind that wraps the end anchor takes seconds on a short input and minutes on a longer one.

/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};
// default config (multiline on, size limits on)
let re = Regex::with_options("(?<=$)", RegexOptions::default()).unwrap();
let hay = vec![b'a'; 512];
let _ = re.find_all(&hay);   // ~13 s; 1024 bytes is > 2 min
```

A lookbehind whose inner expression is a positive lookahead that fails at the
position under test is enough.
 The three corpus patterns that first surfaced it,
all hanging past the 25-second watchdog on a 1 KB input,
 were `(?<=$)`,
`((?<=$))`,
 `(?:(?<=$))`,
 and `(?<=(?= ))`.
 `$` is the trigger because resharp
desugars the end-anchor to a positive lookahead (newline-or-end) under multiline.

Command lines (default config,
 `--bench1 <hexpat> <hexhay> <op> <cfgidx>` times a
single operation and prints seconds):

```sh
# (?<=$) find_all on 'a' * N, default config (cfgidx 0)
repro --bench1 283f3c3d2429 "$(yes a | head -n 512 | tr -d '\n' | xxd -p | tr -d '\n')" find_all 0
# -> 13.3155

# (?<=(?=z)) find_all on 'a' * 256: lookbehind of a positive lookahead that fails
repro --bench1 283f3c3d283f3d7a2929 "$(yes a | head -n 256 | tr -d '\n' | xxd -p | tr -d '\n')" find_all 0
# -> 1.3314
```

## Observed behaviour and scaling

`find_all` of `(?<=$)` on `'a' * N` under the default config:

```text
N=  8   0.0001 s
N= 16   0.0004 s
N= 32   0.0020 s
N= 64   0.0123 s
N=128   0.1348 s
N=256   1.3966 s
N=384   4.9331 s
N=512  13.3155 s
```

The factor per doubling climbs (4,
 5,
 6,
 11,
 10),
 so growth is super-cubic,
 near
O(n^3.2);
 1024 bytes extrapolates to roughly 130 seconds,
 which is why the panic
hunt's 25-second watchdog fired on the 1 KB haystack.
 The work is purely in
matching:
 the pattern compiles instantly.

Which entry point blows up depends on whether an early match lets the scan
short-circuit:

- `(?<=$)`:
   `find_all` blows up (it evaluates the costly lookbehind at every one
  of the N positions),
   but `is_match` and `find_anchored` are instant,
   because a
  match exists at the end of input and the forward scan reaches it at once.
- `(?<=(?= ))` and `(?<=(?=z))`:
   the inner lookahead never holds in an all-`a`
  haystack,
   so the pattern never matches;
   `is_match` then has no early exit and
  must scan every position,
   so `is_match` blows up too (`(?<=(?= ))` `is_match` on
  512 bytes is 12.8 s).

## Trigger isolation

Held at `find_all`,
 N=256,
 default config:

```text
(?<=a)        0.0000   plain literal lookbehind: linear
(?<=a*)       0.0000   nullable non-assertion body: linear
(?<=^)        0.0000   start-anchor lookbehind (^ desugars to a lookbehind): linear
(?=$)         0.0000   forward lookahead of the end-anchor: linear
(?<!a)        0.0000   negative lookbehind: linear
(?<=(?=a))    0.0000   inner positive lookahead that SUCCEEDS: linear
(?<=(?!a))    0.0000   inner negative lookahead: linear
(?<=(?!z))    0.0000   inner negative lookahead: linear
(?<=(?=a)a)   0.0000   inner positive lookahead that succeeds, with a tail: linear
(?<=$)        1.3966   inner positive lookahead (newline-or-end) that FAILS mid-string
(?<=(?= ))    1.2872   inner positive lookahead (space) that FAILS
(?<=(?=z))    1.3314   inner positive lookahead (z) that FAILS
```

The trigger is exactly one shape:
 a lookbehind whose inner expression is a
**positive** lookahead,
 evaluated at a position where the lookahead body does not
match.
 A positive lookahead that always holds (`(?=a)` in an all-`a` haystack)
does not blow up,
 nor does a negative lookahead in either direction,
 nor a
nullable non-assertion body,
 nor the start-anchor (which desugars to a lookbehind,
not a lookahead).

## Root cause localization

The `Kind::Lookbehind` derivative arm
(`resharp-algebra/src/lib.rs:1378`) re-derives both halves of the lookbehind on
every input symbol and rebuilds a fresh lookbehind term:

```rust
//! The derivative step for a lookbehind node:
//! it derives the inner pattern again at every position,
//! which makes the whole match superlinear.
// resharp-algebra/src/lib.rs:1378
Kind::Lookbehind => {
    let lb_prev_der = { /* der of the reverse-tracking "prev" */ };
    let lb_inner = self.get_lookbehind_inner(node_id);
    let lb_inner_der = self.der(lb_inner, mask)?;          // re-derive the inner each step
    this.mk_binary_result(
        lb_inner_der, lb_prev_der,
        &mut (|b, left, right| b.mk_lookbehind_internal(left, right)),
    )?
}
```

When `lb_inner` is a positive lookahead with a non-nullable body,
 the
`Kind::Lookahead` arm (`resharp-algebra/src/lib.rs:1398`,
 non-nullable branch at
`:1410`) keeps deriving the lookahead body forward instead of collapsing to
`BOT`/`EPS`.
 The lookahead body never resolves (it fails on the current symbol but
the lookahead does not commit),
 so each step yields a structurally distinct
lookbehind term.
 The derivative state set therefore grows with the input position
rather than reaching a fixpoint:
 the lazy DFA allocates a new state at every
offset,
 each new state costing a derivative whose size grows with the offset.
 One
scan is O(n^2);
 `find_all`,
 which restarts a scan at each match position,
 is
O(n^3).
 A positive lookahead whose body succeeds collapses immediately (the
`is_nullable` shortcut at `:1403` keeps only the body),
 which is why `(?<=(?=a))`
stays linear.

The fix is to give the lookbehind-of-lookahead derivative a fixpoint:
 canonicalise
the inner failing-lookahead term so equal-modulo-position states are shared (the
inner lookahead carries no per-offset information once its body has failed),
 or
evaluate the lookbehind's inner assertion by a bounded reverse check rather than by
folding it into the forward derivative term.

## Affected configurations

Blows up under every limits-enabled config that keeps the trigger intact:
`default`,
 `unicode(Ascii)`,
 `unicode(Full)`,
 `unicode(Javascript)`,
 and
`hardened(true)` all measure about 1.2 to 1.5 seconds for `(?<=$)` `find_all` at
N=256.
 The `flags` config is the only one that does not,
 and for an incidental
reason that differs per trigger,
 not because the bug is absent:

- For `(?<=$)`,
   `flags` sets `multiline(false)`,
   so `$` desugars to a bare
  end-of-input assertion with no newline lookahead,
   removing the inner positive
  lookahead.
- For `(?<=(?= ))`,
   `flags` sets `ignore_whitespace(true)`,
   so the literal space
  is stripped and `(?= )` becomes an empty (always-true) lookahead,
   removing the
  failing inner lookahead.

`(?<=(?=z))` (a non-whitespace,
 non-anchor inner lookahead) blows up under `flags`
too,
 confirming the config is not a real mitigation.
 The `unbounded_size` config
is out of scope here:
 it disables the size limits,
 and the invariant only governs
the limits-enabled configs.

## Relationship to other findings

- Distinct from BUG-11,
   which is a compile-time super-linear blowup on
  intersection-plus-class-repeat patterns.
   BUG-16 compiles instantly and is
  super-linear at match time,
   a different phase and a different code path
  (the lookbehind derivative,
   not compilation).
- The same `(?<=$)` pattern shape is the subject of a held-back correctness
  cluster (lookbehind-of-anchor position errors flagged by the Lean anchor round),
  which is held back pending a check of RE# lookbehind-of-lookaround semantics.
  This performance defect is independent of that question:
   a successfully compiled
  pattern hanging for 13 seconds on 512 bytes needs no semantic oracle,
   so it is
  filed on its own.

## Code quality

The Lookbehind derivative arm rebuilds a fresh `mk_lookbehind_internal` term every
step from the derived inner and prev,
 with no interning or fixpoint check on the
inner.
 A derivative engine relies on the state set reaching a fixpoint so the lazy
DFA is finite;
 here a positive lookahead inside the lookbehind defeats that
silently,
 because nothing canonicalizes the failing-lookahead term to a form equal
across positions.
 The arm should detect that the inner assertion carries no
per-offset information once its body has failed and collapse it,
 the same way the
`is_nullable` shortcut at `:1403` already collapses a succeeding lookahead.
 A
construction that assumes a fixpoint but has no guard against the one operator that
breaks it is the underlying issue.

## Recommendation for ieviev

Treat lookbehind-of-positive-lookahead as a fixpoint hazard in the derivative.
Either intern the inner failing-lookahead term so the lazy DFA stops minting a new
state per offset,
 or special-case the lookbehind's inner assertion to a bounded
reverse evaluation.
 Until then,
 the `$` desugaring under multiline turns the
extremely common `(?<=$)` into a denial-of-service trigger on attacker-controlled
input of a few kilobytes,
 with the size limits doing nothing to bound it.
