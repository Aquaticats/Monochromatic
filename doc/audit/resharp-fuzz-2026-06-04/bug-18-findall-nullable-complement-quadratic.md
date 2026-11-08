# BUG-18 find_all is O(n^2) on a nullable complement, restarting a forward scan per position

## Classification

- Type:
   performance,
   quadratic match time,
   no oracle needed.
- Phase:
   match time,
   the `find_all` nullable fallback path.
- Severity:
   denial of service.
   `~(a+)` (complement of a non-empty `a` run)
  matches a 96 KB input in 10.5 seconds and a 128 KB input in 18 seconds under
  the default config with the size limits enabled,
   breaking the project's
  "nothing over 10 seconds with limits on" invariant.
   The same pattern's
  `is_match` and `find_anchored` are instant;
   only `find_all` is quadratic.

## Minimal reproducer

```rust
//! Reproduction of the quadratic `find_all` on a nullable complement:
//! the time grows with the square of the input.

/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};
let re = Regex::with_options("~(a+)", RegexOptions::default()).unwrap();
let hay = vec![b'a'; 98304];
let _ = re.find_all(&hay);   // ~10.5 s; 128 KB is ~18 s
```

Command line (`--benchrep <hexpat> <bytehex> <N> <op> <cfgidx>` builds a hay of
`byte` repeated `N` times and times one op):

```sh
# ~(a+) find_all on 'a' * 98304, default config (cfgidx 0)
repro --benchrep 7e2861292b 61 98304 find_all 0   # -> 10.49
repro --benchrep 7e2861292b 61 131072 find_all 0  # -> 18.27
```

## Observed behaviour and scaling

`find_all` of `~(a+)` on `'a' * N`,
 default config:

```text
N= 16384   0.28 s
N= 32768   1.19 s
N= 65536   4.76 s
N= 98304  10.49 s
N=131072  18.27 s
```

Each doubling of N multiplies the time by about four:
 clean O(n^2).
 It crosses 10
seconds at roughly 95 KB of input.

The blowup is `find_all`-specific.
 On the same pattern and inputs:

```text
is_match        N=16384..65536  0.0000 s   (O(1), short-circuits)
find_anchored   N=16384..65536  0.0001 s   (one scan from position 0)
find_all        N=16384  0.35   N=32768 1.18   N=65536 4.58
```

## Trigger isolation

`find_all` scaling on `'a' * N`,
 default config:

```text
~(a+)      0.28 / 1.19 / 4.76    O(n^2)  : complement of a non-empty run
~(\w+)     0.32 / 1.13 / 4.61    O(n^2)  : same shape, perl class
~((a+))    0.27 / 1.16 / 4.60    O(n^2)  : grouping irrelevant
(~(a+))    0.28 / 1.13 / 4.56    O(n^2)  : grouping irrelevant
~(a*)      0.0000                fast    : star body includes empty already
~(\w)      0.0000                fast    : no quantifier
~(b)       0.0000                fast    : body cannot match the haystack
~(ab+)     0.0000                fast    : body cannot match an all-a haystack
~(a+)b     0.0001                fast    : trailing b kills the empty-everywhere matches
~(.+)      0.006 / 0.024 / 0.069 mild    : near-linear, small constant
a, [ab], 0{0,2}, a* all O(n) and instant.
```

The trigger is the complement of a `+`-quantified (non-nullable) body whose
language contains the empty string and whose body matches the haystack alphabet,
so the complement matches empty at every position.
 `~(a*)` does not trigger it
(the star body is already nullable,
 a different derivative shape),
 nor does a
complement whose body cannot match the input (`~(b)`,
 `~(ab+)` on all-`a`).

The quadratic is not complement-specific.
 Any nullable pattern that matches empty
at many positions while its per-position forward scan reads far hits the same path.
The non-complement pattern `${0,2}([a-c]_+&((?:a)*))a{1,3}[^a]\w*` (nullable via
`${0,2}`,
 far-reaching via the trailing `\w*`) has `find_all` of 0.27 / 1.09 /
4.35 s at N = 16384 / 32768 / 65536,
 the same O(n^2).
 A bare nullable like
`(a?)\w*` stays linear because the greedy `\w*` produces one big match instead of a
storm of empty ones;
 the blowup needs both empty-everywhere matching and a far
per-position scan.

## Root cause localization

`find_all` routes a nullable pattern to `find_all_nullable_slow`
(`resharp-engine/src/lib.rs:1794`),
 which restarts a forward scan from every
position:

```rust
//! The slow path of `find_all` for nullable patterns:
//! every position starts a fresh forward scan over the rest of the input,
//! so the work is quadratic.
// resharp-engine/src/lib.rs:1794
/// Finds the matches of a nullable pattern by scanning forward from every candidate start,
/// advancing one byte after an empty match or a miss.
fn find_all_nullable_slow(fwd, b, input, matches) -> Result<(), Error> {
    let mut pos = 0;
    while pos < input.len() {
        let max_end = fwd.scan_fwd_slow(b, pos, input)?;   // fresh O(n) scan from pos
        if max_end != NO_MATCH && max_end > pos {
            matches.push(Match { start: pos, end: max_end });
            pos = max_end;
        } else if max_end != NO_MATCH {
            matches.push(Match { start: pos, end: pos });   // empty match
            pos += 1;                                        // advance one byte
        } else {
            pos += 1;
        }
    }
    // ...
}
```

For `~(a+)` the longest match at every position is the empty string (any non-empty
all-`a` prefix is in `a+`,
 so excluded from the complement),
 so the loop takes the
empty-match branch and advances one byte at a time.
 Each iteration calls
`scan_fwd_slow(pos, input)`,
 which scans forward from `pos` to determine the
longest match and does O(n) work before concluding `max_end == pos`.
 N positions
times an O(n) scan each is O(n^2).

The hardened config does not hit this path:
 `find_all` under `hardened(true)` uses
the DFA driver (`find_all_dfa`,
 `resharp-engine/src/lib.rs:1713`),
 a single linear
pass,
 and measures 3 milliseconds where the default path measures 4.6 seconds at
N=65536.
 That a sibling code path matches the same pattern linearly shows the
quadratic is an avoidable property of the nullable fallback,
 not inherent to the
pattern.

The fix is to make `find_all_nullable_slow` carry state forward instead of
rescanning from scratch at each position (the DFA driver already does this),
 so a
run of empty matches costs O(n) total rather than O(n) per position.

## Affected configurations

Quadratic under every limits-enabled config except `hardened`:
 `default`,
`unicode(Ascii)`,
 `unicode(Full)`,
 `unicode(Javascript)`,
 and `flags` all measure
about 4.5 to 4.7 seconds at N=65536;
 `hardened` is 3 milliseconds because it takes
the DFA driver.
 The `unbounded_size` config is out of scope (it disables the size
limits,
 and the invariant governs only the limits-enabled configs).

Whether hardened returns the same match set here or is fast because it reports a
different set is a separate question (see BUG-8,
 where hardened `find_all` differs
from default on complement patterns);
 this finding is only that the default path
is quadratic where a linear path exists.

## Relationship to other findings

- Distinct from BUG-16 (match-time blowup in the lookbehind derivative) and BUG-17
  (compile-time blowup on bracketed perl classes).
   BUG-18 is in the `find_all`
  driver and is specific to nullable complements.
- The directed corpus surfaced it inside larger patterns,
   the clearest being
  `(0{0,2}|(b{0,1}c+\d{0,1}|~(\w+)))`,
   whose `find_all` is 19 seconds on 128 KB,
  driven entirely by the `~(\w+)` factor.

## Code quality

The method name `find_all_nullable_slow` admits the problem:
 it is a knowingly slow
fallback.
 Three things should have been written differently.

- The per-position rescan.
   `scan_fwd_slow(pos, input)` is called once per `pos`
  with no state carried between calls,
   so a run of empty matches re-scans the
  suffix from scratch each time.
   The DFA driver (`find_all_dfa_inner`) already does
  a single stateful sweep;
   the nullable path should reuse that sweep,
   not restart
  it.
   A correctness-preserving fast path existing right next to a quadratic slow
  path is a maintenance trap.
- Two `find_all` implementations that disagree.
   The default path and the hardened
  DFA path return different results on some complement patterns (BUG-8) and have
  different complexity here.
   Two code paths intended to compute the same function,
  diverging on both result and cost,
   is a design smell:
   one of them is wrong and
  the other is slow.
- No guard on the cost.
   There is no bound tying the per-position scan to the total
  input,
   so the size limits (which cap pattern size) do not cap match cost.

## Recommendation for ieviev

Replace the per-position rescan in `find_all_nullable_slow` with a forward-stateful
sweep,
 or always route through the DFA driver that hardened already uses.
 A
nullable pattern that matches the empty string at many positions is common
(`~(...)`,
 optionals,
 stars in an alternation),
 so this quadratic is reachable far
beyond the minimal `~(a+)`,
 and the size limits do not bound it because the cost is
in the number of input positions,
 not the pattern.
