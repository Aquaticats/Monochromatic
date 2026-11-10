# Leave Bazzite for CachyOS on Btrfs with the labwc session

## Purpose

Record the 2026-09-14 grilling that reopened the desktop migration after
[the ZFS decision](../decision/cachyos-zfs-desktop.md) was superseded.
The session configuration lives in <https://github.com/Aquaticat/labwc-config>.

## Why the migration is still happening

- The Bazzite desktop stalls;
  the evidence in [the stall investigation](../troubleshooting/bazzite-desktop-input-stalls.md)
  associates the stalls with Btrfs qgroup accounting during Snapper cleanup
  and keeps the DRAM-less SPCC NVMe as a co-suspect.
- The Bazzite desktop now triggers out-of-memory kills without an apparent cause.
  That incident is undiagnosed and separate from the stalls until evidence links them.
  The user chose to skip capturing its journal before the wipe.

## Decisions

- **Filesystem**:
  Btrfs first.
  ZFS only when Btrfs is proven to be the problem.
- **ZFS trigger**:
  install on the same NVMe with Btrfs qgroups disabled
  and run the NVMe latency histogram from the stall investigation's retained diagnostic plan.
  Stalls stop:
  qgroups were the cause and Btrfs stays.
  Stalls continue with normal device latency:
  Btrfs is the cause and the ZFS runbook becomes the plan.
  Stalls continue with slow device completions:
  replace the SSD,
  because ZFS cannot hide that.
- **Distribution**:
  CachyOS.
  openSUSE Tumbleweed only when CachyOS is proven insufficient,
  meaning any of:
  the Secure Boot chain proves unstable;
  unexplained OOM kills persist and trace to a CachyOS default rather than an application;
  stalls persist after the ZFS trigger test clears both Btrfs and the SSD.
  A labwc stack breakage alone does not count.
- **Secure Boot**:
  on for the physical desktop,
  with LUKS unlocked by TPM2 plus PIN.
- **Rollback scope**:
  everything on the main NVMe must be rollback-able.
  Data that does not need that guarantee lives on the 4 TB data SSD,
  which gets no Snapper configuration.
- **Coupled boot**:
  the Limine menu must boot any coupled root plus home pair.
  If CachyOS cannot,
  openSUSE Tumbleweed is considered first.
  The user proposed snapshotting only `/`:
  `@home`,
  `@root`,
  and `@srv` fold into `@`,
  so every Snapper snapshot of `/` is a coupled pair and every limine-snapper-sync entry boots one.
  `@cache`,
  `@log`,
  and `@tmp` stay separate and unsnapshotted,
  so rollback keeps the logs that explain it and snapshots exclude regenerable data;
  the user delegated this split.
  In the existing VM,
  a snapshot boot mounts the read-only snapshot under a throwaway overlayfs,
  and `limine-snapper-restore` makes it permanent
  (`C:\Users\user\Documents\hyperv-cachyos\README.md`,
  section "Booting a snapshot").
- **Timeline snapshots**:
  hourly,
  bounded retention,
  cleanup at night with boot catch-up,
  qgroups off,
  and every retained snapshot kept bootable in Limine.
- **Secure Boot chain**:
  sbctl custom keys enrolled with Microsoft certificates,
  keeping the RX 7600 option ROM valid.
  The CachyOS Secure Boot guide says to skip `--firmware-builtin` on ASUS and Gigabyte boards.
  This chain cannot be rehearsed in Hyper-V,
  which has no setup mode.
- **Installer**:
  a TypeScript port of the VM's `install.sh`,
  run from the CachyOS live ISO.
- **Rehearsal**:
  a fresh Hyper-V VM built by that installer;
  the current CachyOS VM stays until the new one passes.
- **Package delivery**:
  a signed personal pacman repository built by CI.
- **Memory policy**:
  start from CachyOS defaults as researched on 2026-09-14
  (no systemd-oomd,
  zram sized to RAM,
  `vm.swappiness` raised to 150 once zram is active)
  and measure before changing them.
- **Helper language**:
  TypeScript,
  per `SCR`.
  The runtime is not fixed to Node:
  the user asked for Bun,
  QuickJS,
  and other runtimes to be evaluated for each workload.
  Correction:
  `wlr-pager` and `panel-menu` were first called waybar-era and unported.
  The live `sfwbar.config` runs `wlr-pager watch` for the 3×3 pager and `panel-menu` for the empty-bar menu,
  so both are ported with the other helpers.
- **Configuration home**:
  labwc-config,
  retargeted from the Bazzite rehearsal to CachyOS.
- **Rehearsal material**:
  the Bazzite and KVM harness moves to an `archive/` directory in labwc-config.
- **Lint and TSDoc tooling for labwc-config**:
  consumed from the user's pnpr instance at <https://pnpr.c.aquati.cat/>,
  a stopgap until the packages are hardened for npmjs.com.
  Downloads need no authentication.
  labwc-config scopes `@monochromatic-dev` to it in `.npmrc`
  and exempts that scope from Deno's one-day minimum dependency age,
  because the user republishes fixes the same day.
  On 2026-09-15 all eight packages labwc-config imports resolved and passed its tests from an empty Deno cache.
  `module-logger` fails under QuickJS-ng (Aquaticat/Monochromatic#526),
  so QuickJS helpers log without it.
- **Deployment**:
  a PKGBUILD package installs helpers,
  system drop-ins,
  default configuration,
  and dependencies.
- **Runtimes**:
  QuickJS-ng for hot paths,
  Deno for every other helper and for the installer.
  A hot path runs from user input to the required surface becoming interactive and must stay under 20 ms.
  The live ISO installs Deno with pacman instead of running a compiled installer.
  Measurements and the user's choice are in labwc-config `doc/decision/typescript-runtime.md`.
- **Runtime packaging for session helpers**:
  from a pacman package so it rolls back with the root snapshot.
  The user's original answer named `nodejs`,
  then reopened the runtime choice itself.

## Settled without asking

These were already validated or are determined by `AGENTS.md`:

- UWSM plus labwc,
  sfwbar,
  foot,
  xwayland-satellite at `DISPLAY=:12`,
  display-manager-free tty1 start,
  the black theme,
  and `uwsm app -t service` launches stay as validated on 2026-08-29.
- labwc-config prose is rewritten to the documentation standards.
- The claim in labwc-config's migration plan that UWSM cgroup isolation fixes the stutter is corrected;
  no host evidence supports it.
- The `MemoryHigh=80%` slice limit from that plan is not carried forward without evidence,
  because it manufactures reclaim pressure.

## Open questions

None at the design level.
The grilling frontier closed on 2026-09-14 with:

- **Installer home**:
  labwc-config.
- **Repository hosting**:
  GitHub Releases on labwc-config.
- **Signing key**:
  an offline primary GPG key trusted by pacman,
  with a revocable signing-only subkey in GitHub Actions secrets.

## Later decisions

- **Launcher**:
  a resident Rust and Slint daemon in labwc-config replaces fuzzel,
  bundling Inter,
  gray-white on black,
  without icons.
  The daemon reads the Meta tap from evdev,
  so the user joins the `input` group,
  and a small Rust client talks to it over a Unix socket.
  Matches rank by prefix,
  then word start,
  then anywhere in the name;
  an empty query lists every entry alphabetically.
  Esc,
  the same trigger,
  a click outside,
  or losing focus dismisses it.
- **Pager**:
  `labwc-pager`,
  a Rust binary,
  replaces `wlr-pager`.

## Progress as of 2026-09-15

Everything below is in labwc-config `main`;
the commit that closes each step is named.

- The Bazzite harness is archived,
  the docs follow the standards,
  and the helpers,
  launcher,
  pager,
  and installer are ported.
- `packaging/PKGBUILD` builds `labwc-config`,
  `labwc-config-helpers`,
  and `labwc-config-launcher`.
  CI builds them in an Arch container;
  publishing to the `repo` release waits for the signing subkey secret.
  `packaging/assemble-repo.ts` produced a pacman-verified repository with a subkey-only keyring in the dev VM.
- A real uwsm session in the dev VM ran every desktop entry and session unit from the packages
  (labwc-config `doc/planning/launcher.md`,
  section "Verified in a real uwsm session on 2026-09-15").
- The installer built the `CachyOS-Rehearsal` Hyper-V guest with Secure Boot enforcing through shim and MOK,
  LUKS unlocked by TPM2 plus PIN after first-boot enrollment,
  hourly Snapper snapshots listed in Limine,
  and a snapshot boot whose home matched the snapshot.
  The rehearsal found nine defects;
  labwc-config `doc/troubleshooting/installer-rehearsal.md` records each cause and fix
  (commits through `fb0f930`).
  Two findings matter beyond the VM:
  limine-snapper-sync's system call filter kills Deno boot hooks,
  and flatpak's user environment generator drops XDG prefixes set in `environment.d`.
- Four of those fixes were first applied by hand;
  a clean reinstall from installer `fb0f930` and packages `r70` then passed every check with no hand edits.
- The physical installer refuses firmware that is not in setup mode
  or that sbctl flags with a quirk such as FQ0001,
  before erasing anything.
  sbctl matches quirks by board model,
  so after the firmware mitigation the machine description acknowledges the quirk ID.
  sbctl 0.18 flags every MSI AMD 600-series or newer board;
  the desktop's board model is not recorded yet.

## Changes left on the dev VM

The `CachyOS` Hyper-V VM served as the package build host and installer host:
its `/etc/pacman.conf` has a `[labwc-config]` `file://` section,
the labwc-config packages are installed,
tty1 logs in automatically,
and `/usr/lib/environment.d/50-labwc-config.conf` is a hand-placed file no package owns.
Snapper snapshot 25,
"before labwc-config packages",
is the rollback point.

## Next action

1.  The user creates the offline primary key and signing subkey
    (labwc-config `doc/planning/pacman-repository.md`,
    section on the key ceremony),
    then sets the `PACMAN_SIGNING_SUBKEY` secret and `PACMAN_SIGNING_KEY_ID` variable.
2.  The user clears the desktop's Secure Boot keys in the firmware menu,
    keeping Secure Boot enabled,
    boots the live ISO,
    and follows labwc-config `doc/runbook/install-desktop.md`.
3.  Measure the launcher and panel hot paths on the desktop against the 20 ms budget.
4.  Run the ZFS trigger test on the desktop.
