# git-lfs 3.8.0: the filter commands rewrite the four hook shims after they were deleted

## Symptom

On 2026-10-29 the four `git lfs` shims in the main checkout's `.git/hooks` (`post-checkout`, `post-commit`,
`post-merge`, `pre-push`) were deleted by hand, because traditional hooks are unsupported by the Git wrapper.
Deleting them does not last: the next command that runs the LFS filter writes them again (verified below in a fresh
repository, not observed in the main checkout), and `git cli-git doctor` then exits `1` and lists them.

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
wrote `pre-push`, `post-checkout`, `post-commit` and `post-merge` into `.git/hooks`
(the binary controls in `package/git-policy/cli/src/native/binary_lfs_server_tests.rs` remove them again for that reason).

## Workaround

- The wrapper does not need the shims: it runs the Git LFS upload itself as a configuration hook on every push it starts
  (`package/git-policy/cli/SPEC.md`, "Git LFS upload before a push").
  A shim that is present runs in addition on a push real Git runs, and its upload finds the objects already stored.
- To keep the shims away, make the hooks directory unwritable (`chmod 0555 .git/hooks`); git-lfs ignores the write error.
  This also blocks every other writer of that directory.
- Pointing `core.hooksPath` at a directory only moves the shims there, and makes the wrapper print its
  `traditional-hooks-unsupported` warning on each commit.
