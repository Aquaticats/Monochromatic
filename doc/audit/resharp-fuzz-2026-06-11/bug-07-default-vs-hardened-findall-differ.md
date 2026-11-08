# BUG-7 default and hardened find_all return different results

> Secondary class for the PATTERN (see `dotnet-adjudication.md`):
>  the dotnet
> reference rejects `~(\A|\n+){2}` ("anchors inside complement"),
>  so rust should
> reject it too.
>  But the BUG is a rust-internal self-inconsistency (default vs
> hardened `find_all` disagree on the same input),
>  demonstrable with no external
> reference -- hardening only swaps the scan algorithm,
>  not the language,
>  so one
> side is wrong by construction.

- Type:
   correctness,
   soundness.
   The two `find_all` algorithms disagree,
   so one is
  wrong (hardening swaps the scan algorithm,
   not the language).
- Phase:
   match time,
   the hardened forward scan versus the default driver.
- Severity:
   soundness.
   The hardened path drops a real match.
- Affected:
   default vs `hardened(true)`.
- Discovery:
   the HARDDIFF oracle over the adversarial corpus (4 distinct
  triggers).

## Reproducer

```rust
//! Reproduction of the difference between the default and the hardened engine on a complement of a repeated group:
//! the two match lists are in the comment.

/// The regex type and the option set that selects the hardened engine.
use resharp::{Regex, RegexOptions};
let pat = r"~(\A|\n+){2}";
let def  = Regex::new(pat).unwrap();
let hard = Regex::with_options(pat, RegexOptions::default().hardened(true)).unwrap();
assert_eq!(def.find_all(b"\n\n").unwrap(), hard.find_all(b"\n\n").unwrap());
// FAILS: default = [1:1, 2:2], hardened = [2:2]
```

Harness:

```sh
# 7e285c417c5c6e2b297b327d = "~(\A|\n+){2}", 0a0a = "\n\n"
repro --show 7e285c417c5c6e2b297b327d 0a0a 0   # fa=1:1,2:2
repro --show 7e285c417c5c6e2b297b327d 0a0a 4   # fa=2:2   (hardened)
```

## Observed versus expected

The default and hardened engines must accept the same language;
 hardening only
substitutes an O(N*S) forward scan to bound worst-case time.
 On `~(\A|\n+){2}`
over `"\n\n"` the default driver reports matches at `1:1` and `2:2` while the
hardened driver reports only `2:2`,
 dropping the `1:1` match.
 One side is wrong;
the disagreement alone is the bug.

Other distinct HARDDIFF triggers in the corpus:
 `^{3}([\w]{2,}0{3}|_?)`,
`~([a-c]+|\A{3}\s?)+`,
 and `[\x00-\x10]*(Z){2,}|(?!_{0}\A{3} {0,2}){3}`.
 All
involve a zero-width / anchor-laden sub-expression where the hardened scan and the
default driver disagree on a zero-width match near a line boundary.

The disagreement also reaches existence (`is_match`),
 the strongest form:
`1?a~(~((1?){2,}\z+){2}){2}` on `"a"` is `is_match = false` (default) versus
`is_match = true` with `find_all = [0:0]` (hardened).
 When the two engines
disagree on whether the input matches at all,
 they are accepting different
languages,
 which a scan-algorithm swap must never do.

## Relationship to 2026-06-04 BUG-8 / BUG-10

This is the BUG-8 / BUG-10 family (default vs hardened `find_all` disagreement),
reported fixed.
 The 06-04 triggers (`~(_a+)`,
 `(?<=^)~(0+)`) are now consistent
(verified),
 but the disagreement still occurs on new triggers,
 so the underlying
two-driver divergence is not eliminated.

## Source pointer

The hardened driver is the `find_all_dfa` path;
 the default driver is the
reverse-collect / prefix path.
 The divergence is in how each handles a zero-width
match adjacent to a `\n` line boundary (the same theme as arm-bug-01 and bug-03,
but a third distinct driver).
