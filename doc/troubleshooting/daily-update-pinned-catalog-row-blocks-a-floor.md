# The daily update lands nothing while a `pinned` catalog row refuses the floor it raised

## Symptom

The daily update's one notification reports the dependency group failed,
and nothing from it lands:

```text
dependency locks: "tighten the catalogs, render the manifests, make every lock satisfy them" failed, so nothing from
this group was landed and its files were put back. `mise exec -- cargo-shadow install` exited with status 1. Last
output: ...
```

The journal (`journalctl --user --unit toolchain-daily-update`) names the resolution failure:
cargo cannot pick one Slint release for `package/ide/app` or `package/music-player/desktop-app`,
because those crates also select the `pinned` row `i-slint-backend-testing`.
The pnpm lock and every other `Cargo.lock` go back with it,
so the dependency group stays where it was and the next day reports the same failure.

## Root cause

The Rust catalog's default floors are tightened to what the lockfiles already hold,
and a named catalog is never tightened:

1.  `mise run catalog:tighten` raises each default `>=` floor to the highest release any consuming crate's `Cargo.lock`
    holds
    (`package/dependency-catalog/cli-tighten/src/cargo/tighten.ts`,
    `planCargoTighten`).
    One crate locking the newer release is enough to raise the floor for all of them.
2.  It skips every entry under `[catalogs.pinned]` and every other named catalog
    (`tighten.ts`, the `SKIP` entries it appends per `document.catalogs`),
    and it writes no lockfile
    (`package/dependency-catalog/cli-tighten/src/cargo/run.ts`).
3.  `mise run sync:files` renders the raised floor into every manifest that uses the entry.
4.  `mise exec -- cargo-shadow install` then has to find one release for a crate that selects both the raised floor and
    the pin.
    The Slint crates have to agree: the published `i-slint-backend-testing 1.19.1` requires `i-slint-core "=1.19.1"`
    (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/i-slint-backend-testing-1.19.1/Cargo.toml:123-124`).
    GitHub issue 6398 records the outcome when the pin stood at `=1.18.1` and the floor had moved to `>=1.19.0`:
    `package/ide/app` and `package/music-player/desktop-app` resolved no dependency set.
5.  The install fails, the hard tail puts every dependency file back, and nothing lands
    (`package/monorepo-task/core-exec/src/daily-update-run.ts`, `runSteps`).

That outcome is the one issue 6398 asks for ("or lands neither"):
the floor never lands without the manifests and locks that satisfy it,
and a pinned row is never left behind in a landed manifest set.
What it also means is that the dependency group makes no progress at all,
pnpm lock included,
until the pin moves.

## What to do when the notification arrives

Raise the pinned row to the release the floor resolves to,
then regenerate the manifests and the locks in the same commit
(`doc/runbook/rust-dependencies.md`, "Update every crate").
The owner's decision of 2026-11-27 is that the pin follows the floor
(`doc/decision/rust-dependency-catalog.md`, "A pinned row follows the floor it is tied to").

## Why the update cannot raise the pin itself

Nothing in `rust-catalog.toml` links the pin to the floor it must equal.
The tie is the comment above the row and the sentence in `catalog:tighten`'s `SKIP` reason,
so no command can compute the pin's next value.

The options, none of them decided:

- Express the tie in the catalog as a field (the floor a pin must equal), then let `catalog:tighten` raise the floor and
  the pin together.
- Have `catalog:tighten` refuse to raise a default floor that a selected exact pin cannot resolve with, naming both
  rows, so the notification carries the whole story and the other floors still land. This leaves the floor where it is,
  which the owner rejected for the Slint case, so it would need that decision reopened.
- Keep today's behaviour, land nothing and notify, and move the pin by hand, which is what the Consequences section of
  `doc/decision/rust-dependency-catalog.md` already describes.

## What was verified, and what was not

- Read from the tool's source: the `SKIP` of named catalogs and the missing lockfile write in `catalog:tighten`, the
  failing and restoring `install` in `package/cargo-shadow/cli/src/install.rs`, and the group restore in
  `daily-update-run.ts`.
- Read from the registry source: `i-slint-backend-testing 1.19.1` requires `i-slint-core "=1.19.1"`.
- Recorded by GitHub issue 6398: the two apps resolved no dependency set once the pin stood below the raised floor.
- Not run: a real `mise run toolchain:update:daily` against the registry in this round.
  The fixture test that proves the "lands neither" half
  (`package/monorepo-task/core-exec/src/daily-update-whole-path.expensive.unit.test.ts`,
  "lands no dependency file when an install reports success and leaves a Cargo lock stale after the floor moved")
  injects a stale lock instead of reproducing the pin's resolution failure.

## Status

Written by the round for issue 6398 on 2026-11-27.
Unapproved and non-final: the mechanism is read from the sources named above,
no real update was run, and the three options in "Why the update cannot raise the pin itself" are the owner's to
choose.
