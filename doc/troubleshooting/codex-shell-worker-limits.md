# Codex Bash hook resource limits

## Scope and environment

Observed on Linux with cgroup v2,
`codex-cli 0.162.1`,
and nextest `0.9.148`.
The local implementation is
[`package/codex-plugin/bash-limit`](../../package/codex-plugin/bash-limit/README.md).
The [decision record](../decision/codex-shell-worker-limits.md) owns the accepted budgets and measurements.

## Native hook boundary

### Increasing capacity without leaving the hook and kernel mismatched

On 2026-10-15 the user authorized 2048 tasks instead of 256.
Changing only systemd caused the installed hook to reject every subsequent command, including restoration:

```text
# codex-bash-limit stderr
Codex worker limits: Worker control pids.max must be 256; restore the systemd slice configuration.
```

The baseline `package/codex-plugin/bash-limit/src/limits.rs:32` required the exact pair
`("pids.max", "256")`; commit `561ac46f20` changes that pair to `("pids.max", "2048")`.
The check remains exact; the increase does not disable enforcement.

The existing `codex` child also had `pids.max=256`.
Raising its parent alone therefore left command descendants capped at 256 tasks.
Both live controls now report 2048, and the persistent slice definition records `TasksMax=2048`.
The native tests pass five cases, including changed-control rejection; Clippy passes.
The installed hook accepts subsequent real shell commands without changing their trust definition.
The 4 GiB memory cap, zero swap, and 200% CPU quota remain intact.

The recovery first restored the parent through the file-edit interface, which is outside the shell hook.
The coordinated update then rebuilt and tested the helper and installed it with the matching kernel limit.
The recovery script retained the old helper and restored the original bound if compilation or verification failed.
Do not raise a checked kernel value in one tool call and defer updating the corresponding helper to another call.
That intermediate mismatch prevents the next shell command from running.

This is a local configuration coordination issue, not an upstream Codex defect.
No upstream report is proposed; the maintained helper owns both the check and its diagnostic.

### Original host contract

Codex maps `exec_command` to a Bash `PreToolUse` event with `tool_input.command`.
The handler consumes `updatedInput.command` by replacing the original call's command field.
Other execution settings continue through the normal handler.
Source inspected:
`codex-rs/core/src/tools/handlers/unified_exec/exec_command.rs`,
functions `pre_tool_use_payload` and `with_updated_hook_input`.

The [official hook documentation](https://learn.chatgpt.com/docs/hooks)
describes command rewrites,
blocking exit status 2,
and exact-definition trust.
The host fixture in `tests/native_host.rs` independently exercises this boundary.

Working cases include shell quotes,
backslashes,
command-substitution text inside quoted literals,
heredocs,
functions,
empty commands,
and explicit exit statuses.
The rewritten command preserves its original suffix exactly.

Malformed events,
missing commands,
changed memory or task budgets,
unbounded CPU quotas,
and absent membership files all produce blocking hook failures.
The shell itself exits with status 70 if attachment fails after hook validation.
The guard test includes a successful payload as a positive control.
Removing the guard in a throwaway worktree made that test fail.

## Hook arguments belong in the command string

A hook definition containing separate `command` and `args` properties launched the binary without its arguments.
The binary emitted:

```text
# codex-bash-limit stderr, surfaced by Codex as a blocked hook
Codex worker limits: Expected --hook or --initialize followed by the worker slice path.
```

The `args` field inspected in Codex's hook engine describes its internal shell launch.
It is not a command-hook configuration field.
The working definition puts the binary,
mode,
and parent path in the `command` string,
with POSIX shell quoting applied to each filename.
JSON serialization then encodes that complete command string.
The native host fixture uses a path containing spaces and apostrophes to exercise both boundaries.

## Joining the cgroup from the sandbox

Launching `systemd-run --user` inside the default Codex sandbox failed because the user bus was inaccessible.
The working shell prefix writes its own PID directly to the already-created child cgroup's `cgroup.procs`.
Session initialization happens outside the tool sandbox and creates the child first.
No daemon call occurs for each command.

Adding only `cgroup.procs` to `sandbox_workspace_write.writable_roots` failed:
Codex's bubblewrap setup attempted to create a `.git` entry under that file path.
Granting the child cgroup directory as the writable root allowed the sandboxed shell to join it.
The parent directory stays read-only inside the sandbox,
so the command cannot change the parent budgets through this grant.

A disposable native sandbox probe joined a separately bounded slice and read its parent controls:
4 GiB memory,
zero swap,
200% CPU,
and 256 tasks.
The ordinary native tests use files shaped like these controls;
they do not claim to test kernel enforcement.

## Startup overhead

The discarded Node implementation measured 62.26 ms median across 30 process launches.
Its shell-only control measured 2.40 ms median.
The user required a native implementation without a recurring 30 ms startup penalty.

The Rust host fixture measured 2 to 4 ms per hook in two runs of 40 commands,
with a 3 ms median in each run.
These numbers include Codex's reported hook execution duration.
They do not promise a maximum under arbitrary host scheduling or load.

## Nextest concurrency options

Nextest emitted:

```text
# cargo nextest run stderr
error: the argument '--test-threads <N>' cannot be used multiple times
```

The installed `cargo nextest run --help` identifies `--jobs` as an alias for `--test-threads`.
Compilation concurrency has a separate `--build-jobs` flag.
The accepted package task supplies neither override and uses nextest defaults.
The worker cgroup independently bounds aggregate resource use.

## Reloading an already-running session

The daemon's Unix control socket uses WebSocket framing.
`codex app-server proxy` forwards raw bytes;
sending newline-delimited JSON directly through it does not perform a WebSocket handshake.
Source inspected:
`codex-rs/app-server-transport/src/transport/unix_socket.rs`
and `codex-rs/stdio-to-uds/src/lib.rs`.
A local TCP-to-Unix relay with Node's built-in WebSocket client successfully reached the daemon.

`config/batchWrite` with `reloadUserConfig: true` activated the trusted hooks in the loaded session.
The current turn nevertheless retained its existing sandbox snapshot and emitted:

```text
# Bash stderr after the native hook's membership prefix
/bin/bash: line 1: .../codex/cgroup.procs: Read-only file system
```

The payload did not execute;
the prefix returned status 70.
The native `thread/settings/update` API accepted the additional writable child directory,
but its sandbox-policy contract applies to subsequent turns.
Rejoining with `thread/resume` showed the updated policy while the ongoing turn still used its old snapshot.
Restarting the session provides the new turn boundary.
The first post-restart check must use a default sandboxed command and read `/proc/self/cgroup`.
Do not treat an escalated command's successful attachment as proof that the old sandbox snapshot refreshed.

## Task pressure during Pi package lint

On 2026-10-14,
the complete Pi auto-mode lint invocation remained active without returning its buffered diagnostics.
During that invocation,
a documentation commit succeeded but its automatic push reported:

```text
# Automatic push output; exact Go subprocess not identified
failed to create new OS thread (have 10 already; errno=11)
fatal error: newosproc
```

The later remote-name diagnostic in the same failed push does not identify the originating failure.
The observed lint process held 15 threads,
and its TypeScript workers held 38 and 102 threads.
The enclosing slice retained `pids.max=256` and reported the cumulative counter `pids.events: max 60`.
These observations are consistent with task pressure;
they do not identify the failing allocation or establish an upstream defect.
The earlier new-reference history fanout incident in
[`cli-git-tag-push-eagain.md`](cli-git-tag-push-eagain.md)
is not established as this incident's cause.

The lint process was deliberately terminated with `SIGTERM`.
The wrapper reported
`[task-oxlint] failed to execute oxlint: Command was terminated with SIGTERM`.
That run is incomplete verification,
not a passed check or proof of a hung linter.
The slice then reported 23 tasks.
A separate `git push origin main` succeeded and published the saved commit.
This verifies push recovery only;
it does not prove the cause or establish a permanent fix.

Subsequent lint verification uses explicit sequential file groups under the same rules and resource limits.
The source manifest and each group's result must be retained before claiming complete coverage.
This approach trades a single invocation for separately auditable results;
the interrupted invocation supplies no completed lint evidence.

The explicit manifest eventually covered all 319 files with unchanged final source hashes.
Completed groups and individual-file invocations reported 14,477 warnings and 495 errors in total.
An individual invocation of `src/source-contribution-test-fixture.ts` reported
`Finished in 396.2s on 1 file with 486 rules using 1 threads.`
It produced 49 warnings and no errors.
This establishes completed failing coverage,
not a passed package check.
The complete logs and source hashes are retained under
`~/temp/agent/pi-lint-groups-20261014-zO6k2A/`.

A trial `GOMAXPROCS=2` invocation observed eight threads in the active TypeScript worker,
compared with 38 in the earlier snapshot.
The group still had not completed when it was stopped.
The [Go runtime documentation](https://go.dev/src/runtime/extern.go)
defines this setting as a bound on simultaneously executing Go threads,
not a bound on all operating-system threads.
No timing improvement or permanent task setting is claimed.

The isolated fixture's ten-second Node CPU profile contained 9,448 samples.
The inclusive `expressionResultSites` stack appeared in 9,403 samples,
with nested calls through `reachableValueSources`.
The deciding local source is
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/effect-result-binding.ts:131`:

```ts
// effect-result-binding.ts:131,146; enclosing callback preserves every reached source
reachableValueSources({ project, node: root })
  .forEach(function collectSource(source): void {
    if (source === root) return;
    expressionResultSites({ project, resultSitesBySymbolId, node: source })
      .forEach(function collectSite(found): void { reached.add(found); });
  });
```

This profile identifies work performed by the repository-owned linter.
It does not establish an upstream defect,
connect that traversal to the automatic push failure,
or prove that a proposed traversal change preserves every effect result.
No linter implementation was changed during the migration check.

The later optional-field fixture refactor reproduced the traversal during formatting.
The first fix child remained active at 829 seconds elapsed;
it subsequently completed,
and the wrapper started its separate verification pass.
That pass's ten-second profile contains 9,190 of 9,234 samples inside `expressionResultSites`
and 7,064 inside `reachableValueSources`.
The identified verification child was deliberately stopped with its formatter output retained.
The log is `${HOME}/temp/agent/pi-contribution-fixture-format-20261014.log`;
the profile is `pi-fixture-format-20261014.cpuprofile` in the same scratch directory.

Commit `a07329f154` separates the fixture into original-reference construction,
run composition,
shared types,
and final request association.
The completed full-rule formatter then reports zero warnings and zero errors across those files
and the native result-guard files,
with a final verification duration of 3.0 seconds.
This is successful verification of the new source structure,
not an unchanged-input benchmark or a linter algorithm fix.
The split preserves explicit original-reference boundaries.
All contribution consumer suites subsequently pass,
including original membership,
copied-reference rejection,
retained-history separation,
and absent-association controls.
The native SDK stream and copied payload/transcript controls also pass.
The final plain lint covers ten changed files with zero warnings and zero errors.

### Upstream filing decision for the task-pressure incident

1.  Upstream fault:
    not established;
    the emitting Go subprocess and failing allocation remain unidentified.
2.  Fixability:
    no upstream change has been identified or tested.
3.  Supported use:
    local worker limits remain the accepted execution boundary.
4.  Contribution policy:
    no external filing is proposed from this evidence.
5.  Maintainer intent:
    not evaluated because there is no identified upstream defect.
6.  Prototype:
    none;
    successful push recovery does not qualify an upstream patch.

Nothing to file upstream from this incident.

## Reproduction and verification

```sh
# Repository root
mise run //package/codex-plugin/bash-limit:build
mise run //package/codex-plugin/bash-limit:test
mise run //package/codex-plugin/bash-limit:test:host
mise run //package/codex-plugin/bash-limit:lint:types
```

The host fixture creates a private Codex home,
records stderr there,
and uses a local deterministic provider.
Its recursion guard prevents a host-launched process from launching another fixture.
Nextest terminates a stalled test after its configured timeout.
Successful runs close the host and remove their fixture.

## Upstream filing decision

1.  Upstream fault:
    the failures came from local configuration assumptions and the sandbox's intended access boundary.
2.  Fixability:
    the supported command field and directory grant solve the required behavior locally.
3.  Supported use:
    native Bash rewrites and explicit hook trust are documented and covered by the host fixture.
4.  Contribution policy:
    no upstream contribution is proposed;
    the repository's Codex integration exclusion was overridden only for this requested local implementation.
5.  Maintainer intent:
    not evaluated because no upstream defect is being claimed.
6.  Prototype:
    the Rust hook and native integration fixtures provide the local implementation and evidence.

Nothing to file upstream.
