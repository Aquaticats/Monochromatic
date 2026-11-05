# pnpr alpha.17 upgrade and publishing repair

## Authorized work

The owner asked to bump pnpr to `0.1.0-alpha.17` during the Cargo-registry discussion.
When asked how to handle its incompatible storage layout,
the owner chose "Repair publishing,
then rebuild packages".
The archive-transfer approach is not the selected implementation.
Repair publishing before deploying a fresh store,
then republish current versions.
Retain the existing volume for rollback.

The Cargo deliverable remains a proposal,
recorded in [self-hosted-cargo-registry.md](../planning/self-hosted-cargo-registry.md).
Cargo registry implementation has not been requested.

## Current state

- Upgrade branch:
  `fix/config-pnpr-alpha17`.
- Worktree:
  `${HOME}/temp/agent/pnpr-alpha17-upgrade-2026-11-04`.
- Commits `c6a84fbdc2` and `9a17d57e45` prepare the upstream image pin,
  generated `maxUsers` configuration,
  a fresh storage volume,
  and an isolated image-verification mode in `pnpr-publish.yml`.
- The live server still reports alpha.16.
  The upgrade branch has not been merged into the deployment branch.
- Image verification run [41941248129](https://github.com/Aquaticats/Monochromatic/actions/runs/41941248129)
  passed signed GitHub workload publication,
  anonymous-write refusal,
  off-target workload refusal,
  and exact archive download.
  Local package tests and type checks passed after building the package.
- Commit `ca5f88b3e8` restores the missing build entries in package `task.ts` files.
  The publisher's real build planner now accepts every configured package:
  154 have build tasks and 15 have no build.
- Commit `8a897f3007` adds the `rebuild_check` workflow input.
  Run [41944061291](https://github.com/Aquaticats/Monochromatic/actions/runs/41944061291)
  is building and packing the configured packages against an empty disposable registry.
  It skips production publication.
- Source findings and incompatible-storage details are in
  [pnpr-alpha17-upgrade.md](../planning/pnpr-alpha17-upgrade.md).

## Publishing repair

Run `41892186106` fails with `MissingCiBuildTaskError`.
The missing build definitions are already recorded in
[mise-free-ci.md](../decision/mise-free-ci.md).
The current native CI runner requires package `task.ts` definitions;
many publishable packages have only `mise.toml` definitions.

The separate task-system migration remains a proposal.
The owner called the existing runner a stopgap,
as recorded in [task-slice-plan-revision.md](task-slice-plan-revision.md).
Repairing publishing does not authorize adopting the proposed replacement task system.

Next action:
finish the actual rebuild check in CI and repair any package failures it identifies,
then land the alpha.17 deployment and republish current versions.
Do not cut over to the empty volume while the publisher remains unable to rebuild them.
