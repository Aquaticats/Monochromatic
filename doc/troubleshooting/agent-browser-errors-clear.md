# agent-browser 0.36.0: `errors --clear` retains deliberately injected errors

## Symptom

The revised Promise lesson's final empty-error assertion failed with:

```text
AssertionError [ERR_ASSERTION]
actual: Error: revised-lesson-error-positive-control
expected: []
```

The harness had deliberately thrown that exception to prove the error monitor worked,
then called `errors --clear` before checking the lesson.
All chat and workshop behavior checks in process `proc_cf10` had passed.
The final error was the retained positive control,
not an unexpected lesson exception.

## Root cause

Inspected release:
`v0.36.0`,
commit `eb05921bad874cd2a1b4fa5d1149f1ed26576cae`.
The read-only clone is `~/temp/agent/agent-browser-quality-explorer-2026-09-06`.
Source references are relative to that clone.

`agent-browser errors --help` advertises `--clear` as `Clear error buffer`.
`cli/src/commands.rs:1728` parses the flag:

```rust
// cli/src/commands.rs
"errors" => {
    let clear = rest.contains(&"--clear");
```

The native dispatcher does not pass the command to this handler:

```rust
// cli/src/native/actions.rs:2673
"errors" => handle_errors(state).await,
```

The handler only reads the existing error record:

```rust
// cli/src/native/actions.rs:6192
async fn handle_errors(state: &DaemonState) -> Result<Value, String> {
    Ok(state.event_tracker.get_errors_json())
}
```

This differs from `handle_console` at `cli/src/native/actions.rs:6181`,
which reads `cmd.get("clear")` and calls `clear_console()`.
It is not evidence that the lesson continued throwing the same exception.

## Verification

Runnable local harness:
`~/temp/agent/promises-revision/verify-error-channel.mjs`.
It opens a disposable data-URL page in a uniquely named session,
checks an initially empty error record,
throws the named positive control,
checks the error,
calls `errors --clear`,
and reads the record again.

Working catalog:

- An intentionally thrown exception appears in `errors`.
- A fresh named session begins with an empty error record.
- Isolating the positive control from the lesson preserves an unfiltered final `errors: []` assertion.
  The revised advanced pass succeeded in `proc_28b9`.

Failing catalog:

- `errors --clear` followed by `errors` still returns the injected exception.
- Reusing that record for an empty-error assertion produces the false lesson failure in `proc_cf10`.

The probe logs:

```text
OBSERVED errors --clear retains the deliberate exception in this installed version
PASS error-channel positive control in a separate disposable session
```

## Verified workaround

Run the error-channel positive control in a separate disposable browser.
Close it,
then create a fresh lesson session.
Assert that the lesson's initial and final error records are empty.

`mise run test:advanced`,
from `~/temp/agent/promises-revision/`,
runs `verify-error-channel.mjs` before the independently named session in `verify-advanced.mjs`.
No error-text filtering is used.

Tradeoff:
this adds a separate browser lifecycle and does not repair clearing within a long-lived session.

## Verified upstream component prototype

The installed-source applicability check found the candidate implementation absent:
the native dispatcher still omits the command,
and the handler only reads errors.
The issue's `errors --clear` input and retained-buffer symptom match this incident.

A fresh private clone at `~/temp/agent/upstream-prototype.Xn969D`
was checked against the cited origin and release commit before applying PR #1654.
The [preserved patch](agent-browser-errors-clear.patch) contains its command-parser test,
command forwarding,
clearing method,
and native-action regression.
No installed executable was modified.

`mise run probe:error-clear-prototype`,
from `~/temp/agent/promises-revision/`,
builds a generated Rust component fixture using:

- The actual before/after dispatcher call expression and `handle_errors` function.
- The complete upstream `EventTracker` and its entry types.
- The PR's error-clearing test,
  with additional read-only,
  empty-buffer,
  console-preservation,
  and later-error controls.

The fixture substitutes a minimal state owner and response envelope for the full daemon.
Compilation and execution use `rust:1.97-bookworm` in a 2 GiB,
2-CPU container,
with only the private clone writable and no host proxy environment or credentials.
Dependencies are resolved into that fixture's own generated lockfile and cache.
The post-patch run uses that same lockfile with `--locked --offline` and no container network.

Process `proc_10a7` recorded the required before/after difference:

```text
Before patch:
3 passed; 4 failed
PASS expected pre-patch regression failures and read-only controls

After patch:
test tests::clearing_empty_record_succeeds ... ok
test tests::clearing_errors_preserves_console ... ok
test tests::errors_clear_empties_page_error_log ... ok
test tests::errors_after_clearing_are_recorded ... ok
test tests::errors_without_clear_preserve_records ... ok
test tests::missing_clear_flag_preserves_records ... ok
test tests::non_boolean_clear_preserves_records ... ok
7 passed; 0 failed
```

The correctness argument follows the tested boundaries:
forwarding `cmd` makes the parsed flag available to the handler;
only boolean `true` clears `error_entries`;
other inputs preserve the record;
clearing does not modify console entries or prevent recording later errors.
The non-boolean control probes the handler's existing fallback,
not a claim that the public CLI accepts a string-valued clear option.

This is component verification,
not a rebuild or end-to-end run of the patched CLI.
The preserved parser test was not run by this component harness.
The installed tool still uses the verified separate-session workaround.

## What does not work

- Assuming the documented flag cleared the native buffer without inspecting its result.
- Treating the retained test exception as an application failure.
- Filtering matching strings from the real lesson's error record.
  Isolation is the chosen boundary instead.

## Upstream filing decision

The existing [issue #1645][issue] contains the same CLI parsing and native-handler diagnosis.
[PR #1654][pr] already contains the clearing method,
command forwarding,
and regression tests.
Both were open when inspected.
Their full bodies and comments were read with `gh issue view --json` and `gh pr view --json`.
Searches covered issues and PRs using `"errors" "clear"`.

1.  Fault:
    confirmed mismatch between the advertised flag and the native handler.
2.  Fixability:
    the existing PR supplies an architecture-compatible implementation.
3.  Supported use:
    installed command help explicitly documents the flag.
4.  Contribution policy:
    the clone has no contribution file or issue/PR template;
    `AGENTS.md` explicitly addresses AI coding agents.
    No contribution ban was found in those files.
5.  Direction:
    an open fix exists;
    its comments contain no maintainer rejection of the change.
6.  Prototype:
    the existing patch was applied in a fresh private clone and its changed components
    passed the recorded before/after checks.
    The PR's full-CLI verification was not independently rerun.

The skill-relative `.out-of-scope/` directory was absent.
No new upstream contribution is prepared:
the existing issue and PR already identify this defect and its remedy.
There is no additional diagnosis or fix to contribute from this lesson test.
No duplicate issue or status-only comment was posted.
The upstream patch has not been installed.
Independent verification is limited to the component prototype described in this document.

[issue]: https://github.com/vercel-labs/agent-browser/issues/1645
[pr]: https://github.com/vercel-labs/agent-browser/pull/1654
