# BUG-23 full-unicode \w bounded-repeated blows up compile time super-linearly

## Classification

- Type:
   performance,
   compile-time algorithmic complexity.
- Phase:
   compile (`Regex::with_options`),
   full-unicode mode only.
- Severity:
   bug.
   `\w{12}` takes 9.6 s and `\w{16}` 15.6 s to compile under the full
  config,
   crossing ieviev's 10 s invariant on a trivial,
   everyday pattern with limits
  enabled.
   `\w{8}` is already 4.7 s.
   The same patterns compile in well under a
  millisecond in the default and ascii configs.

## Minimal reproducer

```rust
//! Reproduction of the compile-time blowup of a bounded repeat of a word class in the full Unicode mode,
//! compared with the default and ASCII modes in the comment.

/// The regex type,
/// its options and the Unicode mode that selects the full classes.
use resharp::{Regex, RegexOptions, UnicodeMode};
let full = RegexOptions::default().unicode(UnicodeMode::Full);
let _ = Regex::with_options(r"\w{16}", full).unwrap(); // ~15.6 s to COMPILE
// default/ascii compile the same pattern in ~0.03 s.
```

`\w{16}` is a flat bounded repeat of the single shorthand class `\w`;
 no nesting,
intersection,
 complement,
 or lookaround is involved.

## Observed behaviour

Compile time,
 full config (cfg2),
 `--compile1`:

```text
\w{2}    0.647 s
\w{4}    1.604 s
\w{8}    4.680 s
\w{12}   9.569 s
\w{16}   15.610 s
```

Roughly O(count^1.6):
 a 2x increase in the repeat count near triples the time.
 Across
configs:

```text
                default   ascii    full
\w{2,4}         0.006     0.0002   0.742
(\w{2,4}){2,4}  0.013     0.0002   3.428
\w{16}          0.028     0.0002   15.610
```

Class specificity at `{8}` in full mode:

```text
\w{8}    4.500 s     \W{8}    1.061 s
\d{8}    0.003 s     \s{8}    0.001 s
.{8}     0.003 s     [a-z]{8} 0.0002 s
```

Only `\w` (and to a lesser degree `\W`) is affected.
 `\d`,
 `\s`,
 `.`,
 and ASCII
ranges bounded-repeated the same way are instant in full mode.
 The trigger is the
full-unicode word-character class specifically.

## Expected behaviour

Compiling `\w{16}` is trivial in every other mode and engine;
 it should take
milliseconds.
 The full-unicode word class is large but fixed;
 bounded-repeating it
must not multiply compile cost super-linearly with the repeat count.

## Root cause

The parser lowers `{n}`,
 `{n,}`,
 and `{n,m}` through `mk_repeat`
(`resharp-parser/src/lib.rs:2030`,
 `:2032`,
 `:2037`),
 which fully unrolls the
quantifier:

```rust
//! The repeat constructor of the algebra,
//! which builds one node per repetition of the body.
// resharp-algebra/src/lib.rs:3710
/// Builds the node for a body repeated between `lower` and `upper` times,
/// as the optional body `upper - lower` times followed by the body `lower` times.
pub fn mk_repeat(&mut self, body_id: NodeId, lower: u32, upper: u32) -> NodeId {
    let opt = self.mk_opt(body_id);
    let mut nodes1 = vec![];
    for _ in lower..upper { nodes1.push(opt); }
    for _ in 0..lower { nodes1.push(body_id); }
    self.mk_concats(nodes1.into_iter())
}
```

The unrolled node itself is cheap (the copies are interned to one `NodeId`,
 and
`collect_sets` dedups them,
 so the minterm partition stays ~2:
 `\w` vs `\W`).
 The
blowup is in DFA construction:
 the derivative of `\w{16}` walks through
`\w{15}`,
 `\w{14}`,
 ...,
 one distinct state per remaining count,
 and each state's
`der` / `prune_fwd` / `transition_term` performs set operations over the full-unicode
`\w` set,
 a large multi-byte BDD.
 With a cheap class (`\d`,
 `\s`,
 ASCII) those set
operations are negligible,
 so only `\w`/`\W` show the cost.
 Empirically the per-state
cost also grows with the count (0.33 s/state at `\w{2}`,
 ~1.0 s/state at `\w{16}`),
so the total is super-linear,
 not merely `count x constant`.

The algebra already has a native counted-repetition node,
 `Kind::Counted` with
`mk_counted` (`resharp-algebra/src/lib.rs:3490`,
 used by the derivative machinery at
`:1524` and `:4523`),
 which represents a bounded repeat as one node with a count
rather than N unrolled copies.
 Routing bounded repeats of an expensive class through
`mk_counted` instead of `mk_repeat`,
 or memoizing the per-count set operations so the
full-unicode `\w` transition is computed once and reused across counts,
 removes the
multiplication.
 The fix lives at the parser lowering (`lib.rs:2030-2037`) or in
`mk_repeat`.

## Affected configurations

Full only.
 Default and ascii compile these in microseconds because their `\w` is the
small ASCII word set.
 The limits-disabling `unbounded_size` config is exempt by
policy and would be no faster.

## Relationship to other findings

- Distinct from BUG-17 (`([\w]{3,5}){3,3}` compile blowup):
   that is the bracketed
  perl class `[\w]` lowering to an un-canonicalized class-set union and is slow in the
  DEFAULT config too (`([\w]{2,4}){2,4}` = 4.0 s default,
   `[\w]{16}` = 17.1 s
  default).
   BUG-23 is the BARE shorthand `\w` and is fast in default (0.013 s,
  0.028 s);
   it is slow only in full mode,
   driven by the full-unicode class size,
   not
  by bracket canonicalization.
   Confirmed side by side:
   `\w{16}` is 0.028 s default /
  15.6 s full,
   while `[\w]{16}` is 17.1 s default / 17.3 s full.
- Distinct from BUG-19 (full-mode `$?\w` construction cost):
   that is a match-time DFA
  construction cost on diverse input;
   BUG-23 is compile-time and scales with the
  bounded-repeat count.

## Code quality

`mk_repeat` unrolling a bounded quantifier into N concatenated copies is the kind of
choice a reasonable author revisits once a `Kind::Counted` node already exists in the
same module:
 the unroll is correct but trades a compact counted representation for one
that multiplies downstream derivative work by the count.
 For cheap classes the trade
is invisible;
 for an expensive set it is the whole bug.
 Worth lowering bounded repeats
to `mk_counted` (or memoizing the per-count transition) so compile cost tracks the
class once,
 not once per repetition.
