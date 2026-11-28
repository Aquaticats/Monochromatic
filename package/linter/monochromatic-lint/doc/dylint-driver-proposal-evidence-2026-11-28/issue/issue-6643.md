# Issue 6643, read with `gh issue view 6643 --json title,body` on 2026-11-28T03:13:25-05:00

TITLE: monochromatic-lint: rust/avoidable-closure is skipped on every host run because its Dylint driver is absent, and a skip reads as a pass

`rust/avoidable-closure` runs on no host run of the linter on the coordinating session's machine, so every count of findings taken there leaves that rule out. Filed by the coordinating session of the Git wrapper work.

## What a run prints

Measured 2026-11-27T20:15:12-05:00 with the installed `0.5.2-alpha.23`, on `package/mpm-shadow/cli`, through the root task (`mise run //:lint:monochromatic-lint -- package/mpm-shadow/cli`) and through the binary alone, with the same result: one `rust/avoidable-closure` line whose message begins "rust/avoidable-closure skipped package/mpm-shadow/cli: the Dylint driver for toolchain `nightly-x86_64-unknown-linux-gnu` at /home/user/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver is absent (cargo-dylint is 6.4.0), and `dylint_driver` 6.4.0 with its dependencies is not in the cargo cache, so `cargo dylint` would have to download it, which this linter does not do". The wrapper's package and `cargo-shadow`'s print the same line. Rounds and second passes of 2026-11-27 each noted the skip and went on, and a run "without findings" was read as clean more than once before the line was noticed.

## Why it matters

The rule exists and its findings are unknown for every Rust package here. A skipped rule looks like a passing one unless the reader knows to look for the line, and the repository's rule for a count asks that its scope be checked before a zero is trusted.

## Asked

- Who puts the Dylint driver on a machine, and how: found from the linter's own source and README, the toolchain plan and lock, and the workflows (does any workflow run the rule today, and with which driver). Stated with file and line.
- The options, each with what it costs, as a proposal under `doc/planning/` and not built until the owner picks: the toolchain's install provides the driver for the pinned nightly; the linter's own task fetches it once, outside the offline run; the rule moves to a check that runs where the driver is; or a skipped rule ends the run with a status that is not a pass, so nobody reads a skip as clean.
- Whichever is picked, one run of the rule over the Rust packages this work owns (the wrapper, `mpm-shadow`, `cargo-shadow`, the linter), with its findings counted.
