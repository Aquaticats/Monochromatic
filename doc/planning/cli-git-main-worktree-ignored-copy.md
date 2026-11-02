# A worktree added from the main checkout carries its ignored state

Status: implemented on branch `main-ignored-copy` (not merged, not released), 2026-11-02.
Scope: `package/git-policy/cli` (crate `monochromatic-git-policy-cli`), Linux.
The measurements ran on this machine under the load there was (one-minute load average between 14 and 35 on 28 threads),
inside `systemd-run --user --scope` or `--unit` with `CPUQuota`, `MemoryMax`, `TasksMax` and `IOWeight` of at most 50.
Fixtures were throwaway worktrees under `.claude/worktrees/` and a clone under
`/var/home/user/.cache/main-ignored-copy`;
all were removed.

## Owner decisions that bound this design

- 2026-11-01: "carrying over ignored files like node_modules into a new worktree is expected, because agents and people
  frequently forget to manually install node_modules in new worktrees."
- 2026-11-01: "yes but that's intended and I also intended for this to not be so heavy because we can take advantage of
  btrfs."
- 2026-11-02: "node_modules must be copied."
- 2026-11-02: "Builds must be copied. Other ignored files must be copied."
- 2026-11-02: "Skip nested worktrees and caches. The point is to make new worktrees immediately functional."
- 2026-11-02: "Skip run records (Recommended)."
- Standing rules used here: keep package managers out of the wrapper (running one from Git is a cross-component concern
  the owner has not approved); keep the settlement lock, the journal and recovery; `--no-worktree-copy` keeps working.

## Summary

- Before: an add run from the main checkout copied only the configured `rulesFile` files
  (`package/git-policy/cli/SPEC.md`, section "Linked-worktree ignored-state synchronization"), so a new worktree had no
  `node_modules` and every `mise run` task failed with `ERR_MODULE_NOT_FOUND` until a dependency install ran.
- Now: the same add copies the main checkout's whole ignored state except what a new worktree does not need.
  The rule is three families of directory names (`src/native/worktree_copy_prune.rs`): other checkouts, caches, and run
  records.
  A linked or bare source is unchanged: it copies everything it ignores except its nested registered worktrees.
- The copy was too slow as written (it re-read every staged byte, one thread cloned one file at a time, the install
  linked and then unlinked every entry, and its log was synchronized every 512 entries), so four changes keep the
  settlement lock, the journal and recovery and cut the cost; see "Design" and "Measurements".
- Wall time of a main-sourced add on this machine is in "Measurements" (section "The add itself").
  It is above the ten seconds the coordinator named as the point to report, because the ignored state is about 190
  thousand entries that must each be cloned, linked into place and recorded; the btrfs snapshot rung of the ladder on
  `btrfs-worktree-ladder` is the route to two or three seconds and is discussed under "The ladder".

## Facts

### The main checkout

- `findmnt --target /var/home/user/Monochromatic`: btrfs, `rw,relatime,seclabel,compress=zstd:1,ssd,discard=async,
  space_cache=v2,user_subvol_rm_allowed,subvolid=257,subvol=/home` (the mount holds `/var/home`).
- The checkout root is already a btrfs subvolume: `btrfs inspect-internal rootid /var/home/user/Monochromatic` printed
  3569 (the home subvolume is 257), the root directory has inode 256, and its birth time is 2026-11-01 17:58.
  The brief and the coordinator's follow-up said it was not yet a subvolume; the measurement says it is.
- `git worktree list --porcelain` registers 228 worktrees; 139 of them have their root inside the main checkout, all
  under `.claude/worktrees/`.
  Of those 139, 91 are subvolumes (inode 256 at their root) and 48 are ordinary directories
  (`node nested.ts` in the scratch directory, which `stat`s each root).
  A worktree created by the released wrapper (0.5.13 and 0.5.14) is an ordinary directory.
- Git's own exclusion stack (`git -C /var/home/user/Monochromatic ls-files --others --ignored --exclude-standard
  --directory`, 0.055 s) names 949 ignored roots.
  Without the 139 nested worktrees they hold 693,346 entries and about 75 GiB apparent
  (`scan-ignored.ts`: one `find -printf .`, one `du --summarize --apparent-size` and one `du --summarize` per root).
  The nested worktrees hold about 22 thousand entries each (four sampled: 22,266; 24,603; 22,746; 22,289), about 3.1
  million in all and 160 GiB apparent.

### What the ignored set holds

Entries and apparent MiB per kind, nested worktrees excluded (`inner.ts` and `scan-ignored.ts`):

- `target/` build outputs, 37 trees: 290,547 entries, 63,270 MiB.
  Inside them: `debug/incremental` 62,257 entries and 19,769 MiB; `debug/deps` 8,363 and 13,968; `debug/build` 21,627
  and
  11,016; `verification` 97,228 and 6,007; `release/deps` 4,849 and 5,375; `dylint` 23,583 and 4,269;
  `zig-global-cache` 22,971 and 379.
  The largest tree is `package/ide/app/target` with 144,538 entries and 39,391 MiB.
- `node_modules`, 221 trees: the repository root one holds 175,713 entries and 3,747 MiB; the 220 per-package ones hold
  145,466 entries and 2,464 MiB.
  In the root one: `.pnpm` (the virtual store) 49,867 entries and 913 MiB; `.pnpm-store` 37,174 and 999;
  `.monochromatic` 53,869 and 1,012; `.cache` 34,663 and 825; 1,297 symbolic links in all.
  In the per-package ones: `.monochromatic` 143,110 entries and 2,458 MiB, so almost all of their entries are logs.
- `.pnpm-store/` at the root, 37,235 entries and 991 MiB (a second, older pnpm store).
- `dist/` 202 trees, 4,960 entries, 601 MiB; `build/` 4 trees, 5,908 entries, 931 MiB;
  `package/vm-builder/cli/output` 5 entries, 3,783 MiB; the root `.cache/` 1,248 entries, 42 MiB (tool launchers,
  `forbidden-strings.rules.txt`, `toolchain.env.json`); every other `.cache` directory, 228 trees with 979 entries and
  about 58 MiB (the 2,227 entries and 101 MiB of all `.cache` directories minus the root one), holding only `typescript`
  build info.
- 191 ignored files (local configuration, `*.local.*`, `.env.local.json`, `mise.local.toml`), 244 MiB.
- No socket, FIFO or device node exists in the ignored set (`find ... -not -type f -not -type d -not -type l` found
  none).
- 56 ignored entries are unreadable by the user, all but a few under `node_modules/.monochromatic`: root-owned files of
  mode 0600 such as `node_modules/.monochromatic/2026-11-01/2026-11-01T19-30-26.647Z.<id>.log.jsonl`.
  A copy that opens every file fails on them.

### The wrapper before this change

- `package/git-policy/cli/src/native/worktree_copy_registry.rs`, `observe_copy_source`: "A main checkout is not a bypass
  any more for the named paths ... but it is never copied as a whole: its ignored trees are the repository's heaviest."
- The stage verified itself by reading both files again: `worktree_copy_compare.rs`, `regular_file_bytes_equal`, called
  for every staged file.
  On the roughly 40 GiB of build products in this set that is a read of about 80 GiB.
- The stage copied one entry at a time, and the install created one hard link per entry, with `lstat` calls for every
  ancestor of every entry and one synchronized log append per 512 entries; completion then unlinked every staged entry.
- An add run in a worktree that is its own subvolume cannot hard link from another subvolume, and pnpm's store choice
  follows that (next section).

### pnpm and the store

- `pnpm store path` in a fresh ordinary-directory worktree under the main checkout printed
  `/var/home/user/Monochromatic/.pnpm-store/v11`, with the warning "The store at /home/user/.local/share/pnpm/store/v11
  is not used because packages cannot be hard linked from it into this project."
  The same command in a worktree that is its own subvolume printed that worktree's own
  `node_modules/.pnpm-store/v11`, and `pnpm install --frozen-lockfile --offline` there failed after 18.8 s with
  `ERR_PNPM_NO_OFFLINE_TARBALL ... mute-stream@3.0.0 ... snapshot not present in local store`.
  The same install with `--store-dir /var/home/user/Monochromatic/.pnpm-store` succeeded in 1.8 s: pnpm clones across
  subvolumes (`package-import-method` is `clone-or-copy` here).
- In an ordinary-directory worktree, `pnpm install --frozen-lockfile --offline` took 2.1 s and 2.0 s and succeeded for
  the
  current `main` lockfile and for the commit `745dcfc22f` used earlier today.
  The coordinator reported that offline failed today; I could not reproduce it in an ordinary-directory worktree.
  The failing case I found is a worktree that is its own subvolume, which is what the btrfs ladder will make every new
  worktree.
- pnpm documents the rule: "If no directory above the project accepts a hard link at all ... the store is created inside
  the project instead: at node_modules/.pnpm-store" (see "Precedent").

### What a new worktree needs from the main checkout

Found by experiment in throwaway worktrees (`ignored-copy-probe-a`, `-b`, `-c`, `-e`, removed afterwards):

- With nothing copied, `mise run //package/git-policy/cli:test:unit` fails with `ERR_MODULE_NOT_FOUND`.
- With only the root `node_modules` (without `.pnpm-store`, `.cache` and `.monochromatic`) it still fails.
- With the root `node_modules` and the package's own `node_modules`, it fails on
  `@monochromatic-dev/test-harness-core-diagnostic`: the per-package `node_modules` of the workspace packages it imports
  are needed too.
- With the root `node_modules` and the per-package `node_modules` of the 18 packages in the workspace closure of the
  task
  (computed from the `@monochromatic-dev` symbolic links; 17 for the CLI package and 13 for the task runner, 18
  together),
  `test:unit` exits 0 with 70 passing lines and `lint:types` exits 0.
  Those copies excluded `.monochromatic`, `.cache` and `.pnpm-store`.
- The container gate launcher `bin/test-native-container.ts` resolves its two bare imports in that set and fails to
  resolve them with nothing copied (`import.meta.resolve`); I did not run the container build itself.
- The session's `PATH` points at the main checkout's `.cache/toolchain-bin` and `node_modules/.bin`, so executables
  resolve to main whatever the worktree holds; the module resolution result above does not depend on that.
- pnpm's `.pnpm-workspace-state-v1.json` holds 239 absolute paths of the main checkout, and `.modules.yaml` one; a
  copied
  `node_modules` carries them.
  I did not test what a later `pnpm install` does with them in a copied tree.

### Classification of the top-level ignored paths

Each path is judged by the owner's purpose: does the new worktree need it to run, build, test or lint at once?

- `node_modules/` (root and per-package) minus its caches and logs: needed (dependencies and links).
- `node_modules/.pnpm-store`, `.pnpm-store/`: cache (a content-addressed download store; pnpm rebuilds it).
- `node_modules/.cache`: cache (jiti, file-enforcer).
- `node_modules/**/.monochromatic`: run records (logs of earlier runs; includes the unreadable root-owned files).
- `target/` minus the next three: needed (builds).
- `target/*/incremental`, `target/zig-global-cache`: cache (compiler state rebuilt on demand).
- `target/verification`: run records (outputs of verification runs).
- `dist/`, `build/`, `output/`, `jniLibs/`, generated `src/*.js`: needed (builds).
- `.cache/` at the repository root: needed (tool launchers and generated inputs that tasks read).
- `.cache/` elsewhere (`typescript` build info): cache.
- `.gradle`, `.kotlin`: cache.
- `.terraform`: kept (provider plugins that `tofu validate` needs without an init).
- `mutants.out`, `mutants.out.old`, `playwright-report`, `test-results`, `.serena/cache`: run records.
- Local files (`*.local.*`, `.env.local.json`, `mise.local.toml`, `.claude/settings.local.json`, `.idea`, `.vscode`,
  `.serena/memories`, fuzz `corpus/`): needed.
- Registered worktrees under `.claude/worktrees/` and any directory holding `.git`: another checkout, never copied.

The owner approved this rule on 2026-11-02 ("Skip nested worktrees and caches", "Skip run records (Recommended)").

### Precedent

Fetched 2026-11-02.
Each quoted phrase was confirmed to be in the rendered page text with `agent-browser`
(`document.body.innerText.includes(...)`, all true); I did not inspect the highlight a fragment draws.

- Claude Code copies only what a `.worktreeinclude` file names, and only if Git ignores it:
  "Only files that match a pattern and are also gitignored are copied" ([source][claude-include]).
  It tells the user to "initialize your development environment there" ([source][claude-setup]).
- worktrunk (`wt step copy-ignored`) copies from the primary worktree by default,
  "By default the copy runs from the primary worktree into the current one" ([source][wt-source]),
  copies "All gitignored files ... by default, except for built-in excluded directories" ([source][wt-what]),
  skips nested worktrees, and reflinks:
  "Reflinks are per file (like cp -Rc), so copy time scales with file count" ([source][wt-cow]).
  Its sample shows 20 s and about no extra disk for a 14 GB `target/`.
  It suggests symbolic links for `node_modules` when a project has no native dependencies.
- VS Code names `node_modules` as the common case:
  "Git doesn't copy files that are excluded by .gitignore" ([source][vscode-include]),
  and offers `git.worktreeIncludeFiles` (copy) and `git.worktreeSymlinkFolders` (symbolic link folders that contain no
  tracked files).
- pnpm's guidance for several checkouts is a shared store: each worktree has its own `node_modules`, and
  "dependencies are shared across all of them" through one content-addressable store ([source][pnpm-worktrees]),
  with `virtualStoreType: global` so a worktree's `node_modules` holds only symbolic links, and the warning "Do not use
  one writable pnpm store for mutually untrusted agents or users."
- pnpm's store location rule: "the store is created inside the project instead" when no directory above the project
  accepts a hard link ([source][pnpm-store]).
- JetBrains' worktree page, git-town's documentation and Graphite's multiple-worktrees page say nothing about ignored
  files in a new worktree (searched 2026-11-02: the JetBrains page text, the Graphite page text, and the git-town
  repository and site for "worktree" and "node_modules"; the searches ran and found no statement).

[claude-include]: https://code.claude.com/docs/en/worktrees#:~:text=Only%20files%20that%20match%20a%20pattern%20and%20are%20also%20gitignored%20are%20copied
[claude-setup]: https://code.claude.com/docs/en/worktrees#:~:text=initialize%20your%20development%20environment%20there
[wt-source]: https://worktrunk.dev/step/#:~:text=By%20default%20the%20copy%20runs%20from%20the%20primary%20worktree
[wt-what]: https://worktrunk.dev/step/#:~:text=All%20gitignored%20files%20are%20copied%20by%20default
[wt-cow]: https://worktrunk.dev/step/#:~:text=Reflinks%20are%20per%20file
[vscode-include]: https://code.visualstudio.com/docs/sourcecontrol/branches-worktrees#:~:text=Git%20doesn%27t%20copy%20files%20that%20are%20excluded%20by%20.gitignore
[pnpm-worktrees]: https://pnpm.io/git-worktrees#:~:text=dependencies%20are%20shared%20across%20all%20of%20them
[pnpm-store]: https://pnpm.io/settings/store#:~:text=the%20store%20is%20created%20inside%20the%20project%20instead

## Design

### The rule

`src/native/worktree_copy_prune.rs` holds the rule as `Prune::MainCheckout { run_records }`:

- another checkout: a directory holding a `.git` file or directory is left out whole; registered worktrees are
  excluded by
  path as before;
- caches: the names `.pnpm-store`, `.gradle`, `.kotlin`; `.cache` at any depth except as the first path component;
  `incremental` and `zig-global-cache` below a `target` component;
- run records, left out while `COPY_RUN_RECORDS` is `false` (it is): `.monochromatic` below a `node_modules` component,
  `verification` directly under `target`, `mutants.out`, `mutants.out.old`, `playwright-report`, `test-results`, and
  `cache` directly under `.serena`.

The decision uses path components only, so it is the same for a root Git listed and a child the walk found.
`CopySelection::MainCheckout` selects it; `CopySelection::Ignored` (linked and bare sources) applies none.

### Unreadable entries

The walk asks `access(2)` whether the user can read each file and list each directory.
An entry that fails is left out with its subtree and named in one stderr line after the copy
(`cli-git: left out 2 unreadable ignored entries from "<source>": "a", "b"`, the first three paths and a count of the
rest);
the stage logs the full list as a warning.
This applies to every source, because a copy must never fail over a file the user cannot open.

### What FICLONE guarantees and how a concurrent writer is detected

This is the SPEC change a reviewer should check ("regular-file bytes" became "regular-file content").

- `FICLONE` makes the destination share the source's data extents as of one instant (`ioctl_ficlone(2)`: the call clones
  the file's data blocks, and a write to either file afterwards is copy-on-write).
  I did not read the btrfs source for the locking; the argument needs only that the clone is a point-in-time snapshot of
  the source file's data, which the manual page states.
  The clone's bytes therefore equal the source's bytes at that instant, whatever a writer does afterwards.
- The risk is a write that lands before the clone and after the manifest, or a write that the copy cannot see
  afterwards.
  The old check caught every difference by reading both files again.
  The new check proves the source did not change between the clone and the end of the copy, by comparing a stamp
  (size, modification time, change time, inode) taken before the clone, after the clone and at the final check.
- Why a stamp is enough: the change time of a file moves on every write, truncate, rename-over and metadata change,
  and no
  program can set it; the modification time and size move on a write; the inode moves when a file is replaced.
  A write that completes between the stamps changes at least the change time.
- The granularity gap: kernel timestamps come from a coarse clock, so a write inside the same tick as the stamp could
  leave
  the stamp unchanged.
  The copy therefore trusts a stamp only when the file's change time is at least 20 ms older than the clock at the
  clone;
  a more recent file is compared byte for byte at the end, as before.
- A file whose clone is refused is copied byte by byte and compared byte for byte, as before.
- The directory, link and manifest checks are unchanged: the final check still walks the source and the stage again and
  requires the same paths, kinds, modes and link targets.
- What is no longer detected: a source file rewritten with the identical size, modification time, change time and inode,
  which needs a clock step back and a restored time; the SPEC already excludes "transient source mutations that revert".

### Parallel cloning, bounded by CPU

- A manifest of at least 4096 entries is copied by `min(available_parallelism, 8)` threads over contiguous slices, after
  every directory exists; the first failure sets a flag that stops the other workers.
  `available_parallelism` follows the process's CPU affinity and cgroup quota, so a quota of two CPUs gets two threads.
- The install creates its files and links in the same way, one batch of 4096 entries at a time: intents for the batch
  are
  recorded and synchronized first, directories and scaffolding are created in order, the leaves are created in parallel
  slices, then the creations are recorded.
  Every path a worker created is returned even when another worker fails, so the in-memory rollback list is complete.
- The stage's removal splits the stage three directories deep and removes the subtrees on up to eight threads.
- Ordering that recovery relies on is kept: claims precede creations in the log, the log is synchronized per batch, the
  journal phases are unchanged, and rollback still walks the created list in reverse.

### What did not change

- The settlement lock, the stage claim, the journal, the install log, recovery and the rollback rules.
- The destination preflight: an existing destination entry must match the stage exactly.
- `--no-worktree-copy`: still recognized, stripped before Git, and it skips the copy and the recovery.
- A main checkout command that creates or moves no worktree still costs no filesystem reads.
- The configuration is no longer loaded for a main-checkout add (only an index writer loads it, as before).

## Measurements

All wall times are medians where a series is stated; "load" is the one-minute load average at the start.

### Copy mechanisms, on a copy of node_modules alone (`bench.ts`, three rounds, shuffled, `CPUQuota=400%`)

Per-file reflinks with `cp --archive --reflink=always`, which is the same mechanism as the stage's `FICLONE` without the
stage, manifest, verification or install, into a directory inside a throwaway worktree on the main checkout's subvolume.
Each arm has a same-command control.

- Closure set (root `node_modules` minus store, cache and logs, and 17 package trees): 50,104 entries, 3.1 s median
  (2.8, 3.2, 3.1); control 3.1 s (3.0, 3.1, 3.7).
- All `node_modules` content of every package, minus `.monochromatic`: 52,626 entries, 9.0 s (14.9, 8.3, 9.0); control
  4.7 s (4.9, 4.2, 4.7).
  The spread between the arm and its control, 4.3 s, is the noise to read the other numbers against.
- Every `node_modules` tree whole: 321,385 entries, 32.1 s (27.2, 32.1, 38.3); control 20.3 s (42.1, 19.0, 20.3).
- All ignored roots except `.claude` and `target`: 402,605 entries, 47.9 s (39.5, 47.9, 81.8); control 45.9 s (45.9,
  34.7,
  61.4).
- Removal of those copies took 1.0 s for the closure set and 13 to 18 s for the larger ones.
- A copy is roughly 60 to 100 microseconds per entry under this load; the previous design measured about 350
  microseconds
  per entry in the btrfs planning document.

### pnpm

- `pnpm install --frozen-lockfile --offline` in a fresh ordinary-directory worktree: 2.1 s, 2.0 s (exit 0).
- The same in a worktree that is its own subvolume: 18.8 s and exit 1 (`ERR_PNPM_NO_OFFLINE_TARBALL`); with
  `--store-dir` naming main's store: 1.8 s, exit 0, and `test:unit` passes (70 passing lines).
- The online install was not timed: the offline case fails only for the store location, which the explicit store fixes.

### The add itself

The throwaway fixture holds main's ignored state without the nested worktrees: 190,087 entries after the rule (152,944
regular files cloned under a stamp), reflinked from the real checkout.
Commands: `bench-add.ts` (in `/var/home/user/temp/agent/main-ignored-copy`) alternates four arms in shuffled order,
each as
`systemd-run --user --scope` with `CPUQuota=800%`, `MemoryMax=8G`, `TasksMax=256`, `IOWeight=30`, then removes the
worktree
and its branch.
Arms: the new build (`new`, and `new-ctl` as the same-binary control), the same code with `COPY_RUN_RECORDS` set to
`true`
(`with-run-records`), and the released 0.5.14 wrapper (`installed-baseline`, which copies one file).
The series was stopped after two full rounds because the `with-run-records` arm alone took several minutes per run.

- `new` and `new-ctl` together, six runs: 87.8, 140.9, 26.8, 92.2, 42.0, 90.7 s; median 89.3 s, range 26.8 s to 140.9 s.
  The two arms' medians differ by 2.9 s (87.8 and 90.7), far below the spread inside one arm, so the host load, not the
  arm, explains the differences.
  One-minute load average at the start of those runs: 16.3 to 38.2.
- `installed-baseline`, three runs: 44.6, 26.2, 4.8 s; median 26.2 s.
  This is Git's own `worktree add` of about 20 thousand tracked files plus the one-file copy; its range shows how much
  of
  every number here is load.
- `with-run-records`, two runs: 345.5 s and 159.0 s (487,637 entries copied, the run records add 297,550 entries);
  removing that worktree took 213 s and 39 s.
- Removing a worktree made by `new` (the user's later `git worktree remove`): 15.6, 9.5, 11.4, 21.6, 40.9, 12.0 s;
  median 13.7 s.
- A single run of the final build at a calmer moment (one-minute load 7.5, five-minute 14.0), with the debug log on:
  31.6 s wall in all, 26.8 s of it the copy, user CPU 5.0 s and system CPU 45.2 s, peak memory 0.5 GB.
  Phases: selecting the roots 1.3 s; staging 11.4 s (walk 2.6, clone by 8 workers 4.6, modes 1.1, final check 3.2);
  creating the journal 1 ms; installing 11.2 s (preflight 0.6, create and record 7.3, modes 1.2); removing the stage
  2.9 s.
- Earlier builds of the same series, for what each change bought (single runs, load 14 to 24): the first working build
  took
  104 s (45 s in the install, 68 s removing the stage in one run); after the cached parents and the larger log batch the
  install took 20 to 29 s; after parallel creation and parallel removal, 7.3 s and 2.9 s in the calmer run above.

What a main-sourced add costs on this machine, as the coordinator asked: about 32 s at light load and a median of
about 89 s
under the load of the afternoon (range 27 s to 141 s), against 5 s to 45 s for the same command without the copy.
That is above ten seconds in every run.
The cost is the entry count: 190,087 entries, each cloned, linked into place and recorded, plus the walks that prove
them.
The btrfs snapshot rung measures 2 to 3 seconds for the same command on a clean subvolume, but it carries the caches and
records this rule leaves out and declines while any nested worktree is an ordinary directory; see "The ladder".

## Verification

- `cargo nextest run --lib` on the host: 2841 passed, 4 skipped; `cargo nextest run --test native_binary`: 445 passed, 1
  skipped; `cargo clippy --all-targets -- -D warnings`: clean.
  The container gate, the Windows clippy lane (`native:clippy:windows`) and the other test targets were not run.
- Guard tests shown red with the guard removed (each change made after committing the tests, then restored by editing the
  lines back, and the working tree compared with the commit):
  1. the main-checkout selection reverted to named paths: `main_checkout_source_carries_dependencies_and_builds` and
     `main_worktree_is_observed_with_the_main_checkout_selection` failed;
  2. the leave-out rule and the checkout skip removed from the walk and the root filter:
     `main_checkout_source_leaves_out_checkouts_caches_and_run_records`,
     `unreadable_entries_and_other_checkouts_are_left_out_of_a_main_stage` and
     `an_interrupted_main_checkout_copy_is_resumed_from_its_stage` failed;
  3. the stamp no longer trusted in the final comparison: `an_unchanged_stamp_stands_in_for_reading_the_bytes` failed.
- A kill mid-copy: `an_interrupted_main_checkout_copy_is_resumed_from_its_stage` (a journal left in the installing phase
  with an intact main-checkout stage is resumed) and the binary test
  `an_interrupted_main_checkout_copy_is_recovered_by_the_next_creation` (a staged journal is recovered by the next add
  from the main checkout, with the whole ignored state arriving).
  I did not send a real signal to a copy in flight.

## Assumptions and limits

- The fixture is `git clone --shared` of the main checkout in `/var/home/user/.cache/main-ignored-copy`, with the main
  checkout's ignored roots reflinked into it (`cp --archive --reflink=always`, 693 thousand entries, no `.claude`).
  It sits in the home subvolume, not the main checkout's subvolume, so a link from stage to destination stays inside one
  subvolume as in the real case, but the clone source is across subvolumes.
- Git LFS smudging was skipped for the fixture (`GIT_LFS_SKIP_SMUDGE=1`); the real checkout's LFS files are checked
  out, which
  adds nothing to the ignored copy.
- Load was never quiet: every number carries the load it ran under, and arms were alternated with a control.
- The container gate was not run; the host library and binary test sets and clippy were.
- A registered worktree that is itself a subvolume appears in a snapshot as an empty stub; I did not run the ladder's
  binary (see "The ladder"), I read its code at `0e432cd0b3`.
- The stamp argument rests on kernel behavior I did not test with a concurrent writer; the unit tests cover a changed
  file
  and the byte-comparison fallback, not a race.

## Options considered

Ranking: A > D > B > C > E, with the reason for each adjacent pair after the list.

- A, the chosen design: the reflink stage under the leave-out rule, verified by stamps, cloned and installed in
  parallel.
  Pros: works on any filesystem (byte copy on ext4), keeps the lock, journal and recovery, needs no configuration,
  copies
  what the owner listed.
  Cons: cost grows with the entry count; it is the slowest of the correct mechanisms on btrfs; the verification weakens
  from bytes to stamps for cloned files.
  Cost: see "The add itself".
  The owner sees: one summary line, "copied N ignored filesystem entries from ...", and a "left out" line when files
  were
  unreadable.
- D, add the install-by-rename of whole roots on top of A (not built): rename `stage/payload/<root>` to the
  destination when
  that destination root is absent.
  Pros: the install and the stage removal become a handful of renames, removing most of the install and removal time.
  Cons: the journal, install log, recovery and rollback all assume entries linked from a stage that stays intact; a root
  moved out of the stage needs its own rollback (rename back) and its own recovery rule.
  Cost: unmeasured, estimated from the phase times below as the larger part of install and removal.
  The owner would see the same output, faster.
- B, the btrfs snapshot rung (`btrfs-worktree-ladder`): one `btrfs subvolume snapshot` of the main checkout.
  Pros: seconds, shares all blocks.
  Cons: it carries caches and run records the owner asked to skip and the nested ordinary-directory worktrees it
  declines
  on; it also needs the destination on the same btrfs filesystem.
  Cost: the ladder agent measured 15.15 s to 2.43 s for a main-checkout add.
  The owner would see a fast add that includes everything ignored.
- C, run `pnpm install` for the new worktree: rejected by the owner's constraint (package managers stay out of the
  wrapper),
  and it does not carry builds.
  Measured 1.8 s to 2.1 s offline when the store is reachable, which is why it stays a good manual step.
- E, symbolic links to the main checkout's `node_modules` (worktrunk's and VS Code's option): not built and not
  measured here.
  By construction a worktree's `node_modules` would resolve through links into the main checkout, so workspace packages
  would resolve to main's sources, not the worktree's; that is a reason to reject it, but I did not run a demonstration.

Reasons: A over D is only about risk, D is faster but changes the journal contract and needs its own fault tests, so A
first.
D over B because B copies caches and records and declines when any nested worktree is an ordinary directory.
B over C because B carries builds and C does not.
C over E because C produces a correct, independent tree and E shares mutable state between worktrees.

## The ladder

Read at `0e432cd0b3` on `btrfs-worktree-ladder` (code in flux, not edited).

- `worktree_ladder_run.rs`, `choose_rung`, takes the snapshot rung only when the source is a subvolume root,
  `snapshot_blocker` finds no nested registered worktree that is an ordinary directory, and the clean check passes.
  The main checkout is a subvolume but has 48 ordinary-directory nested worktrees, so the snapshot rung declines today
  and
  the clone rung runs: it clones tracked files only (`worktree_clone.rs`) and reports nothing as carried
  (`LadderResult { carried: None }`), so the lifecycle's own copy then runs for the destination.
  With this change that copy now carries the ignored state, so a main-sourced add on this machine will do clone rung
  plus
  this copy until the 48 worktrees are removed or migrated.
- When both merge, the ladder needs three things from this change:
  1. `carried` is the destination whose ignored state a snapshot already carried; the lifecycle skips that destination.
     For a main-sourced snapshot the carried state is the whole ignored set, caches and run records included, while this
     change defines a smaller set for main.
     Either the snapshot rung removes the left-out names from the new subvolume with `Prune::MainCheckout` (the rule
     is a
     pure function of paths) or the owner accepts the larger set there.
  2. The clone and fallback rungs must leave the destination as Git's checkout does, with the lifecycle's copy still to
     come; the fixed `StagedSnapshot.unreadable` and the new `CopySelection::MainCheckout` do not change that contract.
  3. Hard links from the stage into a destination that is its own subvolume fail with `EXDEV`; `install_file` already
     falls
     back to an exclusive copy-on-write request, which keeps the data shared, but the install loses the hard-link
     atomicity and the stage removal then deletes real files; a rename-based install (option D) is the better fit there.
- The pnpm finding matters to the ladder: every worktree it creates is its own subvolume, where pnpm's default store
  choice makes `--offline` fail.
  A project-level `storeDir` in `pnpm-workspace.yaml` naming one store on the main subvolume would avoid it; I did not
  change pnpm configuration.

## Review requests

- The stamp argument under "What FICLONE guarantees and how a concurrent writer is detected", and the 20 ms settled
  rule.
- The unreadable-entry policy (skip and say so, for every source).
- Whether `CopySelection::Paths` and `read_named_ignored_roots` should now be deleted: nothing in the release path
  builds
  them any more; they stay because their tests and the binary tests use them.
- Whether the linked-worktree copy should adopt the cache rule as well; the owner's words named only the main checkout.
