# BUG-8 is_match returns false while find_all returns a match

> Secondary class for the PATTERN (see `dotnet-adjudication.md`):
>  the dotnet
> reference rejects this lookaround-in-union / anchor-in-complement pattern,
>  so
> rust should reject it too.
>  But the BUG is a rust-internal self-inconsistency
> (`is_match=false` while `find_all` returns a match),
>  demonstrable with no
> external reference -- the engine flatly contradicts itself.

- Type:
   correctness,
   soundness.
   `is_match` and `find_all` disagree on
  match existence.
- Phase:
   match time.
- Severity:
   soundness.
   A caller using `is_match` as a gate before `find_all`
  skips a haystack that does contain a match.
- Affected:
   the flags config (`case_insensitive + ignore_whitespace +
  dot_matches_new_line + multiline=false`) on the reproducer below;
   the INCONSIST
  oracle found one distinct trigger in the 40k corpus.
- Discovery:
   the INCONSIST oracle (`is_match` xor `find_all` non-emptiness).

## Reproducer

```rust
//! Reproduction of the disagreement between `is_match` and `find_all` under a full set of options.

/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};
let opts = RegexOptions::default()
    .case_insensitive(true)
    .ignore_whitespace(true)
    .dot_matches_new_line(true)
    .multiline(false);
let re = Regex::with_options(r"[0-9]{2}~(\z{1,3}|^{2}\W{0})+", opts).unwrap();
let hay = b"00";
assert_eq!(re.is_match(hay).unwrap(), false);        // says no match
assert_eq!(re.find_all(hay).unwrap().len(), 0);      // FAILS: find_all = [0:2]
```

Harness:

```sh
# pat hex below = "[0-9]{2}~(\z{1,3}|^{2}\W{0})+", 3030 = "00", cfg 5 = flags
repro --show 5b302d395d7b327d7e285c7a7b312c337d7c5e7b327d5c577b307d292b 3030 5
# compile=ok|im=Ok(false)|fa=0:2|fan=Ok(None)|stream=
```

## Observed versus expected

`is_match` must equal `!find_all.is_empty()`:
 the same engine cannot both deny a
match and return one.
 resharp returns `is_match = false` and `find_all = [0:2]`
on `"00"`.
 The `is_match` short-circuit path disagrees with the full `find_all`
scan for this complement-plus-anchor pattern under the flags config;
 `find_all`'s
`0:2` span (the whole input) is the candidate truth,
 but the contradiction itself
is the bug regardless of which side is correct.

## Relationship to 2026-06-04 BUG-3

This is the BUG-3 family (`is_match` vs `find_all` disagreement),
 reported fixed.
The 06-04 triggers (`(\z|(?=a)\w)`,
 `\BU`) are now self-consistent (verified;
`\BU` instead exposes bug-02 in `find_anchored`),
 but the `is_match` /`find_all`
divergence still occurs on a complement-plus-anchor pattern under the flags
config,
 so the underlying defect is narrowed,
 not eliminated.

## Source pointer

The `is_match` fast path versus the `find_all` reverse-collect path
(`resharp-engine/src/lib.rs`,
 `is_match` at `:1931`,
 `find_all` at `:1332`).
 The
flags config (multiline off,
 dot-matches-newline,
 ignore-whitespace) reshapes the
anchor lowering;
 the two paths lower or evaluate the leading `[0-9]{2}` plus the
nullable complement tail differently.
