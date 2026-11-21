# monochromatic-git-policy-cli 0.6.1-alpha.44: a push after `git remote set-url`, or under `remote.<name>.mirror=2`, publishes Git LFS pointers without their objects and exits 0

Read at `main` = `8b67b95709728c87391eb728f3ba33f8f514794e` (2026-11-20).
The defect was recorded there as an open finding in
`doc/handover/cli-git-rust-implementation.md`
("a push after `git remote set-url`,
and under `remote.<name>.mirror=2`,
publishes pointers without their objects and exits 0").
This record diagnoses both endings and names the controls that now show them.

## Symptom

A repository with Git LFS keys,
a local bare remote and an LFS server,
pushed once:

```text
$ git push origin main
...
To .../origin.git
 * [new branch]      main -> main
$ git ls-tree -l origin main -- big.bin
100644 blob <pointer oid>  ...  big.bin
```

`git push` exits 0 and the remote holds the pointer file,
but the LFS server never received the object the pointer names,
so a clone cannot download `big.bin`.

Two triggers,
one ending:

- `git remote set-url origin <a second bare remote>` after the first fetch
  (or the same with `url.<base>.insteadOf` moved).
  The clone's `refs/remotes/origin/*` still describe the first server and its commits.
- `git config remote.origin.mirror 2` before a bare `git push origin`.
  Git reads `2` as true and performs a mirror push of every branch,
  but the wrapper planned an ordinary branch push,
  so only the current branch's objects were considered for upload.

Every push that adds an object the destination lacks can end this way when the clone's tracking refs describe
another server or another commit.

## Root cause

The upload plan decided what to skip from the configuration alone,
never from the destination.
At `main` = `8b67b95709`,
`package/git-policy/cli/src/native/lfs/lfs_upload_plan.rs:508` held:

```rust
let tracking_trusted: bool = configured && push_urls.is_empty() && !has_push_rewrite(all);
```

and `package/git-policy/cli/src/native/lfs/lfs_upload_plan_awaiting.rs:139` held the same line.
A configured remote with no push URL and no `pushInsteadOf` rewrite was trusted,
whatever `git remote set-url`,
a re-added remote or a moved `insteadOf` had done to its tracking refs.
`full_scan` was then false (`lfs_upload_plan.rs:509`),
so both the wrapper's own object walk
(`package/git-policy/cli/src/native/lfs/lfs_object_plan.rs:190-196`,
which stops at `refs/remotes/<remote>/`)
and `git-lfs`'s own scan treated the tracking refs as the destination's contents.
They are what the last fetch wrote down,
not what this push's server holds.

The mirror value had its own list of spellings,
`package/git-policy/cli/src/native/lfs/lfs_upload_settings.rs:361-369` at the same commit:

```rust
pub(crate) fn reads_as_true(value: &str) -> bool {
    for spelling in ["true", "yes", "on", "1", ""] {
        if value.eq_ignore_ascii_case(spelling) {
            return true;
        }
    }
    return false;
}
```

`lfs_upload_plan.rs:455` and `lfs_upload_plan_awaiting.rs:86` read the configured mirror through it.
Git's boolean grammar reads `2`,
`-1` and `0x10` as true and the empty value as false,
so the list disagreed in both directions:
a mirror Git performed was planned as an ordinary branch push,
and a value Git refuses was planned as a push.

A third case had the same cause:
`remote.<name>.url` may be written by Git
(`git push` writes to every URL when the remote names no `pushurl`),
and the plan read only `pushurls`,
so a remote with two URLs was uploaded to once.

## Verification

The two controls are in `package/git-policy/cli/src/native/binary/binary_lfs_server_tests.rs`:
`a_remote_whose_url_moved_uploads_its_objects_to_the_new_server` and
`a_numeric_mirror_uploads_the_objects_of_every_branch_it_publishes`.
Each runs the built wrapper against real Git,
a real bare remote and the inline LFS server,
and reads the server's upload list.

The red run,
at the commit that added only the controls (`2371ea5897`,
before any fix):

```text
FAIL lfs_server::a_remote_whose_url_moved_uploads_its_objects_to_the_new_server
  assertion `left == right` failed: the object reached the server the push went to:
   * [new branch]      main -> main
  left: 0
  right: 1
FAIL lfs_server::a_numeric_mirror_uploads_the_objects_of_every_branch_it_publishes
  the side branch's object was uploaded: ["<main's object>"]
   * [new branch]      main -> main
   * [new branch]      side -> side
Summary [1.033s] 2 tests run: 0 passed, 2 failed, 1253 skipped
```

Both show the ending of the symptom:
the ref is published and the destination's store holds no object for it.

The green run,
on the fix:

```text
PASS [1.250s] lfs_server::a_remote_whose_url_moved_uploads_its_objects_to_the_new_server
PASS [1.264s] lfs_server::a_numeric_mirror_uploads_the_objects_of_every_branch_it_publishes
Summary [1.265s] 2 tests run: 2 passed, 1253 skipped
```

The unit controls are
`lfs_destination_confirms_tracking_refs::tests::both_faces_read_every_answer_alike`
(every destination answer,
on both faces),
`lfs_upload_plan::tests::the_destination_itself_decides_whether_the_tracking_refs_are_evidence`,
`lfs_upload_plan::tests::a_mirror_is_read_through_git_and_never_from_a_list_of_spellings`,
`lfs_upload_plan::tests::a_remote_with_several_urls_is_uploaded_to_each` and
`lfs_destination_confirms_tracking_refs::tests::a_listing_in_another_shape_is_no_listing`.

## Verified workarounds

Before an upgrade,
upload every object the pushed refs reach and then push:

```text
git lfs push --all origin
git push origin main
```

The tradeoff is cost:
`--all` scans and asks about every object reachable from the refs,
not only the ones the push adds,
so a large repository uploads its whole LFS history.
The wrapper's own failure text prints this remedy,
and the full-scan path it names is the one `a_remote_with_a_push_url_keeps_the_full_scan` exercises.
The two commands above were not run as a pair in this investigation;
the control proves the full-scan path,
not the workaround's timing.

Deleting the remote's tracking refs (`git update-ref -d refs/remotes/origin/main`) also makes the
bounded push scan the whole ref,
because there is nothing left to skip.
Its tradeoff is that it discards a cache every later command reads;
the next fetch recreates it from whichever server is configured then.

## What does not work

- Reading `remote.<name>.mirror` from a list of spellings.
  The list has to reproduce Git's boolean grammar,
  and a value it reads wrongly turns a mirror into an ordinary push;
  the fix asks Git with `git config --type=bool --get`.
- Trusting `refs/remotes/<remote>/*` because the remote is configured and has no push URL.
  `git remote set-url` leaves those refs untouched,
  and a re-added remote of the same name inherits nothing.
- Reading a `for-each-ref` listing whose lines are split on whitespace.
  Every Git query in this wrapper trims the answer's ends,
  which eats an empty `%(symref)` field at either end;
  the format puts that field between two filled ones,
  and a line in any other shape is refused rather than read as a clone with no tracking refs.

## Upstream filing decision

Nothing to file upstream.
The defect is in this repository's upload plan,
the fix is here,
and the external programs behave as documented:
Git's boolean grammar,
`git remote set-url` leaving tracking refs alone,
and `git-lfs`'s bounded scan trusting the refs it is given.
