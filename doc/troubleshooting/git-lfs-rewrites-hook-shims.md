# git-lfs 3.8.0: the filter commands rewrite the four hook shims after they were deleted

## Symptom

On 2026-10-29 the four `git lfs` shims in the main checkout's `.git/hooks` (`post-checkout`, `post-commit`,
`post-merge`, `pre-push`) were deleted by hand, because traditional hooks are unsupported by the Git wrapper.
Deleting them does not last: the next command that runs the LFS filter in a Git the wrapper did not start writes them
again (verified below in a fresh repository, not observed in the main checkout).

## Root cause

The filter commands install the hooks themselves.
In the git-lfs source, `commands/command_filter_process.go` and `commands/command_clean.go` each call
`setupRepository()` and then `installHooks(false)`.
`installHooks` loads the four hooks for the hooks directory (`core.hooksPath` when set, otherwise `.git/hooks`)
and writes each one that is absent (`lfs/hook.go`, `Hook.Install`: an existing file is only upgraded when its content
is one of the known older shims).
The filter runs on any Git command that cleans or smudges a path with `filter=lfs`: `git add`, `git status`,
`git checkout`, `git commit -- <path>`.
The write error of `installHooks(false)` is ignored by both callers.

Verified on 2026-10-29 with git-lfs 3.8.0: in a fresh repository with `filter.lfs.*` configured, `git add` of an LFS path
by real Git started directly wrote `pre-push`, `post-checkout`, `post-commit` and `post-merge` into `.git/hooks`,
byte for byte the text `git cli-git doctor` recognizes (`binary_hooks_off_tests.rs`,
`the_recognized_shim_bytes_are_the_ones_the_real_git_lfs_writes`).

## Resolution in the wrapper

- The wrapper does not need the shims: it runs the Git LFS upload itself as a configuration hook on every push it starts
  (`package/git-policy/cli/SPEC.md`, "Git LFS upload before a push").
- Every real Git the wrapper starts runs with `-c core.hooksPath=/dev/null/cli-git-no-hooks`
  (`SPEC.md`, "Real Git starts with traditional hooks off"), so a shim that is present never runs through the wrapper
  and there is no second upload.
- The same setting reaches the clean filter that Git starts, and git-lfs reads `core.hooksPath` to choose the directory
  for its shims: through the wrapper it finds no directory it can write and writes no shim
  (measured with the same test; the direct `git add` of the positive control writes all four).
  The shims therefore return only from a Git started without the wrapper.
- `git cli-git doctor` recognizes a shim by its exact bytes, explains it apart from the unsupported hooks, and exits `0`
  when nothing else is found.
- Deleting the files or leaving them are both fine.

## Workaround for Git started without the wrapper

Make the hooks directory unwritable (`chmod 0555 .git/hooks`); git-lfs ignores the write error.
This also blocks every other writer of that directory.
Pointing `core.hooksPath` at a directory only moves the shims there, and makes the wrapper print its
`traditional-hooks-unsupported` warning on each commit.
