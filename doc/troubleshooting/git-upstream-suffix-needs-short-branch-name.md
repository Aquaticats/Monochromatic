# Git 2.56.0 `refs/heads/<branch>@{upstream}` and `@{push}` fail with "no such branch"

Note 2026-11-09T16:50:29-05:00:
`package/git-policy/cli/src/native/auto_push_reconcile.rs` is now
`package/git-policy/cli/src/native/push/auto_push_reconcile.rs`,
moved 2026-11-03 in `d88fd7ab2a`.

## Symptom

`git rev-parse` dies with exit status 128 when the part before `@{upstream}`,
`@{u}` or `@{push}`
is a full reference name instead of a branch name:

```text
$ git rev-parse --symbolic-full-name refs/heads/main@{upstream}
fatal: no such branch: 'refs/heads/main'
```

The same spelling with the short name answers:

```text
$ git rev-parse --symbolic-full-name main@{upstream}
refs/remotes/origin/main
```

The message says the branch does not exist,
although `refs/heads/main` exists and has an upstream.
A caller that reads a non-zero exit as "no upstream configured" gets a wrong answer and no error.
That is how the cli-git commit service's push planning in 0.5.7 always took the "no upstream" path:
every service push ran `push --set-upstream origin <b>:<b>` and never reconciled a remote branch that had moved.
0.5.8 asks with the short name (`package/git-policy/cli/src/native/auto_push_reconcile.rs`).

## Root cause

`interpret_branch_mark` in `object-name.c` hands the text before the `@` to `branch_get` unchanged
([object-name.c lines 1447 to 1452][object-name-mark]):

```c
// object-name.c (Git v2.56.0)
	if (at) {
		char *name_str = xmemdupz(name, at);
		branch = branch_get(name_str);
		free(name_str);
	} else
		branch = branch_get(NULL);
```

`branch_get` treats that text as a branch name and makes a branch record for it
([remote.c lines 1811 to 1823][remote-branch-get]):

```c
// remote.c (Git v2.56.0)
static struct branch *repo_branch_get(struct repository *repo, const char *name)
{
	struct branch *ret;

	read_config(repo, 0);
	if (!name || !*name || !strcmp(name, "HEAD"))
		ret = repo->remote_state->current_branch;
	else
		ret = make_branch(repo->remote_state, name,
				  strlen(name));
	set_merge(repo, ret);
	return ret;
}
```

`make_branch` prefixes `refs/heads/` to the name,
so `refs/heads/main` becomes a branch whose reference is
`refs/heads/refs/heads/main`,
configured by `branch.refs/heads/main.merge`,
which nobody sets
([remote.c lines 260 to 262][remote-make-branch]):

```c
// remote.c (Git v2.56.0)
	CALLOC_ARRAY(ret, 1);
	ret->name = xstrndup(name, len);
	ret->refname = xstrfmt("refs/heads/%s", ret->name);
```

`branch_get_upstream` then finds no merge configuration,
checks whether that doubled reference exists,
and reports the branch as missing,
naming the text the caller gave
([remote.c lines 1894 to 1902][remote-upstream]):

```c
// remote.c (Git v2.56.0)
	if (!branch->merge || !branch->merge[0]) {
		/*
		 * no merge config; is it because the user didn't define any,
		 * or because it is not a real branch, and get_branch
		 * auto-vivified it?
		 */
		if (!refs_ref_exists(get_main_ref_store(the_repository), branch->refname))
			return error_buf(err, _("no such branch: '%s'"),
					 branch->name);
```

`@{push}` goes through the same `interpret_branch_mark` and `branch_get`,
so it fails the same way.

This is the documented grammar,
not a defect:
`gitrevisions(7)` spells the suffix
`[<branchname>]@{upstream}` and `[<branchname>]@{push}`,
with a branch name
([Documentation/revisions.adoc lines 119 to 132][revisions-doc]).

## Verification

- Git under test:
  `git version 2.56.0` (`/usr/bin/git` on this host).
  Source read at tag `v2.56.0`,
  commit `a018953688f1b10bddf91bff8747068f5f4746a4`.
- Harness:
  `~/temp/agent/upstream-name/repro.ts` makes a throwaway bare remote and a clone with `main` tracking
  `origin/main`,
  with no system or global configuration,
  runs each spelling,
  and deletes the fixture.

```ts
// ~/temp/agent/upstream-name/repro.ts (the loop that prints each spelling)
for (const spelling of ['main@{upstream}', 'refs/heads/main@{upstream}', 'main@{u}', 'refs/heads/main@{u}', 'main@{push}', 'refs/heads/main@{push}', '@{upstream}', 'HEAD@{upstream}']) {
  const result = git(repo, 'rev-parse', '--symbolic-full-name', spelling);
  console.log(`rev-parse --symbolic-full-name ${spelling}: exit ${String(result.status)}: ${result.out}`);
}
```

### Spellings that answer

- `main@{upstream}`:
  exit 0,
  `refs/remotes/origin/main`.
- `main@{u}`:
  exit 0,
  `refs/remotes/origin/main`.
- `main@{push}`:
  exit 0,
  `refs/remotes/origin/main`.
- `@{upstream}` and `HEAD@{upstream}`:
  exit 0,
  `refs/remotes/origin/main` (the current branch).
- `git for-each-ref --format='%(upstream) %(push)' refs/heads/main`:
  exit 0,
  `refs/remotes/origin/main refs/remotes/origin/main`.
  The atoms take the full reference name,
  because `for-each-ref` names references,
  not branches.

### Spellings that fail

All exit 128 with `fatal: no such branch: 'refs/heads/main'`:

- `refs/heads/main@{upstream}`
- `refs/heads/main@{u}`
- `refs/heads/main@{push}`

## Verified workarounds

- Pass the short branch name to the suffix:
  strip `refs/heads/` from the reference first.
  Tradeoff:
  a branch whose short name is ambiguous with a tag or a remote-tracking name is still read as the
  branch,
  because `branch_get` never consults other namespaces;
  that is the wanted reading here.
- Read the upstream through `for-each-ref` with the full reference name and the `%(upstream)` or `%(push)` atom.
  Tradeoff:
  an absent upstream prints an empty field with exit 0 instead of failing,
  so the caller must treat an
  empty answer as "no upstream" explicitly.

## What does not work

- Reading any non-zero exit of `rev-parse <ref>@{upstream}` as "no upstream configured":
  the same exit and a misleading "no such branch" message also come from a full reference name.
  Tell the cases apart only after asking with the short name.

## Upstream filing decision

Not filed:
constraint 1 of the filing check fails.
The behavior matches the documented grammar,
which names a branch,
not a reference;
the reported text is accurate for the branch Git was asked about (`refs/heads/main` as a branch name).
Wording could name the doubled reference,
but that is a message preference,
not a defect.
No `.out-of-scope/` entry was needed,
since nothing is drafted.

[object-name-mark]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/object-name.c#L1447-L1452
[remote-branch-get]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/remote.c#L1811-L1823
[remote-make-branch]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/remote.c#L260-L262
[remote-upstream]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/remote.c#L1894-L1902
[revisions-doc]: https://github.com/git/git/blob/a018953688f1b10bddf91bff8747068f5f4746a4/Documentation/revisions.adoc?plain=1#L119-L132
