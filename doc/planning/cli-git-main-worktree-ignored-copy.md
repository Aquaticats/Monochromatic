# A worktree added from the main checkout carries its ignored state

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
implemented on branch `main-ignored-copy` (not merged,
not released),
2026-11-02.
Scope:
`package/git-policy/cli` (crate `monochromatic-git-policy-cli`),
Linux.
The measurements ran on this machine under the load there was (one-minute load average between 14 and 35 on 28 threads),
inside `systemd-run --user --scope` or `--unit` with `CPUQuota`,
`MemoryMax`,
`TasksMax` and `IOWeight` of at most 50.
Fixtures were throwaway worktrees under `.claude/worktrees/` and a clone under
`/var/home/user/.cache/main-ignored-copy`;
all were removed.

## Owner decisions that bound this design

- 2026-11-01:
  "carrying over ignored files like node_modules into a new worktree is expected,
  because agents and people
  frequently forget to manually install node_modules in new worktrees."
- 2026-11-01:
  "yes but that's intended and I also intended for this to not be so heavy because we can take advantage of
  btrfs."
- 2026-11-02:
  "node_modules must be copied."
- 2026-11-02:
  "Builds must be copied.
  Other ignored files must be copied."
- 2026-11-02:
  "Skip nested worktrees and caches.
  The point is to make new worktrees immediately functional."
- 2026-11-02:
  "Skip run records (Recommended)."
- Standing rules used here:
  keep package managers out of the wrapper (running one from Git is a cross-component concern
  the owner has not approved);
  keep the settlement lock,
  the journal and recovery;
  `--no-worktree-copy` keeps working.

## Summary

- Before:
  an add run from the main checkout copied only the configured `rulesFile` files
  (`package/git-policy/cli/SPEC.md`,
  section "Linked-worktree ignored-state synchronization"),
  so a new worktree had no
  `node_modules` and every `mise run` task failed with `ERR_MODULE_NOT_FOUND` until a dependency install ran.
- Now:
  the same add copies the main checkout's whole ignored state except what a new worktree does not need.
  The rule is three families of directory names (`src/native/worktree_copy_prune.rs`):
  other checkouts,
  caches,
  and run
  records.
  A linked or bare source is unchanged:
  it copies everything it ignores except its nested registered worktrees.
- The copy was too slow as written (it re-read every staged byte,
  one thread cloned one file at a time,
  the install
  linked and then unlinked every entry,
  and its log was synchronized every 512 entries),
  so four changes keep the
  settlement lock,
  the journal and recovery and cut the cost;
  see "Design" and "Measurements".
- Wall time of a main-sourced add on this machine is in "Measurements" (section "The add itself").
  It is above the ten seconds the coordinator named as the point to report,
  because the ignored state is about 190
  thousand entries that must each be cloned,
  linked into place and recorded;
  the btrfs snapshot rung of the ladder on
  `btrfs-worktree-ladder` is the route to two or three seconds and is discussed under "The ladder".

## Facts

### The main checkout

- `findmnt --target /var/home/user/Monochromatic`:
  btrfs,
  `rw,relatime,seclabel,compress=zstd:1,ssd,discard=async,
  space_cache=v2,user_subvol_rm_allowed,subvolid=257,subvol=/home` (the mount holds `/var/home`).
- The checkout root is already a btrfs subvolume:
  `btrfs inspect-internal rootid /var/home/user/Monochromatic` printed
  3569 (the home subvolume is 257),
  the root directory has inode 256,
  and its birth time is 2026-11-01 17:58.
  The brief and the coordinator's follow-up said it was not yet a subvolume;
  the measurement says it is.
- `git worktree list --porcelain` registers 228 worktrees;
  139 of them have their root inside the main checkout,
  all
  under `.claude/worktrees/`.
  Of those 139,
  91 are subvolumes (inode 256 at their root) and 48 are ordinary directories
  (`node nested.ts` in the scratch directory,
  which `stat`s each root).
  A worktree created by the released wrapper (0.5.13 and 0.5.14) is an ordinary directory.
- Git's own exclusion stack (`git -C /var/home/user/Monochromatic ls-files --others --ignored --exclude-standard
  --directory`,
  0.055 s) names 949 ignored roots.
  Without the 139 nested worktrees they hold 693,346 entries and about 75 GiB apparent
  (`scan-ignored.ts`:
  one `find -printf .`,
  one `du --summarize --apparent-size` and one `du --summarize` per root).
  The nested worktrees hold about 22 thousand entries each (four sampled:
  22,266;
  24,603;
  22,746;
  22,289),
  about 3.1
  million in all and 160 GiB apparent.

### What the ignored set holds

Entries and apparent MiB per kind,
nested worktrees excluded (`inner.ts` and `scan-ignored.ts`):

- `target/` build outputs,
  37 trees:
  290,547 entries,
  63,270 MiB.
  Inside them:
  `debug/incremental` 62,257 entries and 19,769 MiB;
  `debug/deps` 8,363 and 13,968;
  `debug/build` 21,627
  and
  11,016;
  `verification` 97,228 and 6,007;
  `release/deps` 4,849 and 5,375;
  `dylint` 23,583 and 4,269;
  `zig-global-cache` 22,971 and 379.
  The largest tree is `package/ide/app/target` with 144,538 entries and 39,391 MiB.
- `node_modules`,
  221 trees:
  the repository root one holds 175,713 entries and 3,747 MiB;
  the 220 per-package ones hold
  145,466 entries and 2,464 MiB.
  In the root one:
  `.pnpm` (the virtual store) 49,867 entries and 913 MiB;
  `.pnpm-store` 37,174 and 999;
  `.monochromatic` 53,869 and 1,012;
  `.cache` 34,663 and 825;
  1,297 symbolic links in all.
  In the per-package ones:
  `.monochromatic` 143,110 entries and 2,458 MiB,
  so almost all of their entries are logs.
- `.pnpm-store/` at the root,
  37,235 entries and 991 MiB (a second,
  older pnpm store).
- `dist/` 202 trees,
  4,960 entries,
  601 MiB;
  `build/` 4 trees,
  5,908 entries,
  931 MiB;
  `package/vm-builder/cli/output` 5 entries,
  3,783 MiB;
  the root `.cache/` 1,248 entries,
  42 MiB (tool launchers,
  `forbidden-strings.rules.txt`,
  `toolchain.env.json`);
  every other `.cache` directory,
  228 trees with 979 entries and
  about 58 MiB (the 2,227 entries and 101 MiB of all `.cache` directories minus the root one),
  holding only `typescript`
  build info.
- 191 ignored files (local configuration,
  `*.local.*`,
  `.env.local.json`,
  `mise.local.toml`),
  244 MiB.
- No socket,
  FIFO or device node exists in the ignored set (`find ... -not -type f -not -type d -not -type l` found
  none).
- 56 ignored entries are unreadable by the user,
  all but a few under `node_modules/.monochromatic`:
  root-owned files of
  mode 0600 such as `node_modules/.monochromatic/2026-11-01/2026-11-01T19-30-26.647Z.<id>.log.jsonl`.
  A copy that opens every file fails on them.

### The wrapper before this change

- `package/git-policy/cli/src/native/worktree_copy_registry.rs`,
  `observe_copy_source`:
  "A main checkout is not a bypass
  any more for the named paths ... but it is never copied as a whole:
  its ignored trees are the repository's heaviest."
- The stage verified itself by reading both files again:
  `worktree_copy_compare.rs`,
  `regular_file_bytes_equal`,
  called
  for every staged file.
  On the roughly 40 GiB of build products in this set that is a read of about 80 GiB.
- The stage copied one entry at a time,
  and the install created one hard link per entry,
  with `lstat` calls for every
  ancestor of every entry and one synchronized log append per 512 entries;
  completion then unlinked every staged entry.
- An add run in a worktree that is its own subvolume cannot hard link from another subvolume,
  and pnpm's store choice
  follows that (see the `pnpm and the store` section).

### pnpm and the store

- `pnpm store path` in a fresh ordinary-directory worktree under the main checkout printed
  `/var/home/user/Monochromatic/.pnpm-store/v11`,
  with the warning "The store at /home/user/.local/share/pnpm/store/v11
  is not used because packages cannot be hard linked from it into this project."
  The same command in a worktree that is its own subvolume printed that worktree's own
  `node_modules/.pnpm-store/v11`,
  and `pnpm install --frozen-lockfile --offline` there failed after 18.8 s with
  `ERR_PNPM_NO_OFFLINE_TARBALL ... mute-stream@3.0.0 ... snapshot not present in local store`.
  The same install with `--store-dir /var/home/user/Monochromatic/.pnpm-store` succeeded in 1.8 s:
  pnpm clones across
  subvolumes (`package-import-method` is `clone-or-copy` here).
- In an ordinary-directory worktree,
  `pnpm install --frozen-lockfile --offline` took 2.1 s and 2.0 s and succeeded for
  the
  current `main` lockfile and for the commit `745dcfc22f` used earlier today.
  The coordinator reported that offline failed today;
  I could not reproduce it in an ordinary-directory worktree.
  The failing case I found is a worktree that is its own subvolume,
  which is what the btrfs ladder will make every new
  worktree.
- pnpm documents the rule:
  "If no directory above the project accepts a hard link at all ... the store is created inside
  the project instead:
  at node_modules/.pnpm-store" (see "Precedent").

### What a new worktree needs from the main checkout

Found by experiment in throwaway worktrees (`ignored-copy-probe-a`,
`-b`,
`-c`,
`-e`,
removed afterwards):

- With nothing copied,
  `mise run //package/git-policy/cli:test:unit` fails with `ERR_MODULE_NOT_FOUND`.
- With only the root `node_modules` (without `.pnpm-store`,
  `.cache` and `.monochromatic`) it still fails.
- With the root `node_modules` and the package's own `node_modules`,
  it fails on
  `@monochromatic-dev/test-harness-core-diagnostic`:
  the per-package `node_modules` of the workspace packages it imports
  are needed too.
- With the root `node_modules` and the per-package `node_modules` of the 18 packages in the workspace closure of the
  task
  (computed from the `@monochromatic-dev` symbolic links;
  17 for the CLI package and 13 for the task runner,
  18
  together),
  `test:unit` exits 0 with 70 passing lines and `lint:types` exits 0.
  Those copies excluded `.monochromatic`,
  `.cache` and `.pnpm-store`.
- The container gate launcher `bin/test-native-container.ts` resolves its two bare imports in that set and fails to
  resolve them with nothing copied (`import.meta.resolve`);
  I did not run the container build itself.
- The session's `PATH` points at the main checkout's `.cache/toolchain-bin` and `node_modules/.bin`,
  so executables
  resolve to main whatever the worktree holds;
  the module resolution result in `What a new worktree needs from the main checkout` does not depend on that.
- pnpm's `.pnpm-workspace-state-v1.json` holds 239 absolute paths of the main checkout,
  and `.modules.yaml` one;
  a
  copied
  `node_modules` carries them.
  I did not test what a later `pnpm install` does with them in a copied tree.

### Classification of the top-level ignored paths

Each path is judged by the owner's purpose:
does the new worktree need it to run,
build,
test or lint at once?

- `node_modules/` (root and per-package) minus its caches and logs:
  needed (dependencies and links).
- `node_modules/.pnpm-store`,
  `.pnpm-store/`:
  cache (a content-addressed download store;
  pnpm rebuilds it).
- `node_modules/.cache`:
  cache (jiti,
  file-enforcer).
- `node_modules/**/.monochromatic`:
  run records (logs of earlier runs;
  includes the unreadable root-owned files).
- `target/` minus the next three:
  needed (builds).
- `target/*/incremental`,
  `target/zig-global-cache`:
  cache (compiler state rebuilt on demand).
- `target/verification`:
  run records (outputs of verification runs).
- `dist/`,
  `build/`,
  `output/`,
  `jniLibs/`,
  generated `src/*.js`:
  needed (builds).
- `.cache/` at the repository root:
  needed (tool launchers and generated inputs that tasks read).
- `.cache/` elsewhere (`typescript` build info):
  cache.
- `.gradle`,
  `.kotlin`:
  cache.
- `.terraform`:
  kept (provider plugins that `tofu validate` needs without an init).
- `mutants.out`,
  `mutants.out.old`,
  `playwright-report`,
  `test-results`,
  `.serena/cache`:
  run records.
- Local files (`*.local.*`,
  `.env.local.json`,
  `mise.local.toml`,
  `.claude/settings.local.json`,
  `.idea`,
  `.vscode`,
  `.serena/memories`,
  fuzz `corpus/`):
  needed.
- Registered worktrees under `.claude/worktrees/` and any directory holding `.git`:
  another checkout,
  never copied.

The owner approved this rule on 2026-11-02 ("Skip nested worktrees and caches",
"Skip run records (Recommended)").

### Precedent

Fetched 2026-11-02.
Each quoted phrase was confirmed to be in the rendered page text with `agent-browser`
(`document.body.innerText.includes(...)`,
all true);
I did not inspect the highlight a fragment draws.

- Claude Code copies only what a `.worktreeinclude` file names,
  and only if Git ignores it:
  "Only files that match a pattern and are also gitignored are copied" ([source][claude-include]).
  It tells the user to "initialize your development environment there" ([source][claude-setup]).
- worktrunk (`wt step copy-ignored`) copies from the primary worktree by default,
  "By default the copy runs from the primary worktree into the current one" ([source][wt-source]),
  copies "All gitignored files ... by default,
  except for built-in excluded directories" ([source][wt-what]),
  skips nested worktrees,
  and reflinks:
  "Reflinks are per file (like cp -Rc),
  so copy time scales with file count" ([source][wt-cow]).
  Its sample shows 20 s and about no extra disk for a 14 GB `target/`.
  It suggests symbolic links for `node_modules` when a project has no native dependencies.
- VS Code names `node_modules` as the common case:
  "Git doesn't copy files that are excluded by .gitignore" ([source][vscode-include]),
  and offers `git.worktreeIncludeFiles` (copy) and `git.worktreeSymlinkFolders` (symbolic link folders that contain no
  tracked files).
- pnpm's guidance for several checkouts is a shared store:
  each worktree has its own `node_modules`,
  and
  "dependencies are shared across all of them" through one content-addressable store ([source][pnpm-worktrees]),
  with `virtualStoreType: global` so a worktree's `node_modules` holds only symbolic links,
  and the warning "Do not use
  one writable pnpm store for mutually untrusted agents or users."
- pnpm's store location rule:
  "the store is created inside the project instead" when no directory above the project
  accepts a hard link ([source][pnpm-store]).
- JetBrains' worktree page,
  git-town's documentation and Graphite's multiple-worktrees page say nothing about ignored
  files in a new worktree (searched 2026-11-02:
  the JetBrains page text,
  the Graphite page text,
  and the git-town
  repository and site for "worktree" and "node_modules";
  the searches ran and found no statement).

[claude-include]:
https://code.claude.com/docs/en/worktrees#:~:text=Only%20files%20that%20match%20a%20pattern%20and%20are%20also%20gitignored%20are%20copied
[claude-setup]: https://code.claude.com/docs/en/worktrees#:~:text=initialize%20your%20development%20environment%20there
[wt-source]: https://worktrunk.dev/step/#:~:text=By%20default%20the%20copy%20runs%20from%20the%20primary%20worktree
[wt-what]: https://worktrunk.dev/step/#:~:text=All%20gitignored%20files%20are%20copied%20by%20default
[wt-cow]: https://worktrunk.dev/step/#:~:text=Reflinks%20are%20per%20file
[vscode-include]:
https://code.visualstudio.com/docs/sourcecontrol/branches-worktrees#:~:text=Git%20doesn%27t%20copy%20files%20that%20are%20excluded%20by%20.gitignore
[pnpm-worktrees]: https://pnpm.io/git-worktrees#:~:text=dependencies%20are%20shared%20across%20all%20of%20them
[pnpm-store]: https://pnpm.io/settings/store#:~:text=the%20store%20is%20created%20inside%20the%20project%20instead

## Design

### The rule

`src/native/worktree_copy_prune.rs` holds the rule as `Prune::MainCheckout { run_records }`:

- another checkout:
  a directory holding a `.git` file or directory is left out whole;
  registered worktrees are
  excluded by
  path as before;
- caches:
  the names `.pnpm-store`,
  `.gradle`,
  `.kotlin`;
  `.cache` at any depth except as the first path component;
  `incremental` and `zig-global-cache` below a `target` component;
- run records,
  left out while `COPY_RUN_RECORDS` is `false` (it is):
  `.monochromatic` below a `node_modules` component,
  `verification` directly under `target`,
  `mutants.out`,
  `mutants.out.old`,
  `playwright-report`,
  `test-results`,
  and
  `cache` directly under `.serena`.

The decision uses path components only,
so it is the same for a root Git listed and a child the walk found.
`CopySelection::MainCheckout` selects it;
`CopySelection::Ignored` (linked and bare sources) applies none.

### Unreadable entries

The walk asks `access(2)` whether the user can read each file and list each directory.
An entry that fails is left out with its subtree and named in one stderr line after the copy
(`cli-git: left out 2 unreadable ignored entries from "<source>": "a", "b"`,
the first three paths and a count of the
rest);
the stage writes the full list to the verbose debug log only,
so stderr carries the one notice.
This applies to every source,
because a copy must never fail over a file the user cannot open.

### What FICLONE guarantees and how a concurrent writer is detected

This is the SPEC change a reviewer should check ("regular-file bytes" became "regular-file content").

- `FICLONE` makes the destination share the source's data extents as of one instant (`ioctl_ficlone(2)`:
  the call clones
  the file's data blocks,
  and a write to either file afterwards is copy-on-write).
  I did not read the btrfs source for the locking;
  the argument needs only that the clone is a point-in-time snapshot of
  the source file's data,
  which the manual page states.
  The clone's bytes therefore equal the source's bytes at that instant,
  whatever a writer does afterwards.
- The risk is a write that lands before the clone and after the manifest,
  or a write that the copy cannot see
  afterwards.
  The old check caught every difference by reading both files again.
  The new check proves the source did not change between the clone and the end of the copy,
  by comparing a stamp
  (size,
  modification time,
  change time,
  inode) taken before the clone,
  after the clone and at the final check.
- Why a stamp is enough:
  the change time of a file moves on every write,
  truncate,
  rename-over and metadata change,
  and no
  program can set it;
  the modification time and size move on a write;
  the inode moves when a file is replaced.
  A write that completes between the stamps changes at least the change time.
- The granularity gap:
  kernel timestamps come from a coarse clock,
  so a write inside the same tick as the stamp could
  leave
  the stamp unchanged.
  The copy therefore trusts a stamp only when the file's change time is at least 20 ms older than the clock at the
  clone;
  a more recent file is compared byte for byte at the end,
  as before.
- A file whose clone is refused is copied byte by byte and compared byte for byte,
  as before.
- The directory,
  link and manifest checks are unchanged:
  the final check still walks the source and the stage again and
  requires the same paths,
  kinds,
  modes and link targets.
- What is no longer detected:
  a source file rewritten with the identical size,
  modification time,
  change time and inode,
  which needs a clock step back and a restored time;
  the SPEC already excludes "transient source mutations that revert".

### Parallel cloning, bounded by CPU

- A manifest of at least 4096 entries is copied by `min(available_parallelism, 8)` threads over contiguous slices,
  after
  every directory exists;
  the first failure sets a flag that stops the other workers.
  `available_parallelism` follows the process's CPU affinity and cgroup quota,
  so a quota of two CPUs gets two threads.
- The install creates its files and links in the same way,
  one batch of 4096 entries at a time:
  intents for the batch
  are
  recorded and synchronized first,
  directories and scaffolding are created in order,
  the leaves are created in parallel
  slices,
  then the creations are recorded.
  Every path a worker created is returned even when another worker fails,
  so the in-memory rollback list is complete.
- The stage's removal splits the stage three directories deep and removes the subtrees on up to eight threads.
- Ordering that recovery relies on is kept:
  claims precede creations in the log,
  the log is synchronized per batch,
  the
  journal phases are unchanged,
  and rollback still walks the created list in reverse.

### What did not change

- The settlement lock,
  the stage claim,
  the journal,
  the install log,
  recovery and the rollback rules.
- The destination preflight:
  an existing destination entry must match the stage exactly.
- `--no-worktree-copy`:
  still recognized,
  stripped before Git,
  and it skips the copy and the recovery.
- A main checkout command that creates or moves no worktree still costs no filesystem reads.
- The configuration is no longer loaded for a main-checkout add (only an index writer loads it,
  as before).

## Measurements

All wall times are medians where a series is stated;
"load" is the one-minute load average at the start.

### Copy mechanisms, on a copy of node_modules alone (`bench.ts`, three rounds, shuffled, `CPUQuota=400%`)

Per-file reflinks with `cp --archive --reflink=always`,
which is the same mechanism as the stage's `FICLONE` without the
stage,
manifest,
verification or install,
into a directory inside a throwaway worktree on the main checkout's subvolume.
Each arm has a same-command control.

- Closure set (root `node_modules` minus store,
  cache and logs,
  and 17 package trees):
  50,104 entries,
  3.1 s median
  (2.8,
  3.2,
  3.1);
  control 3.1 s (3.0,
  3.1,
  3.7).
- All `node_modules` content of every package,
  minus `.monochromatic`:
  52,626 entries,
  9.0 s (14.9,
  8.3,
  9.0);
  control
  4.7 s (4.9,
  4.2,
  4.7).
  The spread between the arm and its control,
  4.3 s,
  is the noise to read the other numbers against.
- Every `node_modules` tree whole:
  321,385 entries,
  32.1 s (27.2,
  32.1,
  38.3);
  control 20.3 s (42.1,
  19.0,
  20.3).
- All ignored roots except `.claude` and `target`:
  402,605 entries,
  47.9 s (39.5,
  47.9,
  81.8);
  control 45.9 s (45.9,
  34.7,
  61.4).
- Removal of those copies took 1.0 s for the closure set and 13 to 18 s for the larger ones.
- A copy is roughly 60 to 100 microseconds per entry under this load;
  the previous design measured about 350
  microseconds
  per entry in the btrfs planning document.

### pnpm

- `pnpm install --frozen-lockfile --offline` in a fresh ordinary-directory worktree:
  2.1 s,
  2.0 s (exit 0).
- The same in a worktree that is its own subvolume:
  18.8 s and exit 1 (`ERR_PNPM_NO_OFFLINE_TARBALL`);
  with
  `--store-dir` naming main's store:
  1.8 s,
  exit 0,
  and `test:unit` passes (70 passing lines).
- The online install was not timed:
  the offline case fails only for the store location,
  which the explicit store fixes.

### The add itself

The throwaway fixture holds main's ignored state without the nested worktrees:
190,087 entries after the rule (152,944
regular files cloned under a stamp),
reflinked from the real checkout.
Commands:
`bench-add.ts` (in `/var/home/user/temp/agent/main-ignored-copy`) alternates four arms in shuffled order,
each as
`systemd-run --user --scope` with `CPUQuota=800%`,
`MemoryMax=8G`,
`TasksMax=256`,
`IOWeight=30`,
then removes the
worktree
and its branch.
Arms:
the new build (`new`,
and `new-ctl` as the same-binary control),
the same code with `COPY_RUN_RECORDS` set to
`true`
(`with-run-records`),
and the released 0.5.14 wrapper (`installed-baseline`,
which copies one file).
The series was stopped after two full rounds because the `with-run-records` arm alone took several minutes per run.

- `new` and `new-ctl` together,
  six runs:
  87.8,
  140.9,
  26.8,
  92.2,
  42.0,
  90.7 s;
  median 89.3 s,
  range 26.8 s to 140.9 s.
  The two arms' medians differ by 2.9 s (87.8 and 90.7),
  far below the spread inside one arm,
  so the host load,
  not the
  arm,
  explains the differences.
  One-minute load average at the start of those runs:
  16.3 to 38.2.
- `installed-baseline`,
  three runs:
  44.6,
  26.2,
  4.8 s;
  median 26.2 s.
  This is Git's own `worktree add` of about 20 thousand tracked files plus the one-file copy;
  its range shows how much
  of
  every number here is load.
- `with-run-records`,
  two runs:
  345.5 s and 159.0 s (487,637 entries copied,
  the run records add 297,550 entries);
  removing that worktree took 213 s and 39 s.
- Removing a worktree made by `new` (the user's later `git worktree remove`):
  15.6,
  9.5,
  11.4,
  21.6,
  40.9,
  12.0 s;
  median 13.7 s.
- A single run of the final build at a calmer moment (one-minute load 7.5,
  five-minute 14.0),
  with the debug log on:
  31.6 s wall in all,
  26.8 s of it the copy,
  user CPU 5.0 s and system CPU 45.2 s,
  peak memory 0.5 GB.
  Phases:
  selecting the roots 1.3 s;
  staging 11.4 s (walk 2.6,
  clone by 8 workers 4.6,
  modes 1.1,
  final check 3.2);
  creating the journal 1 ms;
  installing 11.2 s (preflight 0.6,
  create and record 7.3,
  modes 1.2);
  removing the stage
  2.9 s.
- Earlier builds of the same series,
  for what each change bought (single runs,
  load 14 to 24):
  the first working build
  took
  104 s (45 s in the install,
  68 s removing the stage in one run);
  after the cached parents and the larger log batch the
  install took 20 to 29 s;
  after parallel creation and parallel removal,
  7.3 s and 2.9 s in the "A single run of the final build at a calmer moment" entry.

What a main-sourced add costs on this machine,
as the coordinator asked:
about 32 s at light load and a median of
about 89 s
under the load of the afternoon (range 27 s to 141 s),
against 5 s to 45 s for the same command without the copy.
That is above ten seconds in every run.
The cost is the entry count:
190,087 entries,
each cloned,
linked into place and recorded,
plus the walks that prove
them.
The btrfs snapshot rung measures 2 to 3 seconds for the same command on a clean subvolume,
but it carries the caches and
records this rule leaves out and declines while any nested worktree is an ordinary directory;
see "The ladder".

## Verification

- `cargo nextest run --lib` on the host:
  2843 passed,
  4 skipped;
  `cargo nextest run --test native_binary`:
  445 passed,
  1
  skipped;
  `cargo clippy --all-targets -- -D warnings`:
  clean.
  The container gate,
  the Windows clippy lane (`native:clippy:windows`) and the other test targets were not run.
- Guard tests shown red with the guard removed (each change made after committing the tests,
  then restored by editing
  the
  lines back,
  and the working tree compared with the commit):
  1.  the main-checkout selection reverted to named paths:
      `main_checkout_source_carries_dependencies_and_builds` and
      `main_worktree_is_observed_with_the_main_checkout_selection` failed;
  2.  the leave-out rule and the checkout skip removed from the walk and the root filter:
      `main_checkout_source_leaves_out_checkouts_caches_and_run_records`,
      `unreadable_entries_and_other_checkouts_are_left_out_of_a_main_stage` and
      `an_interrupted_main_checkout_copy_is_resumed_from_its_stage` failed;
  3.  the stamp no longer trusted in the final comparison:
      `an_unchanged_stamp_stands_in_for_reading_the_bytes` failed.
- A kill mid-copy:
  `an_interrupted_main_checkout_copy_is_resumed_from_its_stage` (a journal left in the installing phase
  with an intact main-checkout stage is resumed) and the binary test
  `an_interrupted_main_checkout_copy_is_recovered_by_the_next_creation` (a staged journal is recovered by the next add
  from the main checkout,
  with the whole ignored state arriving).
  I did not send a real signal to a copy in flight.

## Snapshot route measured

Requested by the owner on 2026-11-02 ("Hold until snapshot measured");
`main-ignored-copy` stays unmerged until the
owner
chooses.
Everything ran on a throwaway fixture under `/var/home/user/.cache/snapmeas`,
on the main checkout's filesystem;
the
real
checkout and every real worktree were only read.
The ladder branch `btrfs-worktree-ladder` (head `414bb552d1`) owns the snapshot rung;
I read it and did not run it,
so
the
numbers in the Snapshot route measured section are for the snapshot and the removals the owner asked for,
not for the ladder's own steps (its tree
update,
index install and checkout line),
which its author measured at 2.43 s for a whole main-checkout add.

### The fixture

- `fix-root`,
  a btrfs subvolume holding:
  a reflink copy of main's real `.git` (22 thousand entries,
  1.9 GB),
  the tracked
  tree at main's `HEAD`,
  main's whole ignored state reflinked in (693 thousand entries,
  the same set the add copies),
  four nested linked worktrees that are subvolumes and four that are ordinary directories (each holds the tracked tree,
  about 22.5 thousand entries,
  registered in the copied `.git`).
- The skip set of the rule in `worktree_copy_prune.rs`,
  expressed as `find` predicates in `snap-bench.ts`,
  matches 479
  trees
  and 494,744 entries in the fixture (the same count as the earlier estimate for main).
  Finding them takes 1.2 s.
- Scaling to the real main checkout:
  the skip set is the same set,
  the `.git` is the same copy,
  and the nested count
  is 48
  ordinary and 91 subvolume worktrees instead of 4 and 4;
  per-worktree figures in the
  `Measured costs (three rounds, shuffled, a same-command control, one-minute load 2.1 to 6.0)`
  section are multiplied for that,
  and
  marked.

### What a snapshot shows

- `btrfs subvolume snapshot fix-root snap` shows each nested subvolume worktree as an empty directory with inode
  number 2
  (checked with `stat` and `ls -A`),
  and each nested ordinary worktree as a full copy of its files,
  including its `.git`
  file,
  which still reads `gitdir: <original>/.git/worktrees/<id>`.
- The copy of the main `.git` is a whole second repository directory in the new subvolume.

### Measured costs (three rounds, shuffled, a same-command control, one-minute load 2.1 to 6.0)

- The snapshot itself:
  median 0.15 s over fifteen snapshots (0.02 s to 0.94 s;
  most under 0.4 s).
  Its size does not matter:
  it is a copy-on-write root,
  and the fixture holds 700 thousand entries.
- Deleting the snapshot afterwards with `btrfs subvolume delete`:
  55 ms to 73 ms (the kernel frees the extents
  afterwards;
  I did not time that).
  So removing a worktree that came from a snapshot costs under 0.1 s,
  against 9 s to 41 s for a worktree of copied
  entries.
- Foreground work that must precede the add returning,
  in either variant:
  1.  unlinking the `.git` pointer file of every nested ordinary worktree in the copy:
      3 ms to 4 ms for four (about 50 ms
      for 48,
      extrapolated);
  2.  removing the copied `.git` directory,
      which the ladder replaces with the new worktree's pointer:
      714 ms to 749 ms
      (nine runs) by `rm -rf`;
      renaming it into a trash directory is a single rename.
- Removing the nested ordinary worktrees' trees:
  2.03 s to 2.29 s for four,
  0.52 s each;
  about 25 s for 48
  (extrapolated).
- Removing the skip set (caches and run records by the rule):
  13.1 s to 14.0 s,
  27 microseconds per entry.
  Eight parallel `rm -rf` workers and one worker measured the same (13.1 s to 13.7 s sequential,
  13.3 s to 14.0 s
  parallel,
  nine runs):
  unlinks in one subvolume do not scale with threads.
- Rename-to-trash variant:
  renaming the four ordinary nested trees,
  `.git` and the 479 skip-set trees into one directory
  inside the snapshot took 1.57 s to 1.60 s in total,
  of which 1.2 s is the `find` that lists them;
  emptying that
  directory
  afterwards took 14.2 s to 15.9 s.

### Options

- S1,
  snapshot,
  then delete everything left out in the foreground:
  0.15 s + 0.7 s (`.git`) + 14 s (skip set) + 0.5 s per
  ordinary nested worktree (about 25 s for 48) = about 40 s for the real main checkout,
  and about 15 s if every nested
  worktree were a subvolume.
- S2,
  snapshot,
  rename what is left out into a trash directory,
  return,
  and empty the trash afterwards:
  foreground about
  1.8 s (0.15 s snapshot,
  1.6 s listing and renaming,
  plus the nested pointers);
  background about 40 s of unlinking
  for the real
  checkout (14 s for the skip set,
  25 s for 48 ordinary trees),
  about 15 s if all nested worktrees were subvolumes.
- S3,
  convert the ordinary nested worktrees to subvolumes once,
  then S2 or S1:
  removes the 0.5 s per worktree from every
  add,
  and removes the reason the ladder declines (see `The ladder`).
- S4,
  make the cache and record directories nested subvolumes in main so a snapshot shows them as empty stubs (not
  measured):
  the removal cost would be a few microseconds per stub,
  but cargo,
  pnpm and anyone running `rm -rf` or
  `cargo clean` recreate the directories as ordinary ones,
  so the property would erode;
  I do not recommend it.
- A,
  the reflink stage already implemented on this branch:
  32 s at light load and a median of 89 s under load (see "The
  add itself"),
  no dependency on the filesystem,
  no ladder.

### What the worktree looks like until a background removal finishes

- With S2 the trash directory sits inside the new subvolume,
  because a rename across subvolumes fails with `EXDEV`;
  it
  is an
  untracked directory in the worktree root,
  so `git status` lists it until it is gone,
  unless it lives under a path Git
  already ignores (`.claude/` here) or the removal is foreground.
- Safe meanwhile:
  the left-out caches and run records are moved,
  not present,
  and the new worktree is otherwise final.
  Unsafe if left in place:
  the `.git` files of nested ordinary worktrees (they name the real worktrees' administrative
  directories,
  so a Git command run inside a copy would read and write another worktree's index and `HEAD`) and the
  copied `.git` directory.
  Both must be gone before the add returns;
  both are renames or 4 ms unlinks,
  measured above.
- A crash between the snapshot and the end of the removal leaves a trash directory and a worktree that is otherwise
  complete.
  This first design had the next invocation find the trash by its reserved name;
  that was rejected in the
  section "2026-11-03T00:08:39-05:00:
  review findings on the leave-out and its disposal",
  because a name proves nothing about who made a
  directory.
  The leftover is only disk under an ignored path.
  A crash before the swap into place leaves a stage subvolume that the ladder already deletes on recovery.

### One-time conversion of an ordinary nested worktree (fixture only; needs the owner's approval to run for real)

Procedure and cost for each of four fixture worktrees (`migrate-bench.ts`):

1.  `btrfs subvolume create <path>.migrating`:
    4 ms to 5 ms.
2.  `cp --archive --reflink=always <path>/. <path>.migrating/`:
    1.01 s to 1.02 s for about 22.5 thousand entries.
3.  `mv --no-target-directory --exchange <path> <path>.migrating`:
    4 ms;
    the path now names the subvolume and the old
    tree sits at `.migrating`.
    (Without `--no-target-directory`,
    `mv --exchange` treats the second name as a directory and fails;
    the first attempt
    showed that,
    and removed only the new copy.)
4.  `rm -rf <path>.migrating`:
    0.43 s to 0.50 s.
5.  Every worktree stayed registered (`git worktree list` named the path) and its status stayed clean.

- Cost per worktree:
  about 1.5 s of work,
  so about 72 s for the real 48,
  plus the index.
  The first `git status` after the copy took 5.0 s to 5.3 s (every file has a new inode,
  so Git re-hashes it;
  the
  fixture skips
  Git LFS smudging,
  so a real worktree with LFS files costs more;
  the earlier planning document measured 22 s for the
  main
  checkout) against 55 ms to 64 ms when the inodes were kept.
  The ladder's `migrate-to-subvolume` already rewrites the index stat fields (about 0.2 s per index in its document);
  I did
  not run it.
- Failure and recovery:
  the original is untouched until step 3.
  A crash after step 1 or 2 leaves `<path>.migrating`,
  which is
  deleted with `btrfs subvolume delete`;
  a crash after step 3 leaves the old tree at `<path>.migrating`,
  harmless,
  deleted
  by `rm -rf`.
  The registration never changes because the path and the `.git` file are the same bytes.
  The copy must be quiesced:
  edits made in the worktree during step 2 are lost at the swap,
  so the real procedure
  takes the
  worktree's `index.lock` for a final delta pass and swap,
  as the proven root migration does,
  and refuses while a
  process
  has its working directory inside.
- Needed at all?
  Not if S2 or S1 handles ordinary nested worktrees:
  their removal is the 0.5 s each measured above.
  It is needed only if the ladder's `snapshot_blocker` (which declines the snapshot rung while any nested registered
  worktree is an ordinary directory) stays as it is,
  or to save that 0.5 s per worktree per add.
  Relaxing the blocker is a change on the ladder branch,
  which I did not edit.

### The pnpm store problem (proposal only; no pnpm configuration was changed)

Measured in a subvolume worktree of the fixture against a reflink copy of main's store,
with `pnpm install
--frozen-lockfile --offline` (`pnpm-bench.ts`):

- Baseline:
  fails,
  `ERR_PNPM_NO_OFFLINE_TARBALL`;
  pnpm chose `<worktree>/node_modules/.pnpm-store/v11`.
- `package-import-method=clone` alone:
  fails the same way (the store is chosen before the import method matters).
- `--store-dir <absolute>` on the command line:
  passes in 1.2 s.
- `storeDir: <absolute path>` in `pnpm-workspace.yaml`:
  passes in 1.0 s.
- `storeDir: ../../../../store` (relative,
  resolved from the workspace directory):
  passes in 1.1 s;
  it only fits a fixed
  depth below the store.
- `storeDir: ${HOME}/.cache/snapmeas/store` (the variable expanded):
  passes in 1.2 s.
- `storeDir: ~/.cache/snapmeas/store`:
  fails;
  the tilde is not expanded and pnpm created a directory named `~` in the
  workspace.
- Environment variable `npm_config_store_dir`:
  fails (the setting is ignored;
  the per-project store is chosen).
- Proposal:
  one line in `pnpm-workspace.yaml`,
  `storeDir: ${HOME}/.local/share/pnpm/store` (pnpm's own default global
  store,
  named explicitly).
  Every worktree,
  whether ordinary or its own subvolume,
  then shares one store and imports by clone across subvolumes
  (the
  repository already sets `package-import-method` to `clone-or-copy`).
  Costs:
  the store must be populated once by an online install;
  the main checkout's installs move from the project
  store to the home store;
  one writable store is shared by every checkout of the user,
  which pnpm's documentation warns
  against for mutually untrusted agents.
  I tested the setting only against a copy of the store,
  with `--offline`.

### Ranking

Ranking:
S3 plus S2 > S2 > S1 > A > S4.

- S3 plus S2 over S2:
  converting the ordinary nested worktrees once removes 0.5 s of unlinking per worktree from every
  add
  and lets the ladder accept the snapshot without a blocker change;
  it costs a one-time run of about 72 s on the
  owner's real
  worktrees,
  which needs approval,
  and an index fix.
- S2 over S1:
  the foreground cost falls from about 40 s to about 1.8 s;
  the price is a trash directory in the worktree
  until a background removal ends and a process that must outlive the wrapper.
- S1 over A:
  about 40 s for the real checkout,
  in the foreground and deterministic,
  against 32 s to 141 s under load
  for A,
  with a removal of the new worktree in 0.07 s instead of 9 s to 41 s;
  and it keeps the worktree in one piece that the
  ladder already handles.
  The gap shrinks to nothing only when all nested worktrees are subvolumes and the owner accepts the skip set staying in
  the copy.
- A over S4:
  A works on any filesystem and has been run end to end;
  S4 is not measured and erodes.
- Against the owner's stated wish ("not so heavy because we can take advantage of btrfs"):
  S3 plus S2 returns in about
  2 s
  and finishes 15 s of unlinking later;
  A returns in 32 s at best.
- A and S* are not exclusive:
  A is the fallback when the source is not a subvolume or the snapshot is declined,
  which
  the
  ladder's rung order already expresses;
  this branch makes that fallback carry the ignored state.

## Applying the shared pnpm store

Owner decision,
2026-11-02:
"Set the shared store (Recommended)".
Measured on a throwaway clone with its own `HOME` (reflinked pnpm metadata cache),
then deleted;
the real home store,
main's
store and main's `node_modules` were only read.

- `pnpm-workspace.yaml` is a hand-edited file:
  `file-enforcer.config.jsonc` does not name it,
  so the change goes to
  the file
  itself.
- The exact edit,
  one line at the top level of `pnpm-workspace.yaml`,
  with a comment saying why:
  `storeDir: ${HOME}/.local/share/pnpm/store`.
  The `${HOME}` form expands;
  `~` does not (it created a directory named `~`),
  and the environment variable
  `npm_config_store_dir` is ignored.
- What happens to an existing checkout whose `node_modules` was linked from the old store,
  once the line is present:
  1.  every command except an install keeps working:
      `pnpm exec` ran against the old `node_modules` and passed,
      and
      `HOME`
      unset or pointing elsewhere did not matter to it.
      Correction (seen 2026-11-02 in the main checkout,
      whose `node_modules` was still linked from
      `node_modules/.pnpm-store/v11`):
      `pnpm update -r --no-save` stops with `ERR_PNPM_UNEXPECTED_STORE` until an
      install has relinked `node_modules` to the new store,
      so "every command except an install" holds for `exec` and
      `run` but not for every command;
  2.  `pnpm install --frozen-lockfile --offline` against a new store that does not hold the packages fails with
      `snapshot not present in local store` AND LEAVES `node_modules` EMPTY (only `.pnpm-workspace-state-v1.json`
      remained).
      pnpm removes the old links before it finds the packages missing;
      there is no "unexpected store" refusal.
      With `HOME` unset the same install fails the same way,
      so the setting needs `HOME` in every environment that
      installs
      (CI and containers included);
  3.  once the new store holds the packages,
      the same install re-links everything from it in 1.8 s to 2.9 s and records
      the new
      `storeDir` in `node_modules/.modules.yaml`.
- Filling the store,
  which must come first:
  - The real home store is `~/.local/share/pnpm/store/v11` (49 GB,
    a live pnpm 12 store with an `index.db`),
    so main's
    store cannot be copied over it:
    a copy would replace its index.
    On a fresh `HOME` a reflink copy of main's store took 2.1 s and worked.
  - `pnpm fetch --frozen-lockfile` fills the configured store from the network without linking,
    and took 10.7 s into
    an empty
    store;
    but it also empties the `node_modules` of the directory it runs in,
    so run it in a disposable worktree or
    clone
    that carries the edit,
    never in main.
- Reinstalling main is NOT safe while other sessions use main's `node_modules`:
  while the switch install ran (3.0 s),
  a loop
  resolving two installed packages failed 102 of 215 times.
  Run it when no session is working in main,
  or accept about 3 s of failures.
- Existing worktrees need nothing until they next run an install after receiving the edit;
  then they need the store
  filled
  first,
  or the install empties their `node_modules` as above.
- Order for the owner:
  edit on a branch;
  in a disposable worktree of that branch run `pnpm fetch --frozen-lockfile`
  once;
  merge;
  in main,
  with sessions idle,
  run `pnpm install --frozen-lockfile` (about 3 s);
  other worktrees pick it up at
  their next
  install.

## Conversion tool and the snapshot route (implemented on `feat/git-policy-cli-main-snapshot-worktree`)

Owner decisions,
2026-11-02:
"Convert once + snapshot (Recommended)" and "Set the shared store (Recommended)".
The branch starts at `btrfs-worktree-ladder` (head `38e01bcd33` when I merged) with `main-ignored-copy` merged in;
the
reflink
copy stays as the fallback rung,
because a main-sourced add that is not served by a snapshot still goes through the
lifecycle
copy.

### What was built

- `git cli-git convert-worktree-to-subvolume [--dry-run | --abandon] (--all | <worktree>...)`
  (`worktree_convert.rs`,
  `worktree_convert_command.rs`,
  `worktree_convert_arguments.rs`):
  quiesce,
  copy with cloned
  data into
  `<path>.cli-git-converting`,
  prove with a second pass,
  exchange with `renameat2`,
  refresh the index stat fields,
  remove the
  old tree only when it is the directory the journal recorded.
  The journal `cli-git-convert` lives in the worktree's administrative directory.
  A rerun completes after a kill at any phase,
  including between the exchange and its journal line (the old tree's
  recorded
  device and inode say where it is);
  `--abandon` destroys only the subvolume the journal says it created,
  before the
  exchange.
  It uses no Git process:
  the cleanliness check is the ladder's own in-process one without its untracked-file step
  (`check_clean_tracked`,
  split out of `check_clean`),
  because untracked files are copied along.
- A snapshot of the main checkout now serves the add even with ordinary nested worktrees (`worktree_trash.rs`):
  it
  unlinks the
  copied `.git` pointers,
  moves the nested ordinary worktrees,
  other checkouts,
  caches and run records into
  `.claude/cli-git-trash/<40 hex>` inside the stage by rename,
  and after the swap hands that directory to the commit
  service
  as a work item of the new kind `trash` (`work_item.rs`,
  `service_work.rs`:
  a default `trash` handler method,
  one
  partition in
  the drain loop).
  The place is used only if Git ignores it;
  otherwise,
  and when no route to the service exists or the
  wake
  fails,
  the wrapper removes the directories itself before returning.
  The section "2026-11-03T00:08:39-05:00:
  review findings on the leave-out
  and its disposal" changed what the work item carries and what the removal does when it fails:
  the item now stays until
  the trash is gone.

### Edits inside the ladder's files (for the merge with its recovery rework)

- `worktree_snapshot_source.rs`:
  a new field `SnapshotSource.trash_ignored`,
  computed in `common_source` for a main
  checkout
  (one `git check-ignore`);
  `snapshot_blocker` returns nothing for a main-checkout source after its through-a-link
  check.
  Its test `btrfs_each_condition_that_rules_a_source_out_is_refused_with_its_reason` now asserts that the main
  checkout is not
  blocked and a linked source still is.
- `worktree_snapshot.rs`,
  `take_snapshot`:
  one call,
  `worktree_trash::prune_snapshot(stage, source)`,
  between making
  the stage
  private and `remove_stubs`;
  it returns the manifest of what the leave-out made and moved.
- `worktree_ladder_finish.rs`,
  `serve_snapshot`:
  one call,
  `worktree_trash::dispose_pending_trash(...)`,
  after the
  swap and
  `settle_displaced`;
  it passes that manifest and reports a failure as a warning.
- Five binary tests changed because a main-sourced add now prints the copy's summary line and copies the ignored state
  when no
  snapshot serves it (`binary_worktree_ladder_tests.rs`,
  `binary_worktree_clone_tests.rs`,
  `binary_worktree_snapshot_tests.rs`).
- Not in the ladder's files but touching its seams:
  `entry.rs`,
  `worktree_copy_forward.rs`,
  `worktree_copy_registry.rs` and
  `worktree_copy.rs` (the main checkout is an applicable source for every creating command;
  the named-paths plumbing
  is gone,
  and `observe_copy_source_for` with its `serves_add` flag was folded into `observe_copy_source`).
- The ladder's rule "recovery never deletes what it cannot prove it created" is kept:
  the trash directory is made by
  `prune_snapshot`,
  which writes down its identity and the identity of every directory it moves in;
  the handler
  removes only objects with those identities,
  and the conversion removes only the directory whose device and
  inode its journal recorded.
  A stage that a crash leaves with a trash inside is the stage the ladder's recovery already owns;
  this module adds
  nothing it
  would have to delete.

### Measured on a fixture

The fixture is a main-like subvolume with a copy of main's `.git`,
693 thousand ignored entries,
and four ordinary and four
subvolume nested worktrees of 22.3 thousand entries each;
release build,
scope `CPUQuota=800%`,
load average 6 to 10.

- Conversion,
  real binary,
  `--dry-run --all` over eight worktrees:
  0.86 s.
  The real `--all`:
  16.65 s for the four
  ordinary
  worktrees,
  4.2 s each (the copy,
  the proving pass and the old-tree removal);
  about 200 s for the 48 on the real
  checkout,
  run
  once.
- A main-sourced add with four ordinary nested worktrees present:
  the snapshot rung served it;
  5.7 s to the end of the
  leave-out,
  then,
  with no route to the service,
  31 s of removal in the foreground (37.7 s in all).
- The same add after the conversion:
  3.4 s from the command to the point where the trash is handed over;
  the removal,
  which
  the service does in the background (13.9 s measured here as the foreground fallback),
  is no longer in the command.
  The new worktree:
  `git status` 0.06 s and clean,
  `node_modules`,
  `target/` and `.cache/` present,
  `node_modules/.cache`,
  `.monochromatic` and `target/debug/incremental` absent,
  `.claude/worktrees` empty.
- Limit:
  the hand-over to the real service was controlled with an injected route;
  the end-to-end runs had no route,
  because
  a service is looked up by the new worktree's own Git directory and the fixture has no session bus.
  In production the
  route
  finder returns the managed address and the wake starts the service.

### Guards shown red

Each change was made after committing the tests and then restored:
the exchange not reconciled with the journal
(`abandon_undoes_before_the_swap_and_refuses_after`),
the old tree removed without the identity check
(`a_directory_that_is_not_the_recorded_old_tree_is_never_removed`),
no proving pass
(`a_change_during_the_copy_is_caught_before_the_swap`),
the copied `.git` pointer not unlinked and tracked directories
not
protected (`btrfs_a_snapshot_of_main_sets_aside_exactly_what_the_rule_leaves_out`),
the ordinary nested worktree
blocking the
main checkout again (two tests),
and the handler following a link at the trash name
(`the_handler_empties_only_the_trash_it_is_told_about`).

### Order for the owner

1.  Merge this branch (it contains the ladder and the copy) after the ladder's recovery rework lands;
    release.
2.  Stop sessions inside the nested worktrees;
    run `git cli-git convert-worktree-to-subvolume --dry-run --all` from the
    main
    checkout and read the skip reasons;
    then run it without `--dry-run`,
    worktree by worktree or with `--all`.
3.  Apply the shared pnpm store (section "Applying the shared pnpm store").

### 2026-11-02T22:26:50-05:00: merge of main, the snapshot message, and the earlier-release service

- `main` was merged into the branch with `git merge` (it holds the ladder head `3681539d6b` as `c0d7bf8bf3`,
  and later
  fixes).
  The only conflict,
  in `doc/handover/cli-git-rust-implementation.md`,
  was resolved by keeping both sides of each
  hunk.
  The ladder merge before it had one compile fallout:
  `still_held` in `service_work.rs` now takes the work
  directory,
  and the drain loop passes it.
  The trash handler and the conversion use `remove_tree` and
  `remove_if_identical`,
  which remain;
  the rule "delete only what this process proves it created" is unchanged.
- A main-sourced add that took the snapshot printed "copied 0",
  because no copy ran.
  It now prints nothing from the copy
  and,
  unless `--quiet`,
  one line on standard error (`MAIN_SNAPSHOT_NOTE` in `worktree_trash.rs`) that says the worktree
  was made as a snapshot of the main checkout and that nested worktrees,
  caches and run records were left out.
  `a_snapshot_of_main_sets_aside_what_the_rule_leaves_out` asserts that standard error is empty under `--quiet`.
- Earlier-release service:
  `wake` goes through `reach_current`,
  which compares the announced protocol version and release
  and sends a retire to any other;
  it ends with `OtherRelease` after the start wait.
  The hand-over is therefore never made
  to a service that does not claim `trash` items.
  Two controls with a fixture service on an abstract socket:
  `a_service_of_an_earlier_release_that_will_not_retire_is_never_handed_trash` (it hears only retires,
  the trash is
  emptied in the foreground,
  no pending record is left) and
  `a_service_of_an_earlier_release_is_replaced_before_trash_is_handed_over` (the work message reaches the current release
  after the retire;
  the item waits for it).
  `dispose_pending_trash_with` takes the start wait so the first control needs
  400 ms,
  not the production ten seconds.
- Both controls were shown red by removing the release comparison from `reach_current_untimed` (a service of the current
  protocol but another release was then accepted);
  restoring it turned them green.

### 2026-11-02T23:10:49-05:00: the conversion tool refused clean worktrees (0.5.18)

- The dry run on the real repository listed 28 worktrees as not quiet that were clean.
  The log of that run is
  `convert-dry-run.log` in the scratch directory.
- Sixteen were refused with "HEAD is not a plain ref to a commit".
  They were not detached:
  their `HEAD` names a branch that
  only `packed-refs` holds,
  and `packed-refs` of the repository has tags with an `@` in the name
  (`@monochromatic-dev/module-logger@0.4.1`),
  which is outside the plain alphabet of `repo_read_refs.rs`;
  `packed()` returned
  nothing for the whole file,
  so no ref resolved.
  A detached `HEAD` already converted (the existing conversion tests use one).
  A lookup now leaves out a well-formed line with an unusual name;
  the list and a damaged line still end the read.
- Fifteen were refused as racily clean and two for a stale stat.
  `worktree_clean_content.rs` settles those by content in
  process:
  blob hash of the file,
  a link by its target,
  compared with the index entry's object name.
  The reasons that stay
  refusals:
  a `filter`,
  `ident` or `working-tree-encoding` attribute that may apply (attribute sources:
  tracked
  `.gitattributes` down the path,
  `info/attributes`,
  `core.attributesFile` or the default global file,
  the system file,
  macros
  that set one of them),
  a carriage return in the file (a line-ending conversion strips it on the way into the index;
  a file
  without one is unchanged by `text`,
  `eol`,
  `core.autocrlf` and `core.eol`,
  so those need no attribute lookup),
  a file over
  32 MiB,
  and a type,
  mode or assume-unchanged difference.
- The real repository's `* text=auto eol=lf` therefore refuses nothing by itself,
  and `*.png filter=lfs` refuses only a stale
  png.
- Only the conversion uses it (`check_clean_tracked_by_content`);
  the snapshot and clone rungs keep the stat-only check
  because they install the index they proved.
  After a conversion an entry that was not proven keeps its old stat data
  (`refresh_index_file_with` rewrites only entries whose old file matched),
  so Git hashes it at its first status.
- `Platform::config_scope` gives the conversion the configuration files Git reads;
  a stand-in names none,
  so controls read
  only the repository's own configuration.
  When the system configuration path is unknown (no forward target in the
  environment) the global attributes file cannot be ruled out and the content decision refuses with that reason.
- Dry run of the fixed build on the real repository:
  160 worktrees,
  94 already subvolumes,
  65 would convert,
  1 skipped
  (`package/forbidden-strings/core/Cargo.lock` really differs from the committed content).

### 2026-11-02T23:51:49-05:00: the cross-vendor reviews of the content proof and the conversion

Every finding was checked against the Git source of the installed version (`attr.c` at `v2.56.0`) before it was fixed.

- System attributes (`attr.c`,
  `git_attr_system_file`):
  `system_path(ETC_GITATTRIBUTES)`,
  independent of
  `GIT_CONFIG_SYSTEM`,
  off under `GIT_ATTR_NOSYSTEM`.
  The path is now asked of the executable with `git var GIT_ATTR_SYSTEM`
  (cached per executable identity beside the system configuration path) and an unknown place refuses.
- Lexing (`attr.c`,
  `parse_attr_line`):
  the blanks are `" \t\r\n"` only,
  a line is a C string (NUL ends it),
  and
  `read_attr_from_file` skips a UTF-8 byte order mark on its first line;
  none of this was modelled.
  Paths,
  patterns and
  directory keys are bytes now,
  with ASCII folding.
  A quoted pattern keeps all its tokens and counts as applying.
- Reads:
  `worktree_clean_content.rs` opens every file and every `.gitattributes` below a held root with `openat2` and no link
  followed,
  judges type and mode on the handle,
  compares the state before and after the read,
  and bounds the read at the limit
  plus one byte.
  A `.gitattributes` that is a link is a refusal (Git opens it without following).
- Conversion:
  the journal is created and replaced only through exclusive files,
  records the identity of the stage,
  and is
  guarded by a `flock` per administrative directory;
  `Synced` is provisional;
  the old tree is removed beneath a pinned
  descriptor;
  nested registrations are read from the common directory;
  refusals have kinds;
  a failed refresh keeps the old tree.
- What the cleanliness proof is for:
  the conversion copies the whole tree,
  untracked and ignored files included,
  so a modified
  tracked file would survive the conversion.
  The proof is a policy (do not rebuild a worktree someone is in the middle of),
  and
  the data is protected by the copy,
  the second pass,
  the identity checks,
  and the update-only pass before the removal.
- What stays open:
  a write that keeps the size and modification time of the copy is invisible to the second pass and to the
  update-only pass;
  a process that writes and exits in the window before the exchange is found only by the update-only pass;
  a
  system attributes file under a prefix other than the executable's is not read.

### 2026-11-03T00:08:39-05:00: review findings on the leave-out and its disposal

A cross-vendor review of `worktree_trash.rs` made seven findings.
Each was checked against the code before it was
trusted;
the owner rules that decided every case were "never discard user data" (delete only what this process can
prove it created or moved there) and "a loud,
clear failure" over a silent wrong result.

- Finding 9,
  a registered nested worktree that holds a tracked file:
  confirmed in `prune_snapshot`,
  which moved the
  directory and lost the tracked file.
  The later check of the stage against the index already declined the rung with
  "the source changed while it was copied",
  so nothing was lost silently,
  but the reason was misleading.
  The leave-out
  now works out what the index tracks first and refuses before it unlinks a pointer or moves a directory,
  naming the
  worktree and the tracked file.
- Finding 10,
  tracked gitlinks:
  the defect is real in the function,
  but not reachable through the wrapper:
  `check_clean` refuses a source whose index holds a submodule before any snapshot is taken.
  The protected set now
  also holds every directory-valued index entry at its exact path,
  as hardening,
  and a control calls the function
  with an index that holds a gitlink.
- Finding 11,
  a name proves ownership:
  confirmed.
  A directory named like a trash,
  carried by the snapshot from the
  main checkout's own `.claude/cli-git-trash`,
  was emptied.
  `prune_snapshot` now returns a manifest (the trash
  directory's identity and that of each directory moved into it,
  as device,
  inode and birth time),
  `take_snapshot`
  hands it on,
  and the work item carries it in its `parent` field (a repurposed free-text field;
  `oid` stays the
  trash name).
  The disposal opens the trash,
  compares its identity,
  removes an entry only when it is the recorded
  object,
  and leaves anything unrecorded alone with a reason.
  Nothing scans the trash parent any more.
- Finding 12,
  ancestor links and check-then-use:
  confirmed,
  and wider than reported:
  `create_dir_all` in the
  leave-out also followed a link at `.claude` in the snapshot.
  Every directory from the worktree down to the trash is
  now opened by one plain name below the handle of the one above (`openat2` with no links,
  no mounts,
  no `..`),
  the
  worktree's `.git` file is read the same way and must point at the record's administrative directory,
  and removal
  is done from held handles (`worktree_trash_remove.rs`,
  which does not edit `fs_beneath.rs`).
  When the leave-out
  cannot make its trash as real directories,
  it removes the left-out directories in place instead of failing the rung.
- Finding 13,
  lossy names:
  confirmed (a directory named by byte `0xff` beside a link named with the replacement
  character made the walk enter the link's target and unlink a `.git` file there).
  Names are now kept as bytes for
  every operation and every comparison with the index.
  The leave-out rules still read a lossy text of a name,
  which is
  sound because every name they compare with is ASCII,
  so a name that is not UTF-8 never equals one.
- Finding 14,
  lossy paths in the record:
  confirmed.
  A destination or Git directory that is not UTF-8 is now not
  enqueued;
  its trash is emptied in the foreground by the bytes of the path.
- Finding 15,
  errors turned into success:
  confirmed.
  The disposal returns the reason (path and cause),
  the ladder
  prints it as a warning (the worktree is already in place and complete,
  so it does not fail the add),
  and the pending
  record is withdrawn only after the trash is gone;
  on a failure it stays for the service.
- A real add of a main snapshot spent about ten seconds waiting for the service and then 4.7 minutes emptying the trash in
  the foreground,
  with nothing said (the log of that run is in `/var/home/user/temp/agent/slow-route/add.err`).
  Cause:
  each
  worktree has a service instance of its own,
  and its socket is created by provisioning,
  which only a commit did;
  the
  hand-over ignored the route's provisioning step,
  so the wake waited out the whole start wait for a socket that nothing
  would create.
  The hand-over now provisions first,
  as a commit does (`the_sockets_of_a_new_worktree_are_set_up_before_the_service_is_woken`,
  shown red by skipping the step),
  and a failure of the route,
  the provisioning or the wake is rendered with the client's
  own words in one plain line before the foreground removal.
  The foreground walk itself measured 3.4 s against 3.8 s for
  the old `remove_tree` on a generated tree of 160 thousand files (three alternating rounds,
  same spread);
  it cannot make
  a removal of hundreds of thousands of entries short,
  which is why the service must take it.
  The end-to-end add was not
  re-timed:
  it needs the real session bus,
  which a control may not touch.
- Not done:
  a crash after the swap and before the disposal leaves a trash directory with no record.
  It is only disk
  under an ignored path,
  and nothing may find it by name.

Timing,
`prune_snapshot` alone on a generated tree of about eight thousand directories with 480 set aside,
six
alternating rounds on btrfs:
before 143 to 152 ms,
after 148 to 154 ms (means 147 and 151).
The end-to-end figure of the
add (about 1.85 s from main) was not measured again.

Guards shown red after committing the tests,
each by disabling it and restoring the file:
the tracked check of a
registered nested worktree,
the exact-path protection of a gitlink,
the trash identity check,
the entry identity checks
(both together),
the check for unrecorded entries,
the check of the `.git` file,
a following open instead of a
held-handle open in the disposal,
in the removal and in the making of the trash,
byte names in the walk,
the
refusal to spell a non-UTF-8 path in a record,
the error return on a failed cleanup,
and the withdrawal of the record
before the cleanup.

### 2026-11-03T00:57:00-05:00: deterministic review findings on the content check and the conversion

Scope decision from the owner,
verbatim:
"That is not an our problem.
By using this command or this program,
make the user
understand and accept the risk."
Races caused by other processes are out of scope for the whole wrapper:
a rename or replace
mid-operation,
writes through descriptors or shared mappings,
and edits between a check and the step that relies on it.
The
risk is stated in the help text,
in one notice on standard error before a conversion runs,
in the README and the SPEC.
The cheap
guards stay and no machinery is built for races.
The two reviews are `review-content-2/sol-review.txt` and
`review-convert-2/sol-review.txt` in the scratch directory;
their races are listed under what stays open.

- Content check,
  fixed:
  every quoted attributes line refuses at every source level (the macro marker in a value and a token
  glued to the closing quote were both lost);
  `GIT_ATTR_SOURCE`,
  `attr.tree` and a command-line `--attr-source` or
  `-c attr.tree` refuse the shortcut;
  the stat-only proof walks below a held root;
  a named pipe as an attributes file is found by
  its type;
  the macro closure uses reverse edges and a queue;
  a comparison past its work budget answers "may apply".
- Conversion,
  fixed:
  an unreadable registration listing is a refusal;
  a kept old tree keeps the journal and ends the run as
  incomplete;
  the old root's identity includes its birth time;
  a journal for another path refuses a moved worktree;
  a named pipe
  as the journal,
  a registration file or `.git` no longer blocks while the lock is held;
  a dry run reconciles an unrecorded
  exchange in memory.
- The pass that copied newer old-tree files over the installed tree is gone.
  It existed only for concurrent writers and could
  overwrite an edit.
  A read-only comparison replaces it:
  a difference,
  a holder or a subvolume or mount inside the old tree
  keeps the old tree and is named;
  nothing is copied.
- `remove_tree` on the remove-path branch refuses nested subvolumes and mounts for every caller;
  the conversion now says it in
  its own words before it gets there,
  by naming the path in the comparison.
- Accepted risk,
  not fixed:
  the review items about a stage replaced between creation and recording,
  a journal target replaced
  after loading,
  a final unlink of a foreign empty directory,
  writers that act after the last check,
  and writable shared
  mappings are races with other processes.

## Assumptions and limits

- The fixture is `git clone --shared` of the main checkout in `/var/home/user/.cache/main-ignored-copy`,
  with the main
  checkout's ignored roots reflinked into it (`cp --archive --reflink=always`,
  693 thousand entries,
  no `.claude`).
  It sits in the home subvolume,
  not the main checkout's subvolume,
  so a link from stage to destination stays inside one
  subvolume as in the real case,
  but the clone source is across subvolumes.
- Git LFS smudging was skipped for the fixture (`GIT_LFS_SKIP_SMUDGE=1`);
  the real checkout's LFS files are checked
  out,
  which
  adds nothing to the ignored copy.
- Load was never quiet:
  every number carries the load it ran under,
  and arms were alternated with a control.
- The container gate was not run;
  the host library and binary test sets and clippy were.
- A registered worktree that is itself a subvolume appears in a snapshot as an empty stub;
  I did not run the ladder's
  binary (see "The ladder"),
  I read its code at `0e432cd0b3`.
- The stamp argument rests on kernel behavior I did not test with a concurrent writer;
  the unit tests cover a changed
  file
  and the byte-comparison fallback,
  not a race.

## Options considered

Ranking:
A > D > B > C > E,
with the reason for each adjacent pair after the list.

- A,
  the chosen design:
  the reflink stage under the leave-out rule,
  verified by stamps,
  cloned and installed in
  parallel.
  Pros:
  works on any filesystem (byte copy on ext4),
  keeps the lock,
  journal and recovery,
  needs no configuration,
  copies
  what the owner listed.
  Cons:
  cost grows with the entry count;
  it is the slowest of the correct mechanisms on btrfs;
  the verification weakens
  from bytes to stamps for cloned files.
  Cost:
  see "The add itself".
  The owner sees:
  one summary line,
  "copied N ignored filesystem entries from ...",
  and a "left out" line when files
  were
  unreadable.
- D,
  add the install-by-rename of whole roots on top of A (not built):
  rename `stage/payload/<root>` to the
  destination when
  that destination root is absent.
  Pros:
  the install and the stage removal become a handful of renames,
  removing most of the install and removal time.
  Cons:
  the journal,
  install log,
  recovery and rollback all assume entries linked from a stage that stays intact;
  a root
  moved out of the stage needs its own rollback (rename back) and its own recovery rule.
  Cost:
  unmeasured,
  estimated from the phase times below as the larger part of install and removal.
  The owner would see the same output,
  faster.
- B,
  the btrfs snapshot rung (`btrfs-worktree-ladder`):
  one `btrfs subvolume snapshot` of the main checkout.
  Pros:
  seconds,
  shares all blocks.
  Cons:
  it carries caches and run records the owner asked to skip and the nested ordinary-directory worktrees it
  declines
  on;
  it also needs the destination on the same btrfs filesystem.
  Cost:
  the ladder agent measured 15.15 s to 2.43 s for a main-checkout add.
  The owner would see a fast add that includes everything ignored.
- C,
  run `pnpm install` for the new worktree:
  rejected by the owner's constraint (package managers stay out of the
  wrapper),
  and it does not carry builds.
  Measured 1.8 s to 2.1 s offline when the store is reachable,
  which is why it stays a good manual step.
- E,
  symbolic links to the main checkout's `node_modules` (worktrunk's and VS Code's option):
  not built and not
  measured here.
  By construction a worktree's `node_modules` would resolve through links into the main checkout,
  so workspace packages
  would resolve to main's sources,
  not the worktree's;
  that is a reason to reject it,
  but I did not run a demonstration.

Reasons:
A over D is only about risk,
D is faster but changes the journal contract and needs its own fault tests,
so A
first.
D over B because B copies caches and records and declines when any nested worktree is an ordinary directory.
B over C because B carries builds and C does not.
C over E because C produces a correct,
independent tree and E shares mutable state between worktrees.

## The ladder

Read at `0e432cd0b3` on `btrfs-worktree-ladder` (code in flux,
not edited).

- `worktree_ladder_run.rs`,
  `choose_rung`,
  takes the snapshot rung only when the source is a subvolume root,
  `snapshot_blocker` finds no nested registered worktree that is an ordinary directory,
  and the clean check passes.
  The main checkout is a subvolume but has 48 ordinary-directory nested worktrees,
  so the snapshot rung declines today
  and
  the clone rung runs:
  it clones tracked files only (`worktree_clone.rs`) and reports nothing as carried
  (`LadderResult { carried: None }`),
  so the lifecycle's own copy then runs for the destination.
  With this change that copy now carries the ignored state,
  so a main-sourced add on this machine will do clone rung
  plus
  this copy until the 48 worktrees are removed or migrated.
- When both merge,
  the ladder needs three things from this change:
  1.  `carried` is the destination whose ignored state a snapshot already carried;
      the lifecycle skips that destination.
      For a main-sourced snapshot the carried state is the whole ignored set,
      caches and run records included,
      while this
      change defines a smaller set for main.
      Either the snapshot rung removes the left-out names from the new subvolume with `Prune::MainCheckout` (the rule
      is a
      pure function of paths) or the owner accepts the larger set there.
  2.  The clone and fallback rungs must leave the destination as Git's checkout does,
      with the lifecycle's copy still to
      come;
      the fixed `StagedSnapshot.unreadable` and the new `CopySelection::MainCheckout` do not change that contract.
  3.  Hard links from the stage into a destination that is its own subvolume fail with `EXDEV`;
      `install_file` already
      falls
      back to an exclusive copy-on-write request,
      which keeps the data shared,
      but the install loses the hard-link
      atomicity and the stage removal then deletes real files;
      a rename-based install (option D) is the better fit there.
- The pnpm finding matters to the ladder:
  every worktree it creates is its own subvolume,
  where pnpm's default store
  choice makes `--offline` fail.
  A project-level `storeDir` in `pnpm-workspace.yaml` naming one store on the main subvolume would avoid it;
  I did not
  change pnpm configuration.

## Snapshotting a dirty main checkout (proposal, not built)

Status:
proposal for the owner's decision,
2026-11-03.
The owner chose option B,
and the section
"Snapshot of a main checkout with uncommitted changes:
built,
proved and measured" records what was built and measured.

### The question

Two adds from the main checkout took Git's checkout and the ignored-state copy,
for minutes,
instead of the snapshot,
for
seconds,
because the clean check refused the main checkout.
The main checkout is where the owner works,
so it is often not clean:
this machine commits roughly once a minute,
and a
file edited a moment ago,
an untracked note,
or the `index.lock` of a commit in flight each refuse the snapshot.
Can a dirty main checkout still be snapshotted,
with the new worktree made equal to what Git itself would produce?

### What Git itself produces

- `git worktree add -b topic <path> main` checks out the tree of the commit `main` names:
  `HEAD` is that commit,
  the
  index equals its tree,
  every tracked file has the committed bytes and mode,
  and there is no untracked file.
  Nothing uncommitted in the creating worktree is ever carried,
  staged or not.
- Ignored files are absent from Git's result.
  The wrapper adds the ignored state afterwards,
  and a snapshot carries it.
  The restore below must therefore remove untracked files that are not ignored and keep every ignored one.
- A requested commit-ish other than the source's `HEAD` is handled by the step the snapshot rung already has (the tree
  difference between the source's commit and the new commit).

### The candidate restore

1.  Take the snapshot as today,
    then keep the source's own index from the snapshot (`.git/index` inside it,
    read before
    `.git` is replaced by the new worktree's pointer),
    not the index bytes the clean check read earlier.
    The snapshot is one instant,
    so the index and the files in it agree even if the person keeps working in the source.
2.  Instead of `git read-tree -m -u <source commit> <new commit>`,
    run `git read-tree --reset -u <new commit>` under
    the new
    worktree's Git directory and work tree,
    which makes the index and every tracked file equal to the new commit's tree,
    whatever the source's index and files held.
3.  Run `git clean -f -d` there,
    without `-x`,
    which removes untracked files that are not ignored and empty untracked
    directories,
    and keeps ignored files.
4.  Prove the result (below);
    on any doubt destroy the stage and take the slow route,
    as every failure of the rung does
    today.

Why step 2 can work from the source's index:
in a btrfs snapshot the inode number,
the modification time and the
change time
of every file equal the source's,
and only the device number differs,
which Git's index does not compare.
Measured on a throwaway subvolume of 3000 tracked files with a modified file,
a file staged then modified again,
a
staged
deletion,
an unstaged deletion,
a staged rename,
a mode change,
an untracked file and an untracked directory:
`git status`
in the snapshot printed the same eight lines as in the source,
5 ms,
with no refresh.
So the stat data in the copied index still marks exactly the files that were edited,
and `read-tree --reset -u` rewrites
exactly those and leaves the rest.
The wrapper's current snapshot path installs the source's index "with refreshed stat fields" (`README.md`,
ladder rung
1;
`rewrite_index_stat_with` in `SPEC.md`),
which for a clean source is right and for a dirty one would record the edited
files as unchanged and hide the edits,
so this option must not reuse that step.

### Measured

- Fixture one (3000 files,
  the eight changes above):
  after `git reset --hard` and `git clean -fd` in the snapshot,
  12
  ms,
  `diff -r --exclude=.git --exclude=ignored` against a worktree made by `git worktree add` showed no difference,
  the
  executable bit was restored,
  `ignored/pkg/x` stayed,
  `git status` was empty.
- Fixture two (500 files,
  two commits,
  the same changes plus an edit over a file the second commit added):
  after
  `git read-tree --reset -u <first commit>` and `git clean -fd` in the snapshot,
  `diff -r` against a detached worktree
  at
  the first commit showed no difference,
  and `git write-tree` printed the same tree as that worktree.
- Wall time of that step was 1.03 s with 0.00 s of CPU.
  The cause is in the trace:
  the user's global configuration has
  `core.fsmonitor=true`,
  so the first Git command in a new snapshot started `git fsmonitor--daemon` and waited for it.
  The wrapper's own Git runs do not pin it today:
  `src/native/hooks_off.rs` says `core.fsmonitor` "is left as configured".
  So a Git run of the shipped snapshot rung in a new worktree can pay the same start and leave a daemon behind (my fixtures
  left one per snapshot until I stopped them);
  I did not measure that on the shipped rung.
  This step should pin `core.fsmonitor=false`.
- On the real main checkout,
  read only:
  `git --no-optional-locks status --porcelain=v1 -z --untracked-files=normal` took
  26 ms and `git clean -n -d` 57 ms,
  with 20445 tracked files.
  So the dirty-set discovery is cheap here;
  the cost that remains is rewriting the changed files.
- Not measured:
  the whole step on 20445 files with a realistic number of changes,
  and any run of the real ladder with
  the
  restore (it does not exist).

### Correctness hazards and what the candidate does with each

- Staged versus unstaged changes,
  and a file that is both:
  the index is replaced wholesale,
  so every combination ends
  equal
  to the new tree.
  Measured with a file staged then modified again.
- Deleted files,
  staged or not:
  the entry is in the new tree,
  so it is written back.
  Measured for both.
- Renames:
  Git stores a delete and an add,
  so the new name is removed (it is not in the new tree) and the old one
  restored.
  Measured with a staged rename.
- Untracked files:
  removed by `git clean`;
  the ignored ones stay.
  Measured.
  An untracked directory that holds only
  ignored
  files stays too.
- Empty untracked directories:
  `git clean -d` removes them.
  The real main checkout holds ten today (`git clean -n -d`
  lists them and `git status` does not),
  and the clean snapshot of today carries them although Git's checkout would not
  make them.
  This is an existing difference of the snapshot rung,
  not a cost of the proposal;
  the restore would fix it.
- Mode changes:
  restored while `core.fileMode` is on.
  With it off Git cannot see a mode change in the source either,
  so
  the new worktree would keep it;
  the proof below does not catch that.
- Files that Git cannot see as changed:
  assume-unchanged,
  skip-worktree,
  intent-to-add and unmerged entries,
  a sparse
  checkout,
  an operation in progress,
  a submodule.
  These stay refusals.
  Today the clean check folds them with ordinary
  changes,
  so the check in `worktree_clean.rs` must be split into hard refusals and the changes the restore handles.
- Racy timestamps:
  a file written in the second the index was written is treated by Git as possibly modified,
  so the
  restore rewrites it;
  the cost is a rewrite and the result is right.
- Smudge and clean filters,
  end-of-line conversion,
  Git LFS:
  files the restore rewrites go through Git's checkout,
  so
  the
  filters run as for any checkout.
  Not tested.
- Untracked nested repositories:
  `git clean -f -d` skips them (it needs a second `-f`).
  The main-checkout snapshot
  already
  sets aside every directory that holds a `.git` file,
  so none reaches the restore;
  if that rule changes,
  this becomes
  a hole.
- The person keeps working while the add runs:
  nothing in the restore reads the source after the snapshot,
  so a later
  edit
  cannot reach the stage;
  an edit between the clean check and the snapshot is why the index must come from the snapshot.
- Hooks:
  none run,
  as on every rung.

### How the restore is proven complete

The proof runs in the stage before the swap,
from the stage alone:

1.  `git write-tree` under the new Git directory prints the new commit's tree,
    so the index is exact.
2.  `git ls-files --others --exclude-standard` prints nothing,
    so no untracked file that Git shows is left.
3.  `git diff-files --quiet` after a refresh of the stat data prints nothing,
    so every tracked file equals its index
    entry
    as far as Git can tell.
    This is what the clean check already relies on for a clean source.
4.  For the owner's confidence,
    not in the add:
    a guard test per hazard above that builds the state in a fixture,
    adds a
    worktree through the wrapper and through real Git,
    and compares the trees byte by byte (the way the two measured
    fixtures were compared).
    Each guard is shown red by removing the `git clean` step or the `--reset`.

What the proof cannot show:
an edit that kept the size and both timestamps.
Git's own `status` in the source cannot
show it
either,
so "what Git itself would produce" has no sharper definition there.
A full proof would hash every tracked
file,
which
reads the whole tree and gives up most of what the snapshot saves;
it can stay an option behind a flag.

### Cost

- Added to a clean snapshot add:
  one `git read-tree --reset -u` instead of `-m -u`,
  one `git clean -f -d`,
  three proof
  commands,
  and the rewrite of the changed files.
  For a handful of changed files this is milliseconds plus the process
  starts,
  against minutes for the slow route.
- Unchanged:
  the snapshot (0.15 s measured in "Snapshot route measured"),
  the leave-out of nested worktrees and caches
  (the cost that dominates today;
  my own add on 2026-11-02,
  a snapshot,
  spent from 23:38:50 to 23:43:31 emptying the
  trash in the foreground
  after the debug line "could not be handed to the service;
  emptying it here",
  a separate problem recorded in the
  report that
  came with this proposal).
- Removed for the dirty case:
  the slow route,
  about 190 thousand ignored entries copied after Git's checkout.

### Options

- A,
  keep refusing and say why (built on this branch).
  Pros:
  no new risk;
  the add is correct whatever the source holds;
  the person now knows the cause and the remedy.
  Cons:
  every add from a dirty main checkout is the slow route,
  minutes instead of seconds;
  the main checkout is dirty
  often.
- B,
  snapshot anyway and restore with Git's own `read-tree --reset -u` and `clean`,
  proven in the stage,
  falling to A
  on any doubt.
  Pros:
  seconds in the common case;
  Git does the semantics (filters,
  modes,
  deletions);
  nothing in the person's
  checkout is
  touched;
  it is additive to A,
  so a failure costs only the slow route.
  Cons:
  new code in the ladder's target step and a split of the clean check,
  both in files others are editing;
  the proof
  has a stated blind spot;
  the fsmonitor behavior must be pinned;
  it carries untracked ignored state of a dirty tree by
  design (that is the point).
- C,
  snapshot and restore by the wrapper's own file operations (compute the dirty set,
  rewrite and delete those paths).
  Pros:
  no dependence on Git's reset semantics.
  Cons:
  reimplements checkout semantics (filters,
  end-of-line conversion,
  LFS,
  modes,
  symbolic links,
  directory and file
  swaps) that Git already has and the wrapper otherwise never reimplements;
  more hazards,
  more tests.
- D,
  a clean spare maintained by the service:
  a snapshot of a clean state of the main checkout kept ready,
  taken
  whenever
  the main checkout is clean (or built from the default branch in a scratch subvolume),
  and used as the source for adds.
  Pros:
  never reads the person's dirty tree;
  adds are always the clean-snapshot path.
  Cons:
  a long-lived component and a refresh policy;
  the spare's ignored state (`node_modules`,
  builds) is as old as its
  last refresh,
  which breaks "immediately functional";
  storage for the spare;
  the owner prefers the operating system to
  start things on demand.
- E,
  commit or stash the person's changes before the add and put them back after.
  Pros:
  the source is clean by construction.
  Cons:
  it rewrites the person's work and history;
  an interrupted add leaves their changes stashed;
  rejected by the rule
  that the wrapper never changes work it did not make.

Ranking:
B > A > D > C > E.

- B over A:
  the same correctness as A in the cases B accepts and seconds instead of minutes;
  B is also additive,
  since
  every
  doubt falls to A,
  so choosing B loses nothing A gives.
- A over D:
  A is built and correct,
  while D adds a refresh policy and a component,
  and a stale spare returns a worktree
  whose `node_modules` and builds are out of date,
  which is the thing the owner wants avoided.
- D over C:
  D reuses the proven clean-snapshot path unchanged;
  C writes a second implementation of Git's checkout whose
  mistakes would be silent differences in the new worktree.
- C over E:
  C leaves the person's checkout alone;
  E modifies the user's work,
  which no rung may do.

### Decision for the owner

Whether to build B.
If yes,
the work is:
split the cleanliness verdict in `worktree_clean.rs` into refusals that stay
and
changes the restore handles;
take the index from the snapshot;
add the reset,
clean and proof steps to the target step
in
`worktree_ladder_target.rs` with the guards in `How the restore is proven complete`;
pin `core.fsmonitor=false` for those Git runs.
Two files named here are being edited by other agents at the time of writing,
so this proposal touches neither.

## 2026-11-03T01:09:00-05:00 Snapshot of a main checkout with uncommitted changes: built, proved and measured

The owner chose option B ("B:
snapshot,
then reset") from the proposal "Snapshotting a dirty main checkout (proposal,
not
built)".
Branch `feat/git-policy-cli-snapshot-dirty-main`,
crate `package/git-policy/cli`,
Linux on btrfs.
The owner scoped races out:
a change made to the main checkout while the snapshot is taken or restored is the person's
accepted risk,
and nothing here defends against it.
What stays in scope is that the new worktree equals what a clean
`git worktree add` makes.

### What was built

- `check_clean` returns a refusal with a kind and a reason.
  The kinds are an operation in progress (with the mark's
  name),
  an
  unreadable index or object database,
  an entry no reset can undo,
  a staged change,
  a changed tracked file,
  an index
  entry the
  stat data cannot vouch for,
  and an untracked file.
  Code decides by kind;
  the sentence for the person is chosen by
  kind.
  The conversion's own callers keep their string reasons through wrappers,
  and their controls stayed green.
- A main checkout whose only trouble is an ordinary change is snapshotted with its own index,
  read from inside the
  snapshot
  before the snapshot's `.git` is replaced by the new worktree's pointer.
- In the stage only,
  two real-Git steps run under the new worktree's Git directory:
  `git read-tree --reset -u <new commit>` and
  `git clean --force -d --quiet`.
  They go through the lifecycle's runner,
  which starts real Git directly,
  so the
  wrapper's
  guards of a person's `reset` and `clean` never see them.
  The restore refuses to start when the stage is the source's
  directory
  or the Git directory is the source's.
- The stage is proven before the swap:
  the index equals the new commit's tree,
  Git lists no untracked file,
  and no
  tracked file
  differs from its entry.
  A failed step or proof destroys the stage and Git's own checkout makes the worktree,
  with
  the sentence
  that names the reason.
- Unless `--quiet` is given,
  a line after the snapshot note says the uncommitted changes were left out.
  The journal
  gets an
  ordinary-level entry with the destination,
  the number of paths and the first path.
- The main checkout is read to be snapshotted and listed,
  and is never written.

### What stays a refusal

An operation in progress or a held `index.lock`;
an unmerged,
intent-to-add,
skip-worktree or assume-unchanged entry,
or a
submodule,
anywhere in the index (the scan covers the whole index,
because an ordinary change can come first and hide
one);
a changed or untracked `.gitattributes` file;
a sparse checkout;
a linked worktree as the source,
because its index
lies outside
its directory and is not in the snapshot;
and the clone rung,
which is for clean sources only.

### How it was verified

- A binary matrix compares the new worktree with one real Git makes from the same commit:
  an unstaged edit,
  a file
  staged and
  then edited again,
  a staged deletion,
  an unstaged deletion,
  a staged rename,
  a mode change,
  an untracked file,
  an
  untracked
  directory holding an ignored file,
  an empty untracked directory,
  and an ignored `node_modules` that must be carried.
  It also
  checks the target commit older than the source's,
  `--quiet`,
  the journal entry,
  and that every file of the main
  checkout and
  its index are byte for byte what they were.
- The refusals that stay are each driven in the same fixture:
  a merge,
  a held lock,
  skip-worktree,
  assume-unchanged,
  and a
  changed `.gitattributes`,
  each with a positive control that a snapshot serves the same fixture once the mark is gone.
- Each guard was shown to fail once by removing it:
  the clean step (the proof then ended the rung with the reason
  line),
  the
  reset step (same),
  clean and proof together (the fixture's status listed the untracked files),
  the scan for entries
  no reset
  can undo,
  the `.gitattributes` check (the proof's last step still caught the changed line endings,
  and the control
  failed on
  the missing sentence),
  the restorable kinds,
  and the guard against running in the source.

### Measured on the real repository

The real main checkout is on another filesystem than the scratch area,
so each run used a copy of it made by
`btrfs subvolume snapshot` inside the same subvolume area,
with the dirt created in the copy only;
the main checkout was
never written.
The binary was the debug build of this branch.
The host load average was between 24 and 29 on 28 threads.

- Snapshot with restore,
  first run,
  started at 2026-11-03T00:36:24-05:00:
  the restore was proved 61.3 s later.
- Snapshot with restore,
  second run with timing lines,
  started at 2026-11-03T01:04:35-05:00:
  the source was decided
  dirty after
  10.7 s and the choice of step took 6259 ms;
  the restore proved at 43.5 s.
  Inside the restore the reset took 1103 ms,
  the clean
  56 ms and the proof 57 ms,
  1216 ms together.
  That copy listed 5 changed paths (4 made in the copy,
  1 the real
  checkout had).
- Slow route with the same kind of dirt plus one skip-worktree entry,
  started at 2026-11-03T00:57:51-05:00:
  142.3 s to
  return,
  of which Git's checkout took 32.1 s and the ignored-state copy of 190365 entries 97.6 s.
- A first attempt at the slow route failed with `smudge filter lfs failed` because `git-lfs` was not on the scrubbed
  `PATH` of
  my run;
  it was repeated with `git-lfs` on the `PATH`.
  The same condition would fail a restore that must rewrite an
  LFS file.
- The status of the new worktree after the first run:
  no changed path,
  no untracked file,
  the index equal to `HEAD`,
  `node_modules` present,
  the mode and the deleted file restored.

### Findings outside this work

- Before the trash fixes of main,
  each snapshot add of the day printed `could not be handed to the service; emptying it here`,
  and the foreground emptying of the set-aside directories decided when the command returned:
  the first run returned at
  2026-11-03T00:55:50-05:00,
  19 min 25.7 s after it started,
  though the worktree was complete and proven after 61.3 s.
- Re-measured on the build that merged those fixes (copy of the real repository,
  real session runtime,
  started at
  2026-11-03T01:20:22-05:00):
  no fallback line,
  the worktree was served 19.3 s after the start and the command returned at
  2026-11-03T01:20:57-05:00,
  35.1 s after it started.
  The trash was handed to a socket-activated service instance of the new
  worktree (`cli-git@<instance>.socket` and `.service` started at 01:20:41 to 01:20:42 in the journal),
  which empties it
  after the command has returned.
- A copy of the main checkout made while a Git command ran carried a stale `index.lock` and an `index~pid.lock`;
  the
  wrapper
  rightly refused to treat the copy as a source until the lock was removed.
- The real main checkout holds empty untracked directories that Git's checkout never makes;
  the restore removes them
  from
  the new worktree.

### Limits

- Concurrent edits to the main checkout are out of scope by the owner's decision.
- An edit that kept the size and both timestamps of a file is invisible to Git's status and to the proof alike.
- `core.fsmonitor` is left as configured,
  by the owner's decision.
- The racy-entry content settling is not part of this work.
- A linked worktree with uncommitted changes still takes the slow route.

## 2026-11-03T02:30:23-05:00 Conversion and migration without a lock of their own

Owner,
2026-11-03:
"Do another pkg-wide pass to eliminate locks wherever possible now."
and "Didn't I say locks shouldn't be
used whever possible?
Why are we still using that?"
Branch `fix/git-policy-cli-subvolume-tools-without-locks`.

### What changed in the conversion

- The kernel lock on `cli-git-convert.lock` is gone.
  One command at a time owns a conversion through a numbered claim file in
  the worktree's administrative directory (`cli-git-convert.owner.<generation>`,
  `single_owner_claim.rs`):
  published with one
  exclusive `link`,
  the newest generation is the claim,
  it names the owner by process id and birth identity,
  and a released or
  dead owner's claim is replaced by the next generation.
  No command waits.
- Git's `index.lock` is taken only for the proving pass,
  the exchange and the index rewrite.
  Before,
  it was created before the
  first copy and kept until the old tree was removed.
  The first copy and the removal of the old tree now run without it.
- Release 0.5.20 left `cli-git-convert.lock` in each converted worktree's administrative directory.
  This release probes the
  file once without waiting,
  skips the worktree when an older command holds it,
  and removes it otherwise.
  A conversion that
  release left unfinished (same journal format,
  `index.lock` with the journal's token from the start) is finished by running
  the command again.

### What changed in the migration

- The migration had no single-owner guard:
  `Journal::mark` writes a fixed `.tmp` sibling,
  and a second run resumed the first's
  journal with the first's token and passed `acquire`.
  Every mode now takes a claim file beside the journal first.
- The `index.lock` files the migration holds stay,
  with the reason beside the code (Git's protocol,
  one per index,
  `O_EXCL`,
  never waited for,
  from the last delta to the end of the rewrite).
  The catch-up pass runs under them on purpose:
  it
  can copy files whose index entries the rewrite then describes.

### A migration and a conversion refuse each other

Shortening the conversion's `index.lock` hold removed the accidental exclusion of a migration during the conversion's first
copy (the migration's preflight refuses any `index.lock` in `.git/worktrees/*`).
The two commands now read each other's claim
without taking it:
`live_owner_of` in `single_owner_claim.rs`,
used by the conversion's refusals (kind `OtherCommand`) and the
migration's `git_state_refusals`.
Controls:
`a_conversion_is_refused_while_the_main_checkout_is_being_migrated` and
`a_migration_is_refused_while_a_worktree_is_being_converted`,
each shown red before the refusal and after removing it.

### Inventory of locks in this scope

- `worktree_convert_journal.rs` `ConversionLock` (`flock`,
  per worktree,
  no wait):
  removed,
  replaced by the claim.
- `worktree_convert.rs` `acquire(index.lock)` from before the first copy:
  shortened to the proof,
  swap and rewrite.
- `subvolume_lock.rs` (`lock_paths`,
  `acquire`),
  `subvolume_migrate.rs` (`drop_old_locks`,
  `acquire_all`),
  `subvolume_rollback.rs` and `subvolume_index_files.rs` (`IndexLock::Acquire`,
  `Held`):
  Git's `index.lock`,
  kept,
  reasons
  written beside each.
  `drop_old_locks` removes only the migration's own token locks that travelled with the old tree.
- `subvolume_preflight.rs` only reads lock names and takes none.
- `package/mutation-test/cli-cargo-mutants/src/campaign-lock.ts` (`flock`,
  per campaign,
  refuses and never waits) and
  `src/gate-slot.ts` (six host-wide `flock` slots,
  waits only while all are in use):
  kept,
  reasons written beside each;
  porting
  the campaign lock to a claim file is the owner's decision.

### Verification

- Red first at `d1b6d2e783`:
  three of the new controls failed (kernel lock file created,
  `index.lock` held across the first
  copy and the removal,
  no guard on a second migration).
- Each guard was removed once and the matching controls failed:
  owner liveness,
  the withdrawal of a stale claimant,
  the probe of
  an older release,
  the lock given back after a failed proof,
  the lock taken again before the rewrite,
  the lock given back after
  the rewrite,
  the lock taken before the proof,
  and the migration's claim.

## 2026-11-03T02:42:59-05:00 The settlement lock is gone from worktree add

### The owner's words

- Standing:
  "Use as few locks as possible."
  and "You should (almost) never be waiting."
  and "Avoid serializing
  embarrassingly parallel work."
- 2026-11-03:
  "Didn't I say locks shouldn't be used whever possible?
  Why are we still using that?"
- 2026-11-03,
  `AGENTS.md` rule `NLK`:
  "Concurrency is lock-free first (atomic rename,
  `O_EXCL`,
  one owner per item);
  an
  unavoidable lock covers the smallest unit,
  never a whole repository,
  with its reason written beside it."
- Races caused by other processes are out of scope ("That is not an our problem.
  By using this command or this
  program,
  make the user understand and accept the risk.");
  the wrapper's own deterministic bugs are in scope.

### What the lock cost

- One `git worktree add` waited about 228 s for another session's add to release the lock,
  and a second waited over
  20 minutes and was stopped (measured 2026-11-03).
- The `git worktree add` that created the worktree for this work took over 120 s in this session;
  it printed no
  `waiting for PID` line,
  so that wait was not the lock's and is not counted here.
- The lock was taken before real Git and released after the ladder,
  the copy and the trash hand-off,
  so it was held for
  the whole add,
  minutes on a large repository.

### The jobs the lock did, from the code at `b182ee767f`

All paths are under `package/git-policy/cli/src/native`.

- Attributing new worktrees to this invocation.
  `worktree_copy.rs` took the lock at line 369 (`settle_invocation`),
  captured the set of administrative directories before
  real Git (`worktree_copy_registry.rs`,
  `read_admin_ids`,
  line 126) and compared it after (`find_created_worktrees`,
  line
  429).
  Without a lock,
  a concurrent add's worktree falls into the window and is copied into by the wrong command.
  This is the job the lock existed for.
- Recovery before a creating command (`worktree_copy.rs`,
  `settle_under_lock`,
  lines 294 to 295) and the pending check of
  every other command (`worktree_copy_recovery.rs`,
  `recover_pending_worktree_copies`,
  line 298,
  which took the lock with
  `try_acquire_worktree_copy_lock`,
  line 305).
  Recovery installs a journal and reaps stages,
  so it must never run on the journal or the stage claim of a copy that is
  still running;
  the lock was the only proof of that.
- Single winner among recovering commands:
  the lock again.
- The re-read of the repository identity after the lock was acquired (`settle_under_lock`,
  lines 285 to 292,
  and
  `worktree_copy_forward.rs`,
  lines 312 to 319),
  which only mattered because the wait could be long.
- The inherited lease (`worktree_copy_lock.rs`,
  `validate_inherited_lease`,
  line 755,
  and the environment variable
  `CLI_GIT_WORKTREE_COPY_LEASE` set at `worktree_copy_forward.rs`,
  line 283):
  a nested wrapper started by a filter or a hook
  inside the locked add skipped the lock it was inside,
  and the outer add copied into the nested worktree.
- The sweep of every temporary file in the journal root (`worktree_copy_stage_claim.rs`,
  `remove_unfinished_claims`,
  line
  353),
  safe only because no writer was alive.
- `git worktree move` was a locking command although it registers nothing new (`forwarded_command.rs`,
  `creates_or_moves_worktrees`).
- The ladder,
  the dirty restore,
  the trash hand-off and the snapshot (`worktree_ladder_run.rs`,
  `worktree_ladder_finish.rs`,
  `worktree_dirty_restore.rs`,
  `worktree_trash_dispose.rs`) ran inside the lock but needed none of it:
  every name they use
  (the stage,
  the claim records,
  the destination,
  the converting name,
  the new worktree's Git directory) belongs to one add,
  and each step is a rename,
  an exclusive creation or a Git run on the add's own paths.

### The replacement for each job

- Attribution:
  the destination comes from the command line (`worktree_add_plan.rs`,
  `parse_copy_destination`),
  and the
  registration is found by `bind_registration` (`worktree_add_binding.rs`),
  which follows the pointer in the destination's
  `.git` file to the administrative directory and checks that the directory's `gitdir` file names the destination back.
  Whether the destination was registered before Git ran is recorded first,
  so a destination Git refused is never copied
  into.
  No other worktree's registration is read,
  and `git worktree list` is asked only for the roots to exclude from the copy.
- Ownership:
  a journal is `<id>.<owner>.json`,
  a stage claim carries `owner=<owner>` on its second line,
  and a temporary
  file is `<item>.<owner>.<unique>.tmp` (`worktree_copy_owner.rs`).
  An owner is the process id,
  the birth identity and the PID namespace;
  it is alive by the same proof the lock used.
- Recovery and the pending check skip every item whose owner is alive and recover those whose owner is dead,
  never waiting.
- Single winner:
  taking over a dead owner's journal is one `rename` to a name that carries the new owner
  (`worktree_copy_journal.rs`,
  `adopt_journal`).
  The loser finds the old name gone.
  A winner that dies leaves a name with a dead owner,
  so no lock file exists to go stale.
- Temporary files:
  the sweep removes only a dead writer's,
  so a live writer's rename can never lose its file.
- The identity re-read and the lease are removed.
  A hook or filter that runs the wrapper is an add of its own and copies its own source into its own worktree.
- `git worktree move` is forwarded like any other command.
- The ladder and the rest are unchanged;
  they were never protected by the lock.

### Forms whose destination the wrapper cannot know

- Aliases that expand to `worktree add` are read after expansion,
  so they have a destination.
- Every option `git worktree add -h` lists is read,
  with short options bundled and `-b` or `-B` attached or detached,
  after `-C`,
  `-c`,
  `--config-env` and Git's neutral global flags.
- An abbreviated long option,
  `--git-dir`,
  `--work-tree` and an option between `worktree` and `add` leave the destination
  unknown.
  Real Git runs as it would without the wrapper,
  no copy is made,
  and when Git succeeded one line on standard error says
  so.
- The alternative considered was to keep comparing the set of registrations for these forms.
  It is wrong whenever another add runs at the same time (the other add's worktree is copied into by the wrong command,
  or
  twice),
  and making it right needs the lock back,
  so it was rejected.
- A worktree created by a hook through an absolute path to Git was never copied into by the wrapper for want of an
  invocation;
  before this change the outer add's comparison happened to cover it,
  and now it does not.
  This is a behavior change the owner may want to veto.

### Controls

- `binary_worktree_copy_concurrent_tests.rs`,
  committed failing first (commit `aebdc9acf5`):
  two adds of different
  worktrees,
  one held inside real Git by a smudge filter until a release file appears;
  a move,
  a removal and a conversion
  check beside the held add;
  an add beside a conversion's kernel lock and `index.lock`;
  and an add after a killed add.
  Two of the six failed on purpose with the message `waiting for PID`,
  the add and the move.
- After the change all six pass,
  and the second add finishes while the first is still inside real Git,
  with a summary of
  its own worktree only.

### Guards shown red

Each guard was removed once,
the named tests failed,
and the guard was restored.

- Skip of a live owner's journal:
  `pending_check_leaves_a_live_journal_and_recovers_a_dead_one` and
  `a_creation_leaves_a_live_journal_alone_and_does_not_wait`.
- Skip of a live owner's claim in the reaper:
  `a_live_owners_claim_is_left_whole` and
  `the_pending_check_reaps_abandoned_stages_only_without_a_live_owner`.
- The rename of the take-over:
  `only_one_command_takes_over_a_dead_journal` and eight recovery tests that need a winner.
- The dead-writer test of the temporary sweep:
  `an_unfinished_write_of_a_dead_writer_is_removed_and_a_live_writers_is_kept`.
- The record of whether the destination was registered before Git:
  `a_destination_registered_before_is_not_created` and
  `a_destination_registered_before_is_not_copied_into_again`.
- Copying into every other registered worktree instead of the named destination:
  `concurrent_registrations_are_told_apart_by_destination` and `a_creation_copies_only_into_the_destination_it_named`.
- The tolerance of a journal or claim that vanished between listing and reading:
  `a_vanished_journal_is_skipped_and_a_corrupt_one_is_not` and
  `a_claim_that_vanished_is_skipped_and_a_malformed_one_is_refused`.
- The notice for an unread destination:
  `a_creation_with_an_unread_destination_copies_nothing_and_says_so`.
- The reading of a short option's value:
  three tests of `worktree_add_plan.rs`.
- The exclusion of `move` from the creating commands:
  `only_worktree_add_creates_worktrees`.
- The birth identity comparison:
  `the_running_process_is_alive_and_a_killed_one_is_dead`.
- A first attempt to guard the reading of `-b` and `-B` through a list entry found the entry redundant,
  because the
  short-option branch already reads them;
  the entry was removed.

### Locks left on this path

None are taken by the add,
the copy,
the ladder,
the trash hand-off or the dirty restore.
What remains is Git's own:
`index.lock`,
reference and configuration locks that real Git takes while it runs.
The trash hand-off wakes the new worktree's own service,
whose provisioning belongs to the service lock family.

### For the owner

- Hook-created worktrees through the wrapper now copy from their own source;
  through an absolute path to Git they get no
  copy (see "Forms whose destination the wrapper cannot know").
- The unread-destination forms get no copy and one line.
  The alternative that keeps the copy needs the lock.
- Moving the copy into the new worktree's service would remove the remaining "owner dies,
  next command recovers" step
  too,
  because the service is one instance per worktree.
  It was not done here:
  the summary line is printed by the add,
  so the add would have to wait for the service anyway.

### 2026-11-03T02:49:45-05:00: relative paths in the registration, found by review

- Git writes the registration's `gitdir` file relative to the registration under `--relative-paths` and under the
  configuration `worktree.useRelativePaths`,
  which a user can set globally.
  The binding compared that file byte for byte with the absolute path of the destination's `.git`,
  so the check found no
  registration,
  the add succeeded,
  and no copy was made and nothing was said.
  The earlier comparison of administrative directories resolved such pointers,
  so this was a regression of the first commit
  of this work,
  and the ladder's binding had the same blind spot for a configured user.
- Controls,
  red first:
  `a_registration_written_with_relative_paths_is_bound_and_verified`,
  `a_destination_registered_with_relative_paths_is_found` and the binary control
  `a_worktree_registered_with_relative_paths_is_copied_into`.
- The fix resolves the recorded path against the registration's directory before comparing (`worktree_add_binding.rs`,
  `recorded_destination`),
  in the binding and in its verification,
  which unblocks the ladder for those worktrees as well.
- A destination Git made whose pair still cannot be confirmed now gets one line instead of silence:
  `a_made_destination_whose_registration_cannot_be_confirmed_is_told`.
- The race control `racing_commands_take_a_dead_journal_exactly_once` has the same guard as
  `only_one_command_takes_over_a_dead_journal`:
  the rename of the take-over.
- Not changed here,
  for the coordinator:
  `doc/decision/cli-git-concurrent-commits.md`,
  `doc/decision/cli-git-policies-platform.md`
  and `doc/planning/cli-git-btrfs-worktree.md` still describe the settlement lock as the current design.

### 2026-11-03T03:10:41-05:00: the owner's decision to read more forms of the add

- Owner decision of 2026-11-03,
  on the side effects of the lock-free add:
  "Accept,
  read more forms (Recommended)".
  The design
  stays lock-free,
  and the wrapper also reads abbreviated long options (unique prefix,
  as Git's parse-options does) and the
  global `--git-dir` and `--work-tree`,
  so fewer adds skip the copy.
- The rules were taken from Git 2.56.0 (clone at `/var/home/user/temp/agent/git-2026-11-03`),
  not recalled:
  - `parse-options.c`,
    `parse_long_opt` (lines 520 to 593):
    a leading `no-` negates;
    a second one is accepted by no option
    here;
    an exact name returns at once (line 560 to 563),
    so it wins over prefixes seen earlier;
    otherwise each option whose
    name starts with the text is registered as an abbreviation (lines 566 to 567),
    a second registration makes it ambiguous
    (`register_abbrev`,
    lines 497 to 518,
    with `is_alias` false because `git worktree add` declares no alias group),
    and an
    ambiguous text is the error at lines 578 to 587.
  - `parse-options.c`,
    `do_get_value` (line 136):
    a value on a negated form or on a flag (`PARSE_OPT_NOARG`) is an error.
    Measured with Git 2.56.0:
    `--track=direct` and `--no-track=x` are errors,
    so `--track` takes no value;
    the earlier
    reading that accepted `--track=direct` was wrong and is corrected.
  - `parse-options.c`,
    lines 1113 to 1121:
    `--` and `--end-of-options` end the options.
  - `builtin/worktree.c`,
    `cmd_worktree` (parse of the subcommand):
    no option is accepted between `worktree` and `add`
    (measured:
    `git worktree --verbose add` is "unknown option"),
    so that form is Git's error.
  - `git.c`,
    `handle_options`:
    the global options and which take a value;
    `setup.c`,
    lines 1222 to 1231:
    with `--work-tree`
    the working directory decides,
    so a relative destination lies against it in both the inside and the outside case.
- One table (`ADD_OPTIONS` in `worktree_add_plan.rs`) serves every reading of the options of the add,
  and the control
  `the_option_table_matches_what_git_prints_for_dash_h` compares it with the output of `git worktree add -h`,
  so it cannot
  drift from Git.
- Found when the main merge brought the package-manager repair tests:
  a destination below directories that do not exist yet
  (`.claude/worktrees/nested`) was unreadable,
  because the reading canonicalized the parent.
  Git creates the parents;
  the
  destination is now the real path of the nearest existing ancestor with the rest as written.
- Controls,
  red first (commit `binary_worktree_copy_forms_tests.rs`):
  abbreviations,
  a negated abbreviation,
  `--git-dir` with
  `--work-tree`,
  and missing parents failed;
  the ambiguous prefix and the forms Git rejects kept Git's error and no copy.

## Review requests

- The stamp argument under "What FICLONE guarantees and how a concurrent writer is detected",
  and the 20 ms settled
  rule.
- The unreadable-entry policy (skip and say so,
  for every source).
- Whether `CopySelection::Paths` and `read_named_ignored_roots` should now be deleted:
  nothing in the release path
  builds
  them any more;
  they stay because their tests and the binary tests use them.
- Whether the linked-worktree copy should adopt the cache rule as well;
  the owner's words named only the main checkout.

## Package-manager state of a new worktree, written 2026-11-03T00:57:25-05:00

Scope:
crate `monochromatic-git-policy-cli`,
Linux,
btrfs.
Branch:
`feat/git-policy-cli-package-manager-aware-worktree-copy`.
Fixtures were throwaway btrfs subvolumes under `${HOME}/temp/agent/pm-survey/` (scripts and logs kept there) and
throwaway worktrees of this repository;
all were removed.
Every command ran in its own `systemd-run --user --scope` with `CPUQuota`,
`MemoryMax`,
`TasksMax` and `IOWeight` of
at most 50.
Another process changing files while a worktree is copied or repaired is the person's risk,
by the owner's decision,
and
nothing here defends against it.

### Owner decisions that bound it

- "Our wrapper must work with pnpm w/o a storeDir set,
  enableGlobalVirtualStore on or off.
  Try not to degrade the end user
  experience in any situation."
- "Compatibility with all pnpm configurations is required."
- "Must be compatible with all configurations of all pkg managers we currently use.
  If at any point you feel we're doing
  something in the wrong way,
  ask me."
- "The wrapper may launch pnpm itself for worktree copying",
  widened to "all pkg managers".
- Cargo:
  "Reuse deps,
  rebuild own",
  done by "Run cargo clean -p".
- "But we're not using Mise as a pkg manager."
  and "Mise isn't even a tool manager.",
  so mise is not surveyed.
- The repository's own toolchain wrapper `mpm-shadow` (`package/mpm-shadow/cli`) is a package manager of the repository.
- "That is not an our problem.
  By using this command or this program,
  make the user understand and accept the risk."
  (races with other processes are out of scope).

### What each manager keeps in a worktree

Each result is a verbatim copy (links kept as they are,
reflinked) into a new subvolume,
nested under the source and outside it
at another depth,
then the tool's own command.

- pnpm 12.16.1:
  `node_modules` per package and the root,
  `.pnpm` inside it or,
  with `virtualStoreType: global`,
  links into
  `<storeDir>/v11/links` reached by a relative path that climbs to `/`.
  The coordinator's matrix found every configuration fine except the global virtual store with the store outside the tree.
  On a copy of this repository's worktree one level deeper the entry `node_modules/execa` was dead and
  `require.resolve` still succeeded,
  through the source's `node_modules`,
  so the failure is silent.
  After the repair the entry resolves inside the copy.
  Matrix detail (real pnpm 12.16.1,
  a workspace root plus one package,
  copies nested under `.claude/worktrees/topic` and
  outside the tree at another depth,
  measured 2026-11-03):
  the relative links of a global virtual store with the store
  outside the tree climb to `/` and back down,
  so a verbatim copy two levels deeper resolves to nothing,
  and the first
  probe misled because Node fell back to the source's `node_modules` above a nested copy (check each tree's own entry and
  where resolution lands).
  An outside copy also fails `pnpm run` with `ERR_PNPM_UNEXPECTED_VIRTUAL_STORE` (`.modules.yaml` records `virtualStoreDir`
  as a relative path) until `pnpm install --offline --frozen-lockfile`,
  which repairs it in about 50 ms.
  Fine in every case (own entries resolve inside the copy,
  `pnpm run` and the offline frozen install succeed in under
  0.2 s):
  store set or unset,
  isolated virtual store,
  linker isolated,
  hoisted or pnp,
  and the global virtual store with
  no `storeDir` (pnpm then keeps it in `node_modules/.pnpm-store/v11/links`,
  inside the tree;
  the copy's `.modules.yaml`
  still names the source's in-tree store,
  and whether a later install in the copy writes into the source's store is
  unmeasured).
- Cargo 1.101.0-nightly:
  `target/` and,
  where `build-dir` is set,
  a build directory outside the tree.
  The person's `~/.cargo/config.toml` keys it by `{workspace-path-hash}`,
  so a new worktree has none.
  Every one of the tracked `Cargo.toml` files of this repository belongs to a workspace of its own with a `Cargo.lock`.
- mpm-shadow:
  `.cache/toolchain-bin` (links),
  `.cache/toolchain-launchers` (hard-linked launchers and `.launch` records),
  `.cache/cli-git-bin`,
  `.cache/mpm-shadow-bin`,
  `.cache/pnpm-shadow-bin`,
  `.cache/search-shadow-bin`,
  `.cache/toolchain.env.json` and `.cache/toolchain.guard`.
  Its links to tools are absolute and outside the tree (`~/.local/share/monochromatic-dev/toolchain`,
  `~/.cargo/bin`) and
  resolve from any copy.
  The links and records of repository-published tools name the source:
  `.cache/toolchain-bin/cli-git-native` is an absolute
  link into the source's launchers,
  and seven `.launch` records name the source's `toolchain.lock.json`,
  so a branch that
  changes the lock would run the source's tool versions and guard.
  The other `.cache` entries are plain caches or inputs:
  `forbidden-strings*.txt` (generated rule files read by tasks),
  `rust-catalog-update` (crates.io metadata),
  `typescript` (a `tsbuildinfo`) and `test-scratch`.
- uv 0.13.6:
  a project `.venv` (`pyvenv.cfg` names only the interpreter,
  which lies outside the tree) whose console scripts hold
  the venv interpreter's absolute path as their shebang.
  No `pyproject.toml` or `uv.lock` is tracked in this repository and no `.venv` is ignored state,
  so uv has no state here.
  Fixture:
  the copy runs,
  but its scripts start the source's interpreter (so `sys.prefix` is the source's venv,
  and the copy
  breaks if the source is removed).
  `uv sync --offline --frozen` changes nothing (20 ms);
  `uv sync --offline --frozen --reinstall` rewrites the shebangs (60 ms).
- Bun 1.3.14:
  `node_modules/.bun` with relative links;
  the copy resolves its own entries,
  and `bun install --frozen-lockfile`
  reports no change.
  `bunfig.toml` only disables auto-install;
  no `bun.lock` is tracked.
- npm 11.20.0:
  relative workspace links;
  the copy resolves its own entries;
  `npm ci --offline` took 1.0 s and 4.0 s.
  No `package-lock.json` is tracked.
- Gradle (the committed `gradlew`,
  9.8.1;
  two projects):
  `.gradle`,
  `.kotlin` and `build`.
  Fixture `package/linter/kotlin`:
  in a copy with all three,
  and in one without `.gradle` and `.kotlin`,
  the first build
  re-ran the compile tasks (Gradle keys its history by absolute path) and a second build was up to date,
  so no stale result is
  reused.
  `.gradle` and `.kotlin` are already left out of a main-checkout copy.
- Zig 0.15.2:
  `.zig-cache`,
  `zig-out`,
  and the global cache,
  which this repository points at `target/zig-global-cache` (left
  out already).
  No `build.zig` is tracked.
  Fixture:
  a copied `.zig-cache` made `zig build` report up to date although every file time was fresh,
  and the binary kept
  the source's path (debug information and `@src`);
  with `.zig-cache` removed the rebuild took 1.5 s and the binary names the
  copy.
  So `.zig-cache` is now left out of a copy as a cache.
- Node:
  no state of its own.
- mise:
  dropped by the owner's decision.

### The Cargo hazard and the options

Evidence (this branch's probe `cargo-clean.ts`,
four combinations of layout and place):
a cloned build directory with nothing cleaned gave zero recompiles and `env!("CARGO_MANIFEST_DIR")` and build script output kept
the source's paths (the coordinator's measurement,
repeated by the guard that skips the clean:
the binary control reports no
crate rebuilt).
`cargo clean --offline --package app --package lib` in the copy took 20 to 30 ms,
and the next build compiled the build script,
`lib` and `app` only,
with the registry crate reused,
the paths of the copy,
a second build compiling nothing,
an edit in the
copy rebuilding `lib` and `app`,
and the source's build still fresh with its own paths.
This holds with an external `build-dir` (cloned first) and with the default layout (`target/` already copied),
nested and
outside,
so `cargo clean -p` does what is needed and nothing else had to be chosen.
`cargo metadata --locked --offline --no-deps` wrote no `Cargo.lock` in a directory that had none.
Options the owner did not choose,
for the record:
fresh file times on the worktree's own files (rejected by the owner in favour
of the clean),
and leaving external build state out (every new worktree then builds everything,
including registry crates).

### The symlink rule

The question was whether a copied symlink whose target lies outside the source tree must be rewritten to keep resolving to the
same place,
once pnpm runs.
A census of the main checkout (3205 symbolic links,
nested worktrees and `.git` excluded) found:

- Relative links that stay inside the tree:
  all of the pnpm `node_modules` links today (the isolated layout),
  which any copy keeps
  valid.
- Absolute links outside the tree:
  the `.cache/toolchain-bin` links into `~/.local/share` and `~/.cargo/bin`,
  which resolve
  from any copy.
- Absolute links into the tree:
  five (the launchers in `.cache` and one orphan `node_modules/@monochromatic-dev/pnpm-override-stub-silent`
  from a dependency that is no longer declared).
  pnpm leaves the orphan alone,
  so it keeps pointing at the main checkout in a copy;
  the launchers are rebuilt by mpm-shadow.
- Relative links that leave the tree:
  none today.
  They appear when main's `node_modules` is reinstalled with `virtualStoreType: global`,
  and then they are pnpm's own.

So once pnpm runs the rule has no consumer left among the tools of this repository,
and it was not implemented.
What it would still protect is a worktree whose pnpm repair could not run (no pnpm,
no lockfile match):
its links would stay dead
instead of resolving to the shared store,
and in a nested worktree they would silently resolve through the source's
`node_modules`.
That residual case is reported to the owner as a doubt,
not decided here.

### What was built

`src/native/package_repair*.rs`,
called from `settle_under_lock` in `src/native/worktree_copy.rs` for every worktree the
command created,
a snapshot-served one included;
see `SPEC.md`,
section "Package-manager repair of a new worktree".

- `package_repair_command.rs`:
  find a program on the search path;
  run it in a directory with its own process group,
  a time
  limit,
  captured output and a verdict (succeeded,
  failed with a reason,
  timed out).
- `package_repair_pnpm.rs`:
  `pnpm install --offline --frozen-lockfile`.
- `package_repair_cargo.rs` and `package_repair_tree_clone.rs`:
  metadata,
  the plan (clone and clean,
  clean only,
  or nothing),
  the clone (per-file `FICLONE`,
  modes,
  times and links kept,
  parallel) and `cargo clean --locked --offline --package <member>`.
- `package_repair_toolchain.rs`:
  stale launcher records,
  then `mpm-shadow install --frozen-lockfile` (online when the lock names tools
  that are not installed;
  see the owner answers).
- `package_repair_report.rs`:
  one success line (not under `--quiet`) and one line per failure (always).
- `worktree_copy_prune.rs`:
  `.zig-cache` joins the cache names.

pnpm is started with the `pnpm` the person's `PATH` resolves,
which here is the pnpm-shadow.
Started from the wrapper it scans the workspace's manifests and writes its advice to standard error,
then runs the real pnpm;
the
wrapper captures that text and shows it only inside a failure line.

### Measurements

Load:
the one-minute load average was between 27 and 39 on 28 threads for all of these;
no spread was measured on unchanged input
(a quiet host is never available),
so a difference of a few seconds between two runs is not a claim.
The wrapper was this branch's debug build (`target/debug/cli-git-native`);
the repairs are dominated by the tools they start.

The real add:
`git worktree add` from this branch's worktree (a linked worktree on a subvolume,
140251 ignored entries) into a
worktree one directory deeper,
with `MONOCHROMATIC_VERBOSE=true`,
logged in `pm-add-1.log` under `${HOME}/temp/agent/`.

- Whole command:
  354.97 s,
  of which about 228 s was waiting for another session's add to release the settlement lock
  (the lock holder's own slow add from `main`).
  Work after the lock:
  about 127 s.
- Rung:
  Git's own checkout followed by the ignored-file copy,
  not a snapshot,
  because the source worktree held an uncommitted
  edit of `README.md` (the ladder reported "README.md differs from its index entry").
  Git's checkout took about 29 s and the copy of 140251 entries 69.6 s.
  The rung is independent of the repair.
- The repair:
  28.2 s wall for all tools side by side (it began when the copy settled and ended when the findings were
  printed).
  pnpm took 27.5 s,
  which set the wall time.
  Cargo handled 34 workspaces in parallel on 8 threads:
  32 had a plan line (23 clean-only because `target/` had been copied,
  one
  clone-then-clean,
  the rest nothing to do),
  one build directory of 7117 entries was cloned in under a second.
  The toolchain repair took 5.1 s for `mpm-shadow list` and 0.8 s for `install --frozen-lockfile`.
- That run used the locked clean,
  which failed in ten workspaces (committed locks that disagree with their manifests) and printed
  ten long failure lines;
  the unlocked clean with the lock put back replaced it (see the doubts and the commit that
  introduced `restore_lock`).
  A second add with the fixed binary waited more than 20 minutes behind another session's add for the settlement lock and was
  stopped before it did any work,
  so the fixed repair has no whole-repository timing;
  the stale-lock case is covered by a binary
  control and was checked by hand in one workspace (`cargo clean --offline --package ide` rewrote its committed lock,
  bumping
  `tokio` from 1.53.2 to 1.54.1).

pnpm on its own:

- Converting a tree from the isolated layout to the global virtual store (what a copy of today's main needs):
  57 s,
  42 s of it
  system time.
- An already converted worktree:
  3.4 s when nothing changed,
  0.4 s to 0.7 s on later runs.
- A copy of that worktree one level deeper (links dead before):
  1.9 s with scripts allowed,
  8.5 s with `--ignore-scripts`
  (one run each,
  no spread;
  the difference is not a claim).
  After either,
  `node_modules/execa` resolved inside the copy.
- The pnpm-shadow:
  the first run in a tree took 20.5 s,
  then 0.70 s and 0.63 s through the shadow against 0.58 s,
  0.53 s and
  0.65 s directly through `.cache/toolchain-bin/pnpm`.
  So the shadow works from the wrapper (it scans the manifests,
  prints its advice to standard error and runs the real pnpm) and
  can add tens of seconds the first time;
  the later runs show no difference beyond noise.
  The cold 20.5 s was the run right after a run with other switches,
  so it may also be pnpm's own re-verification;
  the two causes
  were not separated.

Cargo on fixtures (`cargo-clean.ts`;
two crates,
one with a build script,
one registry crate):

- `cargo clean --offline --package app --package lib`:
  20 to 30 ms.
- The build after it:
  about 0.2 s to 0.4 s,
  compiling the build script,
  `lib` and `app`;
  the registry crate was reused;
  a
  second build compiled nothing;
  an edit rebuilt `lib` and `app` in about 0.1 s;
  the source stayed fresh with its own paths.
- Scale of a clone from the real machine (the full clone from main of all 34 workspaces was not run):
  the build directory of
  `package/git-policy/cli` in main holds 63964 entries and 35 GB apparent size;
  `cp --archive --reflink=always` of it
  took 6.6 s (a serial proxy for the repair's parallel clone).
  The machine keeps 1155 such directories under `/var/mnt/encrypted/cargo-build`.

mpm-shadow in a new worktree:
`list` 1.9 s then 1.3 s,
`install --frozen-lockfile` 1.2 s both times ("0 installed,
65
unchanged");
634 entries of main's `.cache`,
`~/.local/bin` and the toolchain root were identical before and after.

Other managers (fixtures):
`uv sync --offline --frozen --reinstall` 62 ms;
`npm ci --offline` 1.0 s and 4.0 s;
`bun install --frozen-lockfile` 10 ms and 20 ms;
Gradle `jar` in a copy 1.5 s and 1.1 s,
then 1.2 s and 0.5 s up to date;
`zig build` after removing `.zig-cache` 1.5 s.


### Doubts for the owner

These are questions for the owner,
not decisions.
Each is phrased to be answered on its own.

- Answered,
  see the section on the owner answers:
  main's reinstall,
  `--ignore-scripts`.
- Owner instruction applied (recorded as adopted,
  veto open):
  "Use as few locks as possible."
  and "You should (almost) never be
  waiting."
  The repairs now run after the settlement lock is released,
  still side by side,
  and a binary control shows that a second add
  completes while the first add's pnpm is still running.
- uv:
  a copied project `.venv` keeps working but its console scripts start the source's interpreter,
  and
  `uv sync --offline --frozen` does not change that;
  only `--reinstall` does (60 ms on a fixture).
  No uv project exists in this repository,
  so nothing was implemented;
  the trigger to implement it is the first tracked `uv.lock`
  (or `pyproject.toml` that names uv) together with a `.venv` among the ignored files that a copy carries.
  Options:
  (a) implement `uv sync --offline --frozen --reinstall` now,
  untested against any real project;
  (b) implement it when
  the first project appears;
  (c) leave `.venv` out of a copy.
  Ranking:
  b > a > c,
  because b buys a real test,
  a adds code with no consumer,
  and c makes every new worktree reinstall
  instead of cloning.
- The pnpm hook substitutes blocked dependencies with workspace stubs;
  the stub link and the ten stale Cargo locks are
  measured in the section on those measurements,
  and still open.
- A snapshot-served or copied worktree whose lock for a Cargo workspace disagrees with its manifest (ten here) is cleaned
  with the lock put back byte for byte,
  which also means the clean's own resolution result is thrown away.
  The disagreement is a defect of the committed locks,
  not of the wrapper;
  should those locks be regenerated?
  Putting a lock back also gives it a new change time,
  so the ladder (which wants every tracked file to match its index entry
  and be older than the index file) refuses a snapshot from that worktree until a `git status` refreshes the index.
  Options:
  (a) leave it,
  (b) run `git update-index --refresh` in the new worktree after a lock was put back,
  (c) regenerate the
  committed locks so nothing is rewritten.
  Ranking:
  c > b > a,
  because c removes the cause for every tool,
  b hides it at the cost of one more Git run per add,
  and a leaves
  the next snapshot from that worktree falling to the slow route.
- The full clone of main's Cargo build state for every workspace was not run (that would read main);
  the scale measured on one
  workspace is in the measurements.
- `native:clippy:windows` was not run (the Windows target is not installed here);
  the new modules are Linux only and gated
  `cfg(target_os = "linux")`.
- Timing figures come from a host whose load average was in the thirties;
  no spread was measured on unchanged input,
  so
  differences of a few seconds between runs are not claims.

### Owner answers received during the work, recorded 2026-11-03T01:39:52-05:00

Each is the owner's choice by the question tool or verbatim.

- Main's `node_modules`:
  "Run it now (Recommended)".
  The coordinator runs `pnpm install --frozen-lockfile` in main;
  the wrapper
  never does.
- Online toolchain install during an add:
  "Install online during the add",
  against the offered ranking.
  So the toolchain repair
  runs `mpm-shadow install --frozen-lockfile` with the network instead of printing `mise run toolchain:install` when the
  worktree's tools differ from its lock.
  The owner was told it can download during an add and can retire tool copies another checkout still links;
  both are accepted
  consequences.
  Kept:
  the line that says what was installed,
  the time limit,
  and the rule that a failed install never fails the add.
- `--ignore-scripts`:
  "Run scripts as allowBuilds says (Recommended)":
  the repair keeps the words offline and frozen only.
- Symlink fallback rule:
  "No fallback rule (Recommended)":
  nothing is built.
- Which pnpm:
  "The repair should start whatever pnpm is first on path.":
  the repair keeps resolving `pnpm` through `PATH`
  (the pnpm-shadow here),
  and the comparison of the shadow with the real binary was dropped.
- Failure lines under `--quiet`:
  they stay visible,
  as Git's own `--quiet` still shows errors (adopted from the owner's
  standing instructions,
  veto open).
- Locks:
  "Use as few locks as possible."
  and "You should (almost) never be waiting.":
  the repairs run after the settlement lock
  is released (adopted,
  veto open).

### The stub link and the stale Cargo locks, measured 2026-11-03T01:46:51-05:00

The stub link:

- The link is `package/rolldown-plugin/import-attributes/node_modules/@monochromatic-dev/pnpm-override-stub-silent`,
  an absolute
  link to `/var/home/user/Monochromatic/package/pnpm-override/stub-silent`,
  dated 2026-11-01T15:01:28-05:00.
  The package it names,
  `@monochromatic-dev/pnpm-override-stub-silent` at `package/pnpm-override/stub-silent`,
  is the target of the
  `silent` action of `.pnpmfile.mjs` (line 92 holds its `workspace:` specifier);
  CODEOWNERS names one owner for everything.
- No code in the repository writes that link:
  a search of the tracked and ignored sources (without `node_modules` and `target`)
  finds the package name only in documentation,
  in `package/config/pnpr/config.yaml` and in the stub's own manifest.
  The importer's manifest does not declare it and `pnpm-lock.yaml` does not link it into that importer.
  pnpm 12.16.1 never wrote an absolute link in the three ways probed in a throwaway workspace:
  a `workspace:` alias to a
  package of another name,
  a `link:` specifier with an absolute path,
  and `pnpm link <absolute path>` all produced relative links.
  The link's time lies inside the stub-silent review lane that the handover closed in commit 33691b3eb9
  (2026-11-01T15:07:40-05:00),
  so it was most likely made by hand or by a script in that lane,
  not by pnpm.
- Proof with a copy:
  in a worktree that is a copy of main,
  `require.resolve('@monochromatic-dev/pnpm-override-stub-silent')`
  from `package/rolldown-plugin/import-attributes` resolves to the main checkout's `package/pnpm-override/stub-silent/index.cjs`,
  not to the worktree's own file.
  So today a worktree on a branch that changes the stub would still load main's stub through that link;
  nothing else loads the
  stub (every consumer that declares it gets a relative link inside its own tree).
- A relative link (`../../../../pnpm-override/stub-silent` from that directory) would resolve inside each worktree and change
  nothing else;
  deleting the link would change nothing either,
  because no manifest asks for it.
  pnpm keeps leaving it alone in a copy.

The ten workspaces whose committed `Cargo.lock` disagrees with their manifest,
found by `cargo metadata --locked --offline` in a
throwaway export of the branch's `HEAD` (`lock-regen.ts`,
kept under `${HOME}/temp/agent/pm-survey/`;
nothing ran in a checkout).
For each,
the packages cargo would move when it resolves offline (it also re-locks the packages named here and nothing else):

- `package/forbidden-strings/cli.fuzz`:
  22 packages,
  among them `anyhow`,
  `arbitrary` 1.4.2 to 1.5.0,
  `clap` 4.6.1 to 4.6.7,
  `forbidden-strings` 0.5.0 to 0.6.1,
  `ignore`,
  `memchr`,
  `serde`,
  and new `digest`,
  `sha2`,
  `syn` 3.0.9.
- `package/forbidden-strings/core.bench`:
  `forbidden-strings` 0.6.0 to 0.6.1 and `memchr` 2.8.2 to 2.8.3.
- `package/forbidden-strings/core.fuzz`:
  `arbitrary` and `derive_arbitrary` 1.4.2 to 1.5.0,
  `memchr` 2.8.2 to 2.8.3.
- `package/ide/app`:
  `globset` 0.4.20 to 0.4.21,
  `ignore` 0.4.33 to 0.4.34,
  `notify` 8.2.0 to 8.2.1,
  `tokio` 1.53.2 to 1.54.1.
- `package/jsonc-edit/core-rust.fuzz`:
  `arbitrary` and `derive_arbitrary` 1.4.2 to 1.5.0,
  `syn` 2.0.119 to 3.0.9.
- `package/linter/monochromatic-lint.fuzz`:
  `arbitrary` and `derive_arbitrary` 1.4.2 to 1.5.0,
  `globset` and `ignore` as above.
- `package/mpm-shadow/cli`:
  seven packages,
  the RustCrypto family (`sha2` 0.10.9 to 0.11.1,
  `digest`,
  `block-buffer`,
  `crypto-common`,
  `cpufeatures`,
  plus new `const-oid` and `hybrid-array`).
- `package/music-player/android-app/rust`,
  `package/music-player/desktop-app`,
  `package/music-player/truepeak-core`:
  `tokio` 1.53.2 to 1.54.1 in each.

Cause:
the manifests of these workspaces were last changed by commit 581190711e (2026-11-01T20:57:42-05:00,
"bump catalog floors and
regenerate derived dependency files"),
which raised minimum versions (`arbitrary >= 1.5.0`,
`tokio >= 1.54.1`,
`memchr >= 2.8.3`)
and,
for the forbidden-strings dependents,
a path crate's version moved to 0.6.1;
their locks were not regenerated.
Owner of every package:
`@Aquaticat` by `.github/CODEOWNERS`;
no handover names a different owner for these ten.
The catalog tooling that raises the floors lives in `package/dependency-catalog` (`cli-rust-update`,
`cli-tighten`),
and
`native:lock:members` (`cargo update --offline --workspace`) is the existing task that records such a change for one crate.
