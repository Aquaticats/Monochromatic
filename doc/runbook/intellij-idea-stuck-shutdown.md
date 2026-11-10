# Recover IntelliJ IDEA from a stuck shutdown

## What this proves

This runbook distinguishes a healthy existing IDEA instance from the specific stuck-shutdown state that returns:

```text
IDE is being shut down
exit=16
```

It terminates only processes from the JetBrains Toolbox IntelliJ IDEA installation,
relaunches through the Toolbox desktop entry,
and verifies both the activation endpoint and KDE Wayland windows.

The prior incident used direct launcher probes,
process inspection,
IDEA logs,
`wmctrl`,
`xdotool`,
and KWin D-Bus scripting.
`wmctrl` and `xdotool` could not see IDEA's native Wayland windows.
The direct launcher and KWin D-Bus paths did expose the failure and verify recovery,
so this runbook preserves those bridges rather than requiring config deletion or blind UI interaction.

Background and source analysis are in
[`doc/troubleshooting/intellij-idea-stuck-shutdown.md`](../troubleshooting/intellij-idea-stuck-shutdown.md).

## Setup

Status:
TODO

1.  On the affected Linux KDE Plasma desktop,
    open **JetBrains Toolbox**.
    The Toolbox window should list installed JetBrains applications.
2.  Find **IntelliJ IDEA Ultimate** or **IntelliJ IDEA**.
    The entry should show an installed version rather than only **Install**.
3.  If IDEA is not installed,
    select **Install**.
    The entry should change to an installed version with a **Launch** action.
    Stop this runbook because a missing installation is not the stuck-shutdown failure.
4.  Open a terminal with **Ctrl+Alt+T**.
    A shell prompt should appear in the active terminal window.
5.  Define the Toolbox installation paths:

    ```bash
    export IDEA_ROOT="$HOME/.local/share/JetBrains/Toolbox/apps/intellij-idea"
    export IDEA_BIN="$IDEA_ROOT/bin/idea"
    printf 'IDEA_ROOT=%s\nIDEA_BIN=%s\n' "$IDEA_ROOT" "$IDEA_BIN"
    test -x "$IDEA_BIN" && printf '%s\n' 'IDEA executable found'
    ```

    The final line should be exactly:

    ```text
    IDEA executable found
    ```

6.  Locate the newest IDEA log:

    ```bash
    export IDEA_LOG="$(
      find "$HOME/.cache/JetBrains" -maxdepth 4 -type f \
        -path '*/IntelliJIdea*/log/idea.log' \
        -printf '%T@ %p\n' \
        | sort --numeric-sort --reverse \
        | head --lines=1 \
        | cut --delimiter=' ' --fields=2-
    )"
    printf 'IDEA_LOG=%s\n' "$IDEA_LOG"
    test -f "$IDEA_LOG" && printf '%s\n' 'IDEA log found'
    ```

    The final line should be exactly:

    ```text
    IDEA log found
    ```

## Steps

Status:
TODO

1.  Detect an existing root IDEA process,
    including one launched with a file or URL argument:

    ```bash
    export IDEA_PID="$(pgrep --oldest --full "^${IDEA_BIN}( |$)" || true)"
    printf 'IDEA_PID=%s\n' "${IDEA_PID:-none}"
    ```

    The output is either `IDEA_PID=none` or one numeric process ID.
2.  If step 1 printed `IDEA_PID=none`,
    select **Launch** in **JetBrains Toolbox**.
    An IDEA project or welcome window should appear.
    Skip to **What to check** because no old process needed termination.
3.  If step 1 printed a numeric process ID,
    probe its activation endpoint:

    ```bash
    activation_output="$("$IDEA_BIN" 2>&1)"
    activation_status=$?
    printf '%s\nexit=%s\n' "$activation_output" "$activation_status"
    ```

    A healthy instance returns `exit=0` and focuses an IDEA window.
    The stuck-shutdown state prints exactly:

    ```text
    IDE is being shut down
    exit=16
    ```

4.  If the step 3 command does not return through IDEA's own startup handshake,
    press **Ctrl+C** once.
    The terminal should show `^C` and return to its prompt.
    Stop this runbook because an unresponsive activation endpoint is a different failure.
5.  If step 3 returned `exit=0`,
    stop the recovery steps and continue at **What to check**.
    The existing IDEA instance is healthy and must not be terminated.
6.  If step 3 returned anything other than the exact stuck-shutdown result,
    stop this runbook.
    Do not kill the process because a different exit code needs a different diagnosis.
7.  Inspect the affected process and shutdown messages:

    ```bash
    ps --pid "$IDEA_PID" --format=pid,ppid,state,etimes,command
    grep --fixed-strings 'Station requested IDE shutdown' "$IDEA_LOG" \
      | tail --lines=5 \
      || true
    ```

    The process row should name `$IDEA_ROOT/bin/idea`.
    A matching log line has this form:

    ```text
    Station requested IDE shutdown. force = false, restart = false
    ```

8.  If any accessible IDEA window contains work that must be preserved,
    select **File**,
    then **Save All**.
    The save indicators should clear before continuing.
9.  Decide whether losing unsaved IDE state is acceptable.
    Continue only with explicit authorization to terminate every process under `$IDEA_ROOT`.
    No process changes occur in this step.
10. Create a private evidence directory before terminating IDEA:

    ```bash
    mkdir --parents "$HOME/temp"
    chmod 700 "$HOME/temp"
    export IDEA_EVIDENCE="$(mktemp --directory "$HOME/temp/idea-shutdown.XXXXXXXX")"
    chmod 700 "$IDEA_EVIDENCE"
    printf 'IDEA_EVIDENCE=%s\n' "$IDEA_EVIDENCE"
    ```

    The output path should begin with `$HOME/temp/idea-shutdown.`.
11. Copy the current IDEA log into the evidence directory:

    ```bash
    cp -- "$IDEA_LOG" "$IDEA_EVIDENCE/idea.log"
    chmod 600 "$IDEA_EVIDENCE/idea.log"
    test -s "$IDEA_EVIDENCE/idea.log" && printf '%s\n' 'IDEA log captured'
    ```

    The final line should be exactly:

    ```text
    IDEA log captured
    ```

12. Capture the stuck process's thread dump:

    ```bash
    if test -x "$IDEA_ROOT/jbr/bin/jcmd"; then
      "$IDEA_ROOT/jbr/bin/jcmd" "$IDEA_PID" Thread.print \
        >"$IDEA_EVIDENCE/thread-dump.txt" 2>&1
      dump_status=$?
    else
      printf '%s\n' 'Bundled jcmd is missing' >"$IDEA_EVIDENCE/thread-dump.txt"
      dump_status=1
    fi
    chmod 600 "$IDEA_EVIDENCE/thread-dump.txt"
    printf 'thread_dump_exit=%s\n' "$dump_status"
    ```

    `thread_dump_exit=0` means the dump succeeded.
    Keep `thread-dump.txt` even when the status is nonzero because it contains the attach error.
13. Preview every process that the termination pattern will match:

    ```bash
    pgrep --list-full --full "^${IDEA_ROOT}/" \
      || printf '%s\n' 'No IDEA processes remain'
    ```

    Continue only if every listed command begins with `$IDEA_ROOT/`.
    The list must not contain Toolbox or another JetBrains product.
14. Send SIGTERM to the previewed IDEA installation process set:

    ```bash
    mapfile -t IDEA_PIDS < <(pgrep --full "^${IDEA_ROOT}/" || true)
    printf 'Sending SIGTERM to: %s\n' "${IDEA_PIDS[*]}"
    if test "${#IDEA_PIDS[@]}" -gt 0; then
      /usr/bin/kill --signal TERM "${IDEA_PIDS[@]}"
    fi
    ```

    IDEA windows may close.
15. Check for surviving IDEA processes after a bounded wait:

    ```bash
    attempts=0
    while pgrep --full "^${IDEA_ROOT}/" >/dev/null && test "$attempts" -lt 20; do
      sleep 0.25
      attempts=$((attempts + 1))
    done
    pgrep --list-full --full "^${IDEA_ROOT}/" \
      || printf '%s\n' 'No IDEA processes remain'
    ```

    If SIGTERM succeeded,
    the output is exactly:

    ```text
    No IDEA processes remain
    ```

16. If step 15 still listed IDEA processes,
    reconfirm authorization for forced termination.
    No process changes occur in this step.
17. Preview the surviving process set again:

    ```bash
    pgrep --list-full --full "^${IDEA_ROOT}/" \
      || printf '%s\n' 'No IDEA processes remain'
    ```

    Continue only if every listed command still begins with `$IDEA_ROOT/`.
18. If authorized and processes survived SIGTERM,
    send SIGKILL to the previewed IDEA installation process set:

    ```bash
    mapfile -t IDEA_PIDS < <(pgrep --full "^${IDEA_ROOT}/" || true)
    printf 'Sending SIGKILL to: %s\n' "${IDEA_PIDS[*]}"
    if test "${#IDEA_PIDS[@]}" -gt 0; then
      /usr/bin/kill --signal KILL "${IDEA_PIDS[@]}"
    fi
    ```

    Any remaining IDEA windows should close immediately.
19. Confirm that no IDEA installation process remains after a bounded wait:

    ```bash
    attempts=0
    while pgrep --full "^${IDEA_ROOT}/" >/dev/null && test "$attempts" -lt 20; do
      sleep 0.25
      attempts=$((attempts + 1))
    done
    pgrep --list-full --full "^${IDEA_ROOT}/" \
      || printf '%s\n' 'No IDEA processes remain'
    ```

    Continue only when the output is exactly:

    ```text
    No IDEA processes remain
    ```

    If a process survives SIGKILL,
    stop this runbook and do not delete `.lock` or `.port` files.
    Save work in unrelated applications and use KDE's normal **Restart** action.
    The KDE login screen should appear after the restart.
    After login,
    resume at **Setup** step 5 to restore the shell variables before continuing.
20. Locate the Toolbox desktop entry whose `Exec` line names the current IDEA binary:

    ```bash
    export IDEA_DESKTOP_FILE="$(
      find "$HOME/.local/share/applications" -maxdepth 1 -type f \
        -name 'jetbrains-idea-*.desktop' \
        -exec grep --files-with-matches --fixed-strings "Exec=\"$IDEA_BIN\"" {} + \
        | head --lines=1
    )"
    if test -n "$IDEA_DESKTOP_FILE"; then
      export IDEA_DESKTOP_ID="$(basename "$IDEA_DESKTOP_FILE" .desktop)"
    else
      export IDEA_DESKTOP_ID=''
    fi
    printf 'IDEA_DESKTOP_ID=%s\n' "${IDEA_DESKTOP_ID:-none}"
    ```

    The value should start with `jetbrains-idea-` or be exactly `none`.
21. If step 20 found an ID that starts with `jetbrains-idea-`,
    launch IDEA through that desktop entry:

    ```bash
    gtk-launch "$IDEA_DESKTOP_ID"
    ```

    An IDEA project or welcome window should appear.
22. If step 20 printed `IDEA_DESKTOP_ID=none`,
    select **Launch** for IDEA in **JetBrains Toolbox** instead.
    An IDEA project or welcome window should appear.

## What to check

Status:
TODO

1.  Confirm that exactly one root IDEA process is running:

    ```bash
    pgrep --list-full --full "^${IDEA_BIN}( |$)"
    ```

    The output should contain one numeric PID followed by `$IDEA_BIN`,
    optionally with a file or URL argument.
2.  Probe the recovered activation endpoint:

    ```bash
    activation_output="$("$IDEA_BIN" 2>&1)"
    activation_status=$?
    printf '%s\nexit=%s\n' "$activation_output" "$activation_status"
    ```

    The exact status should be:

    ```text
    exit=0
    ```

    `IDE is being shut down` must not appear.
3.  Confirm that IDEA accepted the external command:

    ```bash
    grep --fixed-strings 'External instance command received' "$IDEA_LOG" \
      | tail --lines=1
    ```

    The output should contain exactly this message text:

    ```text
    External instance command received
    ```

4.  On KDE Wayland,
    create a temporary KWin verification script:

    ```bash
    export IDEA_KWIN_SCRIPT="$XDG_RUNTIME_DIR/verify-idea-window.js"
    tee "$IDEA_KWIN_SCRIPT" >/dev/null <<'EOF'
    const marker = "IDEA_RUNBOOK_WINDOW:";
    const windows = workspace.windowList().filter(
      (window) => String(window.resourceClass).toLowerCase() === "jetbrains-idea"
    );
    console.info(`${marker} count=${windows.length}`);
    windows.forEach((window) => {
      window.minimized = false;
      workspace.activeWindow = window;
      console.info(`${marker} minimized=${window.minimized} active=${window.active}`);
    });
    EOF
    ```

    The file should exist at `$XDG_RUNTIME_DIR/verify-idea-window.js`.
5.  Load and run the KWin verification script:

    ```bash
    qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript \
      "$IDEA_KWIN_SCRIPT" idea-runbook-window
    qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.start
    ```

    KWin should return a numeric script ID and no error.
6.  Read the KWin verification result:

    ```bash
    journalctl --boot --no-pager --output=cat \
      | grep --fixed-strings 'IDEA_RUNBOOK_WINDOW:' \
      | tail --lines=5
    ```

    Expect a nonzero window count and at least one exact state line:

    ```text
    IDEA_RUNBOOK_WINDOW: minimized=false active=true
    ```

7.  Confirm visually that an IDEA editor or welcome window is visible and active.
    The window should be in front and should not be minimized.

## Restore

Status:
TODO

1.  Unload the temporary KWin script:

    ```bash
    qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript \
      idea-runbook-window \
      || true
    ```

    The command should return without an error dialog.
2.  Delete the temporary KWin script:

    ```bash
    rm --force -- "$IDEA_KWIN_SCRIPT"
    test ! -e "$IDEA_KWIN_SCRIPT" && printf '%s\n' 'Temporary KWin script removed'
    ```

    The output should be exactly:

    ```text
    Temporary KWin script removed
    ```

3.  Print the private evidence directory retained for diagnosis:

    ```bash
    printf 'IDEA_EVIDENCE=%s\n' "$IDEA_EVIDENCE"
    ```

    Keep this directory until the shutdown blocker has been diagnosed or reported.
4.  After the evidence is no longer needed,
    confirm that the printed path begins with `$HOME/temp/idea-shutdown.`.
    No files change in this step.
5.  Delete only the confirmed evidence directory:

    ```bash
    rm --recursive --force -- "$IDEA_EVIDENCE"
    test ! -e "$IDEA_EVIDENCE" && printf '%s\n' 'IDEA evidence removed'
    ```

    The output should be exactly:

    ```text
    IDEA evidence removed
    ```

6.  Clear the shell-only runbook variables:

    ```bash
    unset IDEA_BIN IDEA_DESKTOP_FILE IDEA_DESKTOP_ID IDEA_EVIDENCE IDEA_KWIN_SCRIPT \
      IDEA_LOG IDEA_PID IDEA_PIDS IDEA_ROOT
    ```

    No output is expected.
7.  Leave IDEA running.
    This runbook does not alter IDEA settings,
    plugins,
    caches,
    `.lock`,
    or `.port` files,
    so no application-state restoration is required.
