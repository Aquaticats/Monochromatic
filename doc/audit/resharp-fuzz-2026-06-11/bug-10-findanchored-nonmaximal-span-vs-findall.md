# BUG-10 find_anchored returns a shorter-than-longest span, disagreeing with find_all

> Secondary class for the PATTERN (see `dotnet-adjudication.md`):
>  the dotnet
> reference rejects `~(.{1,3}\z){2,4}` ("anchors inside complement"),
>  so rust
> should reject it too.
>  But the BUG is a rust-internal self-inconsistency
> (`find_anchored` returns a strictly shorter span than `find_all`'s longest match
> at the same offset),
>  demonstrable with no external reference.

- Type:
   correctness,
   soundness.
   `find_anchored` and `find_all` disagree on the
  span of the match at offset 0.
- Phase:
   match time,
   the `find_anchored` longest-end computation versus the
  `find_all` longest-end.
- Severity:
   soundness.
   `find_anchored` is documented as the longest match
  anchored at offset 0,
   which must equal `find_all`'s first span when that span
  starts at 0.
   It returns a strictly shorter span.
- Affected:
   all configs (config-independent).
- Discovery:
   the FANSPANDIFF oracle (`find_anchored` end !
  = `find_all` first-span
  end at offset 0) over the 79k combined corpus:
   30 distinct triggers.

## Reproducer

```rust
//! Reproduction of the non-maximal anchored span:
//! `find_anchored` returns a shorter span than the one `find_all` returns at the same start.

/// The regex type whose two entry points disagree.
use resharp::Regex;
let re = Regex::new(r"~(.{1,3}\z){2,4}").unwrap();
let fa  = re.find_all(b"ab").unwrap();        // [Match{0,2}, Match{2,2}]
let fan = re.find_anchored(b"ab").unwrap();   // Some(Match{0,1})  -- WRONG
// find_all's first match is 0:2 (the longest at offset 0); find_anchored must
// return the same span, but returns the shorter 0:1.
assert_eq!(fan, Some(fa[0]));                 // FAILS: Some(0:1) != Some(0:2)
```

Harness:

```sh
# 7e282e7b312c337d5c7a297b322c347d = "~(.{1,3}\z){2,4}", 6162 = "ab"
repro --show 7e282e7b312c337d5c7a297b322c347d 6162 0
# compile=ok|im=Ok(true)|fa=0:2,2:2|fan=Ok(Some((0, 1)))|stream=0:0,2:2
```

## Observed versus expected

`find_all` returns `[0:2, 2:2]`,
 so the leftmost-longest match starting at offset
0 is `0:2`.
 `find_anchored` (the longest match anchored at 0) must return the
same `0:2`.
 It returns `0:1`,
 a strictly shorter span.
 The two public APIs of the
same engine contradict each other on the longest-end at offset 0;
 one is wrong.
`find_anchored` returning the shorter span points to its longest-end scan
stopping at an earlier accepting position than `find_all`'s driver does for this
complement-with-end-anchor shape.

## Scope

30 distinct triggers,
 all complement patterns containing a trailing end anchor:
`~(.{1,3}\z){2,4}`,
 `~(a_{0}(\z){2})+`,
 `~(\W{0,2}\z{2,})?`,
 `~([Z-a]*[^\w]+\z+)`,
`\A?~([\w]+\S?${2,})+`,
 and similar `~(...\z...)` / `~(...$...)` shapes.
 The end
difference ranges from `fan_end=0 fa_end=1` up to `fan_end=14 fa_end=15` across
the family.
 Config-independent.

## Relationship to other findings

This is the 06-04 BUG-13 / BUG-14 family (`find_anchored` versus `find_all` span
disagreement:
 the lookahead width leak and the dropped lookbehind gate),
 reported
fixed.
 The 06-04 triggers (`(?=(?=c)c{1,3})`,
 `(|(?<=[a-z])b)`) are now correct
(verified),
 but the span disagreement is live again on the complement-with-end-
anchor family.
 Distinct from bug-02:
 there `find_anchored` returns a match where
none exists (`is_match = false`);
 here a match does exist and `find_all` agrees
on its start,
 but the two disagree on its length.
 Same `find_anchored` path
(`resharp-engine/src/lib.rs:1891`),
 different defect (the longest-end value
rather than the existence gate).
