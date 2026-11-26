# Two heartbeat controls share one manager switch, so a threaded run fails one of them

`package/git-policy/cli/src/native/binary/binary_service_heartbeat_tests.rs` holds controls that run as
threads of one process under `cargo test`, and two of them set and reset one process-wide atomic that the whole
file's fake service manager reads.

## Symptom

`cargo test --locked --offline --test native_binary service` at the default threads failed one control twice in
a row on 2026-11-25, and each run of the same filter with `--test-threads=1` passed it:

```console
$ cargo test --locked --offline --test native_binary service -- --skip guest --skip _arm_tests
test result: FAILED. 254 passed; 1 failed; 2 ignored; 0 measured; 1099 filtered out; finished in 114.73s
failures:
    service_heartbeat::a_service_that_starts_late_is_waited_for_while_the_manager_does_not_say_failed

thread 'service_heartbeat::a_service_that_starts_late_is_waited_for_while_the_manager_does_not_say_failed' panicked at src/native/binary/binary_service_heartbeat_tests.rs:551:5:
the wait ended before the service started

$ cargo test --locked --offline --test native_binary service -- --skip guest --skip _arm_tests --test-threads=1
test result: ok. 255 passed; 0 failed; 2 ignored; 0 measured; 1099 filtered out; finished in 687.61s
```

The control alone passes too:
`cargo test --locked --offline --test native_binary service_heartbeat::a_service_that_starts_late_is_waited_for_while_the_manager_does_not_say_failed -- --exact`
answers `test result: ok. 1 passed; 0 failed`.

## Root cause

`MANAGER_SAYS_FAILED` (`:455`) is a `static AtomicBool` that `fake_manager` reads for every manager watch of the
file. Two controls share it:

- `a_service_that_never_starts_ends_the_wait_when_the_manager_says_the_start_failed` (`:514`) sets it true,
  sleeps `OLD_START_WAIT * 6` (1.2 s), and sets it false again.
- `a_service_that_starts_late_is_waited_for_while_the_manager_does_not_say_failed` (`:544`) sets it false,
  sleeps the same 1.2 s, and then asserts its client thread is still waiting.

The watches poll every 50 ms (`watched`). While the first control holds the switch true, the second control's
client is told the unit is `failed`, twice, and its wait ends: the assertion that a service starting late is still
being waited for then fails. Under `cargo test` both controls are threads of one process, so the switch is shared;
the container gate gives each control a process of its own, which is why it does not see this.

## The evidence

`git diff main -- package/git-policy/cli/src/native/binary/binary_service_heartbeat_tests.rs` is empty: the file
is main's. The threaded run of the whole `service` filter failed this control twice, and a third run of the same
filter passed 255; the same filter serially passed 255, the control alone passed, and the whole
`service_heartbeat` module threaded passed 27, because which two controls of the file run at the same moment is
not fixed. The defect is in the file either way; a passing run is the interleaving that happens to hold.

## What a fix needs

The switch has to belong to one control instead of the module: a value the fake manager reads through the
command's own request, or a lock that only one control of the file holds while its switch is set. The two
controls' sleeps are also witnesses of the absence of an ending, which a shared switch defeats
(`doc/planning/cli-git-tests-that-pass-without-their-race.md`).

This page records the defect and the measurement,
and the round of 2026-11-26 that owns `binary/binary_service_heartbeat_tests.rs` landed the fix below.

## The fix that landed (2026-11-26)

Commit `822055daef` removed all four statics.
A `ManagerWatch` hands each of its functions the path it names as the session bus
(`service/service_client_settings_and_watches.rs`),
and no stand-in of this file asks a bus,
so that path is now the folder one control owns:
a `the-start-failed` or `the-unit-is-inactive` file in it is the word the stand-in answers,
and a start leaves `asked-to-start` there.
Each control takes its folder from the harness (`super::support::fixture`),
which removes it however the test ends,
so two controls cannot reach one answer,
and the resets went with the statics.

The same filter before and after the change,
on this host:

```console
$ cargo test --locked --offline --features test-only-switches --test native_binary service -- --skip guest --skip _arm_tests
test result: FAILED. 254 passed; 1 failed; 2 ignored; 0 measured; 1103 filtered out; finished in 104.85s
test result: FAILED. 254 passed; 1 failed; 2 ignored; 0 measured; 1103 filtered out; finished in 94.97s
test result: ok. 255 passed; 0 failed; 2 ignored; 0 measured; 1103 filtered out; finished in 94.03s
test result: ok. 255 passed; 0 failed; 2 ignored; 0 measured; 1103 filtered out; finished in 94.26s
```

The two failing runs are before the change,
and both name `a_service_that_starts_late_is_waited_for_while_the_manager_does_not_say_failed`.
The two passing runs are after it.
The module threaded (`service_heartbeat::`) reported 27 passed after the change,
and each of the five controls that read the fake manager passed alone by `--exact` before and after.

Commit `330f7e8a85` added the guard,
`source_guard/heartbeat_controls_own_their_manager_switch_tests.rs`:
it fails on any `static` the file declares and names the line.
Its own control plants one in a fixture tree,
and with a `static` added back to the file it reports `binary/binary_service_heartbeat_tests.rs:745`.
