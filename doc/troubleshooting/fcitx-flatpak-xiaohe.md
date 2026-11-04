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

With Fcitx stopped through Plasma's input-method setting,
copied the native `profile` and `conf/pinyin.conf` into the Flatpak's `config/fcitx5` directory.
Copied `user.dict` and `user.history` from `~/.local/share/fcitx5/pinyin`
into `~/.var/app/org.fcitx.Fcitx5/data/fcitx5/pinyin`.
The old source files were retained.
The recovery script is `~/temp/agent/restore-fcitx-xiaohe-2026-11-02.ts`.
It restores Plasma's configured Fcitx desktop entry in a `finally` block.

Plasma restarted the Flatpak with the extension mounted.
The successful configuration notification used `kwriteconfig6 --notify`.
The configured trigger remains `Super+space`.

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

[install]: https://fcitx-im.org/wiki/Install_Fcitx_5#:~:text=flatpak%20install%20org.fcitx.Fcitx5.Addon.ChineseAddons
