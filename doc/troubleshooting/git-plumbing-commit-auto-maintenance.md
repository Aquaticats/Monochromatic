# Git 2.55 plumbing commits never run automatic maintenance, so one pack per landing slows every later Git process

Note 2026-11-09T16:47:44-05:00:
these citations in this record name paths that no longer exist.
`package/git-policy/cli/src/policy-engine/commit-landing-objects.ts` left the tree on 2026-10-13 in
`642b6d72cb`.
`package/git-policy/cli/src/policy-engine/commit-landing-support.ts` left the tree on 2026-10-13 in
`642b6d72cb`.
`package/git-policy/cli/src/policy-engine/` is not in the tree;
its files were deleted over several commits,
most of them on 2026-10-13 in `642b6d72cb`.
`package/git-policy/cli/src/policy-engine/commit-landing-auto-maintenance.ts` left the tree on 2026-10-13 in
`642b6d72cb`.

## Symptom

Per-commit wall time through the cli-git wrapper grew with the number of commits the wrapper had already landed,
while direct `git commit` in the same repository stayed flat.
No error is emitted.

The concurrent-commit benchmark saw a serialized 8-commit batch go from 2.65 s to 4.1 s over about 576 commits,
and a host repository go from 465 ms to 588 ms per commit over 400 commits.
Reproduced on the host with the built wrapper
(`package/git-policy/cli` at `2bf18030e`,
Git 2.55.0,
Node 26.10.0),
300 serialized wrapper commits in one disposable repository:
medians per block of 50 commits were
356,
378,
395,
411,
434,
and 463 ms.

The repository then held one pack per wrapper commit:

```text
$ ls .git/objects/pack | grep --count '\.pack$'
300
```

## Root cause

A commit transaction lands through plumbing:
`index-pack --stdin --keep` migrates the commit's objects from the shadow store into the real store as one new pack,
and `update-ref` advances the branch
(`package/git-policy/cli/src/policy-engine/commit-landing-objects.ts`,
`package/git-policy/cli/src/policy-engine/commit-landing-support.ts`).
Neither plumbing command runs automatic maintenance.
Only porcelain does;
in Git v2.55.0 the callers of `run_auto_maintenance` are:

```text
builtin/am.c:1942:		run_auto_maintenance(the_repository, state->quiet);
builtin/commit.c:1965:	run_auto_maintenance(the_repository, quiet);
builtin/fetch.c:2885:		run_auto_maintenance(the_repository, verbosity < 0);
builtin/merge.c:509:			run_auto_maintenance(the_repository, verbosity < 0);
builtin/rebase.c:572:	run_auto_maintenance(the_repository,
```

The paths this record names under `package/git-policy/cli/src/policy-engine/` were removed on
2026-10-13 in `642b6d72cb` (refactor(*):
delete the TypeScript Git policy wrapper,
its tasks,
bins
and trust workflow);
no file in the working tree carries them now.

Native `git commit` runs it right after updating `HEAD` and before `post-commit`
(`builtin/commit.c:1964`):

```c
	repo_rerere(the_repository, 0);
	run_auto_maintenance(the_repository, quiet);
	run_commit_hook(use_editor, repo_get_index_file(the_repository),
			NULL, "post-commit", NULL);
```

`prepare_auto_maintenance` (`run-command.c:1956`) decides whether and how:

```c
	if (repo_config_get_bool(r, "maintenance.auto", &enabled)) {
		int gc_threshold;
		if (!repo_config_get_int(r, "gc.auto", &gc_threshold))
			enabled = gc_threshold > 0;
	}
	...
	if (repo_config_get_bool(r, "maintenance.autodetach", &auto_detach) &&
	    repo_config_get_bool(r, "gc.autodetach", &auto_detach))
		auto_detach = git_env_bool("GIT_TEST_MAINT_AUTO_DETACH", true);
	...
	strvec_pushl(&maint->args, "maintenance", "run", "--auto", NULL);
	strvec_push(&maint->args, quiet ? "--quiet" : "--no-quiet");
	strvec_push(&maint->args, auto_detach ? "--detach" : "--no-detach");
```

So a plain commit keeps its pack count bounded
(Git 2.54 made the `geometric` strategy the default for unscheduled maintenance,
`Documentation/RelNotes/2.54.0.adoc:58`;
the `gc` strategy repacks once `too_many_packs` in `builtin/gc.c:507` exceeds `gc.autoPackLimit`),
while the wrapper's landings accumulated packs without limit.

Every Git process that looks up an object walks the pack list until one pack holds it
(`packfile.c:2149`,
 `find_pack_entry`):

```c
	for (l = store->packs.head; l; l = l->next) {
		struct packed_git *p = l->pack;

		if (!p->multi_pack_index && fill_pack_entry(oid, e, p)) {
```

Each wrapper commit spawns about 55 Git processes,
so their combined time rose with the pack count.
Measured on copies of the 300-pack repository,
alternating five traced commits
(`GIT_TRACE2_EVENT` per-process exit times):
the 300-pack copy spent 264 to 272 ms in Git children per commit and 476 to 497 ms wall,
while a `git repack -a -d` copy spent 160 to 173 ms and 363 to 392 ms wall.

### Rejected readings

Commit history length and reflog length are not the growing term.
With the pre-fix build,
2000 synthetic commits in a single pack gave a 357 ms median,
and 20000 synthetic reflog entries gave 354 ms,
against 355 ms with neither.
Direct `git commit` in the 400-wrapper-commit repository stayed at 11 ms
because its own automatic maintenance repacked on the first direct commit,
which is also why a repository looked healthy again after any native commit.

## Verification

Git 2.55.0 (`/usr/bin/git`),
source tag `v2.55.0` (`e9019fcafe0040228b8631c30f97ae1adb61bcdc`).
Plumbing-only harness that shows the pack count grow and a native commit consolidate it,
run in a disposable directory:

```sh
# Disposable repository with 300 one-commit packs, then one native commit.
repo="$(mktemp --directory)"
git -C "$repo" init --quiet --initial-branch=main
git -C "$repo" config user.name x
git -C "$repo" config user.email x@example.invalid
git -C "$repo" commit --quiet --allow-empty --message=base
node --eval '
const lines = []
for (let i = 0; i < 300; i++) {
  const m = `c ${i}\n`, d = `d ${i}\n`
  lines.push("commit refs/heads/main", `committer x <x@example.invalid> ${1700000000 + i} +0000`,
    `data ${m.length}`, m, ...(i === 0 ? ["from refs/heads/main^0"] : []),
    "M 100644 inline f.txt", `data ${d.length}`, d, "", "checkpoint", "")
}
process.stdout.write(lines.join("\n") + "\n")' \
  | git -C "$repo" -c fastimport.unpackLimit=0 fast-import --quiet
ls "$repo/.git/objects/pack" | grep --count '\.pack$'
git -C "$repo" -c maintenance.autoDetach=false commit --quiet --allow-empty --message=native
ls "$repo/.git/objects/pack" | grep --count '\.pack$'
```

Output with Git 2.55.0,
run with `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`:

```text
300
1
```

### Wrapper before and after the fix

Scratch harness:
a disposable repository with 8 files and a policy-free trusted `cli-git.config.mjs`,
N synthetic one-commit packs from `fast-import` with `checkpoint`,
then 30 serialized `commit --quiet -- <path>` runs through each build's `dist/final/node/index.mjs`,
isolated from global and system Git config.
The band comes from 3 runs of each build with no synthetic packs:
medians 339 to 344 ms.

- 250 packs:
  before 435 ms median with quartiles 429 to 438 ms;
  after 343 ms with quartiles 341 to 347 ms.
- 500 packs:
  before 521 ms median with quartiles 518 to 528 ms;
  after 351 ms with quartiles 344 to 366 ms.
- 1000 packs:
  before 716 ms median with quartiles 694 to 745 ms;
  after 346 ms with quartiles 343 to 353 ms.
- 2000 packs:
  before 1062 ms median with quartiles 1051 to 1067 ms;
  after 344 ms with quartiles 342 to 348 ms.
- 2000 commits in a single pack:
  before 343 ms;
  after 344 ms.

After the fix,
the first commit in a many-pack repository still pays the old cost
(1299 ms at 2000 packs),
and the next few run while the detached repack works,
then the count settles at 5 packs.
600 real serialized wrapper commits with the fixed build gave per-50-commit medians between 340 and 355 ms,
ending at 4 packs,
against 356 rising to 463 ms over 300 commits before the fix
(that earlier run kept the host's global Git config,
which enables `core.fsmonitor`).

### Behaves as expected

- native `git commit`,
   `merge`,
   `rebase`,
   `am`,
   and `fetch`:
   run `git maintenance run --auto` afterwards;
- the wrapper after `131b1776d`:
   runs the same step after each landing
   (`package/git-policy/cli/src/policy-engine/commit-landing-auto-maintenance.ts`).

### Accumulates packs

- `index-pack --stdin` (with or without `--keep`) followed by `update-ref`,
  the pre-fix landing;
- `fast-import` with `checkpoint`,
  used in the harness to reproduce the state quickly.

## Verified workarounds

### Run automatic maintenance after each landing (adopted)

`runAutoMaintenance` reads the four config keys in one
`git config --type=bool-or-int --null --get-regexp` call,
mirrors `prepare_auto_maintenance`,
and runs `git maintenance run --auto --quiet --detach` or `--no-detach`
before `post-commit`,
with the caller's kept global options.
Git before 2.47 has no `maintenance run --detach` (added by merge `1e8962ee08`),
so exit status 129 retries without the flag,
the form native `git commit` used there.

Tradeoffs:
two or three extra Git spawns per commit;
`--quiet` is fixed,
so the "Auto packing" notice never appears;
a detached repack competes for CPU with the next few commits right after the threshold is crossed,
as it does after native commits;
a `pre-auto-gc` hook runs outside the cli-git hook lock.

### Periodic manual `git repack -a -d` or `git maintenance run`

Consolidates the packs,
but the cost regrows between runs,
and nothing reminds the user.

## What does not work

- Migrating objects as loose objects (`unpack-objects`) instead of a kept pack:
  it avoids pack scans but gives up the `.keep` protection that stops a concurrent `git gc --prune=now` or repack
  from dropping objects before the compare-and-swap,
  and loose objects still need `gc --auto` to stay bounded.
- Running `git gc --auto` instead of `git maintenance run --auto`:
  it ignores `maintenance.auto` and the maintenance strategy native commits use since Git 2.54.

## Upstream filing decision

`.out-of-scope/` has no Git entry.

1.  Is it really upstream's fault?
    No.
    Plumbing commands do not run automatic maintenance by design;
    porcelain callers opt in through `run_auto_maintenance`,
    and a tool that replaces porcelain with plumbing must do the same.
2.  Can upstream fix it?
    Not applicable after constraint 1.
3.  Are they supporting this use case?
    Yes,
     through `git maintenance run --auto`,
     which the fix uses.
4.  Would the repo welcome our contribution?
    Not evaluated,
     since there is nothing to contribute.
5.  Will they likely fix it?
    Not applicable.
6.  Prototype?
    Not applicable.

Decision:
 nothing to file.
