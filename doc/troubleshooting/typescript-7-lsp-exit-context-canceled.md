# TypeScript 7.0.2's language server prints `context canceled` and exits with status 1 after the `exit` notification, even after `shutdown`

`tsc --lsp --stdio` from `typescript@7.0.2` treats the protocol's `exit` notification as a cancellation
and reports that cancellation as a failure:
one line on standard error and exit status 1.
A client that shuts the server down the way the protocol describes (`shutdown`, then `exit`) always gets both.
The IDE's Language module (`package/ide/app/src/language`) does exactly that at window close,
and `helix-lsp` logs every standard-error line of a server at ERROR
([`helix-lsp-transport-error-level-records.md`](helix-lsp-transport-error-level-records.md)),
so closing the IDE on a TypeScript project leaves an error in the application log.

The same text on another surface of the same program, the synchronous API,
is recorded in [`typescript-7-unstable-sync-readonly.md`](typescript-7-unstable-sync-readonly.md).
The cause there is a signal; the cause here is the `exit` notification.

## Symptom

On the server's standard error, as its last output:

```text
context canceled
```

Exit status 1.
Occasionally a second line precedes it, when a diagnostics publication was in flight:

```text
Error publishing diagnostics: context canceled
context canceled
```

In the application log:

```text
ERROR helix_lsp::transport: typescript-native err <- "context canceled\n"
```

Sequences that produce it:

- `shutdown`, its answer awaited, then `exit`.
- `shutdown` and `exit` sent back to back, which is what `helix-lsp`'s `force_shutdown` does.
- `exit` without `shutdown`.
- `SIGTERM`.

Sequences that end with status 0 and nothing on standard error:

- `shutdown`, its answer awaited, then standard input closed without `exit`.
- Standard input closed without `shutdown`.

The protocol asks for the opposite of the first case:
"The server should exit with success code 0 if the shutdown request has been received before;
otherwise with error code 1" ([`exit` in the specification][lsp-exit]).

[lsp-exit]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#exit

## Root cause

Source: `microsoft/TypeScript` at tag `v7.0.2` (`1e4744d68260a7cb91b62b12edc3f6a2187faaf1`), directory `tsc`.

The handler of `exit` returns the end-of-input value (`tsc/internal/lsp/server.go:1311`):

```go
// tsc/internal/lsp/server.go:1311
func (s *Server) handleExit(ctx context.Context, _ lsproto.NoParams) error {
	return io.EOF
}
```

The dispatch loop turns that value into a cancellation of its own context,
with no cause recorded (`tsc/internal/lsp/server.go:532` and `:554`):

```go
// tsc/internal/lsp/server.go:532
func (s *Server) dispatchLoop(ctx context.Context) error {
	ctx, lspExit := context.WithCancelCause(ctx)
	defer lspExit(nil)
	for {
		req, err := s.requestQueue.Get(ctx)
		if err != nil {
			return err
		}
```

```go
// tsc/internal/lsp/server.go:554
handleError := func(err error) {
	if errors.Is(err, context.Canceled) {
		if err := s.sendError(req.ID, lsproto.ErrorCodeRequestCancelled); err != nil {
			lspExit(err)
		}
	} else if errors.Is(err, io.EOF) {
		lspExit(nil)
	} else {
```

The next `Get` on the cancelled context returns `context.Canceled`,
because the queue returns the context's error, not its cause (`tsc/internal/lsp/dynamic_queue.go:50` and `:87`):

```go
// tsc/internal/lsp/dynamic_queue.go:87
func (q *dynamicQueue[T]) getReady(ctx context.Context) (*dynamicQueueState[T], error) {
	select {
	case state := <-q.ready:
		return state, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}
```

So the dispatch loop ends with `context.Canceled`, and that is what the group of loops reports.
`Run` lets only `io.EOF` through as a normal end (`tsc/internal/lsp/server.go:423`):

```go
// tsc/internal/lsp/server.go:423
func (s *Server) Run(ctx context.Context) error {
	g, ctx := errgroup.WithContext(ctx)
	s.backgroundCtx = ctx
	g.Go(func() error { return s.dispatchLoop(ctx) })
	g.Go(func() error { return s.writeLoop(ctx) })

	// Don't run readLoop in the group, as it blocks on stdin read and cannot be cancelled.
	readLoopErr := make(chan error, 1)
	g.Go(func() error {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-readLoopErr:
			return err
		}
	})
	go func() { readLoopErr <- s.readLoop(ctx) }()

	if err := g.Wait(); err != nil && !errors.Is(err, io.EOF) && ctx.Err() != nil {
		return err
	}
	return nil
}
```

`ctx` in the last condition is the group's own context,
which the group cancels as soon as one loop returns an error,
so `ctx.Err() != nil` holds whenever `err` is not nil.
The command prints whatever `Run` returns and exits with status 1 (`tsc/cmd/tsgo/lsp.go:67`):

```go
// tsc/cmd/tsgo/lsp.go:67
if err := s.Run(ctx); err != nil {
	fmt.Fprintln(os.Stderr, err)
	return 1
}
return 0
```

Closing standard input takes the other path:
the read loop returns the reader's `io.EOF` (`tsc/internal/lsp/server.go:466`),
the group reports `io.EOF`, `Run` returns nil, and the command exits with status 0.

### Readings that were wrong

- **The client sent `exit` before the answer to `shutdown`.**
  `helix-lsp`'s `force_shutdown` does send both back to back,
  but awaiting the answer first gives the same line and status in every run.
- **Confinement.**
  The line appears with the server inside bubblewrap (the IDE's lifecycle check)
  and outside it (the probe, which starts the server directly).
- **The Node launcher.**
  A server built from the tagged source and started directly behaves the same.

## Verification

Version under test:
`typescript@7.0.2` with `@typescript/typescript-linux-x64@7.0.2`, copied from this repository's package store;
source `microsoft/TypeScript` at `v7.0.2`.
On `main` at `ca197b7b8586900c3b75c0365fac31cfcaaacca7` (2026-10-05) `Run` and `handleExit` are unchanged
(`tsc/internal/lsp/server.go:895` and `:1836`),
and the command, now `tsc/cmd/tsc/lsp.go:71`, still prints the error and returns 1.

### Harness

[`typescript-7-lsp-exit-context-canceled-probe.mjs`](typescript-7-lsp-exit-context-canceled-probe.mjs)
starts the server on a disposable project, initializes it, waits for one hover answer,
ends it with one of six sequences, three times each,
and prints exit status and standard error.

```sh
# repository root
node doc/troubleshooting/typescript-7-lsp-exit-context-canceled-probe.mjs
```

Through the application, with the server confined:

```sh
# package/ide/app
mise run //package/ide/app:inspect:language-lifecycle
```

### Sequences that end cleanly

Status 0 and empty standard error in three of three runs each:

- `shutdown` answered, then standard input closed, no `exit`
- standard input closed without `shutdown`

### Sequences that print the line

Status 1 and `context canceled` in three of three runs each:

- `shutdown` answered, then `exit`
- `shutdown` and `exit` back to back
  (one run in a later repetition also printed `Error publishing diagnostics: context canceled` first)
- `exit` without `shutdown`
- `SIGTERM`

## Verified workarounds

The server is unchanged, and so is the line it prints.
Since 2026-10-06 the application logs that line at INFO instead of ERROR, with its text kept,
as every line a server writes to standard error
([`helix-lsp-transport-error-level-records.md`](helix-lsp-transport-error-level-records.md), "Verified workarounds");
the repository owner kept `shutdown` and `exit` as the way the worker ends this server.
No other handling is applied:

- **End the server by closing standard input after `shutdown`.**
  Verified with the probe: status 0, nothing printed.
  Tradeoff: `helix-lsp` at the pinned revision cannot do it.
  Its only way to close a server's standard input is to drop the `Client`,
  and the `Client` kills its child process first:
  the process handle is created with `kill_on_drop` (`helix-lsp/src/client.rs:238`)
  and is the field dropped before the sender that keeps standard input open (`helix-lsp/src/client.rs:59`).
  Using this sequence needs a change in `helix-lsp` or a transport of our own.
- **Run a server built with the change in "Upstream filing decision".**
  Verified with the probe against that build.
  Tradeoff: projects bring their own TypeScript, so the application cannot choose the build.

## What does not work

- **Waiting for the answer to `shutdown` before `exit`.**
  Same line, same status.
- **Sending only `shutdown` and then dropping the client.**
  Not measured in the application.
  By the source it replaces a clean exit with `SIGKILL` of the sandbox process for every server,
  and the server's own exit then depends on what its children do.
  It is a way to avoid the line by never asking for the exit.
- **Filtering the line out of the log.**
  The same text is what the server prints after `SIGTERM` and when its parent-process watchdog stops it
  (`tsc/cmd/tsgo/lsp.go:101`), so a filter would hide real, unrequested stops.
  The cleanup rule (`AGENTS.md`, code `CXL`) forbids it.

## Upstream filing decision

`.out-of-scope/` was checked:
`typescript-project-references.md` mentions the native compiler for project references only;
no entry covers the language server.

Tracker searches on 2026-10-05:

- `microsoft/TypeScript`, issues, `"context canceled"`:
  [#64242][ts-64242], closed on 2026-09-16 by [#64276][ts-64276].
  It is the synchronous API case.
  The fix touches `tsc/internal/api/server.go` only and treats a cancellation during an expected shutdown as success.
  The language server path was not changed.
- `microsoft/TypeScript`, issues,
  `lsp exit notification exit code` and `language server shutdown exit stderr`: nothing.
- `microsoft/typescript-go` (the closed staging repository), issues, `context canceled`:
  #4809, closed, about the parent-process watchdog; `exit notification` and `lsp exit code shutdown`: nothing.
- Pull requests in `microsoft/typescript-go` for `exit notification context canceled`: nothing.

No report of the `exit` notification case was found, so a new issue is the fitting artifact.

1.  Is it really upstream's fault?
    Yes. The protocol asks for status 0 after `shutdown` and `exit`,
    and a normal end is reported as an error.
2.  Can upstream fix it?
    Yes. Two small changes in one function.
3.  Are they supporting this use case?
    Yes. `--lsp --stdio` is the product's language server, and `shutdown` then `exit` is the protocol's shutdown.
4.  Would the repo welcome our contribution?
    `CONTRIBUTING.md` ("Use of AI Assistance") has no objection to AI coding tools,
    requires disclosure in a pull request, forbids bulk agent-driven contributions and automated comments,
    and accepts bug-fix pull requests only for issues labelled "help wanted".
    A single issue filed by a human, with disclosure, fits; a pull request needs an approved issue first.
5.  Will they likely fix it?
    Likely: the same message on the API surface was fixed within days of [#64242][ts-64242].
6.  Have we prototyped a minimal fix compatible with their architecture?
    Yes.

Prototype:
[`typescript-7-lsp-exit-context-canceled.patch`](typescript-7-lsp-exit-context-canceled.patch),
applied to a fresh clone at `v7.0.2`
(`~/temp/agent/upstream-prototype.zBCA77Xl/TypeScript`, origin `https://github.com/microsoft/TypeScript.git`).
The `exit` notification stops the dispatch loop with `io.EOF` as the recorded cause,
and the loop returns `io.EOF` for that cause, which `Run` already treats as a normal end.
Other cancellations are returned as before.

Verification:
the server was built from the clone with Go 1.27.1 before and after the change,
in a container bounded to 8 GiB, 6 CPUs, and 1024 processes,
with no credentials and with network access only for Go's pinned modules,
and the probe ran against both builds (`TSGO_BINARY`).

- Unmodified build: identical to the published package in all six sequences.
- Changed build: status 0 and empty standard error for
  `shutdown` then `exit`, both back to back, and `exit` without `shutdown`;
  the two standard-input sequences stay clean;
  `SIGTERM` still gives status 1 and `context canceled`.

One open point for upstream:
with the change, `exit` without `shutdown` also ends with status 0, where the protocol asks for 1.
The draft says so.

Decision: fileable after a human has read the trace and rerun the probe. Not filed.

~~~md
Title: Language server prints `context canceled` and exits with status 1 after `shutdown` + `exit`

### Search terms

context canceled, lsp, exit notification, shutdown, exit code

### Version

typescript@7.0.2 (also `main` at ca197b7b: `Run` and `handleExit` are unchanged)

### Steps

1.  Start `tsc --lsp --stdio`, send `initialize` and `initialized`.
2.  Send `shutdown`, wait for its response.
3.  Send the `exit` notification.

### Actual

The process prints `context canceled` on stderr and exits with status 1.
The same happens when `exit` is sent without `shutdown`.
Closing stdin instead of sending `exit` gives status 0 and no output.

### Expected

Status 0 and no output after `shutdown` + `exit`
(LSP: "The server should exit with success code 0 if the shutdown request has been received before").

### Cause

`handleExit` returns `io.EOF`; `dispatchLoop` handles it with `lspExit(nil)`, so the next
`requestQueue.Get(ctx)` returns `context.Canceled`, which is what the errgroup reports.
`Run` only lets `io.EOF` through (`!errors.Is(err, io.EOF) && ctx.Err() != nil`, where `ctx` is the
errgroup's context and is always cancelled at that point), and `cmd/tsgo/lsp.go` prints the error and returns 1.

### Possible fix

```diff
 		req, err := s.requestQueue.Get(ctx)
 		if err != nil {
+			if errors.Is(context.Cause(ctx), io.EOF) {
+				return io.EOF
+			}
 			return err
 		}
...
 			} else if errors.Is(err, io.EOF) {
-				lspExit(nil)
+				lspExit(io.EOF)
```

With this, `exit` without a prior `shutdown` also exits 0; tracking whether `shutdown` was received
would be needed to return 1 there. #64276 fixed the same message for the sync API server.

### Disclosure

This report was prepared with AI assistance. The trace was checked against the v7.0.2 tag, and the change was
verified by building the server before and after it and running a stdio probe over six shutdown sequences.
A human has reviewed: (to be filled in before filing).
~~~

[ts-64242]: https://github.com/microsoft/TypeScript/issues/64242
[ts-64276]: https://github.com/microsoft/TypeScript/pull/64276

## Evidence

Measured on 2026-10-05.

- Probe against the published package:
  `~/temp/agent/ide-language-verify-tsgo-vcMgQY/results.json`.
- Prototype builds, probe logs, and diffs:
  `~/temp/agent/upstream-prototype.zBCA77Xl/`
  (`probe-tsgo-unmodified.log`, `probe-tsgo-prototype-refined.log`, `prototype-refined.diff`).
- The application's lifecycle check before any change:
  `~/temp/agent/ide-language-lifecycle-RWM9sQ/results/typescript.quiet.log.txt`.
