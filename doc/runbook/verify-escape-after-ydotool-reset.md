# Verify Escape after resetting ydotool

## What this proves

This runbook distinguishes an Escape key left pressed on the `ydotoold` virtual input device
from physical-keyboard,
KDE,
Firefox,
and Ghostty handling failures.

The incident investigation first used the repository's nested Wayland compositor.
That bridge proved stock Ghostty forwarded Escape,
but isolation from the host KDE session meant it could not reproduce the cross-application symptom.
Live-desktop key injection was deliberately not used.
A read-only `evtest` query then found `KEY_ESC` pressed only on `ydotoold virtual device`.
Restarting the user ydotool daemon recreated that device with no active keys.

Background and source analysis are in
[the ydotool interruption diagnosis](../troubleshooting/ydotool-interrupted-key-release.md).

## Setup

Status:
TODO

1.  Use the affected Linux KDE Plasma Wayland session with the physical keyboard connected.
    The KDE desktop should be active.
2.  Confirm that `evtest`,
    Firefox,
    Ghostty,
    and ydotool are installed.
    Open a terminal with **Ctrl+Alt+T** and run:

    ```bash
    command -v evtest firefox ghostty ydotoold
    ```

    Expect four absolute executable paths.
3.  Locate the current ydotool virtual input device:

    ```bash
    export YDOTOOL_EVENT="$(
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

    Expect one path such as `YDOTOOL_EVENT=/dev/input/event18`.
    If the value is `missing`,
    start the existing user autostart service:

    ```bash
    systemctl --user start 'app-ydotoold@autostart.service'
    ```

    Then repeat setup step 3.
4.  In an already-running Pi session that predates the virtual-input guard build,
    enter **/reload** in the Pi editor.
    Pi should return to its editor without displaying `Failed to load extension`.
    New Pi processes load the built guard during startup and do not need this step.

## Steps

Status:
TODO

1.  Query the virtual Escape state:

    ```bash
    evtest --query "$YDOTOOL_EVENT" EV_KEY KEY_ESC
    escape_state=$?
    printf 'escape_state=%s\n' "$escape_state"
    ```

    `escape_state=10` means the virtual Escape key is pressed.
    `escape_state=0` means it is released.
    Any other value is a query error and must stop this runbook.
2.  If step 1 printed `escape_state=10`,
    restart the user ydotool daemon:

    ```bash
    systemctl --user restart 'app-ydotoold@autostart.service'
    ```

    The command should return to the prompt without an error.
    This recreates the virtual device instead of injecting a key into the focused window.
3.  Locate the recreated device and query Escape again:

    ```bash
    export YDOTOOL_EVENT="$(
      awk '
        BEGIN { RS = "" }
        /N: Name="ydotoold virtual device"/ {
          if (match($0, /event[0-9]+/))
            print "/dev/input/" substr($0, RSTART, RLENGTH)
        }
      ' /proc/bus/input/devices
    )"
    evtest --query "$YDOTOOL_EVENT" EV_KEY KEY_ESC
    escape_state=$?
    printf 'YDOTOOL_EVENT=%s\nescape_state=%s\n' "$YDOTOOL_EVENT" "$escape_state"
    ```

    The final line must be exactly:

    ```text
    escape_state=0
    ```

4.  Open **Firefox** and navigate to a playable YouTube video.
    The video player should render and show its controls.
5.  Select the YouTube video player.
    The player controls should become visible.
6.  Press **F**.
    The YouTube video should enter full-screen mode.
7.  Press **Escape** once.
    The video should leave full-screen mode immediately.
8.  In **Ghostty**,
    paste and run this bounded byte classifier:

    ```bash
    python3 - <<'PY'
    import os
    import select
    import sys
    import termios
    import tty

    descriptor = sys.stdin.fileno()
    previous = termios.tcgetattr(descriptor)
    try:
        tty.setraw(descriptor)
        readable, _, _ = select.select([descriptor], [], [], 5)
        data = os.read(descriptor, 1) if readable else b''
    finally:
        termios.tcsetattr(descriptor, termios.TCSADRAIN, previous)
    print(f"\nbytes_hex={data.hex() or 'timeout'}")
    PY
    ```

    The command should wait for one key for no more than five seconds.
9.  While the classifier waits,
    press **Escape** once.
    Ghostty should print exactly:

    ```text
    bytes_hex=1b
    ```

## What to check

Status:
TODO

- The post-restart kernel query printed exactly `escape_state=0`.
- Firefox left YouTube full-screen mode after one **Escape** press.
- Ghostty printed exactly `bytes_hex=1b`.
- If the kernel query is `0` but either application still fails,
  record which application failed and whether another physical keyboard behaves differently.
  That result shifts diagnosis to physical-device delivery or KDE's application-facing path.

## Restore

Status:
TODO

1.  If the YouTube video remains full-screen,
    press **F**.
    The video should leave full-screen mode without depending on Escape.
2.  Close the test video tab with **Ctrl+W**.
    The Firefox tab should close.
3.  Leave `app-ydotoold@autostart.service` running.
    It was running before the recovery and no persistent configuration changed.
4.  Clear the shell-only variables:

    ```bash
    unset YDOTOOL_EVENT escape_state
    ```

    No output is expected.
