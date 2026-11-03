# What `cargo update` does under the cargo shadow

Status:
decided by the owner on 2026-11-03 ("I said follow pnpm semantics").
The earlier options (pass through,
shadow-owned update,
run-then-check with rollback,
refuse and redirect) are superseded;
 the cross-vendor review of them,
if it returned,
is in the commit history of this file.

The rule:
`cargo update` under the shadow behaves like `pnpm update`,
pnpm's own policy-aware command
 ([pnpm update][pnpm-update],
read 2026-11-03),
and `cargo generate-lockfile` like `pnpm install --lockfile-only`
 ([pnpm install][pnpm-install]).
Cargo's resolver never chooses freely:
the release-age and trust policy of `rust-catalog.toml` applies while versions are chosen.

The flag mapping lives in `package/cargo-shadow/cli/src/update_plan.rs` and the README of the package.
pnpm has no counterpart for `--breaking` as a cargo-resolved update,
and none for a dry run of `update`;
 the nearest pnpm behavior was taken for each (README,
"Mapping `cargo update` to `pnpm update`").

[pnpm-update]: https://pnpm.io/cli/update
[pnpm-install]: https://pnpm.io/cli/install

## Cross-vendor review of the earlier options

A reviewer from another model vendor (`openai/gpt-6.1-sol`,
no tools,
only the first version of this file) answered on 2026-11-03.
It had no repository access,
so its points are about design,
not about this code.

- It preferred,
  over any live-file rollback,
  running the native operation against a private candidate lockfile,
   validating that candidate,
  and publishing it only if validation passes and the original lockfile has not changed meanwhile.
  It called this feasibility-dependent:
  the toolchain must be able to target a private lockfile,
  copying a workspace breaks path
   dependencies and configuration discovery,
  and a content comparison followed by a rename is not an atomic compare-and-swap.
- Against snapshot and restore,
  which is what the update does today:
  - a concurrent writer between the snapshot and the restore loses its change (restoring is atomic,
    overwriting someone else's work is not);
  - a rejected version is visible to other readers while the check runs;
  - a process killed after cargo wrote the lockfile and before the restore leaves the unchecked lockfile behind;
  - restoring must restore absence,
    not an empty file (the shadow's full update removes the stand-in lockfile it created on a refusal).
- A version that passes the release-age and trust policy can still fail to build;
  the policy is not a compatibility check.
- A "trust policy" with pnpm's name is not automatically the same rule as the crate catalog's;
  the catalog's levels are heuristics from crates.io data.
- It could not verify pnpm's write ordering for `pnpm update`;
  the owner's answer settled the question by rule instead.

The package README states the transaction limits.
A staged candidate lockfile (`cargo -Z unstable-options --lockfile-path`,
if the pinned toolchain has it) is the open improvement.

## Decisions of 2026-11-03 and what each rests on

Each follows from words the owner said;
pnpm references are to the sources of pnpm 12.16.1 (the installed version,
 cloned at tag `v12.16.1`,
commit `c7b46ae`).

- The package exists,
  and the Rust catalog tools live in it:
  "Make a cargo shadow like what we did for pnpm shadow.
  Put the existing rust catalog tools inside it instead of making them separate."
- `cargo update` is `pnpm update`,
  `cargo generate-lockfile` is `pnpm install --lockfile-only`:
  "I said follow pnpm semantics."
  The flag mapping is in `package/cargo-shadow/cli/src/update_plan.rs` and the README.
- The mise tasks `rust:install`,
  `rust:update` and `rust:verify` are removed:
  "I also said to remove the Mise tasks that would then be handled via the wrapper."
  Workflows never use mise ("Workflows should never use mise at all."),
  so `advisory-core.yml` calls `cargo-shadow` directly.
- A failed or refused install step warns and runs the command anyway,
  with `--locked` added once,
  before any `--`.
  pnpm:
  "If that install fails,
  for example because the store is read-only or there is no network,
  pnpm prints a warning and runs
   the command anyway" (<https://pnpm.io/settings/build>).
  pnpm can do that because `pnpm run` never rewrites the lockfile;
  cargo does,
  hence `--locked`.
  Replaces the fail-closed behavior built first.
- The crates.io database dump path is ported (`dump.rs`,
  `csv.rs`),
  because the owner moved the whole tool into the shadow;
  a cold run reads one streaming pass instead of one API request per crate at 1.1 seconds.
- `--breaking` raises the catalog floors and runs the sync,
  as `pnpm update --latest` writes the new range to the catalog entry:
  `pnpm11/installing/deps-resolver/src/index.ts:280-322` (`getUpdatedCatalogs`,
  `getCatalogUpdatesOfProject`) collects,
  for each
   wanted dependency with a catalog lookup and `updateSpec`,
  the updated range;
  `wantedDepShouldUpdateCatalog.ts:3-5` makes that
   true unless `saveSpec` is false (`--no-save`).
  The project manifest keeps `catalog:`;
  the catalog file gets the range.
- Every locked entry is verified on a frozen install,
  and there is no `verify` command:
  `pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343` (`startLockfileVerification`) re-validates every entry of
   the lockfile against the resolver policies on install,
  skipped only when `opts.trustLockfile` is set;
   `verifyLockfileResolutionsCache.ts` caches a passing lockfile by hash.
  `pnpm11/installing/deps-installer/test/install/trustLockfile.ts:21-52` shows both:
  a frozen install rejects a lockfile a verifier
   refuses,
  and `trustLockfile: true` skips it.
  The default of `trustLockfile` is false (`pnpm11/workspace/state/src/validation.ts:428`).
- The bin name stays `cargo-shadow`,
  matching `pnpm-shadow` and `mpm-shadow`;
  cargo also runs it as `cargo shadow`.
- The toolchain `toolchain.lock.json` pins is the one the install step runs,
  so one cargo writes every lockfile.
- Lockfile writes never discard another writer's data ("never discard user data";
  "Use as few locks as possible."):
  no lock is taken;
  a restore replaces a lockfile only when it still holds what the run produced,
  by atomic rename,
  and a journal
   lets the next run finish a killed one (`journal.rs`).
  `cargo -Zunstable-options update --lockfile-path` does not exist on the pinned nightly (`cargo update --help`,
  2026-11-03),
  so
   resolving into a private lockfile was not possible.
- The launcher test is bounded by a nesting counter in the environment and a 20-second deadline.
  With the spawn guard removed from `cargo_run.rs` it fails within seconds (GFP).
