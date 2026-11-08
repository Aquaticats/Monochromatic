# BUG-26 \z\A is reduced to the empty language and fails to match the empty string

## Classification

- Type:
   correctness,
   soundness (false negative).
   A pattern whose language is `{""}`
  is compiled to the empty language,
   so it never matches.
- Phase:
   compile (node construction,
   `mk_concat`).
- Severity:
   low-to-moderate.
   The trigger is narrow (an end-anchor immediately before a
  begin-anchored,
   begin-nullable tail),
   but it is a genuine soundness failure:
   the
  empty string,
   the one input the pattern matches,
   is rejected.
   The same reduction
  silently drops the empty-string branch from any larger pattern that contains the
  shape.

## Minimal reproducer

```rust
//! Reproduction of the end anchor followed by the begin anchor:
//! on the empty input the pattern must match,
//! and the reverse order,
//! which is correct,
//! is the control.

/// The regex type whose `is_match` answers wrongly.
use resharp::Regex;
let re = Regex::new(r"\z\A").unwrap();
assert_eq!(re.is_match(b"").unwrap(), true);   // FAILS: returns false
// \A\z (the reverse order) is correct:
assert_eq!(Regex::new(r"\A\z").unwrap().is_match(b"").unwrap(), true); // ok
```

`\z` (end of input) and `\A` (start of input) are both zero-width and both hold at the
single position of the empty string (position 0 is simultaneously start and end),
 so
`\z\A` matches exactly the empty string.

## Observed behaviour

`is_match("")` by config:

```text
\z\A    default=false  ascii=false  full=false     (WRONG, expected true)
\A\z    default=true   ascii=true   full=true       (correct)
\za*\A  default=false  ascii=false  full=false     (WRONG, a* is nullable so = \z\A)
find_all("") for \z\A = []   (expected [(0,0)])
```

Config-independent.
 The variants `\za*\A`,
 `\za?\A`,
 `\za{0}\A`,
 and `(\z)(\A)` all
reproduce;
 `\za\A` correctly does not match (the `a` requires a character).
 Surfaced
by the differential oracle `repro --divergebatch` (resharp ascii vs the `regex`
crate,
 which matches `\z\A` on the empty string).

## Expected behaviour

`\z\A` matches the empty string and nothing else:
 `is_match("")` is `true`,
`find_all("")` is `[(0,0)]`,
 and `is_match` on any non-empty input is `false`.

## Root cause

`mk_concat` (`resharp-algebra/src/lib.rs:3232`) has a rule that reduces a concat to the
empty language when the head is an end-anchor and the tail cannot be nullable at the
end:

```rust
//! The concatenation rule that reduces an end anchor followed by anything not nullable at the end to the empty language,
//! which is wrong when the tail is the begin anchor on an empty input.
if self.get_kind(head) == Kind::End
    && !tail.is_lookbehind(self)
    && !self.is_nullable(tail, Nullability::END)
{
    return NodeId::BOT;
}
```

For `\z\A` = `Concat(End, Begin)`:
 head is `End`;
 the tail `\A` (`Begin`) is not a
lookbehind;
 and `\A` is nullable under the BEGIN context but not under END,
 so
`is_nullable(tail, END)` is false and the rule returns `NodeId::BOT`.
 The compiled
forward node is `⊥` (confirmed via the `debug` trace:
 `[fwd]: ⊥` for `\z\A` versus
`[fwd]: \z\A` for `\A\z`),
 so `is_empty_lang` is set (`lib.rs:990`) and `is_match`
short-circuits to `false` (`lib.rs:1862`).

The rule's premise is that nothing can follow an end-anchor,
 so the tail must match the
empty string at end-of-input (END-nullable) for the concat to be satisfiable.
 That
misses the empty-input case:
 when the input is empty,
 the end position is also the
begin position,
 so a tail that is nullable under BEGIN (here `\A`) is satisfied there.
The reduction therefore discards the empty-string solution.
 The condition should test
nullability under the combined empty-string context (`Nullability::EMPTYSTRING`,
 i.e.
BEGIN and END) rather than END alone,
 so `End` before a begin-nullable tail is not
collapsed to BOT.
 The reverse order `\A\z` is unaffected because the head is `Begin`,
not `End`,
 and never enters this rule.

## Affected configurations

All (default,
 ascii,
 full,
 js).
 The rule is in shared node construction and does not
depend on the unicode mode.
 The limits-disabling config is irrelevant (correctness,
not limits).

## Relationship to other findings

- A separate begin/end-anchor handling defect from BUG-20 (find_anchored ignoring a
  leading begin assertion) and BUG-21 (begin-context cache contamination):
   those are
  match-time scan/cache issues;
   this is a compile-time node simplification that drops
  the empty-string branch.
   All three share the theme that the begin-of-input boundary
  is mishandled relative to the end boundary.

## Code quality

The rule encodes "an end-anchor makes a non-END-nullable tail unsatisfiable,
" which is
true for non-empty input but false at the empty string,
 where end and begin coincide.
Testing the combined empty-string context (or special-casing a begin-nullable tail)
both fixes the bug and states the invariant the rule actually wants.
