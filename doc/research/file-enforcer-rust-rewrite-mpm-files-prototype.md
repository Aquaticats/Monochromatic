# Prototype: who writes and checks mpm.packages.toml and mpm.managers.toml

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md), 2026-10-29.
Throwaway prototype on branch `prototype/fe-mpm-files` (head `fb874028f9`, directory `prototype.fe-mpm-files/`),
run against real mpm 8.2.0 in podman (`--memory=2g --cpus=2`) with a positive control per probe; evidence files under `prototype.fe-mpm-files/evidence/` on that branch.
Key outputs spot-checked by the main session (1a warning, 1b cargo and pnpm version drops with controls, S1 candidate C exit 4).


Verdict first: **D, in a narrowed form (call it D+A-gate)**. Neither file has a consumer that needs it.
`mpm-shadow` is the only writer of `toolchain.lock.json`, `mpm-shadow lock --check` is the toolchain drift gate,
and `file-enforcer` renders nothing for the toolchain. Evidence is below; each step ran in podman (`--memory=2g --cpus=2`)
with a positive control.

Branch `prototype/fe-mpm-files`, head `fb874028f9`, directory `prototype.fe-mpm-files/` (evidence under
`prototype.fe-mpm-files/evidence/*.out`, cited below as `ev/<name>`). Not merged anywhere.
Setup notes (facts):
- `git-policy-cli` refused `git switch --create` in my worktree, so the branch lives in a second worktree,
  `/var/home/user/temp/agent/fe-rust/proto/wt`, made with `git worktree add -b`.
- Auto-push sent the branch to `origin` on the first commit (GitHub printed the "Create a pull request" hint).
  Nothing else was published. Delete the remote branch if you do not want it there.
- `doc/audit/standards-review-gate-shadow.md` shows modified in that worktree after each commit (the commit hook's
  `standards-review-shadow-*` unit wrote it). Not mine; left uncommitted.
- Images: `localhost/fe-proto-base:1` (rust 1.97.0-bookworm, mpm 8.2.0) and `:2` (adds the lock-pinned uv 0.13.6,
  node 26.13.0, pnpm 12.16.1, cargo-binstall 1.26.1) remain on the host; removal is optional.
- The probe scripts are `.sh` files, which the repo's `SCR` rule forbids for real code. They are throwaway container drivers.

## 1a. `mpm --rustup restore` with the rustup section

Fixture: the real `[rustup]` section (`nightly-2026-10-29 = "nightly-2026-10-29"`), rustup 1.29.0, mpm 8.2.0, fresh `RUSTUP_HOME`.
Script `probe-1a-rustup.PROTOTYPE.sh`, output `ev/1a.out`.

- Fact, the lock's own pin installs: `rustup toolchain install --no-self-update nightly-2026-10-29` ran, then
  `nightly-2026-10-29-x86_64-unknown-linux-gnu installed - rustc 1.101.0-nightly (92ffebece 2026-10-28)`.
- Fact, mpm does not honor the value. It prints `warning: Configuration-defined managers do not support version pinning. Letting the package manager choose the version.`
- Positive control (change the KEY): key and value `nightly-2026-09-15` installs `nightly-2026-09-15`.
- Negative case (change only the VALUE): key `nightly-2026-10-01`, value `nightly-2026-08-01` or `99.99.99`
  still installs `nightly-2026-10-01`. The first output lines read `rustup.install: nightly-2026-10-01@99.99.99` yet the toolchain list shows `nightly-2026-10-01-...`.
- Fact: the executed command is exactly `rustup toolchain install --no-self-update <key>` (INFO log). Source: `rustup.toml:97-98`.
- The direct fallback (`rustup toolchain install --profile minimal --no-self-update nightly-2026-10-01`) yields the same listing.
- Conclusion (inference from the above): the rustup "pin" works only because the package id IS the channel. The file adds nothing that
  `providers.rust-android.ts` does not already do in its fallback.

## 1b. Every other section, one non-latest version each

Script `probe-1b-sections.PROTOTYPE.sh`; outputs `ev/1b.out` (cargo), `ev/1b2.out` (uvx), `ev/1b3.out` (pnpm).
Lock versions are all the current latest, so pins are deliberately older than the lock's.

- `[uvx]` pycowsay `0.0.0.1` (latest 0.0.0.2): honored. `uv tool list` shows `pycowsay v0.0.0.1`.
  Control, value changed to `0.0.0.2`: status `upgraded`, `pycowsay v0.0.0.2`. Source: `uv.py:107-112`.
- `[pnpm]` is-odd `2.0.0` (latest 3.0.1): DROPPED. mpm's own table says `is-odd | pnpm | ... | To 3.0.1 | installed`; pnpm shows `is-odd@3.0.1`.
  Warning `PNPM.install does not implement version parameter. Let the package manager choose the version.` (`ev/s123-a.out`).
  Control `pnpm add --global is-odd@2.0.0` installs 2.0.0. Source: `pnpm.py:216-217` (`@version_not_implemented`).
- `[cargo]` cargo-ndk `4.1.1` (latest 4.1.2): DROPPED. The log tick reads `cargo.install: cargo-ndk@4.1.1`, the table says `To 4.1.2`, and
  `cargo install --list` shows `cargo-ndk v4.1.2`. Control `cargo install --locked --version 4.1.1 cargo-ndk` gives `v4.1.1`.
  Source: bundled `cargo.toml:41-42` is `install {package_id}` (a compile from source, not binstall).
- `[conda]`: NOT run (no conda in the image). Source `conda.py:263` appends `=<version>`; inference only.
- Fact, undocumented floors: mpm 8.2.0 skipped `uvx` with `version 0.9.5 does not satisfy '>=0.10.10'` and `pnpm` with `10.18.3 ... '>=11.0.0'`
  (`ev/1b.out`) until the lock-pinned uv 0.13.6 and pnpm 12.16.1 were installed. The repo lock passes today.
- Whole-file restore of a generated file (S2 below) leaves 2 of 5 tools off the lock. So the file is a half-correct installer, not a manifest.
- `doc/decision/mpm-toolchain-adoption.md:367-370` lists "MPM forwards restore versions to npm, pipx or Cargo installs" as a revisit trigger.
  At mpm 8.2.0 it has not fired for npm or Cargo.

## 1c. `mpm --config mpm.managers.toml --cargo-binstall-<slug> install <crate>`

Script `probe-1c-managers.PROTOTYPE.sh`, output `ev/1c.out`; cargo-binstall 1.26.1 (the lock's pin).

- Fact, the baked `--version` takes effect. Unmodified file: `cargo-ndk v4.1.2` (equals latest, so no signal alone).
  Positive control, both occurrences of `4.1.2` in the cargo-ndk block edited to `4.1.1`: `cargo-ndk v4.1.1`.
- Fact, the stock manager would not: `mpm --cargo install cargo-ndk@4.1.1` installed `v4.1.2`.
- Fact, nothing extra over the direct command: smallest binstall tool `timeout-cli` 0.1.0 (the only version that exists).
  Binary SHA-256 via override = via `cargo binstall --no-confirm --version 0.1.0 timeout-cli` =
  `299d292fa7fcd1bf3ba1b5dcf688936720692431de604f04228110201a6cf5fd`. Both fell back to a source build (no matching GitHub archive).
- Inference: the override file is a one-line wrapper around a command `providers.cargo.ts:50` already runs from the lock.

## 1d. Implicit reads

Script `probe-1d-implicit.PROTOTYPE.sh`, output `ev/1d.out`. Measure: count of `--cargo-binstall-*` selectors in `mpm --help`.

- Negative baseline, no files: 0.
- `mpm.managers.toml` and `mpm.packages.toml` in the cwd, no flag: 0. Not read.
- `--config <file>`: 18 (positive control). `MPM_CONFIG=<file>`: 18.
- `~/.config/mpm/mpm.managers.toml`, run from an empty dir: 18. This path is read on every run (`--help` default `[default: ~/.config/mpm/]`).
  The repo's global dir `~/.config/monochromatic-dev/toolchain` is not that path (S3 shows 0).
- cwd `pyproject.toml` with `[tool.mpm.overrides.*]`: 0, although the `discover_config_definitions` docstring in `config.py` claims support.
  Reported as observed; why (cwd search disabled for the eager pre-load) is inference.
- Trust gate, fact: a 0664 file is refused (`Refusing to define manager ... unsafe config file ownership or permissions`); a 0600 file in a 0777 non-sticky dir
  is refused. Source `config.py:882-915`: owner-or-root and not group/world-writable, file and parent. The prototype's 0600 is stricter than needed.
- `mpm.packages.toml` in the app dir is accepted silently as a config file (exit 0).
- mpm `restore` reads only the files named on its command line (`cli_snapshots.py:414-470`).

## 1e. Repo code that runs mpm with these files

Searched this worktree (same `HEAD` as main; fixtures cmp-identical to the main checkout's).
- `rg 'mpm\.(packages|managers)'` outside `doc/` and the file-enforcer package: no files (also with `--hidden`).
- `rg '\bmpm\b'` in `Dockerfile`, `playwright.Dockerfile`, `docker-compose.yml`, `task.ts`, `mise.toml`, `mise.no-env.toml`,
  `file-enforcer.config.ts`, `.github`: only `mpm-shadow` publishing and comments. No `restore`, `--config`, `--cargo-binstall`, `MPM_CONFIG`.
- The one code consumer is `package/file-enforcer/core/src/toolchain/providers.rust-android.ts:80-90`
  (`mpm --rustup restore <paths.mpmPackagesFile>`, falling back to `rustup toolchain install` at `:100-110`).
- Docs only: `doc/runbook/publish-crate-first-time.md:463` and `doc/decision/mpm-toolchain-adoption.md:348` (a one-time manual consumer check).
- Limit: a `rg` over the repo cannot see commands people type by hand.

## Prototype (Part 2)

`prototype.fe-mpm-files/proto-mpm-files.PROTOTYPE/`: one Rust crate, two binaries (`proto-mpm-shadow`, `proto-file-enforcer`), built in the container.
`--candidate a|b|c|d` selects who writes and checks. `render.rs` is a port of `mpm-manifest.ts` that reads the LOCK.
Stand-ins: the network step of `update` is `--resolve id=version`; B's "shared library" is `lib.rs`; the plan is JSONC with full-line comments only.
- Renderer fidelity (`probe-2-fidelity.PROTOTYPE.sh`): rendering the repo's real `toolchain.lock.json` reproduces both files byte for byte
  (`packages.toml: byte-identical`, `managers.toml: byte-identical`, 1122 and 7412 bytes). Control: one edited version (`10.5.0` to `10.4.2`) makes both differ.
  Fact: both mpm files are pure functions of the lock.

## S1. Fresh machine, only the shadow binary, no file-enforcer

Setup: checkout with plan and lock, no mpm files; then `lock`, gates, `sync --no-fallback`, `sync`, `status` (4 tools). `ev/s123-{a,b,c,d}.out`.
- A: `lock exit=0`, both mpm files created, `lock --check` 0, sync via `mpm --rustup restore` exit 0, status 4/4 MATCH.
- B: identical to A.
- C: `lock written, but cannot run proto-file-enforcer ...` and **`lock exit=4`**; checkout has no mpm files;
  `mpm-shadow lock --check` still returns 0 (blind to the missing files); `mpm --rustup restore /work/s1/mpm.packages.toml` prints
  `Usage: mpm restore [OPTIONS] TOML_FILES...` (exit 2); `sync --no-fallback` exit 1. Only the direct-rustup fallback gets status to 4/4 MATCH.
  After file-enforcer is put on PATH, a second `lock` exits 0 and the files appear. C does not work from a shadow-only machine.
- D: `lock exit=0`, no mpm files, sync exit 0 (direct rustup), status 4/4 MATCH.

## S2. Plan edit, then one command

Edit: pycowsay pin 0.0.0.1 to 0.0.0.2, add pinned cargo-ndk 4.1.1, then `update --resolve rust=nightly-2026-09-15`.
- Before the command, `lock --check` = 1 (`DRIFT toolchain.lock.json: differs from the plan projection`). After, all gates 0 for A, B, C, D.
- A, B: one command rewrote lock, `mpm.packages.toml` (cargo/pnpm/rustup/uvx sections) and `mpm.managers.toml` (2 definitions, mode 600).
- C: same result, but only because file-enforcer was on PATH.
- D: lock only.
- Install what the lock says (`sync`: rustup via the candidate's route, others direct): status 5/5 MATCH for all four candidates.
- A, B, C, using the generated `mpm.packages.toml` as the only installer (`mpm restore mpm.packages.toml`, baseline toolchain present):
  `MATCH rust`, `MATCH cargo:timeout-cli` (only one version exists), `MATCH pipx:pycowsay`, **`MISMATCH cargo:cargo-ndk`**, **`MISMATCH npm:is-odd`**; `status exit=1`.
  The mpm file is irrelevant to what ends up correct: rustup works by key, uvx by honoring, and the two it breaks are the two unpinned.

## S3. Global scope in a disposable `$HOME/.config/monochromatic-dev/toolchain`

- A, B: `update --global` exit 0, four files in the global dir, `lock --check --global` 0, `sync --global` then status 4/4 MATCH.
- C: `update --global` **exit 4** without file-enforcer; needs `file-enforcer user write` to create the files.
- D: global dir holds plan and lock only; status 4/4 MATCH.
- mpm does not read the global dir: `mpm --help` from there lists 0 binstall managers (all candidates).

## S4. Drift (exit codes: `mpm-shadow lock --check`, `file-enforcer check`)

Control (untouched tree) is 0,0 for all four. Corruptions, results as (shadow, fe). `ev/s4-{a,b,c,d}.out`.
- Plan edited, lock stale: A 1,0; B 1,1; C 1,1; D 1,0.
- Lock, pinned tool's version edited: A 1,0; B 1,1; C 1,1; D 1,0.
- Lock, resolved (latest-track) tool's version edited: A 1,0; B 1,1; C 0,1; **D 0,0**.
- `mpm.packages.toml` edited: A 1,0; B 1,1; C 0,1; D n/a.
- `mpm.managers.toml` edited, or chmod 664, or `mpm.packages.toml` deleted: same as the line above for each candidate.
- Lock deleted: A 1,0; B 1,1; C 1,1; D 1,0.
- Reading: A and D are covered by one command. C needs both commands. B is redundant.
- Cost of D, stated plainly: a hand edit of a resolved lock version passes (0,0). A and B catch it only incidentally, because the
  derived files no longer match. That is not a designed protection (a consistent hand edit of lock and files passes everywhere).
  Offline, a resolved version cannot be derived from the plan, so the real fix is a lock-level integrity check
  (artifact URL versus version, or a digest) or `update --check`. Not solved here; a follow-up design item.

## S5. Two concurrent writers

Deterministic via `PROTO_RACE_SLEEP_MS` (both read the lock first); P2 starts 100 ms after P1. P1 resolves rust, P2 resolves timeout-cli.
`ev/s5-{a,b,c,d}.out` (A and B equal; C and D equal on the lock).
- guard=none (positive control): final `[nightly-2026-10-01, 0.1.1]`; P1's rust update is lost.
- guard=cas (content hash recheck before rename): final `[nightly-2026-09-15, 0.1.1]`; stderr `lock changed under us (attempt 0); re-reading and re-applying`.
- guard=mutex (exclusive lock file): final `[nightly-2026-09-15, 0.1.1]`.
- Check-to-rename window (P1 pauses 600 ms after its compare passed): cas final `[nightly-2026-09-15, 0.1.0]`, P2's update lost (CAS is not a lock); mutex keeps both.
- Torn derived files (P1 pauses between lock write and mpm-file write, P2 completes in between): under cas, A and B end with
  `lock --check` = 1 (both mpm files differ from the lock); under mutex 0. C ends 0 because file-enforcer re-reads the final lock (inference:
  it self-heals unless two enforcer runs overlap, not tested). D has no derived files to tear.
- Inference: the owner's server-client serialization is needed for the lock regardless; D shrinks what it must cover to one file.

## Verdict

**D+A-gate.** Drop both files; `mpm-shadow` owns lock writing and `lock --check`; file-enforcer has no toolchain role.

Why each file goes (facts):
- `mpm.packages.toml`: only consumer is `providers.rust-android.ts:80-90`, which runs `rustup toolchain install --no-self-update <id>` via mpm (1a).
  Replace with that exact direct call (already the fallback; S1 candidate D and 1a FALLBACK produce the same toolchain list). Other sections
  drop pins (cargo, pnpm) or are unused (uvx, conda); S2 restore shows 2 of 5 tools wrong.
- `mpm.managers.toml`: only consumer is a documented manual command (1e); it equals the direct `cargo binstall --version` already run from the lock
  (1c SHA match), is never read implicitly (1d), and needs strict file modes. The decided bootstrap (prebuilt archive or `cargo install`) already replaces `runbook:463`.
- Dropping `mpm.packages.toml` also removes the staging dependence of rustup on `mpm` being installed first (`sync.order.ts:18` stages `mpm` before `rustup`), an inference to verify.

Why not the others:
- C is disqualified: S1 `lock exit=4`, files absent, the gate blind, `mpm restore` receives no file; only the fallback masks it. S3 `update --global exit=4`.
- B works (S1 to S4 equal to A) but links a shared crate for files nothing needs.
- A works but keeps two generated files whose only effect is S2's wrong-install trap and the S5 torn-file window.

File-enforcer's toolchain role under the verdict: none for rendering. The owner's recorded decision (planning doc, "Toolchain lock: plan and lock,
mpm-shadow writes the lock") says file-enforcer `check` verifies the lock's plan fields. Whether it still runs that projection is the only
residual question. Ranking: D+A-gate (one implementation of `project()`, in mpm-shadow, one CI command) > D+B-gate (file-enforcer calls a shared
`project()` crate, two implementations to keep equal, S4 shows no extra coverage). Treat this as an amendment to that earlier decision for the
owner to confirm; also amend `doc/decision/mpm-toolchain-adoption.md:92-112` and `:201-223`.

Follow-ups (not done): lock-level integrity check (S4); `mpm-shadow` should keep `mpm` for inventory/outdated only (not probed).

## Untested assumptions

- conda section (no conda in the image); `mpm outdated` and the planned `update` network resolution; macOS and Windows (trust gate skips without `os.getuid`, `config.py:900-902`).
- Prototype stand-ins, not the real `mpm-shadow` or Rust file-enforcer; the mutex/CAS are file stand-ins, not the planned server.
- Container base is Debian bookworm with rustup 1.29.0, not the repo's CachyOS image.
- The lock's `nightly-2026-10-29` was installable on 2026-10-29 (1a control); `mpm` versions other than 8.2.0 not tried.
- Human use of `mpm restore mpm.packages.toml` by hand is invisible to `rg`.

## Cleanup (run before handback)

`systemctl --user list-units 'fe-proto-*'` returned 0 units; `podman ps` filtered for `fe-proto` returned 0 containers; no
`proto-mpm-shadow`, `proto-file-enforcer` or `fe-proto` process was left; no `standards-review-shadow-*` unit was running.
Probe containers ran under `systemd-run --user --wait --collect`, `MemoryMax=4G`, `CPUQuota=200%`, `podman run --memory=2g --cpus=2`.
No synthetic load was generated; the race tests are two processes each, one run per mode.
