# The Git wrapper's `linked-worktree-only` guard rejected `git checkout <commit>` in a `--no-checkout` clone, which failed `libghostty-vt-sys` and the lint of `package/terminal/app`

Recorded 2026-11-07T13:40:00-05:00.

## Symptom

`mise run //package/terminal/app:lint` (host route) stopped in the build script of `libghostty-vt-sys`:

```text
Fetching ghostty 0b8b9984ee560456194262d190aa850224edb1f4 ...
Cloning into '<cargo build dir>/debug/build/libghostty-vt-sys/<hash>/out/ghostty-src'...
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward","policyId":"linked-worktree-only","severity":"error","code":"linked-worktree-only/linked-worktree-required","message":"monochromatic-git-policy-cli: git checkout <branch> is rejected because this worktree has uncommitted changes to tracked files: ..."}
thread 'main' panicked at .../crates/libghostty-vt-sys/build.rs:486:5:
git checkout ghostty commit failed with status exit status: 1
```

The network was not the cause:
the clone succeeded.

## Root cause

The build script of the `libghostty-rs` revision pinned in `rust-catalog.toml` clones Ghostty,
then checks a commit out
(`crates/libghostty-vt-sys/build.rs`,
function `fetch_ghostty`):

```rust
clone.arg("clone").arg("--config").arg("core.longpaths=true").arg("--filter=blob:none").arg("--no-checkout")
// ...
checkout.arg("checkout").arg(GHOSTTY_COMMIT).current_dir(&src_dir);
```

A clone made with `--no-checkout` has no index file.
`git status` reads a missing index as empty,
so every committed file shows as a staged deletion (`D  a.txt` in the probe below),
and the wrapper's dirty query (`git status --porcelain=v1 --untracked-files=no`,
`package/git-policy/cli/src/native/command/rule_worktree_dirty.rs`) answered "uncommitted tracked changes".
A form that only moves `HEAD` is rejected in any worktree while such changes exist (README,
"Discarding commands"),
so the checkout was refused.
The wrapper is on `PATH` ahead of the real Git on the host route (`.cache/monochromatic-git-policy-cli-bin/git`);
the container route has plain Git.

What changed on 2026-11-07:
the crates.io release `libghostty-vt-sys` 0.2.2 vendored Ghostty and ran no Git,
and the Zig 0.17.1 work pinned the git revision instead
(`doc/troubleshooting/ghostty-requirezig-rejects-zig-0-17.md`).

Git itself treats this state as an initial checkout,
with nothing of the user's to protect
(Git v2.56.0,
the installed version):

- `read-cache.c:2429-2432`:
  `is_index_unborn` is `!istate->cache_nr && !istate->timestamp.sec`.
- `builtin/checkout.c:829`:
  `topts->initial_checkout = is_index_unborn(the_repository->index);`.
- `unpack-trees.c:2978`:
  `if (oldtree && !o->initial_checkout)` is the rejection of a staged deletion that an initial checkout skips.

(The line numbers were read from the `v2.56.0` tag files;
the links were not loaded in a browser.)

## Verification

Probe without network,
in a scratch directory:

```bash
git clone --no-checkout src clone      # real Git
cd clone && git status --short         # prints: D  a.txt
git checkout <sha>                     # wrapper: rejected, code linked-worktree-only/linked-worktree-required
/usr/bin/git checkout <sha>            # real Git: HEAD is now at <sha>
```

Binary tests (`package/git-policy/cli/src/native/binary/binary_no_checkout_clone_tests.rs`),
run on the host before the fix:

- red:
  `a_commit_is_checked_out_in_a_clone_made_without_checkout` failed;
- controls,
  passing both before and after:
  `a_checkout_is_refused_while_a_tracked_file_is_modified` and
  `a_checkout_is_refused_while_a_deletion_is_staged_and_the_file_is_gone` (index file present).

After the fix:
all three pass (`3 tests run: 3 passed`);
`native:test:container` passes (`5199 tests run: 5199 passed (6 slow), 8 skipped`).

## Fix

`index_file_is_absent` in `rule_worktree_dirty.rs`,
called from `check_linked_worktree` in `policy/policy_checks.rs`
for the forms that only move `HEAD` (`git checkout <rev>`,
`git switch <branch>`).
The definition is as narrow as Git's:

- the Git directory exists and has no `index` file (an `index` file that exists,
  empty or not,
  never counts);
- `GIT_INDEX_FILE` is unset (a named index is judged by `git status`);
- `git checkout -f`,
  `git reset --hard`,
  `git restore` and `git clean` keep their rules:
  without an index they overwrite untracked files,
  so Git does not protect them either.

No path exemption,
configuration switch or detection of third-party directories was added.

## What does not work

- Stripping the wrapper from `PATH` for the cargo child,
  or choosing the container route when `git` resolves to the wrapper,
  were proposed for `package/terminal/app/mise.toml` and not applied:
  the first routes every command a build script runs past the guard.
  The second remains the fallback if the wrapper fix is not installed.
- An allowlist entry naming a cargo build directory as a fixed path,
  as for mise's data directory and uv's cache:
  it would hard-code a path that host configuration decides,
  while the defect of the fresh clone is the reading of a missing index.
  The environment-named form of that entry,
  `OUT_DIR`,
  is implemented;
  see "Amendment" below.
- Pre-fetching Ghostty as a tarball into `GHOSTTY_SOURCE_DIR`:
  Ghostty's `zig build` still fetches Zig packages from the network,
  and the commit would be a second copy of the constant in the pinned `build.rs`.

## Amendment 2026-11-10T17:10:28-05:00: the build step's own directory is an allowlisted tool cache

The fix above covers the fresh clone,
because that clone has no index file yet.
It does not cover the same clone once it holds an index and looks dirty,
which is the state a second build step,
an interrupted first one,
or a build script that edits what it vendored leaves.
There the wrapper still refused every guarded form,
including the `git checkout <commit>` of a vendoring build script,
with `linked-worktree-only/linked-worktree-required`.

Measured 2026-11-10 on the host with the wrapper built from `main`,
in a throwaway crate whose build script made the source repository and the clone
and then ran the two checkouts
(`OUT_DIR` was `/var/mnt/encrypted/cargo-build/f0/d1b89a91519410/debug/build/out-dir-probe/32314d0e1bec2cbd/out`):

- the build script's own step,
  `git clone --filter=blob:none --no-checkout <source> ghostty-src` then `git checkout <commit>` there:
  exit `0`;
- the same checkout after one edit in that clone:
  exit `1`,
  `code` `linked-worktree-only/linked-worktree-required`,
  message `monochromatic-git-policy-cli: git checkout <branch> is rejected because this worktree has uncommitted changes to tracked files: ...`.

The rule that applies is already in `package/git-policy/cli/SPEC.md`,
"Discarding commands":
"Repositories under an allowlisted tool cache are exempt.
The list is chosen by the invocation's own environment".
Cargo's build-script output directory is such a tool cache,
and Cargo names it in the environment of exactly the process that runs that Git:
`OUT_DIR` is set for every build script and passed on to what the script starts.
So the list gained a third entry read from `OUT_DIR`
(`cargo_build_script_output_dir` in `src/native/invocation/effective_target.rs`),
with the same notice on standard error that names the variable.

What that entry does not do:

- It names no path of this host,
  so `build.build-dir` in a person's `~/.cargo/config.toml` stays host configuration the wrapper never reads.
- It exempts the directory one build step was given and nothing wider:
  a clone beside it,
  and the person's own repositories and worktrees,
  keep every rule they had.
- A form that only moves `HEAD` never reaches the exemption branch,
  so no notice is printed for it;
  the command runs unguarded because such a form asks no measurement at all.
  The same is true today for uv's cache and mise's data directory.

### The rules this case could have taken, and what each would have done

The case:
a Git command whose repository is a clone inside a directory a tool made for itself.
Four rules were considered,
with the shipped one first.

1. An environment-named tool cache,
   which is what the SPEC rule above already says:
   the invocation's own environment names the directory,
   and the repository inside it is exempt.
   It reads no path of this host,
   it covers both Cargo layouts (the host's relocated build directory and the default `<workspace>/target`),
   and it narrows to one build step's own directory.
   Its trust is the environment,
   with the notice on standard error as the mitigation,
   exactly as uv's cache and mise's data directory already work.
2. A fixed path for this host's cargo build directory.
   Rejected by the earlier session and still wrong:
   host configuration decides that path,
   the wrapper would carry one machine's cache in its source,
   and a different `CARGO_BUILD_BUILD_DIR` would bring the refusal back.
3. Any repository nested inside a Git-ignored path of an enclosing repository.
   It would not have covered this case at all
   (this host's build directory is outside every repository),
   it costs an extra Git start to find the enclosing repository,
   and it exempts a clone a person made deliberately inside an ignored folder.
4. Any repository under a directory that carries a `CACHEDIR.TAG`.
   It needs no environment trust,
   but it adds a second reading of a tag file,
   and it exempts a whole tagged tree,
   including a person's own clone kept inside one,
   where rule 1 exempts the one directory a build step was given.

## Upstream filing decision

Not applicable:
the defect is in this repository's own wrapper,
and the fix is here.
