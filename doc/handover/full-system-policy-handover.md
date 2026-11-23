# AppArmor full-system rollout handover

## Status at 2026-11-23T14:50:49-05:00

Incomplete and unapproved. Default enforcement and current-boot workload verification are in place. A genuinely new enforcement cold boot and explicit owner review remain required. Do not describe this as owner-approved or universally enforcing.

The owner selected the broad `apparmor.d` collection for network-facing and local threats. The required development, terminal, Podman, and Incus access to `/home/user` and `/home/user/Monochromatic` is retained. This is `fsp no`, not experimental full-system policy that blocks every unprofiled executable. Unprofiled development processes remain a residual risk.

## Current policy controls

- `/etc/apparmor/modes` contains `default enforce`, `include default`, `reload yes`, and `fsp no`.
- Managed policy was promoted with `aa-install --install`, without `--fsp`. The latest `aa-install --status` reports 710 installed profiles and zero drift.
- Custom `/etc/apparmor.d/helium` was separately changed with `aa-mode --enforce --no-reload helium`.
- Package `flags.d` exceptions remain intentional. Konsole, `fuse-overlayfs`, `pacman`, `auditd`, and other documented exceptions are not enforcing. Describe the result as default enforce with per-profile exceptions.
- Local D-Bus XML keeps `<apparmor mode="disabled"/>`. This is separate from real broker process attachment to `dbus-system` and `dbus-session`.
- `/etc/subuid` and `/etc/subgid` retain `user:100000:65536` and add `root:1000000:1000000000`.
- `/etc/default/limine` enables AppArmor on the main kernel and sets `apparmor=0` on LTS. The main kernel remains selected. The exact LTS recovery entry is untested; editing the main entry to remove `apparmor,` from `lsm=` was demonstrated.

## Local workload rules

- `/etc/apparmor.d/local/sanoid` allows owner-matched flat `File::Temp` files with ten source-derived characters and `Capture::Tiny` readiness markers with the confined parent PID suffix. It grants no other temporary-directory tree.
- `/etc/apparmor.d/local/git` allows owner-matched read, write, and lock operations only beneath `/home/user/temp/agent`, plus Git 2.56.1's six-character `.git_signing_buffer_tmp` file and its `.sig` sibling under `/tmp`. It transitions only `ssh-keygen` to its existing enforcing profile for commit signing. `/etc/apparmor.d/local/ssh-keygen` grants the same buffer and signature paths. Only this repository's measured `.git/hooks/pre-push` receives inherit-exec permission; third-party hooks and build output receive none. Git LFS may read its own `/proc/<pid>/cgroup` path without an `owner` qualifier.
- Custom `/etc/apparmor.d/gh` confines GitHub CLI 2.105.0 with its own config scope, session-bus keyring access, network and TLS access, and `git` through the existing profile. It closes the managed Git profile's previously missing `px` target without an unconfined helper.
- `/etc/apparmor.d/local/apparmor_parser` allows read access to Incus-generated profiles and read/write access to its parser cache tree.
- `/etc/apparmor.d/local/aa-install` and `/etc/apparmor.d/local/aa-mode` each allow only the observed flat `system.slice/*.scope/cpu.max` read. The broader shared `golang-strict.d` override was removed.
- `/etc/apparmor.d/local/helium` allows read-only root-owned `/etc/os-release` with audit enabled. The custom profile uses the real `net.imput.helium` configuration and cache scopes. No `/home/user/temp` browser grant was added.
- `/etc/apparmor.d/abstractions/app/chromium_crashpad_handler.d` audits the pre-existing crash-report path and permissions without adding a path or permission.
- `/etc/apparmor.d/local/keepassxc` currently audits owner-matched `rwl` access to `/home/user/Plain/main.kdbx*`. The real enforcing process now reads its existing database successfully. Save and rename behavior is source-traced but has not been exercised against a disposable database at that path.
- `/etc/apparmor.d/local/checkupdates` allows the five-character `arch-update` runtime database directory and its `local` symlink. It does not broaden other runtime paths. Pacman remains in complain mode.

## Current-boot evidence

The recorded boot is `2035a996-2f6f-45e8-a409-7712c793864b`. It is not the final enforcement cold boot.

- The early loader previously reported `Successfully loaded all binary profiles from AppArmor early policy cache (/etc/apparmor/earlypolicy/48da56f3.0).` before `apparmor.service`.
- `verification-pre-cold-boot-local-tuning-runtime.json` verifies current source modes, loaded kernel modes, and zero managed drift.
- `verified-selected-final-cache-with-gh-2026-11-23T14-50-00-0500.json` verifies all 809 cache profiles, including custom `gh`, source-only compilation, live feature bytes, cache hits, successful loader reload, and rollback preservation. Earlier current-boot Podman and strengthened Incus workloads also passed after the 808-profile selection. No new denials occurred at or after 2026-11-23T19:20:00Z in `current-boot-avc-after-enforcement-2035a996-2f6f-45e8-a409-7712c793864b-2026-11-23T19-21-18.355Z.json`; older denials and later measured Git repair denials remain in scope.
- `positive-enforcement-control-2035a996-2f6f-45e8-a409-7712c793864b-before-cold-boot-2026-11-23T16-05-41-295Z-103181.json` verifies an allowed read, an exact denied PID and AVC, and removal of the temporary source, kernel profile, and cache artifact.
- Signed commit `02f5e836951eaa990e1c6e3f88e5017b51e39d0b` and follow-up `778f7c5f9bbc5a0cacbdc7ac480bc31f4f360e3a` reached `main` with the enforcing `ssh-keygen` and `gh` profiles plus the exact own-hook rule. `gh auth status` read the existing keyring credential successfully. `git ls-remote` returned those same commit hashes.
- `markdown-rendered-review/final-workload-tuning/browser-evidence-aggregated-with-github-cli.json` records successful light and dark rendering, keyboard focus, an empty local console, and all fifteen source citations across the full run and targeted retries. The loopback server and disposable documentation-browser processes and profiles are removed.
- The real scheduled `sanoid.service` initially failed with `Capture::Tiny` status 2. After the readiness-marker correction, the unchanged tee reproduction returned exact captured stderr, and the already-scheduled 14:00 EST service exited 0. Direct ZFS queries confirmed the home, root, and varlog snapshots from that run. `sanoid-narrow-tempfile-verified-2035a996-2f6f-45e8-a409-7712c793864b-2026-11-23T18-59-31.197Z.json` separately proves nonempty readback and a retained self-label denial in the same confined process.
- `checkupdates-runtime-database-verification-4LaKN.json` proves the real five-character directory, the raw and canonical local database targets, nonempty synchronized databases, package comparisons, and fixture removal. An incorrect six-character harness path remained denied and was not added to policy.
- KeePassXC PID 4210 is enforcing. Its explicitly audited database opens are `apparmor="AUDIT"` with paired `SYSCALL success=yes`; an `ALLOWED` matcher was the wrong acceptance condition for this audit rule.
- Rootless and rootful Podman fixtures passed with independent expected-content readback and container removal. Rootless inner `unconfined` and rootful `containers-default-0.70.0 (enforce)` remain distinct.
- Host and real Konsole repository and scratch-file workloads passed. Unprofiled agent processes remain unconfined; Konsole and its descendant are complain.
- `workload-data/enforcement-validation-kernel-mapped-unprivileged-incus-2035a996-2f6f-45e8-a409-7712c793864b-2026-11-23T19-16-57.079Z.json` proves the exact outer enforcing profile, namespace presence during the workload, nonzero kernel UID/GID mappings, guest readback, namespace and outer-profile removal, and empty final instance inventory. Inner guest labels remain separately recorded as `unconfined`.
- `enforced-helium-preboot-after-tuning-audited-boundaries-example.json` proves the native enforcing browser, renderer seccomp filtering, Example Domain rendering, empty CDP console and exception lists, and no unexpected unconfined helper. The screenshot was inspected. CDP close was acknowledged, the launcher exited 0, and the disposable browser, crashpad, relay PIDs, and transient unit exited.
- That browser exercised successful audited OS-release reads, existing crash-report reads, settings read/write opens, lock creation, and lock removal. It did not create a new crash dump. Do not claim dump creation was exercised.
- Both Go helpers were observed with their exact `cpu.max` descriptor open under enforcing labels. Descriptor content matched direct quota reads.

## Audit evidence boundaries

Use `summarize-current-boot-audits.ts`, which reads all retained uncompressed audit rotations, separates `DENIED`, `ALLOWED`, `AUDIT`, and other modes, handles duplicate and unterminated records, and runs `aa-log` over captured bytes. A zero-record result requires a positive recognition control.

Installed `ausearch` reports `Malformed event skipped, rc=9`; do not use it as the oracle. Plain `deny` suppressed the targeted positive-control record, so the control uses `audit deny`. Explicitly audited allowed operations can appear as `AUDIT`, not `ALLOWED`; pair them with the same event's `SYSCALL success=yes` where needed.

Older denied Helium paths under `/home/user/temp` and cache paths from before its directory correction remain rejected history. Successful workloads do not erase those records. The retained complain exceptions, including `fuse-overlayfs` mount records with `error=-13`, remain explicit residual risk.

## Cache and recovery

The active cache includes the latest Sanoid, KeePassXC, checkupdates, Git signing, own-hook, and custom `gh` changes. `build-verify-and-select-tuned-early-policy-cache.ts` compiled source without reading old cache, checked all 809 expected profile artifacts and live feature bytes, verified cache hits, preserved prior runtime policy, and reloaded the real loader.

Original recovery cache: `/etc/apparmor/earlypolicy/.previous-default-complain-policy`.

Pre-enforcement backup: `/var/backups/apparmor-full-system-pre-default-enforcement-2026-11-23T08-52-55-0500`.

Recovery instructions: `/etc/apparmor/recovery.md`. Disabling only `apparmor.service` is insufficient once an early cache exists. Prefer LTS with `apparmor=0`, or edit the main Limine entry and remove `apparmor,` from `lsm=`.

## Remaining work

1. Commit only the intended troubleshooting and handover documents. Preserve the four unrelated repository modifications and include them in the pre-reboot backup without committing them.
2. Reboot into enforcement, require a new boot ID, and run `capture-enforcement-cold-boot.ts` with `enforcement-policy-cold-boot`.
3. Repeat the positive control with `cold-boot`, early-cache and service/session checks, host and Konsole workloads, both Podman roles, the strengthened Incus workload, and enforced Helium with phase `cold-boot`.
4. Inspect the next real scheduled Sanoid run after that boot without manufacturing snapshots. Remove temporary audit qualifiers only after their positive evidence is captured and no further cold-boot observation needs them.
5. Update `/etc/apparmor/full-system-policy.md`, finish native cleanup, and request explicit owner review. The exact LTS recovery entry remains untested.

This handover is not final and not owner-approved.
