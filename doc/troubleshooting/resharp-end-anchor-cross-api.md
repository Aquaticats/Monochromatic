# resharp 0.6.13: find_all, find_anchored, and is_match wrong on end-anchor shapes

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Status note (forbidden-strings 0.2.0):
 the `resharp` dependency is gone.
 The engine swap replaced resharp with the in-house `forbidden-regex` engine
(`package/forbidden-strings/core`),
 so the scanner no longer exercises the resharp APIs described here.
 This document stays as a durable record of the upstream bugs found and filed.

Three soundness bugs in `ieviev/resharp` 0.6.13 (== repo HEAD `f0ce60a`,
behaviorally identical to the published 0.6.13 commit `d89964b`),
 all on the
accepted-superset end-anchor zone,
 all tracing to anchors dropped by forward
simplification.
 Bugs 1 (`find_anchored`) and 2 (`is_match`) disagree with a
correct `find_all`;
 bug 3 is in `find_all` ITSELF (a false negative on
intersection-with-end-anchor).
 Found by the 2026-06-19 fuzz campaign
(`doc/audit/resharp-fuzz-2026-06-19/`);
 bug 3 was found after extending the
denotational oracle to anchors.
 This doc adds the root-cause traces,
 the
upstream-filing check,
 and the prototypes (complete for bugs 1+2;
 bug 3's minimal
routing prototype is shown insufficient,
 the real fix being algebra-deep).

## Symptom

Bug 1,
 `find_anchored` phantom/missing span.
 Pattern `(\z|$)$` (a union of end
anchors concatenated with an end anchor),
 `UnicodeMode::Ascii` and `Default`:

```text
/(\z|$)$/  "\n"  : find_all=[(0,0),(1,1)]  find_anchored=Some(0,1)  WRONG (phantom width-1)
/(\z|$)$/  "\na" : find_all=[(0,0),(2,2)]  find_anchored=None       WRONG (missing match at 0)
```

The pattern is zero-width only,
 so `Some(0,1)` is an impossible span and `None`
drops the real `(0,0)` that `find_all` reports.

Bug 2,
 `is_match` false positive.
 Pattern `_&(?:[ab]|$)?` (any-byte intersected
with an optional union of a class and the end anchor),
 on `"\n"`:

```text
/_&(?:[ab]|$)?/  "\n" : is_match=true  find_all=[]   WRONG (false positive)
```

`_` forces width 1;
 `([ab]|$)?` matches width 1 only via `[ab]`;
 `\n` is not in
`[ab]`,
 so the language is empty on `"\n"`.
 `find_all=[]` is correct;
 `is_match`
over-accepts.

Bug 3,
 `find_all` FALSE NEGATIVE (most severe;
 in the production API).
 Pattern
`.&a(?:$|b)` (any-byte intersected with `a` then end-anchor-or-`b`),
 on `"a\n"`:

```text
/.&a(?:$|b)/  "a\n" : find_all=[]  is_match=false   WRONG (drops a real match)
/./           "a\n" : find_all=[(0,1)]              (left operand matches (0,1))
/a(?:$|b)/    "a\n" : find_all=[(0,1)]              (right operand matches (0,1))
/.&a$/        "a\n" : find_all=[(0,1)]              (subset of the trigger; matches)
```

Both operands individually match span `(0,1)` per resharp's OWN `find_all`,
 and
the subset `.&a$` matches `(0,1)`,
 but the intersection `.&a(?:$|b)` returns `[]`.
An intersection cannot drop a span both operands contain,
 and a superset cannot
match less than its subset:
 a `find_all` false negative (fail-open direction).
Arch-identical (AVX2,
 NEON),
 all four unicode modes.
 Proven from resharp's own
outputs;
 no external oracle needed.

Both violate `api.md`'s asserted contract:
 `is_match` is true exactly when
`find_all` is non-empty,
 and `find_anchored=Some(m)` means `m` is the longest
`find_all` match at offset 0.
 Both are arch-independent (identical on x86_64 AVX2
and Apple M1 NEON).
 They are the 2026-06-11 bug-02 / bug-08 / bug-10 families,
narrowed across versions but not eliminated.

## Root cause

Both bugs route a non-lookbehind anchor pattern through the forward optional scan
instead of the authoritative `find_all` enumeration.

`find_anchored` (`resharp-engine/src/lib.rs:1899-1909`):

```rust
pub fn find_anchored(&self, input: &[u8]) -> Result<Option<Match>, Error> {
    if input.is_empty() { return Ok(self.empty_input_match()); }
    let inner = &mut *self.inner.lock().unwrap_or_else(|e| e.into_inner());
    if self.has_lb {
        return Err(Error::Algebra(resharp_algebra::ResharpError::UnsupportedPattern))
    }
    Ok(inner.fwd.scan_fwd_optional(&mut inner.b, 0, input)?.map(|end| Match { start: 0, end }))
}
```

For a non-lookbehind pattern it delegates to `scan_fwd_optional(b, 0, input)`,
which for the `(\z|$)$` derivative structure returns `Some(1)` (a width leak past
the satisfied zero-width assertion) or `None` (start-nullability not detected).

`is_match` (`resharp-engine/src/ismatch.rs:30-50`) dispatches the
`Bounded`/`FwdPrefix`/`FwdLbPrefix` kinds to `is_match_fwd_ts`
(`ismatch.rs:19-25`),
 which calls the same `scan_fwd_optional`:

```rust
pub(crate) fn is_match_fwd_ts(&self, input: &[u8]) -> Result<bool, Error> {
    let inner = &mut *self.inner.lock().unwrap_or_else(|e| e.into_inner());
    Ok(inner.fwd_ts.scan_fwd_optional(&mut inner.b, 0, input)?.is_some())
}
```

The deeper sub-cause for `is_match` on `_&(?:[ab]|$)?`:
 the `has_anchors` guard
that would route anchor patterns away from the fast path is computed on the
forward-SIMPLIFIED node,
 not the original:

```rust
// resharp-engine/src/lib.rs:1035
let node_fwd_simpl = b.simplify_fwd_initial(node);
// resharp-engine/src/lib.rs:1177
let has_anchors = b.contains_anchors(node_fwd_simpl);
```

`simplify_fwd_initial` proves the `$` branch dead under the width-1 `_`
intersection and drops it,
 so `b.contains_anchors(node_fwd_simpl)` is `false` for
`_&(?:[ab]|$)?` even though `$` alone and `(\z|$)$` both report `true` (confirmed
with a `dbg_flags` probe:
 `has_anchors=false kind=FwdPrefix` for
`_&(?:[ab]|$)?`).
 With `has_anchors` false the fast `scan_fwd_optional` path is
taken and over-accepts.
 So the anchor-routing flag and the runtime path disagree.

Wrong earlier hypothesis worth recording:
 the first prototype gated the
defer-to-`find_all` fix on `has_anchors`.
 That fixes `find_anchored` and the
`is_match` cases where `has_anchors` is correctly set (e.g. `_&(?:[ab](?!\A)|$)?`,
which has a `\A`),
 but it does NOT fix the minimal `_&(?:[ab]|$)?`,
 because the
gate is the very flag that is wrong.
 The fix must use anchor presence in the
ORIGINAL node,
 not the simplified one.

Bug 3 (`find_all` false negative) is deeper.
 The forward matcher is built from
`node_fwd_simpl = simplify_fwd_initial(node)` (`lib.rs:1035`);
 for `.&a(?:$|b)` the
simplification drops the `$` alternative,
 so the forward automaton itself cannot
reach the `$`-branch end.
 The Dfa find_all loop has an internal tripwire for this
(`resharp-engine/src/ldfa.rs:842-846`):

```rust
debug_assert_ne!(
    NO_MATCH, l_max_end,
    "find_all: forward scan found no end for reverse-proposed start 0"
);
if l_max_end != NO_MATCH { matches.push(Match { start: 0, ... }) }
```

In release (`debug_assert` off) the `if` simply skips the push,
 so `find_all`
returns `[]`.
 Anchor dropping also mis-classifies the kind to `FwdPrefix`
(`.&a$` is `Dfa`,
 `.&a(?:$|b)` is `FwdPrefix`),
 but that is secondary:
 forcing it
to `Dfa` does NOT fix `find_all` (it trips the assertion above).
 So bug 3's locus
is `simplify_fwd_initial` / the intersection-with-end-anchor-alternation forward
derivative,
 an algebra-core fix,
 not a routing change.

## Verification

Version under test:
 resharp 0.6.13,
 repo HEAD `f0ce60a` (`origin`
`https://github.com/ieviev/resharp.git`).
 Harness:
 the campaign's
`anchored_probe`,
 `trig`,
 `c1probe`,
 `c1min` binaries
(`/tmp/agent/resharp-denot-oracle`),
 each compiling `resharp = "=0.6.13"` and
calling the APIs directly.

Patterns that fail (find_anchored):
 `(\z|$)$`,
 `($|\z)$`,
 `(\z|$)\z`,
 `(\z|$)$$`.
Patterns that are correct (find_anchored):
 `($)$`,
 `(\z|$)`,
 `(\z|$)[a]`,
`((?=\z)|$)$`,
 `\z$`.
 Compile-rejected (not mis-answered):
 `(a|$)$`,
 `(\b|$)$`.

Patterns that fail (is_match):
 `_&(?:[ab]|$)?`,
 `_&(?:[ab](?!\A)|$)?`,
`_&(?:[a](?!\A)|$)?`,
 `_&(?:(?:_&[ab]?)(?!\A)|$)?`.
 Correct:
 `_&(?:_(?!\A)|$)?`,
`_&(?:[ab](?!\A)|\z)?` (uses `\z` not `$`),
 `[ab]&(?:[ab](?!\A)|$)?` (left operand
a class,
 not `_`).
 Full catalogs:
`doc/audit/resharp-fuzz-2026-06-19/bug-find-anchored-end-anchor-union.md` and
`bug-is-match-false-positive-inter-optional-end-anchor.md`.

## Verified workarounds

Consumer-side,
 at our boundary:
 use `find_all` and derive the needed answer from
it,
 never `find_anchored`/`is_match` directly,
 for anchor-bearing patterns.

```rust
// instead of re.is_match(input)
let any = !re.find_all(input)?.is_empty();
// instead of re.find_anchored(input)
let at0 = re.find_all(input)?.into_iter().next().filter(|m| m.start == 0);
```

Tradeoff:
 `find_all` enumerates all matches,
 so it is slower than the
single-forward-scan `is_match`/`find_anchored` fast paths;
 for anchor-free
patterns the fast paths are correct and cheaper,
 so only route anchor-bearing
patterns through `find_all`.
 This workaround addresses bugs 1 and 2 only.
 For bug
3,
 `find_all` itself is wrong,
 so there is NO consumer-side workaround on the
crate API short of avoiding intersection-with-end-anchor patterns.
 The
`forbidden-strings` scanner uses `find_all` and its rule set has no
intersection-with-anchor patterns,
 so it is unaffected by all three.

## What does not work

- Gating the defer-to-`find_all` fix on `self.has_anchors`.
   `has_anchors` is
  computed on the forward-simplified node and is `false` for `_&(?:[ab]|$)?`,
   so
  the gate misses the minimal case.
   Use anchor presence in the original node.
- Rejecting these at `find_anchored` like the lookbehind case.
   `find_anchored`
  already returns `UnsupportedPattern` for `^`-bearing patterns (which lower to a
  lookbehind),
   but `$`/`\z` lower to lookahead/END,
   so they are accepted and then
  mis-answered.
   Rejecting all anchor patterns would be a larger behavior change
  than restoring correctness via `find_all`.

## Upstream filing decision

All six constraints hold;
 the matching upstream issue is the open #22,
 so the
artifact is an additive comment,
 not a new issue.

1. **Upstream's fault?
   ** Yes.
    For bugs 1+2,
    two APIs contradict `find_all` and the
   `api.md` contract.
    For bug 3,
    `find_all` contradicts itself (both operands match
   `(0,1)`,
    the intersection does not).
    Behavior bugs,
    not wording,
    not
   architectural restrictions.
2. **Can upstream fix it?
   ** Yes.
    Bugs 1+2 prototyped below (pass the full test
   suite).
    Bug 3's minimal routing prototype is insufficient (it trips an internal
   Dfa assertion);
    the fix is in the forward-derivative algebra,
    larger but not
   impossible (issue #22's driver/representation unification).
    Constraint 2 is
   about possibility,
    which holds.
3. **Supporting this use case?
   ** Yes.
    `find_all`/`find_anchored`/`is_match` are
   documented production APIs and the cross-API agreement is asserted intent the
   maintainer is converging on (issue #22).
4. **Would the repo welcome it?
   ** Yes.
    No `CONTRIBUTING.md`,
    issue/PR template,
    or
   AI-assistance ban exists (the absent `.github/` policy was checked for the
   06-04 filing;
    unchanged),
    and ieviev explicitly invited more findings (issue
   #17:
    "If you find some more you can reopen or add a new issue";
    issue #21:
   "excellent job").
    No ban found.
5. **Likely to fix?
   ** Yes.
    ieviev fixed the bug-02/08/10 family across 0.6.9 to
   0.6.13 and opened #22 himself to unify the drivers;
    these are tail triggers of
   exactly that work.
6. **Prototyped a minimal fix?
   ** Yes.
    See "Auto-prototype" below.

`.out-of-scope/` checked:
 no exemption for resharp (`cargo-workspace.md` mentions
resharp's Cargo settings only).

### Auto-prototype (constraint 6)

Prototyped in a disposable upstream clone (`mktemp --directory` under
`/tmp/agent/`,
 `origin` and commit verified `f0ce60a` before editing).
 The
minimal fix defers anchor-bearing patterns (detected from the ORIGINAL node) to
the authoritative `find_all`,
 keeping the fast path for anchor-free patterns:

- add an `anchors_orig` flag computed as `b.contains_anchors(node)` (original
  node) alongside the existing `has_anchors` from `node_fwd_simpl`;
- `find_anchored`:
   when `anchors_orig && find_all != Anchored`,
   return the
  leftmost-longest `find_all` match at offset 0 (the maintainer's own pre-`f12ff0b`
  approach),
   else the existing fast path;
- `is_match`:
   in the `Dfa/Hardened` and `Bounded/FwdPrefix/FwdLbPrefix` arms,
   when
  `anchors_orig`,
   return `!find_all(input)?.is_empty()`,
   else the existing fast
  path.

Full diff:
 [resharp-end-anchor-cross-api.patch](resharp-end-anchor-cross-api.patch)
(2 files,
 36 insertions).
 Verification,
 in the prototype clone:

- Both repros fixed:
   `find_anchored(\z|$)$` now returns `Some(0,0)`/`None`
  matching `find_all`;
   `is_match _&(?:[ab]|$)?` on `"\n"` now `false`.
   The
  campaign repro harness prints `RESULT: ALL FIXED`.
- No regression:
   the upstream test suite passes in a credential-free container
  (`podman run --rm --memory=6g --cpus=4 --volume <clone>:/work
  docker.io/rustlang/rust:nightly cargo test -p resharp`):
   **279 passed,
   0
  failed**,
   including `cross_api_prop.rs` and `engine_test.rs` (which exercise the
  touched contracts).

This fix (bugs 1+2) restores correctness conservatively;
 the maintainer may prefer
to instead correct `has_anchors` detection or land the full driver unification of
#22.
 It demonstrates a working minimal fix for those two and gives a tested
starting point.

Bug 3 prototype (insufficient,
 recorded as a failed probe).
 Extending the same
`anchors_orig` idea to the `FindAll` kind classifier (skip `FwdPrefix`/`FwdLbPrefix`
when `anchors_orig`,
 so `.&a(?:$|b)` routes to `Dfa`) does NOT fix `find_all`:
`.&a(?:$|b)` still returns `[]` and now trips the internal `ldfa.rs:844` assertion,
regressing the upstream `hardened_zero_width_interior_null_matches_default` test
(suite drops to 1 failed).
 This proves the defect is in the forward node built from
`simplify_fwd_initial`,
 not the routing layer:
 both the `FwdPrefix` and `Dfa`
forward scans miss the `$`-branch end.
 The real fix is algebra-core (the #22
driver/representation unification),
 beyond a minimal routing patch.
 Probe
recorded;
 not pursued further per the audit's bounded scope.

### Duplicate: issue #22 (open)

`gh search` found [ieviev/resharp#22](https://github.com/ieviev/resharp/issues/22)
"Unify the find_all / find_anchored / stream drivers behind one match-enumeration
core" (open,
 authored by this project).
 All three bugs are concrete instances of
the driver divergence #22 proposes to fix,
 and bug 3 is direct motivation for it
(the divergence reaching `find_all` itself).
 The thread does not yet contain these
reproductions,
 the `simplify_fwd_initial` anchor-drop root cause,
 the prototypes,
or the probe showing the routing fix is insufficient for `find_all`,
 so an additive
comment advances it.
 Do not open a new issue.

Additive comment draft (post to #22 only with authorization;
 discloses AI
assistance per a conservative reading of constraint 4):

~~~md
Three concrete instances of the driver divergence this issue is about, found
fuzzing 0.6.13 (HEAD f0ce60a). The first two have `find_all` correct and a sibling
API wrong; the third is in `find_all` itself, which makes this issue's unification
a soundness fix, not only a cleanup. All minimal and arch-independent (AVX2 and
NEON identical).

find_all FALSE NEGATIVE on intersection with an end-anchor alternation (the severe
one, in the production API):

```rust
// UnicodeMode::Ascii (all modes identical), input "a\n"
/./          : find_all=[(0,1)]   // left operand matches (0,1)
/a(?:$|b)/   : find_all=[(0,1)]   // right operand matches (0,1)
/.&a$/       : find_all=[(0,1)]   // subset of the trigger
/.&a(?:$|b)/ : find_all=[]        // intersection drops the (0,1) both operands have
```
Both operands match (0,1) per find_all, and the subset `.&a$` matches, but the
superset intersection returns []. Root: the forward node is built from
`simplify_fwd_initial(node)`, which drops the `$`-branch for `.&a(?:$|b)`; the Dfa
loop even has an assertion for it (ldfa.rs:844 "forward scan found no end for
reverse-proposed start 0") that fires in debug and is silently skipped in release
(returning []). Routing the kind off the original node does not help (both Dfa and
FwdPrefix forward scans miss the end); the fix is in the forward derivative itself.

find_anchored phantom/missing span on a union of end anchors:

```rust
// (\z|$)$ , UnicodeMode::Ascii (Default identical)
"\n"  : find_all=[(0,0),(1,1)]   find_anchored=Some(0,1)  // phantom width-1
"\na" : find_all=[(0,0),(2,2)]   find_anchored=None       // missing match at 0
```
`(\z|$)$` is zero-width only, so `Some(0,1)` cannot be in the language.
`find_anchored` delegates non-lookbehind patterns to `scan_fwd_optional`
(lib.rs:1908), which mis-answers this shape.

is_match false positive on intersection with an optional end anchor:

```rust
// _&(?:[ab]|$)? , UnicodeMode::Ascii
"\n" : is_match=true   find_all=[]   // false positive; no match exists
```
Root cause: `has_anchors` is computed on the forward-simplified node
(`b.contains_anchors(node_fwd_simpl)`, lib.rs:1177), and `simplify_fwd_initial`
drops the `$` branch as dead under the width-1 `_` intersection, so
`has_anchors=false` for this pattern (it is `true` for `$` alone). With the flag
false, is_match takes the `scan_fwd_optional` fast path (ismatch.rs:23) and
over-accepts.

For find_anchored and is_match, a minimal fix passes `cargo test -p resharp`
(279 passed, 0 failed): detect anchors from the ORIGINAL node (an `anchors_orig`
flag) and, when set, defer those two to `find_all` (the pre-f12ff0b find_anchored
approach), keeping the fast path for anchor-free patterns. That does NOT fix the
find_all case: routing `.&a(?:$|b)` to the Dfa kind leaves find_all=[] and trips
the ldfa.rs:844 assertion, so the find_all fix has to be in `simplify_fwd_initial`
/ the intersection-with-end-anchor forward derivative, which is squarely what this
issue's unification would address.
Investigation and patches were AI-assisted; the reproductions, the source trace,
the prototype test run, and the failed routing probe were all verified.
~~~
