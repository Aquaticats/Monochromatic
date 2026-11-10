# agent-browser 0.38.1: a daemon-version mismatch resets an un-restored session to about:blank

## Symptom

A read-only-looking `agent-browser --session promises-open-chat-present ... tab list` invocation emitted:

```text
⚠ Daemon version mismatch detected, restarting...
```

Its JSON reported `lifecycle.restartedBackground: true` and
`lifecycle.restoreStatus: "not_configured"`,
with only one `about:blank` tab.
The session had previously been used to present a local choice form,
but its immediate pre-restart tabs and any unsubmitted draft were not captured.
Do not claim that a particular user draft was lost or preserved.
The corrected form was subsequently opened in an empty tab;
a later presentation opened a separate tab instead of reloading an existing form.

## Root cause

Source checkout:
`vercel-labs/agent-browser`,
commit `d01253d`,
whose `cli/Cargo.toml:3` says `version = "0.38.1"`.
This is a deliberate version-compatibility path,
not evidence of an upstream bug.

`cli/src/connection.rs:707-714` reads a per-session version sidecar.
A missing file is treated as a mismatch:

```rust
fn daemon_version_matches(session: &str) -> bool {
    let version_path = get_version_path(session);
    match fs::read_to_string(&version_path) {
        Ok(v) => v.trim() == env!("CARGO_PKG_VERSION"),
        Err(_) => false,
    }
}
```

`cli/src/connection.rs:803-820` checks the existing daemon before dispatching even a tab-list request.
On mismatch it prints the warning and stops that daemon:

```rust
if daemon_ready(session) {
    if !daemon_version_matches(session) {
        eprintln!(
            "{} Daemon version mismatch detected, restarting...",
            crate::color::warning_indicator()
        );
        stop_existing_daemon_for_restart(session);
        restarted = true;
```

`cli/src/connection.rs:774-789` first sends internal action
`__agent_browser_internal_shutdown`,
then falls back to terminating a stale daemon.
`cli/src/native/actions.rs:2692-2700` routes that internal action to `handle_close`.
`cli/src/native/actions.rs:5666-5669` saves configured state and closes browser backends:

```rust
async fn handle_close(state: &mut DaemonState) -> Result<Value, String> {
    let save_result = auto_save_restore_state(state).await;
    close_all_browser_backends(state).await?;
```

`cli/src/native/actions.rs:4718-4723` returns without a saved state when the session has no
restore configuration:

```rust
let Some(session_name) = state.session_name.clone() else {
    state.restore_save_status = "not_configured".to_string();
    state.restore_saved_path = None;
    return Ok(None);
};
```

`tab_list` is not in the no-launch action set in `cli/src/native/actions.rs:2568-2592`.
`cli/src/native/actions.rs:2786-2801` requests an implicit browser launch when none exists,
and `cli/src/native/actions.rs:2892-2917` launches and ensures a page.
Thus the restarted session can answer tab-list with a new blank tab.
This trace explains the disposable reproduction;
it does not establish what was open
immediately before the real warning.

## Verification

Installed CLI:
`agent-browser 0.38.1`.
The source checkout's `origin` is `https://github.com/vercel-labs/agent-browser.git`.
Run the owned disposable fixture,
not the user's named session:

```bash
# /home/user/temp/agent/promises-revision/mise.toml
cd -- /home/user/temp/agent/promises-revision && mise run probe:browser-version-reset
```

`probe-agent-browser-version.mjs` creates a private socket directory under
`/home/user/temp/agent`,
opens a data-URL sentinel,
verifies same-version tab-list reuses it,
then changes only that disposable session's `.version` file to `0.0.0`.
It observed:

```text
versionBefore: 0.38.1
stable: data:text/html,<title>disposable-version-control</title><p>sentinel</p>
warning: ⚠ Daemon version mismatch detected, restarting...
after: about:blank
restore: not_configured
```

The script closes the disposable browser and terminates its owned daemon.
It does not recreate or inspect any possible pre-restart user draft.

- Works cleanly:
  same-version sidecar with a live daemon retains the sentinel tab.
- Fails by version restart:
  mismatched sidecar prints the exact warning,
  sets `restartedBackground`,
  and starts with `about:blank` without restore configuration.
- Missing sidecar also counts as a mismatch by the cited source,
  and the consumer preflight treats it as unsafe;
  it was not separately exercised with a live daemon.

## Verified workarounds

- Before contacting a named daemon,
  compare the CLI version with its
  `<session>.version` sidecar in the socket directory.
  `cli/src/connection.rs:100-160` defines the socket directory priority and version filename.
  `ux-choice-session-preflight.mjs` applies this at the consumer boundary:
  on mismatch it chooses a version-qualified **different** session instead of contacting the old daemon.
  `mise run test:ux-session-preflight` passed same-version,
  mismatched,
  missing,
  and mismatched-alternate sidecar fixtures.
  Tradeoff:
  the new session cannot recover or display the old session's in-memory form draft,
  and it may open a separate browser window.
- `present-ux-choice.mjs` navigates only an active empty tab or opens a separate tab,
  never reloads a form containing a possible choice or free text.
  Its corrected matrix was opened in a new tab and checked for six live cells,
  the ignored-call trace,
  empty form inputs,
  and zero browser errors.
  Tradeoff:
  it may leave a superseded choice tab open for the user's comparison;
  it cannot recover a tab already closed by an earlier daemon restart.

## What does not work

- Reusing only the same `--session` name after a version change:
  the version check runs before the requested command and restarts its daemon.
- Adding a no-reload rule **after** the CLI has already contacted a mismatched daemon:
  the shutdown path has already closed its browser backends.
- Inferring a user's draft status from a later `about:blank` tab:
  no immediate pre-restart form snapshot was taken in this incident.

## Upstream filing decision

The repository's `.out-of-scope/` directory was checked;
no agent-browser exemption matched.
A search of issues and pull requests for `version mismatch` found
[issue 1127][issue-1127] and [merged PR 1134][pr-1134].
The PR intentionally added the sidecar,
warning,
and restart to prevent reusing an incompatible daemon.
Nothing in this investigation advances that resolved thread;
post no comment and file no new issue.

1.  **Upstream fault?**
    No. The observed restart is the merged compatibility behavior.
2.  **Can upstream change it?**
    Technically yes,
    but a different preservation design
    would be a new request,
    not a fix established by this incident.
3.  **Supported use case?**
    Named sessions and optional restore are supported;
    preserving an unconfigured session across a CLI upgrade was not established as a guarantee.
4.  **Would upstream welcome a contribution?**
    Not assessed,
    because this is not a demonstrated fault;
    no issue or patch is proposed.
5.  **Likely fix?**
    No signal that maintainers want to revert the merged restart behavior.
6.  **Minimal compatible upstream fix prototyped?**
    No. The consumer-side version preflight
    was verified instead;
    the upstream fix gate is not entered because constraint 1 fails.

No fileable issue or additive comment draft exists for this expected behavior.

[issue-1127]: https://github.com/vercel-labs/agent-browser/issues/1127
[pr-1134]: https://github.com/vercel-labs/agent-browser/pull/1134
