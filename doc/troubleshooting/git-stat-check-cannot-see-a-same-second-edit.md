# Git's stat check cannot see an edit made in the same second as its index entry

## Symptom

A tracked file holds bytes nobody committed,
and every Git command the wrapper used to make a new worktree agrees with the index that
the file is unchanged:

- `git status --short` prints nothing,
- `git diff-files --quiet` exits 0,
- `git read-tree --reset -u HEAD` leaves the edited bytes in place,
- `git reset --hard` leaves them,
- `git checkout-index --force --all` leaves them,
- and `git worktree add` from an empty destination writes the committed bytes,
  because it has no file to skip.

The edit that does this keeps three things:
the file's length,
its modification time,
and the whole second of its change time.
Writing the file updates its change time,
but Git compares that time only to the second.

## Reproduction

Measured on this machine with Git 2.56.0 (`git version`),
on btrfs:

```sh
# In a throwaway repository, with its own HOME and GIT_CONFIG_GLOBAL=/dev/null.
git init --quiet --initial-branch=main
printf 'tracked content\n' > tracked.txt
git add -- tracked.txt
git commit --quiet --message=initial
# One whole second in the past, so the entry is not racy against the index file.
python3 - <<'PY'
import os, subprocess, time
environment = {"PATH": "/usr/bin:/bin", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null"}
fixed, path = 1_577_934_245, "tracked.txt"
now = time.time()
time.sleep(int(now) + 1 - now + 0.002)          # start of a whole second
os.utime(path, (fixed, fixed))
subprocess.run(["git", "add", "--", path], env=environment, check=True)
before = os.stat(path)                           # the stat the index records
open(path, "w").write("DIRTY!! content\n")       # same length as the committed bytes
os.utime(path, (before.st_atime, before.st_mtime))  # the modification time, put back
after = os.stat(path)
print("recorded change time", before.st_ctime_ns, "now", after.st_ctime_ns)
PY
git status --short                    # prints nothing
git diff-files --quiet; echo $?       # 0
git read-tree --reset -u HEAD
cat tracked.txt                       # DIRTY!! content, unchanged by the reset
git reset --hard
cat tracked.txt                       # DIRTY!! content again
git checkout-index --force --all
cat tracked.txt                       # DIRTY!! content again
```

The recorded and the present change time differ by nanoseconds inside one second
(`1794413832.417677585` against `1794413832.428677495` in the run that caught it).
`git ls-files --debug tracked.txt` shows the recorded value,
and `match_stat_data` in Git's `statinfo.c` compares the change time,
whose nanosecond comparison did not fire on this build:
only the seconds decided.

The same edit made a second later is seen:
the change time's seconds differ,
Git hashes the file,
`git status --short` prints ` M tracked.txt`,
and `git read-tree --reset -u HEAD` writes the committed bytes.
So the blindness is the whole second,
not the nanoseconds alone.

## What the wrapper does about it

The dirty-snapshot restore of `git worktree add`
(`package/git-policy/cli/src/native/worktree/worktree_dirty_restore.rs`)
removed nothing on its own and leaned on `git read-tree --reset -u` and a closing
`git diff-files --quiet`:
with this blindness both leave and accept the edited bytes,
and the new worktree then reported a file clean whose content nobody reviewed.

`worktree_clean_tree_walk.rs`'s `paths_whose_content_differs`
now names the tracked paths whose recorded stat data is not the file's
(by size,
inode,
modification time and change time to the nanosecond,
the comparison `matches_stored` makes)
and whose bytes are not the blob their entry names.
`restore_dirty` removes exactly those paths from the stage before the reset,
which then writes the commit's bytes into them,
as Git's own checkout from an empty destination does.
A file whose bytes cannot be compared at all,
through a conversion attribute for instance,
is removed as well:
that costs a write of the commit's content and never publishes other bytes.

The controls are
`worktree_dirty_restore_tests::a_restore_replaces_an_edit_that_keeps_its_files_size_and_times`
and
`worktree_snapshot_index_step_tests::the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit`,
with the fixture `worktree_ladder_test_support::hide_an_edit`.

## What no stat-based check can see

A filesystem whose change time has whole-second granularity,
or a file whose change time the superuser forges,
makes the same edit invisible to the wrapper's stricter comparison too.
Git's own `status` is blind to it there as well,
so the file is one the person's Git also reports clean;
only reading every tracked file would answer,
which an add from a checkout with no changes does not pay for.
