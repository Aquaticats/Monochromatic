# Interrupted ydotool can leave a virtual key pressed desktop-wide

Note 2026-11-09T16:49:15-05:00:
these citations in this record name paths that no longer exist.
`package/pi-plugin/auto-mode/src/virtual-input-guard.ts` left the tree on 2026-10-19 in `7ac15b19b4`.
`package/pi-plugin/auto-mode/src/index-bypass.unit.test.ts` left the tree on 2026-10-17 in `a3955898f1`.
`package/pi-plugin/auto-mode/src/virtual-input-guard.unit.test.ts` left the tree on 2026-10-19 in
`7ac15b19b4`.

## Symptom

Escape stopped working in unrelated applications on a KDE Wayland session.
Ghostty did not receive Escape,
and Firefox could not leave YouTube full-screen mode with Escape.
Restarting applications did not address the shared input state.

The kernel input query found this split:

- Physical Keychron,
  macro keyboard,
  and mouse keyboard interfaces:
  `KEY_ESC` released.
- `ydotoold virtual device`:
  `KEY_ESC` pressed.
- Every queried modifier on the ydotool device:
  released.
- Python evdev `active_keys()` for the ydotool device:
  `[(1, 'KEY_ESC')]`.

Three consecutive queries returned status 10:

```bash
evtest --query /dev/input/event18 EV_KEY KEY_ESC
printf 'exit=%s\n' "$?"
```

The installed `evtest(1)` manual defines query status 10 as a set state bit
and status 0 as an unset state bit.
The event number is not stable across daemon restarts.

Restarting the user ydotool daemon recreated its uinput device with no active keys:

```bash
systemctl --user restart 'app-ydotoold@autostart.service'
```

The user then confirmed that Escape immediately worked in both Firefox and Ghostty.
That before-state,
reset,
and cross-application after-state establish the stuck virtual key as the incident's cause.

## Root cause

A Pi session invoked this live-desktop command at 2026-08-26 01:28:45 EDT:

```bash
ydotool key 1:1 1:0 && ydotool type --key-delay=20 ':q!' && ydotool key 28:1 28:0
```

The focused Pi process handled the synthetic Escape key-down as cancellation.
Its Bash tool returned `Command aborted` before ydotool sent Escape key-up.
The persistent ydotoold uinput device retained `KEY_ESC=1` for the next hour.

The evidence is in the local Pi transcript:

```text
~/.pi/agent/sessions/--var-home-user-Monochromatic--/
2026-08-26T04-12-21-130Z_01a03c44-dd0a-7b89-a355-d88b79831e79.jsonl
```

The installed package is Fedora `ydotool-1.0.4-8.fc44`.
Upstream tag `v1.0.4` is commit `57ba7d0`.
Its source explains the cancellation window:

1.  `Client/tool_key.c:70` sets a 12 ms delay between key events.
2.  `Client/tool_key.c:124-146` walks arguments one at a time,
    emits one event,
    then sleeps.
3.  `Client/ydotool.c:95-109` writes the input event and `SYN_REPORT`
    as separate datagrams to a Unix datagram socket.
4.  `Daemon/ydotoold.c:377-383` receives each datagram and writes it directly to `/dev/uinput`.
5.  The protocol has no client connection lifecycle from which the daemon can infer
    that an absent key-up means cancellation rather than an intentional hold.

A syntactically balanced command is therefore not an atomic key gesture.
If its key-down cancels or kills the caller during the inter-event delay,
the later key-up never reaches ydotoold.

This is the Escape equivalent of upstream
[issue #170, “Ctrl+C garbles whole session input”][ydotool-170].
A project collaborator explains there that Ctrl+C kills the foreground ydotool process
before it sends release commands.
The collaborator proposes an optional release-on-disconnect behavior,
but the issue remains open.

## Safe isolated reproduction

This harness uses the installed ydotool client with a private fake Unix datagram server.
It never starts ydotoold,
never opens `/dev/uinput`,
and never injects input into the desktop.

Save as `~/temp/agent/ydotool-interruption-probe.py`:

```python
from pathlib import Path
import os
import socket
import struct
import subprocess
import tempfile

EVENT = struct.Struct('@llHHi')


def decode(packet: bytes) -> tuple[int, int, int]:
    _, _, event_type, code, value = EVENT.unpack(packet)
    return event_type, code, value


with tempfile.TemporaryDirectory(dir=Path.home() / 'temp/agent') as directory:
    socket_path = str(Path(directory) / 'ydotool.sock')
    server = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
    server.bind(socket_path)
    server.settimeout(0.2)
    environment = {**os.environ, 'YDOTOOL_SOCKET': socket_path}

    complete = subprocess.run(
        ['/usr/bin/ydotool', 'key', '--key-delay=0', '1:1', '1:0'],
        env=environment,
        check=False,
        capture_output=True,
        text=True,
    )
    complete_events = [decode(server.recv(EVENT.size)) for _ in range(4)]
    print(f'positive_exit={complete.returncode}')
    print(f'positive_events={complete_events}')

    interrupted = subprocess.Popen(
        ['/usr/bin/ydotool', 'key', '--key-delay=1000', '1:1', '1:0'],
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    interrupted_events = [decode(server.recv(EVENT.size)) for _ in range(2)]
    interrupted.terminate()
    interrupted.wait()
    try:
        interrupted_events.append(decode(server.recv(EVENT.size)))
    except TimeoutError:
        pass
    print(f'interrupted_exit={interrupted.returncode}')
    print(f'interrupted_events={interrupted_events}')
```

Prepare private scratch and run it:

```bash
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
python3 "${HOME}/temp/agent/ydotool-interruption-probe.py"
```

Observed output:

```text
positive_exit=0
positive_events=[(1, 1, 1), (0, 0, 0), (1, 1, 0), (0, 0, 0)]
interrupted_exit=-15
interrupted_events=[(1, 1, 1), (0, 0, 0)]
```

Event type 1,
code 1,
value 1 is Escape key-down.
The following type 0 event is `SYN_REPORT`.
The positive control then receives Escape key-up and another synchronization event.
The interrupted case receives no key-up.

## Detection

Locate the ydotool event dynamically:

```bash
YDOTOOL_EVENT="$(
  awk '
    BEGIN { RS = "" }
    /N: Name="ydotoold virtual device"/ {
      if (match($0, /event[0-9]+/))
        print "/dev/input/" substr($0, RSTART, RLENGTH)
    }
  ' /proc/bus/input/devices
)"
printf 'YDOTOOL_EVENT=%s\n' "${YDOTOOL_EVENT:-missing}"
```

Query a suspected key:

```bash
evtest --query "$YDOTOOL_EVENT" EV_KEY KEY_ESC
state=$?
printf 'state=%s\n' "$state"
```

Interpret the result using `evtest(1)`:

- `state=10`:
  pressed.
- `state=0`:
  released.
- Any other status:
  query failure.

For every active key when Python evdev is installed:

```bash
python3 - <<'PY'
from pathlib import Path
import evdev

for path in sorted(Path('/dev/input').glob('event*')):
    try:
        device = evdev.InputDevice(str(path))
    except PermissionError:
        continue
    if device.name == 'ydotoold virtual device':
        active = [
            (code, evdev.ecodes.KEY.get(code, 'UNKNOWN'))
            for code in device.active_keys()
        ]
        print(f'device={path}')
        print(f'active_keys={active}')
PY
```

## Recovery

Restart the existing user daemon:

```bash
systemctl --user restart 'app-ydotoold@autostart.service'
```

Then relocate its event node and verify `state=0`.
The complete application-boundary verification is in
[`doc/runbook/verify-escape-after-ydotool-reset.md`](../runbook/verify-escape-after-ydotool-reset.md).

Restarting ydotoold removes and recreates its virtual input device.
It releases every key from that device,
not only Escape.
Any ydotool injection in progress is interrupted.

If the user daemon has a different unit name,
inspect it before restarting:

```bash
systemctl --user list-units --all --type=service --no-pager \
  | rg --ignore-case 'ydotool' -
```

## Prevention

### Pi hard guard

`package/pi-plugin/auto-mode/src/virtual-input-guard.ts` parses Bash tool calls
before bypass or model-judge handling.
It blocks statically visible caller-scoped ydotool execution,
including:

The paths this record names under `package/pi-plugin/auto-mode/src/` were removed on 2026-10-19 in
`7ac15b19b4` (refactor(pi-plugin-auto-mode):
remove legacy source for ground-up rebuild);
no file
in the working tree carries them now.
`package/pi-plugin/auto-mode/src/index-bypass.unit.test.ts` was removed on 2026-10-17 in
`a3955898f1` (refactor(pi-plugin-auto-mode):
remove superseded extension pipeline);
no file in the
working tree carries it now.

- direct `ydotool` and `/usr/bin/ydotool` commands;
- logical chains containing ydotool;
- generic wrappers such as `env`,
  `timeout`,
  `nohup`,
  and `systemd-run`;
- direct and wrapper-nested `bash -c`,
  `sh -c`,
  and related shell programs;
- malformed shell source containing ydotool.

Allowlisted text inspection such as `rg ydotool .` remains available.
Commands with `-exec`-style capabilities fail closed instead.
A caller-independent broker must expose a narrow input API and own key release internally,
rather than accepting arbitrary ydotool commands from agent-authored shell.
No such broker was validated or adopted during this incident.
Live global input still requires user authorization.

The static guard cannot identify executable names hidden behind shell variables,
commands loaded from script files,
heredocs or standard input,
runtime-generated command-substitution output,
configuration that later spawns ydotool,
or custom interpreter code that spawns it.
`AGENTS.md` rule `VKI` is the policy backstop for those forms.
The hard guard runs even when auto-mode bypass is enabled.
`package/pi-plugin/auto-mode/src/virtual-input-guard.unit.test.ts`
covers command classification.
`package/pi-plugin/auto-mode/src/index-bypass.unit.test.ts`
covers the non-bypassable integration boundary.
All three cited files are gone:
`7ac15b19b4` (2026-10-19) removed `virtual-input-guard.ts` and its unit test,
and `a3955898f1` (2026-10-17) removed `index-bypass.unit.test.ts`.
Removing the guard call,
rebuilding,
and running that test made it fail because bypass executed the fake Bash path;
restoring the guard made it pass.

A one-off disposable verifier also loaded the built extension through real Pi `AgentSession`,
used a scripted local provider to request the ydotool command,
and replaced Bash execution with a fake callback.
The built extension returned the hard-block reason,
the provider completed its follow-up turn,
and the fake Bash callback remained unused.
No desktop input was injected.

### Agent policy

`AGENTS.md` rule `VKI` requires one of these ownership boundaries:

- the repository's nested compositor for isolated GUI testing;
- a caller-independent broker for authorized global virtual input.

It prohibits invoking ydotool from an agent command.
The repository runbook skill no longer presents direct ydotool as a manual-action bridge.

### Broader host choices

- **Disable ydotoold autostart.**
  This prevents all ydotool keyboard injection and therefore all persistent ydotool key state.
  It also breaks features that intentionally depend on ydotool,
  including `package/kwin/key-helper`.
- **Keep ydotoold and route gestures through a durable broker.**
  The broker must outlive the requesting terminal or agent and must always complete release events.
  This preserves intentional injection,
  but the broker still needs validation and narrowly scoped authorization.
- **Add a stuck-key watchdog.**
  A watchdog can reset ydotoold after a bounded hold time.
  It limits outage duration rather than preventing the bad state,
  and it conflicts with intentional long holds.

The repository implemented the Pi hard guard and agent policy because the incident's deciding transcript
shows an agent-owned direct invocation.
It did not disable the user's ydotool functionality globally.

## Failed or misleading approaches

- **Blaming Ghostty's `escape=end_search` listing.**
  Isolated stock and user-config Ghostty probes delivered Escape.
  Firefox failing at the same time contradicted an application-local cause.
- **Putting key-down and key-up in one ydotool command.**
  Source and the fake-socket reproduction show that they remain separate datagrams
  with a cancellation point between them.
- **Setting `--key-delay=0`.**
  This narrows the cancellation window but does not turn the events into one atomic gesture.
- **Sending a release from the affected agent command.**
  The agent command is the ownership boundary that was canceled.
  Recovery must come from an independent process or daemon restart.
- **Restarting Firefox or Ghostty.**
  The pressed bit lived on the kernel-visible ydotool virtual device,
  outside either application.
- **Replacing it with xdotool on this host.**
  The session is KDE Wayland.
  Upstream describes xdotool as an X11 XTEST tool and warns that typing and window search
  do not work correctly on Wayland.
- **Opening a new upstream report.**
  Upstream issue #170 already describes the same foreground-client cancellation mechanism
  and remains open.

## Upstream status

No new upstream report is warranted because
[ReimuNotMoe/ydotool#170][ydotool-170]
is an exact mechanism duplicate.
Its collaborator-authored analysis matches the installed source and isolated reproduction.

The local incident adds an Escape-specific case in which the injected key cancels Pi's Bash tool,
but that is supporting evidence for the existing issue rather than a distinct ydotool defect.
Commenting on the external issue would require separate authorization
and should include human review of the captured evidence.

## Source provenance

- Installed `evtest(1)` manual,
  dated 2026-01-16:
  query status 10 means set and status 0 means unset.
- Fedora `ydotool-1.0.4-8.fc44` package metadata.
- [Fedora f44 `ydotool.spec`][fedora-ydotool-spec],
  accessed 2026-08-26:
  `Source0` is upstream `v1.0.4`,
  and the spec applies no patch.
- Upstream ydotool tag `v1.0.4`,
  commit `57ba7d0`:
  `Client/tool_key.c`,
  `Client/ydotool.c`,
  and `Daemon/ydotoold.c`.
- Local Pi transcript at the path named in Root cause.
- Kernel input inventory from `/proc/bus/input/devices`.
- Read-only `evtest --query` and Python evdev active-key probes.
- User verification after ydotoold restart in Firefox and Ghostty.
- [xdotool's upstream README][xdotool-readme],
  which identifies xdotool as X11 automation and documents its Wayland limitation.

[fedora-ydotool-spec]: https://src.fedoraproject.org/rpms/ydotool/raw/f44/f/ydotool.spec
[xdotool-readme]: https://github.com/jordansissel/xdotool#wayland
[ydotool-170]: https://github.com/ReimuNotMoe/ydotool/issues/170
