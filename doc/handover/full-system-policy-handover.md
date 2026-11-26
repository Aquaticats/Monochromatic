# AppArmor full-system rollout handover

## Status at 2026-11-26T18:25:51-05:00

The owner approved the enforcement result and requested a complete installed-system and repo
inventory.
That inventory is complete.
Current evidence has no additional denial requiring a policy
change,
but this is not a guarantee against future application changes or unexercised operations.
Default enforcement with documented per-profile exceptions is not universal enforcement.

The owner selected the broad `apparmor.d` collection for network-facing and local threats.
The
required development,
terminal,
Podman,
and Incus access to `/home/user` and
`/home/user/Monochromatic` is retained.
This is `fsp no`,
not experimental full-system policy that
blocks every unprofiled executable.
Unprofiled development processes remain a residual risk.

## Current policy controls

- `/etc/apparmor/modes` contains `default enforce`,
  `include default`,
  `reload yes`,
  and `fsp no`.
- Managed policy was promoted with `aa-install --install`,
  without `--fsp`.
  The latest
  `aa-install --status` reports 710 installed profiles and zero drift.
- Custom `/etc/apparmor.d/helium` was separately changed with
  `aa-mode --enforce --no-reload helium`.
- Package `flags.d` exceptions remain intentional.
  Konsole,
  `fuse-overlayfs`,
  `pacman`,
  `auditd`,
  and other documented exceptions are not enforcing.
  Describe the result as default enforce with
  per-profile exceptions.
- Local D-Bus XML keeps `<apparmor mode="disabled"/>`.
  This is separate from real broker process
  attachment to `dbus-system` and `dbus-session`.
- `/etc/subuid` and `/etc/subgid` retain `user:100000:65536` and add `root:1000000:1000000000`.
- `/etc/default/limine` enables AppArmor on the main kernel and sets `apparmor=0` on LTS.
  The main
  kernel remains selected.
  The exact LTS recovery entry is untested;
  editing the main entry to
  remove `apparmor,` from `lsm=` was demonstrated.

## Local workload rules

- `/etc/apparmor.d/local/sanoid` allows owner-matched flat `File::Temp` files with ten
  source-derived characters and `Capture::Tiny` readiness markers with the confined parent PID
  suffix.
  It grants no other temporary-directory tree.
- `/etc/apparmor.d/local/git` allows owner-matched read,
  write,
  and lock operations only beneath
  `/home/user/temp/agent`,
  plus Git 2.56.1's six-character `.git_signing_buffer_tmp` file and its
  `.sig` sibling under `/tmp`.
  It transitions only `ssh-keygen` to its existing enforcing profile
  for commit signing.
  `/etc/apparmor.d/local/ssh-keygen` grants the same buffer and signature paths.
  Only this repository's measured `.git/hooks/pre-push` receives inherit-exec permission;
  third-party hooks and build output receive none.
  Git LFS may read its own `/proc/<pid>/cgroup`
  path without an `owner` qualifier and its owner-matched `mountinfo`.
  Only the exact current
  owner-managed
  `.cache/wrapper-built-by-cargo-from-main-ee0cf2268b-2026-11-22/monochromatic-git-policy-cli`
  binary receives inherit-exec permission;
  a rebuilt wrapper requires a fresh review.
- Custom `/etc/apparmor.d/gh` confines GitHub CLI 2.105.0 with its own config scope,
  session-bus
  keyring access,
  network and TLS access,
  and `git` through the existing profile.
  It closes the
  managed Git profile's previously missing `px` target without an unconfined helper.
- `/etc/apparmor.d/local/apparmor_parser` allows read access to Incus-generated profiles and
  read/write access to its parser cache tree.
- `/etc/apparmor.d/local/aa-install` and `/etc/apparmor.d/local/aa-mode` each allow only the
  observed flat `system.slice/*.scope/cpu.max` read.
  The broader shared `golang-strict.d` override
  was removed.
- `/etc/apparmor.d/local/helium` allows read-only root-owned `/etc/os-release` with audit enabled.
  The custom profile uses the real `net.imput.helium` configuration and cache scopes.
  No
  `/home/user/temp` browser grant was added.
- On 2026-11-25, `/etc/apparmor.d/local/helium` gained read-only access to
  `owner @{tmp}/Spectacle.@{rand6}/{,**}` for dragged screenshots.
  The exact confined PNG read changed from `Permission denied` to the independent SHA-256 match.
  The owner's real drag retry remains open; see
  `doc/troubleshooting/spectacle-screenshot-drag-helium-tempfile.md`.
- Later on 2026-11-25, the owner explicitly approved read-only Helium access to all of
  `~/Plain`, including its KeePassXC databases, for Dolphin file drags.
  `/etc/apparmor.d/local/helium` grants only `owner @{HOME}/Plain/{,**} r`.
  The exact confined Markdown read changed from `Permission denied` to the independent SHA-256
  match, while `.bash_profile` remained denied.
  The owner's real Dolphin-to-Helium drag then succeeded. See
  `doc/troubleshooting/dolphin-file-drag-helium-plain-read.md`.
- `/etc/apparmor.d/abstractions/app/chromium_crashpad_handler.d` audits the pre-existing
  crash-report path and permissions without adding a path or permission.
- `/etc/apparmor.d/local/keepassxc` grants recursive directory-only `owner @{HOME}/**/ r` for the
  picker,
  selected database-file `rwl`,
  KIO state and bookmarks,
  anonymous sockets,
  its own proc
  mount table and thread names,
  and the two observed udev metadata files.
  This exposes directory
  metadata without granting unrelated file contents.
  On 2026-11-25, the selected database rule gained Qt's exact
  `main.kdbx* -> #@{int}` atomic-save relationship and `#@{int}` read/write access.
  The confined create-write-rename-readback fixture passed without modifying the real database.
  Three subsequent real KeePassXC atomic saves then completed successful open, link, and rename
  syscalls with no new denial. See
  `doc/troubleshooting/keepassxc-database-atomic-save-tempfile.md`.
- On 2026-11-24, the same local KeePassXC rule gained `k` on its existing state-file paths after
  Qt 6.12.0's `QLockFile` blocked the GUI during Helium native messaging.
  The exact `KeePassXCrc.lock` fallback received the same narrow permission.
  Thirty-four empty, unheld stale lock artifacts were removed after descriptor checks.
  The GUI returned to `ppoll`, both native socket queues drained, and no `.rmlock` chain returned.
  See `doc/troubleshooting/keepassxc-state-lock-native-messaging-stall.md`.
- `/etc/apparmor.d/local/setpci` grants only `capability sys_admin`,
  which the kernel requires for
  full PCI config-space reads.
  This repairs the Intel NVMe Gen3 workaround without unconfining
  `setpci` or granting `/dev/mem` access.
- `/etc/apparmor.d/local/ssh` grants bind,
  listen,
  accept,
  send,
  and receive on loopback TCP only,
  preserving dynamic SSH reverse-forward ports without non-loopback listeners.
- `/etc/apparmor.d/local/gh` grants inherit-exec only to the exact current owner-managed
  `monochromatic-git-policy-cli` path.
  Helium's local additions include read-only access to the
  existing KeePassXC native-messaging JSON.
  Other local rules cover `man//groff`'s `basename`,
  logind's exact uhid module metadata,
  the portal's own `memory.pressure`,
  and profile-sync-daemon's
  nested Chromium mountpoints.
  Portal pressure writes and fresh mount operations remain unexercised
  until their real services repeat them.
  DMI and `sys_module` restrictions remain unchanged.
- `/etc/apparmor.d/local/checkupdates` allows the five-character `arch-update` runtime database
  directory and its `local` symlink.
  It does not broaden other runtime paths.
  Pacman remains in
  complain mode.
- On 2026-11-23,
  after the cold boot,
  the Git wrapper's session added rules for the wrapper's own
  tests,
  its container gate and its helpers,
  on the owner's "Add rules to unblock whatever you need to
  do as you go this session."
  and "No,
  please do rules for everything instead of a dedicated profile
  that agents would have to remember to use."
  The `git` profile now covers fixtures under the cargo
  build folder and the gate's `/work`,
  their hooks' ordinary tools,
  the wrapper's own process reads
  and its caller's environment,
  and Git LFS's user-manager `cpu.max`.
  Under `gh` the wrapper moves to
  the `git` profile,
  and `gh` reads drafts under `~/temp/agent`.
  `ssh-keygen` reads the signing buffer
  there too.
  A new `local/fuse-overlayfs` names Podman's layer storage,
  which stopped its complain-mode
  flood of the audit log.
  Rules,
  causes and tradeoffs:
  `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
  "The Git wrapper's own
  tests,
  gate and helpers under enforcement".
  The kernel audit backlog stays at 64 and still drops
  records in bursts.
- On 2026-11-26, IVPN Desktop 3.15.15 gained `/etc/apparmor.d/local/openvpn`
  (its configuration, key files and a child profile for its DNS scripts)
  and three custom profiles:
  `ivpn` enforcing,
  `ivpn-ui` as a named `flags=(unconfined)` Electron profile,
  and `ivpn-service`, enforcing since the owner's WireGuard and OpenVPN connect runs
  (its built-in firewall ran under it only in namespace probes).
  `local/ip`, `ps`, `nft`, `resolvectl`, `sysctl` and `sync` allow `network netlink dgram`,
  the socket the IVPN daemon leaks into its children;
  `local/firefox` and a new `local/firefox-crashreporter` cover Firefox ESR in `/opt/firefox-esr`;
  `local/aa-install` reads `cpu.max` under the user manager's `app.slice`.
  On the owner's "Grant tor", `local/torbrowser-tor` runs `lyrebird` in its own enforcing child profile
  (`priority=1` over the stock pluggable-transport rule) with sockets and name lookups,
  and accepts the browser's stop signals;
  `local/torbrowser` may lock its shader cache and create its PulseAudio folder.
  Snowflake connected afterwards (`check.torproject.org` reported `IsTor: true`).
  The same change went upstream as apparmor.d issue #1328 and pull request #1329 (filed 2026-11-26T08:09:10Z).
  Open: once an installed apparmor.d build has the fix
  (test: `rg 'profile pluggable-transports' /etc/apparmor.d/torbrowser-tor` prints a line),
  remove the `lyrebird` child, its `priority=1` rule and the signal rule from `local/torbrowser-tor`,
  reload, and recheck Snowflake.
  Until then nothing breaks: the local `priority=1` rule keeps sending `lyrebird` to the local child.
  No watcher runs: the owner chose GitHub's own notification of the merge as the signal (2026-11-26T08:14:00-05:00).
  `local/apparmor_parser` gained DAC capabilities and read access to `~/temp/agent` for drafts.
  See `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`, "Thirteenth batch".
- Separately on 2026-11-26, an AppArmor test-created `~/.mozilla/firefox` directory made all three
  Firefox channels select fresh legacy profiles after restart.
  Their real XDG profiles under `~/.config/mozilla/firefox` were intact.
  The test/new directory was preserved under a backup name, the original index hashes stayed equal,
  and the owner confirmed all three channels were back.
  This was not an AVC and required no policy or cache change; see
  `doc/troubleshooting/firefox-xdg-profiles-legacy-test-directory.md`.
- Firefox Backup in both channels then failed on `rename_dest` and `unlink` in
  `~/Documents/Restore Firefox`.
  `/etc/apparmor.d/local/firefox` now grants `rwl` only to `FirefoxBackup_*` there,
  with directory metadata access.
  The confined rename-readback-delete probe passed, an unrelated filename remained denied,
  and the owner completed real backups in regular Firefox and ESR.
  See `doc/troubleshooting/firefox-backup-documents-archive-replacement.md`.
- On 2026-11-26, `update-desktop-database` was denied GLib's six-character temporary files in the
  user applications directory.
  `/etc/apparmor.d/local/update-desktop-database` now covers the exact `mimeinfo.cache` and
  `intent.cache` destination/temporary shapes.
  The real update exits 0, both caches were replaced, and `palemoon.desktop` is registered.
  See `doc/troubleshooting/update-desktop-database-user-cache-tempfiles.md`.
- Later on 2026-11-26, `/etc/apparmor.d/local/systemd-tmpfiles` gained recursive `rw` access only
  beneath `/home/user/temp`, the configured tmpfiles cleanup root.
  The real dry-run traversed nested directories with zero permission errors in its collected partial
  output. The owner asked to stop the long run, so complete traversal is not claimed.
  No non-dry cleanup ran. See `doc/troubleshooting/systemd-tmpfiles-home-temp-recursive-clean.md`.

## Current-boot evidence

The current boot is `5b86b6ae-d743-4c00-82eb-7011241cfbbc`,
a real enforcement cold boot.
Earlier
boot `2035a996-2f6f-45e8-a409-7712c793864b` is the pre-reboot tuning baseline.
The latest selected
cache must still receive a complete final verification pass and owner review.

- The early loader previously reported
  `Successfully loaded all binary profiles from AppArmor early policy cache (/etc/apparmor/earlypolicy/48da56f3.0).`
  before `apparmor.service`.
- `verification-pre-cold-boot-local-tuning-runtime.json` verifies current source modes,
  loaded
  kernel modes,
  and zero managed drift.
- `verified-selected-final-measured-cold-boot-tuning-2026-11-23T15-49-00-0500.json` verifies all 809
  cache profiles,
  including custom `gh` and the `setpci` capability,
  source-only compilation,
  live
  feature bytes,
  cache hits,
  successful loader reload,
  and rollback preservation.
  Podman,
  Incus,
  host,
  Konsole,
  and disposable enforced Helium workloads passed on this enforcement cold boot.
- `enforcement-policy-cold-boot.json` proves boot `5b86b6ae-d743-4c00-82eb-7011241cfbbc` is new,
  with the expected LSMs,
  brokers,
  desktop environment,
  process labels,
  and early loader message at
  33.44 seconds.
  The isolated cold-boot positive control also passed with exact AVC and cleanup
  evidence.
- After the picker/state and loopback rules,
  the credential helper,
  real Git push,
  and `man setpci`
  succeeded.
  No new denials appeared after 2026-11-23T20:47:39Z in the immediate direct-log check.
  The full retained-log collector and secondary `aa-log` view completed with no failures or partial
  tails.
- `zfs-nvme-gen3.service` initially failed because `setpci` was denied `sys_admin`;
  the kernel then
  exposed only 64 config bytes.
  After the narrow grant,
  the service reached 8.0 GT/s x4 and
  `Result=success`.
  The installed script now aborts on a failed speed-cap write instead of resetting
  blindly.
  See `doc/troubleshooting/cachyos-setpci-full-config-space-sys-admin.md`.
- KeePassXC PID 4823 opened `/home/user/Plain/` after the directory-only grant,
  with paired
  `SYSCALL success=yes` and a returned directory descriptor.
  Earlier database reads were also
  positively audited.
  There are currently no failed units.
  The real 15:45 Sanoid run exited 0
  without making a duplicate snapshot;
  the existing 14:45 hourly snapshots remain intact.
- `positive-enforcement-control-2035a996-2f6f-45e8-a409-7712c793864b-before-cold-boot-2026-11-23T16-05-41-295Z-103181.json`
  verifies an allowed read,
  an exact denied PID and AVC,
  and removal of the temporary source,
  kernel
  profile,
  and cache artifact.
- Signed commits `02f5e836951eaa990e1c6e3f88e5017b51e39d0b`,
  `778f7c5f9bbc5a0cacbdc7ac480bc31f4f360e3a`,
  `d189b6f6d0355cd41c377d1fdfc4b68aff043668`,
  and
  `2a6969039952edf5564c8f384f03ea9ad4aa7737` reached `main` with the enforcing `ssh-keygen` and `gh`
  profiles plus the exact own-hook rule.
  `gh auth status` read the existing keyring credential
  successfully.
  `git ls-remote` returned those same commit hashes.
- `markdown-rendered-review/final-workload-tuning/browser-evidence-aggregated-with-setpci.json` and
  `browser-evidence-aggregated-with-loopback.json` record successful light and dark rendering,
  keyboard focus,
  an empty local console,
  and all seventeen source citations across the full run and
  targeted retries.
  The loopback servers and disposable documentation-browser processes and profiles
  are removed.
- The real scheduled `sanoid.service` initially failed with `Capture::Tiny` status 2.
  After the
  readiness-marker correction,
  the unchanged tee reproduction returned exact captured stderr,
  and
  the already-scheduled 14:00 EST service exited 0.
  Direct ZFS queries confirmed the home,
  root,
  and
  varlog snapshots from that run.
  `sanoid-narrow-tempfile-verified-2035a996-2f6f-45e8-a409-7712c793864b-2026-11-23T18-59-31.197Z.json`
  separately proves nonempty readback and a retained self-label denial in the same confined process.
- `checkupdates-runtime-database-verification-4LaKN.json` proves the real five-character directory,
  the raw and canonical local database targets,
  nonempty synchronized databases,
  package
  comparisons,
  and fixture removal.
  An incorrect six-character harness path remained denied and was
  not added to policy.
- KeePassXC PID 4210 is enforcing.
  Its explicitly audited database opens are `apparmor="AUDIT"` with
  paired `SYSCALL success=yes`;
  an `ALLOWED` matcher was the wrong acceptance condition for this
  audit rule.
- Rootless and rootful Podman fixtures passed with independent expected-content readback and
  container removal.
  Rootless inner `unconfined` and rootful `containers-default-0.70.0 (enforce)`
  remain distinct.
- Host and real Konsole repository and scratch-file workloads passed.
  Unprofiled agent processes
  remain unconfined;
  Konsole and its descendant are complain.
- `workload-data/enforcement-validation-kernel-mapped-unprivileged-incus-2035a996-2f6f-45e8-a409-7712c793864b-2026-11-23T19-16-57.079Z.json`
  proves the exact outer enforcing profile,
  namespace presence during the workload,
  nonzero kernel
  UID/GID mappings,
  guest readback,
  namespace and outer-profile removal,
  and empty final instance
  inventory.
  Inner guest labels remain separately recorded as `unconfined`.
- `enforced-helium-preboot-after-tuning-audited-boundaries-example.json` proves the native enforcing
  browser,
  renderer seccomp filtering,
  Example Domain rendering,
  empty CDP console and exception
  lists,
  and no unexpected unconfined helper.
  The screenshot was inspected.
  CDP close was
  acknowledged,
  the launcher exited 0,
  and the disposable browser,
  crashpad,
  relay PIDs,
  and
  transient unit exited.
- That browser exercised successful audited OS-release reads,
  existing crash-report reads,
  settings
  read/write opens,
  lock creation,
  and lock removal.
  It did not create a new crash dump.
  Do not
  claim dump creation was exercised.
- Both Go helpers were observed with their exact `cpu.max` descriptor open under enforcing labels.
  Descriptor content matched direct quota reads.

## Audit evidence boundaries

Use `summarize-current-boot-audits.ts`,
which reads all retained uncompressed audit rotations,
separates `DENIED`,
`ALLOWED`,
`AUDIT`,
and other modes,
handles duplicate and unterminated records,
and runs `aa-log` over captured bytes.
A zero-record result requires a positive recognition control.

Installed `ausearch` reports `Malformed event skipped, rc=9`;
do not use it as the oracle.
Plain
`deny` suppressed the targeted positive-control record,
so the control uses `audit deny`.
Explicitly
audited allowed operations can appear as `AUDIT`,
not `ALLOWED`;
pair them with the same event's
`SYSCALL success=yes` where needed.

Older denied Helium paths under `/home/user/temp` and cache paths from before its directory
correction remain rejected history.
Successful workloads do not erase those records.
The retained
complain exceptions,
including `fuse-overlayfs` mount records with `error=-13`,
remain explicit
residual risk.

## Cache and recovery

The active cache includes the latest Sanoid,
KeePassXC,
checkupdates,
Git signing,
own-hook,
custom
`gh`,
and `setpci` changes.
`build-verify-and-select-tuned-early-policy-cache.ts` compiled source
without reading old cache,
checked every expected profile artifact and the live feature bytes,
verified cache hits,
preserved prior runtime policy,
and reloaded the real loader.
The latest such report is `verified-selected-systemd-tmpfiles-temp-recursion-2026-11-26T18-15-32-0500.json`.

Original recovery cache:
`/etc/apparmor/earlypolicy/.previous-default-complain-policy`.

Current verified backup:
`/var/backups/apparmor-full-system-pre-enforcement-cold-boot-2026-11-26T18-15-32-0500`.
All 27
copied objects compared equal.
The earlier pre-enforcement backup remains at
`/var/backups/apparmor-full-system-pre-default-enforcement-2026-11-23T08-52-55-0500`.

Recovery instructions:
`/etc/apparmor/recovery.md`.
Disabling only `apparmor.service` is
insufficient once an early cache exists.
Prefer LTS with `apparmor=0`,
or edit the main Limine entry
and remove `apparmor,` from `lsm=`.

## Installed inventory

The owner-requested complete inventory is in `doc/handover/installed-system-and-repo-inventory.md`.
It covers 2,276 installed packages,
9,273 executable and symlink entries,
all 809 cache files,
all
loaded profile modes,
all systemd unit files,
every running process label,
all 570 repo manifests,
and all 24,756 tracked repo files.
Complete private JSON reports are retained under
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/keep/installed-system-and-repo-inventory/`.

## Remaining work

No implementation work remains from the approved rollout or the requested inventory.
The exact LTS
recovery entry remains untested.
KeePassXC save behavior at the selected database path,
portal
pressure writes,
and fresh profile-sync-daemon mount operations remain explicitly unexercised.
Unprofiled development and repo processes remain unconfined by design and are disclosed in the
inventory rather than misreported as universally enforcing.
IVPN's built-in firewall has not run under the enforcing `ivpn-service` profile on the host,
only in namespace probes; the owner does not use it.

This handover is not final and not owner-approved.
