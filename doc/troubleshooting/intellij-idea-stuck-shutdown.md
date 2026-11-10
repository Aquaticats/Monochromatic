# IntelliJ IDEA 2026.2.1: launching during a stuck shutdown exits 16 without opening a window

## Symptom

IntelliJ IDEA appears not to launch from its Toolbox desktop entry.
The Toolbox launcher path exists and points at the current installation,
but a direct launch prints this message and exits with status 16:

```text
IDE is being shut down
```

The observed machine still had an IntelliJ IDEA process from the prior day.
IDEA had received repeated non-forced shutdown requests,
but the process remained alive and continued to own the single-instance endpoint:

```text
2026-08-12 21:10:39,974  Station requested IDE shutdown. force = false, restart = false
2026-08-12 21:12:00,652  Station requested IDE shutdown. force = false, restart = false
```

Subsequent launch attempts therefore contacted that old process instead of creating a new IDE process.

## Root cause

The immediate launch failure was a stuck existing IDEA instance that had entered the shutdown lifecycle.
The evidence does not identify the component that blocked that shutdown.

Source findings use the shared platform source from
[JetBrains/intellij-community][] at tag `idea/262.9437.185`,
commit `b75ab523e6adbe1d26112219729eacbcfd24daa0`,
which is also tagged `idea/2026.2.1` and matches the installed Ultimate build.

### Step 1: a new launcher delegates to an existing instance

`platform/platform-impl/bootstrap/src/com/intellij/platform/ide/bootstrap/DirectoryLock.java:162-168`
documents that `lockOrActivate` first tries to own the endpoint and otherwise contacts the running instance:

```java
/**
 * Tries to grab a port file (thus locking the directories) and start listening for incoming requests.
 * Failing that, attempts to connect via the existing port file to an already running instance.
 */
public @Nullable CliResult lockOrActivate(
  @NotNull Path currentDirectory,
  @NotNull List<String> args
)
```

`DirectoryLock.java:334-359` sends the new invocation to that instance and returns its exit code and message:

```java
LOG.debug("connecting to " + address);
// ...
request.add(currentDirectory.toString());
request.addAll(args);
sendLines(socketChannel, request);
// ...
var exitCode = Integer.parseInt(response.get(0));
var message = response.get(1);
return new CliResult(exitCode, message.isEmpty() ? null : message);
```

The observed old process was responsive enough to return a structured result.
This was not a stale lock file or a refused connection.

### Step 2: shutdown replaces normal activation with a disposing response

`platform/platform-impl/src/com/intellij/openapi/application/impl/ApplicationImpl.java:865-870`
invokes lifecycle listeners after shutdown has begun:

```java
try {
  lifecycleListener.appWillBeClosed(restart);
}
catch (Throwable t) {
  logErrorDuringExit("Failed to invoke lifecycle listeners", t);
}
```

`platform/platform-impl/bootstrap/src/com/intellij/platform/ide/bootstrap/ApplicationLoader.kt:566-570`
replaces the normal external-command handler at that event:

```kotlin
override fun appWillBeClosed(isRestart: Boolean) {
  setActivationListener {
    CompletableDeferred(
      CliResult(AppExitCodes.ACTIVATE_DISPOSING, IdeBundle.message("activation.shutting.down"))
    )
  }
}
```

`platform/core-api/src/com/intellij/idea/AppExitCodes.java:20-23` assigns status 16 to that response:

```java
public static final int ACTIVATE_NOT_INITIALIZED = 14;
public static final int ACTIVATE_ERROR = 15;
public static final int ACTIVATE_DISPOSING = 16;
```

Finally,
`platform/platform-impl/bootstrap/src/com/intellij/platform/ide/bootstrap/startup.kt:574-576`
prints the response and terminates the new launcher:

```kotlin
else {
  result.message?.let { println(it) }
  exitProcess(result.exitCode)
}
```

The complete launch-failure chain was therefore:
the old process retained the single-instance endpoint,
the new process connected to it,
the old process returned `ACTIVATE_DISPOSING`,
and the new process printed `IDE is being shut down` before exiting with status 16.

### What remains unknown

The old log contained repeated JetBrains AI Assistant unload exceptions before the shutdown requests.
Those exceptions are correlated with the incident but do not prove that the plugin blocked shutdown.
A thread dump was not captured before the authorized kill,
so attributing the hang to that plugin or to any later shutdown listener would be speculation.

## Verification

Verified on 2026-08-12 with:

```text
IntelliJ IDEA 2026.2.1
Build #IU-262.9437.185
JDK 25.0.3, JetBrains Runtime
Linux, KDE Plasma Wayland session
```

### Failing catalog before recovery

A direct invocation reproduced the user-visible failure:

```bash
"$HOME/.local/share/JetBrains/Toolbox/apps/intellij-idea/bin/idea"
printf 'exit=%s\n' "$?"
```

```text
IDE is being shut down
exit=16
```

The existing root IDEA process remained alive after repeated shutdown requests.
Repeated launch attempts therefore exercised the same disposing response rather than starting a fresh process.

### Working catalog after recovery

After terminating the stuck IDEA process tree,
the desktop entry launched a new root process:

```bash
gtk-launch jetbrains-idea-e37c368f-a232-4031-8958-0c5d7640c421
```

Verification crossed the window-manager boundary.
The following one-shot KWin script used IDEA's verified `resourceClass`,
found two IDEA windows,
unminimized them,
and activated each:

```js
// $HOME/temp/agent/verify-idea-window.js
const marker = "PI_IDEA_WINDOW_K7M:";
const ideaWindows = workspace.windowList().filter(
  (window) => String(window.resourceClass).toLowerCase() === "jetbrains-idea"
);
console.info(`${marker} count=${ideaWindows.length}`);
ideaWindows.forEach((window) => {
  window.minimized = false;
  workspace.activeWindow = window;
  console.info(
    `${marker} activated=true minimized=${window.minimized} active=${window.active}`
  );
});
```

It was loaded,
run,
read from KWin's journal output,
and unloaded with:

```bash
qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript \
  "$HOME/temp/agent/verify-idea-window.js" pi-verify-idea-window
qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.start
journalctl --boot --no-pager --output=cat --since='1 minute ago' \
  | rg --fixed-strings 'PI_IDEA_WINDOW_K7M:'
qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript \
  pi-verify-idea-window
```

```text
count=2
activated=true minimized=false active=true
activated=true minimized=false active=true
```

The IDEA log also recorded two project frames during startup.
A second direct launcher invocation then delegated to the healthy instance and returned normally:

```text
exit=0
External instance command received: count changed from 0 to 1
```

## Verified workarounds

### Terminate the stuck process, then relaunch

First confirm that the running IDEA instance returns `IDE is being shut down` with status 16.
If it remains in that state and the user authorizes possible unsaved-data loss,
terminate that IDEA installation's process tree and relaunch through its desktop entry.

In this incident,
SIGTERM stopped the child processes but not the root IDEA process.
SIGKILL removed the root process,
and the next desktop launch succeeded without deleting settings or caches.

Tradeoff:
forced termination can lose unsaved editor state and interrupt active IDE tasks.
Use it only after confirming that the process is stuck and obtaining authorization.

### Wait for a genuine shutdown to finish

Status 16 is expected while an ordinary shutdown is still progressing.
Waiting avoids data loss when the process is making progress and exits on its own.

Tradeoff:
waiting does not recover an instance that remains alive indefinitely.
Confirm process age and repeat the launcher probe before choosing this path.

## Automatic recovery launcher installed on 2026-11-02

The user requested that each desktop launch terminate the current or stuck IDEA processes
and start a fresh IDE instance.
This explicitly changes the shortcut from activating an existing window to restarting IDEA,
including when the existing instance is healthy.
The installed version is IDEA 2026.3 RC,
build `263.7307.112`,
as recorded in the local Toolbox installation's `product-info.json`.
The historical shutdown diagnosis in this document has not been reproduced on that build.

The local recovery implementation lives in
`$HOME/.local/share/intellij-recovery/restart.ts`
and `$HOME/.local/share/intellij-recovery/processes.ts`.
It records actions in `$HOME/.local/state/intellij-recovery/recovery.log`.
It targets current-user processes whose executable or script belongs to the Toolbox IDEA installation
or an IDEA cache directory,
plus their descendants.
It sends `SIGTERM`,
allows 8 seconds for shutdown,
then sends `SIGKILL` to survivors and allows 3 seconds for exit.
It checks process start times before signalling,
refuses to kill its own caller,
and refuses a new instance if matching processes remain.
It preserves settings,
plugins,
indexes,
coordination files,
and Local History.

The original Toolbox desktop entry invokes this wrapper.
A separate `intellij-idea-recover.desktop` entry preserves access if Toolbox rewrites its generated shortcut.
Its application-menu name is **IntelliJ IDEA (Recover and Restart)**.
A backup of the unmodified entry is
`$HOME/.local/state/intellij-recovery/original-launcher-1793681275733.desktop`.
A nonblocking `flock` serializes recovery,
with a 5-second hold after launch to absorb double-clicks.
Recovery runs with `CPUQuota=100%`,
`MemoryMax=512M`,
`TasksMax=128`,
and `IOWeight=25`.
The new IDEA process runs in a separate user service with `CPUQuota=2800%`,
`MemoryMax=40G`,
`TasksMax=8192`,
and `IOWeight=50`.
The host reports 32 online CPUs and 63144 MiB RAM;
the recovery shell itself reports a 2-CPU allocation.

Tradeoffs:
each launch interrupts IDE tasks and can lose unsaved edits.
Descendant terminals and agents close too.
An uninterruptible process can prevent recovery;
the wrapper reports surviving process IDs instead of deleting locks or launching against a live owner.
The Toolbox application's own launch button still uses its own launcher.

The disposable harness is `$HOME/temp/agent/test-intellij-recovery.ts`.
It verified graceful termination,
forced termination of a process that ignores `SIGTERM`,
descendant cleanup,
preservation of an unrelated process,
caller protection,
rejection of shell-command text containing an IDEA path,
rejection of a remembered PID with a different start time,
desktop URI/path forwarding,
double-click exclusion,
and replacement through the unmodified wrapper into a running systemd service.
Fixture cleanup reported no remaining matching processes.
Desktop entries passed `desktop-file-validate`.
The changed original desktop entry was then launched through `gio launch`.
The recovery log recorded termination of the old root PID `2448935`
and creation of `intellij-idea-recovered-1793681333401.service`.
The old root PID disappeared;
`systemctl --user show` reported the service active with main PID `3235452`.
A KWin D-Bus script confirmed an IDEA window belonging to that new PID:

```text
# KWin journal output, 2026-11-02 America/New_York
IDEA_RECOVERY_20261102 count=1
IDEA_RECOVERY_20261102 pid=3235452 minimized=false active=false
```

The KWin verification script was unloaded after inspection.
The live restart completed with `SIGTERM`;
the forced termination path was exercised by the disposable resistant process.
The documentation passed scoped `mise run lint:monochromatic-lint`
and a rendered check with `glow`.

The fixture initially expected GIO to forward a `file:` URL verbatim;
the measured argument was its decoded filesystem path.
The corrected assertion checks that path and separately checks a `jetbrains:` link.
Another failed assertion tested the desktop lock after the child had already exited:
the harness captured inherited output pipes with `execFileSync`.
Using ignored output handles allowed the harness to test the lock while the child was still alive.
The resulting live-lock assertion passed.
Both wrapper invocations use `systemd-run --expand-environment=no`
to preserve literal dollar expressions in arguments.
The desktop test forwarded a link containing `$HOME` and a semicolon unchanged.
A separate verification command that omitted this option emitted
`Referenced but unset environment variable evaluates to an empty string: started`;
rerunning that command with expansion disabled preserved its JavaScript template expression.

This is a requested local launch policy,
not new evidence identifying an upstream shutdown defect.
The existing upstream filing decision therefore remains unchanged.

## What does not work

- **Repeatedly starting IDEA.**
   Each attempt connects to the same old endpoint and receives status 16.
- **Deleting `.lock` or `.port` while the owner is alive.**
   The old instance was responsive,
  so its endpoint was not stale.
   Removing coordination files can allow competing instances against the same config.
- **Deleting config,
   plugins,
   or caches first.**
   The successful relaunch reused all existing state.
  Destructive state reset was unnecessary.
- **Treating the AI Assistant exceptions as the proven cause.**
   Their timing is suggestive,
  but no pre-kill thread dump links them to the shutdown blocker.

## Upstream filing artifact

### Upstream filing decision

The repository's `.out-of-scope/` entries were checked.
None covers IntelliJ IDEA or JetBrains IDE startup failures.

JetBrains directs reproducible bug reports to YouTrack in
`CONTRIBUTING.md` and welcomes bug-fix contributions with tests.
No policy barring AI-assisted reports was found in `CONTRIBUTING.md`,
`README.md`,
 `.github/`,
 or `.ai/`.
The `.ai/` material describes contributor-agent guidance generation rather than a filing restriction.

The duplicate search covered open and closed GitHub issues and pull requests,
YouTrack searches for the exact message,
`ACTIVATE_DISPOSING`,
Station shutdown,
and broader stuck-process startup failures.
No exact report for a responsive instance returning status 16 was found.
The closest tracker cluster is [IJPL-160882][],
an obsolete umbrella for an existing process that does not respond to activation.
Recent reports such as [IJPL-234004][] were closed as duplicates of that umbrella.
That transport failure differs from this incident,
where the old process accepted the connection and intentionally returned `ACTIVATE_DISPOSING`.

The six filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
     Not established.
    IDEA correctly prevented a second instance while shutdown was in progress.
    The component that prevented shutdown completion is unknown and could be an external plugin.
2.  **Can upstream fix it?**
     The launcher and shutdown UX are changeable,
    but no specific blocker or safe source-level correction was identified.
3.  **Are they supporting this use case?**
     Yes.
    The cited source explicitly implements single-instance activation and shutdown responses.
4.  **Would the repo welcome our contribution?**
     Yes for reproducible bug fixes with tests,
    according to `CONTRIBUTING.md`;
     no AI-assistance ban was found.
5.  **Will they likely fix it?**
     Not enough evidence.
    Maintainer comments on IJPL-160882 say a kill button would not solve the cause
    and could create more user problems.
    Its closing comment retires the umbrella in favor of cause-specific tickets.
    Since the 2026.2.1 tag,
    the activation-listener file has two unrelated commits and `DirectoryLock.java` has none.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     No.
    Auto-prototyping is not triggered because constraints 1 and 5 do not hold.
    A source patch without a reproducible shutdown blocker would be speculative.

### Nothing additive to file

Do not file or comment from this incident as-is.
IJPL-160882 already records the user-facing stuck-process recovery problem and the data-loss tradeoff of killing it.
This incident adds a distinct status-16 signal,
but lacks the thread dump needed to identify the shutdown blocker or advance a cause-specific ticket.
A bare report that killing the process worked would not add actionable upstream evidence.

[JetBrains/intellij-community]: https://github.com/JetBrains/intellij-community
[IJPL-160882]: https://youtrack.jetbrains.com/issue/IJPL-160882
[IJPL-234004]: https://youtrack.jetbrains.com/issue/IJPL-234004
