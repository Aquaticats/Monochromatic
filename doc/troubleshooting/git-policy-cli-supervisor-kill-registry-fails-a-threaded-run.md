# Two supervisor controls plant kills in one process-wide registry, so a threaded run fails one of them

`package/git-policy/cli/src/native/binary/binary_supervisor_tests.rs` holds controls that run as threads of one
process under `cargo test`, and the kill registry they plant records in is one static that the whole process shares.

## Symptom

`cargo test --locked --offline --test native_binary --features test-only-switches supervisor_guard -- --skip guest --skip _arm_tests`
failed one control on 2026-11-25, and the same filter and the control alone passed afterwards:

```console
$ cargo test --locked --offline --test native_binary --features test-only-switches supervisor_guard -- --skip guest --skip _arm_tests
thread 'supervisor_guard::a_kill_nobody_claimed_is_found_after_the_grace_only' panicked at src/native/binary/binary_supervisor_tests.rs:704:5:
assertion `left == right` failed: a claimed kill is not reported again
  left: Some("the planted kill")
 right: None
test result: FAILED. 31 passed; 1 failed; 0 ignored; 0 measured; 1325 filtered out; finished in 6.19s

$ cargo test --locked --offline --test native_binary --features test-only-switches supervisor_guard -- --skip guest --skip _arm_tests
test result: ok. 32 passed; 0 failed

$ cargo test --locked --offline --test native_binary --features test-only-switches supervisor_guard::a_kill_nobody_claimed_is_found_after_the_grace_only -- --exact
test result: ok. 1 passed; 0 failed
```

## Root cause

`binary/binary_supervisor.rs` keeps one registry for the process (`with_registry`), and two controls plant a record
in it with the same message and then ask the registry-wide question:

- `a_kill_nobody_claimed_is_found_after_the_grace_only` (`:698`) plants `i32::MAX - 7` with `"the planted kill"`,
  claims its own record with `take_kill`, and asserts `unclaimed_kill` answers `None`.
- `a_kill_a_thread_is_occupied_with_is_not_what_aborts_the_process` (`:714`) plants `i32::MAX - 6` with the same
  `"the planted kill"` and asks `kill_to_abort` about the whole registry.

`unclaimed_kill` (`binary_supervisor.rs:620`) walks every record and answers the first whose age has passed the
grace, whatever its process number. While the second control's record is still there, the first control's last
assertion reads it and fails with a message that looks like its own. Each control alone has the registry to itself,
so each passes in a process of its own, which is what the container gate gives it.

## The evidence

`git diff main -- package/git-policy/cli/src/native/binary/binary_supervisor_tests.rs` shows only this branch's own
runner change in its last control; the two planting controls are main's. The filter failed once and passed twice,
and the control alone passed, so the defect is the shared registry and the order the controls interleave in.

## What a fix needs

The record a control plants has to be the one it reads back: a process number each control owns, or a planted
record found by that number rather than by age alone. The controls' own message could also differ, which would make
a cross-read visible as its own failure instead of a mystery.

This page records the defect and the measurement only. The fix belongs to a round that owns
`binary/binary_supervisor_tests.rs` and the registry behind it; no round has landed it as of 2026-11-25.
