# Depot Sandbox SDK beta.5 can leave a command client alive after completed work

## Symptom

During the standalone linker experiment on 2026-10-16 UTC,
Node.js 26.12.0 processes using `@depot/sandbox` 0.1.0-beta.5 remained alive after their awaited remote commands
reported exit status zero.
The provisioning client remained alive for more than six minutes after its command sequence completed.
The sandbox itself intentionally remained available for subsequent work.

A bounded child-process probe reproduced the symptom with two sequential `/usr/bin/uname --all` commands.
Both command results and a final application-completed marker reached stdout.
The parent then terminated the child after 10,013.920 milliseconds with:

```text
spawnSync /var/home/user/.local/share/mise/installs/node/26.12.0/bin/node ETIMEDOUT
```

The same two-command probe later exited normally in 1,305.656 milliseconds.
This is an intermittent client-lifecycle observation,
not a remote-command failure or a proven billing defect.

## Source boundary and unresolved cause

The inspected [SDK source](https://github.com/depot/sandbox-sdk),
revision `e890dc866839d1636b5919f2c5f45200656852ea`,
creates an HTTP/2 transport in `src/client.ts:22`:

```typescript
// src/client.ts:22, excerpt
const transport = createConnectTransport({
  baseUrl: endpoint,
  httpVersion: '2',
```

The installed distribution has the same transport selection.
The source's `src/sandbox.ts:623` drains command events,
and its `finally` block at line 652 already requests iterator cleanup:

```typescript
// src/sandbox.ts:662, excerpt
if (typeof iterator.return === 'function') {
  try {
    await iterator.return()
```

Consequently,
an absent `iterator.return()` is not the established cause.
The SDK,
transport,
Node runtime,
and server stream lifecycle have not been isolated from each other.
The failing probe's active-resource names included `TCPSocketWrap`,
but that alone does not identify the owner or prove a leaked connection.

## Verification

The disposable harness is retained under `~/temp/agent/depot-sandbox-trial/linker.ts`;
its `linker:lifecycle` mise task starts each child with a ten-second bound.
That bound tests process termination behavior;
it is not a timeout around a production test suite.

Working controls include:

- Reading `/etc/os-release` through `sandbox.fs()`.
- Writing a five-byte file inside the disposable sandbox.
- Executing one command and draining its output.
- A later execution of the unchanged two-command probe.

The failing control executes two commands sequentially,
drains each `logs()` iterator,
awaits each `wait()`,
checks exit status zero,
then prints the final marker.
The remote commands completed at `01:43:49.536Z` and `01:43:49.837Z`;
the local child still exceeded its bound.

## Verified containment and tradeoffs

The single-purpose trial process can terminate explicitly after all command waits,
artifact writes,
and cleanup operations have completed:

```typescript
// Single-purpose trial worker, after all awaited work and the final output marker.
process.exit(0);
```

The explicit-exit probe completed in 1,402.663 milliseconds with both remote statuses zero.
This terminates the local worker,
not the remote sandbox.
It would discard unrelated pending asynchronous work in a shared process,
so it is not an SDK lifecycle fix or a general application recommendation.
The trial also stopped already-completed local clients by their inspected process IDs.

## What does not establish a fix

A later normal exit does not explain the earlier timeout.
The source already contains iterator cleanup,
so adding that same cleanup cannot be justified from this symptom.
Remote exit status zero does not establish that the local client has exited.

## Upstream filing decision

No matching exemption appeared among the files in `.out-of-scope/`.
Issue and pull-request searches for `exit OR hang OR stream` in `depot/sandbox-sdk` returned no results;
these searches do not establish that no related transport issue exists elsewhere.

1.  Upstream ownership is unresolved across the SDK,
    transport,
    runtime,
    and server.
2.  The deciding lifecycle path has not been isolated,
    so a compatible source fix has not been established.
3.  Running commands and streaming output are documented SDK use cases.
4.  The repository README was inspected;
    no dedicated contribution or issue-template file appeared in the scoped file inventory.
5.  The tracker searches found no maintainer decision about this symptom.
6.  No upstream patch was prototyped because the fault has not been assigned to a deciding source path.

No upstream report was sent.
The [repository issue](https://github.com/Aquaticats/Monochromatic/issues/1051)
records the observed failure and remaining diagnosis work.
There is no source-grounded upstream filing draft to publish as-is.
