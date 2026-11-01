# xvfb-run as container PID 1 waits before launching Android Emulator 37.2.12

## Symptom

The container-native library comparison did not launch the emulator at all.
Its bounded ADB watch reported `device 'emulator-5582' not found`.
Inspection of the live container showed:

```text
# Container process inspection, selected fields
PID  PPID  COMMAND   WCHAN
1    0     xvfb-run  sigsuspend
11   1     Xvfb      ep_poll
```

There was no QEMU/emulator process.
The ADB daemon was started by the watch,
not evidence that Android booted.
This failure is separate from the guest System UI/launcher ANRs,
the renderer crashes and the contained memory-limit kill.

## Source boundary

The actual container's `/usr/bin/xvfb-run` was copied read-only for
inspection.
Its startup sequence sets a readiness signal handler,
starts Xvfb and waits before invoking the requested command:

```sh
# /usr/bin/xvfb-run, inspected startup sequence
trap : USR1
(trap '' USR1; exec Xvfb ":$SERVERNUM" $XVFBARGS $LISTENTCP -auth $AUTHFILE >>"$ERRORFILE" 2>&1) &
XVFBPID=$!
wait || :
```

Comparable X server source at
`fc625fe172d9f6a149a594b5214364bedf680239`,
`os/connection.c:184` to `212`,
records the parent PID and does not send its readiness signal to PID 1:

```c
// os/connection.c, InitParentProcess
ParentProcess = getppid();
```

```c
// os/connection.c, NotifyParentProcess
if (RunFromSmartParent) {
    if (ParentProcess > 1) {
        kill(ParentProcess, SIGUSR1);
    }
}
```

That mechanism matches a wrapper waiting as PID 1 with Xvfb as its child.
The source revision is comparable,
not a proved build mapping for the container executable.
The process inspection and paired consumer control are retained separately
from that source inference.

## Verification

The SDK launcher reports Android Emulator `37.2.12.0`,
build `16428233`.
The container image is the existing
`localhost/monochromatic-playwright:latest` runtime,
with its own libraries and Xvfb rather than a host `/usr` bind mount.
The native visit retained 6 GiB/2 CPU caps.

The failing launch placed `xvfb-run` directly at PID 1 and never reached
the emulator command during the readiness watch.
The inspected Podman help describes `--init` as running an init process
that forwards signals and reaps processes.
A bounded control with that option reached the SDK launcher successfully:

```bash
# Owned runtime control; SDK is mounted read-only.
podman run --memory=2g --cpus=2 --init --rm \
  --security-opt label=disable \
  --volume "${HOME}/Android/Sdk:${HOME}/Android/Sdk:ro" \
  localhost/monochromatic-playwright:latest \
  xvfb-run --auto-servernum --error-file=/dev/stderr \
  "${HOME}/Android/Sdk/emulator/emulator" -version
```

It printed:

```text
# Successful child invocation
Android emulator version 37.2.12.0 (build_id 16428233) (CL:N/A)
```

The version control proves that this wrapper path reached its child.
It is not a successful Android boot,
application capture or general graphics-stability claim.
The full native comparison is a separate verification step.

## Verified consumer configuration and tradeoffs

Use `podman run --init` when this signal-based Xvfb wrapper is the
container's entry command.
Do not rewrite the third-party wrapper or suppress its startup diagnostics.
The init process changes PID ownership and signal/reaping behavior;
that is the intended infrastructure difference,
not an application change.
No original AVD or user credential is modified.

## What does not work

- Treating an ADB daemon or running Xvfb as proof that QEMU started.
- Repeating a guest boot watch while the wrapper has not invoked its child.
- Treating XKEYBOARD symbol warnings as this failure's cause:
  the successful version control also prints those warnings.
- Calling the blocked wrapper's forced stop a clean guest shutdown:
  no guest had launched,
  and Podman reported its SIGKILL fallback after the stop interval.

## Upstream filing decision

Nothing is filed or drafted as ready.

- Fault:
  the observed issue is a container-entrypoint/readiness interaction;
  an upstream X server defect is not established.
- Ability:
  wrapper or launcher changes are possible,
  but the verified consumer configuration avoids an upstream patch.
- Supported use:
  Xvfb's parent-notification convention does not establish that PID 1 is a
  valid notification target.
- Contribution policy:
  no matched source/binary contribution path was audited.
- Likely action:
  no maintainer decision about this exact container configuration was
  established.
- Prototype:
  the consumer-side init configuration is tested;
  no third-party source edit or speculative fix is retained.

No external draft is prepared,
so duplicate-tracker and `.out-of-scope/` filing checks are not represented
as completed.
