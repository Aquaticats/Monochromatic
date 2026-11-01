# Invalid `.git` ancestors must not become repository roots

## Symptom

An empty `.git/` directory or malformed `.git` file in an ancestor changed cli-git behavior even though Git rejected
the same directory tree:

```text
git -C <descendant> rev-parse --show-toplevel
fatal: not a git repository
```

Before issue #280,
`findGitRepoRoot` returned the ancestor and cli-git's `requireRoot` rejected the descendant with:

```text
cli-git: not at the root of the git repository
```

The wrapper therefore overrode Git based only on a filesystem name that Git did not recognize as a repository.

## Root cause

`package/fs-path/core/src/find-monorepo-root.ts` previously treated existence of any `.git` entry as sufficient.
`package/git-policy/cli/src/rules/require-root.ts` independently used `find-up` with `type: 'both'` and made the same
assumption.
Neither path inspected directory signatures or gitfile content.

Git itself applies stronger checks in upstream commit `f60db8d575adb79761d363e026fb49bddf330c73`:

- [`validate_headref`][validate-head] requires a symbolic `refs/` HEAD or a detached hexadecimal object ID;
- [`is_git_directory`][is-git-directory] requires valid HEAD plus common `objects/` and `refs/` directories;
- [`get_common_dir_noenv`][common-dir] resolves a linked worktree's optional `commondir` file;
- [`read_gitfile_gently`][read-gitfile] requires `gitdir: `,
  trims trailing CR and LF,
  resolves relative targets against the gitfile directory,
  and validates the target with `is_git_directory`.

The previous existence checks implemented none of these conditions.

## Verification

The diagnosis began with deterministic failing tests at both affected seams:

- `package/fs-path/core/src/find-monorepo-root.unit.test.ts` expected empty directories,
  malformed gitfiles,
  and missing gitfile targets to be rejected;
- `package/git-policy/cli/src/rules/require-root.unit.test.ts` expected invalid ancestors to pass through to real Git.

Before the fix,
the fs-path tests reported that all invalid markers unexpectedly passed,
and require-root threw its incorrect non-root diagnostic for both invalid ancestor forms.

After the fix:

- synthetic normal `.git` directories pass;
- dangling symbolic HEAD links into `refs/` pass without following the missing ref target;
- relative gitfiles targeting valid administrative directories pass;
- linked-worktree gitfiles with a separate `commondir` pass;
- empty `.git` directories fail discovery;
- malformed gitfiles and unusable targets fail discovery;
- cli-git passes invalid ancestors through to real Git;
- valid repository roots still pass;
- valid repository subdirectories still receive cli-git's root diagnostic.

A disposable real-Git fixture also exercised the built wrapper through the consumer boundary:

- an invalid empty ancestor reached Git 2.54.0 and returned its native exit `128` not-repository diagnostic;
- a real linked-worktree root passed;
- a linked-worktree subdirectory was rejected by cli-git;
- a real submodule root with a gitfile passed.

The disposable normal repository,
linked worktree,
submodule source,
and invalid ancestor were removed after verification.

## Verified fix

`package/fs-path/core/src/git-marker.ts` now mirrors Git's default administrative signatures without starting a
subprocess.
Its runtime filesystem seam distinguishes files from directories,
reads symbolic-link targets without following them,
resolves gitfile and `commondir` paths with native path semantics,
and rejects missing or malformed structures.

`findGitRepoRoot` returns a typed `GitRepositoryRootNotFoundError` only when no valid ancestor remains.
Cli-git's `requireRoot` consumes that shared finder and treats the typed absence as pass-through;
unexpected filesystem errors still fail instead of being swallowed.

The implementation intentionally validates Git's default on-disk structure rather than honoring process-only
`GIT_OBJECT_DIRECTORY` overrides.
Cli-git's policy root is the checked repository structure,
not an ambient alternate object store supplied for one command.

## What does not work

### Check only that `.git` exists

Empty directories,
stale files,
sockets,
and other unrelated entries all satisfy existence while Git rejects them.

### Check only the `gitdir: ` prefix

A syntactically plausible gitfile can target a missing file,
an empty directory,
or an administrative directory without valid HEAD,
objects,
or refs.
Git validates the target repository too.

### Require `objects/` and `refs/` beside every HEAD

Linked worktrees store HEAD in a per-worktree administrative directory and point at shared objects and refs through
`commondir`.
Ignoring that pointer would reject valid linked worktrees.

### Start Git for every ancestor candidate

This would make a cross-runtime filesystem utility depend on a process executable and add subprocess overhead to every
wrapper invocation.
The required signatures are deterministic filesystem checks with injected runtime adapters.

## Upstream filing decision

No upstream Git issue should be filed.
Git's source already validates these structures correctly.
The defect was entirely in this repository's marker-only approximations,
and issue #280 tracks the local correction.

[validate-head]: https://github.com/git/git/blob/f60db8d575adb79761d363e026fb49bddf330c73/setup.c#L350-L401
[is-git-directory]: https://github.com/git/git/blob/f60db8d575adb79761d363e026fb49bddf330c73/setup.c#L403-L448
[common-dir]: https://github.com/git/git/blob/f60db8d575adb79761d363e026fb49bddf330c73/setup.c#L322-L348
[read-gitfile]: https://github.com/git/git/blob/f60db8d575adb79761d363e026fb49bddf330c73/setup.c#L965-L1042
