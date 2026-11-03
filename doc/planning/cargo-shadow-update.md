# What `cargo update` does under the cargo shadow

Status: decided by the owner on 2026-11-03 ("I said follow pnpm semantics").
The earlier options (pass through, shadow-owned update, run-then-check with rollback, refuse and redirect) are superseded;
 the cross-vendor review of them, if it returned, is in the commit history of this file.

The rule: `cargo update` under the shadow behaves like `pnpm update`, pnpm's own policy-aware command
 ([pnpm update][pnpm-update], read 2026-11-03), and `cargo generate-lockfile` like `pnpm install --lockfile-only`
 ([pnpm install][pnpm-install]).
Cargo's resolver never chooses freely: the release-age and trust policy of `rust-catalog.toml` applies while versions are chosen.

The flag mapping lives in `package/cargo-shadow/cli/src/update_plan.rs` and the README of the package.
pnpm has no counterpart for `--breaking` as a cargo-resolved update, and none for a dry run of `update`;
 the nearest pnpm behavior was taken for each (README, "Mapping `cargo update` to `pnpm update`").

[pnpm-update]: https://pnpm.io/cli/update
[pnpm-install]: https://pnpm.io/cli/install

## Cross-vendor review of the earlier options

A reviewer from another model vendor (`openai/gpt-6.1-sol`, no tools, only the first version of this file) answered on 2026-11-03.
It had no repository access, so its points are about design, not about this code.

- It preferred, over any live-file rollback, running the native operation against a private candidate lockfile,
   validating that candidate, and publishing it only if validation passes and the original lockfile has not changed meanwhile.
  It called this feasibility-dependent: the toolchain must be able to target a private lockfile, copying a workspace breaks path
   dependencies and configuration discovery, and a content comparison followed by a rename is not an atomic compare-and-swap.
- Against snapshot and restore, which is what the update does today:
  - a concurrent writer between the snapshot and the restore loses its change (restoring is atomic, overwriting someone else's work is not);
  - a rejected version is visible to other readers while the check runs;
  - a process killed after cargo wrote the lockfile and before the restore leaves the unchecked lockfile behind;
  - restoring must restore absence, not an empty file (the shadow's full update removes the stand-in lockfile it created on a refusal).
- A version that passes the release-age and trust policy can still fail to build; the policy is not a compatibility check.
- A "trust policy" with pnpm's name is not automatically the same rule as the crate catalog's; the catalog's levels are heuristics from crates.io data.
- It could not verify pnpm's write ordering for `pnpm update`; the owner's answer settled the question by rule instead.

The package README states the transaction limits.
A staged candidate lockfile (`cargo -Z unstable-options --lockfile-path`, if the pinned toolchain has it) is the open improvement.
