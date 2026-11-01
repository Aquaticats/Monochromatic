# find_all false negative on intersection with an end-anchor alternation (0.6.13, live)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

THIRD new live soundness bug,
 and the most severe of the three:
 `find_all` (the
production API `forbidden-strings` relies on) DROPS a real match.
 Found by the
anchor-extended denotational oracle (`anchor_denot`,
 the lane that closes the
06-19 campaign's anchor-coverage gap),
 adjudicated WITHOUT any external model:
resharp's own outputs contradict each other.
 Verified on 0.6.13 (== HEAD
`f0ce60a`),
 byte-identical on x86_64 (AVX2) and Apple M1 (NEON),
 all four unicode
modes.

## Symptom

Minimal trigger:
 `.&a(?:$|b)` on `"a\n"`.

```text
/.&a(?:$|b)/  (UnicodeMode::Ascii, Default, Full, Javascript all identical)
  "a\n" : find_all=[]   is_match=false
```

Both expected by the contract to be a match at `(0,1)`:

```text
/./          on "a\n" : find_all=[(0,1)]   (the left operand matches (0,1))
/a(?:$|b)/   on "a\n" : find_all=[(0,1)]   (the right operand matches (0,1))
/.&a$/       on "a\n" : find_all=[(0,1)]   (subset of the trigger; matches (0,1))
```

So both operands of the intersection individually match the span `(0,1)`,
 per
resharp's OWN `find_all`,
 yet `.&a(?:$|b)` (their intersection) returns `[]`.
 An
intersection cannot drop a span that both operands contain.
 This is a `find_all`
(and `is_match`) FALSE NEGATIVE.

## Why this is a bug, with no external oracle needed

Two independent resharp-internal proofs,
 neither using the denotational model:

1. **Membership.
   ** `(0,1) in L(.)` and `(0,1) in L(a(?:$|b))` are both witnessed by
   resharp's own `find_all` on each operand.
    `L(A & B)` is `L(A) cap L(B)` by
   definition,
    so `(0,1)` must be in `L(.&a(?:$|b))`.
    resharp returns `[]`.
2. **Monotonicity.
   ** `L(a$) subset of L(a(?:$|b))` (the second adds an alternative
   branch),
    hence `L(.&a$) subset of L(.&a(?:$|b))`.
    resharp gives `.&a$` ->
   `[(0,1)]` but the superset `.&a(?:$|b)` -> `[]`:
    a superset returning fewer
   matches.

The `$` matches at the interior position 1 (multiline default,
 before the `\n`),
which is the match `.&a$` correctly finds;
 adding the consuming alternative
`(?:$|b)` makes `find_all` drop it.

## Trigger boundary

```text
.&a(?:$|b)     BUG    [ab]&a(?:$|b)   BUG    .&a(?:b|$)   BUG (alternation order)
.&a(?:$|a)     BUG    [a\n]&a(?:$|b)  BUG    .&a($|bc)    BUG
.&a$           ok ((0,1))            a(?:$|b)        ok ((0,1), no &)
```

Two ingredients:
 an intersection `L & R` where `L` is width-bearing,
 and `R` has
the shape `<char>(?:$|<consuming>)` (an end anchor `$` alternated with a consuming
branch).
 Replacing the alternation with the bare anchor (`.&a$`) is correct;
dropping the intersection (`a(?:$|b)` alone) is correct.
 The bug is the
combination.

## Root cause

The defect is in the FORWARD node construction,
 not the kind classifier.
 The
forward matcher is built from `node_fwd_simpl = b.simplify_fwd_initial(node)`
(`resharp-engine/src/lib.rs:1035`);
 for `.&a(?:$|b)` the simplification drops the
`$` alternative as locally dead,
 so the forward automaton itself cannot reach the
`$`-branch end,
 while the reverse pass still proposes the start.
 The Dfa find_all
loop has an internal tripwire for exactly this inconsistency
(`resharp-engine/src/ldfa.rs:842-846`):

```rust
debug_assert_ne!(
    NO_MATCH, l_max_end,
    "find_all: forward scan found no end for reverse-proposed start 0"
);
if l_max_end != NO_MATCH {
    matches.push(Match { start: 0, ...
```

In debug this assertion fires;
 in release (`debug_assert` off) the `if` simply
does not push the match,
 so `find_all` silently returns `[]`.
 This is the false
negative.

The kind classifier is a contributing-but-secondary factor.
 The `FindAll` kind is
chosen from `has_anchors_pre = b.contains_anchors(node_fwd_simpl)`
(`lib.rs:1069`);
 since the `$` is dropped,
 `has_anchors_pre` is `false` and the
pattern is classified `FwdPrefix`,
 while the correct `.&a$` keeps the anchor and
gets `Dfa`:

```text
/.&a(?:$|b)/  find_all_kind = FwdPrefix      /.&a$/  find_all_kind = Dfa
```

Earlier reading (in the first commit of this doc) was WRONG:
 it claimed the root
was the `FwdPrefix` misclassification and that routing the kind decision off the
original node would fix `find_all`.
 The prototype disproved this.
 Forcing
`.&a(?:$|b)` to `Dfa` (via an `anchors_orig`-gated classifier guard) leaves
`find_all` still `[]` and trips the `ldfa.rs:844` assertion above (regressing the
upstream `hardened_zero_width_interior_null_matches_default` test):
 the Dfa
forward scan,
 built on the same corrupted `node_fwd_simpl`,
 also cannot find the
end.
 So the fix is not at the routing layer;
 it is in `simplify_fwd_initial` /
the intersection-with-end-anchor-alternation forward derivative (the algebra
core),
 i.e. the driver/representation unification of issue #22.

Relation to the other two findings:
 bugs 2 (`is_match`) and 3 (`find_all`) share
the SYMPTOM (an anchor dropped by forward simplification),
 but only bug 2 is fixed
at the routing/fast-path layer (defer to `find_all`);
 bug 3 is in `find_all`
itself and is algebra-deep.
 Bug 1 (`find_anchored` on `(\z|$)$`) shares the fix
SHAPE (defer via `anchors_orig`) but its locus is `scan_fwd_optional`,
 a sibling.

## Adjudication and severity

Real bug,
 tier "asserted-contract / internal-inconsistency" (self-evident from
resharp's own operand outputs;
 no model needed).
 Most severe of the three 06-19
findings for the CRATE:
 it is in `find_all`,
 the production API,
 and the direction
is a FALSE NEGATIVE (a dropped match),
 i.e. fail-OPEN.
 A crate consumer that gates
on `find_all` over an intersection-with-end-anchor pattern could MISS content that
actually matches.
 The two earlier findings were a `find_anchored` phantom and an
`is_match` false positive (fail-toward-noise);
 this one fails toward silence.

Consumer scope (crate vs forbidden-strings):
 this does NOT affect our consumer.
`forbidden-strings` rules (`package/cli/forbidden-strings/data/`,
betterleaks-style) are ordinary leftmost patterns;
 a search for intersection (`&`)
co-occurring with `$`/`\z` in the rule set finds none,
 and the scanner uses
`find_all`.
 The fail-open framing is a crate-level property for
intersection-with-anchor patterns,
 not a forbidden-strings exposure.

Why the self-consistency lane could not have found this (the campaign-gap payoff):
on the minimal trigger,
 `is_match=false` AND `find_all=[]` AGREE,
 so the C1
contract (is_match iff find_all non-empty) HOLDS.
 Every lane the original campaign
ran (self-consistency,
 and the denotational oracle which had no anchors) is blind
to a COORDINATED false negative;
 only an independent-ground-truth oracle over the
anchor family catches it.
 That oracle (`anchor_denot`) is exactly the gap this
round closed,
 which is why the third finding existed to be found.

Why the prototype does not fix it:
 the earlier patch
(`doc/troubleshooting/resharp-end-anchor-cross-api.patch`) guards `is_match` and
`find_anchored` only.
 A second prototype that additionally routes the `FindAll`
kind off the original node was attempted and FAILED:
 it forces `.&a(?:$|b)` to
`Dfa` but `find_all` stays `[]` and the Dfa path trips the `ldfa.rs:844`
assertion,
 regressing `hardened_zero_width_interior_null_matches_default`.
 The
fix is not at the routing layer;
 it is in `simplify_fwd_initial` / the forward
derivative (algebra core,
 issue #22 territory).
 Recorded as a useful failed probe;
not pursued further.

## Reproduce

```bash
# /tmp/agent/resharp-denot-oracle, resharp = "=0.6.13"
cargo run --release --bin anchor_denot          # the lane that found it (RESHARP-INCONSISTENT line)
# minimal, both arches, resharp-internal proof:
#   /./ and /a(?:$|b)/ both match (0,1) on "a\n"; /.&a(?:$|b)/ returns []
```

## Upstream filing

Not yet drafted as an artifact;
 see the troubleshooting doc update for the
6-constraint check,
 the (to-be-extended) prototype,
 and the additive comment on
issue #22.
 Outward filing is authorization-gated.
