# Ghostty 1.3.1 on Fedora/KDE: `--gtk-single-instance=true` in packaging makes every window share one process, and the fix is a valid `DBusActivatable=false` user desktop entry

Ghostty 1.3.1 (`ghostty-1.3.1-1.fc44.x86_64`,
 GTK app runtime) on KDE Plasma 6.7.1 / KF6 KIO 6.27.0,
Wayland,
 Fedora 44 (Bazzite).
Setting `gtk-single-instance = false` in `~/.config/ghostty/config` has no effect on how ghostty is
launched:
 the Fedora package hardcodes `--gtk-single-instance=true` in its systemd user unit,
 D-Bus
service,
 and desktop file,
 a CLI flag overrides the config value,
 and KDE launches the app by D-Bus
activation of `com.mitchellh.ghostty`,
 which is single-instance by design.
The result is that all windows opened through the normal launcher live in one long-lived server
process,
 so one crash takes down every window at once.
The working fix is to give KDE a valid user-level desktop entry with `DBusActivatable=false` and
`Exec=/usr/bin/ghostty --gtk-single-instance=false`,
 so KDE's KIO launcher runs `Exec` (a fresh
process) instead of activating the shared server.

## Symptom

- `~/.config/ghostty/config` contains `gtk-single-instance = false`,
   yet the running ghostty is
  `/usr/bin/ghostty --gtk-single-instance=true --initial-window=false`.
- Opening terminals (app menu,
   `Ctrl+Alt+T`,
   "open terminal here",
   `kstart --application`) adds a
  window but no new process;
   process count stays flat while window count climbs.
- Editing the entry in KDE Menu Editor (empty Command-line arguments) changes nothing.
- A crash of the one server process closes all of its windows at once.
- If the server unit is masked while KDE still activates it,
   clicking the launcher errors with:
  ```text
  Could not activate remote peer 'com.mitchellh.ghostty': activation request failed: unit is masked
  ```

## Root cause

Three facts combine.

First,
 a ghostty CLI flag overrides the config file,
 so `--gtk-single-instance=true` on the command line
wins over `gtk-single-instance = false` in `~/.config/ghostty/config:83`:

```ini
# ~/.config/ghostty/config
gtk-single-instance = false
```

Verified in both directions (see Verification):
 the running server carries the flag and is
single-instance despite the config,
 and a direct `ghostty --gtk-single-instance=false` is an
independent process despite the config default.

Second,
 the flag is injected by the Fedora ghostty package,
 not by the user.
 The `/usr` desktop entry
is `DBusActivatable=true`,
 and activation is delegated to systemd,
 whose unit hardcodes the flag:

```ini
# /usr/share/dbus-1/services/com.mitchellh.ghostty.service   (ghostty-1.3.1-1.fc44)
[D-BUS Service]
Name=com.mitchellh.ghostty
SystemdService=app-com.mitchellh.ghostty.service
Exec=/usr/bin/ghostty --gtk-single-instance=true --initial-window=false
```

```ini
# /usr/lib/systemd/user/app-com.mitchellh.ghostty.service   (ghostty-1.3.1-1.fc44)
[Service]
Type=notify-reload
BusName=com.mitchellh.ghostty
ExecStart=/usr/bin/ghostty --gtk-single-instance=true --initial-window=false

[Install]
WantedBy=graphical-session.target
```

```ini
# /usr/share/applications/com.mitchellh.ghostty.desktop:7,26   (ghostty-1.3.1-1.fc44)
Exec=/usr/bin/ghostty --gtk-single-instance=true
```

KDE also has a terminal setting that carried the flag (used when other apps open a terminal,
 for
example the "System Update" entry):

```ini
# ~/.config/kdeglobals  (before fix)
TerminalApplication=/usr/bin/ghostty --gtk-single-instance=true
TerminalService=com.mitchellh.ghostty.desktop
```

Third,
 whether a launch uses D-Bus activation (shared server) or runs `Exec` (new process) is decided
by the launcher framework from the desktop entry's `DBusActivatable` key.
 KDE's KIO
`ApplicationLauncherJob` (the code path behind the Plasma Task Manager icon,
 `Ctrl+Alt+T` via
`X-KDE-Shortcuts`,
 and `TerminalService`) only activates when `DBusActivatable` is true:

```cpp
// KDE/kio v6.27.0  src/gui/dbusactivationrunner.cpp:19
bool DBusActivationRunner::activationPossible(const KService::Ptr service, ...)
{
    if (!service->isApplication()) {
        return false;
    }
    if (service->property<bool>(QStringLiteral("DBusActivatable"))) {
        ...
        return true;
    }
    return false;
}
```

```cpp
// KDE/kio v6.27.0  src/gui/kprocessrunner.cpp:108
if (!notYetSupportedOpenActivationNeeded && DBusActivationRunner::activationPossible(service, flags, suggestedFileName)) {
    ...
    instance = new DBusActivationRunner(...);   // D-Bus activation -> shared server
} else {
    instance = makeInstance();                  // Forking/Systemd/Scoped runner -> runs Exec
}
```

So `DBusActivatable=false` in a user desktop entry (`~/.local/share/applications/...`,
 which shadows
`/usr`) makes KIO run `Exec`.
 `service->property<bool>("DBusActivatable")` reads from ksycoca,
 so the
entry must be indexed and valid for the value to take effect.

### The `TryExec` trap (the actual failure encountered here)

A desktop entry whose `TryExec` does not resolve to an existing executable is treated as not installed,
and every launcher silently falls back to the `/usr` entry (`DBusActivatable=true`).
 `TryExec` is a
bare path,
 not a command line:
 `TryExec=/usr/bin/ghostty --gtk-single-instance=false` is the path
`/usr/bin/ghostty --gtk-single-instance=false`,
 which does not exist,
 so the whole user entry is
discarded.
 This was the real cause of the "unit is masked" error:
 the user entry was invalidated by an
errant `TryExec`,
 KDE used `/usr` (`DBusActivatable=true`),
 activation fired,
 and the masked unit made
it fail.
 Confirmed with GIO resolution before and after the fix:

```bash
python3 -c "import gi; from gi.repository import Gio; a=Gio.DesktopAppInfo.new('com.mitchellh.ghostty.desktop'); print(a.get_filename(), a.get_boolean('DBusActivatable'))"
# TryExec broken:  /usr/share/applications/com.mitchellh.ghostty.desktop  True
# TryExec fixed:   /home/USER/.local/share/applications/com.mitchellh.ghostty.desktop  False
```

### Why masking the unit was the wrong approach

D-Bus activation maps one bus name to one server process;
 it is single-instance by construction,
 so it
can never yield a new process per launch.
 Masking `app-com.mitchellh.ghostty.service` does not convert
activation into new-process launching;
 it only makes activation fail ("unit is masked").
 The correct
lever is to avoid activation entirely (`DBusActivatable=false` so launchers run `Exec`),
 not to break
it.
 The unit is left unmasked (stock).

## Verification

Versions under test:

- Ghostty 1.3.1,
   stable,
   app runtime `.gtk`,
   GTK 4.22.4,
   libadwaita 1.9.1 (rpm
  `ghostty-1.3.1-1.fc44.x86_64`).
- KDE Plasma 6.7.1,
   KWin 6.7.1,
   KF6 KIO 6.27.0 (rpm `kf6-kio-gui-6.27.0-2.fc44`),
   Wayland,
   Fedora 44.
- KIO source read at tag `v6.27.0`.

Launch-path catalog,
 run in a no-server state after the fix,
 counting new processes and their flags
(harness:
 capture `pgrep -x ghostty` before and after,
 diff,
 read `/proc/<pid>/cmdline`):

- `kioclient exec ~/.local/share/applications/com.mitchellh.ghostty.desktop` (KIO
  `ApplicationLauncherJob`,
   the pinned-icon path):
   one new process,
   `--gtk-single-instance=false`,
   no
  server.
   Clean.
   Repeated launches stay independent.
- `kstart --application com.mitchellh.ghostty`:
   one new process,
   `--gtk-single-instance=false`,
   no
  server.
- `ghostty --gtk-single-instance=false` (direct binary,
   used by `Meta+N`):
   one new process,
   no server.
- `gtk-launch com.mitchellh.ghostty` (GIO,
   not a KDE launcher):
   runs `Exec` (`=false`) AND also
  activates the `=true` server.
   GIO activates the bus name even with `DBusActivatable=false`;
   a stray
  `=true` server can therefore appear,
   but it cannot capture other launches because a `=false` launch
  never attaches to it.

Bus-name ownership:
 `--gtk-single-instance=false` never claims `com.mitchellh.ghostty`,
 so no server
forms from it.
 Proven in a throwaway session bus:

```bash
dbus-run-session -- bash -c '
  ghostty --gtk-single-instance=false & sleep 3
  ghostty --gtk-single-instance=false & sleep 3
  busctl --address "$DBUS_SESSION_BUS_ADDRESS" list --acquired | grep -i com.mitchellh.ghostty \
    || echo "no acquired owner"   # observed: no acquired owner; two distinct PIDs
'
```

## Verified workarounds

### The fix: a valid `DBusActivatable=false` user desktop entry

All edits are user-level;
 the `/usr` files are read-only on rpm-ostree and are shadowed,
 not modified.

1.  `~/.local/share/applications/com.mitchellh.ghostty.desktop` (shadows `/usr`):
    - Both `Exec=` lines (main entry and the `new-window` action):
       `/usr/bin/ghostty --gtk-single-instance=false`.
    - `DBusActivatable=false`.
    - Leave `TryExec=/usr/bin/ghostty` a bare path.
       Do not append flags to `TryExec`.
2.  `~/.config/kdeglobals`:
     `TerminalApplication=/usr/bin/ghostty --gtk-single-instance=false`.
3.  Refresh caches:
     `update-desktop-database ~/.local/share/applications && kbuildsycoca6 --noincremental`.

No systemd mask and no user D-Bus service shadow are needed;
 both are counterproductive (see What does
not work).
 Verify:
 GIO/KService resolve the `~/.local` entry with `DBusActivatable False`,
 and KIO/
`kstart`/`Meta+N` each spawn one `--gtk-single-instance=false` process with no server.

Tradeoffs:

- No windowless single-instance server,
   so cold opens are slightly slower and each window is its own
  process (more total memory).
   This is the point (crash isolation),
   but it is a real cost.
- `DBusActivatable=false` means the app's D-Bus activation interface is not used for launches.
   KDE's
  `X-TerminalArg*` keys still work through `Exec`,
   so "open terminal here" and "run command in
  terminal" are unaffected.
- Takes effect for new KIO launches immediately (no logout required) once ksycoca is rebuilt.
   A
  `=true` server already running from before a login stays until its windows are closed.
- A GIO-based launcher (`gtk-launch`,
   `gio launch`) can still spin up a stray windowless `=true`
  server;
   it does not capture KDE launches and is harmless,
   but it is not fully preventable without
  breaking activation.

Reverting:
 restore the two `Exec` lines and `DBusActivatable=true` in the user desktop entry (or delete
the user entry to fall back to `/usr`),
 restore `kdeglobals`,
 and rebuild ksycoca.

### Per-invocation: `Meta+N` new-instance

`key-helper-kwin` (KWin script) plus `key-helper-service` (`~/.local/lib/key-helper/index.ts`,
`NEW_WINDOW_OVERRIDES["com.mitchellh.ghostty"]`) launch `ghostty --gtk-single-instance=false` directly,
bypassing D-Bus activation,
 so `Meta+N` always yields a new process.
 Verified:
 `Meta+N` adds a new PID.

## What does not work

- `gtk-single-instance = false` in `~/.config/ghostty/config` alone:
   overridden by the
  `--gtk-single-instance=true` CLI flag in the packaging files.
- Editing the desktop entry `Exec`/args while leaving `DBusActivatable=true`:
   KIO activates by bus name
  and ignores `Exec`.
- `TryExec` with arguments (`TryExec=/usr/bin/ghostty --gtk-single-instance=false`):
   the path does not
  exist,
   so the entry is treated as not installed and launchers fall back to `/usr`
  (`DBusActivatable=true`).
   This silently reintroduces the shared server and was the cause of the
  "unit is masked" error.
- `systemctl --user mask app-com.mitchellh.ghostty.service`:
   does not convert activation into
  new-process launching;
   it makes activation fail with "unit is masked" and breaks the launcher.
- User-local D-Bus service shadow (`~/.local/share/dbus-1/services/com.mitchellh.ghostty.service`):
   on
  Fedora (systemd-delegated D-Bus activation),
   the `/usr` service file's `SystemdService=` mapping is
  used,
   so the user shadow's `Exec=` does not redirect activation.
   Ineffective.
- Killing the running server to switch models in the current session:
   not viable when the server is the
  controlling terminal of the interactive session (`ghostty -> bash -> shell`);
   killing it ends the
  session and closes every other ghostty window.

## Upstream filing decision

`.out-of-scope/` holds no ghostty or KDE exemption.
 Duplicate search
(`gh search issues --repo ghostty-org/ghostty ...`) returned rate-limited on this run (see
`doc/troubleshooting/gh-search-rate-limit.md`);
 the decision does not rest on filing,
 so the
incomplete search does not block it.
 Default policy is do not file.

1.  Is it really upstream's fault?
     No. CLI flags overriding the config file is standard and deliberate,
    and the single-instance server started via systemd/D-Bus activation is a feature.
     The
    `--gtk-single-instance=true` flag lives in Fedora's packaging (`ghostty-1.3.1-1.fc44`),
     a
    distribution choice,
     not the ghostty source.
     KIO's activate-vs-`Exec` decision
    (`dbusactivationrunner.cpp:19`) is correct and honors `DBusActivatable`.
     The user-facing surprise is
    a config key being ineffective for the GUI path,
     a consequence of documented precedence plus
    packaging,
     not a defect.
2.  Can upstream fix it?
     Not applicable;
     no upstream defect.
     The nearest change is a Fedora packaging
    request to not hardcode the flag,
     a preference about a working feature.
3.  Are they supporting this use case?
     Yes;
     both single-instance and non-single-instance are documented
    via `gtk-single-instance`,
     and the fix uses the supported `=false` mode.
4.  Would the repo welcome our contribution?
     Not evaluated to completion;
     moot because constraint 1
    fails.
5.  Will they likely fix it?
     Not applicable;
     nothing to fix.
6.  Prototyped a minimal fix?
     Not applicable;
     constraints 1 and 2 fail,
     so the auto-prototype trigger is
    not met.
     The user-side fix (a valid `DBusActivatable=false` entry) is recorded above.

Decision:
 do not file.
 This is a local configuration and packaging-precedence issue,
 resolved with
user-level configuration.
 No new-issue draft is kept.
