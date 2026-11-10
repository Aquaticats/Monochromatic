# Rolling Linux desktop with encrypted boot-menu rollback vet report

- Status:
  Blocked on consumer-boundary validation;
  intended adoption authorized and gated by the migration runbook
- Lifecycle phase:
  Hard-gate screening and source validation complete;
  consumer-boundary validation required before formal lifecycle advancement
- Subject:
  Rolling Linux desktop with encrypted boot-menu rollback
- Scope:
  Choose a rolling Linux installation architecture for this user’s x86-64 UWSM plus labwc gaming desktop,
  using the dedicated 2 TB NVMe,
  encrypted root,
  and rollback targets selectable directly from the firmware-launched boot menu.
- Started:
  2026-08-29
- Last updated:
  2026-08-29
- Governing skill commit:
  `a05818ad70a40e5769a36de669697ba109891b31`
- Governing skill SHA-256:
  `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`
- Compatibility fingerprint:
  `911c473022b23ef4a3fb839b75f9a348613ecd9cccf70e0dd2e25ff613dc08b0`
- Active audit owner:
  None;
  last active owner was Pi session `01a04220-8e59-7772-ad8a-2c5eb2dedb7b`
- Prior compatible report:
  None found.
  `doc/audit/tech-rolling-linux-desktop-with-encrypted-boot-m-vet-2026-08-29.md` has the incompatible
  fingerprint `636923994432fe5afbe3a13086afb7ffdc004479de4fb2c68842ea6022eff9ce` because it requires Windows
  coexistence and evaluates a KDE Plasma deployment rather than the rehearsed UWSM plus labwc session.

## Adoption authorization

On 2026-08-29,
the user selected the personal CachyOS ZFS plus ZFSBootMenu recommendation for intended adoption and requested an
installation runbook.
This authority is recorded in
[`doc/decision/cachyos-zfs-desktop.md`](../decision/cachyos-zfs-desktop.md).
The controlling procedure is
[`doc/runbook/migrate-bazzite-to-cachyos-zfsbootmenu.md`](../runbook/migrate-bazzite-to-cachyos-zfsbootmenu.md).

The decision does not retroactively satisfy this report's missing equal-depth consumer validation.
The runbook therefore makes a disposable UEFI encrypted-install,
rollback,
promotion,
and UWSM plus labwc test mandatory before the physical NVMe is erased.
No candidate advances to the formal `Validated`,
`Scored`,
`Recommended`,
or `Adopted` state from authorization alone.

## Context

The incumbent is Bazzite 44 on a LUKS-backed Btrfs root.
The user is leaving after cross-application stalls coincided with extreme Btrfs transaction and storage pressure.
The initiating mechanism remains unresolved,
and a distribution change that keeps the same NVMe cannot by itself exclude device-level tail latency.
The evidence and limits are recorded in
[`doc/troubleshooting/bazzite-desktop-input-stalls.md`](../troubleshooting/bazzite-desktop-input-stalls.md).

Measured deployment:

- AMD Ryzen 7 8700F with 16 logical CPUs;
- 64 GB RAM;
- AMD Radeon RX 7600 using the upstream `amdgpu` driver;
- x86-64 UEFI firmware with TPM 2.0;
- a 2 TB NVMe now dedicated to Linux;
- a separate 4 TB LUKS plus Btrfs SATA SSD whose contents must be preserved;
- Secure Boot may be disabled;
- UWSM plus labwc,
  with sfwbar,
  xwayland-satellite,
  Flatpak,
  a display-manager-free tty1 autologin,
  and per-application systemd units.

The session design and successful VM rehearsal are documented in
`/var/home/user/labwc-vm-test/HANDOVER.md` and `/var/home/user/labwc-vm-test/YOUR-SETUP.md`.
Those documents are local evidence rather than independent upstream sources.
Package availability and behavior still require candidate-specific confirmation.

The user delegated rollback scope and installation effort.
This audit therefore prefers rollback of the operating system plus deliberately selected application configuration,
while documents,
media,
game saves,
and the separate data disk remain current.
At least one prior working system must be directly selectable at boot.
Deeper history is scored rather than treated as a hard gate.

## Classification

Every candidate component is an inspectable open-source local technology.
Applicable overlays are:

- incumbent dependency replacement;
- native kernel,
  bootloader,
  installer,
  and prebuilt-package boundaries;
- sensitive data and encryption;
- high-trust boot and recovery execution.

Managed-service and SaaS gates are not applicable because no hosted control plane is being selected.
Browser-baseline gates are not applicable because the selected system is a native operating-system stack.
Windows coexistence is not applicable because the NVMe is now dedicated to Linux.

## Hard constraints

- Rolling release.
- At least one usable rollback target selectable directly from the boot menu
  without first repairing the installed system.
- A rollback target must pair the root state with bootable matching kernel and initramfs or UKI artifacts.
- Root data encrypted at rest.
- AMD Radeon RX 7600 support through an actively maintained kernel and Mesa stack.
- x86-64 UEFI installation.
- Personal files and the separate encrypted data SSD must remain outside rollback promotion.
- The UWSM plus labwc session must have an inspectable,
  maintainable installation path.
- Inspectable source and source-to-package or source-to-image provenance.

Secure Boot is not a hard constraint.
ZFS and ZFSBootMenu are optional mechanisms rather than requirements.
An unencrypted FAT boot partition does not fail root encryption,
but it reduces boot-artifact confidentiality and integrity when Secure Boot is disabled and is scored accordingly.

## Frozen criteria

No relative priority among the remaining soft concerns was specified,
so every criterion has weight 1.
Ratings use the governing skill’s 0 through 4 scale.

- Rollback correctness,
  boot-menu integration,
  and selectable history depth,
  weight 1.
- Encryption architecture,
  boot-chain integrity,
  and recovery,
  weight 1.
- Rolling-update and kernel/filesystem robustness,
  weight 1.
- UWSM plus labwc stack portability,
  weight 1.
- Gaming stack currency and AMD support,
  weight 1.
- Storage-pressure exposure and background-maintenance control,
  weight 1.
- Installer and distribution integration,
  weight 1.
- Operational maintenance burden,
  weight 1.
- Inspectability,
  packaging provenance,
  and recovery documentation,
  weight 1.
- Migration,
  application-configuration rollback,
  and exit cost,
  weight 1.

Maximum score is 40.
Hard-gate failures remain outside arithmetic.
Criteria and weights are frozen before candidate-specific ratings.

## Unresolved preferences

None at this stage.
If sensitivity analysis makes the ordering depend on an unexpressed preference,
the report will return conditional rankings rather than inventing a tiebreaker.

## Discovery

### Frozen query schedule

#### Distribution and ecosystem indexes

- DistroWatch search:
  `rolling release KDE snapshot rollback`.
- ZFSBootMenu official documentation:
  supported distribution guides and boot-environment requirements.
- OpenZFS official documentation:
  supported Linux kernels and native-encryption constraints.
- Official distribution package searches for `zfs`,
  `snapper`,
  `grub-btrfs`,
  `limine`,
  `labwc`,
  `uwsm`,
  `sfwbar`,
  and `xwayland-satellite`.

#### Repository hosts

GitHub repository searches were sorted by updated date and collected in complete 100-result pages until exhaustion or
until two consecutive pages added no screening survivor:

- `linux distribution boot snapshot rollback`;
- `zfsbootmenu installer rolling linux`;
- `grub-btrfs distribution installer`;
- `limine snapper boot snapshot`;
- `nixos boot generations encryption`.

Raw GitHub responses are preserved under `/var/home/user/temp/agent/rolling-linux-discovery`.
The broad GitHub queries returned substantial unrelated and generated-repository noise;
they are useful for discovery saturation,
not candidate quality.

#### Broader web

- `rolling Linux distribution encrypted root boot snapshots KDE`;
- `CachyOS ZFSBootMenu official`;
- `CachyOS Btrfs snapshot boot official`;
- `openSUSE Tumbleweed encrypted root boot snapshot official`;
- `NixOS unstable encrypted root boot generations official`;
- `Garuda Linux encrypted root boot snapshots official`;
- `Void Linux ZFSBootMenu encrypted root official`;
- `Arch Linux ZFSBootMenu encrypted root rolling kernel compatibility`;
- `rolling Linux snapshot boot menu Windows dual boot`.

#### Incumbent and local alternatives

- Current Bazzite deployment and bootable rollback behavior;
- existing Btrfs,
  Snapper,
  LUKS,
  bootloader,
  and data-disk boundaries;
- the rehearsed UWSM plus labwc session;
- repository troubleshooting evidence for the incumbent stalls.

### Expansion round

New taxonomy terms produced one de-duplicated expansion round:

- `rolling Linux blue green boot rollback`;
- `immutable rolling Linux A B boot menu rollback`;
- `Arch immutable dual root rollback`;
- `rolling Linux Limine Snapper boot history`;
- `rolling Linux root on ZFS installer ZFSBootMenu`;
- `transactional rolling desktop boot snapshot rollback`.

The expansion schedule is frozen.
Later taxonomy terms are recorded without recursively adding queries.
No negative filters were used.

### Discovery result

Discovery is saturated with multiple screening survivors.
The hard-gate-confirmed architectures are:

- CachyOS with encrypted Btrfs plus Limine;
- openSUSE Tumbleweed with encrypted Btrfs plus Snapper and its current BLS boot stack;
- Garuda with encrypted Btrfs plus Snapper and GRUB;
- Shanios with encrypted Btrfs blue/green deployments;
- CachyOS with encrypted ZFS plus ZFSBootMenu through the third-party `cachyos-zfs-installer`.

Siduction,
openSUSE Kalpa,
KDE Linux,
AerynOS,
ObsidianOS,
and Nebula exit during targeted screening for the candidate-specific reasons recorded in the ledger.

## Candidate ledger

### CachyOS with encrypted Btrfs plus Limine

- Discovery sources:
  CachyOS official documentation,
  current installer source,
  package sources,
  and `limine-snapper-sync` upstream source.
- Base category:
  Inspectable open-source local technology.
- Overlays:
  Replacement,
  native/prebuilt,
  sensitive-data,
  and high-trust boot overlays.
- Screening result:
  Hard-gate confirmed;
  finalist validation pending.
- Why it survives:
  The installer creates a LUKS2 root behind an unencrypted FAT32 `/boot`;
  Limine presents Snapper snapshots directly;
  current helper source preserves and restores content-addressed matching kernel artifacts.

### openSUSE Tumbleweed with encrypted Btrfs plus Snapper and BLS

- Discovery sources:
  openSUSE installer,
  full-disk-encryption,
  Snapper,
  BLS,
  `sdbootutil`,
  and package documentation.
- Base category:
  Inspectable open-source local technology.
- Overlays:
  Replacement,
  native/prebuilt,
  sensitive-data,
  and high-trust boot overlays.
- Screening result:
  Hard-gate confirmed;
  finalist validation pending.
- Why it survives:
  Current YaST installs support LUKS2 FDE with systemd-boot or GRUB2-BLS;
  `sdbootutil` creates a direct boot entry for every retained bootable snapshot and associates it with matching,
  content-addressed kernel and initramfs artifacts on FAT32 boot storage.

### Garuda with encrypted Btrfs plus Snapper and GRUB

- Discovery sources:
  Garuda release announcement,
  installer generator,
  package sources,
  and snapshot-support sources.
- Base category:
  Inspectable open-source local technology.
- Overlays:
  Replacement,
  native/prebuilt,
  sensitive-data,
  and high-trust boot overlays.
- Screening result:
  Hard-gate confirmed;
  finalist validation pending.
- Why it survives:
  Garuda Temeraire uses LUKS2 plus Argon2 with GRUB 2.14,
  retains `/boot` inside the Btrfs root snapshot,
  generates GRUB snapshot entries,
  and installs a dracut overlay for read-only snapshot boots.

### Shanios blue/green Arch deployments

- Discovery sources:
  official project site,
  installer source,
  deployment source,
  image builder,
  and package sources.
- Base category:
  Inspectable open-source local technology.
- Overlays:
  Replacement,
  native/prebuilt,
  sensitive-data,
  and high-trust boot overlays.
- Screening result:
  Hard-gate confirmed for encryption and rollback;
  finalist validation and session portability pending.
- Why it survives:
  Its installer creates a LUKS root with Argon2id,
  two read-only Btrfs roots,
  and one UKI plus direct systemd-boot entry per slot.
  One prior slot remains selectable;
  deeper history exists only as non-boot-menu backup snapshots.

### CachyOS with encrypted ZFS root plus ZFSBootMenu

- Discovery sources:
  ZFSBootMenu and OpenZFS official documentation,
  CachyOS package sources,
  and `fnichol/cachyos-zfs-installer` 0.5.1.
- Base category:
  Inspectable open-source local technology assembled from distribution and upstream components.
- Overlays:
  Replacement,
  native/prebuilt,
  sensitive-data,
  and high-trust boot overlays.
- Screening result:
  Hard-gate confirmed at the source-architecture level;
  third-party installer runtime validation pending.
- Why it survives:
  ZFSBootMenu unlocks natively encrypted datasets,
  discovers matching kernel and initramfs pairs inside each boot environment,
  and directly clones,
  promotes,
  or rolls back snapshots from its boot UI.
  CachyOS ships version-locked ZFS modules for its kernels,
  but the installation and pacman integration are maintained by one third-party author rather than CachyOS.

### siduction with encrypted Btrfs plus Snapper and GRUB

- Discovery sources:
  siduction installation and Btrfs manuals plus `siduction-btrfs` source.
- Screening result:
  Hard-gate failure.
- Reason:
  siduction directly boots Snapper snapshots and deliberately leaves qgroups disabled,
  but its encrypted-install procedure requires an unencrypted separate `/boot`.
  Its own Btrfs manual warns that a separate `/boot` is excluded from root snapshots and can make rollback inconsistent.
  That fails the matching-kernel-artifact requirement.
- Primary sources:
  <https://manual.siduction.org/hd-install_en.html>
  and <https://manual.siduction.org/sys-admin-btrfs-snapper_en.html>,
  accessed 2026-08-29.

### openSUSE Kalpa

- Discovery sources:
  official Kalpa site and openSUSE community documentation.
- Screening result:
  Category-fit hard-gate failure for this deployment.
- Reason:
  Kalpa remains Alpha,
  says custom partitioning produces a broken unsupported system,
  and reserves host RPM mutation for strictly required host functionality.
  Replacing its supported Plasma session with the complete UWSM plus labwc host stack would create an unsupported
  custom base rather than a ready-to-use candidate.
- Primary sources:
  <https://kalpadesktop.org/documentation/>
  and <https://en.opensuse.org/Portal:Kalpa>,
  accessed 2026-08-29.

### NixOS unstable

- Discovery sources:
  official NixOS generation and encryption documentation.
- Screening result:
  Excluded by explicit user constraint concerning project community and governance.
  Technical capabilities are not scored.

### AerynOS

- Discovery sources:
  official installation,
  filesystem,
  rollback,
  and lacking-features documentation.
- Screening result:
  Encryption hard-gate failure.
- Reason:
  AerynOS offers boot-time state rollback,
  but its official missing-features page states that disk encryption is not yet supported.
- Primary source:
  <https://aerynos.dev/faq/lacking-features/>,
  accessed 2026-08-29.

### ObsidianOS

- Discovery sources:
  official GitHub organization,
  installer source,
  and `obsidianctl` source.
- Screening result:
  Encryption hard-gate failure.
- Reason:
  `modules/install.py:264-297` creates two FAT ESPs and five plain ext4 or F2FS partitions with `mkfs`;
  the installer has no LUKS or other root-encryption operation.
- Clone:
  `/var/home/user/temp/agent/obsidianos-obsidianctl-20260829` at
  `d5e62067a610e94bc4f2ff9eef9f7e2e45bf6e8a`.
- Primary source:
  <https://github.com/Obsidian-OS/obsidianctl>,
  accessed 2026-08-29.

### Nebula Linux

- Discovery sources:
  official site and source repository.
- Screening result:
  Kernel-artifact hard-gate failure.
- Reason:
  `profile/airootfs/etc/calamares/modules/partition.conf:16-28` deliberately places `/boot` on a separate ext4
  partition outside the snapshotted root.
  A root rollback can therefore pair old `/usr/lib/modules` with a current kernel and initramfs.
  The project also labels the release Beta while its site still warns that it is Alpha.
- Clone:
  `/var/home/user/temp/agent/nebula-linux-20260829` at
  `68eca22c54d30a96beb625b1bab0beac17ee6b45`.
- Primary source:
  <https://github.com/nebula-linux-os/Nebula-Linux>,
  accessed 2026-08-29.

### KDE Linux

- Discovery sources:
  official installation,
  rollback,
  and recovery documentation plus source repository.
- Screening result:
  Category-fit hard-gate failure for this deployment.
- Reason:
  KDE Linux has direct boot selection of prior OS generations and an encrypted-install path,
  but the only current edition is explicitly Testing and the supported host image is Plasma-specific.
  Supplying UWSM,
  labwc,
  sfwbar,
  and xwayland-satellite would require a custom system extension or image;
  ready-to-use surviving distributions make that custom implementation ineligible.
- Primary sources:
  <https://linux.kde.org/docs/install/>
  and <https://linux.kde.org/docs/boot-failure-recovery/>,
  accessed 2026-08-29.

## Evidence records

### CachyOS boot-history behavior

- Candidate revision:
  `limine-snapper-sync` 1.31.0 source at commit
  `26caede1286b4f6bc85321f9ffba9fbabed21711`.
- Claim and relevance:
  `SnapshotManager.historyAddSnapshot()` archives kernel,
  initramfs,
  and UKI files under content-addressed names.
  `SnapshotManager.restoreKernels()` checks the target artifacts,
  removes non-target boot files,
  and copies the archived target set back.
  A kernel update therefore does not inherently make a snapshot unusable.
- Gate:
  Rollback correctness and kernel-artifact hard gate.
- Status:
  Pass in inspected helper source;
  end-to-end installer validation remains pending.
- Source paths:
  `src/main/java/org/limine/snapper/processes/SnapshotManager.java:211-254`
  and `:648-697`.
- Primary source:
  <https://gitlab.com/Zesko/limine-snapper-sync>,
  accessed 2026-08-29.
- Counterevidence:
  CachyOS documentation still states that snapshots involving kernel updates cannot be rolled back at
  <https://wiki.cachyos.org/configuration/btrfs_snapshots/>.
  That statement conflicts with current helper source and the helper author’s explanation.
- Remaining limits:
  Snapshots predating the helper,
  missing or pruned history,
  corrupt artifacts,
  and inadequate FAT boot-partition capacity can still prevent restoration.

### CachyOS automatic Limine partition sizing

- Candidate revision:
  CachyOS Calamares commit `f1c20a500e14448e36991f1b9d3ebae153827761`.
- Claim and relevance:
  Current automatic whole-disk partitioning creates a 4,096 MiB FAT32 partition mounted at `/boot` when Limine is
  selected.
- Gate:
  Installer integration and restore capacity.
- Status:
  Pass in current source;
  ISO UI validation remains pending.
- Source paths and excerpts:
  `src/modules/partition/partition.conf:75-80` sets `efiSystemPartition: "/boot"` and
  `efiSystemPartitionSize: 4096M`;
  `src/libcalamares/partition/PartitionSize.cpp:30` maps `M` to `MiB`;
  `src/modules/partition/core/PartitionActions.cpp:175-198` creates a FAT32 ESP using that configured size.
- Primary source:
  <https://github.com/CachyOS/cachyos-calamares>,
  accessed 2026-08-29.
- Documentation discrepancy:
  The 26.01 changelog says 4,192 MB at
  <https://wiki.cachyos.org/cachyos_basic/changelogs/gui_installer/>.
  Current source is decisive for the present proposal and does not contain that number.
- Historical counterevidence:
  ISO 260124 could alternate between 2 GB and 4 GB after navigating backward at
  <https://discuss.cachyos.org/t/inconsistent-calamares-behavior-when-selecting-limine-260124/22236>.
  The current source has one Limine override,
  but the current UI path still needs runtime validation.

### CachyOS snapshot-pressure defaults

- Candidate revision:
  `CachyOS/CachyOS-PKGBUILDS` commit
  `74d4d55e0ddda0daa8823908c4d1ceccaa2ab158`.
- Claim and relevance:
  The shipped root Snapper template sets `QGROUP=""` and `TIMELINE_CREATE="no"`.
  Package operations still create pre/post snapshots,
  and number cleanup remains enabled.
- Gate:
  Storage-pressure exposure and background-maintenance control.
- Status:
  Pass for avoiding the incumbent qgroup configuration;
  ordinary Btrfs transaction pressure remains possible.
- Source path:
  `cachyos-snapper-support/snapper-template-root-cachyos`.
- Primary source:
  <https://github.com/CachyOS/CachyOS-PKGBUILDS/blob/master/cachyos-snapper-support/snapper-template-root-cachyos>,
  accessed 2026-08-29.

### openSUSE BLS snapshot-to-kernel association

- Candidate revision:
  `openSUSE/sdbootutil` commit `786f9a84027ac6aa351eb303206c34123bb23059`.
- Claim and relevance:
  The Snapper plugin runs `sdbootutil add-all-kernels` for each Tumbleweed snapshot.
  The tool reads kernels from the selected snapshot,
  generates or reuses its initramfs,
  stores content-addressed artifacts on FAT32 boot storage,
  and writes a BLS entry with that snapshot’s `rootflags`.
- Gate:
  Direct rollback and matching-kernel-artifact hard gates.
- Status:
  Pass in current source;
  end-to-end installer validation remains pending.
- Source paths:
  `10-sdbootutil.snapper:107-126`,
  `sdbootutil:1734-1957`,
  and `ARCHITECTURE.md` under “Introducing snapshots”.
- Primary sources:
  <https://github.com/openSUSE/sdbootutil>
  and <https://news.opensuse.org/2025/11/13/tw-grub2-bls>,
  accessed 2026-08-29.
- Capacity behavior:
  `sdbootutil:1437-1602` protects active and default snapshots,
  then prunes other boot entries from oldest to newest until a new kernel fits.
  A retained Snapper snapshot can therefore lose its boot entry before Snapper deletes the snapshot.
  The July 2026 XBOOTLDR guidance at <https://news.opensuse.org/2026/07/07/xbootldr/> provides a larger separate FAT32
  boot store when the default 1 GB ESP is insufficient.

### openSUSE encryption and Snapper defaults

- Claim and relevance:
  Current BLS installs support LUKS2 with password,
  TPM2,
  or FIDO2 unlock.
  The root Snapper configuration shown by openSUSE uses `QGROUP=1/0`,
  while timeline creation is disabled on ordinary roots larger than 16 GiB;
  package-manager pre/post snapshots and cleanup remain enabled.
- Gate:
  Encryption and storage-pressure scoring.
- Status:
  Pass for encryption;
  scored concern for reintroducing qgroup accounting on the same NVMe.
- Primary sources:
  <https://news.opensuse.org/2025/11/13/tw-grub2-bls>
  and <https://doc.opensuse.org/documentation/tumbleweed/snapper/>,
  accessed 2026-08-29.
- Evidence limit:
  Upstream Snapper’s `data/default-config` leaves `QGROUP` empty;
  the `1/0` value is openSUSE’s installed policy rather than an invariant of Snapper itself.

### Garuda encrypted snapshot boot path

- Candidate revisions:
  `garuda-linux/pkgbuilds` commit `18ebb7a6fe801e1b2163e2ba6e1581799ec9e644`,
  `garuda-linux/tools/iso-profiles` commit `760f824c1c8b1971112eeb6268762190eaa8e07c`,
  and `garuda-linux/tools/garuda-tools` commit `5c45ad31dda34e409d44e2959af4501dd01a54d1`.
- Claim and relevance:
  The ISO uses GRUB;
  `/boot/efi` is separate but `/boot` remains in the snapshotted root;
  `snapper-support` generates GRUB entries and uses no qgroup;
  `garuda-dracut-support` overlays a writable tmpfs over a read-only snapshot boot.
- Gate:
  Encryption,
  direct rollback,
  matching-kernel-artifact,
  and storage-pressure gates.
- Status:
  Pass in source;
  end-to-end encrypted install validation remains pending.
- Source paths:
  `garuda-tools/lib/util-yaml.sh:278-326`,
  `iso-profiles/shared/Packages-Root`,
  `pkgbuilds/snapper-support/snapper-template-garuda`,
  and `pkgbuilds/garuda-dracut-support/snapshot-overlay.sh:1-22`.
- Primary source:
  <https://forum.garudalinux.org/t/garuda-linux-temeraire-260819/48606>,
  accessed 2026-08-29.
- Version boundary:
  Temeraire introduced LUKS2 plus Argon2 and ships GRUB 2.14,
  whose release added Argon2 KDF support.
  Reports about earlier LUKS1 installs do not describe the current ISO.

### Shanios blue/green behavior

- Candidate revisions:
  `shani8dev/os-installer-config` commit `a76296a84183e0efdd6a2dedb3cded1d72da6693`
  and `shani8dev/shani-deploy` commit `5ea6945fd1bfd7387290ca808371d89a9612582b`.
- Claim and relevance:
  The installer encrypts the root with LUKS plus Argon2id,
  creates `@blue` and `@green`,
  builds one signed UKI per slot,
  and writes direct systemd-boot entries.
  Deployment writes the new image only to the inactive slot and uses a boot counter for automatic fallback.
- Gate:
  Encryption and direct rollback hard gates.
- Status:
  Pass in source;
  end-to-end installer validation remains pending.
- Source paths:
  `scripts/install.sh:289-304`,
  `scripts/configure.sh` functions `generate_uki_entry()` and `generate_loader_conf()`,
  and `shani-deploy/README.md` under “The blue-green model”.
- Primary sources:
  <https://github.com/shani8dev/os-installer-config>
  and <https://github.com/shani8dev/shani-deploy>,
  accessed 2026-08-29.
- Material limits:
  only one previous slot is directly bootable;
  `/etc` and selected `/var` state are shared overlays and do not roll back with a slot;
  continuous `beesd` deduplication is enabled;
  all deployment and installer commits measured through the GitHub API come from one maintainer.

### ZFSBootMenu and CachyOS ZFS assembly

- Candidate revisions:
  ZFSBootMenu commit `e15503228f40b3c95ded551fab86e91f3e3d230f`,
  `cachyos-zfs-installer` 0.5.1 at `9d587de2d34a35ea33094735002d8599afed7eac`,
  and CachyOS package source `74d4d55e0ddda0daa8823908c4d1ceccaa2ab158`.
- Claim and relevance:
  ZFSBootMenu prompts for a native-encryption passphrase,
  finds kernel and initramfs pairs inside each boot environment,
  and exposes snapshot clone,
  clone-and-promote,
  and rollback actions in its boot UI.
  The third-party installer separates persistent home and application data from `zroot/ROOT/*`,
  creates a baseline environment,
  and keeps 24 pacman-created environments by default.
  Its `create-baseline-boot-env.sh` sets `org.zfsbootmenu:active=off`,
  while ZFSBootMenu documents that value as hiding a `mountpoint=/` environment.
  The adoption runbook explicitly sets the baseline to `on` and creates a post-migration `known-good` environment;
  neither remediation has received consumer-boundary validation yet.
- Gate:
  Encryption,
  direct rollback,
  matching-kernel-artifact,
  and personal-data separation hard gates.
- Status:
  Pass in architecture and source;
  complete installer execution remains pending.
- Primary sources:
  <https://docs.zfsbootmenu.org/en/latest/general/native-encryption.html>,
  <https://docs.zfsbootmenu.org/en/latest/general/bootenvs-and-you.html>,
  <https://docs.zfsbootmenu.org/en/latest/online/snapshot-management.html>,
  and <https://github.com/fnichol/cachyos-zfs-installer>,
  accessed 2026-08-29.
- Kernel pairing:
  CachyOS packages a `linux-cachyos-zfs` module with an exact dependency such as `linux-cachyos=7.1.8-1` and
  `Provides: ZFS-MODULE` at <https://packages.cachyos.org/package/cachyos/x86_64/linux-cachyos-zfs>.
  This avoids an unbounded DKMS/kernel mismatch for supported CachyOS kernels,
  but OpenZFS remains an out-of-tree module and real-time kernels remain unsupported.
- Maintenance limit:
  the installer is 97.1 percent single-author by GitHub contribution count and is not an official CachyOS component.

### Source tests and maintenance signals

- `limine-snapper-sync` 1.31.0 has no test source tree,
  and `build.gradle.kts` explicitly sets `tasks.test.enabled = false`.
- `openSUSE/sdbootutil` has no repository test tree,
  but its latest ten inspected pull requests included nine merged fixes and the project is packaged through openSUSE.
- Garuda’s pinned `pkgbuilds`,
  `iso-profiles`,
  and `garuda-tools` revisions all had successful GitLab pipelines.
  The 260819 Mokka manifest at
  <https://iso.builds.garudalinux.org/iso/garuda/mokka/260819/garuda-mokka-linux-garuda-260819.pkgs.txt>
  confirms `grub 2:2.14-1`,
  `garuda-dracut-support 1.6.0-2`,
  Snapper 0.13.1,
  and the Chaotic-AUR keyring and mirror list.
- Shanios package builds passed repeatedly through 2026-08-29,
  and an image build passed at
  <https://github.com/shani8dev/shani-builder/actions/runs/32534186289>.
  Later image and stable-promotion workflows include failures,
  including <https://github.com/shani8dev/shani-builder/actions/runs/33230049052>.
- The third-party CachyOS ZFS installer has no integration-test files;
  its Makefile’s `test` and `check` targets both reduce to shell checks.
  ZFSBootMenu itself has a 38-file test area,
  and current upstream build and script-analysis workflows passed at commit
  `e15503228f40b3c95ded551fab86e91f3e3d230f`.

### High-trust integration surface

The measured non-test integration surfaces,
using Tokei on the cited clones,
are:

- CachyOS Limine helper:
  5,333 code lines across 43 files;
- openSUSE `sdbootutil` core plus hooks:
  4,109 code lines in one main script plus hook files;
- Garuda installer and snapshot integration subset:
  658 code lines across 9 files;
- Shanios deployment and installer scripts:
  13,806 code lines across 11 files;
- third-party CachyOS ZFS installer excluding vendored code:
  941 code lines across 12 files.

These counts measure audit surface,
not quality.
They omit upstream bootloaders,
filesystems,
and general distribution installers shared by the candidates.

## UWSM plus labwc portability

### Common session mechanism

UWSM itself supplies the required labwc plugin,
tty1 shell-profile startup checks,
`app-graphical.slice`,
`background-graphical.slice`,
`session-graphical.slice`,
and `uwsm app` scope or service launch modes.
The upstream documentation explicitly supports `uwsm check may-start` plus `exec uwsm start` from a login-shell profile
and places ordinary applications in separate systemd user units at
<https://github.com/Vladimir-csp/uwsm>.

The rehearsed display-manager-free boot therefore transfers without a distribution-specific display manager:

- disable the installed display manager;
- add a `getty@tty1` autologin drop-in;
- guard `exec uwsm start -- labwc` in the login profile;
- keep `xwayland-satellite.service` ordered after the graphical session;
- launch applications with `uwsm app`,
  using service units when clean activation-environment inheritance is required.

Flatpak is available on every finalist.
The portal routing,
GTK configuration,
sfwbar files,
labwc XML,
and user services live in the user’s home and are independent of the root filesystem choice.
The VM-only SPICE agent override does not transfer to physical hardware.

### CachyOS Btrfs,
Garuda,
and CachyOS ZFS

All three use the Arch package base for the session:

- `labwc` 0.20.2 is in Arch Extra at <https://archlinux.org/packages/extra/x86_64/labwc/>;
- `uwsm` 0.26.7 is in Arch Extra at <https://archlinux.org/packages/extra/any/uwsm/>;
- `xwayland-satellite` 0.8.2 is in Arch Extra at
  <https://archlinux.org/packages/extra/x86_64/xwayland-satellite/>;
- sfwbar 1.0 beta17 is available through the AUR,
  so it requires an inspected AUR build rather than an Arch-signed repository package.
  The AUR recipe at commit `46996951521a2b1d721382fa6db7164f25cbcd98` pins the upstream tag archive with
  SHA-256 `a4915bc7dd0873c45d0d6b01b070e39a91fd16cfadf730d6a9e48db68a8cd09e`.

The complete session can be installed natively and uses the same systemd user behavior on all three architectures.
The ZFS variant changes storage and boot recovery,
not desktop packaging.
CachyOS and Garuda package versions have advanced beyond the VM’s labwc 0.9.6,
UWSM 0.26.1,
sfwbar beta16,
and xwayland-satellite 0.8.1 baselines;
configuration compatibility must therefore be exercised in a finalist VM rather than inferred.

### openSUSE Tumbleweed

Tumbleweed directly packages labwc,
UWSM,
and xwayland-satellite in the distribution.
The package records are available at:

- <https://software.opensuse.org/package/labwc>;
- <https://software.opensuse.org/package/uwsm>;
- <https://software.opensuse.org/package/xwayland-satellite>.

Sfwbar is not in the main Tumbleweed repository,
but the openSUSE `X11:Wayland` development project publishes sfwbar 1.0 beta17 for Tumbleweed.
The source RPM dated 2026-08-11 has SHA-256
`2e3681b52f543ce3c8804438dc62768734d42e322b314a1f73b665a5fc01d60e`.
The source repository directory is
<https://download.opensuse.org/repositories/X11:/Wayland/openSUSE_Tumbleweed/src/>.
This adds one non-default openSUSE repository but avoids a locally authored package.

The tty1,
UWSM,
user-systemd,
and Flatpak design transfers directly.
Tumbleweed’s package naming and paths require a fresh generated host procedure;
the Bazzite `rpm-ostree` commands must not be copied.

### Shanios

The official Shanios image does not contain labwc,
UWSM,
sfwbar,
or xwayland-satellite.
Its read-only root also blocks persistent `pacman` installation.
The desktop image includes `shani-core`,
whose package source depends on Nix,
and the shared fstab mounts a persistent `@nix` subvolume.
Nixpkgs currently packages all four components,
including sfwbar beta17 and xwayland-satellite 0.8.2 at:

- <https://github.com/NixOS/nixpkgs/blob/nixpkgs-unstable/pkgs/by-name/la/labwc/package.nix>;
- <https://github.com/NixOS/nixpkgs/blob/nixpkgs-unstable/pkgs/by-name/uw/uwsm/package.nix>;
- <https://github.com/NixOS/nixpkgs/blob/nixpkgs-unstable/pkgs/by-name/sf/sfwbar/package.nix>;
- <https://github.com/NixOS/nixpkgs/blob/nixpkgs-unstable/pkgs/by-name/xw/xwayland-satellite/package.nix>.

A Nix profile can therefore provide the session binaries,
and the home plus `/etc` overlay can preserve the tty1 and user-unit configuration.
This path has material drawbacks:

- the primary host session depends on Nixpkgs even though NixOS was excluded for governance concerns;
- Shanios does not build or test this session in its images;
- the user must maintain absolute Nix store paths or profile indirection in systemd integration;
- the shared `/etc` overlay and home do not roll back with an OS slot;
- writable selected application configuration cannot naturally follow the blue/green slot.

The alternative is maintaining a custom Shanios image profile,
which is a larger ongoing ownership burden than installing packages on the mutable finalists.

### Selected application configuration

The Btrfs and ZFS finalists all exclude home from system rollback by default,
which correctly keeps documents,
media,
and saves current.
For the mutable-root finalists,
a bounded writable directory inside the rolled-back root or boot-environment dataset can hold only selected
configuration trees;
symlinks from home can point to those trees.
This makes those selected files follow the system snapshot while the rest of home remains current.
The design must be tested for applications that write configuration during a read-only snapshot trial.

Shanios cannot provide the same coupling without new per-slot writable state because both home and its writable `/etc`
overlay are shared across slots.
That is a scored rollback-scope disadvantage rather than an encryption or direct-boot failure.

## Execution manifests

### Read-only source syntax validation

- Candidates and revisions:
  the pinned revisions listed in the evidence records.
- Commands:
  `bash --noprofile --norc -n` over relevant shell sources and Python `ast.parse()` over relevant Python sources.
- Reachable commands:
  Bash and Python parse only;
  candidate code is not sourced or executed.
- Expected reads:
  cloned source files under `/var/home/user/temp/agent`.
- Expected writes and network:
  none.
- Environment:
  incumbent Fedora 44 x86-64 host;
  no credentials passed;
  at most two concurrent parser processes.
- Success condition:
  every parsed file exits zero.
- Failure condition:
  first syntax error is recorded against the candidate;
  no retry with relaxed parsing.
- Evidence limit:
  syntax validation does not validate installer behavior,
  bootability,
  or recovery correctness.

### Syntax-validation result

The first sweep incorrectly omitted `xargs --max-args=1`,
which meant Bash parsed only the first file in each batch and treated later names as positional arguments.
That result is discarded.
The corrected command parsed each file in a separate Bash invocation.
It passed for 9 CachyOS Limine shell files,
3 openSUSE BLS files,
20 Garuda shell files,
11 Shanios shell files,
and 9 third-party ZFS-installer shell files.
Python `ast.parse()` also accepted 16 inspected installer Python files.
The command printed `PER_FILE_SOURCE_SYNTAX_VALIDATION_COMPLETE` and exited zero.
Elapsed time was not captured.

### Deferred consumer-boundary validation

The user explicitly declined downloading every finalist ISO in this session.
No fresh encrypted install,
firmware-to-boot-menu run,
snapshot selection,
or rollback promotion has therefore been executed for any finalist.
This prevents the audit from reaching the governing skill’s `Validated`,
`Scored`,
or `Recommended` lifecycle states.
Source and official integration evidence can order follow-up validation priority,
but cannot be presented as a completed adoption recommendation.

## Hard-gate outcomes

- CachyOS Btrfs plus Limine:
  pass;
  finalist.
- openSUSE Tumbleweed Btrfs plus Snapper and BLS:
  pass;
  finalist.
- Garuda Btrfs plus Snapper and GRUB:
  pass;
  finalist.
- Shanios blue/green:
  pass for rolling,
  encryption,
  and direct prior-slot boot;
  finalist subject to session validation.
- CachyOS ZFS plus ZFSBootMenu:
  pass at source level;
  finalist subject to third-party installer validation.
- siduction:
  fail because encrypted installs separate `/boot` from root snapshots.
- openSUSE Kalpa:
  fail deployment fit because the required host-session replacement is outside its supported customization model.
- AerynOS:
  fail because disk encryption is not supported.
- ObsidianOS:
  fail because the installer creates unencrypted roots.
- Nebula Linux:
  fail because the separate `/boot` can mismatch root snapshots.
- KDE Linux:
  fail deployment fit because a non-Plasma host session requires a custom image extension while only the Testing
  edition is available.
- NixOS:
  excluded by explicit user constraint.

## Validation

No finalist reaches `Validated` because none received a fresh encrypted installation,
direct boot-menu selection,
and rollback-promotion run in a disposable VM.
No disk was repartitioned and no ISO was downloaded.

Source-level results differ materially:

- CachyOS Btrfs plus Limine has the deepest inspected recovery path,
  but `limine-snapper-sync` disables Gradle tests and has no test source tree.
- Garuda’s 260819 package manifest confirms `grub 2:2.14-1`,
  and current package/profile pipelines passed at the pinned revisions;
  no focused snapshot-restore test exists in the inspected integration sources.
- openSUSE’s BLS design has distribution-owned source,
  active maintenance,
  and package integration,
  but no local `sdbootutil` test suite was present and no exact encrypted Tumbleweed snapshot-boot openQA result was
  established.
- ZFSBootMenu’s current build and script-analysis workflows passed,
  but the third-party CachyOS installer’s `test` target is shell checking and contains no integration tests.
- Shanios has a 1,014-line update and rollback harness that was not run because it requires built images.
  Its image workflow passed on 2026-08-21,
  while later image and stable-promotion runs failed;
  package-build runs continued to pass.

The observed CI outcomes are project evidence,
not substitutes for this audit’s missing consumer-boundary run.

## Scoring and sensitivity

Formal scoring is not performed.
The governing workflow permits formal scores only after equal-depth consumer validation.
At the user’s explicit request,
this section instead records a non-governing source-only estimate so the available evidence still produces a decision.
The estimate uses the frozen 10 criteria at equal weight,
the 0 through 4 rating scale,
and a maximum of 40 points.
Ratings measure source evidence and architecture,
not observed installation or runtime behavior.

### Informal source-only score: openSUSE Tumbleweed BLS

- Rollback correctness: 3.
  `sdbootutil` constructs snapshot-specific BLS entries and matching content-addressed artifacts,
  but boot-storage pruning can remove an entry while its Snapper snapshot remains
  (<https://github.com/openSUSE/sdbootutil>).
- Encryption:
   2.5.
  LUKS2 unlock is distribution-supported,
  while BLS boot artifacts remain on FAT storage
  (<https://news.opensuse.org/2025/11/13/tw-grub2-bls>).
- Rolling and kernel robustness:
   3.5.
  The boot stack is distribution-owned and actively maintained,
  but the exact encrypted snapshot path has not been exercised here.
- UWSM plus labwc portability:
   3.5.
  Tumbleweed directly packages labwc,
  UWSM,
  and xwayland-satellite;
  sfwbar comes from `X11:Wayland`
  (<https://software.opensuse.org/package/labwc>).
- Gaming and AMD currency: 3.
  Tumbleweed is rolling,
  but no RX 7600 or game workload was run.
- Storage-pressure control: 2.
  Installed policy uses `QGROUP=1/0` and package snapshots with cleanup,
  although this audit did not establish qgroups as the incumbent stall cause
  (<https://doc.opensuse.org/documentation/tumbleweed/snapper/>).
- Installer and distribution integration: 4.
  Encryption,
  Snapper,
  BLS synchronization,
  and recovery are distribution-owned.
- Operational burden:
   3.5.
  Distribution tooling owns the lifecycle,
  but the BLS path and boot-store sizing still require operator attention.
- Inspectability and provenance:
   3.5.
  Source and maintenance are inspectable,
  but the inspected `sdbootutil` tree has no local end-to-end test suite.
- Migration and exit cost:
   2.5.
  Mutable Btrfs can host selected rollback-coupled configuration,
  but the actual host procedure is untested.
- Estimate:
  31/40,
  or 77.5 percent.

### Informal source-only score: Garuda Btrfs plus GRUB

- Ratings in frozen-criterion order:
  2.5,
  3.5,
  2.5,
  3,
  3.5,
  3.5,
  3,
  2.5,
  2.5,
  and 2.5.
- Estimate:
  29/40,
  or 72.5 percent.
- Evidence basis:
  kernels remain inside the encrypted snapshotted root;
  current GRUB 2.14 supports LUKS2 plus Argon2;
  qgroups are disabled;
  and current pipelines passed.
  The encrypted snapshot boot was not exercised,
  focused restore tests are absent,
  and the base system includes Chaotic-AUR trust
  (<https://forum.garudalinux.org/t/garuda-linux-temeraire-260819/48606>).

### Informal source-only score: CachyOS Btrfs plus Limine

- Ratings in frozen-criterion order:
  3,
  2.5,
  2.5,
  3,
  3.5,
  3.5,
  3,
  2.5,
  2.5,
  and 2.5.
- Estimate:
  28.5/40,
  or 71.3 percent.
- Evidence basis:
  current source pairs snapshots with archived kernel artifacts,
  reserves 4,096 MiB of FAT boot storage,
  and disables qgroups and timeline snapshots by default.
  Boot artifacts are unencrypted,
  helper tests are disabled,
  documentation retains a stale kernel warning,
  and the current ISO path was not run
  (<https://gitlab.com/Zesko/limine-snapper-sync> and <https://github.com/CachyOS/cachyos-calamares>).

### Informal source-only score: CachyOS ZFS plus ZFSBootMenu

- Ratings in frozen-criterion order:
  3,
  3,
  2,
  3,
  3,
  3.5,
  1.5,
  2,
  2.5,
  and 2.
- Estimate:
  25.5/40,
  or 63.8 percent.
- Evidence basis:
  ZFSBootMenu has the strongest native boot-environment UI in the finalist set,
  but rollback remains rated 3 because the complete CachyOS path depends on an unexecuted unofficial integration.
  OpenZFS’s ARC can eliminate physical reads on cache hits;
  its ZIO scheduler prioritizes synchronous I/O over async writes and background scans;
  and its finite dirty-data window plus transaction delay can smooth short write bursts.
  Those mechanisms raise storage-pressure control from 3 to 3.5,
  but they do not hide cache misses,
  synchronous durability waits,
  sustained writes,
  or an already-issued slow NVMe command.
  The source trace is recorded in the
  [OpenZFS latency investigation](../troubleshooting/openzfs-single-device-latency-masking.md)
  and in current OpenZFS documentation at
  <https://openzfs.github.io/openzfs-docs/Performance%20and%20Tuning/ZIO%20Scheduler.html>.
  The installer is single-author by measured contribution count and has no installation tests,
  and OpenZFS remains an out-of-tree kernel dependency
  (<https://docs.zfsbootmenu.org/en/latest/online/snapshot-management.html> and
  <https://github.com/fnichol/cachyos-zfs-installer>).

#### Operational-burden correction

The original 1.5 rating mixed routine operator work with third-party integration risk.
Routine work is more automated than that rating implied:
pacman hooks create boot environments,
retain clone datasets,
verify image placement,
and CachyOS ships an exact-version ZFS module package.
ZFSBootMenu also provides a recovery shell and keeps one backup EFI image.
Operational burden is therefore raised to 2.

Source inspection still found operator-owned integration gaps:

- installer files are copied into `/usr/local` without a package or update channel;
- every package transaction creates both a snapshot and clone,
  while retention deletes only clone filesystems and does not name their origin snapshots for deletion;
- `generate-zbm` is conditional,
  and CachyOS packages its sample regeneration hook under `/usr/share/doc` with targets that do not match
  `linux-cachyos-zfs` or cover `zfsbootmenu` itself;
- exact-version modules avoid DKMS but make kernel updates depend on a matching module package;
- ZED is preset-enabled,
  while the shipped weekly and monthly scrub timers require the operator to choose and enable one;
- pool-feature activation must remain compatible with the ZFSBootMenu and rescue images.

The source trace and unmeasured-runtime limits are recorded in the
[CachyOS ZFS operational-ownership investigation](../troubleshooting/cachyos-zfs-installer-operational-ownership.md).

#### Latency-masking sensitivity outside the frozen rubric

The frozen rubric did not isolate foreground latency smoothing from qgroup exposure,
snapshot maintenance,
and general filesystem robustness.
An illustrative additional criterion rates the three mutable Btrfs finalists at 2.5,
ZFS at 4,
and Shanios at 2 for source-level latency-masking architecture.
The Btrfs value is a labeled placeholder:
Linux page-cache behavior was confirmed,
but the current Btrfs transaction and writeback paths did not receive an equal source trace or workload benchmark.
The added criterion excludes qgroup and snapshot-policy exposure
because the existing storage criterion already counts it.
For the sensitivity arithmetic,
ZFS’s existing storage rating reverts from 3.5 to 3 so ARC,
ZIO scheduling,
and dirty-data control are counted only once.
At equal weight 1,
the overall order remains unchanged.
At maximum weight 5,
ZFS becomes first with 45/60 points,
ahead of Tumbleweed at 43.5/60.
That scenario assigns one unvalidated criterion one third of total weight,
so it demonstrates dominance of that preference rather than a robust measured outcome.
This conditional result is not a benchmark result or a formal rubric revision.

### Informal source-only score: Shanios blue/green

- Ratings in frozen-criterion order:
  2,
  2.5,
  2,
  1.5,
  2.5,
  1.5,
  2,
  1.5,
  2,
  and 1.
- Estimate:
  18.5/40,
  or 46.3 percent.
- Evidence basis:
  LUKS2,
  per-slot UKIs,
  and boot fallback are structurally attractive,
  but only one previous slot is selectable;
  writable configuration is shared;
  continuous bees deduplication is enabled;
  the labwc path depends on Nixpkgs;
  and the large installer and deployment surface has one maintainer
  (<https://github.com/shani8dev/os-installer-config> and <https://github.com/shani8dev/shani-deploy>).

### Informal sensitivity

The source-only order is not preference-stable:

- weighting storage-pressure control at 3 instead of 1 makes Garuda first;
- weighting encryption at 3 produces a Garuda and Tumbleweed tie;
- weighting gaming currency at 5 produces the same tie;
- weighting installer integration,
  operational ownership,
  robustness,
  or inspectability more heavily preserves or widens Tumbleweed’s lead.

No single full-point rating change alters the source-only winner.
The Garuda and CachyOS Btrfs adjacency is less stable:
a single adverse half-point change creates a tie,
and a full-point change can reverse them.
Tumbleweed’s score also counts distribution ownership under integration,
robustness,
and operations.
Reducing its robustness and operations ratings by half a point each to deduplicate that evidence produces 30/40,
which remains first but only one point ahead of Garuda.
The scores remain low-confidence because the omitted runtime checks affect several criteria together.
Desktop packaging also overlaps labwc and gaming.
This estimate therefore must not be read as formal sensitivity analysis.

## Source-evidence validation priority

This is an order for future runtime validation,
not a completed recommendation or adoption decision.

### 1. CachyOS with Btrfs plus Limine

#### Pros

qgroups and timeline snapshots are disabled by default;
Arch directly packages the main labwc stack;
current source pairs snapshots with content-addressed boot artifacts;
the installer automatically reserves 4 GiB for boot history.

#### Cons

boot artifacts remain on unencrypted FAT;
the helper’s own test task is disabled;
current documentation still carries a disproven kernel-rollback warning;
restore depends on preserved boot history and adequate FAT capacity.

### 2. Garuda with Btrfs plus GRUB

#### Pros

qgroups and timeline snapshots are disabled;
kernels stay inside the encrypted snapshotted root;
GRUB 2.14 supports the installer’s LUKS2 plus Argon2 format;
the read-only snapshot overlay is compact and inspectable.

#### Cons

GRUB performs the early encrypted-root read;
snapshot integration has no focused test suite;
Garuda includes Chaotic-AUR trust and packaging in the base system;
sfwbar still comes from the AUR.

### 3. openSUSE Tumbleweed with Btrfs plus BLS

#### Pros

the installer,
FDE,
Snapper,
boot-entry synchronization,
and recovery tooling are distribution-owned;
`sdbootutil` explicitly constructs matching kernel and initramfs artifacts;
labwc 0.9.6 matches the rehearsed VM baseline.

#### Cons

the default root policy enables qgroups;
the current BLS stack is newer than openSUSE’s traditional GRUB integration;
boot-storage pressure can prune an entry while its Snapper snapshot remains;
sfwbar requires the non-default `X11:Wayland` repository.

### 4. CachyOS with ZFS plus ZFSBootMenu

#### Pros

ZFSBootMenu has the strongest native boot-environment UI in the finalist set;
kernel and root state live together;
native encryption is supported;
CachyOS provides version-locked precompiled ZFS modules.

#### Cons

the installer is unofficial and 97.1 percent single-author by contribution count;
OpenZFS remains an out-of-tree kernel dependency;
the integration has no installation test;
the quick-start path executes downloaded root-level code unless replaced with a pinned local checkout.

### 5. Shanios blue/green

#### Pros

LUKS2 plus Argon2id,
signed per-slot UKIs,
one directly bootable previous slot,
and automatic failed-boot fallback are structurally simple;
the Nix store can persist the required labwc packages outside the immutable roots.

#### Cons

the deployment and installer surface is 13,806 measured code lines from one maintainer;
recent image-pipeline failures are unresolved in this audit;
continuous bees deduplication adds background filesystem work;
selected writable configuration and `/etc` do not roll back with a slot;
the primary session would depend on Nixpkgs despite the user’s NixOS governance objection.

Priority order:
CachyOS Btrfs plus Limine > Garuda Btrfs plus GRUB > openSUSE Tumbleweed BLS > CachyOS ZFS plus
ZFSBootMenu > Shanios.

CachyOS precedes Garuda because its exact snapshot-to-boot-history path received deeper source tracing and its installer
reserves dedicated history capacity,
while Garuda retains an earlier encrypted GRUB stage and less focused recovery validation.
Garuda precedes Tumbleweed because it avoids qgroups and keeps kernel artifacts inside the encrypted snapshot with a
smaller custom integration surface;
openSUSE’s broader institutional testing does not yet prove the exact deferred runtime path.
Tumbleweed precedes the ZFS assembly because the whole boot and recovery chain is distribution-owned,
whereas the ZFS integration is a single-author third-party layer over an out-of-tree module.
The ZFS assembly precedes Shanios because ZFSBootMenu and OpenZFS provide staffed,
well-documented primitives beneath a smaller integration layer,
while Shanios concentrates a larger whole-system mechanism in one maintainer and does not natively ship the required
session.

## Ranking and recommendation

### Informal source-only ranking

openSUSE Tumbleweed BLS > Garuda Btrfs plus GRUB > CachyOS Btrfs plus Limine > CachyOS ZFS plus ZFSBootMenu >
Shanios.

Tumbleweed precedes Garuda because its installer,
encryption,
boot synchronization,
and recovery chain are distribution-owned and more operationally documented;
Garuda’s lower default snapshot-pressure surface does not establish that it avoids the unresolved incumbent stall.
Garuda precedes CachyOS Btrfs because it keeps kernels inside the encrypted snapshotted root,
while both have native Arch-family session packages and disabled qgroups.
CachyOS Btrfs precedes the ZFS assembly because its official installer owns the storage path,
whereas the ZFS installer is third-party and adds an out-of-tree kernel boundary.
The ZFS assembly precedes Shanios because ZFSBootMenu has broader upstream maintenance and deeper boot-environment
functionality,
while Shanios concentrates a larger custom system in one maintainer and does not natively ship the required session.

### Informal adoption recommendation

Based only on the available source and package evidence,
adopt **openSUSE Tumbleweed with encrypted Btrfs,
Snapper,
and the current BLS boot stack**.
Confidence is low.
It best matches the user’s preference for distribution-integrated recovery without adding a third-party root installer,
and it has the strongest combined evidence for maintained boot-artifact synchronization,
operational ownership,
and native labwc stack packaging.

Material limits remain part of the recommendation:

- no encrypted installation,
  firmware boot,
  snapshot boot,
  promotion,
  or labwc migration was exercised;
- Tumbleweed’s qgroup and cleanup defaults retain a source-level exposure similar to part of the incumbent stack,
  although qgroups were not proved causal;
- boot-storage pruning can leave a Snapper snapshot without a direct BLS entry;
- retaining Btrfs and the same NVMe means migration does not prove or guarantee removal of the original stall mechanism;
- the 4 TB encrypted data SSD still requires a verified backup and recovery plan before NVMe repartitioning.

If avoiding qgroups and default snapshot-maintenance work is treated as more important than distribution ownership,
Garuda becomes the source-only preference.
If ARC,
foreground-aware I/O scheduling,
and finite write-burst absorption are the dominant preference,
the added-criterion sensitivity makes CachyOS ZFS plus ZFSBootMenu first despite its installer and kernel-integration
costs.
The separate source-evidence validation priority still starts with CachyOS Btrfs because that ordering answers a
different question:
which untested path has the deepest candidate-specific source trace and should be exercised first.

This user-requested recommendation does not advance the governing lifecycle to `Validated`,
`Scored`,
`Recommended`,
or `Adopted`.
It authorizes no installation or decision record.
