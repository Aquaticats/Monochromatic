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
