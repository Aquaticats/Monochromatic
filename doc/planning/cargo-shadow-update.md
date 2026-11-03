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
- An operational failure of the install step (no network, a registry lookup that fails, a cargo error) warns and runs the
  command anyway, with `--locked` added once, before any `--`.
  pnpm:
  "If that install fails,
  for example because the store is read-only or there is no network,
  pnpm prints a warning and runs
   the command anyway" (<https://pnpm.io/settings/build>).
  pnpm can do that because `pnpm run` never rewrites the lockfile;
  cargo does,
  hence `--locked`.
  A frozen check (CI, or the user's own `--locked`) that finds a stale lockfile keeps this behavior too: "CI keeps the frozen
  behavior already recorded (the 2026-10-29 trustLockfile amendment)". A frozen policy refusal stops the command by a delegated
  pick of 2026-11-03 (below).
  Replaces the fail-closed behavior built first,
  and, for a policy refusal of a local install, is replaced in turn by the next two decisions.
- A local (not frozen) install sets a release under `minimumReleaseAge` aside instead of refusing it.
  The owner, 2026-11-03 (question tool, verbatim):
  "Try asking better. pnpm nowadays just warn and runs when a dep could be updated to something published within a day."
  pnpm 12.16.1 resolves to the newest release older than the cutoff
  (`resolving/npm-resolver/src/pickMatchingVersion.ts`, `pickRespectingMinReleaseAge`)
  and the coordinator found no message printed for it in the source.
  The install step that runs before a resolving cargo command therefore moves a version it newly needs, and that is under the
  age limit, to the newest older release in range that passes the policy (the set-aside `cargo update` already does),
  prints one line naming the held-back release and the one it used, and runs cargo.
  Only when no release in range passes does the refusal stand (`minimumReleaseAgeStrict = true`, as pnpm's strict mode).
  The owner can still veto this.
- A committed lockfile that holds a refused release is changed to an older one on a local run, and the command stops only when none fits.
  Both answers are "Set it back, build (Recommended)" (question tool, 2026-11-03).
  Case 1, release age only:
  a merge brings a Cargo.lock holding serde 1.0.230, published 3 hours ago; the limit is one day;
  1.0.229 is nine days old and fits the range; the person runs `cargo build` locally.
  Answer: the shadow changes serde in Cargo.lock from 1.0.230 to 1.0.229, prints one line naming both, and builds.
  If no older release fits the range, it stops.
  Case 2, trust downgrade:
  Cargo.lock holds foo 2.3.1, from an account with no earlier releases, while 2.3.0 came from the crate's trusted CI publisher.
  Answer: the shadow changes foo to 2.3.0 in Cargo.lock, prints one line naming the publisher change, and builds.
  If no earlier release with the earlier trust level fits the range, it stops.
  Both rest on pnpm's effect:
  the lockfile check runs before any download, the refused release never runs, and the script runs on the older copy.
  The questions shown to the owner were scoped to local runs, so CI keeps the frozen behavior already recorded.
  An operational failure keeps warn-and-run-with-`--locked`.
  Which entries are checked follows `trustLockfile`:
  all of them when false (pnpm's default), the ones new since the base revision when true (the repository's setting).
  The set-aside goes through the same transaction and ownership rules as the review findings below, so it never overwrites
  another writer's lockfile.
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
- A frozen install verifies per `trustLockfile`,
  and there is no `verify` command (correction of the first reading,
  which verified
   every locked entry and contradicted the owner's decision of 2026-10-29 in `doc/decision/rust-dependency-catalog.md`:
  "the CI
   check covers only registry versions that no `Cargo.lock` held at a base revision ... the counterpart of pnpm's
   `trustLockfile: true`").
  pnpm:
  `pnpm11/installing/deps-installer/src/install/mutateModules.ts:314-343` (`startLockfileVerification`) re-validates every entry
   of the lockfile against the resolver policies on install,
  skipped only when `opts.trustLockfile` is set;
   `verifyLockfileResolutionsCache.ts` caches a passing lockfile by hash;
  `pnpm11/workspace/state/src/validation.ts:428` has
   `trustLockfile: false` as the default;
  `pnpm11/installing/deps-installer/test/install/trustLockfile.ts:21-52` shows a frozen
   install rejecting a refused entry and `trustLockfile: true` skipping it.
  Here `trustLockfile` is a catalog setting (`package/dependency-catalog/core-native`,
  0.2.0,
  and the TypeScript reader
   `core-rust`),
  true in `rust-catalog.toml`.
  True:
  only versions new since the base revision (`CARGO_SHADOW_TRUST_BASE`,
  `HEAD` by default;
  CI sets the pushed range's previous
   commit or the pull request's base),
  checked for trust.
  False:
  every locked entry,
  remembered by lockfile digest.
  A non-frozen install and `cargo update` check what they newly resolve in both cases.
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

## Review findings fixed (cross-vendor review of 2026-11-03T13:26:08Z)

The review ("Do not rely on this implementation as a policy-enforcement boundary yet.") was a static reading of the diff;
each finding was reproduced with a failing test before it was fixed (the commits are on `feat/cargo-shadow-cli`).
What changed, and what each rests on:

- Linux only:
  the owner's "Ignore Win and mac until I say so."
  The journal's `/proc` liveness probe is valid only there, so the crate refuses to compile elsewhere (`lib.rs`).
- Refused lockfile copies get collision-free names,
  recovery errors keep the journal,
  and unknown ownership never authorizes a restore:
  "never discard user data" and "Fix even pre-existing issues of the pkgs you own."
  Each lockfile a run writes is recorded as the run's own output right after its cargo call returned,
  and a restore replaces only that.
- Unknown or malformed timestamps never grant an age exemption (`trustPolicyIgnoreAfter`),
  and the owner list is asked whenever it can change a trust verdict.
- Unsupported registries are refused by name, because registry identity is not carried end to end.
- The freshness stamp certifies manifest consistency only;
  the policy check has its own remembered pass keyed by lockfile text, settings, mode and base spelling.
  Measured cost of the fast path was not repeated after this change.
- A local install checks existing entries (pnpm's `startLockfileVerification` is not gated on frozen).
- `--breaking` edits the manifests, so they are part of the transaction.
  The catalog write and the `file-enforcer` sync are inside the transaction by a delegated pick of 2026-11-03 (below).
- Update arguments are read in every supported spelling, before and after the command word, and an unmapped option is refused.
- The CSV reader refuses malformed tables and bounds fields, rows and histories.
- A full update whose strict cutoff cannot resolve a crate that `minimumReleaseAgeExclude` exempts resolves without the cutoff and
  then holds the other crates to it.
  This is the one shape cargo's `--publish-time` (no per-crate form) allows, and it is open to the owner's veto.
- Set-aside candidates are bounded by what cargo accepts for the manifests, not by the caret class,
  honor the exemptions and the strictness, and are judged with the owner list.

## Delegated picks of 2026-11-03

The owner is away for about 8 hours and delegated the open questions of the review work:
"Pick options as if I had picked them and invite veto when I say I'm back."
These are the coordinator's picks under that delegation, not owner answers, and the owner's veto is open when they are back.

- A local run compares lockfiles with the upstream branch (`@{upstream}`, falling back to `origin/main`, `origin/master`, `HEAD`)
  under `trustLockfile = true`, so a committed but unpushed merge is new since the base and cases 1 and 2 fire locally.
  CI keeps its base (`HEAD`, or `CARGO_SHADOW_TRUST_BASE`, which wins everywhere).
  A remembered pass is keyed by the resolved commit, not by how the base is spelled.
  Cost: one git start per call under `trustLockfile = true`, a median of 7 ms measured for `git rev-parse` through the `git` on PATH.
- The release age and the trust policy both apply to versions new since the base, on the local and the frozen path.
- A policy refusal stops the command on the frozen path too (pnpm's frozen install fails on a rejected lockfile entry;
  `verifyLockfileResolutions.ts` throws before any fetch). A stale lockfile and operational failures keep warn-and-`--locked`.
- The window in which a cargo that is not the shadow builds on an unapproved live lockfile is accepted and documented
  (owner rule "Use as few locks as possible").
- A restore compares and replaces with `renameat2` and `RENAME_EXCHANGE` through `rustix`'s safe wrapper, and undoes the exchange
  when the file it displaced is not the expected one. `rustix` was added to `rust-catalog.toml` (`>=1.1.6`, the version tempfile already
  locks) and to the crate's `Cargo.toml`; the installed `file-enforcer` rejects `trustLockfile` in this catalog, so the generated
  `[workspace.dependencies]` line was written by hand to match what it would render.
- A strict cutoff with exemptions keeps the approach recorded above: resolve without the cutoff, enforce it on the other crates.
- A `file-enforcer` sync failure after the catalog write of `--breaking` restores the catalog (and the update is not kept).
- A killed `--breaking` run is recovered for the manifests, which are journaled with the lockfiles.
- The trust-downgrade finding names the new publisher and the earlier release with its publisher (the account id, or the
  trusted-publishing repository) next to the trust levels; the registry reports ids, not account names.

Still open: nothing from the earlier list; the cost of the git start on every call is the one thing to look at again.
