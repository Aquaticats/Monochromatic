# Rewrite cli-git in Rust with shipped policies

Accepted and authorized for implementation by the user on 2026-10-04.
The user selected a Rust cli-git replacement,
JSONC configuration through the repository's existing Rust package,
forbidden-strings bundled into cli-git,
shipped policies only,
and removal of executable-configuration trust.
Repository configuration may disable or relax policies directly.
Only latest stable Git is supported.

The same implementing agent owns the unified Rust/Markdown/MDX linter.
It also uses JSONC,
replacing the earlier HCL choice.
The user subsequently authorized simple handwritten configuration merging inside the linter for now;
the separate deepmerge port and fork-test wait are no longer prerequisites.

Preserve the existing required policy,
transaction,
concurrency,
recovery,
worktree,
and push behavior.
Container tests,
mutation testing,
and fuzzing are completion requirements.
The expanded cli-git 2.x catalog is out of scope.
The unified linter's first publication still requires separate user approval.

Scope and tradeoffs:
[`cli-git-rust-rewrite.md`](../planning/cli-git-rust-rewrite.md).
Implementation:
[`cli-git-rust-implementation.md`](../planning/cli-git-rust-implementation.md).
Execution:
[`cli-git-rust-implementation.md`](../handover/cli-git-rust-implementation.md).
