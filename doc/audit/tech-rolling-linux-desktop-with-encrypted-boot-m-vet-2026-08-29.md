# Rolling Linux desktop with encrypted boot-menu rollback vet report

- Status:
  In progress
- Lifecycle phase:
  Context and rubric frozen;
  discovery pending
- Subject:
  Rolling Linux desktop with encrypted boot-menu rollback
- Scope:
  Choose a rolling Linux installation architecture for this user’s x86-64 KDE gaming desktop
  that provides encrypted root and direct boot-menu selection of rollback targets.
- Started:
  2026-08-29
- Last updated:
  2026-08-29
- Governing skill commit:
  `a05818ad70a40e5769a36de669697ba109891b31`
- Governing skill SHA-256:
  `393eb68c5b2b2f7b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`
- Compatibility fingerprint:
  `636923994432fe5afbe3a13086afb7ffdc004479de4fb2c68842ea6022eff9ce`
- Active audit owner:
  Pi session `01a04220-8e59-7772-ad8a-2c5eb2dedb7b`
- Prior compatible report:
  None found

## Context

The incumbent is Bazzite 44 on an encrypted Btrfs root.
The user is leaving after cross-application stalls coincided with extreme Btrfs transaction and storage pressure.
The initiating mechanism remains unresolved.
A distribution change that retains the same SPCC DRAM-less NVMe cannot by itself exclude device-level tail latency.

Measured deployment:

- AMD Ryzen 7 8700F with 16 logical CPUs;
- 64 GB RAM;
- AMD Radeon RX 7600 using `amdgpu`;
- UEFI firmware with TPM 2.0;
- Secure Boot currently enabled but not required for the replacement;
- Windows shares the 2 TB NVMe and must remain bootable;
- approximately 946 GB is available to replace the current LUKS plus Btrfs Linux root;
- a separate 4 TB SATA SSD currently uses LUKS plus Btrfs;
- KDE Plasma Wayland desktop;
- Steam,
  Lutris,
  and MangoHud gaming workload.

The user delegated rollback-scope selection.
The audit therefore selects system state plus deliberately bounded application configuration,
while keeping documents,
media,
and game saves outside promoted rollback targets.
This avoids whole-home rollback data loss while reducing application-version mismatch.

## Classification

Every candidate is an inspectable open-source local technology.
Applicable overlays are:

- incumbent dependency replacement;
- native kernel,
  bootloader,
  installer,
  and prebuilt-package boundaries;
- sensitive data and encryption;
- multi-platform coexistence with Windows UEFI boot.

Managed-service and SaaS gates are not applicable because no hosted control plane is being selected.
Browser-baseline gates are not applicable because the selected system is a native operating-system stack.

## Hard constraints

- Rolling release.
- Reliable rollback targets selectable directly from a boot menu.
- Encrypted root data at rest.
- AMD Radeon RX 7600 support.
- x86-64 UEFI installation.
- Preserve Windows dual boot.
- Keep personal data outside destructive rollback promotion.
- Inspectable source and source-to-package provenance.

Secure Boot is not a hard constraint.
ZFS and ZFSBootMenu are optional mechanisms rather than requirements.

## Frozen criteria

No relative soft preference remains specified,
so every criterion has weight 1.
Ratings use the governing skill’s 0 through 4 scale.

- Rollback correctness and boot-menu integration,
  weight 1.
- Encryption architecture and recovery,
  weight 1.
- Rolling-update and kernel/filesystem robustness,
  weight 1.
- Gaming and KDE stack currency,
  weight 1.
- Operational maintenance burden,
  weight 1.
- Windows dual-boot and installer fit,
  weight 1.
- Inspectability,
  packaging provenance,
  and recovery documentation,
  weight 1.
- Migration and exit cost,
  weight 1.

Maximum score is 32.
Hard-gate failures remain outside arithmetic.

## Unresolved preferences

None.
If sensitivity analysis makes the ordering depend on an unexpressed preference,
the report will return conditional rankings rather than inventing a tiebreaker.

## Frozen discovery query schedule

### Distribution and ecosystem indexes

- DistroWatch search:
  `rolling release KDE snapshot rollback`.
- ZFSBootMenu official documentation:
  supported distribution guides and boot-environment requirements.
- OpenZFS official documentation:
  supported Linux kernels and root-encryption constraints.
- Official distribution package searches for `zfs`,
  `snapper`,
  `grub-btrfs`,
  `limine`,
  and boot-generation support.

### Repository hosts

GitHub repository search,
sorted by updated date,
with complete 100-result pages until exhaustion or two consecutive pages add no survivor:

- `linux distribution boot snapshot rollback`.
- `zfsbootmenu installer rolling linux`.
- `grub-btrfs distribution installer`.
- `limine snapper boot snapshot`.
- `nixos boot generations encryption`.

### Broader web

- `rolling Linux distribution encrypted root boot snapshots KDE`.
- `CachyOS ZFSBootMenu official`.
- `CachyOS Btrfs snapshot boot official`.
- `openSUSE Tumbleweed encrypted root boot snapshot official`.
- `NixOS unstable encrypted root boot generations official`.
- `Garuda Linux encrypted root boot snapshots official`.
- `Void Linux ZFSBootMenu encrypted root official`.
- `Arch Linux ZFSBootMenu encrypted root rolling kernel compatibility`.
- `rolling Linux snapshot boot menu Windows dual boot`.

### Incumbent and local alternatives

- Current Bazzite deployment and bootable rollback behavior.
- Existing Btrfs,
  Snapper,
  LUKS,
  GRUB,
  and Windows partition boundaries.
- Repository troubleshooting evidence for the incumbent stall.

One de-duplicated expansion round will be added from newly observed taxonomy terms.
Later terms will be recorded without recursively expanding the schedule.
No negative filters are scheduled.

## Candidate ledger

Discovery has not begun.
The user-proposed CachyOS plus ZFSBootMenu architecture is a hypothesis,
not yet a screened candidate or recommendation.

## Evidence records

Pending discovery.

## Execution manifests

No third-party candidate code has been executed.
Read-only official documentation and repository inspection do not cross the external-execution gate.
Any later installer or runtime validation requires a separate bounded manifest.

## Hard-gate outcomes

Pending discovery and screening.

## Validation

Pending.

## Scoring and sensitivity

Pending finalist validation.

## Ranking and recommendation

No recommendation is available before discovery saturation,
hard-gate confirmation,
equal-depth validation,
scoring,
and sensitivity analysis.
