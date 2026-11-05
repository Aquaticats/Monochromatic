# pnpr alpha.17 rollout

## Current state

The upgrade and publishing repair are merged on `main` as `1f6a6f8dc8a7f682644b840c2468d46d7f0c5503`.
The generated registry configuration matches the current workspace:
169 selected packages.
Production readiness reported storage and authentication healthy at `2026-11-05T05:06:42.668Z`.
The metrics endpoint changed from the observed alpha.16 version response to the expected disabled endpoint.

Push-triggered publication run
[41971694888](https://github.com/Aquaticats/Monochromatic/actions/runs/41971694888)
succeeded,
but it took its missing-version inventory before deployment changed stores.
The required post-cutover pass is
[41972276692](https://github.com/Aquaticats/Monochromatic/actions/runs/41972276692).
A bounded watcher is running for that pass.

## Remaining checks

- Finish the post-cutover publication pass.
- Download every current selected archive anonymously and check its SHA-512 integrity.
- Install and invoke a representative package in a disposable npm consumer without credentials.
- Finish removing the isolated upgrade worktree and stop all owned watchers.
- Record the live results in the registry decision and remove this completed handover.

The accepted recovery choice,
image evidence,
and deployment constraints are in
[private-npm-registry.md](../decision/private-npm-registry.md).
The task-runner verification and local cgroup measurements are in
[the task CLI README](../../package/monorepo-task/cli/README.md).

The Cargo registry remains the proposal in
[self-hosted-cargo-registry.md](../planning/self-hosted-cargo-registry.md).
It is not part of this deployment.
