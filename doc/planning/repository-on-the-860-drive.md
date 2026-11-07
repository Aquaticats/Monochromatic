# Repository on the 860 drive: measurements, inventory, the symlink design and runbook

Status:
revised 2026-11-07T06:45:00-05:00 to the design the owner picked (a symbolic link at the old path).
The first version of this plan (automount over a locked stub,
option A) is at commit `93fec99100`
and its option text stays in this file as history.
Nothing real was moved,
copied,
mounted,
unmounted,
unlocked,
stopped or deleted while writing either version.
Every command in the runbook that needs root is for the owner to run;
agents do not run `sudo`.

## Owner requirements this plan follows

- "Make this repo backed by 860 on disk."
  (2026-11-07)
- On ordering the 860's unlock before the user session with `Before=user@1000.service`:
  "No,
  that would be the wrong design.
  Make ordering don't matter."
  (2026-11-07)
  So the design works whichever starts first,
  the 860's unlock and mount or anything that uses the repository.
  No unit gets `Requires=` or `After=` on the mount.
- Design pick over the plan's recommended automount:
  "Symlink at the old path.
  You're supposed to take care of dangling links."
  (2026-11-07)
- Worktrees:
  "All worktrees (Recommended)".
  (2026-11-07)
- The unlock timeout question:
  "N/A".
  (2026-11-07)
- On the credential in `.git/config`:
  "We control our lfs setup.
  This is the defect in the lfs pkg we control.
  Fix it."
  (2026-11-07;
  another agent is fixing it.
  Once that lands `.git` holds no secret.
  This plan keeps the 0700 directory anyway.)
- "nothing should ever wait forever or indefinitely"
- "never discard user data",
  "Commits should never fail.",
  "Disk pressure being high is not an excuse.",
  "load is not an excuse."
- The Git wrapper never starts itself at system startup or login (systemd units are fine).
- "Never generate synthetic host load in any session or subagent:
  no `stress-ng`,
  busy loops,
  `--stress-count`,
  or repeated full test runs;
  cap the test's own container (`--cpus` sized to the test) instead."
- `AGENTS.md` rule RBT:
  a reboot can make data on this machine's NVMe permanently unreadable,
  so commit and push before any reboot and never count on uncommitted files surviving one.

## What the plan decides and what it leaves to the owner

The owner picked the design,
so this plan no longer ranks designs for the decision;
it specifies the picked one and makes it safe.

- Decided by the owner:
  `/var/home/user/Monochromatic` becomes a symbolic link into the 860,
  every worktree moves with it,
  and a missing link target ("a dangling link") must be handled by every tool.
- Decided by this plan from measurements,
  open to the owner's veto:
  the place on the 860 (a dedicated 0700 directory),
  the guard on the mountpoint,
  the shared check every tool calls,
  the per-tool work items,
  the two-pass btrfs send and receive copy,
  and the cut-over and rollback order.
- Left to the owner:
  the ranked open questions at the end of the file.
- Not run here:
  anything that needs root,
  so the guards on a real mountpoint and the btrfs send and receive were not exercised.
  The user-level fixture in the section "Dangling-link guards,
  measured" exercised everything a namespace allows,
  and the section "The copy" gives the exact root fixture test for the rest.

## Measurements

All sizes were taken on 2026-11-07 between 05:16:00-05:00 and 05:47:00-05:00 while other sessions kept committing,
so each number is a snapshot,
not a quiet-host figure.

### Tools and bounds

- `du --summarize --one-file-system --block-size=1M` ran inside `systemd-run --user --scope`
  with `CPUQuota=100%`,
  `MemoryMax=1G`,
  `TasksMax=16` and `IOWeight=10`.
- `btrfs filesystem du --summarize --raw` ran in the same scope bounds.
- One unwrapped `du` of the pnpm store (`~/.local/share/pnpm/store`) was started by mistake and stopped within
  minutes;
  its size is therefore not reported.

### The trap in `du --one-file-system` on this repository

Every btrfs subvolume has its own device number (`stat --format=%d` printed 65 for the repository,
244 and 110 for two
worktrees).
`du --one-file-system` therefore skips subvolumes.
Of the 246 entries in `.claude/worktrees`,
242 are subvolume roots (inode 256) and 4 are plain directories.
The first `du` over `.claude` (134046 MiB) counted only the 4 plain directories
(`fast-snapshot-add`,
`cargo-shadow-r21`,
`source-layout`,
`follow-ups-059`).
The repository root is itself a subvolume (inode 256),
and the worktree subvolumes are nested inside it.

### Disks

- NVMe pool:
  label `bazzite_bazzite`,
  one btrfs over two LUKS devices (`dm-0` 1T and `dm-1` 881.1G,
  read from `/sys/fs/btrfs/*/devices`).
  `df`:
  1.9T size,
  1.5T used,
  350G free;
  mounted at `/var/home` as subvolume `/home`,
  `compress=zstd:3`.
- 860 (`sda`,
  Samsung SSD 860 EVO 4TB):
  LUKS `crypt_sda`,
  label `encrypted_storage`,
  one btrfs on one device,
  mounted whole (`subvolid=5`) at `/var/mnt/encrypted`,
  `compress=zstd:3,noatime`.
  `df`:
  3.7T size,
  3.0T used,
  662G free.
  Top-level directory is `user:wheel` mode 777;
  its entries include `Archive`,
  `Backup`,
  `cargo-build`,
  `low`,
  `pcloud`,
  `SteamLinux`,
  `temp`,
  `VMs`,
  `win-salvage`.

### Repository, by part

- Tracked content in the main worktree:
  21543 files,
  876042393 bytes apparent
  (`git ls-files | wc -l` and `stat --format=%s` summed).
- `.git`:
  2158 MiB on disk.
  Packs are 709.78 MiB (4 packs,
  413533 objects);
  loose objects 1.55 MiB (184);
  measured split of the 2158 MiB (capped `du --summarize` per part):
  `objects` 728 MiB (packs and loose objects together),
  `worktrees` 881 MiB (340 administrative directories),
  `lfs` 362 MiB,
  `rr-cache` 94 MiB,
  `lost-found` 15 MiB,
  `logs` 8 MiB,
  wrapper records `monochromatic-git-policy-cli` 16 MiB and its `-worktree-copy` 2 MiB,
  the remainder (index,
  refs,
  small files) is the difference to the total.
  `.git/config` carries a credential inside the `lfs.url` value,
  so every copy of `.git` copies that credential;
  the new subvolume's top directory must be mode 0700 (the 860's top level is mode 777).
- Ignored and build output in the main worktree:
  `package` 73372 MiB (almost all of it ignored build output,
  since tracked content is 0.82 GiB in total),
  `node_modules` 3119 MiB,
  `.pnpm-store` 1095 MiB,
  `.serena` 152 MiB,
  `.cache` 51 MiB,
  `target` 4 MiB.
  The main worktree without worktrees is therefore about 80.5 GiB on disk.
- Worktrees nested in `.claude/worktrees`:
  243 measured of 246 listed (3 vanished while `du` ran,
  removed by other
  sessions).
  Sum of per-worktree `du` is 3843069 MiB,
  which is 3753 GiB,
  and it counts shared extents once per worktree.
  Distribution:
  median 1866 MiB,
  minimum 793 MiB,
  maximum 47048 MiB;
  84 worktrees are above 40000 MiB (copies of main's ignored state) and 137 are below 2000 MiB.
- Reflink sharing:
  `btrfs filesystem du --summarize` on one worktree (`add-explains-slow-route`) printed total 43.99 GiB,
  exclusive 383.86 MiB,
  set shared 43.09 GiB.
  The manual (`btrfs-filesystem(8)`,
  btrfs-progs 7.1) defines exclusive as not shared and set shared as the
  overlap-aware sum of shared space in the argument's tree.
- Whole-repository figure with sharing counted once (`btrfs filesystem du --summarize --raw` on the repository root,
  which descends into nested subvolumes):
  see the section "Whole-set figure".
- Worktrees outside the repository tree:
  the repository listed 341 worktrees when this was first written and 340 at 2026-11-07T06:26:00-05:00,
  of which 246 are under `.claude/worktrees`,
  16 are on the NVMe (`~/worktrees`),
  71 are on the 860 (`/var/mnt/encrypted/temp/agent`),
  and 6 are spelled `~/temp/agent/...`,
  which is also the 860 because `~/temp` is a link to `/var/mnt/encrypted/temp`
  (the first version of this plan counted those 6 as NVMe;
  that was wrong),
  plus the main one.
  Of the 246 nested ones,
  242 are subvolume roots and 4 are plain directories.
  None is locked (`git worktree list --porcelain | rg --count '^locked'` printed 0).
- Branch state of the 224 worktrees on a branch:
  180 branches are already merged into `main`;
  116 worktrees are detached.
  This says nothing about uncommitted files;
  those need `git status` per worktree.
- Files the user cannot read:
  `btrfs filesystem du` printed permission errors for root-owned mode 600 log files under
  `node_modules/.monochromatic/` inside worktrees (one sample:
  `root:root 600`).
  A user-level copy cannot read them;
  only a root-level copy (send and receive,
  or `rsync` as root) keeps them.
  The count (126) is in the section "Whole-set figure".

### Free space after each scope

Scopes,
because the nested worktrees decide what can fit:
- Scope one:
  main worktree only,
  about 80.5 GiB.
  860 free:
  662G minus 80.5 GiB leaves about 581 GiB.
  NVMe free:
  350G while the old copy is kept,
  about 430G after the owner removes the old main part
  (nested worktrees are not part of that gain,
  they stay where they are).
- Scope two:
  main plus all 243 worktrees copied file by file (`cp`,
  `rsync`):
  about 3753 GiB,
  which exceeds 662G
  by more than five times.
  This scope cannot be done.
- Scope three:
  main plus all worktrees with extent sharing kept (send and receive with clone sources):
  263.6 GiB of logical bytes measured in "Whole-set figure",
  fewer on disk after compression.
  860 free:
  662G minus at most 263.6 GiB leaves at least about 398 GiB.
  NVMe free after the owner removes the old copy:
  up to the shared-extent footprint of that copy comes back,
  which this plan did not separate from other data on the NVMe pool and does not estimate.
- Scope four:
  main plus a chosen subset of worktrees (for example the 44 unmerged branches plus the 116 detached
  ones,
  owner's choice):
  between scope one and scope three.

### Whole-set figure

`btrfs filesystem du --summarize --raw /var/home/user/Monochromatic`
(inside `systemd-run --user --scope` with the four bounds,
started 2026-11-07T05:41:25-05:00,
finished before 2026-11-07T05:53:59-05:00;
it descends into nested subvolumes) printed:

- Total 4115920404480 bytes (3833 GiB),
  the sum with every shared extent counted once per reference.
- Exclusive 106125254656 bytes (98.8 GiB).
- Set shared 176884908032 bytes (164.7 GiB).
- Exclusive plus set shared is 283010162688 bytes (263.6 GiB):
  the footprint with sharing preserved,
  as logical bytes before zstd compression.
  The per-worktree `du` sum (3753 GiB) is on-disk compressed bytes per worktree with sharing ignored,
  which is why it
  is close to the Total.
- Unreadable files:
  126 permission errors (all root-owned mode 600 `*.log.jsonl` and `*.probe.tmp` files under
  `node_modules/.monochromatic/`):
  90 in worktree `agent-a6f97f5da6f22de4d` and 36 in `package/wg/cli-quicker`.
  Their bytes are missing from these figures.

## Inventory of what names or depends on the repository path

Spellings seen in the wild:
`/var/home/user/Monochromatic` (the real path) and `/home/user/Monochromatic`
(through the root symlink `/home -> var/home`).
Both resolve to the same directory;
programs that call `realpath` or `canonicalize` see the first.

### Commands and counts

- Tracked files naming the absolute path:
  `git grep --files-with-matches --fixed-strings '/home/user/Monochromatic' | wc -l` printed 191 files
  (420 occurrences),
  mostly `doc/troubleshooting` 80,
  `doc/planning` 31,
  `doc/audit` 15,
  `doc/handover` 14,
  `package/pi-plugin` 11,
  `package/git-policy` 10.
  Historical records stay as written (rule WR6);
  none is a runtime dependency except those listed below.
- User units:
  `rg --follow --files-with-matches --ignore-case 'Monochromatic' ~/.config/systemd/user` printed 4 paths
  (`claude-cpu-watch.service` and its `default.target.wants` link,
  `chromium-issue-581090722-watch.service`,
  `kopia-snapshot-all.service`;
  the last two only name `~/.local/share/monochromatic-dev`,
  the toolchain outside the
  repository).
  Without `--follow`,
  `rg` skips symlinks and misses the first:
  the unit file `claude-cpu-watch.service` is itself a
  symlink to `/home/user/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`,
  so the user manager dereferences the repository path when it loads units,
  before any `ExecStart` runs.
  Under the symlink design the same link is dangling at every login before the 860 is mounted,
  so the unit would not load;
  work item W8 replaces the symlink with a copy of the unit file.
- System units and autostart:
  `rg` over `/etc/systemd`,
  `/usr/lib/systemd`,
  `~/.local/share/systemd`,
  `/etc/xdg/autostart` found nothing;
  `~/.config/autostart/key-helper-service.desktop` names the repository.
- Generated and transient units:
  the wrapper writes `cli-git@.service` and `cli-git@.socket` to
  `/run/user/1000/systemd/user/`;
  12 service instances and 12 socket instances are loaded now.
  `/run/user/1000/systemd/transient` holds 33 units,
  12 of which name the repository (terminals,
  a browser,
  the music
  player,
  `rust-nightly-*` runs,
  `cargo-shadow` scopes);
  they are running work,
  not configuration.
- Shell and mise:
  `~/.bashrc` line 57 exports `ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json`;
  `~/.config/mise/config.toml` names only `~/.local/share/monochromatic-dev` paths.
  `$PATH` holds 6 repository directories at the front (`.cache/*-bin`) and a long tail of
  `package/*/node_modules/.bin`;
  `git` resolves first to `.cache/cli-git-bin/git`,
  which is a symlink to
  `.cache/toolchain-launchers/cli-git-native` inside the repository;
  second is `~/.local/bin/git`.
- Toolchain links in `.cache`:
  187 symlinks,
  of which 12 point back into the repository's own
  `.cache/toolchain-launchers`,
  the rest into `~/.local/share/monochromatic-dev`,
  `~/.local/share/pnpm`,
  `~/.cargo`.
  16 launcher files embed the repository path (for example `cli-git-native.launch` holds the absolute lock file path).
- mise trust:
  `~/.local/state/mise/trusted-configs` holds 48642 links;
  18700 point into the repository and 13897 into
  `/var/mnt/encrypted`.
  mise canonicalizes a config path and hashes the canonical path (`trust` code in `src/config/config_file/mod.rs`,
  `path.canonicalize()` then `hash_to_str(&canonicalized_path)`,
  read from the `main` branch on 2026-11-07).
  A different canonical path therefore means every config is untrusted until trusted again.
- Wrapper state:
  `~/.local/state/cli-git/service` holds 591 instance directories and 3194 files,
  all finished
  records (none pending at the time of counting);
  3963 files under `~/.local/state/cli-git` embed the path text.
  The instance name is `instance_of` in `package/git-policy/cli/src/native/service/service_instance.rs`:
  FNV-1a 128-bit over the bytes of `std::fs::canonicalize(git_dir)`.
  Inside `.git` the wrapper keeps `monochromatic-git-policy-cli` (3573 files),
  `...-worktree-copy` (483 files),
  `cli-git-transactions` (3),
  `cli-git-captures` (2);
  they move with `.git`.
- Worktree administrative files:
  340 `.git/worktrees/<id>/gitdir` files and the same number of `.git` files in the
  worktrees,
  all absolute (`git worktree list` printed 341 lines).
- Editor workspaces:
  `rg --files-with-matches` over `~/.config` found 34 JetBrains files
  (`recentProjects.xml`,
  `trusted-paths.xml`,
  `LanguageServersSettings.xml` with 68 matches,
  workspace files),
  21 `.wrangler`,
  18 `net.imput.helium`,
  4 `lapce-stable`,
  2 each in `Codex`,
  `Emdash`,
  `Polypane`,
  `t3code`,
  `browser-os`;
  `~/.local/share/recently-used.xbel` has 35 matches.
  `monochromatic.code-workspace` and `.idea` use relative or repository-relative paths.
- Harness:
  `~/.claude/projects` has 14 folders;
  `-var-home-user-Monochromatic` (134 session entries) is keyed by the
  working directory string.
  `~/.claude/settings.json` has 4 hook,
  statusline and permission lines naming
  `package/claude-code-plugin/...`;
  `~/.claude.json` names the repository 5 times (two MCP servers `mvm` and
  `search-fetch` run from `package/mcp/mvm/dist/` and `package/search-fetch/mcp/dist/`);
  `~/.codex/config.toml` names it 3 times (one MCP server with `--cd /home/user/Monochromatic`).
- pnpm:
  `pnpm-workspace.yaml` sets `storeDir: ${HOME}/.local/share/pnpm/store` (on the NVMe),
  `packageImportMethod: clone-or-copy` and `virtualStoreType: global`.
  A repository on the 860 cannot clone from a store on the NVMe;
  pnpm then copies.
  `doc/troubleshooting/pnpm-symlinked-store-path.md` records that a symlinked spelling of a path in pnpm 12.16.1
  produced broken relative links.
- cargo:
  `.cargo/config.toml` uses a path relative to the config file's real path for `ZIG_GLOBAL_CACHE_DIR`;
  the build output (`target`) is inside the repository.
- Containers:
  19 podman containers;
  5 are in state `created` with bind mounts of worktree paths under
  `.claude/worktrees`;
  1 is running and does not mount the repository.
- Backups:
  `kopia snapshot list --all` lists the source `user@bazzite:/home/user/Monochromatic` among 10;
  the global policy has "Scan one filesystem only:
  false".
  The timer `kopia-snapshot-all.timer` runs one minute after login and hourly.
  Two other sources,
  `/var/mnt/encrypted/Archive` and `/var/mnt/encrypted/low`,
  are already on the 860.
  `btrfs-qgroup-rescan-home.service` documents that snapper snapshots `/var/home` hourly;
  snapshots of a parent
  subvolume do not contain nested subvolumes,
  so the repository (a nested subvolume) is not part of them;
  this is
  btrfs semantics,
  not probed here.
- Running processes:
  30 with the repository root as working directory,
  6 each in `settle-command-review-fixes` and
  `rust-latest-nightly`,
  3 in `rust-baseline-nightly`,
  7 `claude` processes,
  12 `cli-git-service` processes,
  plus
  MCP servers,
  the Codex app server,
  `claude-cpu-watch`,
  an IDE.
  (`find /proc -maxdepth 2 -name cwd -user user -printf '%l\n'`.)

### What happens when the repository is unreachable at the moment the item runs

This section was written for the first version.
The behavior under the symlink design,
with measurements,
is in the section "Work items per tool".

- `claude-cpu-watch.service`:
  the user manager reads the unit file through a symlink into the repository;
  if the
  target is missing at manager start or at `daemon-reload`,
  the unit is not loaded and nothing retries.
  If it is loaded and `cpu-watch.ts` is missing,
  `Restart=always` with `RestartSec=2` and the default start limit
  (5 starts in 10 s,
  read with `systemctl --user show`) ends in a failed unit that stays failed until `reset-failed`.
  This is the item most sensitive to ordering.
- Claude Code hooks (`bash-output-filter`,
  statusline) and the two MCP servers:
  each command is `node /var/home/user/Monochromatic/...`;
  with the path missing the command exits with an error,
  so tool calls and MCP servers fail for that session until restarted.
- Codex MCP server with `--cd /home/user/Monochromatic`:
  fails to start.
- `~/.config/autostart/key-helper-service.desktop`:
  already broken today,
  it names `packages/kwin/...` and the
  directory is `package` (`ls` printed "No such file or directory");
  independent of this move.
- `$PATH` entries inside the repository:
  missing directories are skipped by the shell;
  `git` then resolves to
  `~/.local/bin/git`,
  the wrapper outside the repository.
- Wrapper services:
  their program (`~/.local/share/monochromatic-dev/toolchain/cargo/...`),
  journal
  (`~/.local/state/cli-git/service/<instance>`) and sockets (`$XDG_RUNTIME_DIR/cli-git`) are outside the repository;
  they stay up and each request about an unreachable Git directory fails.
  `instance_of` hashes a path that cannot be resolved as given,
  so a dangling link makes a different name.
- kopia:
  an unreachable source path is reported as an error by the hourly job;
  behavior with an empty stub (a snapshot
  of nothing) is inferred,
  not probed,
  and the locked stub below avoids it.
- mise:
  trust links are keyed by canonical path;
  unreachable means the repository's config is not loaded
  (inside the repository only;
  shells elsewhere are unaffected).
- Worktrees on the NVMe outside the repository (22):
  their `.git` file points into the repository's
  `.git/worktrees`;
  unreachable means "not a git repository" errors,
  no loss.
- Worktrees on the 860 (71):
  today,
  with the 860 not yet mounted,
  their paths are missing,
  and a `git worktree prune`
  (also run by `git gc`) removes the administrative directory of a worktree whose directory is missing once
  `gc.worktreePruneExpire` (default three months) has passed.
  None is locked,
  so this hazard exists today and is independent of this plan;
  `git worktree lock` on those 71 removes it.
- pcloud (`pcloud.service`) already uses `RequiresMountsFor=/var/mnt/encrypted` and `After=var-mnt-encrypted.mount`
  in the user manager;
  it is the owner's existing pattern and is unaffected by the repository.
- Containers in state `created`:
  their bind sources are resolved at start;
  a start while the repository is
  unreachable fails with a missing source.

## Options considered (history; the owner picked option B on 2026-11-07)

This section and the next are the first version's analysis,
kept as the record of what was weighed.
The owner picked option B ("Symlink at the old path.
You're supposed to take care of dangling links.").
The sentences that call B a hazard describe what a bare link does;
the section "Dangling-link guards,
measured" and the work items answer them.

All options are judged on the owner's rule:
whichever starts first,
the unlock and mount or anything that uses the
repository,
the result must be correct,
nothing waits forever,
nothing is written to the NVMe by mistake.

### Facts about the mount mechanism, read from source

Read from `src/core/automount.c` and `src/core/job.c` at the `main` branch on 2026-11-07
(installed systemd is 262.1-1.fc45;
the exact tag was not read,
so the fixture test in the runbook repeats these):

- An automount unit mounts autofs at its path,
  and the first access starts the paired mount unit.
- A process that waits on the path is released with an error when the mount unit reaches dead or failed:
  `automount_send_ready(a, a->tokens, -ENODEV)` in `automount_trigger_notify`.
  `job_finish_and_invalidate` calls `unit_trigger_notify` even for a timed-out or dependency-failed job,
  so a
  failed unlock also ends in that error.
- If the mount unit hits its start limit,
  the automount unit fails and unmounts the autofs;
  the waiters get `-EHOSTDOWN` and the path becomes the bare directory underneath.
  So the directory underneath must be safe to expose.
- The automount `Where=` must be a canonical path (`unit_fail_if_noncanonical_mount_path`),
  so the unit is named for
  `/var/home/user/Monochromatic`,
  not `/home/user/Monochromatic`.
- A mount unit gains `Before=local-fs.target` unless `nofail` is among its options
  (`systemd.mount(5)`,
  "Default Dependencies";
  `mount_is_nofail` in `src/core/mount.c`),
  and `mount-860.service` is `After=local-fs.target`,
  so the mount unit needs `nofail` or the pair forms an ordering cycle.
- Idle expiry is off unless `TimeoutIdleSec=` is set;
  the plan leaves it off.
- `mount-860.service` is `Type=oneshot`,
  for which `TimeoutStartSec=` defaults to infinity
  (`systemd.service(5)`);
  `systemctl show` printed `TimeoutStartUSec=infinity`.
  That is a wait without end in any chain that depends on it.
- `JobTimeoutSec=` defaults to infinity and cancels the job without failing the unit (`systemd.unit(5)`).

### Option A: automount of a repository subvolume on the 860 over a locked stub at the old path

- Layout:
  a subvolume `/var/mnt/encrypted/Monochromatic` holds the repository (nested worktree subvolumes inside it).
  `var-home-user-Monochromatic.automount` and `.mount` (bind) serve `/var/home/user/Monochromatic`.
  The mount unit has `Requires=mount-860.service` and `After=mount-860.service`,
  so the first access starts the
  unlock;
  `mount-860.service` gets a start timeout and a bounded notification.
  The old directory is a root-owned,
  mode 000,
  immutable (`chattr +i`) empty stub.
- Before the 860 is mounted:
  the first access blocks only for the time the unlock takes (bounded by the unit
  timeouts) and then either succeeds or returns `ENODEV` ("No such device");
  no process ever holds a handle inside
  the stub,
  because every lookup goes through autofs.
  If the automount itself is absent or failed (for example the start-limit case),
  the stub answers `EACCES`
  and nothing can be created in it.
  The stub is on the NVMe and empty,
  so nothing is hidden by the mount later.
- Reboot:
  `local-fs.target` ordering makes the automount ready before login (`Before=local-fs.target` is a default
  dependency of automount units);
  no process needs the 860 until it touches the path.
  Login-time readers (the user manager reading the symlinked `claude-cpu-watch.service`,
  the hourly kopia job)
  trigger the unlock themselves.
  Hazard to test:
  `mount-860.service` ends with `notify-send` through the user session bus;
  if the user manager is
  itself blocked on the repository path,
  that bus may be unable to start and `notify-send` can wait.
  The fix is `timeout 5` around that call (see the runbook);
  it is a deadlock candidate,
  derived by reading,
  not seen.
- If `mount-860.service` fails:
  three failed attempts end with exit status 1;
  the mount unit fails with a dependency
  error;
  waiters get `ENODEV`;
  the next access retries,
  within the unit's own start limit
  (`StartLimitBurst=5` per 120 s);
  after that the automount fails and the stub (locked) shows `EACCES`
  until `systemctl reset-failed` and a retry.
  Nothing waits without bound.
- Worktree subvolumes,
  reflinks,
  snapshots:
  they stay as subvolumes inside the 860 subvolume;
  `worktree add` snapshots still work (same filesystem;
  the snapshot source is the bind-mounted subvolume root,
  which
  must be probed on the fixture).
  Worktrees on the 860 under `/var/mnt/encrypted/temp/agent` become same-filesystem with main for the first time.
  Worktrees on the NVMe (22) cross filesystems:
  the wrapper's copy fallback applies,
  as it already does for the 71
  on the 860.
- Wrapper:
  the canonical path is unchanged,
  so the 340 instance hashes,
  the 591 state directories,
  the 12 live
  units,
  mise trust links (18700),
  the harness project folder,
  kopia's source path and every `.git` file stay valid.
  The 12 running `cli-git-service` processes hold the NVMe copy open and must be stopped at the cut-over
  (they are socket-activated and start again on the next commit).
- Undo:
  stop and disable the automount,
  `chattr -i` and remove the stub,
  rename the kept NVMe copy back;
  commits made
  on the 860 since the cut-over are carried back by a reverse copy first (runbook,
  "Undo").
- Needs root:
  unit files,
  the drop-in,
  `systemctl`,
  `chattr`,
  the stub,
  and (for the copy) `btrfs send` and
  `btrfs receive`.

### Option A2: plain bind mount unit (no automount) over the same locked stub

- Same as A without the autofs;
  the mount is started by `WantedBy=user@1000.service` or `local-fs.target`.
- Before the mount:
  the stub answers `EACCES`;
  nothing writes to the NVMe.
  Nothing triggers the unlock when a program arrives first,
  so a login-time reader that comes before the mount
  fails and,
  for `claude-cpu-watch`,
  stays failed.
  That is a start-order dependency,
  which the owner rejected.
- A process that holds a handle in the stub before the mount keeps seeing the stub afterwards (a mount does not move
  existing working directories),
  which the locked,
  empty stub makes harmless but not useful.
- Everything else matches A.

### Option A3: mount the 860 subvolume directly at the old path (`What=/dev/mapper/crypt_sda`, `subvol=Monochromatic`)

- Same automount,
  but the mount unit mounts the device instead of binding a path under `/var/mnt/encrypted`.
- It removes the dependency on `/var/mnt/encrypted` being mounted,
  but it mounts the same btrfs a second time with
  different options (`compress`,
  `noatime`);
  how the kernel treats conflicting options on a second mount was not
  probed,
  and it still needs `mount-860.service` to create `/dev/mapper/crypt_sda`.
- Worth a fixture test next to A;
  not recommended before that test.

### Option C: move the canonical path to the 860 and change every reference

- The repository lives at `/var/mnt/encrypted/Monochromatic`;
  the old path is removed or becomes a stub.
- Before the 860 is mounted:
  `/var/mnt/encrypted` is an ordinary NVMe directory,
  so `mkdir --parents` of a path
  under it succeeds on the NVMe and is hidden by the mount later,
  the hazard the owner named.
  Guarding it needs an immutable underlying directory that cannot be set while the 860 is mounted
  (a private mount namespace can reach it;
  see the runbook).
- Reboot and failed unlock:
  every reference fails with "No such file or directory" until the 860 is up,
  with no
  trigger that brings it up;
  start-order dependent.
- Changes:
  340 worktree `.git` and `gitdir` files (`git worktree repair`),
  all 340 instance hashes,
  591 state
  directories orphaned,
  18700 mise trust links,
  the harness project folder,
  kopia's source,
  the units,
  the hooks
  and MCP entries,
  editor workspaces.
- Undo:
  reverse every edit;
  a large manual change set.

### Option B: symbolic link at the old path to the 860

- Before the 860 is mounted:
  a dangling link answers "No such file or directory",
  but a command that creates parents
  through it (`mkdir --parents /var/home/user/Monochromatic/x`) creates `/var/mnt/encrypted/Monochromatic/x` on the
  NVMe,
  hidden by the mount later.
- Canonical path changes for every program that resolves links:
  the wrapper (`canonicalize`),
  mise (hash of the
  canonical path),
  `git` itself.
  Programs that keep the old spelling see a second spelling of one directory;
  the repository already has a record of
  that failure class in `doc/troubleshooting/pnpm-symlinked-store-path.md` (pnpm relative links).
- No ordering mechanism at all;
  undo is `rm` of the link.

### Option D: btrfs device add (the 860 joins the NVMe pool)

- Disqualified.
  `btrfs device add` overwrites an existing filesystem unless forced,
  which destroys the 860's 3.0T of data
  (`never discard user data`).
  The pool would then need the 860 present to mount `/var/home`,
  so a late or failed unlock stops the home
  directory from mounting:
  the strongest possible start-order dependency.
  Undo is a full balance and device removal.

### Send and receive is a copy method, not a path option

It is covered in the section "Copy methods",
together with the alternatives.

## Ranking before the owner's pick (history)

Written 2026-11-07T05:47:00-05:00,
before the pick.

Ranking:
A > A2 > C > B > D,
with A3 held out until its fixture result,
because:

- A over A2:
  both keep the canonical path,
  but only A turns the first access into the trigger for the unlock and
  returns an error after a bounded time,
  so it does not depend on which starts first;
  A2 depends on someone starting
  the mount.
- A2 over C:
  A2 keeps the path,
  the 340 instance hashes,
  18700 trust links,
  the harness folder and kopia's source
  unchanged and fails clearly with `EACCES`;
  C changes every reference and has the unguarded
  `mkdir --parents` hazard.
- C over B:
  both change the canonical path,
  but C changes it everywhere and explicitly,
  so a missed reference fails
  loudly;
  B leaves two spellings of one directory alive,
  which is the pnpm failure class,
  and hides the same hazard
  behind a dangling link.
- B over D:
  B is undone with `rm` and loses nothing;
  D destroys the 860's data and makes the home mount depend on
  the 860.
- A3 sits next to A:
  it needs the fixture result on conflicting mount options before it can be ranked.

### The owner's pick, 2026-11-07

The owner picked option B over the recommended A.
The ranking above stays as history.
The current ranking for the picked design is B first,
because it is the owner's decision,
and the two penalties the first version gave it are answered by measurement:

- "Writes through a dangling link land on the NVMe":
  plain tools (coreutils,
  Python,
  Rust,
  Node,
  `rsync`,
  `git`) all fail through a dangling link;
  only a program that resolves with a missing-tolerant call and then creates the result writes,
  and a root-owned mountpoint refuses that (fixture states 1 and 2).
- "The canonical path changes":
  every consequence is listed with its handling in the section "What the canonical path changes".

Two costs of B remain that A did not have:
replacement of the link by a real folder is not preventable by a mode (fixture state 6),
and every tool needs the shared check (W1).

## Copy methods

### M1: btrfs send and receive with clone sources (needs root)

- `btrfs send` needs read-only snapshots;
  `btrfs-send(8)` says all snapshots in a send must be read-only and a
  clone source must be in exactly the same state on both sides.
- A send of the main subvolume does not include nested subvolumes (btrfs semantics);
  each worktree subvolume needs its
  own send,
  with the already sent main and worktrees as clone sources (`-c`),
  so the receiver can clone shared
  extents instead of writing them again.
- `--compressed-data` (protocol 2) sends compressed extents as they are,
  if both kernels support it.
- The received subvolumes are read-only;
  a writable snapshot of each,
  placed at its final path,
  is instant and keeps
  sharing.
- Reads root-owned files too (the 90 or more unreadable log files).
- Unverified here:
  that the clone sources actually preserve sharing between worktrees that were snapshots or reflink
  copies of main,
  and the run time of the extent lookups;
  the fixture measures both with `btrfs filesystem du` on the
  receiver.

### M2: unprivileged copy plus per-worktree snapshots

- Copy main once with `rsync`,
  then for each worktree create a snapshot of the copy on the 860
  and run `rsync --inplace --no-whole-file` from the old worktree onto it,
  so unchanged files stay shared.
- No root for the data (subvolume create and snapshot on a user-owned top directory are allowed on btrfs),
  but it
  cannot read the root-owned files,
  and whether `rsync` keeps extents shared (modification times,
  in-place writes) is
  unverified.

### M3: plain `cp` or `rsync` of everything

- Cannot fit (scope two in "Free space after each scope").
  Fine for scope one (main only),
  done as root so root-owned files are included.

Ranking for scope three:
M1 > M2 > M3,
because M1 keeps sharing and reads everything,
M2 keeps sharing only if
`rsync` behaves as assumed and skips root-owned files,
M3 does not fit.


## The symlink design

### Layout

- `/var/mnt/encrypted/repository`:
  an ordinary directory on the 860's btrfs top level,
  owner `user:user`,
  mode 0700.
- `/var/mnt/encrypted/repository/Monochromatic`:
  the main checkout,
  a btrfs subvolume with every worktree subvolume nested in it exactly where it is now
  (`.claude/worktrees/<name>`).
- `/var/mnt/encrypted/repository/received-read-only`:
  the read-only copies that `btrfs receive` makes.
  They are the parents of the final increments and stay until the owner removes them;
  they share extents with the live tree,
  so they cost almost no space.
- `/var/home/user/Monochromatic`:
  a symbolic link (absolute target) to `/var/mnt/encrypted/repository/Monochromatic`.
  `/var/home/user` is owned by `user` with mode 0750 (measured),
  so the owner can create and remove the link without root.
- `/var/home/user/repository-move-snapshots`:
  mode 0700 directory on the NVMe that holds the read-only source snapshots while the copy runs.
  They pin NVMe space until deleted,
  and the NVMe mount has `user_subvol_rm_allowed`,
  so the owner deletes them as the user.

### Why a dedicated 0700 directory

- The 860's top level is `user:wheel` mode 777 (measured),
  and the repository root directory is mode 755 (measured),
  so a snapshot of it keeps mode 755.
  Only the parent can carry the restriction,
  and `.git` holds LFS objects,
  the wrapper's records and `rr-cache` even after the credential fix.
- The parent `repository` does not exist on the NVMe.
  A program that resolves the link and then creates the resolved path must first create `repository` under the
  unmounted mountpoint,
  which the mountpoint guard refuses (next section).
- The directory groups the repository with its received snapshots,
  so the 860's top level gains one entry,
  not three.
- Rejected:
  `/var/mnt/encrypted/Monochromatic` directly under the 777 top level.
  It has no restriction that survives a snapshot,
  and a program that makes the missing parent has nothing to refuse it.

### The mountpoint guard

The guard is that the directory `/var/mnt/encrypted` on the NVMe,
the one hidden while the 860 is mounted,
is owned by root,
mode 0755,
and immutable.

- `stat` of `/var/mnt` printed `root:root 755` on the NVMe.
  The directory `/var/mnt/encrypted` itself could not be inspected without root:
  while the 860 is mounted `stat` shows the 860's top level (`user:wheel 777`,
  device 108),
  and no user-level route to the hidden directory exists
  (a rootless bind mount of `/var/mnt` failed with "wrong fs type,
  bad option,
  bad superblock";
  `/sysroot/ostree/deploy/default/var` is a different subvolume from the live `/var`,
  which is `dm-1[/var]` in `findmnt`).
- Root-owned mode 0755 stops every user-level program from creating anything under the unmounted mountpoint
  (fixture result in the next section).
- Immutable (`chattr +i`) additionally stops root-run programs and `chmod`.
  Whether the flag survives the mount is the owner's root test below;
  it is not claimed here.
  The trap:
  `sudo chattr +i /var/mnt/encrypted` while the 860 is mounted sets the flag on the 860's top-level inode,
  not on the hidden NVMe directory.
  The commands below reach the hidden directory through a non-recursive bind mount of `/var`,
  which does not carry the 860 mount with it,
  so the 860 stays mounted and nothing is unmounted.

Owner's root test (R1),
which inspects,
guards and proves the flag in one pass,
bounded:

```bash
# R1: inspect the hidden mountpoint through a non-recursive bind of /var (the 860 stays mounted).
sudo mkdir /run/underlying-var
sudo mount --bind /var /run/underlying-var
stat --format='%U:%G %a %n' /run/underlying-var/mnt/encrypted
lsattr -d /run/underlying-var/mnt/encrypted
ls -A /run/underlying-var/mnt/encrypted
```

Expected:
owner `root:root`,
mode 755,
no flags,
and an empty listing.
If the listing is not empty,
something was written under the mountpoint while the 860 was away;
it is hidden by the mount and must be read and moved by the owner before anything else.
Then:

```bash
# R2: make the hidden directory root-owned, unwritable and immutable.
sudo chown root:root /run/underlying-var/mnt/encrypted
sudo chmod 0755 /run/underlying-var/mnt/encrypted
sudo chattr +i /run/underlying-var/mnt/encrypted
lsattr -d /run/underlying-var/mnt/encrypted
lsattr -d /var/mnt/encrypted
sudo umount /run/underlying-var
sudo rmdir /run/underlying-var
```

The first `lsattr` must show the `i` flag,
the second (the mounted 860) must not.
Whether a mount can be placed over an immutable directory is proven by the next login:
`mount-860.service` runs `mount` at login and notifies on failure.
After that login repeat R1 and expect the `i` flag still set and the 860 mounted.
If the mount fails with the flag set,
run `sudo chattr -i` on the hidden directory through the same bind and keep root-owned 0755 as the guard.

### Fixture results: what a process sees in each state

Fixture:
`podman unshare` (a user namespace in which the owner's uid is root and uids 1 to 65536 map to the subuid range),
with tmpfs mounts that exist only inside that namespace.
Programs ran as namespace uid 1000 through `setpriv`.
The "NVMe" is a tmpfs;
the mountpoint is a directory on it;
the "860" is a second tmpfs (mode 0777 like the real top level) mounted over the mountpoint;
the link sits in a directory that uid 1000 owns,
and points to `<mountpoint>/repository/Monochromatic`.
The fixture script is `/var/home/user/temp/agent/dangling/guard-fixture.ts`
and its last output is `guard-result.txt` in the same folder.
Run at 2026-11-07T06:09:00-05:00 inside `systemd-run --user --scope` (`CPUQuota=100%`,
`MemoryMax=1G`,
`TasksMax=128`,
`IOWeight=10`).

#### State 1: 860 not mounted,
mountpoint not guarded (owned by the user)

- Through a dangling link,
  these all failed and created nothing:
  `mkdir -p L/x/y` (`File exists`),
  `touch L/f`,
  `install -D`,
  `cp -r`,
  `tar -x -C`,
  `ln -s`,
  `rsync` (`mkdir ... failed: File exists`),
  `git init L/r`,
  `git -C L status`,
  Node `mkdirSync(..., {recursive: true})` (`ENOENT`),
  Node `writeFileSync`,
  Python `os.makedirs` (`FileNotFoundError` or `FileExistsError`),
  Rust `create_dir_all` (`File exists (os error 17)`).
  The reason:
  the final component is a link whose target is missing,
  so `mkdir(2)` answers `EEXIST` and every other call answers `ENOENT`.
- Resolve-then-create wrote to the NVMe:
  `mkdir -p "$(realpath -m L)/x"` returned success and created `repository/Monochromatic/x` on the NVMe,
  hidden by the 860 mount later.
  (Later probes in that same state then saw an existing target,
  so the rows after it in the raw output are not independent results.)
  The programs that resolve with a missing-tolerant call are `realpath -m`,
  `readlink -m`,
  and Python `Path.resolve()` (non-strict);
  Node `realpathSync` and Rust `canonicalize` throw on a missing target
  (measured:
  `ENOENT` and `No such file or directory`).
  `git grep` over tracked code outside `doc/` finds no use of `realpath -m`,
  `readlink -f` or `canonicalize-missing`.

#### State 2: 860 not mounted,
mountpoint root-owned 0755

- The same resolve-then-create command failed:
  `mkdir: cannot create directory '.../repository': Permission denied`,
  and the mountpoint stayed empty.
- All the probes of state 1 failed the same way as before;
  nothing was created.
- Not testable at user level:
  `chattr +i` on the mountpoint (`Operation not permitted`;
  a namespace has no `CAP_LINUX_IMMUTABLE` over the host,
  and tmpfs refuses it).
  The owner's root test R1 and R2 covers it.

#### State 3: 860 mounted,
repository directory present

- Every probe succeeded,
  as the repository needs.
  Node,
  Python,
  Rust,
  `rsync`,
  `git init` and `git -C L status` all worked through the link.

#### State 4: 860 mounted but the `repository` directory is gone

- The plain probes failed again (dangling link),
  but `realpath -m` followed by `mkdir -p` created `repository` on the 860,
  because the 860's top level is mode 777.
  A wrong or fresh drive mounted at the mountpoint would show the same.
  This is why each tool checks that `<link>/.git` exists,
  not only that the link resolves.

#### State 5: lazy detach with a holder

- A plain `umount` of the mountpoint refused while a process had its working directory inside:
  `target is busy`.
- After `umount -l`,
  the holder kept writing into the detached mount
  (`ok-on-detached-mount`),
  its `pwd` failed (`getcwd: cannot access parent directories`),
  and new lookups through the link failed as in state 2.
- `mount-860.service` stops with a plain `umount ... 2>/dev/null` followed by `cryptsetup luksClose`
  (read in `/etc/systemd/system/mount-860.service`),
  so a stop while any process holds the repository refuses and says nothing.
  This plan never uses `umount -l` or `--force` on the 860:
  a lazy detach strands holders on a mount nobody can see,
  which loses their writes.
  The holder list in the section "Drain" exists for this.

#### State 6: the link replaced by a real directory

- The guard does not stop this.
  `rm -f L` then `mkdir -p L/x` as the owner of the containing directory succeeded,
  and the new directory sits on the NVMe.
  The link lives in `/var/home/user`,
  which the user owns,
  so no mode,
  sticky bit or flag on a symbolic link prevents its replacement.
- The only defence is that every tool refuses unless `<link>/.git` resolves,
  and that the shared check reports "a real folder replaced the link" when `lstat` of the path is a directory.
  Work item W15.

### Other measurements

- `~/temp` is itself a symbolic link to `/var/mnt/encrypted/temp` (`ls -l`),
  so all agent scratch and 71 worktrees already depend on the 860,
  and today they dangle whenever the 860 is away.
  The design extends an existing pattern;
  it does not introduce a first one.
- Subvolumes on the 860 cannot be deleted by the user:
  the 860 is mounted without `user_subvol_rm_allowed` (`findmnt`),
  and `btrfs subvolume delete` as the user printed `deletion failed with EPERM`.
  Two scratch subvolumes made by that test,
  `/var/home/user/temp/agent/dangling/svfix` and `.../svfix-snap`,
  are left for the owner to delete with `sudo btrfs subvolume delete`.
  Removing a worktree that is a subvolume (242 of the 246 nested worktrees)
  needs that option after the move.
  See open question Q3.
- Mount-time facts:
  the 860 is mounted whole (`subvolid=5`),
  options `compress=zstd:3,noatime,ssd,space_cache=v2`.

## Ordering: nothing depends on the mount

- No unit gets `Requires=`,
  `After=` or `RequiresMountsFor=` on `/var/mnt/encrypted`,
  and no unit is added that orders itself against `mount-860.service`.
  Existing:
  `pcloud.service` has `RequiresMountsFor=/var/mnt/encrypted`;
  it is the owner's existing pattern for pcloud and is not changed by this plan.
- The wrapper's units carry no repository path
  (`/run/user/1000/systemd/user/cli-git@.service` has `ExecStart` in `~/.local/share/monochromatic-dev` and
  `StateDirectory=cli-git/service/%i`,
  read 2026-11-07),
  so they start and run with the 860 away,
  and a request about an unreachable repository is answered with an error.
- Rule for a unit or tool that needs the repository:
  at start it asks the shared check (W1);
  if the answer is "not ready" it fails plainly with the cause and the next use asks again,
  or (for a daemon) it retries on a fixed delay.
  No call waits for the mount.
- `mount-860.service` is left as it is:
  it is `Type=oneshot` with `TimeoutStartSec=infinity`,
  but nothing in the repository's chain waits on it,
  which is why the owner's answer to the unlock timeout question was "N/A".
  It gains two changes in the root steps:
  `user_subvol_rm_allowed` in its mount options (open question Q3),
  and nothing else.

## Work items per tool

The shared contract comes first because every other item calls it.
Each item gives what the tool does today
(measured here,
read from source,
or inferred;
the label is in brackets),
the change,
and the test.

Messages all name the cause in plain words and the next step.
The probe returns one of five answers:

- Ready:
  the path is a symbolic link,
  its target resolves to a directory,
  and `<target>/.git` exists.
- The 860 is not mounted:
  the target is missing and `/var/mnt/encrypted` is not a mount point in `/proc/self/mountinfo`.
  Message:
  "the 860 drive is not mounted yet (/var/mnt/encrypted is empty),
  so the repository at /var/home/user/Monochromatic is
  not there;
  unlock it (the mount-860 service does that at login) and run this again".
- Mounted but the folder is missing:
  the mountpoint is a mount point and the target is missing.
  Message:
  "the 860 drive is mounted but /var/mnt/encrypted/repository/Monochromatic does not exist;
  nothing was written".
- A real folder replaced the link:
  `lstat` of `/var/home/user/Monochromatic` says directory.
  Message:
  "/var/home/user/Monochromatic is a real folder,
  not the link to the 860;
  nothing was written".
- No repository inside:
  the target is a directory without `.git`.

Exit status:
128 for the Git wrapper (Git's own fatal status),
75 (`EX_TEMPFAIL`) for every other tool.
Every call is a few `lstat`,
`stat` and one read of `/proc/self/mountinfo`;
none opens a file inside the target,
creates anything,
or waits.

1.  W1,
    the shared check.
    Today:
    nothing shares one.
    Change:
    a small package with a library and a bin,
    `package/repository-link/check` with bin `repository-link-check`
    (name is a proposal;
    it follows `SGD` and `VGW`),
    installed outside the repository (`~/.local/share/monochromatic-dev/toolchain-global`)
    so the check itself never dangles,
    plus a second bin `repository-link-run <command...>` that checks and then replaces itself with the command.
    The Git wrapper is Rust,
    so it carries the same logic as a function and the same test table.
    Tests:
    one table of fixture states (ready,
    not mounted,
    mounted without folder,
    replaced by a folder,
    no `.git`,
    link to a directory outside the 860)
    run against the library,
    both bins and the Rust copy,
    with adversarial paths (a target with a newline,
    a space,
    `..` segments,
    a link chain).
2.  W2,
    the Git wrapper client (`cli-git-native`,
    `package/git-policy/cli`).
    Today:
    `~/.local/bin/git` is a link to the global launcher in `toolchain-global/launchers`,
    outside the repository,
    so it still runs when the repository is away [measured];
    the repository-scope entry `.cache/cli-git-bin/git` is skipped by `$PATH` [measured with `which -a git`].
    With a scratch link to a missing target:
    `git -C link status`,
    `git -C link/sub status`,
    `git --git-dir=link/.git status` and `git -C link rev-parse --show-toplevel` each ended in 0.012 to 0.092 s with
    `fatal: cannot change to 'link': No such file or directory` or `fatal: not a git repository`,
    no hang,
    no new file in `~/.local/state/cli-git/service` [measured 2026-11-07T06:01:00-05:00].
    A worktree whose `.git` file names the missing repository:
    `status` and `log` answered `fatal: gitfile does not point to a valid repository`,
    and `commit` answered `fatal: cannot change to 'wt1': No such file or directory`
    for a folder that exists [measured];
    that message is wrong for a commit.
    `instance_of` hashes the path as given when `canonicalize` fails
    (`service_instance.rs`,
    function `instance_of`) [read from source].
    Change:
    run W1 first for any target path that names the repository link;
    print the plain message with status 128;
    make `instance_of` return an error when `canonicalize` fails,
    so no instance,
    journal directory or socket is ever named for an unreachable path,
    and make the commit path report the real cause.
    Tests:
    the dangling fixture asserts the message,
    status 128,
    and no new directory under the service state folder or the runtime folder;
    a worktree-through-missing-link fixture asserts the commit message names the repository,
    not the working folder.
3.  W3,
    the commit service (`cli-git-service`).
    Today:
    started only by the first connection to a socket the wrapper wrote;
    its unit names no repository path;
    it holds a plan for failed pushes in `push-retries` with doubling pauses [read from source and `README.md`].
    What it does when the repository vanishes mid-request is not measured.
    Change:
    a request whose Git directory is unreachable is answered with the plain message and left for the client to retry
    on its next command;
    a push plan whose directory is unreachable keeps its doubling pause and its record and never runs on another path.
    Tests:
    a service fixture whose repository folder is renamed away between two requests,
    asserting an answer within the existing ceiling,
    no spin (the count of attempts across a fixed interval),
    and the plan record still present after the folder returns.
4.  W4,
    `mpm-shadow` and the launchers in `.cache/toolchain-launchers`.
    Today:
    a launcher is a hard link of the `mpm-shadow` binary plus a `<name>.launch` record;
    `run_launcher` reads the lock file with `.ok()` and then replaces itself with the real binary,
    so an unreadable lock drops the version warning silently and still runs
    (`package/mpm-shadow/cli/src/toolchain/launcher.rs`,
    function `run_launcher`) [read from source].
    `ensure_launcher` calls `fs::create_dir_all` on the launcher folder;
    Rust `create_dir_all` through a dangling link returned `File exists (os error 17)` and created nothing
    [measured].
    The launchers themselves live inside the repository,
    so when it is away the program is not found at all [measured with `which -a`].
    Change:
    when the lock file is unreadable and its path is under the repository link,
    print one line naming the 860 (or the cause W1 returns) and still run the real binary
    (the global launcher is the correct fallback,
    so this must not fail);
    when `ensure_launcher` is asked to install under the link and W1 is not ready,
    fail with the plain message before any write.
    Tests:
    `launcher_tests.rs` gains an unreadable-lock case asserting the line and the exec;
    an install-through-dangling-link case asserting nothing was created.
5.  W5,
    `cargo-shadow`,
    `pnpm-shadow`,
    `search-shadow`.
    Today:
    same launcher scheme;
    each program runs with its working folder inside the repository,
    so a dangling start cannot happen;
    the failure is mid-run (the 860 goes away under a running cargo or pnpm),
    which the operating system reports as `ENOENT` or `EIO` [inferred,
    not measured].
    With the repository away,
    `$PATH` falls through to other programs [measured with `which -a`]:
    `cargo` and `pnpm` to the global `exposure` links (for `cargo`,
    an ELF),
    `rg` to a mise shim,
    and `find`,
    `du`,
    `grep` to `/usr/bin`.
    These are the unshadowed tools,
    so the shadows' policies (lock trust for `cargo`,
    the search limits for `rg`) are skipped silently outside the repository.
    Change:
    none in the programs beyond W1 at start for an explicit repository path;
    the silent fall-through is open question Q1.
    Tests:
    a launcher fixture with a missing `.launch` target asserting a plain message.
6.  W6,
    Claude Code hooks and the status line in `~/.claude/settings.json`.
    Today:
    `PreToolUse` on `Bash` runs `node /var/home/user/Monochromatic/package/claude-code-plugin/bash-output-filter/bundle/node/index.mjs`,
    and the status line runs `node .../statusline/src/statusline.ts`.
    Node with a missing script printed a stack with `Error: Cannot find module` and exited 1 [measured].
    Claude Code treats an exit code other than 0 or 2 from a hook as a non-blocking error and lets the action go ahead;
    "when the script path doesn't exist or isn't executable,
    the shell exits with a code like 127";
    `onFailure` set to `block` makes a failing hook block
    ([hooks reference][claude-hooks],
    sections on exit codes and on hook failures,
    fetched 2026-11-07).
    Change:
    the command becomes `repository-link-run node <script>`,
    so the message is plain and the status is 75 (non-blocking);
    a hook that enforces a rule (the `guardrail` package under `package/claude-code-plugin`,
    if it is configured) sets
    `onFailure: block`
    so a missing script blocks instead of letting the action through.
    The output filter only trims text,
    so it stays non-blocking.
    Tests:
    a hook fixture with a missing script asserts the status and the first line.
7.  W7,
    MCP servers:
    Claude's `mvm` and `search-fetch`
    (`~/.claude.json`,
    `node .../package/mcp/mvm/dist/...` and `.../package/search-fetch/mcp/dist/...`)
    and Codex's server with `--cd /home/user/Monochromatic` (`~/.codex/config.toml`).
    Today:
    a missing script fails the server start with the Node stack [measured for Node with a missing script];
    Codex's `--cd` to a missing folder is inferred to fail the same way [not measured].
    Change:
    commands go through `repository-link-run`;
    Codex's `--cd` stays and the server's own error is accepted.
    Tests:
    start each configured command with the link dangling and assert the first line.
8.  W8,
    `claude-cpu-watch.service`.
    Today:
    the unit file in `~/.config/systemd/user` is a symbolic link into the repository
    (`ls -l` printed `-> /home/user/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`),
    so the user manager reads through the link while loading units;
    if the repository is away at manager start or at `daemon-reload` the unit is not loaded and nothing retries
    [read from the earlier measurement of 2026-11-07T05:47:00-05:00;
    a transient probe unit meant to re-measure the start limit was refused by the tool guard and was not retried].
    With the unit loaded and `cpu-watch.ts` missing,
    `Restart=always` with `RestartSec=2` and the default start limit
    (5 starts in 10 s,
    read in the earlier run with `systemctl --user show`)
    ends in a failed unit that stays failed.
    Change:
    the unit file in `~/.config/systemd/user` becomes a regular copy
    (the repository keeps the original;
    a check compares the two);
    its start runs `ExecStartPre=repository-link-check` so a dangling path fails with status 75 and the plain message,
    with `Restart=on-failure`,
    `RestartSec=30`,
    and `StartLimitIntervalSec=0`
    so it never ends in a permanent failure and retries every thirty seconds until the mount is there.
    Tests:
    `systemd-analyze verify` on the unit;
    a transient copy with the link dangling,
    run by the owner,
    asserting it stays in `activating (auto-restart)`,
    then reaches `active (running)` after the target appears.
    Alternatives are open question Q2.
9.  W9,
    `~/.local/bin/spawn-claude` and `spawn-pi`.
    Today:
    symbolic links into the repository;
    a shell reports `No such file or directory` with no cause [inferred from the link form;
    the shell message for a missing command target is not repository-specific].
    Change:
    replace each with a regular two-line script outside the repository that runs W1 and then the program,
    so the message is plain.
    Tests:
    run each with the link dangling.
10. W10,
    `env-local-json` (`ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json` in `~/.bashrc`).
    Today:
    the reader throws a typed error naming the path (`EnvLocalJsonPathError`,
    `EnvLocalJsonReadError`) [read from source];
    the writer writes a temporary file beside the store and renames it,
    and creates no directories [read from source],
    so a dangling path fails without writing.
    Change:
    the error text gains the cause from W1 when the path is under the repository link.
    Tests:
    the reader and writer unit tests gain a dangling-path case.
11. W11,
    `mount-860.service`.
    Today:
    stops with a plain `umount`,
    which refuses with a held mount [measured in the fixture],
    and `cryptsetup luksClose`;
    both silent (`2>/dev/null`).
    Change:
    none required.
    Optional,
    owner's file:
    list holders in the stop path with a message,
    and never `umount -l`.
    Tests:
    stop with a holder (a shell with its working folder in the repository) asserting the unit stays active.
12. W12,
    `kopia-snapshot-all.service`.
    Today:
    the source stays `user@bazzite:/home/user/Monochromatic`.
    kopia resolves a symbolic-link source before reading it:
    `getLocalFSEntry` calls `resolveSymlink` and returns an error if it fails
    (`cli/config.go`,
    line 115 at blob `b1abe8c3`,
    read through `gh api` on 2026-11-07),
    and `getContentToSnapshot` records the source path as given.
    So the source identity is unchanged,
    the data comes from the 860,
    and a dangling link is an error for that source before any upload;
    no empty snapshot is made [read from source].
    `snapshot create --all` reports the error and the unit fails for that hour [inferred from the code path].
    Change:
    none.
    The first snapshot after the move is expected to re-read files whose inode changed;
    kopia compares by path,
    size and modification time,
    so btrfs send and receive (which keeps modification times) should keep it incremental [inferred,
    check the first run's upload size].
    Tests:
    a post-cut-over `kopia snapshot create` of the repository source,
    comparing uploaded bytes with the previous snapshot.
13. W13,
    other readers:
    `podman` containers in state `created` with bind mounts of worktree paths (start fails with a missing source;
    after the move the bind source is the link spelling,
    which resolves when mounted);
    editors (JetBrains,
    Helium,
    Lapce,
    Codex,
    Emdash,
    Polypane,
    t3code),
    which only read;
    terminals started with the repository as working folder,
    which fall back to home;
    `mise` tasks,
    which run inside the repository only.
    Change:
    none.
    Tests:
    after cut-over start one `created` container and open the repository in the IDE through the link.
14. W14,
    one spelling for tools that compute relative paths.
    Today and result:
    section "What the canonical path changes".
    Tests:
    that section's pnpm and mise checks.
15. W15,
    the link itself.
    Today:
    nothing watches it;
    the fixture's state 6 shows a real folder can replace it.
    Change:
    W1 reports a real folder at the link path;
    the wrapper's `doctor` prints the same line;
    the cut-over checklist runs `repository-link-check` first.
    Tests:
    state 6 of the fixture table.

[claude-hooks]: https://code.claude.com/docs/en/hooks

## What the canonical path changes

A process that resolves the link sees `/var/mnt/encrypted/repository/Monochromatic`.
Measured with a shell inside a linked folder (fixture,
2026-11-07T06:19:00-05:00):
`pwd` printed the link spelling,
`pwd -P` printed the physical one,
and Node printed `process.cwd()` as the physical path and `process.env.PWD` as the link spelling.
That pair is the source of every two-spelling effect below.

- Commit-service instance hashes.
  `instance_of` is FNV-1a over `canonicalize(git_dir)`,
  so every worktree's name changes (one per worktree:
  340 at the last count,
  the main `.git` and 339 administrative directories).
  Old state:
  591 folders under `~/.local/state/cli-git/service` (595 when counted again at 2026-11-07T06:03:00-05:00,
  other sessions keep committing),
  3213 `.done` records and no other file,
  `busy/` and `push-retries/` empty everywhere at that count.
  Handling:
  drain first (section "Drain");
  stop the old instances;
  leave the old folders in place as the finished-commit journal (rollback makes them valid again);
  new commits create new instances on first use.
  Hazard:
  a non-empty `push-retries/` at cut-over is a plan for a push that would resume under the old name and never run,
  so the drain refuses to proceed while any exists.
  The wrapper's records inside `.git` (`monochromatic-git-policy-cli`,
  `...-worktree-copy`,
  `cli-git-transactions`,
  `cli-git-captures`) move with `.git` in the copy.
  `cli-git-transactions` holds 3 `.pending` folders (dated 2026-10-03,
  2026-10-04 and 2026-11-01);
  they are carried unchanged and listed at the drain.
- mise trust.
  Trust links are keyed by the canonical path (source read in the first version);
  the 18700 links for the old path stay as stale files.
  Measured in a fixture with `MISE_STATE_DIR` and the other mise folders set to scratch:
  `mise trust` run from a link spelling recorded the physical path;
  `mise trust --all` from the main checkout trusted the root file and two nested package files;
  a linked worktree then showed the same files as trusted (`mise trust --show`).
  The manual says the same ("Trusting a config file in a repository's main checkout also trusts the same path in its
  git worktrees"),
  and `mise run`,
  `mise install` and `mise exec` trust their active config on their own.
  Handling:
  after cut-over run `mise trust --all` once from the repository,
  then `mise trust --show` in the root and in one package and one worktree.
  `trust --all` skips hidden folders and `node_modules`,
  so a config under a hidden folder is trusted by its first `mise run`.
  Re-trusting is through mise itself;
  nothing writes into `~/.local/state/mise/trusted-configs` by hand.
- pnpm.
  `doc/troubleshooting/pnpm-symlinked-store-path.md` records pnpm 12.16.1 computing relative links from a symlinked
  spelling.
  Fixture (pnpm 12.16.1,
  `virtualStoreType: global`,
  a project reached through a link three folders deep,
  the store beside the link):
  an install from a logical working folder,
  `--dir <link spelling>` from elsewhere,
  and `--dir <physical>` all wrote the same eleven-`..` link text and `require('dep-a')` worked in each.
  pnpm used the physical working folder in all three.
  That did not reproduce the failure for a symlinked project root.
  Handling:
  the configuration keeps one spelling,
  the link `/var/home/user/Monochromatic` (not the third spelling `/home/user/Monochromatic`);
  after cut-over run `pnpm install` in a fresh worktree and then
  `find node_modules -xtype l` in it,
  which must print nothing;
  the existing store setting `${HOME}/.local/share/pnpm/store` is unchanged.
  Not measured:
  a real install in the real tree (3 GiB).
- Worktree `.git` and `gitdir` files.
  340 absolute paths spelled `/var/home/user/Monochromatic/...` resolve through the link,
  so the copy works without `git worktree repair`.
  Measured (scratch repository):
  `git worktree repair` run from a linked spelling reported ".git file incorrect" and rewrote the worktree's `.git`
  file from the link spelling to the physical one,
  and after a rename of the main folder `repair <worktree path>` fixed a broken pair.
  Handling:
  run `git worktree repair` once after cut-over to make all 340 pairs physical;
  it is optional,
  but it removes the third spelling from every `gitdir` file and from `git worktree list`.
  Git writes the physical spelling for every new `worktree add`.
- Harness project folders.
  Claude Code keys `~/.claude/projects/<folder>` by the working directory;
  the existing folders `-var-mnt-encrypted-temp-agent-...` for sessions started from `~/temp/...` prove it uses the
  physical path (`~/temp` is a link),
  so new sessions in the repository key as `-var-mnt-encrypted-repository-Monochromatic`.
  The old folder `-var-home-user-Monochromatic` holds the history (10716 entries).
  `~/.claude.json` has a `projects` entry for `/var/home/user/Monochromatic` with trust and MCP settings;
  `~/.pi/agent/sessions/--var-home-user-Monochromatic--` is keyed the same way.
  Handling,
  while no session is running:
  rename the old Claude folder to the new name,
  copy the `projects` entry in `~/.claude.json` to the new key with a scratch `.ts` and check by key names only,
  and rename the Pi folder likewise.
  Which spelling the harness uses is confirmed by starting one session and seeing which folder gets the new `.jsonl`.
  Open question Q5.
- kopia.
  The source path is unchanged (W12).
- Editor workspaces.
  JetBrains,
  Helium,
  Lapce and others keep the old spelling;
  it resolves through the link.
  Open the project through the link,
  and expect indexes keyed by whichever spelling each IDE is given.
- Containers.
  Bind sources recorded as the link spelling resolve when mounted;
  checked with W13.
- Permission rules.
  `.claude/settings.local.json` has 134 occurrences of the path,
  matching command text and path patterns.
  Whether Claude Code matches path rules against the link spelling or the physical one is not measured;
  the first session after cut-over shows it as new permission prompts.
- Files that name the path inside tracked docs (191 files,
  420 occurrences) stay as written (rule `WR6`).

## Drain

Goal:
nothing changes in the repository between the final snapshot and the swap,
and nothing is lost.
Agents are not killed.

1.  List the holders.
    The user can see only its own processes:

    ```bash
    # Holders by working folder and open file, repository and every nested worktree.
    find /proc -maxdepth 3 \( -name cwd -o -path '/proc/*/fd/*' \) -lname '/var/home/user/Monochromatic*' -printf '%p %l\n'
    ```

    Processes of other users (rootless containers run under subuids) are listed by root with the same command.
    Expect:
    the terminals,
    `claude` sessions,
    MCP servers,
    the Codex server,
    the IDE,
    and `rust-*` runs (30 processes with the repository as working folder at 2026-11-07T05:47:00-05:00).
2.  Ask each session to finish,
    with a deadline of the owner's choosing.
    At the deadline run the command again.
    If a holder remains,
    the cut-over does not happen and the holder list is the report.
    Nothing is signalled.
3.  List uncommitted work in every worktree,
    before and after the move.
    The script is `/var/home/user/temp/agent/dangling/list-uncommitted-work.ts`;
    it runs `/usr/bin/git --no-optional-locks -C <worktree> status --porcelain=v1 --untracked-files=all` for each line of
    `git worktree list --porcelain`,
    counts changed and untracked files,
    reports an unfinished merge,
    rebase,
    cherry-pick,
    revert or bisect (files in the Git directory) and a held `index.lock`,
    and writes one tab-separated row per worktree.
    `--no-optional-locks` means it writes nothing.
    Run:

    ```bash
    systemd-run --user --scope --property=CPUQuota=100% --property=MemoryMax=1G --property=TasksMax=64 --property=IOWeight=10 \
      timeout 3000 node /var/home/user/temp/agent/dangling/list-uncommitted-work.ts \
      /var/home/user/Monochromatic /var/home/user/temp/agent/dangling/uncommitted-before.tsv 180
    ```

    The copy carries uncommitted files and the old tree is kept,
    so nothing is discarded.
    The list exists for two checks:
    any row with an unfinished operation or a held `index.lock` is settled by its owner first,
    and after the final pass the same script on the 860 tree must print the same table
    (`diff` of the two files is empty).
    Measured run:
    Run from 2026-11-07T06:15:00-05:00 to 2026-11-07T06:26:00-05:00 in the bounded scope above
    (the same script and bounds):
    340 worktrees listed and none missing or failing.
    44 have something a move would strand:
    32 of the 246 nested worktrees,
    7 of the 71 on the 860,
    3 of the 6 reached through `~/temp/agent`,
    and 2 of the 16 under `~/worktrees`;
    the main checkout is clean.
    In total 350 changed and 5188 untracked files.
    No worktree has an unfinished merge,
    rebase,
    cherry-pick,
    revert or bisect,
    and none has a held `index.lock`.
    The largest is 782 files in
    `/var/home/user/temp/agent/music-player-theme-compose-prototype` (on the 860,
    so it does not move).
    The table is `/var/home/user/temp/agent/dangling/real-uncommitted.tsv`.
    Only the 32 dirty nested worktrees move;
    the rest stay where they are.
    This is a snapshot:
    other sessions keep working,
    so the script is run again after the drain.
4.  Wrapper services.
    For every loaded `cli-git@*.service` (15 at 2026-11-07T06:03:00-05:00),
    its state folder `~/.local/state/cli-git/service/<instance>` must have no file in `busy/` and none in
    `push-retries/`,
    and `git monochromatic-git-policy-cli service status` must answer.
    Also list `.git/cli-git-transactions/*.pending`.
    The check is bounded:
    repeat at a fixed interval for the owner's deadline,
    then stop and report the branches named in any `push-retries/` file;
    for those the owner pushes by hand before the swap.
5.  Stop the user units that run from the repository
    (`systemctl --user stop claude-cpu-watch.service` and the `cli-git@*` sockets and services).
    They start again after the swap.
    The services are socket-activated,
    and the wrapper never starts itself at login.

## The copy

Method M1 (send and receive with clone sources,
section "Copy methods") in two passes,
as root.
Footprint to expect on the 860:
at most 263.6 GiB (section "Whole-set figure"),
leaving at least about 398 GiB free on 662G.
Compression (`zstd:3`) makes the on-disk footprint smaller.

The generator `/var/home/user/temp/agent/dangling/print-move-commands.ts` prints the commands and runs none.
It reads `git worktree list --porcelain`,
keeps the worktrees under the main folder that are subvolume roots (inode 256;
242 of the 246 nested ones at 2026-11-07T06:21:00-05:00),
and writes a manifest of names.
The 4 plain-directory worktrees travel inside the main subvolume's send.
The 16 worktrees under `~/worktrees` (NVMe) and the 77 on the 860 (71 under `/var/mnt/encrypted/temp/agent` and 6 reached
through `~/temp/agent`,
which is a link to the same place) are outside the tree and do not move.

```bash
# On the NVMe, as the user (0700 folders), then on the 860 as root.
mkdir -m 0700 /var/home/user/repository-move-snapshots
sudo mkdir -m 0700 /var/mnt/encrypted/repository
sudo chown user:user /var/mnt/encrypted/repository
mkdir -m 0700 /var/mnt/encrypted/repository/received-read-only
```

```bash
# Print the first-pass and final-pass commands to files the owner reads before running anything.
node /var/home/user/temp/agent/dangling/print-move-commands.ts \
  /var/home/user/Monochromatic /var/home/user/repository-move-snapshots \
  /var/mnt/encrypted/repository/received-read-only /var/mnt/encrypted/repository/Monochromatic \
  /var/home/user/temp/agent/dangling/move-manifest.tsv --pass=first > pass-first.txt
```

Worked example,
the first pass for the main checkout and the first worktree (generator output,
2026-11-07T06:21:00-05:00):

```bash
# main /var/home/user/Monochromatic
btrfs subvolume snapshot -r /var/home/user/Monochromatic /var/home/user/repository-move-snapshots/s000
systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10 \
  sh -c 'btrfs send /var/home/user/repository-move-snapshots/s000 | btrfs receive /var/mnt/encrypted/repository/received-read-only'
btrfs subvolume show /var/mnt/encrypted/repository/received-read-only/s000 > /dev/null && echo received s000
# worktree /var/home/user/Monochromatic/.claude/worktrees/add-explains-slow-route
btrfs subvolume snapshot -r /var/home/user/Monochromatic/.claude/worktrees/add-explains-slow-route /var/home/user/repository-move-snapshots/s001
systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10 \
  sh -c 'btrfs send -c /var/home/user/repository-move-snapshots/s000 /var/home/user/repository-move-snapshots/s001 | btrfs receive /var/mnt/encrypted/repository/received-read-only'
btrfs subvolume show /var/mnt/encrypted/repository/received-read-only/s001 > /dev/null && echo received s001
```

(The generator prints each `systemd-run` and its `sh -c` on one line;
they are shown here on two for reading.)

The final pass,
after the drain,
for the same two (generator output,
`--pass=final`):

```bash
btrfs subvolume snapshot -r /var/home/user/Monochromatic /var/home/user/repository-move-snapshots/f000
systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10 \
  sh -c 'btrfs send -p /var/home/user/repository-move-snapshots/s000 /var/home/user/repository-move-snapshots/f000 | btrfs receive /var/mnt/encrypted/repository/received-read-only'
btrfs subvolume snapshot /var/mnt/encrypted/repository/received-read-only/f000 /var/mnt/encrypted/repository/Monochromatic
# then for each worktree, in order:
btrfs subvolume snapshot -r /var/home/user/Monochromatic/.claude/worktrees/add-explains-slow-route /var/home/user/repository-move-snapshots/f001
systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10 \
  sh -c 'btrfs send -p /var/home/user/repository-move-snapshots/s001 -c /var/home/user/repository-move-snapshots/f000 /var/home/user/repository-move-snapshots/f001 | btrfs receive /var/mnt/encrypted/repository/received-read-only'
rmdir /var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/add-explains-slow-route
btrfs subvolume snapshot /var/mnt/encrypted/repository/received-read-only/f001 /var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/add-explains-slow-route
```

The generator's `--clone-sources=main` lists only the main snapshot as clone source instead of all earlier ones;
the default `all` shares most but costs more per send.
The first real worktree send is timed under both and the choice is the owner's (Q6).

The bounds (`CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=32`,
`IOWeight=10`) are sized to one stream and chosen by the plan author,
not measured.
No synthetic load is generated;
the copy is real work.

### Fixture test before the real run (root, throwaway loop files)

Four facts are unverified and decide the copy:
that `-c` keeps sharing between sibling worktrees,
that an incremental `-p` send with clone sources works,
that the empty stub of a nested subvolume in a received parent can be removed with `rmdir`,
and that `btrfs receive` checks the stream (a corrupted stream must fail).

```bash
# F1: two throwaway btrfs files on the 860's scratch folder, bounded; removed at the end.
mkdir -p /var/home/user/temp/agent/send-fixture
truncate --size=3G /var/home/user/temp/agent/send-fixture/source.img
truncate --size=3G /var/home/user/temp/agent/send-fixture/dest.img
sudo mkfs.btrfs --quiet /var/home/user/temp/agent/send-fixture/source.img
sudo mkfs.btrfs --quiet /var/home/user/temp/agent/send-fixture/dest.img
sudo mkdir /run/send-fixture-source /run/send-fixture-dest
sudo mount -o loop /var/home/user/temp/agent/send-fixture/source.img /run/send-fixture-source
sudo mount -o loop /var/home/user/temp/agent/send-fixture/dest.img /run/send-fixture-dest
```

```bash
# F2: a main subvolume with a 300 MiB file, two nested worktree subvolumes that are snapshots of it, one edited.
sudo btrfs subvolume create /run/send-fixture-source/main
sudo dd if=/dev/urandom of=/run/send-fixture-source/main/big bs=1M count=300 status=none
sudo mkdir /run/send-fixture-source/main/wt
sudo btrfs subvolume snapshot /run/send-fixture-source/main /run/send-fixture-source/main/wt/a
sudo btrfs subvolume snapshot /run/send-fixture-source/main /run/send-fixture-source/main/wt/b
sudo dd if=/dev/urandom of=/run/send-fixture-source/main/wt/b/extra bs=1M count=5 status=none
sudo mkdir /run/send-fixture-source/snap /run/send-fixture-dest/recv
sudo btrfs subvolume snapshot -r /run/send-fixture-source/main /run/send-fixture-source/snap/s000
sudo btrfs subvolume snapshot -r /run/send-fixture-source/main/wt/a /run/send-fixture-source/snap/s001
sudo btrfs subvolume snapshot -r /run/send-fixture-source/main/wt/b /run/send-fixture-source/snap/s002
```

```bash
# F3: send main, then a with -c main, then b with -c main -c a; each into the destination.
sudo sh -c 'btrfs send /run/send-fixture-source/snap/s000 | btrfs receive /run/send-fixture-dest/recv'
sudo sh -c 'btrfs send -c /run/send-fixture-source/snap/s000 /run/send-fixture-source/snap/s001 | btrfs receive /run/send-fixture-dest/recv'
sudo sh -c 'btrfs send -c /run/send-fixture-source/snap/s000 -c /run/send-fixture-source/snap/s001 /run/send-fixture-source/snap/s002 | btrfs receive /run/send-fixture-dest/recv'
sudo btrfs filesystem du --summarize /run/send-fixture-dest/recv/s000 /run/send-fixture-dest/recv/s001 /run/send-fixture-dest/recv/s002
sudo btrfs filesystem du --summarize /run/send-fixture-dest/recv
df --block-size=1M /run/send-fixture-dest
```

Expected:
the destination uses about 305 MiB,
not about 900,
and the last `du` shows `Set shared` near 300 MiB.
If it uses about 900 MiB the clone sources did not keep sharing and the method is wrong for this repository.

```bash
# F4: stub removal and the final placement.
sudo btrfs subvolume snapshot /run/send-fixture-dest/recv/s000 /run/send-fixture-dest/final
ls -la /run/send-fixture-dest/final/wt
sudo rmdir /run/send-fixture-dest/final/wt/a
sudo btrfs subvolume snapshot /run/send-fixture-dest/recv/s001 /run/send-fixture-dest/final/wt/a
sudo btrfs filesystem du --summarize /run/send-fixture-dest/final
```

Expected:
`wt/a` is an empty directory in the received parent,
`rmdir` succeeds,
and the snapshot places the worktree there.
If `rmdir` or the snapshot fails,
placement changes to `mv` of a writable snapshot into place and the generator changes with it.

```bash
# F5: an incremental send with a parent, and a corrupted stream.
sudo btrfs subvolume snapshot -r /run/send-fixture-source/main /run/send-fixture-source/snap/f000
sudo sh -c 'btrfs send -p /run/send-fixture-source/snap/s000 /run/send-fixture-source/snap/f000 | btrfs receive /run/send-fixture-dest/recv'
sudo sh -c 'btrfs send /run/send-fixture-source/snap/s001 > /run/send-fixture-dest/stream.bin'
sudo sh -c 'printf "X" | dd of=/run/send-fixture-dest/stream.bin bs=1 seek=100000 conv=notrunc status=none'
sudo sh -c 'btrfs receive /run/send-fixture-dest/recv-bad < /run/send-fixture-dest/stream.bin'
```

The last receive must fail with a checksum error;
`/run/send-fixture-dest/recv-bad` is made first with `sudo mkdir`.
Clean up:
`sudo umount` both,
`sudo rmdir` both folders,
delete the two image files.

### Verification of the real copy

After every send the generator's `btrfs subvolume show ... && echo received` line proves the received subvolume
exists;
the stream is checksummed by `btrfs receive` (F5).
After the final pass:

1.  `git fsck --full` on the 860 main (bounded scope;
    it is long and reads every object).
2.  `git for-each-ref | sha256sum` and `git worktree list --porcelain | sha256sum` on the 860 tree equal the values
    recorded from the NVMe tree before the drain,
    except that `worktree list` shows the same paths through the link.
3.  File counts equal for main and for a sample of worktrees
    (the six largest and six smallest by the per-worktree `du` list,
    plus every worktree with a non-clean row in `uncommitted-before.tsv`):
    `find <snapshot> -xdev -type f | wc -l` against the same on the received copy.
4.  A content check of the same sample:
    `rsync --archive --checksum --dry-run --itemize-changes --delete <ro snapshot>/ <final>/` run as root prints nothing
    (this reads both sides in full;
    run it in its own scope,
    one worktree at a time).
5.  Sharing kept:
    `sudo btrfs filesystem du --summarize --raw /var/mnt/encrypted/repository/Monochromatic` shows exclusive plus
    set shared close to 283010162688 bytes (263.6 GiB,
    the figure from the NVMe) and `Total` close to 4115920404480.
    As root the 126 unreadable files are counted too,
    so the figures are slightly larger than the user-level ones.
    A footprint much larger than 263.6 GiB means sharing was lost and the swap does not happen.
6.  `list-uncommitted-work.ts` on the 860 tree prints the same table as `uncommitted-before.tsv`.
7.  Root-owned files survived:
    `sudo find /var/mnt/encrypted/repository/Monochromatic -user root -name '*.log.jsonl' | wc -l` is not zero
    in the two worktrees that had them.

Any difference stops the runbook with the old tree untouched.

## Cut-over

Until step 8 the NVMe tree is still the repository,
so steps 1 to 7 need no undo for the repository itself.

1.  Commit and push everything that matters (rule `RBT`).
2.  Root steps that need no quiet:
    R1 and R2 (mountpoint guard),
    the `repository` folder and the other folders above,
    and the mount option:

    ```bash
    # Q3 if accepted: allow the user to delete subvolumes on the 860 now, and at every mount from the next login.
    sudo mount -o remount,user_subvol_rm_allowed /var/mnt/encrypted
    findmnt --target /var/mnt/encrypted --output OPTIONS
    sudo systemctl edit --full mount-860.service
    ```

    In the unit's `mount -o compress=zstd:3,noatime` add `,user_subvol_rm_allowed`.
3.  Install W1 and the work items W2 to W10 (they are code;
    they land through the repository as normal commits and a toolchain update).
    The cut-over does not depend on them,
    but the plan's guarantees do.
4.  Replace the symlinked unit:
    copy `package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service` over
    `~/.config/systemd/user/claude-cpu-watch.service` (remove the link first),
    and apply W8's edits,
    then `systemctl --user daemon-reload`.
5.  Pass one of the copy (repository keeps running).
6.  Drain (steps 1 to 5 of the section "Drain").
7.  Final pass and verification.
8.  Swap,
    as the user:

    ```bash
    mv /var/home/user/Monochromatic /var/home/user/Monochromatic.nvme-before-860-2026-11-07T00-00-00-05-00
    ln -s /var/mnt/encrypted/repository/Monochromatic /var/home/user/Monochromatic
    readlink -f /var/home/user/Monochromatic
    ```

    Replace the timestamp with the real one.
    The rename is within one filesystem and atomic;
    the two commands leave a short window with no path,
    which is why the drain comes first.
    The old tree is the kept copy.
    Its worktrees' `.git` files still name `/var/home/user/Monochromatic/.git`,
    which is now the 860 tree,
    so the old tree is cold storage,
    not a second working repository.
9.  Harness and trust,
    with no session running:
    `mise trust --all` from the repository,
    rename the harness project folders and copy the `~/.claude.json` entry (section "What the canonical path
    changes"),
    `git worktree repair` from the repository.
10. Start `claude-cpu-watch.service`,
    and make one commit through the wrapper in a throwaway worktree (`git worktree add` creates a snapshot on the
    860).
11. Checks:
    `repository-link-check` prints ready;
    `git -C /var/home/user/Monochromatic status`;
    `git worktree list` has the same number of entries as recorded;
    `systemctl --user is-active claude-cpu-watch.service`;
    `mise trust --show`;
    one `created` container starts;
    the hourly kopia job's first run uploads about as much as a normal increment;
    a fresh worktree's `pnpm install` and `find node_modules -xtype l` (nothing);
    the harness lists the old sessions.
12. Reboot test after step 11 (commit and push first):
    log in and wait for `mount-860.service`;
    with the 860 still locked (rename the key path in a drop-in once) open a terminal in the repository and run `git
    status`,
    which must print the plain message in well under a second,
    and `systemctl --user status claude-cpu-watch.service` must show `activating (auto-restart)`;
    restore the key path and confirm the unit becomes active within one retry delay.

## Rollback

- Before step 8 of the cut-over:
  nothing to undo.
  Remove the received copies later (they cost almost nothing,
  see the layout).
- After step 8 and before any commit on the 860 tree:
  stop the user units that use the repository,
  then `rm /var/home/user/Monochromatic` (it is a link;
  `rm` of a link removes only the link) and
  `mv /var/home/user/Monochromatic.nvme-before-860-<timestamp> /var/home/user/Monochromatic`.
  The old instance hashes,
  the old trust links,
  the old harness keys (rename them back) and the old state folders are valid again,
  because the old canonical path is back.
  The new instance folders and new trust links become stale files and harm nothing.
- After commits on the 860 tree:
  first carry the work back.
  In the old tree run
  `git fetch /var/mnt/encrypted/repository/Monochromatic '+refs/*:refs/from-860/*'`
  for committed work (worktree HEADs and indexes are not refs;
  they are not carried by `fetch`).
  For uncommitted work and worktree state use the reverse of the copy:
  the generator with source and destination swapped and `--pass=first` into a new folder on the NVMe,
  or `rsync` of the named worktrees as root.
  Then do the previous bullet.
- The old tree stays until the owner removes it
  (`btrfs subvolume delete` of each nested subvolume then the root;
  the NVMe mount allows it as the user).
  Removal is read-first:
  compare against `uncommitted-before.tsv` and the 860 tree's table,
  and never delete a tree whose table has an unfinished operation that was not carried over.

## Open questions for the owner

Ranked by how much they change the plan.
Each has options,
pros,
cons and a ranking.

### Q1. `$PATH` entries inside the repository

With the repository away,
the shell skips `.cache/*-bin` and falls to unshadowed tools (W5).

- Leave as it is:
  pros,
  no change,
  cannot hang;
  cons,
  `cargo`,
  `pnpm`,
  `rg` and the search tools silently lose their policy outside a working repository,
  and `command not found` for the shadow's own commands.
- Global stubs placed after the repository entries in `$PATH`,
  for the shadowed names that are not also system tools (`cargo`,
  `pnpm`,
  `mpm`,
  `rg`):
  pros,
  a plain message instead of a silent fall-through;
  cons,
  a stub in the way of the real tool for every other project on this machine,
  so it must exit through to the real tool when the working folder is not under the repository,
  which is exactly the silent fall-through again.
- Move the shadows' launchers out of the repository (the global scope that `git` already uses):
  pros,
  they never dangle;
  cons,
  the launcher's per-tree lock comparison is the point of the repository scope,
  and this is a redesign of `mpm-shadow` scopes.
- Ranking:
  leave > global stubs > move the launchers,
  because a shadow's policy only applies to work inside the repository and no work happens there while it is away;
  stubs add a hazard for every other project;
  moving launchers redesigns a tool for a case with no write risk.

### Q2. How `claude-cpu-watch` copes with a dangling path at start

- Retry every thirty seconds with `StartLimitIntervalSec=0` and a pre-start check (W8):
  pros,
  recovers on its own with no ordering,
  never ends in a permanent failure;
  cons,
  one journal line per retry for as long as the 860 is away.
- `ExecCondition=` on the check:
  pros,
  the unit goes inactive cleanly with no log noise;
  cons,
  nothing starts it again after the mount,
  so it stays off until the next login or a manual start,
  which is a start-order dependency.
- A `.path` unit with `PathExists=` on the script:
  pros,
  no polling;
  cons,
  `systemd.path(5)` says path units use inotify and "suffer by the same limitations as inotify",
  and a mount appearing is not a change event for a watcher on the folder above,
  so it can wait indefinitely [inferred;
  not tested];
  the same page says a path unit beneath a mount unit gets an automatic requirement and ordering dependency on it,
  which is the dependency the owner rejected.
- Ranking:
  retry loop > `ExecCondition` > `.path` unit,
  because only the first recovers without help and without an unbounded wait;
  `ExecCondition` breaks the owner's ordering rule;
  the `.path` unit can wait indefinitely and adds the mount dependency.

### Q3. `user_subvol_rm_allowed` on the 860

- Add it to the 860's mount (live remount and `mount-860.service`):
  pros,
  worktrees that are subvolumes (242 of 246) can be removed in one step as the user,
  as on the NVMe;
  the wrapper's `doctor` stops reporting it;
  cons,
  it applies to the whole 860 mount,
  so any subvolume the user can write may be deleted by the user.
- Leave it off and use `sudo btrfs subvolume delete`:
  pros,
  no change to the mount;
  cons,
  agents cannot remove worktrees (measured:
  `EPERM` as the user),
  and the wrapper's behavior in that case is unmeasured.
- Ranking:
  add > leave,
  because removal of 242 worktrees without root is how the repository works today,
  and the extra reach is limited to what the user already owns.

### Q4. Hooks that enforce rules when their script is missing

- Non-blocking for the output filter,
  `onFailure: block` for hooks that enforce a rule (W6):
  pros,
  no rule is skipped silently;
  cons,
  with the repository away a rule hook blocks every tool call in a session that is not in the repository.
- Non-blocking for all:
  pros,
  sessions elsewhere keep working;
  cons,
  rule hooks are silently skipped.
- Ranking:
  split > non-blocking for all,
  because a skipped rule is a silent loss and a blocked session says why.

### Q5. Harness history

- Rename the old project folders and copy the settings entry (cut-over step 9):
  pros,
  history and trust carry over;
  cons,
  an edit of live harness state while no session runs.
- Start fresh at the new key:
  pros,
  nothing to edit;
  cons,
  the old sessions cannot be resumed from the repository,
  and the project trust dialog and MCP settings reappear.
- Symbolic link from the new key to the old folder:
  pros,
  old and new sessions share one folder;
  cons,
  whether the harness follows a linked project folder is unmeasured.
- Ranking:
  rename > fresh start > link,
  because rename keeps history without relying on unmeasured link handling,
  and a fresh start loses it.

### Q6. Clone sources and compression for the real run

- `--clone-sources=all`:
  pros,
  keeps sharing between every pair of worktrees;
  cons,
  each send looks up more clone sources and may be slow late in the run.
- `--clone-sources=main`:
  pros,
  fast;
  cons,
  sharing that exists only between two worktrees is lost,
  so the footprint on the 860 may exceed 263.6 GiB.
- Ranking:
  all > main,
  with the first twenty sends timed under both before choosing,
  because losing sharing costs space that is scarce (662G free) and slowness costs only time.
  `--compressed-data` for `btrfs send` was not evaluated.

### Q7. The pnpm store

- Leave on the NVMe:
  pros,
  no change;
  cons,
  installs in the 860 repository copy instead of clone (a different filesystem).
- Move to `/var/mnt/encrypted/.pnpm-store`:
  pros,
  clones work again;
  cons,
  other repositories depend on the 860.
- Ranking:
  leave > move for now,
  because a measurement of the first install should decide.

### Q8. Snapshots and backups on the 860

- Snapper covers `/var/home` only and never covered the repository (a nested subvolume);
  the 860 has no snapper configuration.
  Kopia keeps the same source.
- Option:
  a snapper configuration for the 860's `repository` folder.
  Not part of this plan;
  the owner's call.

### Q9. The 16 worktrees under `~/worktrees` on the NVMe

The owner chose "All worktrees" for the question that counted the nested ones.
These 16 are outside the repository tree,
so the copy leaves them where they are.
Their `.git` files name the repository through the link,
so they keep working while the 860 is mounted and show "not a git repository" while it is not.

- Leave them:
  pros,
  nothing moves,
  no new copy;
  cons,
  they cross filesystems (the wrapper's copy fallback applies to new ones),
  and 2 of them have uncommitted files (scan in the section "Drain").
- Move them to the 860 (one send each,
  then a path change in the owner's habits):
  pros,
  one filesystem for everything;
  cons,
  their paths change,
  so every reference to them changes,
  and `git worktree move` is needed per worktree.
- Ranking:
  leave > move,
  because leaving changes nothing and the 2 dirty ones keep their state;
  moving changes paths for no stated need.

## What was not verified

- Nothing that needs root was run:
  the mountpoint's real owner,
  mode and flags,
  whether a mount covers an immutable directory,
  and the whole copy (`send -c`,
  `-p`,
  stub removal,
  stream checks).
  R1,
  R2 and the loop-file fixture are the tests.
- The user-level fixture used tmpfs in a user namespace,
  not btrfs and not the real 860;
  its results for ownership and mode are the kernel's general behavior,
  not the real mount's.
- A transient systemd unit meant to re-measure the `claude-cpu-watch` start limit,
  and a follow-up `systemctl --user show`,
  were refused by the tool guard and were not retried;
  the start-limit values are from the earlier run.
- The commit service's behavior when its repository vanishes mid-request,
  `cargo`,
  `pnpm` and `search-shadow` mid-run failures,
  Codex's `--cd`,
  the harness's handling of a linked project folder,
  permission-rule matching across the link,
  IDE index keys,
  and kopia's first incremental size after the move.
- A real `pnpm install` in the real tree through the link.
- Uncommitted work changes while other sessions run:
  the scan in the section "Drain" is one snapshot and is repeated after the drain.
- The systemd source was read at the `main` branch,
  not at the installed 262.1 tag (history section).
- Two scratch subvolumes,
  `svfix` and `svfix-snap` in `/var/home/user/temp/agent/dangling`,
  could not be deleted without root.
