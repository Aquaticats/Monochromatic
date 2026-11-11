# Git 2.56 `git gc --prune=now` in one worktree takes a commit that only another worktree's per-worktree reference names

## Symptom

A commit is named by nothing but a per-worktree reference (`refs/worktree/...`) of a linked worktree,
and no reflog,
index or branch names it.
A `git gc --prune=now` run **in another worktree of the same repository** then deletes the object.
Nothing is printed,
and no exit status changes:
`gc` exits 0.

```text
$ git -C repo for-each-ref --contains 9905018f2c9b --format='%(refname)'
$ git -C linked for-each-ref --contains 9905018f2c9b --format='%(refname)'
refs/worktree/cli-git-service/7
$ git -C repo gc --prune=now
$ git -C repo cat-file -e 9905018f2c9b; echo $?
128
```

The cli-git wrapper met this in round 133 of its reviews:
a commit held by a reference lock lives only on `refs/worktree/cli-git-service/<batch>` of its worktree,
and a worktree removal or a maintenance run from elsewhere took it
(`package/git-policy/cli/src/native/worktree/worktree_removal_held_commits.rs`).

## Root cause

In Git 2.56 the chain is four calls long,
and the roots are collected at the end of it.

`git gc` runs its object-database maintenance with the incremental strategy and the prune expiry of
`gc.pruneExpire` (default `2.weeks.ago`),
which `--prune=now` replaces
([`builtin/gc.c:718`](https://github.com/git/git/blob/1891de46f79dc444a47db1c87bd0c8c76b5463f2/builtin/gc.c#L718-L730)):

```c
static int maintenance_task_odb(struct maintenance_run_opts *opts,
				struct gc_config *cfg,
				int keep_largest_pack,
				int aggressive)
```

`odb_optimize`
([`odb.c:745`](https://github.com/git/git/blob/1891de46f79dc444a47db1c87bd0c8c76b5463f2/odb.c#L745-L749))
hands that to the files backend,
which repacks and then runs a `git prune` child with that expiry
([`odb/source-files.c:1056`](https://github.com/git/git/blob/1891de46f79dc444a47db1c87bd0c8c76b5463f2/odb/source-files.c#L1056-L1070)):

```c
int odb_optimize(struct object_database *odb,
		 const struct odb_optimize_options *opts)
{
	return odb_source_optimize(odb->source, opts);
}
```

```c
	if (opts->strategy != ODB_OPTIMIZE_GEOMETRIC && opts->prune_expire) {
		struct child_process prune_cmd = CHILD_PROCESS_INIT;

		strvec_pushl(&prune_cmd.args, "prune", "--expire", NULL);
		/* run `git prune` even if using cruft packs */
		strvec_push(&prune_cmd.args, opts->prune_expire);
```

`git prune` builds its set of roots in `mark_reachable_objects`
([`reachable.c:302`](https://github.com/git/git/blob/1891de46f79dc444a47db1c87bd0c8c76b5463f2/reachable.c#L302-L346)):

```c
	/* Add all external refs */
	refs_for_each_ref(get_main_ref_store(the_repository), add_one_ref,
			  revs);

	/* detached HEAD is not included in the list above */
	refs_head_ref(get_main_ref_store(the_repository), add_one_ref, revs);
	other_head_refs(the_repository, add_one_ref, revs);
```

`refs_for_each_ref` and `refs_head_ref` read the **worktree the command runs in**,
so that worktree's own `refs/worktree/...` is a root there and nowhere else.
Every other worktree contributes exactly one thing,
its `HEAD`
([`worktree.c:602`](https://github.com/git/git/blob/1891de46f79dc444a47db1c87bd0c8c76b5463f2/worktree.c#L602-L632)):

```c
	worktrees = get_worktrees(repo);
	for (p = worktrees; *p; p++) {
		struct worktree *wt = *p;
		struct object_id oid;
		int flag;

		if (wt->is_current)
			continue;

		strbuf_reset(&refname);
		strbuf_worktree_ref(wt, &refname, "HEAD");
```

`other_head_refs` walks `HEAD` and nothing else,
so a linked worktree's `refs/bisect`,
`refs/worktree` and `refs/rewritten` are not prune roots
for a `gc` started in another worktree.
`git-worktree(1)` says those three prefixes are per-worktree and reachable from elsewhere
as `worktrees/<id>/refs/worktree/...`,
but neither that page nor `git-gc(1)` promises that a per-worktree reference keeps its objects
when maintenance runs in another worktree.

An earlier reading of this node was wrong:
the wrapper's held commits were assumed to be safe in a live worktree because
`git for-each-ref --contains` in that worktree lists them.
Listed from the right directory,
yes;
counted as a prune root from another one,
no.
The first catalog entry below is the measurement that disproved it.
That reading stands in `doc/decision/cli-git-concurrent-commits.md` lines 563 to 565
("While the scratch ref exists the new objects are reachable,
so `git prune` and `git gc --prune=now` keep them"),
with a check of 2026-11-04 that a linked worktree was present for;
its separate limit at lines 567 to 569 covers maintenance started *during* a landing,
not a commit that a lock keeps held after one,
so the record's claim needs a note naming this measurement
when its owner next edits it.

## Verification

Under test:
Git 2.56.0 on the development host (`git --version`),
with git/git at
`1891de46f79dc444a47db1c87bd0c8c76b5463f2`.

A runnable harness,
run in a directory you do not mind filling
(`env PATH=/usr/bin:/bin sh gc-per-worktree-probe.sh`,
because this repository's wrapper shadows `git` on a developer's `PATH`
and refuses a commit without a pathspec):

```sh
#!/bin/sh
# What Git's own maintenance does with a commit that only a linked worktree's per-worktree
# reference names.  Run it in a directory you do not mind filling: it makes its own repository,
# and run it with real Git first on PATH (`env PATH=/usr/bin:/bin sh gc-per-worktree-probe.sh`):
# this repository's wrapper shadows `git` and refuses a commit without a pathspec.
set -eu
root=$PWD/gc-per-worktree-probe
rm -rf "$root"
mkdir -p "$root/home"
export HOME="$root/home"
export GIT_CONFIG_NOSYSTEM=1
export GIT_CONFIG_GLOBAL=/nonexistent-global-config
export GIT_AUTHOR_NAME=probe GIT_AUTHOR_EMAIL=probe@example.invalid
export GIT_COMMITTER_NAME=probe GIT_COMMITTER_EMAIL=probe@example.invalid

git init --quiet --initial-branch=main "$root/repo"
cd "$root/repo"
echo base > base.txt
git add -- base.txt
git commit --quiet --message=base
base=$(git rev-parse HEAD)

git worktree add --quiet -b feature "$root/linked"
echo held > "$root/linked/edited.txt"
git -C "$root/linked" add -- edited.txt
git -C "$root/linked" commit --quiet --message=held
held=$(git -C "$root/linked" rev-parse HEAD)

# Nothing but the linked worktree's own reference names the commit.
git -C "$root/linked" reset --hard --quiet "$base"
git reflog expire --expire=now --all
git -C "$root/linked" update-ref --no-deref refs/worktree/cli-git-service/7 "$held"

git gc --prune=now
if git cat-file -e "$held" 2>/dev/null; then
  echo "kept: $held"
else
  echo "taken: $held"
fi
```

Its output on the version above:

```text
taken: c61fe33673249bb9ccf78146b3b46d070a0d40c6
```

### Catalogs

Each line is one run of the harness's fixture with the maintenance command named,
followed by `git cat-file -e <held>` in the main checkout.

Patterns that keep the commit:

- `git gc --prune=now` in the linked worktree itself:
  `status 0, object exists true`.
- `git gc` in the main checkout,
  with `gc.pruneExpire` at its default of two weeks:
  `status 0, object exists true` (the object is young,
  and the default expiry keeps it).
- `git gc --prune=now` in the main checkout after `git update-ref refs/cli-git/kept/7 <held>`,
  a reference of the common Git directory:
  `status 0, object exists true`.

Patterns that take it:

- `git gc --prune=now` in the main checkout:
  `status 0, object exists false`.
- `git repack -a -d`,
  then `git prune --expire=now` in the main checkout:
  `repack -a -d: status 0`,
  then `status 0, object exists false`.

## Verified workarounds

Both are what a program that keeps a commit should do at its own boundary,
not edits to Git.

### Keep the commit on a reference of the common Git directory first

```sh
git -C repo update-ref --no-deref refs/cli-git/kept/7 "$held"
```

Takes:
one `update-ref`,
and the object stays reachable through every maintenance run in every worktree.
Tradeoffs:
the reference is durable until someone removes it,
so a commit kept for a night is still there next year;
and the name must not replace a reference another commit was kept under,
which is why the wrapper writes it create-only (`update-ref <ref> <new> ""`).

### Run the maintenance in the worktree that holds the reference

`git -C linked gc --prune=now` keeps the commit,
because that run's `refs_for_each_ref` covers `linked`'s own per-worktree refs.
Tradeoffs:
it only works while that worktree exists at that path,
so it cannot be used by a program in another checkout,
and it does nothing for a commit whose worktree is already gone.

## What does not work

- Relying on a live linked worktree to keep its own held commits reachable from the main checkout.
  Measured above:
  `gc --prune=now` there takes the object while the worktree is still on disk.
- Relying on the default `gc.pruneExpire` as a safety net.
  It holds for two weeks,
  and `git gc --prune=now`,
  which clean-up jobs run,
  overrides it.
- Expecting Git to warn.
  A `gc --prune=now` that takes such a commit prints nothing and exits 0.
- Reaching for `git worktree remove` and then maintenance:
  the removal deletes the administrative directory,
  and with it the per-worktree reference that was the only holder.

## Where the wrapper stands

`git worktree remove` and `git worktree prune` are covered:
before Git runs,
the commit of every held batch of every linked worktree is kept on `refs/cli-git/kept/<batch>`
of the common Git directory,
on `refs/cli-git/kept/<batch>-rescue` when that name cannot be written,
and the command stops with status 2 when neither can be
(`package/git-policy/cli/SPEC.md`,
"Cheap worktree creation and removal on btrfs,
XFS and overlayfs";
commit `93ef6b2438`;
controls in `src/native/binary/binary_worktree_removal_held_commit_tests.rs`).
A forwarded `git gc --prune=now`,
`git prune --expire=now` or `git repack -a -d`
run from another checkout is **not** covered:
the wrapper forwards it,
and until a removal keeps a held commit on a reference of the common Git directory,
the only reference it has is its worktree's per-worktree one,
which that maintenance does not count (first catalog entry).
Nothing in this round changed that path;
the two remedies,
for the owner to choose between,
are keeping each held batch on a common reference from the moment it is held,
and guarding those three commands the way removals are guarded.
`doc/decision/cli-git-concurrent-commits.md` lines 563 to 565 assumes the per-worktree reference protects
pending objects from `git prune` and `git gc --prune=now`;
that holds only for maintenance in the worktree that owns the reference.

## Upstream filing decision

Nothing was filed,
and none of the six constraints is left "not yet" without a reason.

1. **Is it really upstream's fault?**
   Behaviour,
   not wording:
   `other_head_refs` collects only `HEAD`,
   and nothing in `git-worktree(1)` or `git-gc(1)`
   promises that another worktree's per-worktree reference is a prune root.
   The omission is consistent,
   and the documentation never claims otherwise,
   so this is a design edge rather than a defect to report.
2. **Can upstream fix it?**
   Yes in principle:
   a walk of every worktree's per-worktree refs in `other_head_refs` would do it,
   at the cost of reading every registration on every prune.
3. **Are they supporting this use case?**
   No statement either way:
   no test,
   example or documentation covers
   "keep a commit alive on a per-worktree reference from another worktree's maintenance".
4. **Would the repo welcome our contribution?**
   Not established.
   git/git's `Documentation/SubmittingPatches` expects patches on the mailing list,
   from a real identity,
   with discussion before a feature,
   and the project is famously conservative about the reachability walk.
   A report from this session would need a mailing-list thread we cannot run here.
5. **Will they likely fix it?**
   No signal either way:
   no search of the list archive or the tracker was run,
   because by 1 and 3
   there is no claim of breakage to make.
   The GitHub tracker cannot answer that question at all:
   git/git has issues disabled
   (`gh repo view git/git --json hasIssuesEnabled` answered `false` on 2026-11-11),
   so `gh search issues --repo git/git "worktree"` and the same search for `refs/worktree`
   both answer `[]` by construction;
   the project's tracker is its mailing list,
   which this session did not search.
6. **Have we prototyped a minimal fix?**
   No.
   The consumer-side fix is the one this repository needed and it is landed in the wrapper
   (a common reference before any removal,
   create-only,
   with a name beside it when the first is locked).

Filing would put a real person's account in front of a conservative project over a behaviour
its documentation does not promise,
and the workaround costs one `update-ref`.
No draft is kept,
and future sessions should not file this without new evidence
that Git documents the guarantee it breaks.
