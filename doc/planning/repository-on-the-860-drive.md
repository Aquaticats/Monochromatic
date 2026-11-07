# Repository on the 860 drive: measurements, inventory, options and runbook

Status:
plan only,
written 2026-11-07T05:50:00-05:00 for the owner's decision.
Nothing was moved,
copied,
mounted,
unmounted,
unlocked,
stopped or deleted while writing it.
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
- "nothing should ever wait forever or indefinitely"
- "never discard user data",
  "Commits should never fail.",
  "Disk pressure being high is not an excuse.",
  "load is not an excuse."
- `AGENTS.md` rule RBT:
  a reboot can make data on this machine's NVMe permanently unreadable,
  so commit and push before any reboot and never count on uncommitted files surviving one.

## What the plan decides and what it leaves to the owner

The plan recommends one design (an automount of a repository subvolume that lives on the 860,
with the old path as a
locked empty stub) and one copy method (btrfs send and receive with clone sources,
after a fixture test).
Both are inferences from reading source and manuals plus the measurements in this file.
The runbook starts with a fixture test on a throwaway because no part of the mount design was run here.
The owner's open questions are at the end of the file,
ranked.

## Measurements

All sizes were taken on 2026-11-07 between 05:00:00-05:00 and 05:45:00-05:00 while other sessions kept committing,
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
  the rest is wrapper records and 340 `worktrees/<id>` administrative directories.
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
  the repository lists 341 worktrees,
  of which 247 are under
  `.claude/worktrees`,
  22 are on the NVMe (`~/temp/agent` 6,
  `~/worktrees` 16) and 71 are on the 860
  (`/var/mnt/encrypted/temp/agent`),
  plus the main one.
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
  The count is in the whole-set section.

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
  main's 80.5 GiB plus the exclusive bytes of the worktrees.
  The 137 small worktrees are below 2000 MiB each in total,
  the 84 large ones share most extents with main
  (sample:
  384 MiB exclusive of 44 GiB).
  The measured whole-set figure replaces this estimate;
  the estimate is main plus roughly 250 to 300 GiB,
  unmeasured.
- Scope four:
  main plus a chosen subset of worktrees (for example the 44 unmerged branches plus the 116 detached
  ones,
  owner's choice):
  between scope one and scope three.

### Whole-set figure

The reflink-aware measurement started at 2026-11-07T05:41:25-05:00 and was still running when this file was first
committed.
If the next section is empty,
rerun `btrfs filesystem du --summarize --raw /var/home/user/Monochromatic`
inside `systemd-run --user --scope` with the same four bounds,
and read total,
exclusive and set shared;
the sum of exclusive and set shared is the sharing-preserving footprint,
before compression.

Result:
pending.

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
  symlink to `/home/user/Monochromatic/package/claude-cpu-cap/daemon/systemd/claude-cpu-watch.service`.
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

## Options

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

## Ranking

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

## Runbook

Every long command runs in its own scope:
`sudo systemd-run --scope --collect` with the properties `CPUQuota=200%`,
`MemoryMax=2G`,
`TasksMax=32` and
`IOWeight=10`;
these bounds are sized to one copy stream and chosen by the plan author,
not measured.
No stress load is generated;
the copy is real work.

1.  Fixture test on a throwaway,
    owner runs,
    before anything real.
    Create a scratch subvolume tree on the 860 and reproduce A with a fixture path (for example
    `/var/home/user/temp/agent/fixture-860/Monochromatic`):
    the automount and mount units with `Requires=mount-860.service`,
    an immutable stub,
    then verify with the real command,
    not by reading:
    1.  A failed unlock (rename the key path in a copy of the unit,
        or make `cryptsetup` fail) returns `ENODEV` to
        `ls` after the bounded time,
        and a second try works after `reset-failed`.
    2.  `mkdir` and `touch` in the stub both fail;
        `mount --bind` over an immutable directory works.
    3.  `btrfs subvolume snapshot` of the bind-mounted subvolume root works and `btrfs filesystem du` shows sharing.
    4.  A send of two reflinked subvolumes with `-c` leaves the receiver's `Set shared` large (sharing kept).
    5.  `notify-send` while the user manager is blocked on the path:
        does `mount-860.service` finish?
    6.  Mounting the same btrfs twice with different options (A3).
2.  Pre-checks,
    read-only:
    record `git worktree list --porcelain | sha256sum`,
    `git for-each-ref | sha256sum`,
    `git fsck --full` on the NVMe repository (a long,
    load-heavy run,
    so in its own bounded scope),
    the file count
    (`find ... -type f | wc -l` per subvolume),
    and the list of root-owned files
    (`sudo find ... -user root`).
    Confirm the 860 has the required free space for the chosen scope.
3.  Lock the 71 worktrees on the 860 (`git worktree lock` per worktree) so a missing mount cannot prune them.
4.  Install the bounding drop-in on `mount-860.service` and the `timeout 5` around its notification
    (root commands below);
    run `systemctl daemon-reload`;
    confirm the value with `systemctl show mount-860.service --property=TimeoutStartUSec`.
5.  Pre-copy while everything keeps running:
    a first pass of the chosen method into
    `/var/mnt/encrypted/Monochromatic.incoming`,
    which can be repeated,
    with the 12 wrapper services,
    the editors
    and the agents untouched.
6.  Choose the quiet moment.
    Agents are not killed.
    A holder list is produced from `/proc/*/cwd` and `/proc/*/fd`
    for the repository and every nested worktree.
    Each session is asked to finish and exit;
    the wait has a deadline;
    at the deadline the cut-over does not happen and the holders are reported,
    so no work is lost to a kill.
    Stop the user units that run from the repository
    (`systemctl --user stop claude-cpu-watch.service`,
    the 12 `cli-git@*.socket` and `.service`)
    and note that they start again afterwards.
7.  Final pass:
    read-only snapshots of main and each chosen worktree (`btrfs subvolume snapshot -r`),
    incremental
    send (`-p` previous snapshot) to the 860,
    receive,
    make writable snapshots at the final paths inside the new
    `Monochromatic` subvolume,
    mode 0700 on its top directory.
8.  Verify the copy:
    `git fsck --full` on the 860 copy;
    `git for-each-ref | sha256sum` and
    `git worktree list --porcelain | sha256sum` equal to the recorded values;
    per-subvolume file counts equal;
    `rsync --archive --checksum --dry-run --itemize-changes` (as root) between a read-only snapshot and the copy
    prints nothing;
    the tracked-file and `.git` checksums (`git ls-files -z | xargs -0 sha256sum`) equal.
    Any difference stops the runbook with the old copy untouched.
9.  Cut-over,
    owner runs as root,
    in this order:
    1.  Rename the NVMe repository:
        `mv /var/home/user/Monochromatic /var/home/user/Monochromatic.nvme-before-860-<timestamp>`
        (same filesystem,
        atomic,
        nested subvolumes move with it).
        This is the kept old copy.
    2.  Create the locked stub and start the automount (commands below).
    3.  Start the user units again.
10. Checks after the cut-over:
    `findmnt /var/home/user/Monochromatic` shows the bind from the 860;
    `git -C ... status`;
    one test commit through the wrapper in a throwaway worktree;
    `git worktree list` equals the recorded value;
    `systemctl --user is-active claude-cpu-watch.service`;
    `mise trust --show` in the repository;
    the harness lists
    the old sessions;
    `systemctl list-units 'cli-git@*'`.
11. Checks after a reboot (commit and push first,
    rule RBT):
    log in without opening the repository,
    run
    `findmnt` (expect the autofs only),
    open a terminal in the repository (expect the bounded unlock,
    then success),
    repeat with `mount-860.service` made to fail once (rename of the key path in a drop-in) to see `ENODEV` and the
    locked stub;
    check `claude-cpu-watch.service` and the hourly kopia job for the new state.
12. The old copy is kept:
    the renamed NVMe directory stays until the owner removes it.
    Removal is an owner action
    (`btrfs subvolume delete` of the nested worktree subvolumes,
    then the root;
    the mount option
    `user_subvol_rm_allowed` allows it as the user on the NVMe).
    Its worktrees' `.git` files point at the live
    repository,
    so it is cold storage,
    not a second working repository.
13. pnpm:
    leave the store on the NVMe until the owner decides (open question);
    expect copies instead of clones
    in the 860 repository;
    measure the first install.

### Undo

1.  Stop the user units that use the repository.
2.  Copy the commits made on the 860 back:
    in the old copy run `git fetch /var/mnt/encrypted/Monochromatic '+refs/*:refs/from-860/*'`,
    plus a send or `rsync` of changed worktrees;
    without this step,
    commits made since the cut-over exist only on the 860.
3.  `systemctl disable --now var-home-user-Monochromatic.automount`;
    stop the mount unit.
4.  `chattr -i` the stub,
    remove it,
    rename the old copy back.
5.  Restart the user units;
    delete the unit files only after the old copy is confirmed in use.

### Root commands for the owner (Option A)

The unit names come from `systemd-escape --path --suffix=mount /var/home/user/Monochromatic`.

```bash
# /etc/systemd/system/mount-860.service.d/bounded-start.conf
sudo mkdir --parents /etc/systemd/system/mount-860.service.d
sudo tee /etc/systemd/system/mount-860.service.d/bounded-start.conf <<'EOF'
[Service]
TimeoutStartSec=150s
EOF
```

The bound of 150 s is the plan author's choice:
one recorded run took 8 s (`ExecMainStartTimestamp` 09:32:39 and
`ExecMainExitTimestamp` 09:32:47 on 2026-11-05),
three attempts with a 10 s pause between them bound the worst case
at several times that;
the owner may change it.
The notifications are a separate edit of `mount-860.service`:
each `runuser ... notify-send` gets a `timeout 5`
prefix (a full-file edit with `sudo systemctl edit --full mount-860.service`,
because the script is one inline
`ExecStart`).

```ini
# /etc/systemd/system/var-home-user-Monochromatic.mount
[Unit]
Description=Repository subvolume of the 860 at its old path
Requires=mount-860.service
After=mount-860.service
JobTimeoutSec=240s

[Mount]
What=/var/mnt/encrypted/Monochromatic
Where=/var/home/user/Monochromatic
Type=none
Options=bind
TimeoutSec=60s
```

```ini
# /etc/systemd/system/var-home-user-Monochromatic.automount
[Unit]
Description=Automount of the repository subvolume of the 860

[Automount]
Where=/var/home/user/Monochromatic

[Install]
WantedBy=local-fs.target
```

```bash
# Cut-over, after the rename in runbook step 9 and with the user units stopped.
sudo mkdir /var/home/user/Monochromatic
sudo chown root:root /var/home/user/Monochromatic
sudo chmod 0000 /var/home/user/Monochromatic
sudo chattr +i /var/home/user/Monochromatic
sudo systemctl daemon-reload
sudo systemctl enable --now var-home-user-Monochromatic.automount
```

Copy of scope one as root,
in a bounded scope (sizes:
roughly 80.5 GiB):

```bash
sudo systemd-run --scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 \
  --property=IOWeight=10 rsync --archive --hard-links --acls --xattrs --one-file-system --numeric-ids \
  --exclude=/.claude/worktrees/ /var/home/user/Monochromatic/ /var/mnt/encrypted/Monochromatic.incoming/
```

The owner makes `Monochromatic.incoming` a subvolume first (`btrfs subvolume create`,
mode 0700);
the send and receive commands for scope three are generated per chosen worktree by a scratch `.ts` after the owner
answers the first open question,
because shell loops are not allowed in this repository's rules (`1CB`).
To see what sits under the mountpoint `/var/mnt/encrypted` without touching anything:
`sudo unshare --mount --propagation private sh -c 'umount /var/mnt/encrypted; ls -ldA /var/mnt/encrypted'`.

## Open questions for the owner

Ranked by how much they change the plan;
each with options,
pros,
cons and a ranking.

### Which worktrees move

- All 243:
  pros,
  nothing is stranded;
  cons,
  only scope three fits,
  it needs M1 and its fixture result.
- The 44 unmerged branches plus the 116 detached ones,
  with the 180 merged-branch worktrees left in the cold old
  copy:
  pros,
  fits easily;
  cons,
  merged worktrees may still hold uncommitted files and the old copy's paths no longer
  match.
- Main only,
  worktrees recreated later by `worktree add`:
  pros,
  smallest;
  cons,
  discards the physical state of 243
  worktrees (kept only in the cold copy),
  which must never be deleted unread.
- Ranking:
  all > unmerged and detached > main only,
  because the first loses nothing and the last depends on the owner
  reading every cold worktree before deleting the old copy.

### Where the subvolume lives

- `/var/mnt/encrypted/Monochromatic`:
  pros,
  simplest,
  same level as `Archive` and `temp`;
  cons,
  the top-level
  directory is mode 777.
- A dedicated subdirectory `/var/mnt/encrypted/repository/Monochromatic` with mode 0700:
  pros,
  protects the
  credential in `.git/config`;
  cons,
  one more level.
- Ranking:
  dedicated 0700 directory > top-level,
  because the credential in `.git/config` is copied.

### The unlock timeout

- 150 s (this plan):
  pros,
  covers three attempts;
  cons,
  a failing unlock takes that long to report.
- 60 s:
  pros,
  quicker failure;
  cons,
  may cut a slow unlock under load,
  which the unit comment says can fail silently.
- Ranking:
  150 s > 60 s,
  because a premature failure costs a retry and a long wait costs only patience,
  but the
  owner may weigh "nothing should ever wait forever" differently.

### Whether the 860's unlock moves to `crypttab` and `fstab`

- Keep `mount-860.service` with the bound (this plan):
  pros,
  smallest change;
  cons,
  a hand-written unlock script.
- `crypttab` with the keyfile and an `x-systemd.automount` fstab line for `/var/mnt/encrypted`:
  pros,
  the standard
  mechanism,
  also covers pcloud;
  cons,
  the unit comment records silent wrong keys from argon2id under login-time
  contention that the retry loop works around,
  and `crypttab` has no such retry.
- Ranking:
  keep the unit > move,
  because the retry exists for a measured failure and the move is a separate decision.

### The pnpm store

- Leave on the NVMe:
  pros,
  no change;
  cons,
  installs in the repository copy instead of clone.
- Move to `/var/mnt/encrypted/.pnpm-store` (exists already):
  pros,
  clones work again;
  cons,
  other repositories'
  installs also depend on the 860.
- Ranking:
  leave > move for now,
  because a measurement of the first install should decide.

### Snapshots and backups of the repository on the 860

- Kopia keeps the source `/home/user/Monochromatic`;
  unchanged path,
  no action.
- Snapper covers `/var/home` only;
  the repository was not part of those snapshots (nested subvolume) and still
  is not;
  the 860 has no snapper configuration.
- Option:
  a snapper configuration for the 860 subvolume.
  Ranking:
  not part of this plan,
  owner's call.

## What was not verified

- Nothing in the mount design was run;
  the first runbook step is the test.
- The systemd source was read at the `main` branch,
  not at the installed 262.1 tag.
- The reflink-preserving behavior of send with clone sources and of `rsync --inplace` was not run.
- The whole-set `btrfs filesystem du` figure was pending at the first commit.
- Uncommitted work inside the 243 worktrees was not measured.
- The `notify-send` deadlock candidate is a reading of the unit script,
  not an observation.
- The underlying mountpoint directory `/var/mnt/encrypted` (beneath the 860) was not inspected because the 860 is
  mounted and nothing was unmounted;
  the `unshare` command above reads it.
