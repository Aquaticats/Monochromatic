# Depot sandbox SDK beta.5 rejects a bare command that succeeds by absolute path

## Symptom

On 2026-10-15,
`@depot/sandbox` version `0.1.0-beta.5` successfully created a standalone sandbox,
but `runCommand({ cmd: 'uname', args: ['--all'] })` failed with:

```text
execute command: execute: stream error: unknown: execute sxq33pxvmxmlh6q command:
start exec: exec: "uname": executable file not found in $PATH
```

The `finally` block stopped that sandbox successfully.

## Root cause boundary

The server's command lookup failed;
the server-side environment and lookup implementation were not inspected.
This is not proof that the executable is absent from the image.
The absolute-path probe subsequently succeeded.

The cloned [SDK source](https://github.com/depot/sandbox-sdk),
revision `e890dc866839d1636b5919f2c5f45200656852ea`,
passes the command and environment to the server in `src/sandbox.ts:326`:

```typescript
// src/sandbox.ts:326, excerpt
const stream = this.client.rpc.runCommand({
  sandbox: {selector: {case: 'id', value: this.sandboxId}},
  cmd: opts.cmd,
  args: opts.args ?? [],
  cwd: opts.cwd,
  env: opts.env ?? {},
```

This source establishes the client boundary,
not why the server could not resolve the bare name.
The [SDK reference](https://depot.dev/docs/api/sandbox-sdk-reference)
permits absolute executable paths and documents direct execution without shell expansion.

## Verification and workaround

The probes used the existing Depot login without creating or printing a token.
Each requested 2 vCPUs,
4,096 MB RAM,
100 GB disk,
and a three-minute lifetime.
The response allocated 150 GB disk.

- Bare `uname` failed in sandbox `sxq33pxvmxmlh6q`.
- `/usr/bin/uname` succeeded in sandbox `sbgtwn2zcft7s58`,
  printed its Linux kernel and architecture,
  and returned zero.
- Both sandboxes reached `finished` after `stop({ blocking: true })`.
- The successful create/execute/stop sequence took 4.396 seconds locally.
  This is one observation,
  not a latency guarantee.

The verified invocation change is:

```diff
// caller of sandbox.runCommand
- { cmd: 'uname', args: ['--all'] }
+ { cmd: '/usr/bin/uname', args: ['--all'] }
```

Absolute paths depend on the selected image's layout.
An explicitly supplied `PATH` has not been tested.
Do not treat a bare command failure as proof that the entire sandbox service is unavailable.

## Upstream filing decision

No matching exemption appeared in `.out-of-scope/`.
`gh search issues PATH --repo depot/sandbox-sdk` returned no results.
That narrow search does not establish the absence of related reports.

1.  Upstream fault is unproven because the server environment is unknown.
2.  Fixability cannot be assessed without the deciding server path.
3.  Direct commands are a documented SDK use case.
4.  Contribution policy has not been assessed because no actionable cause is established.
5.  Maintainer intent about this behavior is unknown.
6.  No upstream patch was prototyped;
    the absolute-path caller change is the verified workaround.

There is no actionable upstream filing to add at this stage.
No external report was sent.
