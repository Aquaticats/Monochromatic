# The git wrapper's test-only switches are compiled out of release builds

Status: accepted by the owner on 2026-11-25,
answering item 8 of `package/git-policy/cli/doc/debt-ledger-2026-11-25.md`.

## Decision

Every `CLI_GIT_TEST_ONLY_*` environment variable the release code reads
is compiled out of release builds behind a Cargo feature,
the way the start probe's raised deadline already is
(`start_probe/probe_deadline.rs`, feature `start-probe-raised-deadline`).
A release build reads none of them; the binary-level controls run against a build with the feature on.

The owner chose "Gate all behind a feature (Recommended)"
over gating only the switches that kill or stall the service,
and over keeping them and documenting the risk.

## Why

The ledger measured 18 such constants in release files.
Some kill the commit service
(`service/service_fault.rs` sends the process `SIGKILL` when `CLI_GIT_TEST_ONLY_SERVICE_FAULT` names the fault point),
some pause or kill a transaction phase (`transaction/transaction_phase.rs`),
the add route can `raise(SIGKILL)` (`worktree_clone_add_route/add_route_plan.rs`),
and `service/service_warm_request_control_window.rs` parks on one.
Any exported variable, or a unit's `Environment=` line, can arm them in normal use,
against the project's "Commits should never fail".
The start probe's feature exists for the same reason, as its `Cargo.toml` comment says.

## What it costs

The binary-level controls need a build with the feature on,
which is one more compile per gate run (a build of this crate peaks at about 6 GiB, measured 2026-11-25),
and 17 sites to convert.
