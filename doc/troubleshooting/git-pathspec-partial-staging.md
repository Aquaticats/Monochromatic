# Git 2.55.0 pathspec commit after partial staging records unstaged bytes from the same file

## Symptom

A file contains both intended staged changes and unrelated unstaged changes.
`git diff --cached -- <file>` shows only the intended hunk.
Running this command still commits every working-tree change in the named file:

```sh
git commit --message "scoped change" -- <file>
```

This occurred in commit `d28eb9155`.
Only the new `package/array-at/core` importer was staged in `pnpm-lock.yaml`,
 but the pathspec commit also recorded pre-existing catalog-upgrade changes from the working file.
The commit therefore reported 940 insertions and 768 deletions rather than only the importer block.
A corrective commit comment records that understated scope.

No error is emitted.
The commit succeeds because Git implements pathspec commits as working-tree partial commits,
 not as a filter over already staged hunks.

## Root cause

The behavior is documented rather than accidental.
Git commit documentation at upstream commit `1a3e64c6c4a623626ff0687008732a8e007e2a1c`,
 `Documentation/git-commit.adoc:406-410`,
 says a command-line pathspec commits matching file contents without recording changes already added to the index:

```text
<pathspec>...::
        When <pathspec> is given on the command line, commit the contents of
        the files that match the pathspec without recording the changes
        already added to the index. The contents of these files are also
        staged for the next commit on top of what have been staged before.
```

Git enters its partial-commit path in `builtin/commit.c:500-516`:

```c
/*
 * A partial commit.
 *
 * (0) find the set of affected paths;
 * (1) get lock on the real index file;
 * (2) update the_index with the given paths;
 * ...
 * (5) reset the_index from HEAD;
 * (6) update the_index the same way as (2);
 * ...
 */
commit_style = COMMIT_PARTIAL;
```

The implementation then loads the real index and updates each selected path from working-tree state.
`builtin/commit.c:527-548` contains both updates:

```c
if (list_paths(&partial, !current_head ? NULL : "HEAD", &pathspec))
        exit(1);

discard_index(the_repository->index);
if (repo_read_index(the_repository) < 0)
        die(_("cannot read the index"));

repo_hold_locked_index(the_repository, &index_lock, LOCK_DIE_ON_ERROR);
add_remove_files(&partial);
...
create_base_index(current_head);
add_remove_files(&partial);
```

The first `add_remove_files(&partial)` refreshes selected paths in the real index.
The second builds the temporary index used for the commit from `HEAD` plus those selected working-tree paths.
A partially staged file is therefore replaced as a whole for that pathspec commit.

This repository's `cli-git` wrapper reinforces explicit path selection by injecting `--only`.
`package/git-policy/cli/src/rule/commit-only.ts:170-180` documents both injection and opt-outs:

```ts
 * The rule injects `-o` (a.k.a. `--only`) into `git commit` commands when
 * not already specified, forcing every commit to name the paths it includes
 * rather than silently picking up whatever happens to be staged.
 * ...
 * Skipped when `-o`, `--only`, or `--no-only` is already present in the
 * post-subcommand region (the user made an explicit choice). The
 * wrapper-only flag `--no-enforce-only` is the escape hatch: it is stripped
 * from args before forwarding, and injection is also skipped for that
 * invocation.
```

The earlier reading was wrong:
an explicit commit pathspec does not mean "commit the staged portion of this path."
It means "construct a partial commit from current working-tree contents for this path."

## Verification

### Version and source

The reproduction used `/usr/bin/git` version `2.55.0`.
The source trace used `git/git` commit `1a3e64c6c4a623626ff0687008732a8e007e2a1c`.
Using `/usr/bin/git` bypassed this repository's `cli-git` wrapper and isolated upstream Git semantics.

### Failing catalog

Run in a disposable directory:

```sh
scratch="$(mktemp --directory)"
/usr/bin/git -C "$scratch" init --quiet
/usr/bin/git -C "$scratch" config user.name Reproduction
/usr/bin/git -C "$scratch" config user.email reproduction@example.invalid
printf 'first=base\nsecond=base\n' > "$scratch/settings.txt"
/usr/bin/git -C "$scratch" add -- settings.txt
/usr/bin/git -C "$scratch" commit --quiet --message base
printf 'first=staged\nsecond=base\n' > "$scratch/settings.txt"
/usr/bin/git -C "$scratch" add -- settings.txt
printf 'first=staged\nsecond=unstaged\n' > "$scratch/settings.txt"
/usr/bin/git -C "$scratch" diff --cached -- settings.txt
/usr/bin/git -C "$scratch" diff -- settings.txt
/usr/bin/git -C "$scratch" commit --quiet --message partial -- settings.txt
/usr/bin/git -C "$scratch" show HEAD:settings.txt
```

Before the commit,
 cached diff contains only `first=staged`,
 while working-tree diff contains only `second=unstaged`.
Committed content is:

```text
first=staged
second=unstaged
```

Failing forms have the same pathspec semantics:

- `git commit --message partial -- settings.txt`
- `git commit --only --message partial -- settings.txt`
- repository wrapper command with explicit `settings.txt`,
   because `cli-git` injects `--only`

### Working catalog

Recreate the fixture and replace pathspec commit with:

```sh
/usr/bin/git -C "$scratch" commit --quiet --no-only --message staged-only
/usr/bin/git -C "$scratch" show HEAD:settings.txt
/usr/bin/git -C "$scratch" diff -- settings.txt
```

Committed content is only staged state:

```text
first=staged
second=base
```

Working-tree diff still contains:

```diff
-second=base
+second=unstaged
```

The index is clean after commit.

## Verified workarounds

### Use `--no-only` with no pathspec

For this repository,
 prefer:

```sh
git add --patch <file>
git diff --cached -- <file>
git diff -- <file>
git commit --no-only --message "<message>"
```

`--no-only` tells both Git and `cli-git` to commit existing index state rather than construct a pathspec commit.
The workaround was verified by the working catalog.

Tradeoff:
this commits every staged path,
 not only one named file.
Before committing,
 inspect `git diff --cached --name-only` and `git diff --cached` to prove the whole index is in scope.
Use it only when a pathspec cannot express intended staged hunks,
 such as one file containing both task and concurrent-work changes.

### Use wrapper escape hatch when native opt-out cannot be expressed

`git commit --no-enforce-only --message "<message>"` makes `cli-git` strip its wrapper flag
and forward a pathless commit.
`package/git-policy/cli/src/rule/commit-only.ts:177-180` defines this behavior.

Tradeoff:
this bypasses commit-only enforcement for the invocation.
Repository rule CLG permits an enforcement bypass only when no scoped pathspec fits.
Prefer native `--no-only` because it records an explicit Git selection-mode choice without bypassing the wrapper rule.

### Isolate generation before staging

Generate a shared artifact in a disposable worktree that contains no concurrent working changes,
 then apply only generated task hunks to the main worktree.

Tradeoff:
generators whose output depends on concurrent configuration changes may produce a stale or incompatible artifact.
Compare source configuration and generated output before applying.

## What does not work

### Partial staging followed by a pathspec commit

`git add --patch <file>` controls index state,
 but `git commit -- <file>` replaces selected path content from working tree while constructing the partial commit.
The cached diff is not the content source for that named path.

### Adding `--only`

`--only` makes selection intent explicit but retains the same working-tree pathspec semantics.
It does not mean "only staged hunks from this path."

### Inspecting only cached diff

A clean cached patch does not predict pathspec-commit content when named files have unstaged modifications.
Inspect both `git diff --cached -- <file>` and `git diff -- <file>`.

### Reverting the accidental commit

Reverting would remove concurrent catalog changes that were not owned by this task.
The repository policy treats those changes as concurrent work.
The safe remediation was to preserve them and add a corrective commit comment describing actual scope.

## Upstream filing decision

No matching `.out-of-scope/` exemption exists for Git commit pathspec behavior.
Searches of open and closed `git/git` issues and pull requests for commit,
 pathspec,
 index,
 and partial-staging combinations returned no matching thread.

1. **Is it really upstream's fault?**
    No.
   Git documents and tests pathspec commits as partial commits from working-tree paths.
   The incident came from using that mode after partial staging.
2. **Can upstream fix it?**
    Not without changing documented command semantics or adding another explicit mode.
   Existing pathless commit already provides staged-index semantics.
3. **Are they supporting this use case?**
    Yes.
   `Documentation/git-commit.adoc:414-417` identifies pathless `git commit` as the basic way to record staged state.
4. **Would the repository welcome our contribution?**
    No filing is warranted,
   and `Documentation/SubmittingPatches:572-595` warns that material appearing AI-generated will be rejected.
5. **Will they likely fix it?**
    No relevant defect exists to fix.
   The desired behavior is already available through pathless commit and `--no-only`.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    No upstream fix was prototyped because constraints one and five fail.
   The verified consumer-side workaround selects existing staged-index behavior.

Nothing should be filed upstream.
There is no issue or comment draft because the behavior matches documented Git semantics
and repository-local tooling already exposes a verified opt-out.
