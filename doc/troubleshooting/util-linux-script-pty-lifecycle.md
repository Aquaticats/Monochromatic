# util-linux script 2.42.2: piped PTY sessions require explicit exit and child supervision

## Symptom

The native Depot CLI debugging leg needs an actual terminal.
Giving it a PTY with `script` introduces another process and session boundary.
Closing the input pipe does not establish that its remote interactive shell exited,
or that terminating the wrapper reaped its child.

This finding concerns our proposed native Depot CLI debugging bridge.
The [sealed source and local-control receipt][receipt] qualifies the local PTY and its cleanup.
It does not qualify a Depot SSH session.
No upstream failure was reproduced.
The initially incomplete timeout supervision was our controller defect.

## Root cause

The read-only source is util-linux tag `v2.42.2`,
commit `f44509230b9f8224993b8c6157f85888359a64e1`,
from [the upstream repository][upstream].
The installed executable reports `script from util-linux 2.42.2`.

The separately captured Depot CLI source is commit `1f96d6d`.
`pkg/cmd/ci/ssh.go:74` selects metadata output when stdin is not a terminal:

```go
// depot/cli pkg/cmd/ci/ssh.go:74 to 76
if info || !helpers.IsTerminal() {
    return printSSHInfo(sandboxID, sessionID, output)
}
```

Its interactive path calls `pty.Run`.
`pkg/pty/session.go:37` requires raw terminal input:

```go
// depot/cli pkg/pty/session.go:37 to 39
oldState, err := term.MakeRaw(fd)
if err != nil {
    return fmt.Errorf("make raw: %w", err)
}
```

Thus ordinary piped `depot ci ssh` is a metadata operation,
not execution of the RPC terminal challenge.
The local bridge supplies that terminal.

`term-utils/script.1.adoc:162` explicitly documents the piped-input limitation:

```text
# util-linux term-utils/script.1.adoc:162, excerpt
When stdin is not a terminal ... then the session can hang,
because the interactive shell within the script session misses EOF
```

The native command operands are not a direct `execvp` argument vector.
`term-utils/script.c:948` joins them into a shell command,
and `term-utils/script.c:1041` executes that command through the selected shell:

```c
/* util-linux term-utils/script.c:948 */
ctl.command = ul_strv_join(cmd_argv, " ");
```

```c
/* util-linux term-utils/script.c:1041 */
execl(shell, shname, "-c", ctl.command, (char *)NULL);
```

Consequently,
the bridge uses `--command` with a shell-quoted `exec` command.
Each argument is quoted for that shell boundary.
The selected CLI replaces the shell.
This does not make an arbitrary command string into an argument-preserving interface.

The child initializes its PTY slave before executing the shell.
`lib/pty-session.c:291` creates a separate session:

```c
/* util-linux lib/pty-session.c:291 */
setsid();
```

Killing only the wrapper's process group is therefore insufficient evidence that this child was terminated.
The source itself distinguishes normal final waiting from its caught-signal branch:

```c
/* util-linux term-utils/script.c:1095 to 1104 */
if (!caught_signal && ctl.child != (pid_t)-1)
    ul_pty_wait_for_child(ctl.pty);

if (caught_signal && ctl.child != (pid_t)-1) {
    fprintf(stderr, "\nSession terminated, killing shell...");
    kill(ctl.child, SIGTERM);
    sleep(2);
    kill(ctl.child, SIGKILL);
```

Our bridge keeps the wrapper alive while terminating its exact recorded child,
then awaits the wrapper's close result.
The local receipt records actual PID identities and absence;
it does not infer cleanup from a requested signal.

## Verification

The [source archive][sources] retains:

- `controller/mise.toml`,
  containing the runnable `qualify-native-cli-pty-bridge` task.
- `controller/native-ssh-owned-process.mjs`,
  the helper imported by that task.
- The exact util-linux source files cited in this note.

To reconstruct the isolated harness,
copy that task and helper into a private scratch directory.
Give its local `native-ssh-proposal-manifest.json` an `area` field naming that directory.
The task requires Node `26.12.0` and util-linux `script 2.42.2`.
Its actual invocation is:

```sh
# Run from the reconstructed private harness directory.
mise run --quiet qualify-native-cli-pty-bridge
```

The task sends an invented challenge through the PTY.
The expected response hash is absent from the input.
The child reports actual stdin/stdout TTY state and computes the response itself.
The task uses no Depot API credential or remote session.

### Working controls

- The child observed terminal stdin and stdout,
  answered the challenge and returned `0`.
- The deliberate negative child returned `7`;
  `--return` preserved that exit code through `script`.
- Both normal controls reaped the wrapper and left every recorded owned PID absent.

### Deliberate failure controls

- A direct child ignored `SIGTERM`.
  The supervisor escalated to `SIGKILL`,
  reaped it and verified its recorded identity absent.
- A `script` child in its separate session ignored `SIGTERM`.
  The supervisor killed that exact child,
  kept the wrapper available to reap it and observed wrapper status `137`.
  Both recorded identities were absent.

These are local process and transport controls.
They establish neither remote VM termination nor completeness of an arbitrary descendant inventory.

## Verified workarounds

Use the qualified task's combination of `--quiet`,
`--return`,
`--flush`,
`--echo never` and shell-quoted `--command`.
Send an explicit remote `exit $?` after the challenge consumer.
Retain the raw terminal transcript and the actual exit status.
The tradeoff is a shell grammar boundary and terminal control bytes in captured output.

Supervise the exact owned child identities,
including the separate-session child,
with bounded `SIGTERM` and `SIGKILL` escalation.
Await the direct child's close result and inspect the recorded identities after cleanup.
The helper uses Linux `/proc` identities;
it is not a portable general process-tree supervisor.
Its absence claim covers the recorded processes.

## What does not work

The first controller implementation sent only `SIGTERM` and then awaited close without escalation.
Its outer timer merely set `exitCode`.
Source review rejected that implementation before cloud allocation.
The deliberately TERM-ignoring local controls qualify the corrected supervision.

Treating `script -- command argument...` as direct argv preservation is contradicted by the source.
That alternative was source-rejected;
no runtime argument-corruption reproduction is claimed.

A pipe ending is not proof that an interactive child received EOF or exited.
The manual documents the limitation.
The qualified control sends an explicit exit instead.

## Upstream filing artifact

Nothing to add upstream.
There is no demonstrated upstream defect and no proposed upstream patch.
No external issue or comment was filed.

### Upstream filing decision

1.  Upstream fault:
    no.
    Our adapter needed explicit shell termination,
    proper quoting and owned-child supervision.
2.  Fixability:
    no upstream change is needed for this verified boundary.
3.  Supported use:
    the manual documents `--command` and `--return`,
    and explicitly cautions about noninteractive pipes.
4.  Contribution acceptance:
    not assessed for a filing,
    because no upstream defect is alleged.
    No policy restriction is inferred.
5.  Expected acceptance:
    not assessed.
    No maintainer refusal or guarantee is inferred.
6.  Prototype:
    no upstream prototype is appropriate for this adapter correction.

The repository `.out-of-scope/` inventory was checked;
it contains no util-linux exemption.
Read-only searches of upstream issues and pull requests for `script child SIGTERM` returned no matches
on `2026-10-18`.
That narrow search is not a claim that no related history exists.
The upstream-fault gate already fails.

[receipt]: ../audit/depot-native-ssh-acceptance-2026-10-18.evidence.json
[sources]: ../audit/depot-native-ssh-acceptance-2026-10-18.sources.json.gz
[upstream]: https://github.com/util-linux/util-linux
