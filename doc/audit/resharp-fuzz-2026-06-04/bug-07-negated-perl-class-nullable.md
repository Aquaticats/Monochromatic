# BUG-7 negated perl classes match the empty string

## Classification

- Type:
   correctness,
   wrong nullability.
- Phase:
   compile time nullability computation,
   surfaced at match time.
- Severity:
   soundness,
   and the single largest cluster in this campaign.
   Any
  pattern containing `\D`,
   `\S`,
   or `\W` can match where it should not.

## Minimal reproducer

```rust
//! Reproduction of the nullable negated class:
//! in ASCII mode,
//! `\D` matches the empty input.

/// The regex type,
/// its options and the Unicode mode that selects ASCII classes.
use resharp::{Regex, RegexOptions, UnicodeMode};
let re = Regex::with_options(r"\D", RegexOptions::default().unicode(UnicodeMode::Ascii)).unwrap();
assert_eq!(re.is_match(b"").unwrap(), false); // FAILS: resharp returns true
```

Command line:

```sh
repro '\D' --sweep | grep DIVERGE
# DIVERGE|ascii|rs=true|rx=false|hay=|pat="\\D"
```

`\S` and `\W` behave identically.
 The positive classes `\d`,
 `\s`,
 `\w`,
 the
dot `.`,
 and negated character classes `[^0]`,
 `[^a]` are all correct.

## Observed behaviour

`\D`,
 `\S`,
 `\W` each report `is_match("") == true`.
 On the empty input
`is_match` returns the pattern's `empty_nullable` flag,
 so resharp has computed
these single-byte negated classes as nullable.
 Downstream,
 any pattern that
suffixes one of them inherits the wrong nullability:

```sh
repro 'a*\D' --sweep | grep DIVERGE
# rs=true rx=false on hay="" and hay="0" and hay="1"
```

`a*\D` should require at least one non-digit byte,
 so it must not match `""` or
`"0"`.
 resharp matches both.

## Expected behaviour

`\D`,
 `\S`,
 `\W` each consume exactly one byte and are not nullable,
 matching the
`regex` crate with `.unicode(false)` byte-for-byte on pure-ascii input.

## Root cause

The negation is being lowered as a language complement rather than a byte-class
negation,
 so the empty string (which is not a single `\w`,
 `\s`,
 or `\d`
character) falls inside the negated language and makes the node nullable.

The correct byte-negation lowering exists at
`resharp-parser/src/lib.rs:205` (`ascii_perl_set_item`),
 which wraps the
positive ascii class in a negated bracketed class,
 but that path is only taken
when `ascii_perl_classes` is set,
 and `with_options`
(`resharp-engine/src/lib.rs:921`) only sets that flag for
`UnicodeMode::Javascript`.
 Ascii mode (`unicode = false`,
`ascii_perl_classes = false`) reaches a different lowering that produces the
nullable result.

The exact node-construction line is now pinned.
 `perl_class_node`
(`resharp-parser/src/lib.rs:1276`) has three branches:
 `global_ascii_perl` (js)
negates with `resharp_algebra::neg_class(tb, pos)` (`:1309`,
 correct);
`global_unicode` (default/full) returns precomputed `non_word`/`non_digit`/
`non_space` predicates (`:1322`/`:1334`/`:1346`,
 correct);
 and the final `else`
(ascii,
 both flags false) negates with `tb.mk_compl(pos)` (`:1373`),
 the regex
language complement operator `~`,
 instead of `neg_class`.
 `~(\w)` is the set of
strings that are not a single word char,
 which includes the empty string and
every non-word substring,
 so `is_match` is nullable and true everywhere.
 The
one-line fix is to use `resharp_algebra::neg_class(tb, pos)` at `:1373`,
 matching
the js branch at `:1309`.

## Impact on the campaign

This single defect explains roughly all 108 distinct `DIVERGE` patterns found in
the first 80k-pattern sweep after filtering to pure-ascii haystacks and removing
anchors and resharp-only operators.
 Any generated pattern containing `\D`,
 `\S`,
or `\W` diverged from the `regex` crate for this reason.

## Notes

- Scope confirmed by direct cross-config testing:
   only the ascii config is wrong.
  `\W`/`\D`/`\S` give the correct `false` for `is_match("a")`,
   `is_match("1")`,
  `is_match(" ")`,
   and `is_match("")` in the default,
   full,
   and js configs;
   only
  ascii (`UnicodeMode::Ascii`) returns the spurious `true`.
   The bug is the
  `mk_compl`-vs-`neg_class` branch,
   not a mode-independent nullability error.
- Scope confirmed by the bracketed form:
   `[\W]`,
   `[\D]`,
   `[\S]`,
   and `[^\w]` are all
  correct in ascii (they route through the `rewrite_ascii_perl` / `neg_class` path),
  so the defect is limited to the bare shorthand `\W`/`\D`/`\S`.
- The earlier draft filed this as a separate bug (BUG-24) before its overlap with
  BUG-7 was noticed;
   BUG-24 has been merged here and removed.
   The pinned root cause
  and the cross-config / bracketed scope above are what that round added.
- The in-tree `diff_regex` target deliberately excludes the perl classes,
   which
  is why coverage-guided fuzzing alone did not surface this;
   the directed oracle
  that includes them did.
