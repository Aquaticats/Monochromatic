# BUG-17 a perl shorthand inside a character class makes bounded-repeat compile super-linear

## Classification

- Type:
   performance,
   super-linear compile time,
   no oracle needed.
- Phase:
   compile time (`Regex::with_options`),
   before any input is seen.
- Severity:
   denial of service at compile.
   A 13-character pattern,
  `([\w]{3,5}){3,3}`,
   takes 15 seconds to compile under the default config with
  the size limits enabled,
   breaking the project's own "nothing over 10 seconds
  with limits on" invariant.
   The semantically identical `(\w{3,5}){3,3}` (no
  brackets) compiles in 20 milliseconds.

## Minimal reproducer

```rust
//! Reproduction of the compile-time blowup of a bracketed class under a bounded repeat:
//! two patterns,
//! the second of which takes more than ten seconds to construct.

/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};
let _ = Regex::with_options("[\\w]{3,5}", RegexOptions::default()).unwrap();
// ~1.76 s for a five-token pattern

let _ = Regex::with_options("([\\w]{3,5}){3,3}", RegexOptions::default()).unwrap();
// ~15.3 s; > 10 s invariant violated
```

The trigger is a single editorial choice:
 writing a perl shorthand inside a
character class,
 `[\w]`,
 instead of bare,
 `\w`.
 The two are identical languages,
but the bracketed form does not reach the fast single-predicate path.

Command line (`--compile1 <hexpat> <cfgidx>` prints compile seconds):

```sh
# [\w]{3,5} hex, default config (cfgidx 0)
repro --compile1 5b5c775d7b332c357d 0     # -> 1.7619|ok=true
# (\w{3,5}){3,3} hex, default config
repro --compile1 285c777b332c357d297b332c337d 0   # -> 0.0196|ok=true
```

## Observed behaviour and isolation

All measured with `--compile1`,
 default config (cfgidx 0),
 solo:

```text
\w{3,5}              0.0059   bare shorthand: fast
[\w]{3,5}            1.7619   shorthand inside a class: 300x slower
(\w{3,5})            0.0061   grouped bare: fast
(\w{3,5}){1,1}       0.0067   fast
(\w{3,5}){2,2}       0.0174   bare, nested repeat: fast
(\w{3,5}){3,3}       0.0196   bare, nested x3: fast
([\w]{3,5}){2,2}     6.5696   bracketed, nested repeat: blows up
([\w]{3,5}){3,3}    15.3266   bracketed, nested x3: > 10 s
[\w\d]{3,20}         3.5608   union of overlapping shorthands, single repeat
(\d{3,5}){3,3}       0.0002   contiguous class: fast
```

The blowup needs two ingredients that compound:

- A class whose lowering is a union of several pieces.
   `[\w]` lowers to a
  multi-member class set,
   not the single predicate that bare `\w` produces,
   even
  though they are equal.
   A union of overlapping shorthands,
   `[\w\d]`,
   blows up on
  its own with a single repeat (3.56 s).
- Bounded repetition `{m,n}` over that class.
   Bare `\w` stays fast at any nesting;
  the bracketed `[\w]` is 1.76 s for one repeat,
   6.57 s nested twice,
   15.3 s
  nested three times.

It is not class size.
 The entire byte range `[\x00-\xff]{3,5}` nested three deep
compiles in 0.5 ms,
 and the explicit `[A-Za-z0-9_]{3,20}` (the exact expansion of
`\w`) compiles in 0.3 ms. Only the perl shorthand inside brackets is slow,
 so the
defect is a missing canonicalization of `[\w]` to its predicate,
 not the class
content.

## Scaling

`([\w]{3,5}){N,N}` under the default config:

```text
N=1   1.66 s
N=2   6.45 s
N=3  15.50 s
N=4  30.19 s
```

Super-linear in the outer repeat count (about N^2 with a large constant),
 crossing
10 seconds between N=2 and N=3.
 It is mode-independent:
 the same pattern measures
1.69 / 6.46 / 15.27 s under `unicode(Full)`,
 so unicode mode is not the driver
(the directed corpus happened to surface it under `full` only because those
patterns set that flag).

## Root cause localization

`mk_repeat` (`resharp-algebra/src/lib.rs:3710`) lowers `{lower,upper}` by
unrolling into a concatenation of `upper - lower` optional copies plus `lower`
mandatory copies of the body node:

```rust
//! The repeat constructor of the algebra:
//! it builds one node per repetition,
//! so a large class repeated several times multiplies its own size.
// resharp-algebra/src/lib.rs:3710
/// Builds the node for a body repeated between `lower` and `upper` times,
/// as the optional body `upper - lower` times followed by the body `lower` times.
pub fn mk_repeat(&mut self, body_id: NodeId, lower: u32, upper: u32) -> NodeId {
    let opt = self.mk_opt(body_id);            // EPS | body
    let mut nodes1 = vec![];
    for _ in lower..upper { nodes1.push(opt); }
    for _ in 0..lower { nodes1.push(body_id); }
    self.mk_concats(nodes1.into_iter())
}
```

The unrolled structure itself is linear and shares one body node,
 so the blowup is
not here;
 it is in the subsequent derivative and minterm closure over that
structure,
 and it depends entirely on how `body_id` represents the class.
 Bare
`\w` is a single predicate node,
 so the concatenation of optionals stays a small
derivative state set.
 `[\w]` is lowered to a class-set union (the parser rewrites a
perl shorthand inside a bracket into a union of literals and ranges;
 see the
perl-to-union construction around `resharp-parser/src/lib.rs:186`),
 so each
unrolled copy multiplies that union and the derivative closure over
`(EPS | union){k}` is super-polynomial in `k`.

The fix is to canonicalize a perl shorthand appearing inside a character class to
the same single predicate that bare `\w` produces (or to fold any class-set union
to a single minterm predicate before repetition),
 so `[\w]`,
 `\w`,
 and
`[A-Za-z0-9_]` all share the fast path they already share when not bracketed.

## Affected configurations

Reproduces under every limits-enabled config (`default`,
 `unicode(Ascii)`,
`unicode(Full)`,
 `unicode(Javascript)`,
 `hardened`,
 `flags`);
 the compile cost is
the same because it is the class lowering and the repeat unroll,
 neither of which
the configs change.
 The `unbounded_size` config is out of scope:
 it disables the
size limits,
 and the 10-second invariant only governs the limits-enabled configs.

## Match-time manifestation

The same un-canonicalized `[\w]` class also costs seconds at match time,
 without
any repetition,
 when an anchor sits in front of it and the input is diverse.
`$[\w]` (end-anchor then bracketed word class) runs `is_match` over 16 KB of
varied bytes in about 1.15 seconds,
 fixed in the input length,
 under the default
config:

```text
$[\w]    is_match cyc(N):  N=4096 1.16   16384 1.15   32768 1.14
$[\w\d]  is_match cyc(16384) 1.13
$[a-z]   0.0000   $[\d] 0.004   $[\s] 0.0001   $[abc] 0.0000   $\w 0.0009
```

Only the bracketed perl word shorthand (`$[\w]`,
 `$[\w\d]`) is slow;
 the explicit
ranges,
 the smaller shorthands,
 and the bare `$\w` are all fast,
 the same split as
the compile case.
 So the bracketed-class representation is expensive to construct
into a lazy DFA as well as to repeat,
 and the adjacent anchor's lookahead is what
forces that construction over diverse input.
 Fixing the canonicalization removes
both the compile blowup and this match-time cost.
 (The full-mode analogue,
 where
bare `\w` is large enough to trigger the same anchor-adjacent construction cost,
 is
BUG-19.
)

## Relationship to other findings

This is the confirmed root cause of BUG-11.
 BUG-11's exact trigger
`[\w]{3,5}[\w]([^a]&a+)` compiles in 2.79 s;
 replacing only the brackets,
`\w{3,5}\w([^a]&a+)`,
 drops it to 0.0068 s (400x faster),
 and the intersection
alone `([^a]&a+)` is 0.0003 s.
 The bracketed prefix `[\w]{3,5}[\w]` is 2.14 s by
itself with no intersection.
 So BUG-11's cost is entirely the bracketed `[\w]`,
 the
intersection is incidental,
 and BUG-11 is the same defect as BUG-17 seen through a
pattern that also happened to contain an intersection.

Distinct from BUG-16,
 which is a match-time blowup in the lookbehind derivative;
BUG-17 is entirely at compile time and never reaches the matcher.

## Code quality

One language,
 three internal representations.
 `\w`,
 `[\w]`,
 and `[A-Za-z0-9_]`
denote the identical class,
 but each lowers to a different node and the three
differ by orders of magnitude in compile and match cost (bare `\w` fast,
 explicit
ranges fast,
 bracketed shorthand catastrophic).
 A class should be normalized to a
single canonical predicate at parse time regardless of how it was spelled;
 that the
fast path already exists for two of the three spellings shows the normalization is
missing,
 not impossible.
 The cost surfacing only for `\w` (and `[\w\d]`) and not
`[\d]`/`[\s]` confirms it is the multi-piece set that is left un-folded.
 A
character-class set carrying several overlapping predicates into repetition,
 rather
than one merged minterm,
 is the defect;
 the same un-folded set is what makes BUG-11
and BUG-19 expensive too.

## Recommendation for ieviev

Canonicalize perl shorthands inside character classes to the predicate form bare
shorthands already use,
 ideally folding every character-class set to a single
minterm predicate before it can be repeated.
 Until then `[\w]{3,5}` and any
`[...]` containing a shorthand is a compile-time denial-of-service trigger at
trivial pattern sizes,
 and the `max_repeat` size cap does not bound it (the cap
limits each `{m,n}` to 500 but the cost is in the class lowering,
 not the count).
