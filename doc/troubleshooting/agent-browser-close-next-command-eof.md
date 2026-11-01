# agent-browser 0.39.1: the first command after `close` fails with `Invalid response: EOF`

## Symptom

On 2026-10-20 on this Linux host,
the first `agent-browser` command after `agent-browser close` failed,
whatever it was (`set viewport 390 950`,
`open about:blank`,
`eval`,
`snapshot`,
`tab list`,
`get url`):

```text
✗ Invalid response: EOF while parsing a value at line 1 column 0 (the command reached the daemon but no response came back; it may or may not have run, so it was not retried)
```

Running the same command again straight away succeeded.
With `--json` the failure is `{"code":"outcome_unknown","success":false,...}`.

Reproduced here as written (agent-browser 0.39.1,
headless Chrome 149.0.7827.54):

```sh
agent-browser --session probe open about:blank
agent-browser --session probe close                       # ✓ Browser closed
agent-browser --session probe set viewport 390 950        # ✗ Invalid response: EOF ...
agent-browser --session probe set viewport 390 950        # ✓ Done
```

The string is the same one that [agent-browser-eagain.md](agent-browser-eagain.md) records for a 30 second read timeout,
but the cause is different.
The timeout shape is `Failed to read: Resource temporarily unavailable (os error 11)` after a long wait;
this shape is `Invalid response: EOF` within milliseconds of a `close`.

## Root cause

Source checked:
`vercel-labs/agent-browser` tag `v0.39.1`,
commit `1165952d74ceed98fdc2dd97190ad41f44dd0e24`,
whose `cli/Cargo.toml` says `version = "0.39.1"`.
The npm package under test is the mise install of `agent-browser` 0.39.1 (`agent-browser --version` prints `0.39.1`).

### The daemon acknowledges `close`, then keeps listening for 100 ms

`cli/src/native/daemon.rs:594-610` writes the `close` reply first and only then sleeps and signals the accept loop:

```rust
let mut resp = serde_json::to_string(&response).unwrap_or_default();
resp.push('\n');
if writer.write_all(resp.as_bytes()).await.is_err() {
    break;
}

if close_completed_response(&action, &response) {
    // ...
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    close_notify.notify_one();
    return;
}
```

During that 100 ms the accept loop (`cli/src/native/daemon.rs:261-263`,
`loop { tokio::select! { accept_result = listener.accept() => ...`) is still running and the
socket file still exists.
The loop only stops at `cli/src/native/daemon.rs:330` (`_ = close_notify.notified() => { ... break; }`),
and the socket,
pid and version files are removed after `run_socket_server` returns,
at `cli/src/native/daemon.rs:148-155`.

### The CLI treats an accepting socket as a live daemon

`cli/src/connection.rs:404-409`:

```rust
pub fn daemon_ready(session: &str) -> bool {
    #[cfg(unix)]
    {
        let socket_path = get_socket_path(session);
        UnixStream::connect(&socket_path).is_ok()
    }
```

`cli/src/connection.rs:804` (`if daemon_ready(session) {`) in `ensure_daemon` reuses that daemon,
because its `.version` file still matches and the config check passes.
The comment at `cli/src/connection.rs:795-802` says socket connectivity is the sole liveness check.

### The lost request is classified as not safe to resend

The request is written and the daemon exits before replying,
so `exchange` reads an empty line.
`cli/src/connection.rs:1202-1207`:

```rust
reader
    .read_line(&mut response_line)
    .map_err(|e| SendError::OutcomeUnknown(format!("Failed to read: {}", e)))?;

serde_json::from_str(&response_line)
    .map_err(|e| SendError::OutcomeUnknown(format!("Invalid response: {}", e)))
```

`cli/src/connection.rs:1077-1087` (`Err(SendError::OutcomeUnknown(e)) if !read_only => ...`) returns the
`outcome_unknown` response with the "was not retried" suffix for any action not in `READ_ONLY_ACTIONS`
(`cli/src/connection.rs:1026-1045`:
`url`,
`title`,
`snapshot`,
`tab_list` and similar).
This is deliberate:
the changelog entry for #2085 says mutating commands must not be re-sent after bytes were written.

### Read-only commands recover, unless a launch option makes the CLI send a `launch` first

For a read-only action the EOF is retried:
the daemon is gone by then,
so the retry fails with `Failed to connect`,
and `send_command_with_respawn` (`cli/src/main.rs:2209-2221`) sees `daemon_unreachable`,
calls `ensure_daemon` and resends.

This host has `~/.agent-browser/config.json` with `{"args": "--ozone-platform=wayland"}`.
`should_send_local_launch_config` (`cli/src/main.rs:293-323`,
the `flags.args.is_some()` term at line 300) is then true,
so every invocation first sends a `launch` command through plain `send_command` (`cli/src/main.rs:2121`).
`launch` is not read-only,
so that first request gets the `outcome_unknown` treatment and the CLI prints the raw error and exits
(`cli/src/main.rs:2122-2131`),
before the real command is ever sent.
That is why even `tab list` and `get url` failed on this host.

### What the dying daemon does with the request

Traced with `strace -f -tt -e trace=accept4,unlink,exit_group,read,write -p <daemon pid>` on a disposable session
(times are 2026-10-20 17:01:01,
in seconds and milliseconds):

```text
.716  unlink(".../ab-strace.stream")      # close reply already written (daemon.rs:601-603)
.778  accept4(...) = 14                   # the next CLI command connects to the old daemon
.779  unlink(".../DevToolsActivePort")    # by daemon thread 1104229, Chrome launch setup
.781  pid 1104615 read(3, "\177ELF...")     # a new Chrome process starts loading its binary
.818  unlink(".../ab-strace.sock")        # run_socket_server has returned; cleanup (daemon.rs:148)
.818  unlink(".../ab-strace.pid")
.826  exit_group(0) of the new Chrome's children (pids 1104617 and up)
```

So the command is accepted and begins executing (an implicit browser launch),
but the daemon exits about 40 ms later
without answering.
The launch evidence is the process ids:
pid 1104615 is first seen at `.781`,
after the accept at `.778`,
and is a Chrome process (its first traced call reads an ELF header,
and it later unlinks `SingletonSocket` in a Chrome profile directory);
the daemon itself is pid 1104168 and `strace` was attached before the `close`.

### An earlier reading that was wrong

The first thought was that the daemon had already deleted its socket,
so the CLI would have seen `Failed to connect`.
The strace disproves that:
the connection is accepted at `.778` and the socket is unlinked at `.818`.
The race is a connection accepted by a daemon that is about to exit,
not a missing socket.

## Verification

Version under test:
agent-browser 0.39.1,
tag `v0.39.1`,
commit `1165952`.
All probes used disposable session names beginning `ab-` and closed them afterwards;
no other session was touched.

### Failing catalog (first command after `close`)

- Plain `close`,
  then `set viewport 390 950`:
  4 of 4 failed (loop of four fresh sessions).
- `close >/dev/null 2>&1`,
  then `set viewport`:
  3 of 3 failed.
- `close | head -1`,
  then `set viewport`:
  3 of 3 failed.
- `close` with output on the terminal,
  then `set viewport`:
  2 of 3 failed,
  1 passed.
- Caller delay between `close` and the next command of 0,
  10 and 30 ms:
  9 of 9 failed (three runs each).
- With the host `config.json` in place,
  ten runs of `tab list`,
  `get url`,
  `open about:blank` and `snapshot`:
  9 failed,
  1 passed.
- With `--config` pointing at a file containing `{}`:
  `set viewport`,
  `open about:blank` and `eval 1` failed.

### Working catalog

- Caller delay of 60,
  100 and 200 ms:
  9 of 9 passed (three runs each,
  one second pause between iterations).
- Waiting until `$XDG_RUNTIME_DIR/agent-browser/<session>.sock` no longer exists:
  6 of 6 passed.
- A fixed `sleep 0.2`:
  6 of 6 passed.
- With `--config` pointing at `{}`:
  `tab list` and `get url` passed (the daemon was respawned,
  with the notice `[agent-browser] launched browser`).

An earlier sweep with a different loop shape (fresh session per delay,
no pause between iterations)
showed no failures at any delay,
including 0.
It was not explained,
so treat the window as timing-sensitive:
the source gives 100 ms plus teardown,
and the measured boundary here is between 30 and 60 ms of caller delay,
which includes the new process start.

### Did the lost command leave a browser behind?

Three trials of `open`,
`close`,
`open about:blank` (failed with EOF),
then a four second wait:
zero processes under `~/.agent-browser/browsers/` and no `.pid` or `.sock` files for those sessions.
Whether the lost command's effect happened is unknown,
because the session that would show it no longer exists.

## Verified workarounds

### Wait for the old socket to disappear (recommended)

```sh
session=probe
agent-browser --session "${session}" close
while [ -S "${XDG_RUNTIME_DIR}/agent-browser/${session}.sock" ]; do sleep 0.02; done
agent-browser --session "${session}" set viewport 390 950
```

6 of 6 passed.
Tradeoffs:
it depends on the socket directory layout (`AGENT_BROWSER_SOCKET_DIR`,
then `XDG_RUNTIME_DIR/agent-browser`,
per `cli/src/connection.rs` `get_socket_dir`),
so it needs the same environment as the CLI;
and a wedged daemon would loop forever,
so bound it in real scripts.

### Sleep 200 ms after `close`

6 of 6 passed.
Tradeoff:
it is a time guess against a fixed 100 ms in the daemon plus scheduling,
so a loaded host could still land inside the window.

### Retry the first command once (the workaround the incident used)

Wrap the first command in try/catch and run it again on `Invalid response: EOF`.
Tradeoff:
the first attempt may have started work in the dying daemon (a Chrome launch was observed in the trace).
For `open <url>` of a page with server-side effects,
the request could be sent twice.
Retry only commands that are safe to repeat,
or combine this with the socket wait.

## What does not work

- Relying on the CLI's own retry for mutating commands:
  by design it never re-sends after bytes were written.
- Treating `close` as synchronous:
  it returns once the reply is written,
  not once the daemon has exited.
- Raising `AGENT_BROWSER_DEFAULT_TIMEOUT`:
  the failure takes milliseconds,
  not 30 seconds.
- Blaming the launch-option `config.json` as the cause:
  with `{}` the mutating commands still fail;
  the config only widens the failure to read-only commands.

## Not verified

- A fresh session name for the next command (it should avoid the old daemon entirely;
  not run).
- Whether the lost command ran to completion inside the dying daemon.
- macOS and Windows behavior;
  the daemon code path is shared but only this Linux host was exercised.

## Upstream filing decision

The repository's `.out-of-scope/` directory was listed and searched with
`rg -n -i "agent-browser|browser" .out-of-scope`
(exit code 1,
no match);
no exemption applies.
The upstream tracker already has this bug:
[issue 2134][issue-2134] (open,
2026-10-16) reports the same string and a 1 in 20 failure rate against a CDP Chrome,
and [PR 2138][pr-2138] (open,
mergeable,
approved by a collaborator on 2026-10-19) changes `cli/src/native/daemon.rs`
and `cli/tests/session_files_cli.rs` so the accept loop drops the listener before the `close` reply is sent.
[Issue 1837][issue-1837] (open) is the same window seen as `Failed to connect` on a `--headed` relaunch.

Nothing in this investigation is absent from those threads (cause,
window,
wait-for-pid workaround,
fix).
The one extra fact,
that a launch-option `config.json` turns the failure into one for read-only commands,
is a consequence of the same race and does not change the fix.
Post no comment and file no new issue.

1.  **Upstream fault?**
    Yes.
    The daemon acknowledges `close` while still accepting connections.
2.  **Can upstream fix it?**
    Yes,
    and PR 2138 does it.
3.  **Supported use case?**
    Yes.
    `close` followed by another command in the same session is ordinary use.
4.  **Contribution welcome?**
    No `CONTRIBUTING.md`,
    issue template or pull request template exists in the clone
    (`.github/` holds only `workflows/`);
    the repository `AGENTS.md` is guidance for agents working on the code,
    with no filing policy;
    a collaborator approved a community PR for this very bug.
    No AI-assistance ban was found.
5.  **Will they likely fix it?**
    Yes.
    A collaborator approved PR 2138 four days before this investigation.
6.  **Minimal fix prototyped?**
    Not by us:
    upstream's own PR 2138 carries the fix and a test.
    No new-issue draft exists,
    so there is nothing to keep in a fence.

[issue-2134]: https://github.com/vercel-labs/agent-browser/issues/2134
[issue-1837]: https://github.com/vercel-labs/agent-browser/issues/1837
[pr-2138]: https://github.com/vercel-labs/agent-browser/pull/2138
