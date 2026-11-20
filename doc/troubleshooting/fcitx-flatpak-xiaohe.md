# Fcitx 5.1.23 Flatpak installation leaves Xiaohe Shuangpin unavailable

## Symptom

On 2026-11-02,
the user reported losing Xiaohe Shuangpin after a system upgrade and installation of Fcitx through Flatpak.
The running Flatpak offered `keyboard-us` and `keyboard-cn` in its `Default` input-method group.
The old native profile still selected `shuangpin`.

## Root cause

The observed installation had separate configuration and a missing Chinese input engine.
The upgrade itself was not established as the cause.

- `flatpak list` contained `org.fcitx.Fcitx5` version `5.1.23`,
  but no `org.fcitx.Fcitx5.Addon.ChineseAddons` extension.
- `~/.config/fcitx5/profile` contained `DefaultIM=shuangpin`.
- `~/.config/fcitx5/conf/pinyin.conf` contained `ShuangpinProfile=Xiaohe`.
- `~/.var/app/org.fcitx.Fcitx5/config/fcitx5/profile` contained `DefaultIM=keyboard-cn`.
- The running service returned `Call failed: Addon does not exist.` for its Pinyin configuration.

The installed extension's metadata names `The Fcitx Team` as developer,
identifies it as an add-on to `org.fcitx.Fcitx5`,
and describes support for Pinyin and other Chinese input functionality.
Fcitx's [installation guide][install] explicitly documents installing this extension with Flatpak.

## Verification

The read-only failure probe was:

```sh
# Fcitx session service: query the actual running engine configuration.
busctl --user call org.fcitx.Fcitx5 /controller \
  org.fcitx.Fcitx.Controller1 GetConfig s fcitx://config/addon/pinyin
```

Before recovery,
the probe failed with `Addon does not exist.`
After recovery,
it returned configuration containing `ShuangpinProfile=Xiaohe`.

The running group was checked separately:

```sh
# Fcitx session service: enumerate the configured input methods.
busctl --user call org.fcitx.Fcitx5 /controller \
  org.fcitx.Fcitx.Controller1 InputMethodGroupInfo s Default
```

Before recovery:

```text
# Observed D-Bus reply.
sa(ss) "us" 2 "keyboard-us" "" "keyboard-cn" ""
```

After recovery:

```text
# Observed D-Bus reply.
sa(ss) "us" 2 "keyboard-us" "" "shuangpin" ""
```

The installed Chinese Addons version was `5.1.15`,
Flatpak commit `9e21f2dbf9520c065bb52fd06e7526c9cdeeee241a7660843406e04962026bda`.
The Fcitx application commit was `41eaac428e5bf7dd55a09ddfa203f208a2bdb8ec79ddcf7f27972806d36ec227`.
These probes verify the running configuration and engine availability.
Typing into an application was not tested.

## Verified recovery

Installed the extension from the same system installation and Flathub remote as the application:

```sh
# Install the documented Chinese engine extension.
flatpak install --system --noninteractive --assumeyes flathub org.fcitx.Fcitx5.Addon.ChineseAddons
```

Backed up the Flatpak configuration and `kwinrc` under
`~/temp/agent/fcitx-xiaohe-backup-zSdf2u`.
An initial unsuccessful restart attempt also created
`~/temp/agent/fcitx-xiaohe-backup-cGn2Ka`.
(Note added 2026-11-19T19:12:45-05:00:
neither backup folder exists any more.)

With Fcitx stopped through Plasma's input-method setting,
copied the native `profile` and `conf/pinyin.conf` into the Flatpak's `config/fcitx5` directory.
Copied `user.dict` and `user.history` from `~/.local/share/fcitx5/pinyin`
into `~/.var/app/org.fcitx.Fcitx5/data/fcitx5/pinyin`.
The old source files were retained.
(Note added 2026-11-19T19:12:45-05:00:
`~/.local/share/fcitx5` and `~/.var/app/org.fcitx.Fcitx5` no longer exist,
so neither copy of the Pinyin user dictionary survives;
see "Native-only state and duplicate login launch".)
The recovery script is `~/temp/agent/restore-fcitx-xiaohe-2026-11-02.ts`.
(Note added 2026-11-19T19:12:45-05:00:
the script no longer exists.)
It restores Plasma's configured Fcitx desktop entry in a `finally` block.

Plasma restarted the Flatpak with the extension mounted.
The successful configuration notification used `kwriteconfig6 --notify`.
The configured trigger remains `Super+space`.
(Note added 2026-11-19T19:12:45-05:00:
that Flatpak configuration no longer exists.
The native Fcitx's live trigger keys are `Zenkaku_Hankaku`, `Hangul` and `Super+Super_L`,
and no live Fcitx hotkey is `Super+space`.)

Tradeoffs:
the old native profile replaces the newly created Chinese keyboard entry,
and the old Pinyin preferences include enabled cloud Pinyin and prediction.
Other Flatpak preferences were retained.
The native and Flatpak configurations remain separate after this copy.

## What does not work

- Installing the main Fcitx Flatpak alone did not supply the Chinese engine in this observed installation.
- The native Xiaohe preference was present,
  but it was not present in the new Flatpak configuration.
- Writing Plasma's input-method setting without `--notify`,
  then calling KWin's `reconfigure`,
  did not stop the running service within the recovery script's 15-second deadline.
  That attempt restored the original desktop-entry setting and did not copy Fcitx settings.

## Upstream filing decision

There is nothing to file for this incident:
the documented extension installation and migration of existing settings restored engine availability.

- Upstream fault:
  not established.
- Fixability:
  no upstream defect was identified to fix.
- Supported use case:
  the official installation guide documents the Chinese Addons extension.
- Contribution policy:
  not reviewed because no upstream filing is proposed.
- Maintainer intent:
  not assessed because no upstream change is proposed.
- Prototype:
  no upstream patch is needed for the verified recovery.

The repository's `.out-of-scope/` search found no Fcitx or Flatpak entry.
No upstream issue or comment was drafted or sent.

## Process shutdown investigation, 2026-11-04T09:36:00-05:00

The user asked whether Fcitx had been killed or had failed to start automatically.
This is a separate incident from the missing Chinese Addons extension.
The installed application remained version `5.1.23`,
with the application commit recorded in Verification.

### Observed lifecycle

The current boot's system journal retained the initial launch,
the later shutdown,
and the current launch:

- At `2026-11-02T13:29:06-05:00`,
  `kwin_wayland_wrapper[7192]` logged `Starting fcitx5 5.1.23` during desktop startup.
- At `2026-11-02T14:03:22-05:00`,
  the documented configuration recovery restarted Fcitx as PID `2749765`.
- At `2026-11-02T15:33:58-05:00`,
  that process began logging `Unloading addon` messages.
  The final message was `Unloading addon wayland` at `2026-11-02T15:34:02-05:00`.
  Its Flatpak scope ended at `2026-11-02T15:34:03-05:00`.
- The next matching launch record was `2026-11-04T09:30:38-05:00`,
  PID `3976727`,
  before this investigation began.
  Its process name was `fcitx5-bin`.

These records establish that automatic startup had worked and that a later process stopped.
The add-on unloading sequence suggests an orderly shutdown.
It does not identify the requester or exclude a handled termination signal.
No Fcitx kill or crash record was found in the current boot's inspected journal;
absence of such a record does not establish the shutdown's cause.
The initiating action remains unknown.

### Current configuration and verification

`~/.config/kwinrc:310` still selected the existing Flatpak desktop entry:
`/var/lib/flatpak/exports/share/applications/org.fcitx.Fcitx5.desktop`.
(Note added 2026-11-19T19:12:45-05:00:
`kwinrc` now selects the native entry,
`/usr/share/applications/org.fcitx.Fcitx5.desktop`,
and the Flatpak export path no longer exists.)
`systemctl --user status` reported its current Flatpak scope active since `09:30:38 EST`.
Read-only D-Bus calls with `--auto-start=no` returned:

- `CurrentInputMethod`:
  `shuangpin`.
- `InputMethodGroupInfo`:
  `keyboard-us` and `shuangpin`.
- Pinyin `GetConfig`:
  `ShuangpinProfile=Xiaohe`.

The process and controller were responsive;
typing in an application was not tested.
No restart or configuration change was performed during this investigation.
An exact-name process search for `fcitx5` missed the actual name `fcitx5-bin`,
so that search alone was not evidence of absence.

The shutdown evidence was queried with:

```sh
# Current boot journal: the previous Fcitx process's shutdown window.
systemd-run --user --scope --quiet \
  --property=CPUQuota=100% --property=MemoryMax=512M \
  --property=TasksMax=32 --property=IOWeight=10 \
  journalctl --boot --since='2026-11-02 15:33:45' --until='2026-11-02 15:34:15' \
  --no-pager --output=short-iso-precise _PID=2749765 _PID=6906 _PID=6887 _PID=5698
```

### Recovery and upstream disposition

The service had already resumed before inspection,
so no new recovery procedure was tested.
The shutdown was not reproduced and no regression test or upstream patch was attempted.
Upstream fault and fixability remain unestablished.
The supported Flatpak installation remains documented in the installation guide.
Contribution policy and maintainer intent were not reassessed because no upstream change is proposed.
There is nothing to file from this evidence;
no issue or comment was drafted or sent.

## Native-only state and duplicate login launch, 2026-11-19T19:12:45-05:00

The user asked to remove every non-native Fcitx copy
and to make sure the remaining copy launches correctly.
No Flatpak copy was left to remove.
The native copy was launched correctly by KWin,
and a second launch at every login came from the system autostart entry.

### Observed state

- `flatpak list --all` listed no installed refs at all.
  `flatpak --version` (`1.18.4`) and `flatpak remotes` (`flathub`, system) answered,
  so the empty list was not a broken command.
  `/var/lib/flatpak/app` and `~/.var/app/org.fcitx.Fcitx5` did not exist.
- When and by whom the Flatpak copy was removed is not established:
  `flatpak history` returned no rows.
- `which -a fcitx5` found only `/usr/bin/fcitx5`.
  The installed native packages were
  `fcitx5 5.1.23-1.1`,
  `fcitx5-chinese-addons 5.1.15-2.1`,
  `fcitx5-configtool 5.1.16-3.1`,
  `fcitx5-lua 5.0.18-1.1`
  and `fcitx5-qt 5.1.16-2.1`.
- `~/.config/kwinrc` `[Wayland]` selected the native entry:
  `InputMethod[$e]=/usr/share/applications/org.fcitx.Fcitx5.desktop`.
- The running Fcitx was PID `3645`,
  `/usr/bin/fcitx5`,
  started at `2026-11-19T11:12:49-05:00` with `kwin_wayland` (PID `3490`) as its parent,
  in the cgroup `session.slice/plasma-kwin_wayland.service`.
  It owned `org.fcitx.Fcitx5` on the session bus.
  `InputMethodGroupInfo` returned `keyboard-us` and `shuangpin`,
  and the Pinyin `GetConfig` reply contained `ShuangpinProfile=Xiaohe`.
- `~/.local/share/fcitx5` did not exist.
  A bounded `find` for `user.dict` and `user.history`
  under `~/.local/share`, `~/.config`, `~/.var` and `~/temp/agent`
  found neither,
  so the Pinyin engine has no learned user dictionary.
- Plasma Wayland is the only installed session:
  `/usr/share/wayland-sessions/plasma.desktop` exists
  and `/usr/share/xsessions` does not.

### Duplicate launch at login

KWin starts Fcitx from the `InputMethod` entry,
which is the launch path Fcitx documents for Plasma on Wayland ([Fcitx Wayland guide][wayland-kcm]).
A second launch came from `/etc/xdg/autostart/org.fcitx.Fcitx5.desktop`.
`systemd-xdg-autostart-generator` turns that entry into `app-org.fcitx.Fcitx5@autostart.service`
with an unconditional `ExecStart=:/usr/bin/fcitx5`.
The unit ran at `2026-11-19T11:12:54-05:00`,
after KWin's copy,
logged this line,
unloaded its addons,
and exited with status 0:

```text
# Journal of app-org.fcitx.Fcitx5@autostart.service.
E2026-11-19 11:12:54.839664 sharedlibraryloader.cpp:132] Failed to create addon: dbus Unable to request dbus name. Is there another fcitx already running?
```

The entry carries `X-KDE-Wayland-VirtualKeyboard=true`,
but that key does not stop autostart.
KWin's Virtual Keyboard settings page reads it to list candidate input methods
([`src/kcms/virtualkeyboard/kcmvirtualkeyboard.cpp:61-63`][kwin-vk-key] at commit `e99068b40ad042441b830b72d7968d98f505e26b`):

```cpp
// KDE/kwin src/kcms/virtualkeyboard/kcmvirtualkeyboard.cpp:61-63
    m_services = KApplicationTrader::query([](const KService::Ptr &service) {
        return service->property<bool>("X-KDE-Wayland-VirtualKeyboard");
    });
```

`rg 'X-KDE-Wayland-VirtualKeyboard'` over shallow clones of `KDE/kwin` (`src/`),
`KDE/plasma-workspace` and `KDE/plasma-desktop`,
taken on 2026-11-19,
found no other use,
and the generated unit has no condition.
The second process does no harm while it loses the bus name,
but it starts at every login.

### Applied fix

A same-named entry in the user's autostart folder with `Hidden=true`
disables a system autostart entry for that user ([autostart specification][autostart-hidden]):

```ini
# ~/.config/autostart/org.fcitx.Fcitx5.desktop
[Desktop Entry]
Hidden=true
```

Before writing,
the file was checked to be absent
and the `kwinrc` `InputMethod` line to be the native entry;
after writing,
the `InputMethod` line was re-read unchanged.

Verification,
using the real `/usr/lib/systemd/user-generators/systemd-xdg-autostart-generator`
with throwaway output folders,
so the live session was not reloaded:

- Fixture run with a throwaway `XDG_CONFIG_HOME`:
  without the override the generator produced `app-org.fcitx.Fcitx5@autostart.service` (the control),
  with it the unit was absent,
  and every other unit was kept.
- Run with the live environment after the write:
  the Fcitx unit was absent from the units it would create at next login.

A real login was not tested.
Fcitx and KWin were not restarted,
because the [Fcitx Wayland guide][wayland-kcm] warns that the socket KWin passes cannot be reused by a restarted Fcitx.

Tradeoffs:

- If the Virtual Keyboard setting is cleared,
  or a session whose compositor does not launch input methods is installed later
  (a Plasma X11 session,
  for example),
  nothing starts Fcitx at login until the override is deleted.
- Changes the Fcitx package makes to its autostart entry no longer apply for this user.

### Open gap: `XMODIFIERS`

No inspected session process carried `XMODIFIERS`,
`GTK_IM_MODULE`,
`QT_IM_MODULE` or `SDL_IM_MODULE`
(read from `/proc/<pid>/environ` of `Xwayland`, `plasmashell`, `fcitx5`, `dolphin`, `odytty`, `helium` and `firefox`).
Fcitx's Plasma setup asks for `XMODIFIERS=@im=fcitx` for XWayland applications ([Fcitx Wayland guide][wayland-xmodifiers]),
and `rg XMODIFIERS` over KWin's `src/` found no code that sets it.
Typing in an XWayland application was not tested.
The variable was not set in this session;
whether to set it is the user's decision.

### What does not work

- Searching `pgrep --list-full fcitx` or `flatpak list` for a second copy:
  both showed only the native process and no Flatpak refs,
  so there was nothing to uninstall.
- Relying on `X-KDE-Wayland-VirtualKeyboard=true` to keep the autostart copy from running:
  the generated unit ignores it,
  as the journal line shows.

### Upstream filing decision

There is nothing to file.

- Upstream fault:
  not established.
  The second process exits cleanly by design when another process owns the bus name.
- Fixability:
  not assessed,
  because no defect is established.
- Supported use case:
  Fcitx documents the Virtual Keyboard launch for Plasma on Wayland.
- Contribution policy:
  not reviewed because no filing is proposed.
- Maintainer intent:
  not assessed because no upstream change is proposed.
- Prototype:
  none needed;
  the per-user override is enough.

`.out-of-scope/` has no Fcitx,
Flatpak or autostart entry (listed 2026-11-19).
No issue or comment was drafted or sent.

[install]: https://fcitx-im.org/wiki/Install_Fcitx_5#:~:text=flatpak%20install%20org.fcitx.Fcitx5.Addon.ChineseAddons
[wayland-kcm]: https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland#:~:text=Use%20%22Virtual%20keyboard%22%20KCM%20to%20launch%20fcitx5.
[wayland-xmodifiers]: https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland#:~:text=XMODIFIERS%3D%40im%3Dfcitx%20for%20XWayland%20application
[kwin-vk-key]: https://github.com/KDE/kwin/blob/e99068b40ad042441b830b72d7968d98f505e26b/src/kcms/virtualkeyboard/kcmvirtualkeyboard.cpp#L61-L63
[autostart-hidden]: https://specifications.freedesktop.org/autostart/latest/#startup:~:text=which%20contains%20the%20key
