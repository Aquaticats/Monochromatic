# REG-1 zero-width negative lookahead duplicates find_all spans

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Classification

- Type:
   correctness regression.
   `find_all` returns the same zero-width match
  twice (and three or more times for longer inputs) for a negative lookahead of a
  zero-width assertion,
   such as `(?!\A)`.
- Introduced by:
   commit `4ffe1cc` "cleaning up and simplifying edge cases" (on
  `main` as `264e85b` "bump version",
   tag `v0.6.9`).
   Parent `a7ab016` (the
  campaign baseline) does not have the defect.
- Phase:
   compile (negative-lookahead lowering) feeding match-time null collection.
- Severity:
   low-to-moderate.
   The trigger family is narrow (a zero-width negative
  lookahead,
   blocked only at position 0),
   but it regresses a pattern that the
  parent compiled correctly,
   and it produces a `find_all` result that violates the
  non-overlapping leftmost-longest contract (the same span appears more than once).
  `is_match` and `find_anchored` are unaffected.

## Minimal reproducer

```rust
//! Reproduction of the duplicated zero-width span:
//! the negative lookahead of the begin anchor emits the empty match at offset one twice,
//! which the parent commit did not.

/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};
let re = Regex::with_options(r"(?!\A)", RegexOptions::default()).unwrap();
let fa: Vec<(usize, usize)> = re.find_all(b"ab").unwrap().iter().map(|m| (m.start, m.end)).collect();
// parent a7ab016: [(1, 1), (2, 2)]            correct
// v0.6.9       : [(1, 1), (1, 1), (2, 2)]     WRONG: 1:1 emitted twice
assert_eq!(fa, vec![(1, 1), (2, 2)]);
```

## Observed behaviour

`find_all` on `(?!\A)`,
 identical across configs (default,
 full,
 hardened):

```text
hay     parent a7ab016            v0.6.9 (regressed)
"ab"    1:1,2:2                   1:1,1:1,2:2
"abc"   1:1,2:2,3:3               1:1,1:1,2:2,2:2,3:3
"aaaa"  1:1,2:2,3:3,4:4           1:1,2:2,2:2,3:3,3:3,4:4
" a b " 1:1,2:2,3:3,4:4,5:5       1:1,2:2,3:3,3:3,4:4,4:4,5:5
```

The duplicated positions are the interior nullable positions;
 which positions
double depends on where the byte-class transitions fall (so `"aaaa"` doubles 2 and
3,
 while `"abc"` doubles 1 and 2),
 but the contract violation (a repeated identical
span) holds for every input of length two or more.
 The defect surfaces in the
internal null vector that drives `find_all`.
 With the engine `debug` feature:

```text
pat="(?!\A)" hay="abc"
parent a7ab016 : [nulls] [3, 2, 1]              -> find_all [(1,1),(2,2),(3,3)]
v0.6.9         : [nulls] [3, 2, 2, 1, 1]         -> find_all [(1,1),(1,1),(2,2),(2,2),(3,3)]
```

The whole `(?!<begin-anchor>)` zero-width family regresses the same way:
`(?!\A)`,
 `(?!\A)|a`,
 `a|(?!\A)`,
 `(?!\A)a?`,
 `(?!\A)a*`,
 `(?!\A){2}`,
`(?!\A)(?!\A)`,
 `(?!\A)(?!\z)`,
 `(?!\A)(?!$)`,
 `(?!\z)(?!\A)`,
 `(?!$)(?!\A)`.
 Of
these,
 `(?!\A)`,
 `(?!\A)|a`,
 and `a|(?!\A)` compiled correctly on the parent,
 so
they are true regressions,
 not warts on newly accepted patterns.

## Expected behaviour

`find_all` must return each match once.
 `(?!\A)` is the empty match at every
position except 0,
 so on an n-byte input it must return exactly the zero-width
spans `1:1, 2:2, ..., n:n`,
 each once.

## Root cause

The commit added a zero-width fast path to `mk_neg_lookahead`
(`resharp-algebra/src/lib.rs:3554`):

```rust
//! The constructor of a negative lookahead after the change that added a branch for bodies of zero width.

/// Builds the node for a negative lookahead of `body`,
/// with a direct lowering when the body has zero width and the old lowering otherwise.
pub fn mk_neg_lookahead(&mut self, body: NodeId, rel: u32) -> NodeId {
    let (_, p_max) = self.get_min_max_length(body);
    if p_max == 0 {
        let not_body = self.mk_compl(body);
        return self.mk_inter(NodeId::EPS, not_body);   // <- new branch
    }
    // ... old lowering: lookahead over ~(body . TS) . \z
}
```

When the lookahead body is zero-width (`p_max == 0`,
 true for `\A`,
 `\z`,
 `^`,
 `$`,
`\b`,
 `\B`),
 `(?!body)` is now lowered to `EPS & ~body` (the engine prints this as
`(&~(\A))`,
 matching the commit's own `not-begins?` normalize test:
`(?!\A)(?=[A-Z])` becomes `(&~(\A))(?=[A-Z]_*)`).
 That structure registers the
interior nullable position under two distinct null states that both satisfy the
`END` mask,
 so the reverse null collector (`collect_rev`,
 the `nulls.push(pos)`
sites at `resharp-engine/src/engine.rs:1544` onward) pushes the same position
twice,
 and `find_all` emits one zero-width span per null entry.

Bisection proof (single-hunk revert on a fork copy):
 reverting only this branch,
with every other hunk of the commit left in place,
 restores `[nulls] [3, 2, 1]`
and `find_all [(1,1),(2,2),(3,3)]`.
 Reverting only `resharp-algebra/src/nulls.rs`
(the `and_id` set-intersection rewrite) does not change the duplicate,
 ruling out
that hunk.
 The defect is the negative-lookahead lowering,
 not the null-set algebra.

The same new branch is also a net improvement on the rest of the family:
 it makes
`(?!\A)a`,
 `(?!\A)*`,
 `(?!\A)\w`,
 `(?!\A)b+` compile (the parent rejected them with
`UnsupportedPattern`),
 and it makes `(?!\A)(?=[A-Z])`,
 `(?!\A){2}`,
 `(?!\A)(?!\A)`
match where the parent wrongly returned no match.
 The duplicate-span defect is the
single regression riding along with that fix,
 and a dedup of `nulls` after
collection (or a lowering that registers each nullable position once) would keep
the fix without the duplicate.

## Affected configurations

All (default,
 ascii,
 full,
 js,
 hardened).
 The duplication is in the shared reverse
null collection,
 not in any one scan driver,
 so it is config-independent and the
limits-disabling config is irrelevant (this is a correctness defect,
 not a timing
one).

## Scope of the commit (for the campaign record)

Verified by a before/after `find_all` differential over the 6426-pattern hard
corpus (96390 pattern-by-haystack pairs,
 default config):
 zero differences.
 The
commit's entire behavioural footprint is the zero-width negative-lookahead and
`\b\B`-anchor-composition family.
 Outside that family nothing changed,
 so this
regression is the only behavioural risk the commit carries on the fuzzed surface.

Alongside the regression,
 the commit also:

- turns the parent's compile-time panic on `\A((?<=a)B+|x)` into a clean
  `UnsupportedPattern` (the new reverse `R . ((?<=x)|y)` handling at
  `resharp-algebra/src/lib.rs` around the `reverse` concat case),
- newly rejects `\A\b\B`,
   `^\b\B`,
   `(?!\A)^`,
   `(?!\z)^` at compile,
   where the
  parent compiled them to an inconsistent `is_match=false` with a spurious
  `find_anchored` zero-width match (a BUG-20-shaped contradiction);
   the rejection
  removes that contradiction,
- removes three `\z(?<!,)` end-anchor-with-negative-lookbehind tests from
  `resharp-engine/tests/lookaround.toml` and marks the two cross-checking tests
  (`is_match_agrees_with_find_all`,
   `hardened_cross_feature`) `#[ignore]` for
  release-only runs.

## Relationship to the filed bugs

- None of the 23 filed bugs (BUG-1 through BUG-27,
   no BUG-24) are fixed by this
  commit.
   Every filed reproducer returns byte-identical results on the parent and
  on v0.6.9,
   verified by the full reproducer harness.
   This regression is a new,
  separate defect,
   not a reopened bug.
- Closest in theme to BUG-13 (a zero-width lookahead leaking into the match span)
  and BUG-4 (find_all emitting a wrong zero-width-related span),
   both zero-width
  `find_all` defects,
   but distinct:
   those are span-value errors,
   this is a
  duplicate-emission count error in a different code path (negative-lookahead
  lowering plus reverse null collection).

## Oracle gap

The campaign's internal-consistency oracle does not catch this.
 Two identical
zero-width spans `1:1` then `1:1` do not trip `OVERLAP` (which fires only on
`start < prev_end`,
 and `1 < 1` is false),
 nor `INCONSIST` (`is_match` is true and
`find_all` is non-empty,
 so they agree),
 nor `BOUNDS`.
 The duplicate was found by a
direct before/after `find_all` differential with an explicit duplicate-span check,
not by the internal oracle.
 A `DUPSPAN` check (any two emitted matches with equal
`start` and equal `end`) should be added to the oracle so a future zero-width
duplication regresses loudly.

## Verification tooling

- After clone:
   `/tmp/agent/resharp-after-20260604` (main at `264e85b`,
   v0.6.9).
- Parent clone:
   `/tmp/agent/resharp-fuzz-20260604` (`a7ab016`,
   campaign baseline).
- Bisection fork:
   `/tmp/agent/resharp-bisect` (copy of after with only the
  `mk_neg_lookahead` zero-width branch reverted;
   rebuild restores correct output).
- Differential harnesses:
   `/tmp/agent/recheck.ts` (23 reproducers),
   `dup.ts` and
  `classify.ts` plus `genpairs.ts` (before/after `find_all` differential),
  `dbg-after` / `dbg-before` / `dbg-bisect` (engine `debug`-feature null dumps).
