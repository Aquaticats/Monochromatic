# BUG-1 re-entrancy guard panic in union and intersection rewrites

## Classification

- Type:
   internal invariant panic,
   self-labelled "this is a bug,
   please file an
  issue with the pattern".
- Phase:
   compile time,
   inside `Regex::with_options` (derivative preparation).
- Severity:
   panic in a debug or default build;
   in a release build the same
  guard takes `return None`,
   so the rewrite is silently skipped and the result
  can be wrong instead of crashing.
- Gating:
   `#[cfg(feature = "reentrant-assert")]` (a default feature) plus
  `#[cfg(debug_assertions)]`.

## Minimal reproducer

```rust
//! Reproduction of the reentrant rewrite panic:
//! constructing this pattern panics before any matching happens.

/// The regex type whose construction panics.
use resharp::Regex;
// panics during construction, before any matching:
let _ = Regex::new(r".*(.+)*.+");
```

Command line:

```sh
repro '.*(.+)*.+' ''
```

It panics even with an empty haystack,
 which proves the panic is at compile
time (all of `is_match`,
 `find_all`,
 `find_anchored` short-circuit on an empty
input before any scan).

## Observed behaviour

```text
thread panicked at resharp-algebra/src/lib.rs:2595:
reentrant union rewrite ".*" | ".*(.+)*.+", this is a bug, please file an issue with the pattern
```

The sibling intersection guard at `resharp-algebra/src/lib.rs:2891` produces the
same class of panic for intersection-rewrite re-entrancy.

## Expected behaviour

Construction either succeeds or returns a typed `Err`.
 It must not panic and
must not silently skip a rewrite.

## Root cause

`resharp-algebra/src/lib.rs`,
 `attempt_rw_union_2`:

```rust
//! The guard added around the union rewrite,
//! which detects a union rewrite that re-enters itself.

/// Rewrites the union of two nodes,
/// with the `reentrant-assert` guard that panics in debug builds and returns no rewrite in release builds when the same union is already being rewritten.
fn attempt_rw_union_2(&mut self, left: NodeId, right: NodeId) -> Option<NodeId> {
    #[cfg(feature = "reentrant-assert")]
    if !self.rw_active.insert((Kind::Union, left, right)) {
        #[cfg(debug_assertions)]
        panic!("reentrant union rewrite {:?} | {:?}, ...", self.pp(left), self.pp(right));
        #[cfg(not(debug_assertions))]
        return None;
    }
    let r = self.attempt_rw_union_2_inner(left, right);
    #[cfg(feature = "reentrant-assert")]
    self.rw_active.remove(&(Kind::Union, left, right));
    r
}
```

`rw_active.insert` returns false when the same `(Kind, left, right)` rewrite is
already in progress higher in the call stack.
 The nested unbounded quantifiers
in `.*(.+)*.+` drive the derivative engine to request a union rewrite that is
already being computed,
 so the guard fires.
 This is the union-rewrite analogue
of the already-tracked "intersection over alternation:
 unbounded algebra
recursion" finding.

## Notes

- The guard is the project's own diagnostic,
   so this is the guard correctly
  detecting a re-entrancy the rewrite system does not expect.
- In the suppressed fork used for the rest of this campaign,
   both guards were
  set to `return None` so the fuzzer could reach the rest of the surface.
- Distinct triggers:
   many nested-unbounded-quantifier shapes reach this,
   for
  example `(?:(?:(?:(?:1)+){1,2})+){2,2}` (found by `diff_regex`).
