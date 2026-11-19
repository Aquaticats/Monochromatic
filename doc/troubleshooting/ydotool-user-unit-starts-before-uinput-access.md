# ydotool 1.0.4's user unit gives up at boot when linger starts it before the seat session grants /dev/uinput

## Status at 2026-11-18T10:49:04-05:00

An agent wrote this record on the CachyOS machine;
the owner has not reviewed or approved it.
The agent read the journals,
traced the sources,
ran the throwaway-unit harness in "Verification"
and applied the drop-in in "Verified workarounds".
Open:
no real reboot has exercised the drop-in yet,
and the owner had not yet confirmed Ctrl+F4 in Helium after the same-boot restart.

## Symptom

After the reboot on 2026-11-18 (boot at 10:13:37 EST),
Ctrl+F4 did nothing in Helium.
key-helper (`package/kwin/key-helper`) maps Ctrl+F4 to Ctrl+W in browsers.
Its KWin script still matched Helium and called the daemon,
and the daemon logged one error per press:

```text
Nov 18 10:25:47 cachyos-pc1 key-helper[3260]: [error] [2026-11-18T15:25:47.721Z] [kwin-key-helper-keys] [sendKeys] ydotool/key error: 'ydotool key 29:1 17:1 17:0 29:0' exited with code 2
Nov 18 10:25:57 cachyos-pc1 key-helper[3260]: [error] [2026-11-18T15:25:57.823Z] [kwin-key-helper-keys] [sendKeys] ydotool/key error: 'ydotool key 29:1 17:1 17:0 29:0' exited with code 2
Nov 18 10:25:58 cachyos-pc1 key-helper[3260]: [error] [2026-11-18T15:25:58.659Z] [kwin-key-helper-keys] [sendKeys] ydotool/key error: 'ydotool key 29:1 17:1 17:0 29:0' exited with code 2
Nov 18 10:25:59 cachyos-pc1 key-helper[3260]: [error] [2026-11-18T15:25:59.508Z] [kwin-key-helper-keys] [sendKeys] ydotool/key error: 'ydotool key 29:1 17:1 17:0 29:0' exited with code 2
```

The ydotool client exits 2 when it cannot connect to the daemon's socket
([`Client/ydotool.c:157-173` at v1.0.4][client-connect]):

```c
// Client/ydotool.c:157-173 (ydotool v1.0.4), case labels for EACCES and EPERM elided
	if (connect(fd_daemon_socket, (const struct sockaddr *) &sa, sizeof(sa))) {
		int err = errno;
		printf("failed to connect socket `%s': %s\n", sa.sun_path, strerror(err));

		switch (err) {
			case ENOENT:
			case ECONNREFUSED:
				puts("Please check if ydotoold is running.");
				break;
			...
		}

		exit(2);
	}
```

The daemon was not running.
`systemctl --user status ydotool.service` showed `inactive (dead) since Wed 2026-11-18 10:14:10 EST`,
and `journalctl --user -b -u ydotool.service` showed five identical cycles within about one second,
then the start limit:

```text
Nov 18 10:14:09 cachyos-pc1 systemd[1670]: Started Starts ydotoold service.
Nov 18 10:14:09 cachyos-pc1 ydotoold[1720]: failed to open uinput device: Permission denied
Nov 18 10:14:09 cachyos-pc1 ydotoold[1720]: You're advised to run this program as root, or YMMV.
Nov 18 10:14:09 cachyos-pc1 systemd[1670]: ydotool.service: Main process exited, code=exited, status=2/INVALIDARGUMENT
Nov 18 10:14:09 cachyos-pc1 systemd[1670]: ydotool.service: Failed with result 'exit-code'.
Nov 18 10:14:09 cachyos-pc1 systemd[1670]: ydotool.service: Scheduled restart job, restart counter is at 1.
...
Nov 18 10:14:10 cachyos-pc1 ydotoold[2179]: failed to open uinput device: Permission denied
Nov 18 10:14:10 cachyos-pc1 ydotoold[2179]: You're advised to run this program as root, or YMMV.
Nov 18 10:14:10 cachyos-pc1 systemd[1670]: ydotool.service: Main process exited, code=exited, status=2/INVALIDARGUMENT
Nov 18 10:14:10 cachyos-pc1 systemd[1670]: ydotool.service: Failed with result 'exit-code'.
Nov 18 10:14:10 cachyos-pc1 systemd[1670]: ydotool.service: Scheduled restart job, restart counter is at 5.
Nov 18 10:14:10 cachyos-pc1 systemd[1670]: ydotool.service: Start request repeated too quickly.
Nov 18 10:14:10 cachyos-pc1 systemd[1670]: ydotool.service: Failed with result 'start-limit-hit'.
Nov 18 10:14:10 cachyos-pc1 systemd[1670]: Failed to start Starts ydotoold service.
```

When this was investigated at about 10:28,
`getfacl /dev/uinput` already showed `user:user:rw-`.
Access had arrived;
systemd had stopped trying.
Only this one failure variant was observed.

## Root cause

### Timeline of the boot

- 10:14:09.289925:
  `user@1000.service` starts
  (`systemctl show user@1000.service -p ExecMainStartTimestamp --timestamp=us`).
- 10:14:09.329687:
  logind session 1 starts,
  class `manager`,
  no seat
  (logind D-Bus `Session.Timestamp` of `session/_31`).
- 10:14:09.468672:
  first `failed to open uinput device: Permission denied` (user journal).
- 10:14:10.485389:
  fifth failure,
  then `start-limit-hit` (user journal).
- 10:14:11.743003:
  logind session 2 starts:
  `seat0`,
  VT 1,
  class `user`,
  service `login`,
  the getty autologin on tty1
  (logind D-Bus `Session.Timestamp` of `session/_32`, and `loginctl show-session 2`).
- 10:14:16.663:
  `Reached target Startup of XDG autostart applications.` (user journal).
- 10:25:47 to 10:25:59:
  the `sendKeys` errors (key-helper journal).

All five ydotoold attempts ended 1.26 s before the first seat session existed.

### Step 1: linger starts the user manager at boot, before any login

`loginctl show-user 1000` reports `Linger=yes`;
`/var/lib/systemd/linger/user` was created 2026-11-17 16:04:05.
logind adds lingering users when it starts
([`src/login/logind.c:329-330` at v262.2][logind-linger-enumerate]):

```c
// src/login/logind.c:329-330 (systemd v262.2)
        /* Add lingering users */
        r = manager_enumerate_linger_users(m);
```

and then starts every user it knows
([`src/login/logind.c:1297-1302`][logind-startup-users]):

```c
// src/login/logind.c:1297-1302 (systemd v262.2)
        /* And start everything */
        HASHMAP_FOREACH(seat, m->seats)
                (void) seat_start(seat);

        HASHMAP_FOREACH(user, m->users)
                (void) user_start(user);
```

The linger file alone is enough to start `user@.service`
([`src/login/logind-user.c:412-423`][logind-wants-manager]):

```c
// src/login/logind-user.c:412-423 (systemd v262.2)
static bool user_wants_service_manager(const User *u) {
        assert(u);

        LIST_FOREACH(sessions_by_user, s, u->sessions)
                if (SESSION_CLASS_WANTS_SERVICE_MANAGER(s->class))
                        return true;

        if (user_check_linger_file(u) > 0)
                return true;

        return false;
}
```

So the user manager came up at 10:14:09.29,
2.45 s before the autologin session on tty1.

### Step 2: the packaged unit starts with the user manager

The installed unit is wanted by `default.target`,
so it starts as soon as the user manager does,
and it sets no restart delay
([`Daemon/ydotool.service.in:4-13` at v1.0.4][unit-v104]):

```ini
# Daemon/ydotool.service.in:4-13 (ydotool v1.0.4)
[Service]
Type=simple
Restart=always
ExecStart=@CMAKE_INSTALL_FULL_BINDIR@/ydotoold
ExecReload=/usr/bin/kill -HUP $MAINPID
KillMode=process
TimeoutSec=180

[Install]
WantedBy=default.target
```

`/usr/lib/systemd/user/ydotool.service` is owned by `ydotool 1.0.4-3.1` (`pacman -Qo`).
That package comes from the `cachyos-extra-znver4` repository with packager CachyOS,
a rebuild of Arch `extra` `ydotool 1.0.4-3`.
After substituting `/usr/bin` for `@CMAKE_INSTALL_FULL_BINDIR@`,
`diff` against the upstream v1.0.4 template is empty
(installed file sha256 `45ceb25a2992bf282e4fb2796d8f4c991dc8cca8f5217c17c5156f05f66d1a49`),
so neither Arch nor CachyOS changes the unit.
Arch's PKGBUILD installs the upstream build output as is and adds only a udev rule
([`PKGBUILD:29-33` at `62c3920`][arch-pkgbuild]):

```sh
# PKGBUILD:29-33 (Arch packaging/packages/ydotool at 62c3920)
package() {
  DESTDIR="$pkgdir" ninja -C build install

  install -Dm644 80-uinput.rules -t "$pkgdir"/usr/lib/udev/rules.d/
}
```

### Step 3: ydotoold exits when it cannot open /dev/uinput

The daemon gives up on the first failed open
([`Daemon/ydotoold.c:265-270` at v1.0.4][daemon-open]):

```c
// Daemon/ydotoold.c:265-270 (ydotool v1.0.4)
	int fd_ui = open("/dev/uinput", O_WRONLY);

	if (fd_ui < 0) {
		perror("failed to open uinput device");
		exit(2);
	}
```

### Step 4: this account reaches /dev/uinput only through the active seat session's ACL

`/dev/uinput` is `crw-rw----+ root input`.
Arch's `80-uinput.rules` sets that group and mode
([`80-uinput.rules:3` at `62c3920`][arch-uinput-rules]):

```text
# 80-uinput.rules:3 (Arch packaging/packages/ydotool at 62c3920)
KERNEL=="uinput", GROUP="input", MODE="0660", OPTIONS+="static_node=uinput"
```

The account is not in `input`:
`id` lists the groups `user`, `sys`, `network`, `nopasswdlogin`, `rfkill`,
`users`, `video`, `storage`, `lp`, `audio` and `wheel`.
The user ACL comes from `TAG+="uaccess"` in installed rules from other packages,
none from this repository or from ydotool:

- `/usr/lib/udev/rules.d/40-kdeconnect-uinput.rules:5` (kdeconnect 26.08.3-2.1)
  and `/usr/lib/udev/rules.d/60-steam-input.rules:5` (steam-devices 1.0.0.87-3):
  `KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess", OPTIONS+="static_node=uinput"`
- `/usr/lib/udev/rules.d/60-sunshine.rules:2` (sunshine 2026.1109.513-2):
  `KERNEL=="uinput", SUBSYSTEM=="misc", OPTIONS+="static_node=uinput", GROUP="input", MODE="0660", TAG+="uaccess"`

`/run/udev/static_node-tags/uaccess/` lists `uinput`.
logind applies ACLs to those static nodes for the user of the seat's active session only
([`src/login/logind-seat.c:336-347`][logind-static-acl]
and [`src/login/logind-seat.c:349`][logind-static-dir]):

```c
// src/login/logind-seat.c:336-349 (systemd v262.2), line 348 elided
static int static_node_acl(Seat *s) {
#if HAVE_ACL
        int r, ret = 0;
        _cleanup_set_free_ Set *uids = NULL;

        assert(s);

        if (s->active) {
                r = set_ensure_put(&uids, NULL, UID_TO_PTR(s->active->user->user_record->uid));
                if (r < 0)
                        return log_oom();
        }
...
        _cleanup_closedir_ DIR *dir = opendir("/run/udev/static_node-tags/uaccess/");
```

It runs when the seat's active session changes
([`src/login/logind-seat.c:447-453`][logind-set-active]):

```c
// src/login/logind-seat.c:447-453 (systemd v262.2), in seat_set_active()
        r = seat_trigger_devices(s);
        if (r < 0)
                return r;

        r = static_node_acl(s);
        if (r < 0)
                return r;
```

and a new session reaches that through `seat_read_active_vt()` in `session_start()`
([`src/login/logind-session.c:931-932`][logind-session-vt]):

```c
// src/login/logind-session.c:931-932 (systemd v262.2), in session_start()
        if (s->seat)
                seat_read_active_vt(s->seat);
```

Session 1 (class `manager`) has no seat.
Until session 2 started at 10:14:11.743,
no session on `seat0` existed,
so nothing could have granted this account the ACL.
The exact time logind applied it is not recorded:
`journalctl -b -t systemd-logind` returned no entries for this boot from this account,
while the same query for the previous boot did return logind lines,
and the device's ctime was overwritten later
(see "Probe errors caught before trusting a result").

### Step 5: systemd's default pacing gives up within about a second

With no `RestartSec=`,
the restart delay is 100 ms
([`src/basic/constants.h:6` at v262.2][sd-restart-default];
[`man/systemd.service.xml:611-615`][sd-restart-man] says "Defaults to 100ms."),
and the start limit is five starts per 10 s
([`src/basic/constants.h:20-21`][sd-startlimit-default]):

```c
// src/basic/constants.h:20-21 (systemd v262.2)
#define DEFAULT_START_LIMIT_INTERVAL (10*USEC_PER_SEC)
#define DEFAULT_START_LIMIT_BURST 5
```

[`man/systemd-system.conf.xml:694-696`][sd-startlimit-man] agrees:
"DefaultStartLimitIntervalSec= defaults to 10s. DefaultStartLimitBurst= defaults to 5."
The running user manager reports the same values:

```text
$ systemctl --user show -p DefaultRestartUSec -p DefaultStartLimitIntervalUSec -p DefaultStartLimitBurst
DefaultRestartUSec=100ms
DefaultStartLimitIntervalUSec=10s
DefaultStartLimitBurst=5
```

Five failures 100 ms apart used the whole budget in 1.02 s,
long before the 2.45 s it took autologin to create the seat session.

### Why it worked the day before

The 2026-11-17 boot never took this path.
The user manager started at 14:02:28.
ydotool was installed at 15:05:16 (`/var/log/pacman.log`)
and started at 15:07:24,
an hour into a running graphical session.
The linger file appeared at 16:04:05,
also after that boot.
2026-11-18 was the first boot with both linger and the enabled unit.

The old Bazzite system did not depend on this timing either.
It ran ydotoold from XDG autostart as `app-ydotoold@autostart.service`
(`doc/troubleshooting/ydotool-interrupted-key-release.md`, section "Symptom"),
and its account was in `input`
(`~/Downloads/old-drives/windows-shortcuts-2026-11-17.md`, the key-helper section).
On this boot the XDG autostart target was reached at 10:14:16.663,
4.9 s after the seat session.

### Upstream already fixed the unit, in no release

[PR #221][pr-221],
merged 2024-02-05 as commit `a2e7e5defc8be8525397bda5b49e50f3a177c460`,
reports the same journal lines and adds a start delay,
a restart delay and an earlier target.
Commit `e3a1dac` later moved the file
([`Daemon/systemd/ydotoold.service.in:4-15` at master `708e96f`][unit-master]):

```ini
# Daemon/systemd/ydotoold.service.in:4-15 (ydotool master at 708e96f)
[Service]
Type=simple
Restart=always
RestartSec=3
ExecStartPre=/bin/sleep 2
ExecStart=@CMAKE_INSTALL_FULL_BINDIR@/ydotoold
ExecReload=/usr/bin/kill -HUP $MAINPID
KillMode=process
TimeoutSec=180

[Install]
WantedBy=basic.target
```

The newest release and tag is still v1.0.4 (2023-01-30),
and master was 38 commits ahead of it on 2026-11-18.
Distributions building from the v1.0.4 tarball,
Arch and CachyOS included,
ship the old unit.

## Verification

- ydotool 1.0.4-3.1 (`cachyos-extra-znver4`, built 2026-10-22),
  upstream tag v1.0.4 at `57ba7d0af525e82da2de0e275d169477f293b197`.
- systemd 262.2-1-arch,
  upstream tag v262.2 at commit `63511d7283d7d0e2a2fc28075481afaf1a7be8f5`.
- Linux 7.3.4-1-cachyos,
  KDE Plasma on Wayland,
  getty autologin on tty1,
  linger enabled.

The real failure needs a reboot to repeat,
so the harness models "start fails until access appears" with a flag file in throwaway user units.
It runs no ydotool and touches no device.
It ran as written on 2026-11-18:

```bash
#!/usr/bin/env bash
# ~/temp/agent/ydotool-boot-race-doc-2026-11-18/keep/proxy-harness.sh
# Models "ydotoold fails until /dev/uinput access appears" with a flag file.
# Usage: proxy <unit-name> <seconds-until-access> [systemd-run -p options...]
# No --collect: a unit that hits its start limit would be garbage-collected,
# and `systemctl show` would then print defaults (inactive, success, 0 restarts).
set -u
proxy() {
  local unit=$1 delay=$2; shift 2
  local d flag
  d=$(mktemp -d "${HOME}/temp/agent/ydotool-retry-proxy.XXXXXXXX"); flag="$d/access-granted"
  systemd-run --user --quiet --unit="$unit" -p Restart=always "$@" \
    /bin/sh -c "test -e '$flag' || { echo 'no access yet'; exit 2; }; exec sleep 300"
  sleep "$delay"; touch "$flag"; sleep 8
  echo "== $unit (access after ${delay} s)"
  systemctl --user show "$unit" -p ActiveState -p Result -p NRestarts
  systemctl --user stop "$unit"; systemctl --user reset-failed "$unit" 2>/dev/null; rm -r "$d"
}
proxy ydotool-retry-proxy-default 3
proxy ydotool-retry-proxy-dropin 7 -p RestartSec=2 -p StartLimitIntervalSec=90 -p StartLimitBurst=30
proxy ydotool-retry-proxy-master 2.45 -p RestartSec=3 -p 'ExecStartPre=/bin/sleep 2'
```

Output (`keep/proxy-harness.log` in the same folder):

```text
== ydotool-retry-proxy-default (access after 3 s)
ActiveState=failed
Result=start-limit-hit
NRestarts=5
== ydotool-retry-proxy-dropin (access after 7 s)
ActiveState=active
Result=success
NRestarts=4
== ydotool-retry-proxy-master (access after 2.45 s)
ActiveState=active
Result=success
NRestarts=1
```

A separate run used the drop-in's values with access that never arrives
(`/bin/sh -c "echo 'no access yet'; exit 2"`):
the first start was at 10:44:18.657,
`start-limit-hit` came at 10:45:23.347 (64.7 s later),
with `NRestarts=30` (`keep/never-granted.log`).

### Works

- Drop-in values (`RestartSec=2`, 30 starts per 90 s),
  access after 7 s:
  active after 4 restarts.
- Upstream master's values (`RestartSec=3`, `ExecStartPre=/bin/sleep 2`),
  access after 2.45 s,
  the real gap on this boot:
  active after 1 restart.
- On the real unit,
  once access existed:
  `systemctl --user reset-failed ydotool.service`,
  then `systemctl --user start ydotool.service` at 10:30:34.
  It stayed active,
  and no `sendKeys` error was logged from 10:28 to 10:40.

### Fails

- Packaged defaults (100 ms, five starts per 10 s),
  access after 3 s in the proxy:
  `start-limit-hit` after 5 restarts.
- Packaged defaults on the real boot,
  access no earlier than 2.45 s after the user manager started:
  `start-limit-hit` at 10:14:10.
- Drop-in values,
  access never:
  `start-limit-hit` after 64.7 s.

### Live state after the fix

```text
$ systemctl --user is-active ydotool.service
active
$ systemctl --user show ydotool.service -p RestartUSec -p StartLimitBurst -p StartLimitIntervalUSec -p DropInPaths -p MainPID
DropInPaths=/home/user/.config/systemd/user/ydotool.service.d/retry-until-uinput-access.conf
StartLimitIntervalUSec=1min 30s
StartLimitBurst=30
RestartUSec=2s
MainPID=10314
$ ss -xlp | grep -i ydotool
u_dgr UNCONN 0      0                            /run/user/1000/.ydotool_socket 75823            * 0    users:(("ydotoold",pid=10314,fd=4))
```

## Verified workarounds

### Drop-in that keeps retrying for about a minute

The drop-in's full text:

```ini
# ~/.config/systemd/user/ydotool.service.d/retry-until-uinput-access.conf
# Keep retrying ydotoold at boot until logind grants this user access to /dev/uinput.
# Linger starts the user manager, and this unit, at boot, before the tty1 autologin seat session
# exists, so the seat's uaccess ACL on /dev/uinput has not reached the user yet.
# With the packaged unit's default 100 ms restart delay, the 5 failed starts used
# up systemd's default limit (5 starts in 10 s) within one second on 2026-11-18,
# so Ctrl+F4 (key-helper sends Ctrl+W through ydotool) stopped working after a reboot.
# 30 starts 2 s apart take 60 s, inside the 90 s window: it gives up after about a minute.
# Details: doc/troubleshooting/ydotool-user-unit-starts-before-uinput-access.md (Monochromatic).
[Unit]
StartLimitIntervalSec=90
StartLimitBurst=30

[Service]
RestartSec=2
```

Apply it:

```sh
systemctl --user daemon-reload
systemctl --user reset-failed ydotool.service
systemctl --user start ydotool.service
```

`StartLimitIntervalSec=` and `StartLimitBurst=` belong in `[Unit]`.
30 starts at 2 s each is shorter than the 90 s window,
so the limit can still trip.

Tradeoffs:

- It still gives up.
  If the seat session does not exist within about a minute of the user manager starting,
  the unit fails as before (measured: 64.7 s).
  With linger that clock starts at boot,
  so a manual or slow login more than a minute after boot leaves ydotoold dead
  until `reset-failed` and `start` are run.
  Autologin made the gap 2.45 s on this boot.
- Each failed try logs ydotoold's two error lines plus systemd's exit and restart lines,
  up to 30 times.
- An SSH-only login has no seat session and never gets the ACL,
  so the unit retries for a minute and then fails,
  the same end state as before,
  just later.
- The access model stays narrow:
  the account still reaches `/dev/uinput` only through the active seat session's ACL,
  not through the `input` group.
- It only overrides pacing.
  If a later package ships upstream master's unit,
  this drop-in's 2 s restart delay replaces master's 3 s,
  and master's `ExecStartPre=/bin/sleep 2` still applies.
- No real reboot has tested it yet;
  only the proxy runs in "Verification" support it.

### Restart by hand in the same boot

```sh
systemctl --user reset-failed ydotool.service
systemctl --user start ydotool.service
```

This worked at 10:30:34 once access existed.
It lasts only until the next boot.
key-helper needs no restart:
it runs the `ydotool` client on each key press,
so the next press finds the socket.
The owner had not yet confirmed a working Ctrl+F4 when this was written.

## What does not work

### Ordering after `graphical-session.target` on its own

`After=` delays a unit only when both units are being started
([`man/systemd.unit.xml:832-838` at v262.2][sd-after-man]:
"If unit foo.service contains the setting Before=bar.service and both units are being started,
bar.service's start-up is delayed until foo.service has finished starting up.").
At boot with linger,
`default.target` pulls in `ydotool.service` while no graphical session exists to be started,
so the ordering would not hold it back.
This comes from the man page;
it was not tried here.
Moving the install target to `graphical-session.target` would be a different change,
and it was not tried either.

### Joining the `input` group

It would remove the race,
because the group permission on `/dev/uinput` does not depend on a session,
and Arch's `ydotool.install` asks for it
([`ydotool.install:1-4` at `62c3920`][arch-install]).
But every node in `/dev/input` here is `crw-rw---- root input`,
so every process the account runs could read raw keystrokes.
The owner chose to stay out of `input` on 2026-11-17
(`~/Downloads/old-drives/windows-shortcuts-2026-11-17.md`).
Arch's package [issue #1][arch-issue-1] raises the same concern.

### Restoring the old `ydotoold.desktop` autostart next to the packaged unit

It was restored on 2026-11-17 and removed again,
because with `ydotool.service` enabled it only starts a second daemon that fails
(`~/Downloads/old-drives/windows-shortcuts-2026-11-17.md`).
Replacing the packaged unit with the autostart (disabling `ydotool.service`) was not tried.

### Probe errors caught before trusting a result

- The first proxy run used `systemd-run --collect`.
  The unit that hit its start limit was garbage-collected,
  so `systemctl show` printed defaults (`inactive`, `Result=success`, `NRestarts=0`).
  The harness in "Verification" drops `--collect`.
- `/dev/uinput`'s ctime reads 10:30:34.607,
  the moment ydotoold was started by hand,
  so it does not date the ACL.
- `journalctl -b -u systemd-logind` and `-t systemd-logind` returned no entries for this boot.
  The session times in "Timeline of the boot" come from logind's D-Bus `Timestamp` property instead.

## Upstream filing decision

Checked by 2026-11-18T10:49:04-05:00.

### Out-of-scope list

No entry in `.out-of-scope/` covers ydotool,
systemd or udev.
Its files are `bun-install.md`, `cargo-workspace.md`, `claude-code-upstream-bugs.md`, `codex-harness.md`,
`jsr.md`, `lightningcss.md`, `low-impact-typescript-formatting.md`, `module-es-monolith.md`,
`pi-gpt55-long-context.md`, `terminal-title-fork-parity-tests.md` and `typescript-project-references.md`.

### Duplicate search

On the ydotool tracker (`gh search issues --repo ReimuNotMoe/ydotool`):

- "uinput Permission denied boot", "ydotool.service fails to start" and "linger" returned nothing.
- "start-limit-hit", "RestartSec" and "new release" returned [#311][issue-311],
  older release requests #180, #90 and #19,
  and #42 (ship a service file),
  all closed.
- [#311][issue-311] ("Request: new release to include systemd service reliability fixes", 2026-02-12)
  names the same missing `RestartSec=3` and `ExecStartPre=/bin/sleep 2`,
  and the same "Start request repeated too quickly" failure.
  Its author closed it the next day as completed,
  with no comments and no maintainer reply,
  and no release followed.

On the packaging trackers:

- The Arch `archlinux/packaging/packages/ydotool` issues (GitLab API, all states):
  #2 "Default user service is broken: `failed to open uinput device: Permission denied`" (1.0.4-2)
  was closed 2025-12-16,
  and #1 (the `input` group exposes every keyboard) is open.
  Their comments are returned only to signed-in users (HTTP 401),
  so they were not read.
- CachyOS: `gh search issues --owner CachyOS ydotool` returned nothing.
  The CachyOS package is a rebuild of Arch's.

### Work in progress

`Daemon/systemd/ydotoold.service.in` was last changed 2024-05-02 (`e3a1dac`),
and master's newest commit is from 2025-12-22.
`gh search prs --repo ReimuNotMoe/ydotool service` shows no open pull request touching the unit;
the open ones are #317 (XKB text) and #272 (README install steps).

### Constraint 1: is it upstream's fault?

Partly.
The v1.0.4 unit's pacing gives up in a second,
and upstream accepted that as its problem by merging #221.
The trigger here (linger plus ACL-only access) is local configuration.

### Constraint 2: can upstream fix it?

Yes,
and it already has on master;
what is missing is a release.

### Constraint 3: are they supporting this use case?

Yes:
ydotool ships the user unit.

### Constraint 4: would the repo welcome our contribution?

No `CONTRIBUTING`,
issue or pull request templates,
or vouch or trust list exist at `708e96f`;
the tree's `.github` holds only `FUNDING.yml` and build workflows.
No AI policy was found.
Outside pull requests are merged (#221 came from a contributor).

### Constraint 5: will they likely fix it?

The fix is merged,
but no release has followed in almost four years,
despite #19, #90, #180 and #311,
and #311 got no maintainer reply.

### Constraint 6: have we prototyped a minimal fix?

The upstream fix is the prototype.
It is absent from the installed unit (the `diff` in "Step 2: the packaged unit starts with the user manager"),
the version (v1.0.4) and the journal lines match,
and the harness shows master's values recover from this boot's 2.45 s gap.
No patch needs to be kept here.

### Decision

File nothing upstream.
A new ydotool issue would duplicate #311.
A comment there would add only "+1" and the linger trigger,
which master's unit already handles,
on an issue its author closed:
**there is nothing to add.**

For Arch,
the package's own instruction is to join `input` (`ydotool.install`),
which avoids the race,
so constraint 3 fails for the ACL-only setup used here,
and a similar user-service report (#2) is already closed.
The backport request in "Draft for Arch (do not file as-is)" is kept as a record only.
Filing anything puts the poster's account in front of strangers;
some projects run trust or blocklist systems whose verdicts can follow a person across projects.
Whether to file is the poster's decision.
The readiness check was not run,
because nothing here is marked fileable.

### Draft for Arch (do not file as-is)

~~~md
Title: ydotool 1.0.4 user unit gives up at boot; backport upstream's RestartSec/ExecStartPre fix

The shipped `ydotool.service` is upstream v1.0.4's unit, with no `RestartSec=`.
When `/dev/uinput` is not yet writable as the user manager starts (for example, linger
plus uaccess-only access, before the seat session exists), ydotoold exits 2 five times
in about a second and the unit ends in `start-limit-hit`.

Upstream fixed the unit on master in PR #221 (commit a2e7e5d: `RestartSec=3`,
`ExecStartPre=/bin/sleep 2`, `WantedBy=basic.target`), but has made no release since v1.0.4
(see ReimuNotMoe/ydotool#311).

Suggested fix: ship master's `Daemon/systemd/ydotoold.service.in` values as a patch to
`Daemon/ydotool.service.in` in the PKGBUILD until upstream releases.

Known weakness of this report: the package's install message asks users to join `input`,
which avoids the race; this setup deliberately does not.
~~~

## Link checks

On 2026-11-18,
`agent-browser` loaded every GitHub permalink in this record
and the cited line range matched the quoted text.
It also found the fragment text on the #221 and #311 pages.
gitlab.archlinux.org denied `agent-browser` with an Anubis "Access Denied" page,
so the GitLab links were not loaded in a browser.
Their content was read instead through the GitLab API at the same commit
`62c3920b955db1d244e5798f65d7ac31dbcedc40`.
The logs are `gh-links.log`, `other-links.log` and `gitlab-api-check.log`
in `~/temp/agent/ydotool-boot-race-doc-2026-11-18/keep/`.

[client-connect]: https://github.com/ReimuNotMoe/ydotool/blob/57ba7d0af525e82da2de0e275d169477f293b197/Client/ydotool.c#L157-L173
[daemon-open]: https://github.com/ReimuNotMoe/ydotool/blob/57ba7d0af525e82da2de0e275d169477f293b197/Daemon/ydotoold.c#L265-L270
[unit-v104]: https://github.com/ReimuNotMoe/ydotool/blob/57ba7d0af525e82da2de0e275d169477f293b197/Daemon/ydotool.service.in#L4-L13
[unit-master]: https://github.com/ReimuNotMoe/ydotool/blob/708e96ff27e381a8c549418a9d34cdde12305317/Daemon/systemd/ydotoold.service.in#L4-L15
[pr-221]: https://github.com/ReimuNotMoe/ydotool/pull/221#:~:text=the%20service%20starts%20too%20soon
[issue-311]: https://github.com/ReimuNotMoe/ydotool/issues/311#:~:text=hits%20the%20rate%20limiter
[arch-pkgbuild]: https://gitlab.archlinux.org/archlinux/packaging/packages/ydotool/-/blob/62c3920b955db1d244e5798f65d7ac31dbcedc40/PKGBUILD#L29-33
[arch-uinput-rules]: https://gitlab.archlinux.org/archlinux/packaging/packages/ydotool/-/blob/62c3920b955db1d244e5798f65d7ac31dbcedc40/80-uinput.rules#L3
[arch-install]: https://gitlab.archlinux.org/archlinux/packaging/packages/ydotool/-/blob/62c3920b955db1d244e5798f65d7ac31dbcedc40/ydotool.install#L1-4
[arch-issue-1]: https://gitlab.archlinux.org/archlinux/packaging/packages/ydotool/-/work_items/1
[logind-linger-enumerate]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind.c#L329-L330
[logind-startup-users]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind.c#L1297-L1302
[logind-wants-manager]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind-user.c#L412-L423
[logind-static-acl]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind-seat.c#L336-L347
[logind-static-dir]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind-seat.c#L349
[logind-set-active]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind-seat.c#L447-L453
[logind-session-vt]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/login/logind-session.c#L931-L932
[sd-restart-default]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/basic/constants.h#L6
[sd-startlimit-default]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/basic/constants.h#L20-L21
[sd-restart-man]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/man/systemd.service.xml#L611-L615
[sd-startlimit-man]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/man/systemd-system.conf.xml#L694-L696
[sd-after-man]: https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/man/systemd.unit.xml#L832-L838
