# BUG-12 negative lookahead of a class makes a non-nullable pattern nullable

## Classification

- Type:
   correctness,
   wrong nullability,
   spurious empty match.
- Phase:
   compile-time nullability,
   surfaced at match time on the empty input and
  at any zero-width position.
- Severity:
   soundness.
   Both `is_match` and `find_all` agree on the wrong answer,
  so no self-consistency oracle catches it.
   It was found only by the Lean
  formalization (the verified ground truth) and confirmed by the dotnet engine.

## Minimal reproducer

```rust
//! Reproduction of the nullable negative lookahead:
//! the pattern matches the empty input although it must not.

/// The regex type whose `is_match` answers wrongly.
use resharp::Regex;
let re = Regex::new(r"(?!\w)0+").unwrap();
assert_eq!(re.is_match(b"").unwrap(), false); // FAILS: resharp returns true
```

Command line:

```sh
repro --pair "$(printf '%s' '(?!\w)0+' | xxd -p | tr -d '\n')" ''
# rust: im=1|fa=0:0|le=0   (wrong)
# dotnet on the same: im=0|fa=|le=-1   (correct)
# Lean llmatch: no match (correct)
```

## Observed behaviour

`(?!\w)0+` on the empty string:
 `(?!\w)` is a zero-width assertion that succeeds
when the next character is not a word character,
 which holds vacuously at end of
input,
 but `0+` then requires at least one `0`.
 There is no such character,
 so
the pattern does not match the empty string.
 resharp returns `is_match = true`
and `find_all = [(0,0)]`,
 both wrong.
 The dotnet engine and the Lean
formalization both return no match.

The simpler `(?!a)b` does not trigger it (resharp is correct there);
 the negative
lookahead body must be a class or a complex sub-expression.

## Expected behaviour

`(?!\w)0+` is not nullable,
 because its second factor `0+` is not nullable,
 so
`is_match("")` is false.

## Root cause

The nullability computation for a concatenation that begins with a negative
lookahead over a predicate is wrong:
 resharp marks `(?!class) . R` as nullable
even when `R` is not nullable,
 so the pattern's `empty_nullable` flag is set and
`is_match` short-circuits to true on the empty input (and `find_all` emits the
zero-width `(0,0)` match to stay consistent with it).

## Distinct triggers

11 distinct patterns from the Lean differential,
 all of the shape
`(?!<class or sub-expr>) <non-nullable rest>`:

```text
(?!~(b{0}))[a-z]
~( )(?!\W+)[a-z]
(?!(?<= {0})\d{1,2})\D{1,2}
(?!\D)()*\D{2,2}
(?=(?!\W{1,2})\S{1,2})~(\s)
(?!(?= )\d)\W{2,3}
((?!\W{1,2})\d{2,3}&~(_))
(?!(?=[a-c]{1,2})\d*)[a-c]
(?!\D)(1&[a-c]+)
(?!((?<=a{0,2})\S|\d0))(?=\S*)(?!\w)\S
(?!\w)0+.{0,2}
```

## Notes

- This is the headline result of the Lean ground-truth oracle:
   a correctness bug
  that is self-consistent (is_match and find_all agree with each other) and so
  invisible to the INCONSIST,
   BOUNDS,
   HARDDIFF,
   and STREAM oracles.
   Only a
  trusted external semantics exposes it.
- The oracle compared 6185 non-anchor pairs and found exactly this one class of
  disagreement,
   which is also evidence that resharp's non-anchor is_match is
  otherwise correct on the sampled space.
