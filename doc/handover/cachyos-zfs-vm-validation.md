# CachyOS encrypted ZFS VM validation handover

## Purpose

This handover preserves the live state of the CachyOS encrypted ZFS validation.
It must be updated after each material VM,
installer,
boot,
rollback,
or design milestone.

The current test is a disposable UEFI consumer installation on a virtual disk.
The user instructed that the VM and its disk must be retained after validation.
Do not clean up the domain,
virtual disk,
attached source images,
or evidence unless the user later authorizes cleanup.

## Current status

- **Installation result**:
  CachyOS Calamares displayed `All done.` and reported a successful installation.
- **Current transition**:
  The patched-layout installation reached Calamares **All done**.
  The local patch applied cleanly to the pinned installer commit and has SHA-256
  `e9d7271f4f7d2a110b8782049299ee765061d3914b344072d9fa027f2c7341f0`.
  The archive and patch hashes,
  patched `zfs.conf`,
  full Calamares session log,
  and effective settings are preserved on the evidence disk.
  The effective sequence contained all 7 custom ZFS jobs,
  and exactly 1 Calamares process was running before destructive confirmation.
  The summary selected only virtual `/dev/vda`,
  native encryption,
  no desktop,
  and the default base package groups.
  The installed ESP contains `EFI/ZFSBootMenu/vmlinuz-linux-cachyos.EFI`,
  measured at 42,821,120 bytes.
  Post-install ZFS properties showed `zroot/data/var` and `zroot/data/var/lib` at `none` plus `off`,
  while every explicit persistent child remained mounted.
  The VM shut down cleanly after **Done** closed Calamares and released the evidence-disk `tee` process.
  Installed boot reached ZFSBootMenu,
  accepted the disposable native-encryption passphrase,
  and reached tty1.
  The actual disposable account `useruser` authenticated successfully.
  Running root and `/var/lib/pacman` both resolve to `zroot/ROOT/default`.
  Persistent home resolves to `zroot/data/home/useruser`.
  The corrected-layout transaction installed `tree` and created
  `zroot/ROOT/be-20260829-135720-pre-install`.
  Current `tree` verification reported 7 total files and 0 altered files,
  and the root-backed post-snapshot marker is present.
  The first menu interceptor matched stale terminal text and was discarded.
  A later reboot attempt did not run because `sudo` timed out before receiving a password.
  After returning to a confirmed shell,
  unprivileged `systemctl reboot` succeeded.
  The exact countdown watcher then sent Escape at the real ZFSBootMenu countdown.
  The native-encryption passphrase was accepted after interception.
  ZFSBootMenu showed both default and pre-install environments.
  The pre-install environment booted to tty1.
  Its older password hash correctly rejected the rotated disposable password,
  so the revoked credential was not reused.
  ZFSBootMenu's read-only chroot showed `/` and `/var/lib/pacman` on the pre-install environment,
  no `tree` package record,
  no binary,
  and no post-snapshot marker.
  Returning to default accepted the rotated credential and restored coherent current state:
  `/` plus `/var/lib/pacman` on default,
  persistent home on `zroot/data/home/useruser`,
  `tree 2.3.2-1` with 0 altered files,
  the binary,
  and the marker.

  Setting only `org.zfsbootmenu:active=on` did not reveal the installer baseline.
  The clone had inherited `mountpoint=none` from `zroot/ROOT` because the baseline script never sets a local mountpoint.
  The ZFSBootMenu recovery shell correctly refused a property mutation while the pool was read-only.
  From the running default environment,
  setting baseline `mountpoint=/` produced measured properties `/`,
  `noauto`,
  and `active=on`.
  ZFSBootMenu then listed 3 environments and booted baseline to tty1.
  No revoked credential was entered.
  Its read-only chroot showed `/` and `/var/lib/pacman` on `zroot/ROOT/baseline`,
  no `tree` package record,
  no binary,
  and no post-snapshot marker.
  This proves baseline package coherence after the mountpoint correction.
  From default after credential rotation,
  a separate `zroot/ROOT/known-good` clone was created from
  `zroot/ROOT/default@known-good-20260829-validated`.
  Its measured properties are `mountpoint=/`,
  `canmount=noauto`,
  `active=on`,
  and inherited-parent command line.
  ZFSBootMenu listed and booted known-good.
  The rotated credential authenticated,
  `/` plus `/var/lib/pacman` resolved to known-good,
  persistent home remained separate,
  and `tree` plus its marker reported 0 altered files.
  Directly selecting default afterward reached tty1 and accepted the rotated credential,
  proving fallback from known-good.
  Complete issue and pull-request listing found no upstream duplicate.
  Valid open and closed keyword searches also found none for the package-state or baseline-mountpoint defects.
  No external issue was posted.

  The corrected default environment installed UWSM 0.26.2,
  labwc 0.20.2,
  sfwbar beta17,
  xwayland-satellite 0.8.2,
  and the rehearsed support packages through pacman-ZFS hooks.
  A long direct QMP typing attempt dropped characters and produced an uppercase `PRINTF` error.
  That path was abandoned.
  A retained read-only FAT stage image then overwrote the affected files with exact path-adjusted copies.
  Guest copies have 3 UWSM environment lines,
  3 labwc autostart lines,
  72 `rc.xml` lines,
  and 263 sfwbar configuration lines.
  XML parsing passed,
  no stale `/home/user/` path remained,
  xwayland-satellite is enabled,
  and `uwsm check may-start` passed on tty1.
  The render watcher timed out because its clock pattern was fixed at hour 15,
  but direct VM framebuffer inspection showed the black labwc desktop and sfwbar panel at hour 16.
  `wayland-wm@labwc.service` and xwayland-satellite were active.
  sfwbar,
  its `wlr-pager` child,
  swaync,
  and nm-applet were independently scoped under `app-graphical.slice`.
  The pager switched workspaces,
  fuzzel opened from the launcher and closed on a desktop click,
  and the taskbar right-click menu exposed **New instance**.
  That action launched another foot in its own service with `DISPLAY=:12`.

  The original Super+Enter foot inherited `DISPLAY=:0` because `launch-feedback` used scope-mode `uwsm app`.
  The canonical retained script and guest copy now use `uwsm app -t service`,
  matching the clean activation-environment rule already used by `launch-new`.
  The corrected path created an independent app service with `DISPLAY=:12`.
  A Fish login guard and tty1 autologin drop-in are installed for the CachyOS default Fish shell.
  Cold boot reached the display-manager-free session automatically.
  The final mechanism keeps the UWSM guard in root-backed `/etc/user-rollback/user/start-labwc.fish` and sources it
  through persistent `~/.config/fish/conf.d/uwsm.fish`.
  Both files passed `fish --no-execute`.
  Stopping `wayland-wm@labwc.service` made getty respawn the full session through that exact bridge;
  labwc and xwayland-satellite were active afterward.
  Installing `breeze-gtk` and selecting Breeze-Dark through GSettings corrected pavucontrol's initial light appearance.

  The known-good environment was promoted by setting pool `bootfs`.
  An unattended ZFSBootMenu countdown booted known-good,
  where `/` and `/var/lib/pacman` both resolved to that environment and `tree` reported 7 files with 0 altered files.
  Restoring `bootfs=zroot/ROOT/default` returned to the automatic UWSM plus labwc desktop.
  Default then reported coherent package state,
  active labwc and xwayland-satellite services,
  `DISPLAY=:12`,
  and Breeze-Dark.

  During the first timed-out `sudo` attempt,
  delayed input reached fish after the password prompt closed and printed a disposable VM user password as an unknown
  command.
  A later login-sequencing error rendered its first replacement as a username.
  Both values were immediately revoked and removed from the credential JSON.
  The final disposable password contains only ASCII letters and digits,
  remains only in the mode-0600 credential file,
  and has never been printed.
  No non-disposable resource used any disposable credential.
  The Pi transcript contains both revoked values,
  so neither may ever be restored.
  The ZFS passphrase was not exposed.

  Copying a byte-identical `/etc/shadow` hash with offline `usermod --root` was rejected as the revocation procedure:
  offline comparisons matched,
  but a fresh `sudo` authentication in default failed.
  No cause was assigned from that correlation.
  Running each target's supported `passwd --root` operation against a temporary `zfsutil` mount fixed default,
  and a fresh graphical-terminal `sudo -k; sudo -v` accepted the final credential after reboot.
  The same operation completed successfully for baseline and all 4 retained pre-install environments.
  Known-good had inherited the final credential when created.
  Baseline,
  known-good,
  default,
  and all 4 pre-install environments then passed fresh `sudo` authentication after reboot or direct selection.
  Each tested root matched the intended dataset.
  Every temporary mount was removed.
  The supported password operations rewrote each older environment's local password state without reusing either
  revoked password.
  The first installed disk reached Ly 1.4.1 after a successful native-encryption unlock.
  It contains Wayfire because the original runbook prescribed Wayfire as a temporary graphical base.
  The user correctly identified **No Desktop** as closer to the intended UWSM plus labwc system.
  The Wayfire disk is retained as installation and encrypted-boot evidence,
  but it is no longer the target desktop-validation base.
  Task 39 created a separate no-desktop validation domain and disk without deleting the Wayfire control.
  The authenticated live environment reached CachyOS Hello.
  The first watcher remained active because its case-sensitive `CachyOS` matcher missed OCR output `CachyOs`.
  A direct guest-frame capture proved the live environment was ready.
  The first installer-launch click used an incorrect coordinate and left CachyOS Hello open.
  A current guest frame located **Launch installer** at guest coordinate `642,605`,
  and the corrected VM-only click was sent.
  That action opened stock CachyOS Calamares rather than applying the pinned ZFS installer profile first.
  Stock Calamares was canceled at its bootloader page before partitioning or confirmation.
  Its session did not mutate the virtual disk.
  The read-only installer-source image was then mounted inside the guest,
  and the archive hash again matched the pinned value.
  The pinned third-party installer was launched from the verified extracted archive.
  It again reduced the bootloader page to `systemd-boot` and the partition plan to ZFS.
  `/dev/vda` was the only installation target,
  with the same 2 GiB EFI plus 126 GiB ZFS plan.
  Native ZFS encryption was enabled with matching disposable passphrases.
  **No Desktop** is selected.
  Calamares explicitly states that this choice starts in text-only mode and permits installing a desktop later.
  Only the default CachyOS,
  shell,
  base-development,
  and common package groups remained selected.
  User `user` and hostname `cachyos-zfs-nodesktop-vm` were configured with the disposable password.
  The destructive summary again named only `/dev/vda`,
  with a new GPT,
  2048 MiB FAT32 EFI partition,
  and 129021 MiB ZFS root partition.
  **Install Now** was confirmed after reviewing that boundary.
  The no-desktop installation reached terminal success:
  `All done.` and `CachyOS has been installed on your computer.`
  **Restart now** remained disabled.
  The installer was closed.
  The first attempted `systemctl poweroff` went into System Settings search rather than Konsole and had no effect.
  Refocusing the retained Konsole window exposed the installer cleanup transcript,
  including `Failed to export pool "cannot export 'zroot': pool is busy"` despite terminal Calamares success.
  No pool repair or clear command was used.
  The corrected guest `systemctl poweroff` reached `shut off` immediately.
  Flatpak-packaged `virsh change-media --eject --config` then exited with code 139 and no diagnostic message,
  but after-state domain XML proved the ISO source had been removed from persistent `hda` configuration.
  A fresh instrumented no-desktop control reached CachyOS Hello from the authenticated ISO.
  Stock Calamares was never launched,
  and `pgrep --exact calamares` confirmed that no Calamares process existed before the pinned setup began.
  Guest block-device inspection confirmed only virtual `vda`,
  read-only `vdb` and `vdc` source images,
  and writable `vdd` evidence storage.
  The installer archive again matched the pinned SHA-256 inside the guest.
  The evidence image was mounted with live-user ownership through FAT mount options.
  A direct `chown` correctly failed because FAT does not store Unix ownership.
  The pinned installer is running with `DEBUG=1` and teeing its terminal output to the evidence image.
  Before destructive confirmation,
  a second terminal proved that effective `/usr/share/calamares/settings.conf` contained all custom execution entries:
  script copy,
  ZFSBootMenu,
  encryption,
  mkinitcpio,
  baseline,
  pacman-ZFS,
  and user-home setup.
  The effective sequence and complete settings file were copied to the evidence image and synced.
  Calamares then selected **No Desktop** with only the required default package groups.
  The summary again named only `/dev/vda`,
  a 2048 MiB EFI partition,
  and a 129021 MiB encrypted ZFS root partition.
  **Install Now** was confirmed.
  The clean no-desktop control reached terminal Calamares success.
  Its retained log proves:

  - ZFSBootMenu image generation succeeded;
  - the EFI boot entry was created;
  - native-encryption configuration completed;
  - `zroot/ROOT/baseline` was created;
  - the ESP contains `EFI/ZFSBootMenu/vmlinuz-linux-cachyos.EFI`,
    measured at 42,821,120 bytes.

  The installer cleanup again reported `cannot export 'zroot': pool is busy`,
  so that warning is not sufficient to explain the failed run's empty ESP.
  The debug output,
  effective sequence,
  settings,
  full Calamares log,
  selected key events,
  archive hash,
  and ESP listing were synced to the writable evidence image.
  The guest entered `systemctl poweroff` after unmounting the evidence image and reached `shut off`.
  The evidence files were copied to
  `/var/home/user/temp/agent/cachyos-zfs-nodesktop-clean-evidence`,
  with SHA-256 recorded for every file.
  Persistent domain XML proves the authenticated ISO source was ejected.
  The clean no-desktop domain restarted from its installed EFI disk.
  ZFSBootMenu counted down for `zroot/ROOT/default`,
  requested the native-encryption passphrase,
  accepted the disposable credential,
  and booted to tty1.
  Login as `user` reached a fish shell with no preinstalled graphical session.
  The running root is `zroot/ROOT/default` on ZFS,
  and the pool reports healthy.
  Kernel `6.18.42-1-cachyos-lts` is paired exactly with `linux-cachyos-lts-zfs 6.18.42-1`.
  Normal kernel `7.2.2-1` and `linux-cachyos-zfs 7.2.2-1` are also installed.
  The root uses native `aes-256-gcm` encryption with passphrase format,
  encryption root `zroot`,
  and key location `file:///etc/zfs/keys/zroot.key`.
  The keyfile is root-owned,
  mode `000`,
  and 40 bytes.
  Firmware boot order starts with the ZFSBootMenu entry targeting the measured EFI image.
  The failed no-desktop domain previously restarted from its installed EFI disk,
  but the encrypted-boot watcher timed out after 635 seconds without seeing ZFSBootMenu or text login.
  A direct positive-control capture found the guest in the EDK II UEFI shell.
  UEFI mapped the 2048 MiB first partition as `FS0:`,
  but `dir` reported zero files and zero directories at its root.
  The installed ESP is empty,
  so no ZFSBootMenu image can be launched.
  This is a stop-condition failure rather than a usable no-desktop installation.
- **Installed path under test**:
  Third-party CachyOS encrypted ZFS installer with ZFSBootMenu.
- **Accepted recovery requirement**:
  A directly selectable boot-menu rollback target is no longer mandatory.
  Authenticated-media recovery is accepted because the retained recovery domain passed clone,
  direct-boot reconstruction,
  native-encryption authentication,
  graphical startup,
  and fresh user-authentication checks.
- **Recovery architecture result**:
  Keep the installed ZFSBootMenu EFI image as a tested fallback.
  Authenticated media can reconstruct an independent systemd-boot path that explicitly names a recovered clone.
- **Physical disks**:
  No physical host disk is attached to the VM.
- **Cleanup**:
  Forbidden by current user instruction.

## Authenticated recovery validation

Task 38 passed in a separate retained UEFI domain.
The test did not mutate the original patched-layout disk.
It used a compressed copy and the pinned authenticated CachyOS ISO as recovery media.
No physical host disk was attached.

### Retained recovery environment

- Domain:
  `cachyos-zfs-usb-recovery-validation`
- UUID:
  `1e85631f-d7e1-436e-a06b-c0e9d8d48758`
- Disk:
  `/mnt/encrypted/VMs/cachyos-zfs-usb-recovery-validation/disk.qcow2`
- Virtual size:
  128 GiB
- Allocated size after compressed copy:
  `7109890048` bytes
- SHA-256 before recovery work:
  `200c2d5296cb1a738e46e5d3d46bf838605718a490781bcaaeb69d6e176b6be0`
- Disk mode:
  `0600`
- Credential file:
  `/mnt/encrypted/VMs/cachyos-zfs-usb-recovery-validation/disposable-credentials.json`
- Credential-file mode:
  `0600`
- Current media state:
  the ISO is ejected from the persistent and live domain configuration.
- Current boot order:
  direct systemd-boot first,
  retained ZFSBootMenu second,
  and firmware UI third.
- Domain state at final verification on 2026-08-29:
  running on the final recovered graphical environment.

The original domain
`cachyos-zfs-layout-validation`
remains shut off with its disk unchanged by the recovery procedure.

### Recovery-media compatibility

The pinned live ISO booted kernel `7.1.6-1-cachyos` with OpenZFS module and userspace version `2.4.3-1`.
The installed target used OpenZFS `2.4.3-2`.
The live environment discovered the pool but did not mount it automatically.

The accepted import sequence was:

```bash
sudo zpool import -N -R /mnt/recovery -f zroot
sudo zfs load-key -L prompt zroot
```

`-N` prevented automatic dataset mounting.
`-R /mnt/recovery` established an alternate root.
The force import was confined to the retained single-owner disk copy after its source VM was shut off.
The protected native-encryption passphrase was accepted through the guest console.
No passphrase or current credential was printed.

### Clone-based restoration

The recovery environment listed candidate snapshots before selecting one.
It cloned:

```text
zroot/ROOT/default@be-20260829-154436-pre-install
```

into the writable dataset:

```text
zroot/ROOT/usb-recovery-20260829
```

The clone retained its origin relationship and was not an in-place rollback.
The original default,
baseline,
known-good,
and transaction environments remained intact.
The clone reported the expected pre-transaction marker and package state.
`tree` verified 7 files with 0 altered files.
Persistent home remained outside the recovered root on `zroot/data/home/useruser`.

The historical clone's local password state and root-backed Fish bridge were deliberately refreshed before final boot.
The password refresh used supported `passwd --root` behavior only after `findmnt` proved the mounted root identity.
Neither revoked credential was reused.
Both Fish files passed `fish --no-execute`.

### Direct boot reconstruction

The live environment mounted the recovered root and the copied disk's ESP after verifying both identities.
It rebuilt the clone's installed initramfs and installed a separate systemd-boot path:

- kernel:
  `/EFI/CachyOS/vmlinuz-linux-cachyos`
- initramfs:
  `/EFI/CachyOS/initramfs-linux-cachyos.img`
- loader entry:
  `/loader/entries/cachyos-usb-recovery.conf`
- kernel options:
  `zfs=zroot/ROOT/usb-recovery-20260829 rw`
- pool boot target:
  `zroot/ROOT/usb-recovery-20260829`

The copied kernel and initramfs matched their root-backed sources byte for byte.
The pool exported cleanly before the first direct boot.
The direct path booted the recovered clone without entering ZFSBootMenu.

The retained ZFSBootMenu EFI image was not removed.
A separate firmware entry targeting that image also booted the recovered clone successfully.
This proves the fallback independently of the direct systemd-boot path.

### Initramfs security correction

The first direct reconstruction was rejected after inspection found
`/etc/zfs/keys/zroot.key`
inside the initramfs copied to the unencrypted ESP.
The cause was the installer-provided `FILES` entry in `/etc/mkinitcpio.conf`.
A bootable result was not accepted as recovery proof while that key remained exposed.

The accepted clone now has:

```text
FILES=()
keylocation=prompt
```

Both installed presets were rebuilt.
`lsinitcpio` proved the key path absent before the rebuilt image replaced the ESP copy.
`cmp` then proved the ESP image and root-backed initramfs were byte-identical.

The generic `fsck` hook was also removed from this ZFS-only initramfs.
With a `zfs=` root command line,
that hook had emitted:

```text
ERROR: device '', not found. Skipping fsck.
```

The hook had no block-device root to check.
After removing it and rebuilding both presets,
the direct boot reached the native ZFS passphrase prompt without that error.
The accepted image remained free of `/etc/zfs/keys/zroot.key`.

### Final consumer-boundary checks

The final direct boot visibly required:

```text
Enter passphrase for 'zroot':
```

Entering the protected native-encryption passphrase reached the recovered labwc desktop.
The final runtime checks proved:

- `/` is `zroot/ROOT/usb-recovery-20260829`;
- `/var/lib/pacman` has the same recovered-root rollback semantics;
- home is the persistent `zroot/data/home/useruser` dataset;
- `/proc/cmdline` names `zroot/ROOT/usb-recovery-20260829`;
- pool `bootfs` names `zroot/ROOT/usb-recovery-20260829`;
- pool `keylocation` is `prompt`;
- `FILES=()` remains configured;
- the ZFS hook remains present;
- the generic `fsck` hook remains absent;
- labwc is active;
- xwayland-satellite is active;
- `tree` package files and pacman metadata agree;
- the ESP initramfs does not contain `/etc/zfs/keys/zroot.key`;
- direct systemd-boot remains first in firmware order;
- retained ZFSBootMenu remains second;
- a fresh `sudo -k; sudo -v` accepted the protected final disposable credential.

The authenticated recovery requirement is therefore satisfied.
This does not prove that the official CachyOS installer owns the direct architecture.
It proves that pinned authenticated media can safely reconstruct
and boot it from the validated third-party installation.

## User control constraint

Do not capture host screenshots,
send synthetic input to the host,
or otherwise control the host desktop.

Allowed VM interaction is isolated to the guest through libvirt,
QEMU monitor input,
and the loopback-only VNC display.
Guest framebuffer captures are VM evidence,
not host screenshots.

No Cage,
`xdotool`,
Spectacle,
virt-manager window automation,
or host desktop input has been used.

## Authenticated installation input

- ISO path:
  `/var/home/user/Downloads/cachyos-desktop-linux-260809.iso`
- Measured ISO size:
  `3188850688` bytes
- Verified ISO SHA-256:
  `959f6577f45e25ee9fd8c220fd221b08e4ea79412c7315c0f922dd6d86d5e33c`
- Installer commit:
  `9d587de2d34a35ea33094735002d8599afed7eac`
- Installer archive path:
  `/var/home/user/temp/agent/cachyos-zfs-installer-9d587de2.tar.gz`
- Verified installer archive SHA-256:
  `f83565958f5d32054c2a6dbb7bc0295eedc69b21de47d18bcdd1d65ee3d2073c`

The archive was copied into a read-only FAT source image,
mounted inside the live guest,
and hashed again inside the guest.
The guest-reported hash matched the pinned value.

## VM identity and storage

- Libvirt connection:
  `qemu:///session`
- Domain name:
  `cachyos-zfs-validation`
- Domain UUID:
  `85886808-bc42-4ee1-8b07-b91599cae9f7`
- Firmware:
  UEFI through edk2
- Memory:
  8192 MiB
- Virtual CPUs:
  4
- Primary virtual disk:
  `/mnt/encrypted/VMs/cachyos-zfs-validation/disk.qcow2`
- Virtual disk capacity:
  128 GiB
- Storage behavior:
  Sparse qcow2 in a Btrfs directory carrying the `C` no-copy-on-write attribute
- Disposable credential record:
  `/mnt/encrypted/VMs/cachyos-zfs-validation/disposable-credentials.json`
- Credential-file mode:
  `0600`
- Labwc rehearsal image:
  `/mnt/encrypted/VMs/cachyos-zfs-validation/labwc-configs.img`
- Installer source image:
  `/mnt/encrypted/VMs/cachyos-zfs-validation/installer-source.img`

Never print or copy the credential values into this handover,
logs,
commands,
or user-facing messages.
The guest input helper can read the required JSON property directly.

## No-desktop VM identity and storage

- Libvirt connection:
  `qemu:///session`
- Domain name:
  `cachyos-zfs-nodesktop-validation`
- Domain UUID:
  `2bb347c3-34ec-4d61-af26-1f691923693e`
- Firmware:
  UEFI through edk2
- Memory:
  8192 MiB
- Virtual CPUs:
  4
- Primary virtual disk:
  `/mnt/encrypted/VMs/cachyos-zfs-nodesktop-validation/disk.qcow2`
- Virtual disk capacity:
  128 GiB
- Storage behavior:
  Sparse qcow2 with the `C` no-copy-on-write attribute
- Disposable credential record:
  `/mnt/encrypted/VMs/cachyos-zfs-nodesktop-validation/disposable-credentials.json`
- Guest display:
  loopback-only VNC
- Guest framebuffer evidence:
  `/var/home/user/temp/agent/cachyos-zfs-nodesktop-vnc.png`

This domain reuses the read-only labwc and installer-source images from the retained Wayfire control.
The authenticated ISO is attached only as virtual CD-ROM installation media.
No physical host block device is attached.

Use the VM-only input helper with this environment variable:

```bash
VM_DOMAIN=cachyos-zfs-nodesktop-validation \
  node /var/home/user/temp/agent/guest-vm-input.mjs
```

## Clean no-desktop control identity and storage

- Domain name:
  `cachyos-zfs-nodesktop-clean-validation`
- Domain UUID:
  `f9a2f58d-7740-4037-b2c1-86d4effea718`
- Primary virtual disk:
  `/mnt/encrypted/VMs/cachyos-zfs-nodesktop-clean-validation/disk.qcow2`
- Virtual disk capacity:
  128 GiB sparse qcow2
- Disposable credential record:
  `/mnt/encrypted/VMs/cachyos-zfs-nodesktop-clean-validation/disposable-credentials.json`
- Writable 256 MiB evidence image:
  `/mnt/encrypted/VMs/cachyos-zfs-nodesktop-clean-validation/installer-evidence.img`
- Guest framebuffer evidence:
  `/var/home/user/temp/agent/cachyos-zfs-nodesktop-clean-vnc.png`

This control uses UEFI,
8192 MiB memory,
four virtual CPUs,
loopback-only VNC,
and no virtual GPU acceleration.
It reuses the authenticated ISO and read-only source images.
No physical block device is attached.

The evidence image must retain:

- pinned-installer debug output;
- the effective Calamares settings and custom execution sequence;
- the complete Calamares installation log.

## Patched-layout control identity and storage

- Domain name:
  `cachyos-zfs-layout-validation`
- Domain UUID:
  `65a92c82-6e71-47a1-812e-149795db7708`
- Primary virtual disk:
  `/mnt/encrypted/VMs/cachyos-zfs-layout-validation/disk.qcow2`
- Patched source image:
  `/mnt/encrypted/VMs/cachyos-zfs-layout-validation/patched-installer-source.img`
- Writable evidence image:
  `/mnt/encrypted/VMs/cachyos-zfs-layout-validation/installer-evidence.img`
- Disposable credentials:
  `/mnt/encrypted/VMs/cachyos-zfs-layout-validation/disposable-credentials.json`
- Guest framebuffer:
  `/var/home/user/temp/agent/cachyos-zfs-layout-vnc.png`
- Running display:
  `vnc://127.0.0.1:1`
- Read-only desktop configuration stage:
  `/mnt/encrypted/VMs/cachyos-zfs-layout-validation/labwc-stage.img`

The read-only source image contains the unchanged pinned archive and the 896-byte local layout patch.
The patch changes only
`src/calamares/etc/calamares/modules/zfs.conf`.
It keeps `data/var` and `data/var/lib` as unmounted namespace parents.
No physical block device is attached.

The copied patched-install evidence has these SHA-256 values:

- `bin-install.log`:
  `5460e3ac69a2739479ac0336d7da927542a971c8f0e987fd16a7270c6e12f027`
- `calamares.install.log`:
  `a7e22594c8aba7d2b6cbc591f24f97705cc8427af045d1a51b95becdf96f5712`
- `effective-sequence.txt`:
  `4ce470be1196cd4d73c022e416c39b67eafa49c427b4e282aa3585466f2adbee`
- `esp-files.txt`:
  `105d48818686d014d3171a35b2f076ce4c9eaaa415c2fb2f8027b8a41e6a6750`
- patched `zfs.conf`:
  `0a3c855dcd5a3c8c61c9512bd5ed22b1d5898b0245f19d69dcc20671f8dc0c30`
- `zfs-layout.txt`:
  `a56f8b18aad51d5d700bee5c511b8330443633e873a0b22978df5f9f24d6d4f0`

The `useruser` account and dataset were an input-automation mistake,
not an installer home-layout anomaly.
Entering the full name `User` auto-filled login `user`.
The automation then appended another `user` instead of replacing the existing field.
Installed login succeeded as `useruser`,
and its home is correctly backed by `zroot/data/home/useruser`.
A physical run must replace or confirm every auto-filled identity field before continuing.

## Attached guest storage

The installation session used:

- `/dev/vda`:
  primary 128 GiB qcow2 installation target;
- `/dev/vdb`:
  read-only FAT image containing the rehearsed labwc plans and configuration;
- `/dev/vdc`:
  read-only FAT image containing the pinned installer archive;
- `hda`:
  authenticated CachyOS ISO.

Only `/dev/vda` appeared as the Calamares installation target.
The reviewed summary named `/dev/vda` explicitly before installation was confirmed.

## VM display and guest input

The initial SPICE plus virgl display stopped exposing a QEMU screenshot surface after the live desktop started.
The domain was converted to loopback-only VNC,
and virtual GPU acceleration was disabled.
The current display resolves through:

```bash
flatpak run \
  --command=virsh \
  org.virt_manager.virt-manager \
  --connect qemu:///session \
  domdisplay cachyos-zfs-validation
```

The expected installed configuration is `vnc://127.0.0.1:0` while the VM is running.

Guest frames can be captured without touching the host desktop:

```bash
flatpak run \
  --command=gvnccapture \
  org.virt_manager.virt-manager \
  127.0.0.1:0 \
  /var/home/user/temp/agent/cachyos-zfs-validation-vnc.png
```

The VM-only QMP input helper is:

```text
/var/home/user/temp/agent/guest-vm-input.mjs
```

It is hard-coded to `cachyos-zfs-validation` and supports guest-only click,
text,
JSON-property text,
and key-chord operations.
It does not generate host input.

## UEFI evidence

Inside the live guest,
`efibootmgr --verbose` succeeded and listed:

- a UEFI QEMU DVD-ROM boot entry;
- UEFI Misc Device entries;
- an active boot order.

This proves that the installation environment booted through virtual UEFI rather than legacy BIOS.

## Installer selections actually exercised

The installed VM used:

- American English;
- time zone `America/New_York`;
- Generic 105-key PC;
- English US default keyboard layout;
- the installer page's only offered bootloader choice,
  `systemd-boot`;
- erase only `/dev/vda`;
- ZFS root;
- native ZFS encryption enabled;
- matching disposable ZFS passphrases;
- Wayfire as the selected initial desktop;
- default required CachyOS,
  shell,
  base,
  and common package groups;
- username `user`;
- hostname `cachyos-zfs-vm`;
- the disposable user password for both user and administrator access.

The current package page did not expose the kernel-choice page described by the draft runbook.
No real-time kernel was selected manually.
Installed package verification must establish the actual kernel and exact ZFS module pairing after first boot.

## No-desktop boot failure under investigation

The no-desktop installer reported terminal success despite producing an empty ESP.
The retained Konsole transcript also showed Calamares fail to export `zroot` during cleanup because the pool was busy.
No destructive repair was attempted.

The initial hypothesis that **No Desktop** omitted every installed kernel package was wrong.
An authenticated-ISO inspection imported the same pool read-only after confirming no concurrent VM used it.
The first import correctly refused the stale host-id evidence left by the failed installer export.
A forced but still read-only import then reported `pool 'zroot' is healthy`,
and the disposable native-encryption passphrase loaded the key successfully.

The installed root contains:

- `linux-cachyos 7.2.2-1`;
- `linux-cachyos-zfs 7.2.2-1`;
- `linux-cachyos-lts 6.18.42-1`;
- `linux-cachyos-lts-zfs 6.18.42-1`;
- both kernel images and both initramfs images under root-backed `/boot`.

The exact kernel-module pairs are therefore present.
The same inspection found:

- no installed `zfsbootmenu` package;
- no installed `zfs-meta` package;
- no `zroot/ROOT/baseline` dataset;
- no pacman-ZFS hook artifacts;
- no ZFSBootMenu image or any other file on the ESP.

The package log records the base ZFS and exact kernel-module installation,
but no ZFSBootMenu package transaction.
This proves that the custom post-install pipeline did not execute or persist,
not that image generation lacked a kernel.
The reason the custom jobs were skipped remains unresolved.
Launching and canceling stock Calamares before the pinned installer is a contamination unique to this run,
so a fresh no-desktop control must avoid that sequence.

After inspection,
ordinary ZFS unmounts completed but export still reported the pool busy.
`fuser` named live-system systemd namespace-resource processes as holders.
No forced export,
clear,
or repair was used;
the live guest was shut down to release the disposable pool.

## Desktop-selection correction

The Calamares desktop page visibly offered **No Desktop**.
The runbook nevertheless prescribed Wayfire as a known graphical fallback,
and that instruction was followed without reconsidering whether the fallback matched the target architecture.
That was an error.

Wayfire adds an unrelated compositor,
its packages,
and a preconfigured graphical login path before the intended UWSM plus labwc stack is tested.
This weakens the consumer-boundary test because successful graphics could depend on the fallback environment.
**No Desktop** is the correct installation choice for this migration.
It forces the validation to prove that the intended session packages,
services,
portal selection,
login flow,
and configuration are sufficient by themselves.

The first disk remains useful for:

- authenticated installer execution;
- the `/dev/vda`-only erase boundary;
- native ZFS encryption;
- ZFSBootMenu launch and passphrase prompt;
- first installed boot to Ly.

It is not accepted as the final UWSM plus labwc validation environment.
A second retained disk and domain must repeat installation with **No Desktop**.
The migration runbook was corrected before that repeat installation began.

## Reviewed destructive summary

Before installation,
Calamares reported:

- erase disk `/dev/vda` only;
- create a new GPT partition table on `/dev/vda`;
- create a `2048 MiB` FAT32 EFI system partition;
- mount the EFI partition at `/boot/efi`;
- create a `129021 MiB` ZFS root partition;
- install CachyOS on the new ZFS system partition.

This differs from the 4 GiB CachyOS Limine layout investigated for the separate Btrfs candidate.
The ZFS validation produced a 2 GiB EFI plan.

## Installation completion evidence

The managed guest-frame watcher reached this terminal text:

```text
All done.
CachyOS has been installed on your computer.
You may now restart into your new system, or continue using the CachyOS Live environment.
```

Watcher process output was retained by the process tool under process name `wait-cachyos-install`
before process cleanup.
The latest guest completion frame is:

```text
/var/home/user/temp/agent/cachyos-zfs-validation-vnc.png
```

## Shutdown transition evidence

The host-side ACPI request did not stop the live environment within 2 minutes.
This did not prove that CachyOS cannot shut down.
Guest evidence showed that the first terminal command was mistyped because input began before Konsole had focus:

```text
ls poweroff
ls: cannot access 'poweroff': No such file or directory
```

The command was not a `systemctl` failure.
The corrected `systemctl poweroff` was entered only after the terminal was visibly focused.
Process `wait-cachyos-poweroff-retry` observed `shut off` immediately and exited successfully.

## Installed boot transition

After the successful installer result:

- the live system reached `shut off` through guest `systemctl poweroff`;
- the authenticated ISO was ejected from persistent `hda` configuration;
- XML inspection found no remaining ISO source path;
- `/dev/vdb` and `/dev/vdc` remained attached as instructed;
- the domain started successfully;
- its VM-only display returned at `vnc://127.0.0.1:0`.

ZFSBootMenu reached an 8-second automatic-boot countdown for `zroot/ROOT/default` without prior input.
This countdown alone does not establish that encryption was bypassed:
ZFSBootMenu can discover an encrypted environment before requesting its key during boot selection.
The next watcher distinguished the states successfully.
After the countdown,
ZFSBootMenu displayed `Enter passphrase for 'zroot':`.
No second input had been sent before that prompt appeared.
The disposable ZFS passphrase was accepted for the continuing boot attempt.
Boot then reached Ly 1.4.1 with the intended hostname,
Wayfire session,
and `user` login.

## Pacman hook transaction evidence

On the clean no-desktop control,
`pacman --sync --noconfirm tree` exercised the installed hooks at the consumer boundary.
The pre-transaction hook created
`zroot/ROOT/be-20260829-125545-pre-install`
from snapshot `zroot/ROOT/default@be-20260829-125545-pre-install`.
Its description is `Pre-install Packages: tree`,
and its command line inherits `%{parent}`.
The post-transaction hook found one retained environment,
kept it under the configured retention count of 24,
verified the ZFSBootMenu EFI image,
and verified root-backed kernels.

After the transaction,
`/usr/bin/tree` existed and pacman reported `tree 2.3.2-1` in the running default environment.
A root-backed marker was created after the hook snapshot.

ZFSBootMenu listed and booted the pre-install environment successfully.
Its running root was `zroot/ROOT/be-20260829-125545-pre-install`.
The root-backed `tree` binary and post-snapshot marker were absent as expected,
but persistent `/var/lib` remained mounted from `zroot/data/var/lib`.
Pacman therefore still reported `tree 2.3.2-1`.
`pacman -Qkk tree` reported missing `/usr/bin/tree` and its manual page,
with 2 altered files.

Rebooting `zroot/ROOT/default` restored the binary,
marker,
and coherent pacman state.
`pacman -Qkk tree` then reported 7 total files and 0 altered files.
The pre-transaction clone is bootable but is not a coherent package rollback.
The hidden baseline has the same dataset boundary and must not be treated as safe system rollback.

A real-ZFS before-and-after prototype on a retained 2 GiB virtual disk proved the cause and a minimal layout change.
With persistent `/var` and `/var/lib` parents,
the cloned binary was `before` while the database was `after`.
With those parents changed to `mountpoint=none` and `canmount=off`,
both values in the clone were `before`,
and explicit `/var/lib/containers` persistence still mounted correctly.
The prototype pools were destroyed,
and the virtual disk was detached but retained.

The full diagnosis and fileable upstream draft are in
`doc/troubleshooting/cachyos-zfs-boot-environment-pacman-state.md`.
No upstream issue was posted.

## Immediate next actions

Rollback promotion,
return to default,
credential revocation,
UWSM plus labwc validation,
and authenticated-media recovery are complete under tasks 37,
38,
and 40.

1. Retain every domain,
disk,
source image,
credential file,
and evidence artifact.

1. Keep formal finalist scoring separate from the user-authorized CachyOS adoption path.
Tasks 21,
23,
24,
and 25 remain pending.

1. Do not begin physical installation until this recovery milestone is committed and the runbook checks pass.

1. Require the protected Samsung SSD to be physically disconnected before the physical installer runs.

## Authenticated recovery design

The relaxed requirement does not make snapshots independently bootable.
The validated authenticated-media design proved all of these operations:

- boot an authenticated environment with a ZFS module compatible with the pool;
- discover and import the pool without mounting datasets at unintended host paths;
- load the native encryption key;
- inspect candidate root snapshots;
- clone a chosen snapshot to a new writable root dataset or perform another explicitly safe restoration;
- preserve persistent home and data datasets;
- make the restored root the intended boot target;
- mount the EFI system partition;
- rebuild matching kernel and initramfs artifacts;
- create or repair a systemd-boot entry that names the restored root dataset;
- export the pool cleanly;
- reboot successfully without relying on the original broken root;
- retain an escape path if the first recovery attempt does not boot.

A root snapshot does not contain the FAT EFI system partition.
Without ZFSBootMenu,
matching boot artifacts become explicit recovery work rather than menu-managed state.

The result distinguishes:

- third-party installer ownership of the installed ZFS layout and ZFSBootMenu setup;
- manual authenticated-media ownership of the direct recovery reconstruction;
- routine ZFSBootMenu rollback from emergency clone reconstruction;
- keyless ESP artifacts from root-keystore convenience;
- source snapshot retention from writable recovered-root activation.

Do not claim that the official CachyOS installer owns this architecture.
The tested direct path is an emergency reconstruction procedure,
not a substitute installer profile.
Keep ZFSBootMenu as the routine rollback interface and tested fallback.

## Existing project documents

- Migration runbook:
  `doc/runbook/migrate-bazzite-to-cachyos-zfsbootmenu.md`
- Adoption decision:
  `doc/decision/cachyos-zfs-desktop.md`
- Active technology audit:
  `doc/audit/tech-rolling-linux-desktop-with-encrypted-boot-m-911c4730-vet-2026-08-29.md`
- CachyOS ZFS operational ownership investigation:
  `doc/troubleshooting/cachyos-zfs-installer-operational-ownership.md`
- OpenZFS latency investigation:
  `doc/troubleshooting/openzfs-single-device-latency-masking.md`

These documents retain ZFSBootMenu as part of the selected architecture.
The direct systemd-boot path is a tested authenticated-media recovery result,
not a replacement for routine ZFSBootMenu ownership.

## Task state

- Task 34,
  verify VM inputs and storage:
  completed.
- Task 35,
  create disposable UEFI VM:
  completed.
- Task 36,
  install encrypted CachyOS ZFS:
  completed after ZFSBootMenu unlock and installed boot reached Ly 1.4.1.
- Task 37,
  validate rollback and desktop:
  completed after durable promotion,
  return-to-default,
  and graphical-session checks.
- Task 38,
  evaluate ZFS recovery without ZFSBootMenu:
  completed after authenticated-media clone recovery,
  keyless direct boot,
  graphical startup,
  and fresh sudo authentication.
- Task 39,
  install no-desktop CachyOS validation:
  completed after encrypted boot reached the authenticated tty1 user shell.
- Task 40,
  rehearse boot-environment credential revocation:
  completed after all retained environments passed fresh authentication.

## Stop conditions

Stop and diagnose before continuing if:

- the domain is not shut off after the requested ACPI shutdown;
- the ISO cannot be ejected from both live and persistent configuration;
- firmware does not launch the installed EFI path;
- the native ZFS passphrase is rejected;
- ZFSBootMenu cannot discover `zroot/ROOT/default`;
- boot requires a second unexpected pool passphrase;
- the running root is not a ZFS dataset under `zroot/ROOT`;
- package versions do not provide an exact CachyOS kernel and ZFS-module match;
- the pool reports data errors;
- the installed environment differs materially from the reviewed summary;
- any command would touch a physical block device;
- a recovery experiment would destroy the only working VM root;
- host desktop control would be required.
