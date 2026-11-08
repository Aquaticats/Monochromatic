# BUG-27 word boundary nullability flips on the empty string under nullable composition

## Classification

- Type:
   correctness,
   soundness.
   When a word boundary `\b`/`\B` is composed with a
  nullable subexpression in a concat,
   its truth on the empty string flips:
   `\b`
  becomes spuriously satisfiable (matches the empty string) and `\B` spuriously
  unsatisfiable (fails on the empty string).
- Phase:
   compile (word-boundary lowering and concat simplification) feeding the
  empty-input nullability.
- Severity:
   low-to-moderate.
   The trigger is narrow (empty input plus a nullable filler
  around the boundary),
   but it is a genuine false positive and false negative on the
  primary `is_match` API,
   in every config.

## Minimal reproducer

```rust
//! Reproduction of the flipped nullability of the word boundary assertions on the empty input,
//! once under composition with a zero-width repeat.

/// The regex type whose `is_match` answers wrongly.
use resharp::Regex;
// \b is FALSE on the empty string (no word char to bound), so \b a{0} \b matches nothing.
assert_eq!(Regex::new(r"\ba{0}\b").unwrap().is_match(b"").unwrap(), false); // FAILS: true
// \B is TRUE on the empty string, so \B a{0} \z matches the empty string.
assert_eq!(Regex::new(r"\Ba{0}\z").unwrap().is_match(b"").unwrap(), true);  // FAILS: false
```

A bare `\b` is correct on the empty string;
 the flip only appears once the boundary is
concatenated with a nullable filler such as `a{0}`.

## Observed behaviour

`is_match("")`,
 all configs identical (default/ascii/full):

```text
bare \b            -> false   (correct)
bare \B            -> true    (correct)
\b a{0} \b         -> true    (WRONG, expected false)
\b a{0} \z         -> true    (WRONG, expected false)
\z a{0} \b         -> true    (WRONG, expected false)
\A a{0} \b         -> true    (WRONG, expected false)
\B a{0} \B         -> false   (WRONG, expected true)
\B a{0} \z         -> false   (WRONG, expected true)
```

Bare adjacent boundary forms (`\b\b`,
 `\B\z`,
 `\b\z`,
 `\bb*\b`,
 `\b()\b`) are rejected
at compile (`UnsupportedResharpRegex`);
 a `{0}` quantifier slips past that rejection
(`a{0}` reduces to the empty string only after the boundary composition is built),
compiles,
 and then evaluates wrongly.
 On non-empty input the same patterns are
correct;
 only the empty-string position is affected.
 Surfaced by `repro
--divergebatch` (resharp vs the `regex` crate,
 which gives `\b`=false / `\B`=true on
the empty string).

## Expected behaviour

A word boundary `\b` matches only where exactly one side is a word character;
 on the
empty string neither side is,
 so `\b` is false and `\B` is true,
 regardless of any
adjacent nullable subexpression.
 `\b a{0} \b` must not match `""`;
 `\B a{0} \z` must.

## Root cause

`\b` is lowered to boundary lookarounds built on the language complement of a word
char,
 `~(\w)` (the `debug` forward dumps show `\b` as `(?<=\A~(\w...))` on the left and
`(?=~(\w...))` on the right).
 `~(\w)` is nullable:
 it matches the empty string (the
empty string is not a single word char),
 which is the right behaviour for the
end-of-input side of a boundary (`a\b` on `"a"` correctly matches:
 the lookahead
`~(\w)` is satisfied at end).
 The defect appears on the empty string,
 where the single
position is both the begin and end side:
 both the lookbehind `~(\w)` and the lookahead
`~(\w)` are satisfied,
 so the composed boundary is treated as true.
 A correct `\b`
additionally requires a word char on exactly one side;
 the bare-`\b` lowering enforces
this (bare `\b` on `""` is correctly false),
 but the requirement is lost when the
boundary is concatenated with a nullable filler,
 so the empty-string nullability
collapses to "both sides non-word -> boundary holds.
" `\B` (the negation) inherits the
inverse flip.

This is the same conceptual hazard as BUG-7 (a language complement `~` that matches
the empty string standing in for an element-level constraint) and is adjacent to
BUG-26 (empty-string nullability mishandled in concat simplification),
 but in a
distinct code path:
 the word-boundary lowering plus the concat nullability of that
lowering,
 rather than the ascii `\W` class lowering (BUG-7) or the `End`-head concat
rule (BUG-26).
 Pinning the exact simplification that drops the "word char on one side"
requirement needs a walk of the lowered lookaround derivative;
 the empirical flip and
its config independence are certain.

## Affected configurations

All (default,
 ascii,
 full,
 js);
 the boundary lowering is shared.
 Correctness defect,
so the limits-disabling config is irrelevant.

## Relationship to other findings

- BUG-7:
   ascii `\W` matching the empty string via `mk_compl`;
   same "complement is
  nullable" theme,
   different code path (perl-class lowering vs boundary lowering).
- BUG-26:
   `\z\A` reduced to the empty language;
   same "empty-string nullability of an
  anchor composition is wrong" theme,
   opposite direction (false negative there,
   both
  directions here),
   different rule.
- BUG-20/BUG-21:
   begin-context handling at match time;
   this one is compile-time
  nullability of boundary composition.

## Code quality

The end-of-input side of a boundary legitimately uses the nullable `~(\w)`,
 but the
empty-string case needs the "exactly one side is a word char" exclusivity that bare
`\b` already encodes.
 The composition path should preserve that exclusivity (or the
boundary lowering should not rely on a nullable complement for both sides at once),
 so
that `\b`/`\B` cannot both collapse to "both sides non-word" on the empty string.
