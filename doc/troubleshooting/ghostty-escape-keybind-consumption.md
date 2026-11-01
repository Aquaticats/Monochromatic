# Ghostty 1.3.1 GTK passes stock Escape; bare `escape=end_search` is a separate consumption trap

## Symptom

Escape appeared not to reach applications in an existing Ghostty session.
This was operationally important because pi binds `app.interrupt` to Escape.
Firefox later failed to leave YouTube full-screen mode with Escape too,
which contradicted a Ghostty-local cause.

A kernel input query then found `KEY_ESC` held only on `ydotoold virtual device`.
Restarting the user ydotool daemon cleared that state,
and the user confirmed that Escape immediately worked in Firefox and Ghostty.
The incident cause and prevention are documented in
[Interrupted ydotool can leave a virtual key pressed desktop-wide](ydotool-interrupted-key-release.md).

No automation was run against the actual desktop because a misdirected key event could interfere with the live session.
The exact installed Ghostty binary,
the current user configuration,
and pi-tui 0.84.2 all passed Escape inside this repository's nested Wayland compositor.
The separate reproducible Ghostty failure is narrower:
an explicit unprefixed binding

```ini
keybind = escape=end_search
```

consumes every Escape press before PTY encoding when no search is active.
Ghostty prints that exact line from `ghostty +list-keybinds`,
but the printed form omits the compiled default's `performable:` flag.
Copying the output back into a configuration therefore changes its behavior.

Observed PTY bytes for injected `a`,
Escape,
then `b`:

- Stock Ghostty defaults:
   `61 1b 62`.
- Current `~/.config/ghostty/config`:
   `61 1b 62`.
- Explicit `performable:escape=end_search`:
   `61 1b 62`.
- Explicit bare `escape=end_search`:
   `61 62`.
- Active Ghostty search with the stock binding:
   `61 62` for the first Escape,
  because that press closes search by design.
- Kitty keyboard protocol flag 1 active:
   `61 1b 5b 32 37 75 62`.
  Escape reached the PTY as `CSI 27 u`,
  not as one `1b` byte.

There was no Ghostty warning or error for the consumed cases.

## Root cause

### The separate Ghostty configuration mechanism

This mechanism did not cause the cross-application incident.
It explains why a copied or hand-written bare Ghostty binding can create a similar terminal-only symptom.

Ghostty 1.3.1 installs the Linux Escape binding with `performable = true`.
The source at tag `v1.3.1`,
commit `332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28`,
sets the physical Escape key to `end_search`
(`src/config/Config.zig:6741-6746`):

```zig
try self.set.putFlags(
    alloc,
    .{ .key = .{ .physical = .escape } },
    .end_search,
    .{ .performable = true },
);
```

The binding flags default to consuming input and not being performable
(`src/input/Binding.zig:28-49`):

```zig
pub const Flags = packed struct {
    /// True if this binding should consume the input when the
    /// action is triggered.
    consumed: bool = true,

    // ...

    /// True if this binding should only be triggered if the action can be
    /// performed. If the action can't be performed then the binding acts as
    /// if it doesn't exist.
    performable: bool = false,
```

`end_search` reports whether a search actually existed
(`src/Surface.zig:5254-5269`):

```zig
.end_search => {
    // We only return that this was performed if we actually
    // stopped a search, but we also send the apprt end_search so
    // that GUIs can clean up stale stuff.
    const performed = self.search != null;

    if (self.search) |*s| {
        s.deinit();
        self.search = null;
    }

    _ = try self.rt_app.performAction(
        .{ .surface = self },
        .end_search,
        {},
    );

    return performed;
},
```

After running a binding action,
`maybeHandleBinding` falls through to normal key encoding only when a performable action returned false
(`src/Surface.zig:2987-3023`):

```zig
// If we have the performable flag and the action was not performed,
// then we act as though a binding didn't exist.
if (leaf.flags.performable and !performed) {
    self.endKeySequence(.flush, .retain);
    return null;
}

// ...

if (consumed) {
    // ...
    return .consumed;
}
```

The stock binding therefore has state-sensitive behavior:

- Search active:
  `end_search` returns true,
  and Ghostty consumes Escape to close search.
- Search inactive:
  `end_search` returns false,
  `performable:` makes the binding act as absent,
  and encoding continues.

A bare `escape=end_search` changes only the flags.
`end_search` still returns false when idle,
but `performable` is false,
`consumed` remains true,
and `maybeHandleBinding` returns `.consumed`.
The encoder never receives the event.

When the event reaches the encoder,
`Surface.encodeKey` delegates to `input.key_encode.encode`
(`src/Surface.zig:3135-3162`):

```zig
if (input.key_encode.encode(
    &writer,
    event,
    encoding_opts,
)) {
    const written = writer.buffered();

    // Special-case: we did nothing.
    if (written.len == 0) return null;
```

In legacy mode,
the unmodified Escape entry is one `0x1b` byte
(`src/input/function_keys.zig:220-239`):

```zig
result.set(.escape, &.{
    // modified variants omitted
    .{ .sequence = "\x1b" },
});
```

With Kitty keyboard protocol enabled,
Ghostty selects the Kitty encoder
(`src/input/key_encode.zig:78-86`),
and Escape maps to codepoint 27 with final `u`
(`src/input/kitty.zig:44-48`):

```zig
return if (opts.kitty_flags.int() != 0) try kitty(
    writer,
    event,
    opts,
) else try legacy(
```

```zig
const raw_entries: []const RawEntry = &.{
    .{ .escape, 27, 'u', false },
```

That path produces `ESC [ 27 u`.
It is a different encoding,
not a dropped key.

### Why `+list-keybinds` is misleading here

The installed command prints:

```text
keybind = escape=end_search
```

The formatter writes the trigger and action,
but its leaf branch never writes `leaf.flags`
(`src/input/Binding.zig:2110-2140`):

```zig
.leaf => |leaf| {
    // When we get to the leaf, the buffer_stream contains
    // the full sequence of keys needed to reach this action.
    buffer.print("={f}", .{leaf.action}) catch return error.OutOfMemory;
    try formatter.formatEntry([]const u8, buffer.buffer[0..buffer.end]);
},
```

The same omission affects `performable:`,
`unconsumed:`,
`all:`,
and `global:`.
The command's line is not a round-trip-safe representation of the binding flags.
It also is not evidence that the live default is bare.

### Why Ghostty was not the desktop cause

The nested positive control proved the harness can show a missing Escape:
the bare override produced `61 62` while the stock binding produced `61 1b 62`.
The current user Ghostty file contains no `keybind`,
`key-remap`,
or `config-file` entry,
and `ghostty +show-config` contains no such override.

The later host-level evidence established a different boundary:

- Every queried physical keyboard interface reported `KEY_ESC` released.
- `ydotoold virtual device` reported `KEY_ESC` pressed.
- Python evdev reported Escape as that device's only active key.
- Restarting ydotoold cleared every active key.
- The same physical Escape key then worked in Firefox and Ghostty.

That before-state,
recovery,
and cross-application after-state establish the virtual input device as the incident cause.
Search focus,
Kitty keyboard mode,
modifiers,
and Ghostty executable identity remain useful generic diagnostic branches,
but they are not needed to explain this incident.

## Verification

### Versions and isolation boundary

Verified on 2026-08-26 with:

- Ghostty 1.3.1,
  Fedora package `ghostty-1.3.1-1.fc44.x86_64`,
  GTK 4.22.4,
  source tag `v1.3.1` at `332b2aefc6e72d363aa93ab6ecfc86eeeeb5ed28`.
- `monochromatic-nested-wayland-session` 0.1.1 at repository commit
  `510cd545b177464981adc83b789cea3b1e8acb70`.
- `@earendil-works/pi-tui` 0.84.2.
- KDE Wayland as the parent session.

All key injection used the nested compositor's Unix-socket control API.
Its `input.rs` forwards into the nested seat,
so no `/dev/uinput`,
KWin automation,
or actual desktop input injection was used.
Every potentially waiting probe had an internal deadline and terminated without requiring Escape.

### Runnable nested harness

Save this file outside the repository after preparing the private scratch root:

```python
# ~/temp/agent/ghostty-escape-probe.py
from pathlib import Path
import os
import select
import socket
import subprocess
import sys
import termios
import time
import tty

SCRATCH = Path.home() / "temp/agent"
REPO = Path.cwd()
COMPOSITOR = REPO / (
    "package/nested-wayland-session/cli/target/release/"
    "monochromatic-nested-wayland-session"
)


def child() -> None:
    output_path = Path(sys.argv[2])
    ready_path = Path(sys.argv[3])
    fd = sys.stdin.fileno()
    original = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        ready_path.write_text("ready\n", encoding="utf-8")
        with output_path.open("wb") as output:
            for _ in range(16):
                readable, _, _ = select.select([fd], [], [], 2.0)
                if not readable:
                    break
                output.write(os.read(fd, 1))
                output.flush()
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, original)


def parent() -> None:
    label = sys.argv[2]
    ghostty_args = sys.argv[3:]
    socket_path = SCRATCH / f"ghostty-escape-{label}.sock"
    output_path = SCRATCH / f"ghostty-escape-{label}.bytes"
    ready_path = SCRATCH / f"ghostty-escape-{label}.ready"
    for path in (socket_path, output_path, ready_path):
        path.unlink(missing_ok=True)

    command = [
        str(COMPOSITOR),
        "--socket",
        str(socket_path),
        "--size",
        "800x600",
        "--",
        "/usr/bin/ghostty",
        *ghostty_args,
        "-e",
        "/usr/bin/python3",
        str(Path(__file__).resolve()),
        "child",
        str(output_path),
        str(ready_path),
    ]
    process = subprocess.Popen(
        command,
        cwd=REPO,
        env=os.environ.copy(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        deadline = time.monotonic() + 10.0
        while time.monotonic() < deadline:
            if socket_path.exists() and ready_path.exists():
                break
            if process.poll() is not None:
                break
            time.sleep(0.02)
        if not socket_path.exists() or not ready_path.exists():
            raise RuntimeError("fixture did not become ready")

        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.settimeout(2.0)
            client.connect(str(socket_path))
            client.sendall(b"key a\nkey escape\nkey b\n")
            reader = client.makefile("rb")
            responses = [
                reader.readline().decode("utf-8").rstrip()
                for _ in range(3)
            ]
        _, stderr = process.communicate(timeout=10.0)
    except BaseException:
        process.terminate()
        try:
            process.communicate(timeout=2.0)
        except subprocess.TimeoutExpired:
            process.kill()
            process.communicate(timeout=2.0)
        raise

    payload = output_path.read_bytes()
    print(f"responses={responses!r}")
    print(f"exit={process.returncode}")
    print(f"bytes_hex={payload.hex()}")
    if process.returncode != 0:
        print(stderr, file=sys.stderr)
        raise SystemExit(process.returncode)


if sys.argv[1] == "child":
    child()
else:
    parent()
```

Run the catalogs from the repository root:

```sh
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
mise run //package/nested-wayland-session/cli:build

python3 "${HOME}/temp/agent/ghostty-escape-probe.py" parent stock \
  --config-default-files=false --gtk-single-instance=false

python3 "${HOME}/temp/agent/ghostty-escape-probe.py" parent user \
  --gtk-single-instance=false

python3 "${HOME}/temp/agent/ghostty-escape-probe.py" parent bare \
  --config-default-files=false --gtk-single-instance=false \
  --keybind=escape=end_search

python3 "${HOME}/temp/agent/ghostty-escape-probe.py" parent performable \
  --config-default-files=false --gtk-single-instance=false \
  --keybind=performable:escape=end_search
```

Expected distinguishing output:

```text
stock:       bytes_hex=611b62
user:        bytes_hex=611b62
bare:        bytes_hex=6162
performable: bytes_hex=611b62
```

The `responses=['ok', 'ok', 'ok']` lines acknowledge nested-seat injection.
Only the recorder bytes prove PTY delivery.

### Working catalog

- Compiled defaults with user files disabled:
  `61 1b 62`.
- Current user configuration:
  `61 1b 62`.
- Current user configuration after a config reload:
  `61 1b 62`.
- Explicit `performable:escape=end_search`:
  `61 1b 62`.
- Explicit `escape=unbind`:
  `61 1b 62`.
- Explicit `escape=text:\x1b`:
  `61 1b 62`.
- Kitty keyboard flag 1,
  requested by writing `CSI > 1 u` before the keys:
  `61 1b 5b 32 37 75 62`.
- Kitty mode pushed and then popped with `CSI < u` before the keys:
  `61 1b 62`.
- pi-tui 0.84.2 `ProcessTerminal` inside the same nested Ghostty:
  it received `1b5b323775`,
  `matchesKey(data, Key.escape)` returned true,
  and the fixture wrote an `escape` marker.

### Consuming catalog

- Explicit bare `escape=end_search`,
  no search active:
  `61 62`.
  This is the confirmed unconditional-consumption mechanism.
- Stock performable binding with a programmatic search active:
  `61 62` for the first Escape.
  This is intended because the action closed search.
- Stock performable binding after that search was closed:
  `61 1b 62`.

### Bounded manual classifier for the actual desktop

This diagnostic does not inject input.
It reads the next bytes from its own PTY for three seconds and restores terminal mode even when no bytes arrive:

```sh
python3 - <<'PY'
import os
import select
import sys
import termios
import tty

fd = sys.stdin.fileno()
original = termios.tcgetattr(fd)
print("Press Escape within three seconds", flush=True)
try:
    tty.setraw(fd)
    readable, _, _ = select.select([fd], [], [], 3.0)
    data = os.read(fd, 32) if readable else b""
finally:
    termios.tcsetattr(fd, termios.TCSADRAIN, original)
print(data.hex() if data else "NO_BYTES")
PY
```

Interpretation:

- `1b`:
  Ghostty delivered legacy Escape;
  investigate the application handler.
- `1b5b323775`:
  Ghostty delivered Kitty `CSI 27 u`;
  investigate application protocol parsing or stale protocol mode.
- `NO_BYTES`:
  the event was consumed before this PTY or the physical event was not delivered.
- Another sequence:
  inspect modifiers,
  input-method state,
  or a remap before assuming a drop.

Run this only in a separate disposable terminal surface,
not in the surface holding an irreplaceable session.

## Configuration repairs for the separate bare-binding trap

For Escape failure across unrelated applications,
use the [ydotool reset runbook](../runbook/verify-escape-after-ydotool-reset.md)
instead of changing Ghostty configuration.

### Preserve Ghostty's stock search behavior explicitly

```ini
keybind = performable:escape=end_search
```

This passed `61 1b 62` while idle and consumed Escape only while search was active.
It is the preferred repair for a configuration that contains the bare line copied from `+list-keybinds`.

Tradeoff:
on Ghostty 1.3.1 this restates the compiled default and does not repair an incident caused outside the binding set.

Reload with Ghostty's default `Ctrl+Shift+,` binding rather than closing the current session.

### Remove Ghostty's Escape action

```ini
keybind = escape=unbind
```

This passed `61 1b 62` in the nested fixture.

Tradeoff:
Escape no longer closes Ghostty's search UI through the core binding.
Use the search UI close control or another explicit search-close binding.

### Force one legacy Escape byte

```ini
keybind = escape=text:\x1b
```

This passed `61 1b 62` in the nested fixture.

Tradeoff:
this bypasses Ghostty's state-sensitive search action and Kitty keyboard encoding.
Applications expecting `CSI 27 u` receive a legacy byte instead,
and Escape no longer closes Ghostty search.
Use this only as a recovery binding,
not as the first fix.

### Pop stale Kitty keyboard mode at a shell prompt

```sh
printf '\033[<u'
```

A fixture that pushed flag 1,
then emitted this pop before input,
changed Escape from `1b5b323775` back to `1b`.

Tradeoff:
this pops one protocol level.
Running it inside an application that intentionally enabled Kitty keyboard reporting
breaks that application's modified-key handling.
Use it only at a shell prompt after an application exited without restoring terminal state.

## What does not work

- Treating `ghostty +list-keybinds` as a round-trip configuration dump.
  Its formatter omits binding flags,
  so copying `escape=end_search` removes the load-bearing `performable:` behavior.
- Adding bare `keybind = escape=end_search` as a fix.
  It is the exact configuration that reproduced unconditional consumption.
- Using `unconsumed:escape=end_search` when search isolation matters.
  It delivered `1b` both idle and during active search,
  so the same press closes Ghostty search and reaches the application.
- Blaming the old GTK performable-accelerator bug
  [ghostty-org/ghostty#4522][ghostty-4522].
  It was fixed by [PR #5421][ghostty-5421] before Ghostty 1.3.1.
  The installed source contains the fix at `src/input/Binding.zig:2480-2485`:

  ```zig
  // This is true if we're going to track this entry as
  // a reverse mapping. There are certain scenarios we don't.
  // See the reverse map docs for more information.
  const track_reverse: bool = !flags.performable;
  ```

  The current nested positive control also passed.
- Treating Kitty `CSI 27 u` as a Ghostty drop.
  The bytes reached the PTY,
  and pi-tui 0.84.2 recognized them as Escape in the consumer-boundary probe.
- Inferring cause from an empty warning journal.
  The binding path is silent by design,
  and journal silence does not identify event delivery.
- Killing all Ghostty processes to test a theory.
  That can destroy the session whose interrupt path is under investigation.
  Use a new isolated surface or the nested compositor.

## Upstream filing artifact

### Duplicate search

The upstream issue and pull-request searches covered open and closed state with:

- `escape end_search`
- `Escape key swallowed`
- `Esc not sent terminal`
- `performable escape`
- `list-keybinds performable`
- `show-config performable keybind`

No exact issue or pull request matched the desktop symptom or the omitted flag in `+list-keybinds`.
Related records were read in full:

- [Issue #4328][ghostty-4328] and [PR #4345][ghostty-4345] introduced `performable:`.
- [Issue #4522][ghostty-4522] and [PR #5421][ghostty-5421] fixed GTK accelerator interception.
- [Discussion #11410][ghostty-11410] covered search bindings while the GTK search box is focused.
  Its selected answer cites macOS-only [PR #12492][ghostty-12492],
  so the GTK fix status is unclear.
- [Issues #3114][ghostty-3114] and [#4505][ghostty-4505] covered other incorrect `+list-keybinds` output.

Nothing in those threads establishes a Ghostty cause for the desktop incident.
The later kernel evidence attributes it to ydotool instead.
There is no additive comment to post to the Ghostty threads.

### Upstream filing decision

`.out-of-scope/` was checked first.
Its Ghostty mention is inside the Claude Code exemption and does not exempt Ghostty reports.
Default policy remains not to file.

1. **Is it really upstream's fault?**
   No for the reported incident on current evidence.
   Stock Ghostty,
   the current user configuration,
   config reload,
   legacy encoding,
   Kitty encoding,
   and the pi-tui consumer boundary all passed in isolation.
   A bare user override is confirmed to fail,
   but that override is absent from the current config.
   The `+list-keybinds` flag omission is a real diagnostic hazard,
   but it did not cause this incident unless its output had previously been copied into runtime configuration.

2. **Can upstream fix it?**
   Ghostty cannot fix the actual incident because the stuck state belonged to another virtual input device.
   Upstream can separately make `+list-keybinds` serialize flags,
   but that would address the diagnostic hazard rather than this incident.

3. **Are they supporting this use case?**
   Yes.
   Escape encoding,
   Kitty keyboard protocol,
   GTK search,
   and `performable:` keybindings are supported features with source,
   documentation,
   and tests.

4. **Would the repo welcome our contribution?**
   Not as an unreviewed copy of this document.
   `CONTRIBUTING.md` routes uncertain bugs to Issue Triage discussions.
   `AI_POLICY.md` permits AI assistance only with disclosure and requires a human to review and edit the content.
   It also requires the human to understand the code.
   No human-edited upstream artifact exists in this investigation.

5. **Will they likely fix it?**
   Not applicable to the incident because Ghostty was not the failing layer.
   Related input and formatter defects were accepted and fixed,
   but the separate formatter omission was not filed during this investigation.

6. **Have we prototyped a minimal fix compatible with their architecture?**
   No upstream fix was prototyped.
   The auto-prototype gate is not triggered because constraints 1 and 4 fail.
   The verified configuration variants are diagnostics and user-side workarounds,
   not a source patch.

Decision:
do not file an issue or discussion and do not keep a sendable draft.
The desktop cause is resolved as an interrupted ydotool release,
and the available evidence would make a Ghostty defect report false attribution.
The separate `+list-keybinds` flag omission remains documented here without an upstream filing.

[ghostty-3114]: https://github.com/ghostty-org/ghostty/issues/3114
[ghostty-4328]: https://github.com/ghostty-org/ghostty/issues/4328
[ghostty-4345]: https://github.com/ghostty-org/ghostty/pull/4345
[ghostty-4505]: https://github.com/ghostty-org/ghostty/issues/4505
[ghostty-4522]: https://github.com/ghostty-org/ghostty/issues/4522
[ghostty-5421]: https://github.com/ghostty-org/ghostty/pull/5421
[ghostty-11410]: https://github.com/ghostty-org/ghostty/discussions/11410
[ghostty-12492]: https://github.com/ghostty-org/ghostty/pull/12492
