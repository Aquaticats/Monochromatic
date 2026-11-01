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

## Second cause of the same refusal: a watcher that was exiting (live incident, 2026-10-31)

### Symptom

`wg-quicker down fr-par-fr1`,
run with the build of commit `fad3cdc216` against a tunnel and route watcher that the build of `557ea24e24` had started at 22:24,
failed at 23:57:43 with the same two lines as in the Node upgrade incident of this document:

```text
AggregateError: Application-bypass cleanup failed for fr-par-fr1; ownership state retained.
BypassRouteError: Refusing to signal PID 55398 because command is not bypass watcher for
/run/wg-quicker/interface-596e9a12408f3a60f757a723f3de0dd6.json.
```

The stack shows where the check ran:
`watcherIsRunning` called from `waitForWatcherStop` called from `stopBypassWatcher`.
`waitForWatcherStop` runs only after the `SIGTERM` was sent,
so the first identity check had passed and the watcher had been signaled.

### Exposure window

Measured from the sudo journal (`journalctl --identifier=sudo`) and the failing run:

- 23:57:42:
  `down` starts with the new build.
  Its first steps removed the OpenSnitch allowance,
  the policy rules,
  the update-host rules and the bypass routes.
- 23:57:43:
  `down` fails,
  with the interface still present.
  `ip route get 1.1.1.1` then answered `via 192.168.253.1 dev wlp9s0`:
  all traffic left outside the tunnel,
  from a tunnel that looked up.
- 23:59:14:
  `down` with the old build completed and removed the tunnel.
  The half-removed state lasted 92 seconds.
- 23:59:41:
  `up` with the new build started,
  and it exited 0 at 00:00:39.
  No tunnel carried traffic for 177 seconds in all.

### Root cause

#### It was not a persisted-format difference

The first suspicion was that the new build read a state or watcher record that the old build wrote differently.
It did not:

- `git log` shows no commit after `557ea24e24` that touches `tunnel-bypass-state.ts`,
  `tunnel-bypass-watcher-sidecar.ts`,
  `tunnel-bypass-path.ts`,
  `linux-process-identity.ts`,
  `tunnel-bypass-watch-service.ts` or `bypass-watch.ts`.
- The built `index.mjs` of three builds were compared function by function:
  the one live before the first live apply,
  the one that started the tunnel,
  and the one that failed.
  `parseBypassState`,
  `parseWatcherIdentity`,
  `bypassStatePath`,
  `watcherIdentityPath`,
  `bypassStateKey` and `readWatcherIdentity` are byte-identical after renaming the numbered logger variables,
  and the state serializer is the same text.
- The sudo journal shows the same bundle path (`/var/home/.../package/wg/cli-quicker/dist/final/node/index.mjs`) for the `up` of 22:24 and the `down` of 23:57,
  so the watcher script argument was not different either.
- The new watcher's command line (read from `/proc` after the second `up`) is `node`,
  the watcher script,
  and the state path:
  three arguments,
  as the check expects.

#### A process that is exiting reads an empty command line

While a signaled process releases its address space,
`/proc/PID/stat` still shows a live state and `/proc/PID/cmdline` already reads empty.
Measured with a throwaway `node` process holding 200 MB that was sent `SIGTERM` and polled with the same two reads as `readLinuxProcessIdentity`:
four of six runs showed state `R` with an empty command line,
in up to hundreds of consecutive samples,
before the process became a zombie.
The watcher of the incident had run for an hour and a half (22:24 to 23:57);
its memory size at that moment was not measured,
and the time the exit takes depends on it.

`watcherIsRunning` compared the (empty) command line with the expected arguments and,
finding them different,
threw the refusal that exists to stop a reused PID from being signaled:

```ts
// package/wg/cli-quicker/src/tunnel-bypass-watch-service.ts:125
if (live.commandLine
  .length
  === 0)
  return false;
```

That guard is the fix.
The start time already identifies the process,
so an empty command line of a live-state process is a watcher that is going away,
and `waitForWatcherStop` goes on waiting for its process group to disappear.
A live process running another command is still refused.

The race is in every build that has the check,
including the build that started the tunnel,
so a `down` with the old build can fail the same way.
The `down` of 23:59:14 passed because the watcher had exited by then.

#### The teardown order turned a failed stop into a half-removed tunnel

`cleanup` removed the policy rules first,
and `cleanupBypassState` went on with the network removal after the watcher stop failed
(its comment:
"Network cleanup attempted even when sidecar validation blocks watcher signaling"),
and only then threw.
The kill switch,
the link deletion and the DNS revert never ran.
The check that failed is a step that can fail,
and it ran after the routing was gone.

### Fix

- `cf48274391`:
  an empty command line of a live-state process counts as a watcher that is no longer running.
- `ab046a3a97`:
  `down` runs the steps that can fail first.
  The owner chose the option "Keep the tunnel up":
  "Run every step that can fail (stopping watchers,
  reading state) before touching routing;
  if one fails,
  stop with the tunnel fully up and a loud error saying what to do."
  The accepted cost is that "a later step failing (for example deleting the interface) is still possible;
  that case falls back to blocking".
  The owner also answered no to a separate `wg-quicker restart` command.
- `c02dd1949c`:
  the first update-host refresh retries in five seconds when every lookup timed out.

### The refresh that timed out

After the second `up`,
the first update-host refresh timed out on all 21 lookups (`/run/systemd/resolve/io.systemd.Resolve did not finish answering within 10000 ms`)
and the retry 30 seconds later succeeded.
What the logs establish:
NetworkManager assumed the new device at 23:59:43,
`systemd-resolved` received the DNS list for it in the same second and logged nothing else at its default level,
and the link runs with `+DNSOverTLS` and six servers,
IPv6 first.
The refresh inside `up` took about 55 seconds from the OpenSnitch notice (23:59:43) to its report (00:00:38).
The first `up` of 22:24 needed 22 seconds for the same step and succeeded.
The mechanism was not established and could not be reproduced without another live `down` and `up`.
The hypothesis is that strict DNS-over-TLS needs a TCP and TLS handshake through a tunnel whose own first handshake had not completed,
and that `systemd-resolved` tries the servers in turn,
so one lookup outlasts the ten-second limit of the client.

### Verification

- The race:
  a probe with a throwaway process (the heading "A process that is exiting reads an empty command line"),
  unit tests of `watcherIsRunning` (any node binary,
  exiting,
  another command,
  other start time,
  zombie,
  gone),
  of the stop wait through a scripted exit,
  and of `stopBypassWatcher` against a real stand-in process whose state and record are written in the literal format of `557ea24e24`.
  Reading an empty command line as another command,
  and accepting another command,
  each fail a named test.
- The order:
  tests with injected failures for every step,
  the verified interface delete against a real interface that cannot be deleted (the loopback),
  and the block script in two throwaway network namespaces joined by a veth pair
  (reachable before the block;
  blocked between the namespaces,
  with the loopback and the own address fine,
  after it;
  reachable again after deleting the table).
- The refresh:
  tests of the five-second retry and of the unchanged delays,
  and a real timed-out call that carries the marker.

### Recovery and upgrading a running tunnel

The recovery of the incident was the old build's `down` (restored in place),
then the new build's `up`.
After the fix is built,
`down` with the new build stops a watcher that any older build started,
because the persisted formats never changed,
and it keeps the tunnel up if anything fails first.
Before the fix,
the sequence "`down` with the build that started the tunnel,
then `up` with the new build" is still the conservative one,
but it does not avoid the race:
that build has the race too.
If a `down` reports the refusal,
run `wg-quicker down` again at once:
the watcher is gone by then and the retry removes the rest.

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
