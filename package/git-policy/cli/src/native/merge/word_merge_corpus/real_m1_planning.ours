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
the stage writes the full list to the verbose debug log only, so stderr carries the one notice.
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

- `cargo nextest run --lib` on the host: 2843 passed, 4 skipped; `cargo nextest run --test native_binary`: 445 passed, 1
  skipped; `cargo clippy --all-targets -- -D warnings`: clean.
  The container gate, the Windows clippy lane (`native:clippy:windows`) and the other test targets were not run.
- Guard tests shown red with the guard removed (each change made after committing the tests, then restored by editing
  the
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

## Snapshot route measured

Requested by the owner on 2026-11-02 ("Hold until snapshot measured"); `main-ignored-copy` stays unmerged until the
owner
chooses.
Everything ran on a throwaway fixture under `/var/home/user/.cache/snapmeas`, on the main checkout's filesystem; the
real
checkout and every real worktree were only read.
The ladder branch `btrfs-worktree-ladder` (head `414bb552d1`) owns the snapshot rung; I read it and did not run it, so
the
numbers below are for the snapshot and the removals the owner asked for, not for the ladder's own steps (its tree
update,
index install and checkout line), which its author measured at 2.43 s for a whole main-checkout add.

### The fixture

- `fix-root`, a btrfs subvolume holding: a reflink copy of main's real `.git` (22 thousand entries, 1.9 GB), the tracked
  tree at main's `HEAD`, main's whole ignored state reflinked in (693 thousand entries, the same set the add copies),
  four nested linked worktrees that are subvolumes and four that are ordinary directories (each holds the tracked tree,
  about 22.5 thousand entries, registered in the copied `.git`).
- The skip set of the rule in `worktree_copy_prune.rs`, expressed as `find` predicates in `snap-bench.ts`, matches 479
  trees
  and 494,744 entries in the fixture (the same count as the earlier estimate for main).
  Finding them takes 1.2 s.
- Scaling to the real main checkout: the skip set is the same set, the `.git` is the same copy, and the nested count
  is 48
  ordinary and 91 subvolume worktrees instead of 4 and 4; per-worktree figures below are multiplied for that, and
  marked.

### What a snapshot shows

- `btrfs subvolume snapshot fix-root snap` shows each nested subvolume worktree as an empty directory with inode
  number 2
  (checked with `stat` and `ls -A`), and each nested ordinary worktree as a full copy of its files, including its `.git`
  file, which still reads `gitdir: <original>/.git/worktrees/<id>`.
- The copy of the main `.git` is a whole second repository directory in the new subvolume.

### Measured costs (three rounds, shuffled, a same-command control, one-minute load 2.1 to 6.0)

- The snapshot itself: median 0.15 s over fifteen snapshots (0.02 s to 0.94 s; most under 0.4 s).
  Its size does not matter: it is a copy-on-write root, and the fixture holds 700 thousand entries.
- Deleting the snapshot afterwards with `btrfs subvolume delete`: 55 ms to 73 ms (the kernel frees the extents
  afterwards;
  I did not time that).
  So removing a worktree that came from a snapshot costs under 0.1 s, against 9 s to 41 s for a worktree of copied
  entries.
- Foreground work that must precede the add returning, in either variant:
  1. unlinking the `.git` pointer file of every nested ordinary worktree in the copy: 3 ms to 4 ms for four (about 50 ms
     for 48, extrapolated);
  2. removing the copied `.git` directory, which the ladder replaces with the new worktree's pointer: 714 ms to 749 ms
     (nine runs) by `rm -rf`; renaming it into a trash directory is a single rename.
- Removing the nested ordinary worktrees' trees: 2.03 s to 2.29 s for four, 0.52 s each; about 25 s for 48
  (extrapolated).
- Removing the skip set (caches and run records by the rule): 13.1 s to 14.0 s, 27 microseconds per entry.
  Eight parallel `rm -rf` workers and one worker measured the same (13.1 s to 13.7 s sequential, 13.3 s to 14.0 s
  parallel, nine runs): unlinks in one subvolume do not scale with threads.
- Rename-to-trash variant: renaming the four ordinary nested trees, `.git` and the 479 skip-set trees into one directory
  inside the snapshot took 1.57 s to 1.60 s in total, of which 1.2 s is the `find` that lists them; emptying that
  directory
  afterwards took 14.2 s to 15.9 s.

### Options

- S1, snapshot, then delete everything left out in the foreground: 0.15 s + 0.7 s (`.git`) + 14 s (skip set) + 0.5 s per
  ordinary nested worktree (about 25 s for 48) = about 40 s for the real main checkout, and about 15 s if every nested
  worktree were a subvolume.
- S2, snapshot, rename what is left out into a trash directory, return, and empty the trash afterwards: foreground about
  1.8 s (0.15 s snapshot, 1.6 s listing and renaming, plus the nested pointers); background about 40 s of unlinking
  for the real
  checkout (14 s for the skip set, 25 s for 48 ordinary trees), about 15 s if all nested worktrees were subvolumes.
- S3, convert the ordinary nested worktrees to subvolumes once, then S2 or S1: removes the 0.5 s per worktree from every
  add, and removes the reason the ladder declines (see below).
- S4, make the cache and record directories nested subvolumes in main so a snapshot shows them as empty stubs (not
  measured): the removal cost would be a few microseconds per stub, but cargo, pnpm and anyone running `rm -rf` or
  `cargo clean` recreate the directories as ordinary ones, so the property would erode; I do not recommend it.
- A, the reflink stage already implemented on this branch: 32 s at light load and a median of 89 s under load (see "The
  add itself"), no dependency on the filesystem, no ladder.

### What the worktree looks like until a background removal finishes

- With S2 the trash directory sits inside the new subvolume, because a rename across subvolumes fails with `EXDEV`; it
  is an
  untracked directory in the worktree root, so `git status` lists it until it is gone, unless it lives under a path Git
  already ignores (`.claude/` here) or the removal is foreground.
- Safe meanwhile: the left-out caches and run records are moved, not present, and the new worktree is otherwise final.
  Unsafe if left in place: the `.git` files of nested ordinary worktrees (they name the real worktrees' administrative
  directories, so a Git command run inside a copy would read and write another worktree's index and `HEAD`) and the
  copied `.git` directory.
  Both must be gone before the add returns; both are renames or 4 ms unlinks, measured above.
- A crash between the snapshot and the end of the removal leaves a trash directory and a worktree that is otherwise
  complete; the next invocation in the repository can find the trash by its reserved name and finish, as it does for a
  stage today.
  A crash before the swap into place leaves a stage subvolume that the ladder already deletes on recovery.

### One-time conversion of an ordinary nested worktree (fixture only; needs the owner's approval to run for real)

Procedure and cost for each of four fixture worktrees (`migrate-bench.ts`):

1. `btrfs subvolume create <path>.migrating`: 4 ms to 5 ms.
2. `cp --archive --reflink=always <path>/. <path>.migrating/`: 1.01 s to 1.02 s for about 22.5 thousand entries.
3. `mv --no-target-directory --exchange <path> <path>.migrating`: 4 ms; the path now names the subvolume and the old
   tree sits at `.migrating`.
   (Without `--no-target-directory`, `mv --exchange` treats the second name as a directory and fails; the first attempt
   showed that, and removed only the new copy.)
4. `rm -rf <path>.migrating`: 0.43 s to 0.50 s.
5. Every worktree stayed registered (`git worktree list` named the path) and its status stayed clean.

- Cost per worktree: about 1.5 s of work, so about 72 s for the real 48, plus the index.
  The first `git status` after the copy took 5.0 s to 5.3 s (every file has a new inode, so Git re-hashes it; the
  fixture skips
  Git LFS smudging, so a real worktree with LFS files costs more; the earlier planning document measured 22 s for the
  main
  checkout) against 55 ms to 64 ms when the inodes were kept.
  The ladder's `migrate-to-subvolume` already rewrites the index stat fields (about 0.2 s per index in its document);
  I did
  not run it.
- Failure and recovery: the original is untouched until step 3. A crash after step 1 or 2 leaves `<path>.migrating`,
  which is
  deleted with `btrfs subvolume delete`; a crash after step 3 leaves the old tree at `<path>.migrating`, harmless,
  deleted
  by `rm -rf`. The registration never changes because the path and the `.git` file are the same bytes.
  The copy must be quiesced: edits made in the worktree during step 2 are lost at the swap, so the real procedure
  takes the
  worktree's `index.lock` for a final delta pass and swap, as the proven root migration does, and refuses while a
  process
  has its working directory inside.
- Needed at all? Not if S2 or S1 handles ordinary nested worktrees: their removal is the 0.5 s each measured above.
  It is needed only if the ladder's `snapshot_blocker` (which declines the snapshot rung while any nested registered
  worktree is an ordinary directory) stays as it is, or to save that 0.5 s per worktree per add.
  Relaxing the blocker is a change on the ladder branch, which I did not edit.

### The pnpm store problem (proposal only; no pnpm configuration was changed)

Measured in a subvolume worktree of the fixture against a reflink copy of main's store, with `pnpm install
--frozen-lockfile --offline` (`pnpm-bench.ts`):

- Baseline: fails, `ERR_PNPM_NO_OFFLINE_TARBALL`; pnpm chose `<worktree>/node_modules/.pnpm-store/v11`.
- `package-import-method=clone` alone: fails the same way (the store is chosen before the import method matters).
- `--store-dir <absolute>` on the command line: passes in 1.2 s.
- `storeDir: <absolute path>` in `pnpm-workspace.yaml`: passes in 1.0 s.
- `storeDir: ../../../../store` (relative, resolved from the workspace directory): passes in 1.1 s; it only fits a fixed
  depth below the store.
- `storeDir: ${HOME}/.cache/snapmeas/store` (the variable expanded): passes in 1.2 s.
- `storeDir: ~/.cache/snapmeas/store`: fails; the tilde is not expanded and pnpm created a directory named `~` in the
  workspace.
- Environment variable `npm_config_store_dir`: fails (the setting is ignored; the per-project store is chosen).
- Proposal: one line in `pnpm-workspace.yaml`, `storeDir: ${HOME}/.local/share/pnpm/store` (pnpm's own default global
  store, named explicitly).
  Every worktree, whether ordinary or its own subvolume, then shares one store and imports by clone across subvolumes
  (the
  repository already sets `package-import-method` to `clone-or-copy`).
  Costs: the store must be populated once by an online install; the main checkout's installs move from the project
  store to the home store; one writable store is shared by every checkout of the user, which pnpm's documentation warns
  against for mutually untrusted agents.
  I tested the setting only against a copy of the store, with `--offline`.

### Ranking

Ranking: S3 plus S2 > S2 > S1 > A > S4.

- S3 plus S2 over S2: converting the ordinary nested worktrees once removes 0.5 s of unlinking per worktree from every
  add
  and lets the ladder accept the snapshot without a blocker change; it costs a one-time run of about 72 s on the
  owner's real
  worktrees, which needs approval, and an index fix.
- S2 over S1: the foreground cost falls from about 40 s to about 1.8 s; the price is a trash directory in the worktree
  until a background removal ends and a process that must outlive the wrapper.
- S1 over A: about 40 s for the real checkout, in the foreground and deterministic, against 32 s to 141 s under load
  for A,
  with a removal of the new worktree in 0.07 s instead of 9 s to 41 s; and it keeps the worktree in one piece that the
  ladder already handles.
  The gap shrinks to nothing only when all nested worktrees are subvolumes and the owner accepts the skip set staying in
  the copy.
- A over S4: A works on any filesystem and has been run end to end; S4 is not measured and erodes.
- Against the owner's stated wish ("not so heavy because we can take advantage of btrfs"): S3 plus S2 returns in about
  2 s
  and finishes 15 s of unlinking later; A returns in 32 s at best.
- A and S* are not exclusive: A is the fallback when the source is not a subvolume or the snapshot is declined, which
  the
  ladder's rung order already expresses; this branch makes that fallback carry the ignored state.

## Applying the shared pnpm store

Owner decision, 2026-11-02: "Set the shared store (Recommended)".
Measured on a throwaway clone with its own `HOME` (reflinked pnpm metadata cache), then deleted; the real home store,
main's
store and main's `node_modules` were only read.

- `pnpm-workspace.yaml` is a hand-edited file: `file-enforcer.config.jsonc` does not name it, so the change goes to
  the file
  itself.
- The exact edit, one line at the top level of `pnpm-workspace.yaml`, with a comment saying why:
  `storeDir: ${HOME}/.local/share/pnpm/store`.
  The `${HOME}` form expands; `~` does not (it created a directory named `~`), and the environment variable
  `npm_config_store_dir` is ignored.
- What happens to an existing checkout whose `node_modules` was linked from the old store, once the line is present:
  1. every command except an install keeps working: `pnpm exec` ran against the old `node_modules` and passed, and
     `HOME`
     unset or pointing elsewhere did not matter to it;
  2. `pnpm install --frozen-lockfile --offline` against a new store that does not hold the packages fails with
     `snapshot not present in local store` AND LEAVES `node_modules` EMPTY (only `.pnpm-workspace-state-v1.json`
     remained).
     pnpm removes the old links before it finds the packages missing; there is no "unexpected store" refusal.
     With `HOME` unset the same install fails the same way, so the setting needs `HOME` in every environment that
     installs
     (CI and containers included);
  3. once the new store holds the packages, the same install re-links everything from it in 1.8 s to 2.9 s and records
     the new
     `storeDir` in `node_modules/.modules.yaml`.
- Filling the store, which must come first:
  - The real home store is `~/.local/share/pnpm/store/v11` (49 GB, a live pnpm 12 store with an `index.db`), so main's
    store cannot be copied over it: a copy would replace its index.
    On a fresh `HOME` a reflink copy of main's store took 2.1 s and worked.
  - `pnpm fetch --frozen-lockfile` fills the configured store from the network without linking, and took 10.7 s into
    an empty
    store; but it also empties the `node_modules` of the directory it runs in, so run it in a disposable worktree or
    clone
    that carries the edit, never in main.
- Reinstalling main is NOT safe while other sessions use main's `node_modules`: while the switch install ran (3.0 s),
  a loop
  resolving two installed packages failed 102 of 215 times.
  Run it when no session is working in main, or accept about 3 s of failures.
- Existing worktrees need nothing until they next run an install after receiving the edit; then they need the store
  filled
  first, or the install empties their `node_modules` as above.
- Order for the owner: edit on a branch; in a disposable worktree of that branch run `pnpm fetch --frozen-lockfile`
  once;
  merge; in main, with sessions idle, run `pnpm install --frozen-lockfile` (about 3 s); other worktrees pick it up at
  their next
  install.

## Conversion tool and the snapshot route (implemented on `feat/git-policy-cli-main-snapshot-worktree`)

Owner decisions, 2026-11-02: "Convert once + snapshot (Recommended)" and "Set the shared store (Recommended)".
The branch starts at `btrfs-worktree-ladder` (head `38e01bcd33` when I merged) with `main-ignored-copy` merged in; the
reflink
copy stays as the fallback rung, because a main-sourced add that is not served by a snapshot still goes through the
lifecycle
copy.

### What was built

- `git cli-git convert-worktree-to-subvolume [--dry-run | --abandon] (--all | <worktree>...)`
  (`worktree_convert.rs`, `worktree_convert_command.rs`, `worktree_convert_arguments.rs`): quiesce, copy with cloned
  data into
  `<path>.cli-git-converting`, prove with a second pass, exchange with `renameat2`, refresh the index stat fields,
  remove the
  old tree only when it is the directory the journal recorded.
  The journal `cli-git-convert` lives in the worktree's administrative directory.
  A rerun completes after a kill at any phase, including between the exchange and its journal line (the old tree's
  recorded
  device and inode say where it is); `--abandon` destroys only the subvolume the journal says it created, before the
  exchange.
  It uses no Git process: the cleanliness check is the ladder's own in-process one without its untracked-file step
  (`check_clean_tracked`, split out of `check_clean`), because untracked files are copied along.
- A snapshot of the main checkout now serves the add even with ordinary nested worktrees (`worktree_trash.rs`): it
  unlinks the
  copied `.git` pointers, moves the nested ordinary worktrees, other checkouts, caches and run records into
  `.claude/cli-git-trash/<40 hex>` inside the stage by rename, and after the swap hands that directory to the commit
  service
  as a work item of the new kind `trash` (`work_item.rs`, `service_work.rs`: a default `trash` handler method, one
  partition in
  the drain loop). The place is used only if Git ignores it; otherwise, and when no route to the service exists or the
  wake
  fails, the wrapper removes the directories itself before returning, and withdraws the work item it wrote.

### Edits inside the ladder's files (for the merge with its recovery rework)

- `worktree_snapshot_source.rs`: a new field `SnapshotSource.trash_ignored`, computed in `common_source` for a main
  checkout
  (one `git check-ignore`); `snapshot_blocker` returns nothing for a main-checkout source after its through-a-link
  check.
  Its test `btrfs_each_condition_that_rules_a_source_out_is_refused_with_its_reason` now asserts that the main
  checkout is not
  blocked and a linked source still is.
- `worktree_snapshot.rs`, `take_snapshot`: one call, `worktree_trash::prune_snapshot(stage, source)`, between making
  the stage
  private and `remove_stubs`.
- `worktree_ladder_finish.rs`, `serve_snapshot`: one call, `worktree_trash::dispose_pending_trash(...)`, after the
  swap and
  `settle_displaced`.
- Five binary tests changed because a main-sourced add now prints the copy's summary line and copies the ignored state
  when no
  snapshot serves it (`binary_worktree_ladder_tests.rs`, `binary_worktree_clone_tests.rs`,
  `binary_worktree_snapshot_tests.rs`).
- Not in the ladder's files but touching its seams: `entry.rs`, `worktree_copy_forward.rs`,
  `worktree_copy_registry.rs` and
  `worktree_copy.rs` (the main checkout is an applicable source for every creating command; the named-paths plumbing
  is gone,
  and `observe_copy_source_for` with its `serves_add` flag was folded into `observe_copy_source`).
- The ladder's rule "recovery never deletes what it cannot prove it created" is kept: the trash directory is made by
  `prune_snapshot` with a name of forty hexadecimal digits, the handler verifies the name, a real directory at the one
  place
  and a `.git` file in the worktree, and the conversion removes only the directory whose device and inode its journal
  recorded.
  A stage that a crash leaves with a trash inside is the stage the ladder's recovery already owns; this module adds
  nothing it
  would have to delete.

### Measured on a fixture

The fixture is a main-like subvolume with a copy of main's `.git`, 693 thousand ignored entries, and four ordinary and four
subvolume nested worktrees of 22.3 thousand entries each; release build, scope `CPUQuota=800%`, load average 6 to 10.

- Conversion, real binary, `--dry-run --all` over eight worktrees: 0.86 s. The real `--all`: 16.65 s for the four
  ordinary
  worktrees, 4.2 s each (the copy, the proving pass and the old-tree removal); about 200 s for the 48 on the real
  checkout, run
  once.
- A main-sourced add with four ordinary nested worktrees present: the snapshot rung served it; 5.7 s to the end of the
  leave-out,
  then, with no route to the service, 31 s of removal in the foreground (37.7 s in all).
- The same add after the conversion: 3.4 s from the command to the point where the trash is handed over; the removal,
  which
  the service does in the background (13.9 s measured here as the foreground fallback), is no longer in the command.
  The new worktree: `git status` 0.06 s and clean, `node_modules`, `target/` and `.cache/` present,
  `node_modules/.cache`,
  `.monochromatic` and `target/debug/incremental` absent, `.claude/worktrees` empty.
- Limit: the hand-over to the real service was controlled with an injected route; the end-to-end runs had no route,
  because
  a service is looked up by the new worktree's own Git directory and the fixture has no session bus. In production the
  route
  finder returns the managed address and the wake starts the service.

### Guards shown red

Each change was made after committing the tests and then restored: the exchange not reconciled with the journal
(`abandon_undoes_before_the_swap_and_refuses_after`), the old tree removed without the identity check
(`a_directory_that_is_not_the_recorded_old_tree_is_never_removed`), no proving pass
(`a_change_during_the_copy_is_caught_before_the_swap`), the copied `.git` pointer not unlinked and tracked directories
not
protected (`btrfs_a_snapshot_of_main_sets_aside_exactly_what_the_rule_leaves_out`), the ordinary nested worktree
blocking the
main checkout again (two tests), and the handler following a link at the trash name
(`the_handler_empties_only_the_trash_it_is_told_about`).

### Order for the owner

1. Merge this branch (it contains the ladder and the copy) after the ladder's recovery rework lands; release.
2. Stop sessions inside the nested worktrees; run `git cli-git convert-worktree-to-subvolume --dry-run --all` from the
   main
   checkout and read the skip reasons; then run it without `--dry-run`, worktree by worktree or with `--all`.
3. Apply the shared pnpm store (section "Applying the shared pnpm store").

### Round 2: merge of main, the snapshot message, and the earlier-release service

- `main` was merged into the branch with `git merge` (it holds the ladder head `3681539d6b` as `c0d7bf8bf3`, and later
  fixes). The only conflict, in `doc/handover/cli-git-rust-implementation.md`, was resolved by keeping both sides of each
  hunk. The ladder merge before it had one compile fallout: `still_held` in `service_work.rs` now takes the work
  directory, and the drain loop passes it. The trash handler and the conversion use `remove_tree` and
  `remove_if_identical`, which remain; the rule "delete only what this process proves it created" is unchanged.
- A main-sourced add that took the snapshot printed "copied 0", because no copy ran. It now prints nothing from the copy
  and, unless `--quiet`, one line on standard error (`MAIN_SNAPSHOT_NOTE` in `worktree_trash.rs`) that says the worktree
  was made as a snapshot of the main checkout and that nested worktrees, caches and run records were left out.
  `a_snapshot_of_main_sets_aside_what_the_rule_leaves_out` asserts that standard error is empty under `--quiet`.
- Earlier-release service: `wake` goes through `reach_current`, which compares the announced protocol version and release
  and sends a retire to any other; it ends with `OtherRelease` after the start wait. The hand-over is therefore never made
  to a service that does not claim `trash` items. Two controls with a fixture service on an abstract socket:
  `a_service_of_an_earlier_release_that_will_not_retire_is_never_handed_trash` (it hears only retires, the trash is
  emptied in the foreground, no pending record is left) and
  `a_service_of_an_earlier_release_is_replaced_before_trash_is_handed_over` (the work message reaches the current release
  after the retire; the item waits for it). `dispose_pending_trash_with` takes the start wait so the first control needs
  400 ms, not the production ten seconds.
- Both controls were shown red by removing the release comparison from `reach_current_untimed` (a service of the current
  protocol but another release was then accepted); restoring it turned them green.

### Round 3: the conversion tool refused clean worktrees (0.5.18)

- The dry run on the real repository listed 28 worktrees as not quiet that were clean. The log of that run is
  `convert-dry-run.log` in the scratch directory.
- Sixteen were refused with "HEAD is not a plain ref to a commit". They were not detached: their `HEAD` names a branch that
  only `packed-refs` holds, and `packed-refs` of the repository has tags with an `@` in the name
  (`@monochromatic-dev/module-logger@0.4.1`), which is outside the plain alphabet of `repo_read_refs.rs`; `packed()` returned
  nothing for the whole file, so no ref resolved. A detached `HEAD` already converted (the existing conversion tests use one).
  A lookup now leaves out a well-formed line with an unusual name; the list and a damaged line still end the read.
- Fifteen were refused as racily clean and two for a stale stat. `worktree_clean_content.rs` settles those by content in
  process: blob hash of the file, a link by its target, compared with the index entry's object name. The reasons that stay
  refusals: a `filter`, `ident` or `working-tree-encoding` attribute that may apply (attribute sources: tracked
  `.gitattributes` down the path, `info/attributes`, `core.attributesFile` or the default global file, the system file, macros
  that set one of them), a carriage return in the file (a line-ending conversion strips it on the way into the index; a file
  without one is unchanged by `text`, `eol`, `core.autocrlf` and `core.eol`, so those need no attribute lookup), a file over
  32 MiB, and a type, mode or assume-unchanged difference.
- The real repository's `* text=auto eol=lf` therefore refuses nothing by itself, and `*.png filter=lfs` refuses only a stale
  png.
- Only the conversion uses it (`check_clean_tracked_by_content`); the snapshot and clone rungs keep the stat-only check
  because they install the index they proved. After a conversion an entry that was not proven keeps its old stat data
  (`refresh_index_file_with` rewrites only entries whose old file matched), so Git hashes it at its first status.
- `Platform::config_scope` gives the conversion the configuration files Git reads; a stand-in names none, so controls read
  only the repository's own configuration. When the system configuration path is unknown (no forward target in the
  environment) the global attributes file cannot be ruled out and the content decision refuses with that reason.
- Dry run of the fixed build on the real repository: 160 worktrees, 94 already subvolumes, 65 would convert, 1 skipped
  (`package/forbidden-strings/core/Cargo.lock` really differs from the committed content).

### Round 4: the cross-vendor reviews of the content proof and the conversion

Every finding was checked against the Git source of the installed version (`attr.c` at `v2.56.0`) before it was fixed.

- System attributes (`attr.c`, `git_attr_system_file`): `system_path(ETC_GITATTRIBUTES)`, independent of
  `GIT_CONFIG_SYSTEM`, off under `GIT_ATTR_NOSYSTEM`. The path is now asked of the executable with `git var GIT_ATTR_SYSTEM`
  (cached per executable identity beside the system configuration path) and an unknown place refuses.
- Lexing (`attr.c`, `parse_attr_line`): the blanks are `" \t\r\n"` only, a line is a C string (NUL ends it), and
  `read_attr_from_file` skips a UTF-8 byte order mark on its first line; none of this was modelled. Paths, patterns and
  directory keys are bytes now, with ASCII folding. A quoted pattern keeps all its tokens and counts as applying.
- Reads: `worktree_clean_content.rs` opens every file and every `.gitattributes` below a held root with `openat2` and no link
  followed, judges type and mode on the handle, compares the state before and after the read, and bounds the read at the limit
  plus one byte. A `.gitattributes` that is a link is a refusal (Git opens it without following).
- Conversion: the journal is created and replaced only through exclusive files, records the identity of the stage, and is
  guarded by a `flock` per administrative directory; `Synced` is provisional; the old tree is removed beneath a pinned
  descriptor; nested registrations are read from the common directory; refusals have kinds; a failed refresh keeps the old tree.
- What the cleanliness proof is for: the conversion copies the whole tree, untracked and ignored files included, so a modified
  tracked file would survive the conversion. The proof is a policy (do not rebuild a worktree someone is in the middle of), and
  the data is protected by the copy, the second pass, the identity checks, and the update-only pass before the removal.
- What stays open: a write that keeps the size and modification time of the copy is invisible to the second pass and to the
  update-only pass; a process that writes and exits in the window before the exchange is found only by the update-only pass; a
  system attributes file under a prefix other than the executable's is not read.

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

## Snapshotting a dirty main checkout (proposal, not built)

Status: proposal for the owner's decision, 2026-11-03. No behavior changed for it; the wrapper still refuses to
snapshot a
source it cannot prove clean, and since this branch it says why (see "Telling why an add was not made as a snapshot" in
`package/git-policy/cli/SPEC.md`).

### The question

Two adds from the main checkout took Git's checkout and the ignored-state copy, for minutes, instead of the snapshot,
for
seconds, because the clean check refused the main checkout.
The main checkout is where the owner works, so it is often not clean: this machine commits roughly once a minute, and a
file edited a moment ago, an untracked note, or the `index.lock` of a commit in flight each refuse the snapshot.
Can a dirty main checkout still be snapshotted, with the new worktree made equal to what Git itself would produce?

### What Git itself produces

- `git worktree add -b topic <path> main` checks out the tree of the commit `main` names: `HEAD` is that commit, the
  index equals its tree, every tracked file has the committed bytes and mode, and there is no untracked file.
  Nothing uncommitted in the creating worktree is ever carried, staged or not.
- Ignored files are absent from Git's result. The wrapper adds the ignored state afterwards, and a snapshot carries it.
  The restore below must therefore remove untracked files that are not ignored and keep every ignored one.
- A requested commit-ish other than the source's `HEAD` is handled by the step the snapshot rung already has (the tree
  difference between the source's commit and the new commit).

### The candidate restore

1. Take the snapshot as today, then keep the source's own index from the snapshot (`.git/index` inside it, read before
   `.git` is replaced by the new worktree's pointer), not the index bytes the clean check read earlier.
   The snapshot is one instant, so the index and the files in it agree even if the person keeps working in the source.
2. Instead of `git read-tree -m -u <source commit> <new commit>`, run `git read-tree --reset -u <new commit>` under
   the new
   worktree's Git directory and work tree, which makes the index and every tracked file equal to the new commit's tree,
   whatever the source's index and files held.
3. Run `git clean -f -d` there, without `-x`, which removes untracked files that are not ignored and empty untracked
   directories, and keeps ignored files.
4. Prove the result (below); on any doubt destroy the stage and take the slow route, as every failure of the rung does
   today.

Why step 2 can work from the source's index: in a btrfs snapshot the inode number, the modification time and the
change time
of every file equal the source's, and only the device number differs, which Git's index does not compare.
Measured on a throwaway subvolume of 3000 tracked files with a modified file, a file staged then modified again, a
staged
deletion, an unstaged deletion, a staged rename, a mode change, an untracked file and an untracked directory:
`git status`
in the snapshot printed the same eight lines as in the source, 5 ms, with no refresh.
So the stat data in the copied index still marks exactly the files that were edited, and `read-tree --reset -u` rewrites
exactly those and leaves the rest.
The wrapper's current snapshot path installs the source's index "with refreshed stat fields" (`README.md`, ladder rung
1;
`rewrite_index_stat_with` in `SPEC.md`), which for a clean source is right and for a dirty one would record the edited
files as unchanged and hide the edits, so this option must not reuse that step.

### Measured

- Fixture one (3000 files, the eight changes above): after `git reset --hard` and `git clean -fd` in the snapshot, 12
  ms,
  `diff -r --exclude=.git --exclude=ignored` against a worktree made by `git worktree add` showed no difference, the
  executable bit was restored, `ignored/pkg/x` stayed, `git status` was empty.
- Fixture two (500 files, two commits, the same changes plus an edit over a file the second commit added): after
  `git read-tree --reset -u <first commit>` and `git clean -fd` in the snapshot, `diff -r` against a detached worktree
  at
  the first commit showed no difference, and `git write-tree` printed the same tree as that worktree.
- Wall time of that step was 1.03 s with 0.00 s of CPU. The cause is in the trace: the user's global configuration has
  `core.fsmonitor=true`, so the first Git command in a new snapshot started `git fsmonitor--daemon` and waited for it.
  The wrapper's own Git runs do not pin it today: `src/native/hooks_off.rs` says `core.fsmonitor` "is left as configured".
  So a Git run of the shipped snapshot rung in a new worktree can pay the same start and leave a daemon behind (my fixtures
  left one per snapshot until I stopped them); I did not measure that on the shipped rung.
  This step should pin `core.fsmonitor=false`.
- On the real main checkout, read only: `git --no-optional-locks status --porcelain=v1 -z --untracked-files=normal` took
  26 ms and `git clean -n -d` 57 ms, with 20445 tracked files.
  So the dirty-set discovery is cheap here; the cost that remains is rewriting the changed files.
- Not measured: the whole step on 20445 files with a realistic number of changes, and any run of the real ladder with
  the
  restore (it does not exist).

### Correctness hazards and what the candidate does with each

- Staged versus unstaged changes, and a file that is both: the index is replaced wholesale, so every combination ends
  equal
  to the new tree. Measured with a file staged then modified again.
- Deleted files, staged or not: the entry is in the new tree, so it is written back. Measured for both.
- Renames: Git stores a delete and an add, so the new name is removed (it is not in the new tree) and the old one
  restored.
  Measured with a staged rename.
- Untracked files: removed by `git clean`; the ignored ones stay. Measured. An untracked directory that holds only
  ignored
  files stays too.
- Empty untracked directories: `git clean -d` removes them. The real main checkout holds ten today (`git clean -n -d`
  lists them and `git status` does not), and the clean snapshot of today carries them although Git's checkout would not
  make them. This is an existing difference of the snapshot rung, not a cost of the proposal; the restore would fix it.
- Mode changes: restored while `core.fileMode` is on. With it off Git cannot see a mode change in the source either, so
  the new worktree would keep it; the proof below does not catch that.
- Files that Git cannot see as changed: assume-unchanged, skip-worktree, intent-to-add and unmerged entries, a sparse
  checkout, an operation in progress, a submodule. These stay refusals. Today the clean check folds them with ordinary
  changes, so the check in `worktree_clean.rs` must be split into hard refusals and the changes the restore handles.
- Racy timestamps: a file written in the second the index was written is treated by Git as possibly modified, so the
  restore rewrites it; the cost is a rewrite and the result is right.
- Smudge and clean filters, end-of-line conversion, Git LFS: files the restore rewrites go through Git's checkout, so
  the
  filters run as for any checkout. Not tested.
- Untracked nested repositories: `git clean -f -d` skips them (it needs a second `-f`). The main-checkout snapshot
  already
  sets aside every directory that holds a `.git` file, so none reaches the restore; if that rule changes, this becomes
  a hole.
- The person keeps working while the add runs: nothing in the restore reads the source after the snapshot, so a later
  edit
  cannot reach the stage; an edit between the clean check and the snapshot is why the index must come from the snapshot.
- Hooks: none run, as on every rung.

### How the restore is proven complete

The proof runs in the stage before the swap, from the stage alone:

1. `git write-tree` under the new Git directory prints the new commit's tree, so the index is exact.
2. `git ls-files --others --exclude-standard` prints nothing, so no untracked file that Git shows is left.
3. `git diff-files --quiet` after a refresh of the stat data prints nothing, so every tracked file equals its index
   entry
   as far as Git can tell. This is what the clean check already relies on for a clean source.
4. For the owner's confidence, not in the add: a guard test per hazard above that builds the state in a fixture, adds a
   worktree through the wrapper and through real Git, and compares the trees byte by byte (the way the two measured
   fixtures were compared). Each guard is shown red by removing the `git clean` step or the `--reset`.

What the proof cannot show: an edit that kept the size and both timestamps. Git's own `status` in the source cannot
show it
either, so "what Git itself would produce" has no sharper definition there. A full proof would hash every tracked
file, which
reads the whole tree and gives up most of what the snapshot saves; it can stay an option behind a flag.

### Cost

- Added to a clean snapshot add: one `git read-tree --reset -u` instead of `-m -u`, one `git clean -f -d`, three proof
  commands, and the rewrite of the changed files. For a handful of changed files this is milliseconds plus the process
  starts, against minutes for the slow route.
- Unchanged: the snapshot (0.15 s measured in "Snapshot route measured"), the leave-out of nested worktrees and caches
  (the cost that dominates today; my own add on 2026-11-02, a snapshot, spent from 23:38:50 to 23:43:31 emptying the
  trash in the foreground
  after the debug line "could not be handed to the service; emptying it here", a separate problem recorded in the
  report that
  came with this proposal).
- Removed for the dirty case: the slow route, about 190 thousand ignored entries copied after Git's checkout.

### Options

- A, keep refusing and say why (built on this branch).
  Pros: no new risk; the add is correct whatever the source holds; the person now knows the cause and the remedy.
  Cons: every add from a dirty main checkout is the slow route, minutes instead of seconds; the main checkout is dirty
  often.
- B, snapshot anyway and restore with Git's own `read-tree --reset -u` and `clean`, proven in the stage, falling to A
  on any doubt.
  Pros: seconds in the common case; Git does the semantics (filters, modes, deletions); nothing in the person's
  checkout is
  touched; it is additive to A, so a failure costs only the slow route.
  Cons: new code in the ladder's target step and a split of the clean check, both in files others are editing; the proof
  has a stated blind spot; the fsmonitor behavior must be pinned; it carries untracked ignored state of a dirty tree by
  design (that is the point).
- C, snapshot and restore by the wrapper's own file operations (compute the dirty set, rewrite and delete those paths).
  Pros: no dependence on Git's reset semantics.
  Cons: reimplements checkout semantics (filters, end-of-line conversion, LFS, modes, symbolic links, directory and file
  swaps) that Git already has and the wrapper otherwise never reimplements; more hazards, more tests.
- D, a clean spare maintained by the service: a snapshot of a clean state of the main checkout kept ready, taken
  whenever
  the main checkout is clean (or built from the default branch in a scratch subvolume), and used as the source for adds.
  Pros: never reads the person's dirty tree; adds are always the clean-snapshot path.
  Cons: a long-lived component and a refresh policy; the spare's ignored state (`node_modules`, builds) is as old as its
  last refresh, which breaks "immediately functional"; storage for the spare; the owner prefers the operating system to
  start things on demand.
- E, commit or stash the person's changes before the add and put them back after.
  Pros: the source is clean by construction.
  Cons: it rewrites the person's work and history; an interrupted add leaves their changes stashed; rejected by the rule
  that the wrapper never changes work it did not make.

Ranking: B > A > D > C > E.

- B over A: the same correctness as A in the cases B accepts and seconds instead of minutes; B is also additive, since
  every
  doubt falls to A, so choosing B loses nothing A gives.
- A over D: A is built and correct, while D adds a refresh policy and a component, and a stale spare returns a worktree
  whose `node_modules` and builds are out of date, which is the thing the owner wants avoided.
- D over C: D reuses the proven clean-snapshot path unchanged; C writes a second implementation of Git's checkout whose
  mistakes would be silent differences in the new worktree.
- C over E: C leaves the person's checkout alone; E modifies the user's work, which no rung may do.

### Decision for the owner

Whether to build B. If yes, the work is: split the cleanliness verdict in `worktree_clean.rs` into refusals that stay
and
changes the restore handles; take the index from the snapshot; add the reset, clean and proof steps to the target step
in
`worktree_ladder_target.rs` with the guards listed above; pin `core.fsmonitor=false` for those Git runs.
Two files named here are being edited by other agents at the time of writing, so this proposal touches neither.

## Review requests

- The stamp argument under "What FICLONE guarantees and how a concurrent writer is detected", and the 20 ms settled
  rule.
- The unreadable-entry policy (skip and say so, for every source).
- Whether `CopySelection::Paths` and `read_named_ignored_roots` should now be deleted: nothing in the release path
  builds
  them any more; they stay because their tests and the binary tests use them.
- Whether the linked-worktree copy should adopt the cache rule as well; the owner's words named only the main checkout.
