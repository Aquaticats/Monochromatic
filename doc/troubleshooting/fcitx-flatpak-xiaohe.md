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

- Upstream fault: not established.
- Fixability: no upstream defect was identified to fix.
- Supported use case: the official installation guide documents the Chinese Addons extension.
- Contribution policy: not reviewed because no upstream filing is proposed.
- Maintainer intent: not assessed because no upstream change is proposed.
- Prototype: no upstream patch is needed for the verified recovery.

The repository's `.out-of-scope/` search found no Fcitx or Flatpak entry.
No upstream issue or comment was drafted or sent.

[install]: https://fcitx-im.org/wiki/Install_Fcitx_5#:~:text=flatpak%20install%20org.fcitx.Fcitx5.Addon.ChineseAddons
