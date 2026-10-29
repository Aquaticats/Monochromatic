# helix-lsp at `ba40e54` logs server stderr lines, the end of server stderr, and error responses at ERROR, so an embedder's log shows errors for healthy servers

`helix-lsp` at revision `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc` writes three kinds of record at `error!`
that do not report a failure of the client:
every line a language server prints on standard error,
the end of that stream when the server process ends,
and every JSON-RPC error response.
The IDE's Language module (`package/ide/app/src/language`) embeds `helix-lsp`
and forwards its `log` records into the application log,
so a clean open, request, close, and quit leaves ERROR-level records.
The repository rule on quiet cleanup (`AGENTS.md`, code `CXL`) forbids that.

Related:
[`helix-lsp-embedding-roots-and-stop.md`](helix-lsp-embedding-roots-and-stop.md) covers two other embedding behaviors,
[`typescript-7-lsp-exit-context-canceled.md`](typescript-7-lsp-exit-context-canceled.md)
covers one server line that arrives through this path,
and [`rust-analyzer-notify-missing-user-config.md`](rust-analyzer-notify-missing-user-config.md) covers another.

## Symptom

Records in the application log,
all with the target `helix_lsp::transport`.

### A server wrote a line to standard error

```text
ERROR helix_lsp::transport: rust-analyzer err <- "2026-10-05T14:50:17.468643919-04:00  WARN notify error: No path was found.\n"
ERROR helix_lsp::transport: rust-analyzer err <- "2026-10-05T14:50:20.336538348-04:00  WARN overly long loop turn took 140.733818ms: ..."
ERROR helix_lsp::transport: typescript-native err <- "context canceled\n"
```

The level is ERROR whatever the server meant.
Both rust-analyzer lines carry rust-analyzer's own level, `WARN`.

### A server process ended

```text
ERROR helix_lsp::transport: rust-analyzer err: <- StreamClosed
ERROR helix_lsp::transport: typescript-native err: <- StreamClosed
ERROR helix_lsp::transport: scripted-ls err: <- StreamClosed
```

One record per process,
after a requested `shutdown` and `exit` as well as after a crash.

### A server answered a request with an error

```text
ERROR helix_lsp::transport: rust-analyzer <- ServerError(-32801): content modified
```

`-32801` is the protocol's `ContentModified`,
which rust-analyzer returns while it is still loading the project or after the document changed.
The Language module handles that answer by asking again.

## Root cause

### Standard-error lines

One task per server reads the server's standard error line by line
and logs each line at `error!` (`helix-lsp/src/transport.rs:159`):

```rust
// helix-lsp/src/transport.rs:159
async fn recv_server_error(
    err: &mut (impl AsyncBufRead + Unpin + Send),
    buffer: &mut String,
    language_server_name: &str,
) -> Result<()> {
    buffer.truncate(0);
    if err.read_line(buffer).await? == 0 {
        return Err(Error::StreamClosed);
    };
    error!("{language_server_name} err <- {buffer:?}");

    Ok(())
}
```

### The end of standard error

When `read_line` returns zero bytes, `recv_server_error` returns `Error::StreamClosed`,
and the task that called it logs that value at `error!` before it ends (`helix-lsp/src/transport.rs:355`):

```rust
// helix-lsp/src/transport.rs:355
async fn err(transport: Arc<Self>, mut server_stderr: BufReader<ChildStderr>) {
    let mut recv_buffer = String::new();
    loop {
        match Self::recv_server_error(&mut server_stderr, &mut recv_buffer, &transport.name)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                error!("{} err: <- {err:?}", transport.name);
                break;
            }
        }
    }
}
```

A pipe reaches its end when the process that wrote to it exits,
so this record is written for every server process that ends while the client's runtime is still running.
The task that reads the server's responses treats the same value as a normal end and stays silent
(`helix-lsp/src/transport.rs:315`):

```rust
// helix-lsp/src/transport.rs:315
Err(err) => {
    if !matches!(err, Error::StreamClosed) {
        error!(
            "Exiting {} after unexpected error: {err:?}",
            &transport.name
        );
    }
```

That silent form was introduced for the response reader by Helix pull request #9332;
the standard-error reader kept the unconditional `error!`.

### Error responses

Every response that carries a JSON-RPC error is logged at `error!` before it is handed to the caller
(`helix-lsp/src/transport.rs:260`):

```rust
// helix-lsp/src/transport.rs:260
let (id, result) = match output {
    jsonrpc::Output::Success(jsonrpc::Success { id, result, .. }) => (id, Ok(result)),
    jsonrpc::Output::Failure(jsonrpc::Failure { id, error, .. }) => {
        error!("{language_server_name} <- {error}");
        (id, Err(error.into()))
    }
};
```

### How the records reach the application log

`package/ide/app/src/native.rs` starts logging with `tracing_subscriber::fmt()...init()`,
which also installs the bridge from the `log` crate that `helix-lsp` uses.
The filter keeps `helix-lsp` at warnings and errors
(`HELIX_LOG_DIRECTIVE` in `package/ide/app/src/language.rs`),
because `helix-lsp` logs every protocol message in full at `info`.
All three records are errors to that filter.

### A reading that was wrong

The first reading was that `StreamClosed` is an ordering fault in the Language worker:
the transport dropped before the server process had ended,
or `exit` sent before the answer to `shutdown`.
The evidence against it:

- The record appears when the worker sends `shutdown` and `exit`,
  waits until every server has ended by itself (`language servers were asked to stop still_running=0`),
  and only then drops the registry.
- The source shows no condition on the record: the end of the stream is always logged.
- With the worker's wait removed in a disposable copy,
  the lifetime test fails for another reason (the server never receives `shutdown`),
  so the wait is not what produces the record.

## Verification

Version under test:
`helix-lsp` from `https://github.com/helix-editor/helix` at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`,
which was also the head of `master` on 2026-10-05.

### Harness

Container suite, scripted server, no real language server:

```sh
# package/ide/app
mise run //package/ide/app:test:language quiet::
```

`tests/language/quiet.rs` installs the application's log setup with an in-memory writer,
runs open, one hover, close, and shutdown,
and collects the ERROR-level records.
Since 2026-10-06 the scripted server also writes `context canceled` to standard error at shutdown
and answers the first hover with `-32801`, so all three record shapes occur in that lifetime,
and the test runs with the bridge described in "Verified workarounds".

Host, real servers confined by bubblewrap, disposable projects:

```sh
# package/ide/app
mise run //package/ide/app:inspect:language-lifecycle
```

### Records that appear

Scripted server, clean lifetime:

- `scripted-ls err: <- StreamClosed`, once.

TypeScript 7.0.2 server, open, one hover, close, quit:

- `typescript-native err <- "context canceled\n"`
- `typescript-native err: <- StreamClosed`

rust-analyzer `1.100.0-nightly (1303417 2026-09-21)`, open, one hover, close, quit:

- `rust-analyzer err <- "... WARN notify error: No path was found.\n"`, twice
- `rust-analyzer err <- "... WARN overly long loop turn took ..."`, timing dependent
- `rust-analyzer <- ServerError(-32801): content modified`, timing dependent
- `rust-analyzer err: <- StreamClosed`

### Records that do not appear

- Nothing at ERROR from `ide_app` targets in any of these runs.
- No `Exiting ... after unexpected error`, which is the response reader's record for a real transport failure.
- With the prototype in "Upstream filing decision" applied, the scripted lifetime logs nothing at ERROR.

## Verified workarounds

### Re-labelling in the application (adopted)

The repository owner chose this handling on 2026-10-06 ("Re-label in the IDE").
`package/ide/app/src/logging/relabel.rs` installs the `log` crate's logger for the process.
It lowers exactly the three shapes, when they come from the `helix_lsp::transport` target at ERROR,
and passes everything else, at every level, through `tracing_log::format_trace` unchanged:

- a standard-error line, `{name} err <- "{line}"`, to INFO,
  the level at which `helix-lsp` logs every other message it reads from a server
  (`helix-lsp/src/transport.rs:147`);
- the end of standard error, `{name} err: <- StreamClosed`, to DEBUG;
- an error answer with code `-32801` (content modified) or `-32800` (request cancelled), to DEBUG,
  because the worker asks again for these and shows nothing (`src/language/request/answer.rs`).

Text, target, module, file, and line stay as `helix-lsp` wrote them.
An error answer with any other code, a failure to read or write a server's streams,
and every record from another `helix-lsp` module keep ERROR.
The application's default level is WARN and keeps `helix_lsp` at warnings unless `RUST_LOG` names it,
so `RUST_LOG=helix_lsp=debug` shows the re-labelled records together with the protocol messages.

Verification:
the strict test `quiet::clean_lifetime_logs_no_error_level_record` is enforced and asserts each shape at its new level;
`quiet::unknown_helix_error_records_keep_their_level` asserts that a refused `initialize` still logs two ERROR records;
`inspect:language-lifecycle-guards` removes each re-labelling, and widens it to every transport ERROR,
and observes the named test fail each time and pass when restored
(`~/temp/agent/ide-language-lifecycle-guard-Z50y1S/results.json`, 2026-10-06, 20 runs as expected).

Tradeoffs:

- It matches `helix-lsp`'s message text at the pinned revision.
  A changed format string on a `helix-lsp` update makes that record ERROR again, which the enforced test reports;
  it cannot hide a record, because an unrecognized record keeps its level.
- A server's standard error is no longer visible at the default level.
  Since 2026-10-06 the bridge keeps each server's last 8 lines anyway (`src/logging/stderr_tail.rs`):
  a server that crashes, or ends during its start, leaves the worker's warning `language server process ended`
  with them in its `stderr_tail` field,
  and a server stopped for not answering `initialize` in time leaves them in a warning after the start-deadline error.
  Every line a server writes appears with `RUST_LOG=helix_lsp=info`.
- The upstream fix for the end-of-stream record ("Upstream filing decision") remains worth filing;
  the bridge does not depend on it.

### Prototype of the upstream fix

One change is verified as a prototype:
building against a `helix-lsp` whose standard-error reader stays silent at the end of the stream
(the diff is in "Upstream filing decision").
With it the strict test `quiet::clean_lifetime_logs_no_error_level_record` passed before the bridge existed.
Tradeoffs:
it needs a patched copy of a pinned dependency (a `[patch]` section or a fork) until upstream changes,
and it removes only the end-of-stream record;
standard-error lines and error responses stay at ERROR.

## What does not work

- **Another shutdown order in the worker:**
  waiting for the answer to `shutdown` before `exit`,
  or waiting longer for the process,
  does not change which task logs the end of the stream.
- **Dropping the registry without waiting for the processes:**
  the record then depends on whether the runtime ends before the reader task runs.
  That hides the record by timing,
  kills servers that would have ended by themselves,
  and makes the scripted lifetime test fail because `shutdown` never reaches the server.
- **Giving the server another standard error through the launch command:**
  `helix-lsp` always pipes the child's standard error (`helix-lsp/src/client.rs:235`),
  so a redirect inside the launch only moves the end of that pipe to the start of the session.
- **Turning the `helix_lsp::transport` target off in the filter:**
  it also hides `Exiting ... after unexpected error` and `Failed to send notification`,
  which report real failures, and it discards server standard error,
  where servers print panics.
  It is the filtering the cleanup rule forbids.

## Upstream filing decision

`.out-of-scope/` was checked: no entry covers Helix or `helix-lsp`.

Tracker searches on 2026-10-05 in `helix-editor/helix`:
issues for `StreamClosed`, `"err <-"`, `stderr error level`, `language server stderr`,
and `log level lsp error helix.log`;
pull requests for `StreamClosed`, `stderr`, and `stderr log level language server`.
The issue search for `StreamClosed` returned results (a positive control), none about log levels;
`language server stderr` returned #6334, a SourceKit crash;
the pull-request search for `StreamClosed` returned #9332,
which introduced the silent end-of-stream handling in the response reader.
No report or change about the level of these records was found.

The three records are judged separately.

### The end-of-stream record

1.  Is it really upstream's fault?
    Yes, as an inconsistency:
    the response reader treats `StreamClosed` as a normal end and the standard-error reader logs it as an error,
    for the same event.
2.  Can upstream fix it?
    Yes. One match arm.
3.  Are they supporting this use case?
    Yes. It happens inside Helix as well, after `:lsp-stop`, `:lsp-restart`, and any server exit;
    embedding is not required to see it.
4.  Would the repo welcome our contribution?
    `docs/CONTRIBUTING.md` welcomes contributions of any size and states no rule about AI assistance;
    `.github/ISSUE_TEMPLATE/bug_report.yaml` and `enhancement.md` state none either.
    Tracker searches for `LLM generated` and `AI generated pull requests policy` found no stated ban.
    A filing must still disclose assistance and name what a human verified.
5.  Will they likely fix it?
    No signal against it: no won't-fix, no stated non-goal, no declined request.
6.  Have we prototyped a minimal fix compatible with their architecture?
    Yes.

Prototype, applied to a fresh clone at the pinned revision
(`~/temp/agent/upstream-prototype.pGRX48oY/helix`, origin `https://github.com/helix-editor/helix.git`):

```diff
--- a/helix-lsp/src/transport.rs
+++ b/helix-lsp/src/transport.rs
@@ -359,6 +359,7 @@ async fn err(transport: Arc<Self>, mut server_stderr: BufReader<ChildStderr>) {
                 .await
             {
                 Ok(_) => {}
+                Err(Error::StreamClosed) => break,
                 Err(err) => {
                     error!("{} err: <- {err:?}", transport.name);
                     break;
```

Verification command, with a disposable target directory, Cargo home copy, and the clone:

```sh
# package/ide/app
mise run //package/ide/app:inspect:language-lifecycle-guards "$CACHE" "$CARGO" "$HELIX" helix
```

Note, 2026-10-06: this `helix` part was removed from `inspect:language-lifecycle-guards`
when the application started re-labelling the record, because the strict test now passes with or without the change.
The command runs as written on `main` at commit `45db45d02`;
to reproduce the prototype result, check out `package/ide/app` at that commit.

Result on 2026-10-05 (`~/temp/agent/ide-language-lifecycle-guard-8DAtvs/results.json`):
built against the unmodified clone, the strict test fails with `scripted-ls err: <- StreamClosed`;
with the change it passes, and the enforced test passes;
with the change reverted it fails again.

Decision: fileable after a human has read the trace and rerun the command. Not filed.

~~~md
Title: helix-lsp: stderr reader logs `StreamClosed` at error level on every language server exit

Labels: C-bug, A-language-server

### Summary

When a language server process exits, `Transport::err` logs `<name> err: <- StreamClosed` with `error!`.
This happens for every exit, including after `:lsp-stop`, `:lsp-restart`, and a normal `shutdown`/`exit`.
`Transport::recv` already treats the same value as a normal end of stream and logs nothing for it.

### Source

`helix-lsp/src/transport.rs` at ba40e547426b0f9896c8bdc699a4ab11f2b37dbc:

- `recv_server_error` (line 159) returns `Err(Error::StreamClosed)` when `read_line` returns 0.
- `err` (line 355) logs every `Err` with `error!("{} err: <- {err:?}", transport.name)`.
- `recv` (line 315) skips the log with `if !matches!(err, Error::StreamClosed)`.

### Reproduction

Start any language server, stop it (`:lsp-stop`), and open the log (`:log-open`):

```
[ERROR] rust-analyzer err: <- StreamClosed
```

### Suggested fix

Mirror `recv` in `err`:

```diff
                 Ok(_) => {}
+                Err(Error::StreamClosed) => break,
                 Err(err) => {
                     error!("{} err: <- {err:?}", transport.name);
                     break;
```

### Disclosure

This report was prepared with AI assistance. The source trace and the patch were checked against the pinned
revision by building an embedder against a patched clone: a test that fails on the record without the change
passes with it and fails again when the change is reverted. A human has reviewed: (to be filled in before filing).
~~~

### Standard-error lines at ERROR

1.  Is it really upstream's fault?
    No.
    Helix's default log level is `Warn` (`helix-term/src/main.rs:9`),
    so logging standard error at `error!` is what makes a failing server's output visible in the editor's log
    without a verbosity flag.
    The level is a choice that fits the editor.
2.  Can upstream fix it? Not applicable after constraint 1.
3.  Are they supporting this use case? Embedding `helix-lsp` outside Helix is not a documented use.
4.  Would the repo welcome our contribution? Not evaluated.
5.  Will they likely fix it? Not evaluated.
6.  Prototype: none.

Decision: do not file.

### Error responses at ERROR

1.  Is it really upstream's fault?
    No.
    Recording every failed request at the transport is a consistent choice for the editor;
    the source makes no distinction between error codes.
    An embedder that reports every failed request itself sees the record as a duplicate,
    which is a difference in needs, not a defect.
2.  Can upstream fix it? Not applicable after constraint 1.
3.  Are they supporting this use case? Not a documented use.
4.  Would the repo welcome our contribution? Not evaluated.
5.  Will they likely fix it? Not evaluated.
6.  Prototype: none.

Decision: do not file.

~~~md
Title: (do not file as-is) helix-lsp: server stderr lines and JSON-RPC error responses are logged at error level

Both levels suit Helix. Recorded only so a future embedder-support discussion has a source trace:
- helix-lsp/src/transport.rs:168 logs each stderr line with `error!`, whatever level the server printed.
- helix-lsp/src/transport.rs:263 logs each error response with `error!`, including ContentModified (-32801).
~~~

## Evidence

Measured on 2026-10-05 in the worktree `.claude/worktrees/ide-language-verify`.

- Host lifecycle check before any change:
  `~/temp/agent/ide-language-lifecycle-RWM9sQ/results/` (`*.quiet.log.txt` hold the full application logs).
- Guard controls for the lifetime tests:
  `~/temp/agent/ide-language-lifecycle-guard-SPKETW/results.json`.
- Prototype control:
  `~/temp/agent/ide-language-lifecycle-guard-8DAtvs/results.json`.
