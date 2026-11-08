# BUG-4 re-entrancy guard panic at compile time is still reachable

- Type:
   crash,
   compile-time denial of service.
   A user pattern panics inside
  `Regex::new` / `with_options`.
- Phase:
   compile time,
   the algebra union rewrite.
- Severity:
   high.
   A panic on untrusted input aborts the calling thread (and,
   if
  it unwinds through the `inner` mutex,
   can poison a shared `Regex`,
   cf. 06-04
  BUG-25).
   The panic message itself says "this is a bug,
   please file an issue".
- Affected:
   all configs (shown in ascii / default).
- Architecture:
   confirmed byte-identical on aarch64 (Apple M1) and x86-64.
  `armprobe "(.*.+)*.+" "aaa"` on the M1 panics at the same
  `resharp-algebra/src/lib.rs:2724` site (this is the arch-independent algebra
  rewrite,
   not a SIMD path;
   the ARM run only makes "ARM-confirmed" airtight).
- Discovery:
   the `diff_regex` libFuzzer target (~120 distinct inputs,
   all this
  site) and the self-consistency oracle (45 more).

## Reproducer

```rust
//! Reproduction of the live reentrant union rewrite panic:
//! constructing this pattern panics with the message quoted in the comments.

/// The regex type whose construction panics.
use resharp::Regex;
let _ = Regex::new(r"(.*.+)*.+"); // panics during construction
// thread panicked at resharp-algebra/src/lib.rs:2724:
//   reentrant union rewrite ".*" | ".*(.*.+)*.+", this is a bug, please file an issue
```

Harness:

```sh
# 28 2e2a2e2b 29 2a 2e2b  = "(.*.+)*.+"
repro --show 282e2a2e2b292a2e2b 616161 1
# compile=PANIC|.../resharp-algebra/src/lib.rs:2724 reentrant union rewrite ...
```

## Observed versus expected

Expected:
 any pattern either compiles or returns `Err` (parse / capacity / size).
A panic is never acceptable on user input.
 resharp panics at
`resharp-algebra/src/lib.rs:2724` ("reentrant union rewrite") while computing a
derivative whose union rewrite recurses into itself.
 The trigger family is a
`*`/`+` quantifier wrapped around an expression that already contains `.*`/`.+`
(e.g. `(.*.+)*.+`,
 `(0*.{3}b{0,2})+...`,
 `(.{0,2}.{2,}[a-c]{3}\W*)*\w{2}.*`),
producing a self-referential union during the star derivative.

## Relationship to 2026-06-04 BUG-1

This is BUG-1 (the re-entrancy guard panic,
 `attempt_rw_union_2`).
 It was
reported fixed;
 the exact 06-04 minimal `.*(.+)*.+` now compiles,
 but the guard
panic is still reachable on a broad class of nested-quantifier patterns.
 The
developer's own "please file an issue" message confirms the path is a known
incomplete rewrite,
 not a deliberate rejection.
 Narrowed,
 not eliminated.

## Source pointer

`resharp-algebra/src/lib.rs:2724` (the `reentrant union rewrite` panic).
 The
companion intersection guard is the sibling site.
 The release fallback for these
guards is `return None`;
 the assertion fires in debug-assertions / fuzz builds
and any build that keeps the guard active (the `reentrant-assert` feature was a
default feature in the 06-04 baseline;
 confirm whether it still is).
 Even where
the release path returns `None`,
 that silently changes the compiled language,
which is its own correctness concern.
