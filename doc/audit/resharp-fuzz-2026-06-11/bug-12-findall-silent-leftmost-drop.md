# bug-12: find_all silently drops the leftmost match (no panic, debug and release)

> Secondary class (see `dotnet-adjudication.md`):
>  the dotnet reference REJECTS
> this pattern at compile ("lookarounds inside union not supported").
>  rust's
> `ensure_supported_rec` guard accepts it,
>  then returns a wrong span.
>  This is an
> implementation divergence:
>  rust should reject the pattern (as the reference
> does) or return the correct span.
>  The Lean value below corroborates that rust's
> result is wrong;
>  it is not relied on as the sole authority.

Severity:
 soundness (silent,
 both debug and release).
 Found by the Lean
position-level differential (seed-1001 run,
 case R2280,
 trust0,
 anchor-free,
translator-faithful).
 Confirmed on the unmodified v0.6.12 stock crate and against
the Lean ground truth.
 Closely related to bug-11 (same `find_all`
reverse/forward null subsystem) but a distinct manifestation:
 bug-11 panics in
debug builds;
 this returns a wrong answer with NO assertion,
 even under
`debug-assertions`,
 so a consumer's test suite sees no crash.

Architecture:
 confirmed byte-identical on aarch64 (Apple M1) and x86-64.
`armprobe "((?!b)|ba)&(aa)?" "ab"` on the M1 returns `find_all=[(2,2)]` (Lean
ground truth is `0:0`;
 rust silently drops the leftmost match),
 the same wrong
result as x86.
 The ARM run makes "ARM-confirmed" demonstrated rather than
inferred.

## Minimal reproducer

```rust
//! Reproduction of the silently dropped leftmost match:
//! three inputs of the same pattern lose their first matches,
//! with the wrong and the expected results in the comments.
// stock resharp v0.6.12, default config
/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};

let re = Regex::with_options(r"((?!b)|ba)&(aa)?", RegexOptions::default()).unwrap();
re.find_all(b"ab");   // -> [(2,2)]   WRONG: drops the leftmost match (0,0)
re.find_all(b"abab"); // -> [(4,4)]   WRONG: drops (0,0) and (2,2)
re.find_all(b"aba");  // -> [(3,3)]   WRONG: drops (0,0) and (2,2)
```

Identical in debug-assertions and release builds (no panic in either).
`is_match` is unaffected (`true`),
 since some match still exists.

## Ground truth (Lean `llmatch`, leftmost-longest first span)

```txt
hay     Lean first span   rust find_all     rust first span
"ab"    0:0               [(2,2)]           2:2   (wrong)
"abab"  0:0               [(4,4)]           4:4   (wrong)
"aba"   0:0               [(3,3)]           3:3   (wrong)
```

The pattern is `((?!b) | ba) & (aa)?`.
 `(?!b)` is a zero-width assertion that
holds wherever the next byte is not `b` (or at end),
 so the empty span `[s,s]`
matches `((?!b)|ba)` whenever `text[s] != 'b'`;
 intersecting with `(aa)?` (which
contains the empty string) keeps every such empty span.
 On `"ab"` the leftmost
match is the empty span `0:0` (byte 0 is `a`).
 The engine instead returns only
the last match (`2:2`),
 dropping `0:0` entirely.

## Root cause and relationship to bug-11

`find_all` (`find_all_dfa`) collects candidate match-START positions in a reverse
pass and confirms each with a forward scan in `scan_fwd_all` (`ldfa.rs`).
 bug-11
is a FORWARD-pass fault:
 the reverse pass proposes a legitimate start (a match
really exists there),
 and the forward scan fails to confirm it,
 returning
`NO_MATCH` and tripping `debug_assert_ne!(NO_MATCH, l_max_end)` at
`ldfa.rs:833/887/906` (panic in debug,
 drop in release).

bug-12 is a REVERSE-pass fault:
 for `(nullable-alternation) & (nullable
right-operand)` the reverse pass itself UNDER-collects,
 never proposing the
leftmost null starts,
 so `scan_fwd_all` is never asked about them,
 no assert
fires,
 and `find_all` silently omits them.
 Different passes are at fault,
 not the
same pass in two directions.

This was verified,
 not inferred,
 by printing the `nulls` slice that
`scan_fwd_all` receives (a one-line instrument in the engine copy) and confirming
each disputed start against Lean,
 on `"abab"`:

```txt
((?!b)|ba)&(aa)?   nulls = [4]        leftmost starts 0,2 ABSENT; Lean says 0:0 is a match -> reverse fault (silent drop)
((?!b)|ba)&(aa)*   nulls = [4, 2, 0]  0 IS proposed; Lean says 0:0 is a match; forward returns NO_MATCH for it -> forward fault (panic)
((?!a)|b)&(~((c))) nulls = [4, 2, 1]  reverse proposes 1/2; the dropped 2:2 is a real match; forward refuses it -> forward fault (panic)
```

The discriminator is which pass mishandles a start that is GENUINELY a match
(Lean-confirmed):
 for `(aa)?` the reverse pass never offers offsets 0/2 at all;
for `(aa)*` the reverse pass offers 0 and the forward pass refuses it.
 Same
`((?!b)|ba)&X` skeleton,
 identical match set,
 but `X = (aa)?` breaks the reverse
pass while `X = (aa)*` breaks the forward pass.
 A fix to the forward derivative
(bug-11) would not make the reverse pass start proposing the offsets it omits
(bug-12),
 and vice versa:

```txt
((?!b)|ba)&(aa)?   -> [(4,4)]                       bug-12 (reverse omits 0,2)
((?!b)|ba)&(aa)*   -> panic at ldfa.rs:837          bug-11 (forward refuses proposed 0)
((?!a)|b)&(~((c))) -> panic at ldfa.rs:906          bug-11 (forward refuses proposed start)
```

They are counted as distinct because the `nulls` evidence above shows two faults
in two different passes,
 not one fault seen twice,
 and the observable failure and
severity differ:
 bug-11 is a loud crash in any debug/test build (plus a release
drop);
 bug-12 is a silent wrong `find_all` in EVERY build,
 which no panic ever
surfaces.
 Both live in the reverse-null-collection / forward-confirmation
subsystem,
 so a single rewrite of that subsystem could plausibly fix both at
once;
 if the maintainer confirms one fix
covers both,
 treat them as one.
 The split is deliberately conservative about the
shared subsystem while preserving the distinct silent-soundness fault,
 which is
materially worse for consumers than a crash they would notice.

## Provenance

Lean lane,
 seed-1001 run:
 R2280
(`((((((?!b))){2,2}|(ba))|(((_){0,}|\w)&((\Da)&b)))&((~((~((aa)))))){0,1})` on
`"abab"`,
 lean `0:0` vs rust `4:4`).
 Minimized to `((?!b)|ba)&(aa)?` (the
`_*`/`\w`/`\D`/empty-intersection scaffolding was incidental;
 the empty span
`(aa)?` right-operand and the zero-width alternation branch are the essentials).
The internal oracles miss it for the same reason as bug-11's release half:
 the
result is internally self-consistent (find_all sorted,
 non-overlapping,
non-empty,
 agreeing with is_match),
 just positionally wrong,
 so only an external
position reference exposes it.
