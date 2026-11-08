# BUG-10 default find_all drops a trailing zero-width match

## Classification

- Type:
   correctness,
   the default find_all path omits a match that the hardened
  path and the dotnet reference both report.
- Phase:
   match time,
   the default (non-hardened) optimised find_all path.
- Severity:
   soundness.
   This is the opposite side of BUG-8:
   there the hardened
  path was wrong,
   here the default path is wrong,
   so both optimised paths have
  distinct zero-width handling defects.

## Minimal reproducer

```rust
//! Reproduction of the dropped trailing zero-width match:
//! the default engine misses the empty match at the end of the input that the hardened engine returns.

/// The regex type,
/// its options and the match type that `find_all` returns.
use resharp::{Regex, RegexOptions, Match};
let pat = r"(?<=^)~(0+)";
let def = Regex::new(pat).unwrap();
let hard = Regex::with_options(pat, RegexOptions::default().hardened(true)).unwrap();
let hay = b"\n";
// default:  [(0,1)]            <- BUG, missing the (1,1) match at end of input
// hardened: [(0,1),(1,1)]
assert_eq!(def.find_all(hay).unwrap(), hard.find_all(hay).unwrap()); // FAILS
```

Command line:

```sh
repro '(?<=^)~(0+)' --sweep | grep HARDDIFF
```

## Observed behaviour

```text
HARDDIFF_FA|def=[(0, 1)]|hard=[(0, 1), (1, 1)]|hay=0a|pat="(?<=^)~(0+)"
```

The dotnet reference agrees with the hardened engine:
 `fa=0:1,1:1`.
 So the
default engine is the wrong side here,
 dropping the zero-width match at offset 1
(end of input).

## Expected behaviour

The default and hardened engines return identical find_all results,
 including
the zero-width match at end of input.

## Root cause

`resharp-engine/src/lib.rs` find_all dispatch.
 The default path
(`find_all_fwd_bounded` / `find_all_fwd_prefix` / `find_all_fwd_lb_prefix`)
fails to emit the end-of-input zero-width match for this lookbehind plus
complement shape,
 while the hardened `find_all_dfa` path emits it correctly.
 The
NO_MATCH sentinel work in BUG-2 and BUG-4 is in the same family of input-edge
handling.

## Notes

- BUG-8 and BUG-10 are distinct:
   BUG-8 has hardened wrong (`~(_a+)`),
   BUG-10 has
  default wrong (`(?<=^)~(0+)`).
   Together they show the two optimised find_all
  paths each mishandle a different zero-width or boundary case.
