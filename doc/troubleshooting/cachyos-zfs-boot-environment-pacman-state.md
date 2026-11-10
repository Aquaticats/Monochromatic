# CachyOS ZFS installer 0.5.1 package rollback leaves pacman state ahead of boot environment

## Symptom

A package transaction creates a ZFS boot environment successfully,
and ZFSBootMenu boots that environment successfully.
The root-backed package files roll back,
but pacman's database does not.

The reproduced transaction installed `tree 2.3.2-1`.
The installed pre-transaction hook reported:

```text
Creating boot environment: be-20260829-125545-pre-install
Type: install
Description: Pre-install Packages: tree
Successfully created boot environment: be-20260829-125545-pre-install
```

After booting that environment,
the measured state was:

```text
ROOT
zroot/ROOT/be-20260829-125545-pre-install zfs
TREE_BINARY
ABSENT
PACMAN_DB
tree 2.3.2-1
MARKER
ABSENT
PACMAN_MOUNT
zroot/data/var/lib zfs
```

`pacman -Qkk tree` then reported:

```text
warning: tree: /usr/bin/tree (No such file or directory)
warning: tree: /usr/share/man/man1/tree.1.gz (No such file or directory)
tree: 7 total files, 2 altered files
```

Booting back into `zroot/ROOT/default` restored coherent state:

```text
ROOT
zroot/ROOT/default zfs
TREE_BINARY
/usr/bin/tree
PACMAN_DB
tree 2.3.2-1
MARKER
PRESENT
VERIFY
tree: 7 total files, 0 altered files
```

The pool remained healthy throughout the test.
No pool clear,
rollback,
or repair command was used.

## Root cause

The tested installer is version `0.5.1` at commit
`9d587de2d34a35ea33094735002d8599afed7eac`.
Its documented model says installed packages belong to boot environments.
`README.md:235-241` states:

```markdown
Datasets under `zroot/ROOT/` are included in boot environments. Changes to the
operating system, installed packages, and system configuration live here. When
you roll back to a previous boot environment, these changes revert.

Datasets under `zroot/data/` persist across boot environments. User files,
application data, logs, and container images remain unchanged when you switch
boot environments. This separation prevents data loss during system rollbacks.
```

The implemented dataset layout places all of `/var` and `/var/lib` outside the root boot environment.
`src/calamares/etc/calamares/modules/zfs.conf:43-49` defines:

```yaml
    # /var hierarchy (persistent, not part of boot envs)
    - dsName: data/var
      mountpoint: /var
      canMount: on
    - dsName: data/var/lib
      mountpoint: /var/lib
      canMount: on
```

Pacman's installed-package database is under `/var/lib/pacman`.
The consumer test confirmed that `/var/lib` was mounted from `zroot/data/var/lib` while the selected root was
`zroot/ROOT/be-20260829-125545-pre-install`.
The database therefore persisted across the root switch.

The hook is configured to snapshot only a dataset under `zroot/ROOT`.
`src/pacman-zfs/usr/local/lib/pacman-zfs-common.sh:22-24` sets:

```bash
  # Set defaults
  RETENTION_COUNT="${RETENTION_COUNT:-24}"
  ZFS_ROOT_POOL="${ZFS_ROOT_POOL:-zroot/ROOT}"
```

`src/pacman-zfs/usr/local/bin/pacman-zfs-pre:143-158` derives a snapshot from the current root environment,
then clones only that snapshot:

```bash
# Generate BE name
timestamp=$(generate_timestamp)
be_name="be-${timestamp}-pre-${be_type}"
full_be_path="${ZFS_ROOT_POOL}/${be_name}"
snapshot_name="${ZFS_ROOT_POOL}/${current_be}@${be_name}"

# Create snapshot of current BE
zfs snapshot "$snapshot_name" \
  || abort_transaction "Failed to create snapshot: $snapshot_name"

# Clone snapshot to new BE
zfs clone "$snapshot_name" "$full_be_path" \
  || abort_transaction "Failed to clone snapshot to: $full_be_path"
```

No hook operation snapshots,
clones,
or selects `zroot/data/var/lib` with the root environment.
Consequently:

1.  The pre-transaction root snapshot captures package files such as `/usr/bin/tree`.
2.  Pacman installs package files into the running root dataset.
3.  Pacman records the installation under persistent `/var/lib/pacman`.
4.  ZFSBootMenu later selects the pre-transaction root clone.
5.  The selected root lacks the newly installed files,
    while persistent pacman state still reports the package as installed.

This is a dataset-boundary mismatch,
not a ZFSBootMenu selection failure and not pool damage.

## Verification

### Inputs

The consumer reproduction used:

- CachyOS desktop ISO `260809`;
- ISO SHA-256
  `959f6577f45e25ee9fd8c220fd221b08e4ea79412c7315c0f922dd6d86d5e33c`;
- installer commit
  `9d587de2d34a35ea33094735002d8599afed7eac`;
- installer archive SHA-256
  `f83565958f5d32054c2a6dbb7bc0295eedc69b21de47d18bcdd1d65ee3d2073c`;
- encrypted single-device `zroot` inside a retained UEFI VM;
- `zfs-utils 2.4.3-2`;
- normal kernel and module pair `7.2.2-1`;
- LTS kernel and module pair `6.18.42-1`;
- ZFSBootMenu `3.1.0-1`.

The retained clean VM is `cachyos-zfs-nodesktop-clean-validation`.
Its installation evidence is under
`/var/home/user/temp/agent/cachyos-zfs-nodesktop-clean-evidence`.

### Consumer reproduction

From the installed default environment:

```bash
sudo pacman --sync --noconfirm tree
sudo touch /etc/tree-installed-after-snapshot
systemctl reboot
```

In ZFSBootMenu,
select `zroot/ROOT/be-20260829-125545-pre-install`.
After logging in:

```bash
findmnt --noheadings --output SOURCE,FSTYPE /
command -v tree || true
pacman --query tree
if test -e /etc/tree-installed-after-snapshot; then
  printf '%s\n' PRESENT
else
  printf '%s\n' ABSENT
fi
findmnt --noheadings --output SOURCE,FSTYPE /var/lib
pacman --query --check --check tree
```

The root dataset and marker moved to their pre-transaction state.
The persistent pacman database did not.

### Clean patterns

The following paths behaved cleanly:

- The pacman pre-hook created a bootable root clone.
- ZFSBootMenu listed and booted the clone after native-encryption unlock.
- Root-backed files and the post-snapshot marker matched the selected root snapshot.
- Returning to `zroot/ROOT/default` restored the package binary,
  marker,
  and pacman database to a coherent state.
- `pacman -Qkk tree` reported `0 altered files` in `default`.

### Failing patterns

The following combination failed:

- package files under root-backed `/usr`;
- package database under persistent `/var/lib/pacman`;
- a hook that snapshots only `zroot/ROOT/<environment>`;
- booting a pre-transaction root clone after pacman has committed its persistent database update.

The failure is deterministic for package operations that change root-backed files and pacman database state.
Install,
upgrade,
and removal operations can all create analogous disagreement.

## Verified workarounds

### Return to the original environment

Booting `zroot/ROOT/default` restored the tested package state:
`/usr/bin/tree` existed,
pacman reported `tree 2.3.2-1`,
and `pacman -Qkk tree` reported no altered files.

Tradeoff:
this is an escape from the inconsistent environment,
not rollback.
It does not help when the original environment is the environment that needs recovery.

### Keep namespace parents unmounted

A source-compatible prototype changed only the parent datasets:

```diff
diff --git a/src/calamares/etc/calamares/modules/zfs.conf b/src/calamares/etc/calamares/modules/zfs.conf
index c09ea0c..88916a2 100644
--- a/src/calamares/etc/calamares/modules/zfs.conf
+++ b/src/calamares/etc/calamares/modules/zfs.conf
@@ -40,13 +40,14 @@ datasets:
     - dsName: data/srv
       mountpoint: /srv
       canMount: on
-    # /var hierarchy (persistent, not part of boot envs)
+    # Namespace-only parents for selected persistent /var datasets.
+    # Keep package-manager state under /var/lib/pacman inside each boot environment.
     - dsName: data/var
-      mountpoint: /var
-      canMount: on
+      mountpoint: none
+      canMount: off
     - dsName: data/var/lib
-      mountpoint: /var/lib
-      canMount: on
+      mountpoint: none
+      canMount: off
     # Podman images and containers
     - dsName: data/var/lib/containers
       mountpoint: /var/lib/containers
```

The private prototype clone is
`/var/home/user/temp/agent/cachyos-zfs-rollback-prototype.d7zQSfAJ`.
Its origin and commit were verified before editing.

A disposable 2 GiB virtual disk exercised both layouts with real ZFS datasets.
The unchanged layout produced:

```text
PRE_FIX
binary=before
database=after
```

The binary probe read from the mounted root clone.
The database probe read from the still-mounted persistent `/var/lib` dataset,
matching the topology that produced the consumer failure.

The patched layout produced:

```text
POST_FIX
binary=before
database=before
persistent_child=postpool/data/var/lib/containers
```

The patched parent layout keeps `/var/lib/pacman` in the root environment.
It preserves explicitly named child datasets such as `/var/lib/containers`.
Both prototype pools were destroyed after the checks,
and the disposable virtual disk was detached but retained.

Tradeoff:
all `/var` state not assigned an explicit child dataset becomes root-environment state.
That improves package and service-state consistency during rollback,
but operators must decide which application state genuinely needs persistence and add explicit child datasets for it.

The full rehearsal must make these boundaries explicit:

- `/var/lib/flatpak` becomes root-environment state.
  System Flatpak deployments then roll back with system packages,
  while user application data under home remains persistent.
- `/var/cache/pacman` becomes root-environment state.
  An older environment may not retain packages downloaded after its snapshot,
  and retained environments may reference distinct cache blocks.
- `/var/lib/bluetooth` becomes root-environment state unless assigned its own dataset.
  Pairing state may therefore follow the selected environment.
- `/var/lib/NetworkManager` becomes root-environment state unless assigned its own dataset.
  Runtime network state may therefore follow the selected environment.
- `/var/lib/systemd` becomes root-environment state unless assigned its own dataset.
  Timer and linger state may therefore follow the selected environment.
- Explicit children for containers,
  Docker,
  libvirt,
  LXC,
  logs,
  spool,
  and temporary data remain persistent under the prototype.

### Full patched consumer validation

A fresh retained UEFI VM applied the 896-byte patch to the pinned installer archive.
The source identities were:

- upstream archive SHA-256:
  `f83565958f5d32054c2a6dbb7bc0295eedc69b21de47d18bcdd1d65ee3d2073c`;
- local patch SHA-256:
  `e9d7271f4f7d2a110b8782049299ee765061d3914b344072d9fa027f2c7341f0`;
- patched `zfs.conf` SHA-256:
  `0a3c855dcd5a3c8c61c9512bd5ed22b1d5898b0245f19d69dcc20671f8dc0c30`.

The no-desktop installation completed every custom ZFS job,
wrote the ZFSBootMenu EFI image,
and booted through native encryption to tty1.
Installed properties showed `mountpoint=none` plus `canmount=off` on both namespace parents.
Running `/` and `/var/lib/pacman` both resolved to `zroot/ROOT/default`.

Installing `tree` created `zroot/ROOT/be-20260829-135720-pre-install`.
The current environment reported `tree 2.3.2-1`,
7 total package files,
0 altered files,
the binary,
and the post-snapshot marker.
ZFSBootMenu listed both environments and booted the pre-install environment to tty1.

The disposable user password had been rotated after the snapshot,
so the rotated password was correctly rejected by the older environment.
The revoked old credential was not reused.
ZFSBootMenu's read-only chroot then showed both `/` and `/var/lib/pacman` on the selected pre-install environment.
`pacman -Q tree` and `pacman -Qkk tree` both reported that `tree` was not installed,
the binary was absent,
and the marker was absent.
This completes the package-file and package-database rollback check without modifying the selected environment.

Returning to `zroot/ROOT/default` accepted the rotated password.
Running `/` plus `/var/lib/pacman` again resolved to `zroot/ROOT/default`,
while home resolved to `zroot/data/home/useruser`.
Pacman reported `tree 2.3.2-1`,
0 altered files,
the binary was present,
and the marker was present.

### Credential state also rolls back

The password rejection proves that `/etc/shadow` follows root boot-environment state.
That is coherent system rollback,
but it can restore a revoked local password hash.
After changing a compromised login credential,
every baseline,
known-good environment,
and transaction clone containing the old hash must be destroyed or deliberately refreshed.
The encrypted-pool passphrase remains a separate control and was not changed in this test.

The retained patched VM later rehearsed deliberate refresh after another disposable credential disclosure.
Copying the trusted environment's hash into offline environments with `usermod --root` produced byte-identical,
nonempty shadow fields across every clone.
That was not sufficient evidence:
a fresh `sudo -k; sudo -v` in default still rejected the credential after reboot.
The test did not isolate whether another account field,
password-aging state,
or a different integration detail caused the rejection.
Raw hash copying is therefore not the accepted procedure.

The verified procedure mounted each non-running root dataset at an empty temporary directory with
`mount --types zfs --options zfsutil`,
then ran `passwd --root <mountpoint> <user>` and entered the final credential twice.
`passwd` reported successful updates for baseline,
default,
and all retained transaction environments.
Known-good accepted the final credential directly.
After default was refreshed through `passwd --root`,
a rebooted graphical terminal accepted it through a fresh `sudo -k; sudo -v`.
Baseline,
known-good,
default,
and all 4 retained pre-install environments then passed the same fresh sudo authentication after reboot or direct
selection;
each running root matched its intended dataset.
Every temporary mount was then absent.
This proves a supported refresh path without reusing either revoked password.

### Clone-based authenticated recovery preserves the corrected boundary

A separate retained recovery domain copied the patched-layout disk before testing emergency reconstruction.
The original patched domain was shut off and its disk was not attached to the recovery domain.
The pinned live ISO imported the copied pool with:

```bash
sudo zpool import -N -R /mnt/recovery -f zroot
sudo zfs load-key -L prompt zroot
```

No dataset mounted during import.
The recovery test cloned
`zroot/ROOT/default@be-20260829-154436-pre-install`
to the new writable root
`zroot/ROOT/usb-recovery-20260829`.
It did not use in-place `zfs rollback`.

The mounted clone kept package files and `/var/lib/pacman` inside the same root environment.
The persistent home dataset remained separate.
`tree` reported 7 files with 0 altered files,
so clone-based emergency recovery did not reintroduce the original package-database split.
The historical clone's local credential was refreshed with `passwd --root` only after `findmnt` proved the mounted root
identity.

The live environment rebuilt the clone's installed initramfs,
copied matching kernel and initramfs artifacts to the ESP,
and wrote a systemd-boot entry containing:

```text
options zfs=zroot/ROOT/usb-recovery-20260829 rw
```

Pool `bootfs` named the same clone.
The accepted recovery initramfs uses `FILES=()`,
omits the irrelevant generic `fsck` hook,
and contains no `/etc/zfs/keys/zroot.key`.
Pool `keylocation=prompt` forced native-encryption authentication before direct boot.

After that prompt,
the clone reached the automatic UWSM plus labwc session.
Final runtime checks found `/` and `/var/lib/pacman` on the recovered clone,
home on `zroot/data/home/useruser`,
matching command-line and `bootfs` values,
active labwc and xwayland-satellite services,
and successful fresh sudo authentication.
The retained ZFSBootMenu image also booted the clone as an independent fallback.

This proves that the corrected package boundary survives the accepted authenticated-media recovery path.
It does not make a root snapshot independently bootable:
the FAT ESP still requires matching reconstructed boot artifacts.

## What does not work

### Treating a successful boot as rollback proof

The pre-transaction clone booted normally.
That did not make its package state coherent.
Boot success tests kernel and root mountability,
not agreement between package files and pacman's database.

### Snapshotting only the root dataset

The existing hook snapshots the current root exactly as implemented.
It cannot capture a database mounted from another dataset.
Changing retention,
description,
or ZFSBootMenu selection properties does not close that boundary.

### Treating baseline or known-good clones as coherent under the current layout

The installer baseline and any manually created root-only known-good clone
share the same persistent `/var/lib` boundary.
They can boot while disagreeing with pacman's database after later package transactions.
Neither counts as a usable system rollback target under the current installed layout.

### Relying on the post-transaction verification

The post-hook verified that the ZFSBootMenu image and root-backed kernels existed.
It did not compare root package files with `/var/lib/pacman` for every retained environment.
The transaction therefore completed successfully despite creating an incoherent rollback target.

### Filesystem repair commands

`zpool status -x` reported a healthy pool.
`zpool clear`,
rollback,
and destructive repair do not address the dataset ownership mismatch and were not used.

### Persisting all of `/var/lib`

Keeping all service state current sounds protective,
but pacman's database is system state coupled to root-backed package files.
Treating it as independent persistent data breaks package rollback semantics.

## Upstream filing decision

No matching exemption exists under `.out-of-scope/`.
Valid open and closed issue and pull-request searches returned empty arrays
for pacman database persistence across boot environments.
Complete tracker listing found closed issues 1 and 3 plus closed pull request 2.
Those concern installation,
a trackpad,
and SATA or SCSI EFI entry handling rather than package rollback.
No duplicate exists in the current tracker.

The filing constraints evaluate as follows:

### 1. Upstream responsibility

Yes.
The installer defines the split dataset layout and root-only pacman hook,
while its README promises that installed-package changes revert with boot environments.

### 2. Upstream fixability

Yes.
The prototype first demonstrated corrected snapshot-inheritance semantics in isolation.
The retained patched VM then completed installation,
pacman-hook,
ZFSBootMenu,
and package-coherence checks.

### 3. Supported use case

Yes.
`README.md:84-89`,
`README.md:98-103`,
and `README.md:357-361` advertise automatic package boot environments and system rollback.

### 4. Contribution acceptance

Yes.
`README.md:583-596` invites fixes of any size.
`CODE_OF_CONDUCT.md` is present.
No issue template,
contribution restriction,
or AI-assistance ban was found.

### 5. Likelihood of an upstream fix

Yes,
with ordinary uncertainty.
The maintainer reproduced and addressed issue 1 after receiving logs.
No documented non-goal or rejection of this use case was found.

### 6. Minimal compatible prototype

Yes.
The retained patch passed a real-ZFS layout fixture with an explicit persistent child dataset.
It also passed the installer,
pacman-hook,
boot,
and read-only rollback inspection paths in a fresh retained VM.

All filing constraints pass.
No external issue was opened because posting to a third-party repository requires user authorization.
The following draft is ready for human review.

### Draft issue

~~~md
Title: Package boot environments leave `/var/lib/pacman` ahead of rolled-back root

Labels: bug

The installer documents that installed packages live under `zroot/ROOT`.
It says they revert when a previous boot environment is selected.
The current dataset layout instead mounts all of `/var/lib` from persistent `zroot/data/var/lib`.
The pacman pre-hook snapshots and clones only the current `zroot/ROOT/<environment>` dataset.
The persistent `/var` design is deliberate,
but pacman's database is coupled system state rather than independent application data.

I reproduced this with installer 0.5.1 at commit `9d587de2d34a35ea33094735002d8599afed7eac` in an encrypted UEFI VM:

1. Install `tree` through pacman. The hook creates a pre-install boot environment.
2. Create a marker under `/etc` after the hook snapshot.
3. Boot the pre-install environment through ZFSBootMenu.
4. Observe that `/usr/bin/tree` and the marker are absent, as expected for the older root.
5. Observe that `pacman -Q tree` still reports `tree 2.3.2-1` because `/var/lib/pacman` stayed current.
6. `pacman -Qkk tree` reports `/usr/bin/tree` and its manual page missing.

Measured mounts:

```text
/         zroot/ROOT/be-20260829-125545-pre-install
/var/lib  zroot/data/var/lib
```

The pool remained healthy. Returning to `zroot/ROOT/default` restored coherent package state and made
`pacman -Qkk tree` report zero altered files.

Root cause:

- `src/calamares/etc/calamares/modules/zfs.conf` mounts `data/var` at `/var` and `data/var/lib` at `/var/lib`.
- `src/pacman-zfs/usr/local/bin/pacman-zfs-pre` snapshots and clones only the current dataset under `zroot/ROOT`.
- Pacman's database therefore does not follow the selected root boot environment.

A minimal prototype keeps `data/var` and `data/var/lib` as namespace-only parents:

```diff
-    # /var hierarchy (persistent, not part of boot envs)
+    # Namespace-only parents for selected persistent /var datasets.
+    # Keep package-manager state under /var/lib/pacman inside each boot environment.
     - dsName: data/var
-      mountpoint: /var
-      canMount: on
+      mountpoint: none
+      canMount: off
     - dsName: data/var/lib
-      mountpoint: /var/lib
-      canMount: on
+      mountpoint: none
+      canMount: off
```

A real-ZFS layout fixture produced `binary=before, database=after` before the patch and
`binary=before, database=before` after it.
Explicit children such as `data/var/lib/containers` still mounted at `/var/lib/containers`.
A fresh patched installation then booted the pre-transaction environment and showed package files plus pacman state at
the pre-transaction version.

The tradeoff is intentional:
unlisted `/var` state becomes boot-environment state.
Persistent application state should be represented by explicit child datasets.
Pacman's database must not persist independently of package files.

This report and prototype were prepared with AI assistance.
The consumer reproduction,
source trace,
before-and-after ZFS fixture,
and retained evidence paths should be reviewed by a human before filing or merging.
~~~
