# wg-quicker 0.0.1 down refused its watcher after a Node 26.5.0 to 26.7.0 upgrade

## Symptom

`wg-quicker down mx-que-mx1` removed owned bypass routes and rules,
but retained ownership state and left the WireGuard link present:

```text
AggregateError: Application-bypass cleanup failed for mx-que-mx1; ownership state retained.
BypassRouteError: Refusing to signal PID 141663 because command is not bypass watcher for
/run/wg-quicker/interface-b4b65a09f0dece795f8437488962ae76.json.
```

A following `wg-quicker up mx-que-mx1` then failed because teardown had not deleted the link:

```text
CommandError: Command failed (0): ip link show dev mx-que-mx1
`mx-que-mx1' already exists
```

## Root cause

The detached watcher started under the mise-managed Node 26.5.0 executable.
The later `down` process ran under Node 26.7.0.
Live process inspection showed that the watcher script and state path were exact,
but its executable argument named the 26.5.0 installation:

```text
/var/home/.../mise/installs/node/26.5.0/bin/node \
  /var/home/.../package/wg/cli-quicker/dist/final/node/bypass-watch.mjs \
  /run/wg-quicker/interface-b4b65a09f0dece795f8437488962ae76.json
```

Before commit `d96581cdd`,
`package/wg/cli-quicker/src/tunnel-bypass-watch-service.ts:101` at commit `0a42e826e`
compared that persisted process against the current CLI's `process.execPath`:

```ts
if (!processCommandMatches({
  identity: live,
  expected: [
    process.execPath,
    WATCHER_PATH,
    statePath,
  ],
},)) {
```

A mise runtime update changes `process.execPath` without changing the already-running process.
The comparison therefore rejected a valid watcher before signaling it.
Cleanup retained state intentionally because signaling an unverified PID would be unsafe.

PID reuse was not the cause.
The sidecar PID and kernel start time matched the live process,
and the live command still named the expected watcher script and state path.
The watcher had not rewritten its arguments.

## Fix

`package/wg/cli-quicker/src/linux-process-identity.ts:161` now separates executable identity from stable process
arguments.
It requires one nonempty executable argument and then compares every remaining argument exactly:

```ts
export function processArgumentsMatch(
  {
    identity,
    expected,
  }: {
    readonly identity: LinuxProcessIdentity;
    readonly expected: readonly string[];
  },
): boolean {
  const actualLength = identity
    .commandLine
    .length;
  const expectedLength = expected.length + 1;
  if (actualLength !== expectedLength)
    return false;
  const [executable,] = identity.commandLine;
  if ((executable === undefined) || (executable === ''))
    return false;
  return expected.every(function sameArgument(
    value,
    index,
  ): boolean {
    return identity.commandLine[index + 1] === value;
  },);
}
```

`package/wg/cli-quicker/src/tunnel-bypass-watch-service.ts:94` retains the other process-identity gates,
then requires the exact watcher script and state arguments:

```ts
const live = await readLinuxProcessIdentity({ pid: identity.pid, },);
if ((live === PROCESS_ABSENT)
  || (live.startTime !== identity.startTime)
  || (live.state === 'Z')) {
  return false;
}
if (!processArgumentsMatch({
  identity: live,
  expected: [
    WATCHER_PATH,
    statePath,
  ],
},)) {
```

`stopBypassWatcher` also checks the sidecar owner token against persisted tunnel state before this process check at
`package/wg/cli-quicker/src/tunnel-bypass-watch-service.ts:311`.
The incident host's `/run/wg-quicker` directory was mode `0700` and owned by `root:root`;
watcher and tunnel state files were mode `0600` and owned by `root:root`.
Ignoring only the versioned executable path does not relax script,
state,
owner,
PID,
start-time,
or zombie checks.

## Verification

### Red and green integration reproduction

`package/wg/cli-quicker/src/tunnel-bypass.integration.test.ts:125` starts the built watcher through an alternate Node
symlink in a disposable network namespace.
The test rewrites the sidecar to the incident's legacy `ownerId`,
`pid`,
and `startTime` shape before teardown.

Run:

```console
mise run //package/wg/cli-quicker:test:integration:bypass
```

Before the fix,
the test failed with the same `Refusing to signal PID ... because command is not bypass watcher` diagnostic.
After the fix,
it printed:

```text
wg-quicker bypass integration passed
```

The unit catalog in `package/wg/cli-quicker/src/linux-process-identity.unit.test.ts` verifies these cases:

- exact watcher arguments under prior and current executable paths are accepted;
- missing and empty executable arguments are rejected;
- changed watcher script and state arguments are rejected.

### Package checks

These commands passed after the final source change:

```console
mise run //package/wg/cli-quicker:buildAndTest
mise run //package/wg/cli-quicker:lint:types
mise run //package/wg/cli-quicker:lint:oxlint
mise run //package/wg/cli-quicker:test:integration:route
mise run //package/wg/cli-quicker:test:integration:bypass
```

Oxlint reported `0 warnings and 0 errors` across the package's 58 TypeScript files.
Both integration commands used disposable root network namespaces.

### Incident recovery

The live recovery first verified the sidecar owner,
PID,
kernel start time,
watcher script,
state path,
and process group.
After signaling that verified watcher group,
`wg-quicker down mx-que-mx1` removed the retained interface and state.

The fixed artifact then completed `wg-quicker up mx-que-mx1`.
Post-recovery checks found:

- interface `mx-que-mx1` up with WireGuard fwmark `0xca6c`;
- watcher running under Node 26.7.0 with the expected script and state arguments;
- bypass table `52000` containing owned IPv4 and IPv6 physical defaults;
- rule preference `50` selecting table `52000` for mark `8888`;
- a recent peer handshake and successful HTTPS request through the tunnel.

The peer did not answer ICMP echo requests to `1.1.1.1`.
That did not indicate tunnel failure because WireGuard transfer counters advanced,
the handshake refreshed,
and HTTPS completed.

## What does not work

- Repeating `down` with the pre-fix artifact repeats the executable-path mismatch.
- Running `up` while failed teardown retains the link reports that the interface already exists.
- Deleting ownership files before cleaning owned routes and rules discards the evidence needed for safe teardown.
- Signaling the recorded PID without verifying owner,
  kernel start time,
  script,
  state path,
  and process group risks targeting an unrelated process.
- Accepting every command-line argument was unnecessary.
  Only the executable installation path needed to vary.

## Upstream filing decision

No upstream filing applies.
Node and mise exposed the expected old and current runtime paths;
the incorrect comparison was in this repository.
Commits `0a42e826e`,
`d96581cdd`,
`fbcf81858`,
and their follow-up lint-only commits are the local reproduction,
fix,
and verification artifacts.
