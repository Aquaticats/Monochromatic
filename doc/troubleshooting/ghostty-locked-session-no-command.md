# Ghostty 1.3.1 runs no `-e` command while the screen locker holds the session: the pty child waits on a GLArea resize

## Symptom

`ghostty -e <command>` launched from an agent session appears to open a window
that "instantly dies",
 and the command never runs.
What is actually on the system:

-   a live `ghostty` process,
     minutes old,
     sleeping in `poll`,
     23 threads,
    `/dev/dri/renderD128` open three times;
-   ZERO child processes;
-   no output from the command,
     and no file the command was told to write.

Observed shell transcript,
 with the process still alive after three minutes:

```sh
$ ps --no-headers -eo pid,etimes,args | rg 'title=diag-lc'
 984097  192 ghostty --gtk-single-instance=false --title=diag-lc -e bash -lc pi ...
$ cat /proc/984097/task/*/children
                       # empty: no child was ever spawned
$ rg '^State' /proc/984097/status
State:	S (sleeping)
$ cat /proc/984097/wchan
poll_schedule_timeout.constprop.0
```

There is no error message anywhere:
 not on the launching shell's stderr,
 not in
the redirect the command was given,
 not in the journal.
The failure is silent in both directions,
 which is why it reads as "the window
died" rather than "the window never got a surface".

## Root cause

Ghostty's GTK apprt does not create the terminal until the OpenGL area has been
realized AND resized once,
 and the terminal is what spawns the `-e` command.
While the KDE screen locker holds the session,
 a newly mapped toplevel never
reaches that point,
 so the process sits in its event loop forever with no child.

The core surface starts out null,
 and the comment says exactly why
(`src/apprt/gtk/class/surface.zig:618`):

```zig
        /// The core surface backing this GTK surface. This starts out
        /// null because it can't be initialized until there is an available
        /// GLArea that is realized.
        //
        // NOTE(mitchellh): This is a limitation we should definitely remove
        // at some point by modifying our OpenGL renderer for GTK to
        // start in an unrealized state. There are other benefits to being
        // able to initialize the surface early so we should aim for that,
        // eventually.
        core_surface: ?*CoreSurface = null,
```

Realizing the GLArea is not enough on its own.
 `glareaRealize` says the
initialization is deferred again,
 to the first resize
(`src/apprt/gtk/class/surface.zig:3226`):

```zig
        // If we already have an initialized surface then we notify it.
        // If we don't, we'll initialize it on the first resize so we have
        // our proper initial dimensions.
        if (priv.core_surface) |v| realize: {
```

The resize handler is what finally builds it
(`src/apprt/gtk/class/surface.zig:3293` for the handler,
 and the call at its
tail):

```zig
        // If we don't have a surface, then we initialize it.
        self.initSurface() catch |err| {
            log.warn("surface failed to initialize err={}", .{err});
        };
```

```zig
    fn initSurface(self: *Self) InitError!void {
        const priv: *Private = self.private();
        assert(priv.core_surface == null);
```

`CoreSurface` owns the termio and therefore the pty and the child process.
The field is declared on the core surface (`src/Surface.zig:119`):

```zig
/// The terminal IO handler.
io: termio.Termio,
io_thread: termio.Thread,
```

`Surface.init` is the only place it is initialized,
 with the exec backend
(`src/Surface.zig:654`):

```zig
        try termio.Termio.init(&self.io, alloc, .{
            .size = size,
            .full_config = config,
            .config = try termio.Termio.DerivedConfig.init(alloc, config),
            .backend = .{ .exec = io_exec },
```

and the same function spawns the thread that runs it (`src/Surface.zig:712`):

```zig
    self.io_thr = try std.Thread.spawn(
        .{},
        termio.Thread.threadMain,
        .{ &self.io_thread, &self.io },
    );
```

The child process is spawned from that thread and nowhere else
(`src/termio/Exec.zig:84`,
 whose `threadEnter` runs on entry to the IO thread):

```zig
pub fn threadEnter(
    self: *Exec,
    alloc: Allocator,
    io: *termio.Termio,
    td: *termio.Termio.ThreadData,
) !void {
    // Start our subprocess
    const pty_fds = self.subprocess.start(alloc) catch |err| {
```

`Subprocess.start` is what opens the pty and forks (`src/termio/Exec.zig:884`):

```zig
    pub fn start(self: *Subprocess, alloc: Allocator) !struct {
        read: Pty.Fd,
        write: Pty.Fd,
    } {
        assert(self.pty == null and self.process == null);
```

So no realize plus resize means no core surface,
 no `Surface.init`,
 no termio,
 no
IO thread,
 and therefore no `fork`/`exec` of the `-e` command at all.
That is consistent with every measurement above:
 GTK and the GL context are up,
which is why the DRM render node is open,
 and nothing downstream of the surface
exists.

WHAT DISPROVED THE OBVIOUS READINGS,
 recorded so the next investigator does not
re-derive them:

-   "The login shell kills it."
     A `bash -lc` window and a `bash -c` window
    launched together both wrote their marker files and survived,
     while the
    screen was unlocked.
     The login shell is not involved.
-   "`pi` is not on PATH under a login shell."
     `bash -lc 'which pi'` resolves it.
-   "The argument is too long."
     The largest prompt passed was 91606 bytes against
    a `MAX_ARG_STRLEN` of 131072,
     and the same argument works when the session is
    unlocked.
-   "Too many ghostty instances have accumulated."
     Killing the stalled instances
    and relaunching with four alive still stalled.

The finding that settled it was a POSITIVE CONTROL:
 the simplest possible
command,
 `bash -c 'echo CTL1 > mark; sleep 60'`,
 in exactly the form that had
worked twenty minutes earlier,
 also stalled.
 Since the command form had been
ruled out,
 the difference had to be session state,
 and it was:

```sh
$ dbus-send --session --print-reply --dest=org.freedesktop.ScreenSaver \
    /org/freedesktop/ScreenSaver org.freedesktop.ScreenSaver.GetActive
   boolean true
```

Note that `loginctl` disagrees with the compositor here and must not be trusted
for this check:
 it reported `LockedHint=no`,
 `Active=yes`,
 `State=active` for the
session at the same moment the screen locker reported active.

## Verification

Version under test:
 Ghostty 1.3.1 (`ghostty --version`),
 source read at tag
`v1.3.1`,
 commit `332b2ae`,
 on Fedora with KDE Plasma on Wayland
(`WAYLAND_DISPLAY=wayland-0`,
 `XDG_SESSION_TYPE` Wayland,
 `plasmalogin`).

Harness,
 which needs no external tool and distinguishes the two states:

```sh
# Is the compositor's screen locker holding the session?
dbus-send --session --print-reply --dest=org.freedesktop.ScreenSaver \
  /org/freedesktop/ScreenSaver org.freedesktop.ScreenSaver.GetActive

# Launch a window whose command proves it ran.
rm --force ~/temp/agent/ctl.mark
setsid --fork ghostty --gtk-single-instance=false --title=ctl \
  -e bash -c 'echo RAN > ~/temp/agent/ctl.mark; sleep 60' > /dev/null 2>&1
sleep 7
cat ~/temp/agent/ctl.mark 2>/dev/null || echo STALLED

# When it stalls, confirm the shape rather than guessing:
pid=$(pgrep --newest --full 'title=ctl')
cat /proc/"${pid}"/task/*/children   # empty means no command was spawned
```

WORKS,
 with the screen locker inactive:

-   `ghostty -e bash -c '<command>'`
-   `ghostty -e bash -lc '<command>'`,
     login shell included
-   `ghostty -e hx <file>`
-   a 91606-byte single argument built by `"$(cat prompt-file)"`
-   several instances launched together with `--gtk-single-instance=false`

STRENGTH OF THIS HALF OF THE CATALOG:
 the failing half was measured against a
`GetActive` reading of `true` taken at the same moment.
The working half was not;
 it is a timing correlation,
 since those runs happened
while the user was at the machine and the screen was demonstrably in use.
The reading is almost certainly `false` there,
 but nobody captured it.
CLOSING THIS COSTS ONE HARNESS RUN:
 while the session is unlocked,
 run the block
above and record the `GetActive` line beside the `RAN` marker.
Do that before treating the correlation as measured.

STALLS,
 with the screen locker active,
 all of them silently and identically:

-   every one of the above,
     including the trivial `echo` case
-   with and without `--gtk-single-instance=false`
-   with stdout and stderr redirected to files,
     which stay absent because
    nothing runs to open them

## Verified workarounds

1.  ASK THE COMPOSITOR FIRST,
      and do not open a window when it says locked.
     Tradeoff:
      it is a KDE-specific interface.
      `org.freedesktop.ScreenSaver` is
     honored by KDE and GNOME but is not universal,
      and a compositor without it
     returns an error rather than `false`,
      so treat a failed call as "unknown"
     and fall back to the next workaround rather than as "unlocked".

     ```sh
     locked=$(dbus-send --session --print-reply --dest=org.freedesktop.ScreenSaver \
       /org/freedesktop/ScreenSaver org.freedesktop.ScreenSaver.GetActive 2>/dev/null \
       | tail --lines=1 | rg --only-matching 'true|false')
     ```

2.  PROVE THE COMMAND RAN,
      rather than assuming a launched window works.
      Have
     the command write a marker as its first action and check for the marker a
     few seconds later.
      Tradeoff:
      it costs a fixed wait per launch and needs the
     command to cooperate;
      it catches every cause of a non-starting window,
      not
     only this one,
      which is why it is worth the wait.

3.  DO NOT NEED A WINDOW.
      For anything whose output an agent will read anyway,
     run the tool headless into a file (`pi --print ... > out.log`) and read the
     file.
      Tradeoff:
      the session is no longer interactive,
      so a human cannot
     steer it mid-run,
      which is the whole reason the window was wanted.

4.  KEEP THE WINDOW ALIVE PAST THE COMMAND with a trailing `sleep`,
      so a window
     that DID start is still there to be read.
      Tradeoff:
      this addresses a
     different failure (a window vanishing because its command finished) and
     does nothing for the locked case;
      it was mistaken for a fix once already
     during this investigation.

## What does not work

-   `--gtk-single-instance=false` on its own.
     It is genuinely needed when a
     primary instance is running with that flag,
     since otherwise the launch is
     handed to the existing instance,
     but it does not affect the locked case.
-   Killing the stalled instances and retrying.
     Four alive or seven,
     the next
     launch stalls the same way.
-   Redirecting stderr to a file to find the error.
     There is no error:
     the
     process never reaches the command.
-   Reading `loginctl show-session ... --property=LockedHint`.
     It reported `no`
     while the screen locker reported active,
     so it answers a different question
     and will send an investigator down the wrong path.
-   Waiting longer.
     Instances observed at three minutes and at forty seconds were
     both still childless,
     and neither recovered on its own.

## Upstream filing decision

Checked `.out-of-scope/` first:
 the only file mentioning any of these tools is
`.out-of-scope/claude-code-upstream-bugs.md`,
 which exempts Claude Code,
 not
Ghostty.
 No exemption matches,
 so the six constraints apply.

1.  IS IT REALLY UPSTREAM'S FAULT?
      PARTLY.
      A compositor that refuses to map a
     window while the session is locked is behaving as designed,
      and no terminal
     can paint under a lock screen.
      What belongs to Ghostty is that the CHILD
     PROCESS is coupled to the surface:
      `-e` is a request to run a command,
      and a
     user reasonably expects the command to run and its output to be waiting when
     the screen is unlocked.
      Ghostty's own source calls the coupling a limitation
     to remove (`src/apprt/gtk/class/surface.zig:621`).
2.  CAN UPSTREAM FIX IT?
      YES,
      and they have already described the fix in that
     NOTE:
      let the renderer start unrealized so the core surface,
      and therefore
     the pty,
      can be initialized before the GLArea exists.
      Large and structural,
     which the constraint explicitly allows.
3.  ARE THEY SUPPORTING THIS USE CASE?
      YES for `-e`,
      which is a documented
     command-line option with an example in `ghostty --help`.
      Running one while
     the screen is locked is not documented either way.
4.  WOULD THE REPO WELCOME OUR CONTRIBUTION?
      NOT CHECKED YET.
      `CONTRIBUTING.md`,
     the issue templates and any AI-assistance policy in the clone at
     `${HOME}/temp/agent/ghostty-2026-08-15` have not been read,
      and this
     constraint cannot be assumed.
5.  WILL THEY LIKELY FIX IT?
      UNKNOWN.
      The upstream tracker has not been searched
     for this behavior.
6.  HAVE WE PROTOTYPED A MINIMAL FIX?
      NO.

So constraints 4 and 5 are unevaluated rather than met,
 and the auto-prototype
trigger ("1 through 5 hold or sorta-hold") is not reached.
 NOTHING IS FILED,
 and
no draft issue is kept,
 because a draft written before the duplicate search and
the contribution-policy read would be a draft nobody should send.

WHAT THE NEXT SESSION SHOULD DO IF THIS IS WORTH PURSUING,
 in order:
 read
`CONTRIBUTING.md` and the issue templates in the clone;
 run
`gh search issues --repo ghostty-org/ghostty` and `gh search prs` over both open
and closed state for terms from the symptom (`-e` command not running,
 no child
process,
 locked session,
 screen locker) and from the cause (core surface
initialized on first resize,
 unrealized GLArea);
 read any matching thread in
full;
 and only then decide between an additive comment,
 a new issue,
 and nothing.
The consumer-side workarounds above stand regardless of upstream movement,
 which
is why none of this blocks the work that found it.
