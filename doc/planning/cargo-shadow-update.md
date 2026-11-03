# What `cargo update` does under the cargo shadow

Status: proposal, awaiting the owner's decision.
The shadow (`package/cargo-shadow/cli`) runs the install step before any cargo command that resolves dependencies,
 the counterpart of pnpm's `verifyDepsBeforeRun: install`.
`cargo update` and `cargo generate-lockfile` are different: they are cargo's own lockfile writers,
 so an install step before them would be overwritten by the command that follows.
Today they pass through untouched, and cargo's resolver then picks versions outside the trust policy
 in `rust-catalog.toml` (release age, trust level of the publisher).
pnpm's `update` is pnpm's own command and applies `minimumReleaseAge` and `trustPolicy` while it resolves.

## Facts the options rest on

- `cargo update` has no `--publish-time`; only `cargo -Zunstable-options generate-lockfile --publish-time <cutoff>` does
  (`package/dependency-catalog/cli-rust-update/README.md`, re-probed 2026-10-29).
  The existing `rust:update` therefore re-resolves every lockfile from scratch with a cutoff,
  then checks the trust level of each newly locked version and sets refused versions aside with `cargo update --precise`.
- `cargo update` takes `--workspace`, `-p <spec>`, `--precise <version>`, `--recursive`, `--breaking`, `--dry-run`,
  `--locked`, `--offline`, `--manifest-path`; scripts and people use `-p x --precise y` to pin a version.
- The install step itself runs the real cargo directly, never the shadow, so it is unaffected by any option below.

## Option A: pass `cargo update` through untouched

- Pros: nothing to build; cargo's flags keep their meaning; nothing surprising for tools that call `cargo update`.
- Cons: a plain `cargo update` moves locked versions to anything cargo's resolver likes, including a release published
  minutes ago or by a publisher the trust policy would refuse.
  That is the bypass this work set out to close, left open on the one command whose job is to move versions.
  `rust:verify` in CI would catch it later, after the lockfile is already in a commit.

## Option B: the shadow makes `cargo update` the catalog update

`cargo update` in a catalog repository runs the same procedure as `rust:update` for the workspace that holds the current
 directory: constrained re-resolution, trust check, set-aside, keep only on success.
- Pros: closest to pnpm, where `update` is the policy-aware command; release age is applied during resolution,
  so the result is the newest allowed version instead of a refusal.
- Cons: it replaces the command rather than running it, so cargo's flags (`-p`, `--precise`, `--breaking`, `--dry-run`)
  need an explicit mapping or a refusal each; it re-resolves everything (minutes, network) even for `cargo update -p x`;
  it needs the pinned nightly for `-Z`.

## Option C: run cargo's `update` unchanged, then check and roll back

The shadow snapshots the lockfile, runs the real `cargo update` with the arguments untouched,
 reads which registry versions the lockfile did not hold before, checks them against the release-age and trust policy,
 and restores the snapshot, failing with the findings, when any version is refused.
- Pros: every flag keeps cargo's meaning; the policy is enforced on the result, not on the intent;
  small to build because the install step already has the snapshot, diff and check code.
- Cons: release age cannot steer the resolution, so too-young versions are refused instead of avoided
  (the user then runs `cargo update -p <crate> --precise <older>`, or `rust:update`, which sets them aside automatically);
  one network round trip per newly locked crate after each update.

## Option D: refuse `cargo update` and `cargo generate-lockfile`, name `rust:update`

The shadow exits 1 with a message that points at `mise run rust:update` (and `rust:install`) in a catalog repository.
- Pros: trivial; no bypass possible through the shadow; one way to move versions.
- Cons: breaks every flow that pins with `--precise`; surprising for a command people expect to work;
  the refusal is a rewrite of the user's intent, which the owner has said the Git wrapper must not do.

## Ranking

C > B > D > A.

- C over B: C keeps the user's command and flags exactly as typed ("native emits whatever it wants")
  and still enforces the policy, while B replaces the command and must invent a flag mapping.
- B over D: B at least performs an update and applies release age during resolution; D only redirects.
- D over A: D leaves no bypass; A leaves the original incident's bypass open on the update command itself.

## Questions for the reviewer

- Is there a fifth option, or a flaw in C's roll back (for example a lockfile written by cargo that is partly kept by a
  concurrent process, or a version that passes the check but breaks the build)?
- Does pnpm apply its trust policy before or after it writes the lockfile in `pnpm update`?
