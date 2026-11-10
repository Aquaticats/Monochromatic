# BUG-1 re-entrancy guard panic still present in v0.6.12 on ARM64

## Classification

- Type:
   compile-time panic (host-aborting).
- Phase:
   compile time,
   inside `Regex::with_options`.
- Severity:
   user input can abort the host process in debug builds (where
  the `reentrant-assert` feature is enabled by default).

## Status

The developer claimed BUG-1 was fixed.
 It is not.
 The panic still fires on
ARM64 v0.6.12 on patterns that cause recursive union rewrite.

## Minimal reproducer

From `diff_regex` fuzz target crash on ARM64 v0.6.12:

```text
artifact: diff_regex_crash-dd2e0be5878b40e4e0c4b7dd8597f0b3abf5de45
full_hex: 283f3c283f3d8a6c2c5b302d7c21a35b7c7e
```

Panic message:
```text
reentrant union rewrite "[^a-c].*" | "[^a-c].*(0[^a-c].*)*0[^a-c].*",
this is a bug, please file an issue with the pattern
```

Stack trace (key frames):
```text
#17 attempt_rw_union_2 at resharp-algebra/src/lib.rs:2724
#18 mk_union at resharp-algebra/src/lib.rs:3811
    ... (recursive mk_union → attempt_rw_union_2 loop)
#55 calc_prefix_sets_inner at resharp-engine/src/prefix.rs:43
#56 calc_prefix_sets at resharp-engine/src/prefix.rs:91
#57 compute at resharp-engine/src/prefix.rs:261
#58 select_prefix_simd at resharp-engine/src/prefix.rs:914
#59 from_node_inner at resharp-engine/src/lib.rs:1068
#60 Regex::with_options at resharp-engine/src/lib.rs:997
```

To reproduce:
```sh
cargo +nightly fuzz run diff_regex <artifact> \
  --fuzz-dir fuzz --target aarch64-unknown-linux-gnu -- -runs=1
```

## Observed behaviour

The `diff_regex` fuzz target on ARM64 found this crash after 18,242 runs
(~40 seconds).
 The `compile` target did not find it because the `compile`
target's option-sweep does not produce the exact pattern shape that triggers
the recursive union rewrite (the `DiffPattern` generator produces patterns
with alternation and bounded repeat that the option-sweep does not).

## Root cause

`attempt_rw_union_2` at `resharp-algebra/src/lib.rs:2724` calls
`mk_union`,
 which calls `attempt_rw_union_2_inner`,
 which calls
`attempt_rw_union_2` again (frames #31-#48 show the recursion).
 The
`reentrant-assert` feature guard panics when `rw_union_depth > 0`,
detecting the recursion.
 In release builds,
 this returns `None` instead
of panicking.

The developer may consider this "by design" since the release fallback is
correct,
 but:
1.  The `reentrant-assert` feature is a **default feature**,
     so debug builds
    (which every developer and test suite uses) will panic on user input.
2.  The panic message says "this is a bug",
     confirming the developer's own
    assessment that this is a bug,
     not intended behaviour.

## Relationship to 06-04 campaign

This is the exact same BUG-1 from the 2026-06-04 campaign.
 The developer
claimed it was fixed but it was not;
no commit between v0.6.9 and v0.6.12
touches the `attempt_rw_union_2` re-entrancy guard.

## Artifact

Persistent binary + decoded reproducer:
```text
/work/artifacts/pristine-diff-regex/diff_regex_crash-dd2e0be5878b40e4e0c4b7dd8597f0b3abf5de45
/work/artifacts/pristine-diff-regex/diff_regex_crash-dd2e0be5878b40e4e0c4b7dd8597f0b3abf5de45.txt
```
