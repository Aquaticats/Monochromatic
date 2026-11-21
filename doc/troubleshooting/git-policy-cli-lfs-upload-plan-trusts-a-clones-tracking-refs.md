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

The first fix asked the destination for its branch heads and trusted every tracking ref only when it reported all of
them at the commits held here.
That was sound but all or nothing:
one tracking ref a teammate moved on,
or a merged branch left behind,
made every push read the whole pushed history and name every LFS object in it to the server.
The fix now keeps each tracking ref the destination reports at the commit held here,
drops only the others,
and stops the scan at every ref it kept;
`--all` stays for the `git-lfs` scan form,
whose own skip list matches a cached ref by branch name alone.
The question is also paid once:
a typed push's manual-push discovery already confirms the pushed refs with one `git ls-remote --refs <url>`,
that query now lists every branch head too,
and the discovery carries the answer to the upload plan,
so a typed push opens no connection for it.
A service push runs no discovery and asks the destination itself,
one connection more than `main` charged it,
which the same round measured at 205 ms,
355 ms and 1017 ms for the added one-way delays of 0 ms,
10 ms and 50 ms;
the project accepts that cost.
A typed push measured 0.998 to 1.005 of `main` per round at those delays,
inside the identical-binary control's spread in every cell,
with 3 connections on both binaries in every sample.

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
`lfs_destination_confirms_tracking_refs::tests::a_stale_tracking_ref_drops_only_itself`,
`lfs_upload_plan::tests::the_destination_itself_decides_whether_the_tracking_refs_are_evidence`
(which now also reads a destination that reports one of two tracking refs),
`lfs_upload_plan::tests::a_destination_already_read_answers_without_a_second_query`,
`lfs_object_plan::tests::a_stale_tracking_ref_does_not_drag_the_known_history_in`,
`lfs_upload_plan::tests::a_mirror_is_read_through_git_and_never_from_a_list_of_spellings`,
`lfs_upload_plan::tests::a_remote_with_several_urls_is_uploaded_to_each` and
`lfs_destination_confirms_tracking_refs::tests::a_listing_in_another_shape_is_no_listing`.

The binary control for the stale-ref cost is
`binary_lfs_server_tests::a_stale_tracking_ref_drops_only_itself`.
It passes at `main` (one `git-lfs` run naming the one object the push adds,
one batch request,
one object asked
about),
failed on the all-or-nothing fix (three objects named where the push added one),
and passes again with
per-ref trust.
On a 2000-commit fixture with one stale tracking ref,
the all-or-nothing fix sent 21 batch requests and named 2052 objects against `main`'s one and one,
at 3734 ms against 71 ms.
The per-ref rule measured again beside `main` in the round that made it,
8 rounds after 1 warmup with an identical-binary control in each round:
`main` median 69.657 ms and the wrapper median 69.079 ms,
branch over `main` per round 0.983 (0.903 to 1.132),
inside the control's spread,
whose later over earlier of one binary was 0.956 (0.887 to 1.069) and 0.988 (0.908 to 1.061),
with 1 batch request and 1 object named per push on both binaries.
The fresh fixtures measured 1.015 (0.977 to 1.046),
also inside the control.

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
The per-ref rule needs neither workaround for a ref the destination moved on:
the wrapper drops that ref from the places it stops at and keeps the rest,
so a stale tracking ref costs the ref and nothing else.

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
