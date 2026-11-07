# mpm-shadow 0.2.6 `install` in a worktree made from a snapshot removes the source checkout's postinstall links and empties its `.cache/*-bin` folders

Recorded 2026-11-07T09:30:00-05:00.
The tool is `package/mpm-shadow/cli`, which this repository owns,
so there is no upstream to file against (see "Upstream filing decision").

## Symptom

In the main checkout, link folders under `.cache` were found empty:
`.cache/cli-git-bin`, `.cache/mpm-shadow-bin` and `.cache/cargo-shadow-bin` at 08:42 on 2026-11-07,
and `.cache/cargo-shadow-bin`, `.cache/monochromatic-git-policy-cli-bin`, `.cache/mpm-shadow-bin` and `.cache/pnpm-shadow-bin` at 09:08:16.
While they were empty, `command -v git` in main resolved to `~/.local/bin/git`,
and `mpm` and `cargo` lost their repository shims.
Nothing in main's `git status` changed, because `.cache` is ignored.
Nobody had run anything in main.

## Root cause

Two steps, each documented behavior of its own tool, combine.

### Step 1: a new worktree starts with the source's record

The git wrapper's `git worktree add` makes the new worktree as a btrfs snapshot of the source checkout,
ignored files included (`package/git-policy/cli/README.md`, the worktree ladder).
The root `.cache` is kept (`package/git-policy/cli/src/native/worktree_copying/worktree_copy_prune.rs`, nested caches are the exception),
so the new worktree holds main's `.cache/toolchain-launchers/.postinstall-links.json`.
That record stores absolute paths, as link path to link target:

```json
{
  "links": {
    "/var/home/user/Monochromatic/.cache/mpm-shadow-bin/mpm": "/var/home/user/Monochromatic/.cache/toolchain-launchers/mpm-shadow"
  }
}
```

The wrapper then runs `mpm-shadow install --frozen-lockfile` in the new worktree
(`package/git-policy/cli/src/native/repair/package_repair_toolchain.rs`, `repair_toolchain`),
because the copied launcher records name main's lock file.

### Step 2: `reconcile` trusts every recorded path

`package/mpm-shadow/cli/src/toolchain/postinstall_links.rs`, before the fix, `declared_link_paths` (line 106) resolves each declared link against the scope that runs:

```rust
declared.insert(link_directory(ctx.scope, &link.link_dir).join(&link.link_name));
```

In the new worktree that is the new worktree's path, so none of main's recorded paths is declared.
`reconcile` (line 150) then treats each of them as a link the lock dropped (line 161 is the `declared.contains(path)` test that fails),
`inspect` (line 125) finds main's link still pointing at the recorded target, which is main's own launcher, and answers `Held::OwnLink`,
and line 167 removes it:

```rust
Held::OwnLink => match fs::remove_file(path) {
```

The check "still the symlink the record names" cannot tell a link this scope made from a link the record merely describes,
because a snapshot makes both look identical.
The new worktree's own links are made in the same run, so its folders fill while main's empty.
Only links that are in the record and still point at the recorded launcher go; that is why a link of another target stays (see "Unsettled").

### Evidence that this, and not another step, did it

- At 09:07:45 another agent ran `git worktree add` for `fix/git-policy-cli-mask-verbose-git-output`
  (user journal: a `systemd-run` scope for that command, ended 09:08:28 after 43.5 s).
  Its link folders and its record were written at 09:08:16.050 to 09:08:16.053.
- Main's four link folders all have the modification time 09:08:16.052271939, and are empty,
  within the same few milliseconds in which that worktree's own link folders were filled.
  The user journal for 09:07:30 to 09:08:30 lists two started command scopes only (systemd-run scopes, so not every command):
  that add, and the diagnosing session's own add at 09:08:18, which ran with `--no-worktree-copy` after the folders were already empty.
- Main's own record still lists the four links, which no longer exist, so no install ran in main.
- The new worktree's record names only its own paths, which is what the fix keeps and the old code also produced,
  because `kept` is rebuilt from `made` and the retired entries are dropped.
- Same shape at 08:42: the fold agent ran `git worktree add` from main at 08:41:58
  (the add's timing log: start 1794058918465 ms, total 86.1 s), its commit service started 08:42:01,
  and its link folders were filled at 08:42:49.
- A copy of that situation in the test suite reproduces it (see "Verification").

### What was ruled out

- The cleanup of old copies (`old_copies.rs`, `old_copy_places.rs`) reaches every checkout's `.cache/*-bin` folders,
  but it re-points a link to the managed copy and never removes one
  (`plan_links` builds `Repoint` values; `old_copy_remove.rs` removes only the copy in the shared cargo bin directory).
  It cannot leave a folder empty.
- The wrapper's own fixups after the snapshot act on the new worktree's copy, never on main.
- No subagent ran `rm`, `rmdir` or an install in main between 08:38 and 08:46
  (their Bash commands in that window were reads, one `git worktree add`, and a wait on a test run).
- The recorded owner choice in `package_repair_toolchain.rs` ("Install online during the add", 2026-11-03)
  says an add can retire tool copies another checkout still links.
  That is about copies of programs; it says nothing about deleting another checkout's link files,
  and no file in `doc/decision/` was found that does (searched for "postinstall", "retire" and "record").
  This is a defect, not a recorded design.

## Verification

Version under test: `monochromatic-mpm-shadow` 0.2.6, branch `fix/mpm-shadow-cli-install-keeps-other-checkouts-links`,
red commit `7e4eb28785`.

Run from `package/mpm-shadow/cli` (the harness is the repository task):

```sh
# package/mpm-shadow/cli
mise run //package/mpm-shadow/cli:test -- postinstall an_install_in_a_copy
```

Before the fix the end-to-end test `an_install_in_a_copy_of_a_checkout_leaves_that_checkouts_links_in_place`
(`tests/copied_checkout_record.rs`) failed on its assertion that the source checkout's link is still a link:
`No such file or directory`.
It builds a disposable git repository with two worktrees, runs the built binary's `install --frozen-lockfile` in the source,
copies `.cache` into the second worktree with `cp --archive` (links stay links, as in a snapshot),
runs `install --frozen-lockfile` in the second, and reads the source's link.
After the fix it passes, with the unit tests in `src/toolchain/postinstall_links_tests.rs`.

Patterns that work cleanly:

- A link declared, then dropped from the plan, in the scope that made it: retired, one notice, none after.
- A link renamed or moved to another directory inside the checkout: the old one is retired.
- The global scope's link in `~/.local/bin`: retired when dropped.

Patterns that failed before the fix:

- A record copied from another checkout: that checkout's links were removed.
- A record naming a link inside a checkout nested in this one (a directory with a `.git` entry in between): removed.

## Fix

`reconcile` skips, without inspecting or removing, every recorded path that is not the scope's own
(`is_own_link` in `postinstall_links.rs`):
a repository scope owns paths inside its checkout root, resolved, and none inside a nested checkout.
The global scope owns every path its record names, because nothing copies that record.
Skipped paths are not written to the new record, so a copied record is replaced by the new worktree's own on the first install.

Tradeoff: a repository scope no longer retires a link whose plan `linkDir` is an absolute path outside the checkout.
The plan uses only relative `linkDir` values (`toolchain.plan.jsonc`).
A link of that kind is left where it is, so remove it by hand if the plan drops it.

## Workarounds until the fixed `mpm-shadow` is installed

The installed 0.2.6 still has the defect, and so does every `git worktree add` that runs its repair.

- Create a worktree with `git worktree add --no-worktree-copy`, then run `mise run toolchain:install` in it.
  There is no copied record, so nothing foreign is retired.
  The tradeoff is that the ignored state (`node_modules` and the like) is not copied.
- Restore a checkout whose folders are empty with `mise run toolchain:install` in that checkout.
  This repairs the links but does not stop the next add from emptying them again.

## What does not work

- Deleting the copied record from the new worktree after the add: the repair's install has already run.
- Making the wrapper skip `.postinstall-links.json` in the copy is possible, but the record is mpm-shadow's,
  and `reconcile` still acts on any record it reads, so a hand-copied record would do the same.
  The owner of the invariant is mpm-shadow.

## Unsettled

- The 08:42 report named three folders and not `pnpm-shadow-bin`, while 09:08:16 emptied all four.
  The cause for the difference was not measured:
  the `pnpm` link in main at 08:42 may have pointed elsewhere than its recorded launcher,
  which `inspect` reports as `Held::Foreign` (kept, warning only).
- Four `cargo-shadow-bin` folders under `.claude/worktrees` have been empty since 2026-11-04.
  Two have modification times (14:19:41 and 16:04:22) before the commit that introduced retirement
  (`5b7213a46a`, 2026-11-04T16:10:27-05:00), so this cause does not explain them;
  the one at 16:37:32 is consistent with it and was not checked further.

## Upstream filing decision

Not applicable: `mpm-shadow` is this repository's own package, so the defect is fixed in this repository.
No external issue, duplicate search or trust-system check applies.
