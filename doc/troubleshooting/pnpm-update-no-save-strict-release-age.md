# pnpm 12.4.2 `update -r --no-save` fails with `ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE` without naming the immature pick

## Symptom

From the repository root,
 on 2026-09-22:

```shell
# /var/home/user/Monochromatic
pnpm update -r --no-save
```

```text
Progress: resolved 0, reused 496, downloaded 0, added 0
Error: ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE

  × updating dependencies
  ╰─▶ minimumReleaseAgeStrict cannot be combined with --no-save: approval would require writing to
      minimumReleaseAgeExclude in pnpm-workspace.yaml, which --no-save prevents.
  help: Drop --no-save so the exclude list can be persisted, or set minimumReleaseAgeStrict: false.
```

The error names no package.
The same command had succeeded on earlier days,
 so the failure looked like a pnpm upgrade regression.
It is not:
 see "Root cause".

The trigger is any update that picks at least one version younger than `minimumReleaseAge` (1440 minutes here)
 whose name is not covered by `minimumReleaseAgeExclude`,
 while `minimumReleaseAgeStrict: true` is set.

## Root cause

### Which pick was immature

The immature pick was `@earendil-works/chord@0.87.1`.

`pnpm-workspace.yaml` excluded `@earendil-works/pi-*` from the age gate,
 so `@earendil-works/pi-coding-agent@0.87.1` was eligible for selection on its release day.
That release depends on chord with a caret range floored at the same-day chord release:

```shell
npm view @earendil-works/pi-coding-agent@latest version dependencies --json
# "version": "0.87.1"
# "@earendil-works/chord": "^0.87.1",
```

```shell
npm view @earendil-works/chord time --json
# "0.87.0": "2026-09-21T16:41:30.718Z",
# "0.87.1": "2026-09-22T19:38:00.606Z"
```

The run was at `2026-09-22T21:47Z`,
 about two hours after chord `0.87.1`.
No chord version older than the cutoff satisfies `^0.87.1`,
 so the resolver had to pick an immature chord,
 and chord was not excluded.
The previous pi release follows the same pattern
 (`@earendil-works/pi-coding-agent@0.86.1` depends on `@earendil-works/chord` `^0.86.1`),
 so every pi release makes this command fail until chord is older than the cutoff.
That explains "it always succeeded before":
 earlier runs happened when no non-excluded immature pick existed.

The error does not say this.
The package was identified by rerunning in a throwaway worktree
with `minimumReleaseAgeStrict: false` and without `--no-save`,
 which makes pnpm persist and print the immature picks
 (see "Verification").

### Why strict mode plus `--no-save` refuses

Source is pnpm tag `v12.4.2`,
 commit `9502f3c457717dae3a4ddbf4315a8c4aee16fdb4`.

`pnpm update --no-save` selects the `Forbidden` exclude policy.

`pnpm/crates/package-manager/src/update/install.rs:285`

```rust
fn lockfile_policy(self) -> crate::InstallLockfilePolicy {
    crate::InstallLockfilePolicy {
        // ...
        excludes: if self.version.save {
            PolicyExcludes::Persist
        } else {
            PolicyExcludes::Forbidden
        },
```

The violation handler refuses a strict-mode run that would need an approval it cannot record,
 but only after it has found at least one immature pick.

`pnpm/crates/package-manager/src/minimum_release_age.rs:137`

```rust
let strict = config.resolved_minimum_release_age_strict();
if !strict && policy_excludes != PolicyExcludes::Persist {
    return Ok(());
}
let immature = sorted_immature_violations(violations);
if immature.is_empty() {
    return Ok(());
}
// ...
if policy_excludes == PolicyExcludes::Forbidden {
    return Err(MinimumReleaseAgeError::StrictRequiresSave);
}
```

`StrictRequiresSave` is a unit variant,
 so the immature list computed on the line before it is dropped.

`pnpm/crates/package-manager/src/minimum_release_age.rs:17`

```rust
pub enum MinimumReleaseAgeError {
    #[display(
        "minimumReleaseAgeStrict cannot be combined with --no-save: approval would require writing to minimumReleaseAgeExclude in pnpm-workspace.yaml, which --no-save prevents."
    )]
    #[diagnostic(
        code(ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE),
        // ...
    )]
    StrictRequiresSave,
```

The neighboring non-interactive strict error,
 `NoMatureMatchingVersion`,
 does carry the list via `format_violation_error(&immature)`,
 so the omission is specific to this variant.
The TypeScript CLI has the same gap:
 `pnpm11/installing/commands/src/policyHandlers.ts:175` throws the same code with a fixed message.

### Rejected reading: pnpm upgrade regression

pnpm 12 before PR [pnpm/pnpm#14842][pr-14842]
 refused `update --no-save` under strict mode even when nothing was immature
 ([pnpm/pnpm#14835][issue-14835]).
That regression is fixed in 12.4.2:
 the `immature.is_empty()` early return quoted in "Why strict mode plus `--no-save` refuses" is present,
 and adding the chord exclusion makes the same command pass (see "Verification").
pnpm 11 had the same conditional refusal,
 so a pnpm 11 run would also fail on this day's chord release.

## Verification

Version under test:

```shell
pnpm --version
# 12.4.2
```

All runs used a throwaway worktree of commit `f21a6a73b` at `~/temp/agent/mono-mra-20260922`.

Positive control,
 unchanged config,
 fails:

```shell
pnpm update --recursive --no-save --reporter append-only
# Error: ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE
```

Identify the immature pick with loose mode and saving enabled:

```shell
sed --in-place 's/^minimumReleaseAgeStrict: true$/minimumReleaseAgeStrict: false/' pnpm-workspace.yaml
pnpm update --recursive --reporter append-only
# Added 1 entry to minimumReleaseAgeExclude in pnpm-workspace.yaml
#   (set minimumReleaseAgeStrict to true to gate these updates with a prompt):
#   @earendil-works/chord@0.87.1
```

With `pnpm-workspace.yaml` and `pnpm-lock.yaml` restored from `HEAD`
 and only `'@earendil-works/chord'` added to `minimumReleaseAgeExclude`,
 the original command gets past the age policy:

```shell
pnpm update --recursive --no-save --reporter append-only
# Progress: resolved 0, reused 688, downloaded 0, added 0, done
# [ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies
# ✕ missing peer @luma.gl/webgpu
```

The remaining `ERR_PNPM_PEER_DEP_ISSUES` failure is a separate incident:
 the update moves `package/dependency-catalog/cli-cube` to `@deck.gl/core@9.4.0`,
 which pulls `@luma.gl/gpgpu@9.4.2`,
 whose peer range requires `@luma.gl/webgpu` `~9.4.0`.
`HEAD`'s lockfile has no `@luma.gl/gpgpu`.
Its ESM dist references only `@luma.gl/core`,
 `@luma.gl/engine`,
 and `@luma.gl/shadertools`:

```shell
# node_modules/.pnpm/@luma.gl+gpgpu@9.4.2_*/node_modules/@luma.gl/gpgpu
rg --no-filename --only-matching "[\"']@luma\.gl/[a-z]+[\"']" dist --glob '*.js' | sort | uniq --count
#      31 '@luma.gl/core'
#      78 '@luma.gl/engine'
#       4 '@luma.gl/shadertools'
```

Commit `418f0d829` marks that peer optional through `packageExtensions`
 (`'@luma.gl/gpgpu'` → `peerDependenciesMeta` → `'@luma.gl/webgpu'` → `optional: true`).
With both changes,
 `pnpm update --recursive --no-save` exits 0 in the throwaway worktree.
Tradeoff:
 if a later gpgpu release starts importing `@luma.gl/webgpu`,
 pnpm no longer flags the missing peer
 and the failure moves to runtime in `package/dependency-catalog/cli-cube`.

Patterns,
 for this repository's config
 (`minimumReleaseAge: 1440`,
 `minimumReleaseAgeStrict: true`):

Works:

- `pnpm update -r --no-save` when every picked version is older than the cutoff or matches an exclude.
- `pnpm update -r` in a TTY:
   pnpm prompts to approve immature picks and appends them to `minimumReleaseAgeExclude`.

Fails with `ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE`:

- `pnpm update -r --no-save` when any non-excluded pick is younger than the cutoff,
   such as within a day of a pi release,
   before the chord exclusion.

## Verified workarounds

### Exclude chord alongside pi (superseded by the scope exclusion)

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - '@earendil-works/chord'
  - '@earendil-works/pi-*'
```

Committed as `33cea9de6`.

Tradeoff:
 chord versions skip the age gate entirely,
 the same trust already granted to `@earendil-works/pi-*`.
A compromised chord release would be installable on day one.
Chord is published by the same scope in lockstep with pi,
 so the pi exclusion already implied accepting same-day chord releases;
 the list now states it.

### Exclude the whole publisher scope (applied)

The durable form of the chord exclusion is `'@earendil-works/*'` in place of the chord and `pi-*` entries,
 so the next lockstep sibling Earendil adds cannot reopen this failure.
The user approved it on 2026-09-22;
 committed as `35d60172f`.

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - '@earendil-works/*'
```

Verified in a scratch copy of the workspace manifests and lockfile:
 `pnpm update --recursive --no-save --lockfile-only` exits 0 with the scope glob,
 and the same copy with only `'@earendil-works/pi-*'` fails with `ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE`.

Filing the upstream diagnostic fix is tracked in [Aquaticat/Monochromatic#557][issue-557].

Tradeoff:
 every package in the scope skips the age gate,
 including ones not yet published.
On 2026-09-22 the lockfile's scope members were exactly `chord` and the five `pi-*` packages.

### Run `mise run deps:update` instead of the bare command

`package/deps-update/cli` (commits `d4d52e34e`,
 `02f7448f1`,
 `cfd65c2fa`) runs `pnpm update --recursive --no-save`.
On this refusal it resolves a temporary copy of the workspace manifests in loose mode,
 reads the versions pnpm appends to `minimumReleaseAgeExclude`,
 and reports each one with its publish time,
 its maturity time,
 its direct dependents,
 and both choices (exclude or wait).
It also names any later failure the loose resolution hit,
 such as `ERR_PNPM_PEER_DEP_ISSUES`.

Verified on the throwaway worktree at `f21a6a73b` (pre-fix config):

```text
pnpm update refused 1 version(s) younger than minimumReleaseAge:

@earendil-works/chord@0.87.1
  published: 2026-09-22T19:38:00.606Z
  passes the age gate: 2026-09-23T19:38:00.606Z
  pulled in by: @earendil-works/pi-agent-core@0.87.1, @earendil-works/pi-coding-agent@0.87.1

Choose one:
  - Trust the publisher: add the package name (or its scope glob) to
    minimumReleaseAgeExclude in pnpm-workspace.yaml, then rerun.
  - Wait: rerun after 2026-09-23T19:38:00.606Z.

The update will then stop on a separate failure the scratch resolution also hit:
  [ERR_PNPM_PEER_DEP_ISSUES] Unmet peer dependencies
```

The loose resolution took 1.9s in a measured run
 because `--lockfile-only` skips fetching and linking.

Tradeoffs:
 the diagnosis resolves against the registry a second time,
 so a release published between the two resolutions can appear in the report without having caused the refusal.
Registry authentication is not sent for publish-time lookups.

## What does not work

`pnpm view <name>@<version> time --json` for publish times:
 on pnpm 12.4.2 it returned a `time` map for `@earendil-works/chord` ending at `0.86.0`,
 while `pnpm view @earendil-works/chord versions --json` in the same shell listed `0.87.0` and `0.87.1`.
`deps:update` reads the registry packument directly instead.
The cause was not traced.

Dropping `--no-save` in a non-interactive session:
 strict mode then raises `ERR_PNPM_NO_MATURE_MATCHING_VERSION` instead of prompting
 (`pnpm/crates/package-manager/src/minimum_release_age.rs:158`,
 `if !can_prompt {`).
In a TTY it prompts,
 and approving appends a pinned `name@version` entry,
 which accumulates per release
 (see `doc/troubleshooting/pnpm-minimum-release-age-exclude-first-match.md`
for why accumulated per-version entries misbehave).

`minimumReleaseAgeStrict: false`:
 loose mode with `--no-save` returns early and installs immature picks silently,
 removing the gate for every package rather than chord only.

`PNPM_CONFIG_TRUST_LOCKFILE=true`
 (`doc/troubleshooting/pnpm-minimum-release-age-trust-lockfile.md`):
 it trusts entries already in the lockfile,
 while this failure is a fresh resolution of a version the lockfile does not hold yet.

Downgrading pnpm:
 pnpm 11 raises the same error for the same input (`pnpm11/installing/commands/src/policyHandlers.ts:175`).

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was checked on 2026-09-22;
 no pnpm entry exists
 (files:
 `bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
 `codex-harness.md`,
 `jsr.md`,
 `lightningcss.md`,
 `low-impact-typescript-formatting.md`,
 `module-es-monolith.md`,
 `pi-gpt55-long-context.md`,
 `terminal-title-fork-parity-tests.md`,
 `typescript-project-references.md`).

Duplicate search on 2026-09-22:

```shell
gh search issues --repo pnpm/pnpm "STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE" --limit 10
gh search issues --repo pnpm/pnpm "minimumReleaseAgeStrict no-save" --limit 10
gh search prs --repo pnpm/pnpm "minimumReleaseAgeStrict no-save" --limit 10
gh search issues --repo pnpm/pnpm "minimumReleaseAgeStrict" --limit 10
gh search prs --repo pnpm/pnpm "minimumReleaseAgeStrict" --limit 10
```

The narrow searches returned nothing.
The broad searches returned [pnpm/pnpm#14835][issue-14835] and its fix [pnpm/pnpm#14842][pr-14842]
 (refusal even with nothing immature,
 fixed),
 [pnpm/pnpm#15091][issue-15091] (double prompt in `update -g`),
 and [pnpm/pnpm#11203][issue-11203] (no fallback to an eligible intermediate version).
None asks for the refusal to name the immature picks.

Constraints,
 for the missing package list only
 (the refusal itself is intended behavior):

- Is it really upstream's fault?
   Yes for the diagnostic.
   The handler holds the sorted immature list at the refusal site and drops it,
   while the sibling `NoMatureMatchingVersion` error reports the same list.
- Can upstream fix it?
   Yes;
   the prototype below is a local change to one enum variant.
- Are they supporting this use case?
   Yes.
   `PolicyExcludes::Forbidden` exists for `update --no-save`,
   and `pnpm/crates/cli/tests/suite/update.rs:787` tests this refusal.
- Would the repo welcome our contribution?
   Yes.
   `CONTRIBUTING.md` "AI-assisted contributions" welcomes agent-assisted work
   and requires a footer naming the agent and model,
   a duplicate-PR check,
   and locally passing tests.
   The draft carries that footer.
- Will they likely fix it?
   No contrary signal found;
   the maintainers fixed the adjacent #14835 within a day.
- Have we prototyped a minimal fix compatible with their architecture?
   See "Prototype".

### Prototype

Patch:
 [`pnpm-update-no-save-strict-release-age.patch`](pnpm-update-no-save-strict-release-age.patch),
 against tag `v12.4.2` (commit `9502f3c457717dae3a4ddbf4315a8c4aee16fdb4`),
 prepared in a disposable clone whose `origin` was verified as `https://github.com/pnpm/pnpm.git`.
It turns `StrictRequiresSave` into `StrictRequiresSave { violations: String }`,
 fills it with the existing `format_violation_error(&immature)`,
 appends it to the message,
 and updates the unit test that asserts the message.

It covers the Rust CLI (pnpm 12) only.
The TypeScript CLI under `pnpm11/` has the same gap and is described in the draft,
 not prototyped.

Verification ran in a secret-free container with no repository mounts other than the clone
 (the clone's `.cargo/config.toml` vendored-source block was removed for the run and is not part of the patch):

```shell
podman run --memory=2g --cpus=2 --rm --volume "$PWD:/work:Z" --workdir /work/pnpm \
  docker.io/library/rust:1.97.0 \
  cargo test --package pnpm-package-manager --lib minimum_release_age
```

Patched:

```text
test minimum_release_age::tests::strict_no_save_is_rejected_only_once_a_pick_is_immature ... ok
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 687 filtered out
```

Control,
 patched test against unpatched `minimum_release_age.rs`:

```text
test minimum_release_age::tests::strict_no_save_is_rejected_only_once_a_pick_is_immature ... FAILED
  left: "minimumReleaseAgeStrict cannot be combined with --no-save: ... which --no-save prevents."
 right: "minimumReleaseAgeStrict cannot be combined with --no-save: ... which --no-save prevents.\n1 version does not meet the minimumReleaseAge constraint:\n  foo@1.0.0 foo@1.0.0 is too new"
test result: FAILED. 17 passed; 1 failed
```

All six constraints hold for pnpm 12,
 so the draft is fileable.
It has not been filed;
 filing is an outward-facing action that waits for the user's go-ahead.

### Draft issue

~~~md
Title: `ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE` does not name the immature picks

Labels: area: supply chain security

With `minimumReleaseAgeStrict: true`, `pnpm update -r --no-save` refuses when a
pick is younger than `minimumReleaseAge`, which is correct since #14842. But the
error does not say which package was immature:

```
Error: ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE
  × updating dependencies
  ╰─▶ minimumReleaseAgeStrict cannot be combined with --no-save: approval would
      require writing to minimumReleaseAgeExclude in pnpm-workspace.yaml, which
      --no-save prevents.
```

In a large workspace the culprit can be a transitive dependency that an excluded
package drags in (for us: `@earendil-works/pi-*` is excluded, and each release
requires the same-day `@earendil-works/chord`). The only way to find it was to
rerun in loose mode without `--no-save` in a scratch checkout.

The list is already computed at the refusal site and then dropped
(`pnpm/crates/package-manager/src/minimum_release_age.rs`, v12.4.2):

```rust
let immature = sorted_immature_violations(violations);
// ...
if policy_excludes == PolicyExcludes::Forbidden {
    return Err(MinimumReleaseAgeError::StrictRequiresSave);
}
```

`NoMatureMatchingVersion` reports the same list via `format_violation_error`.

Reproduction: `pnpm/crates/cli/tests/suite/update.rs`
`update_no_save_is_refused_when_a_pick_is_immature` produces the refusal; its
stderr does not contain the immature package name.

Suggested fix: make `StrictRequiresSave` carry `format_violation_error(&immature)`
and append it to the message (patch below, tested with
`cargo test --package pnpm-package-manager --lib minimum_release_age`). The same
gap exists in `pnpm11/installing/commands/src/policyHandlers.ts`, which could
reuse the list built by `failOnImmature`.

```diff
--- a/pnpm/crates/package-manager/src/minimum_release_age.rs
+++ b/pnpm/crates/package-manager/src/minimum_release_age.rs
@@ -17,15 +17,15 @@
     #[display(
-        "minimumReleaseAgeStrict cannot be combined with --no-save: approval would require writing
  to minimumReleaseAgeExclude in pnpm-workspace.yaml, which --no-save prevents."
+        "minimumReleaseAgeStrict cannot be combined with --no-save: approval would require writing
  to minimumReleaseAgeExclude in pnpm-workspace.yaml, which --no-save prevents.\n{violations}"
     )]
     #[diagnostic(
         code(ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE),
         help(
-            "Drop --no-save so the exclude list can be persisted, or set minimumReleaseAgeStrict: false."
+            "Drop --no-save so the exclude list can be persisted, add these picks to minimumReleaseAgeExclude,
  or set minimumReleaseAgeStrict: false."
         )
     )]
-    StrictRequiresSave,
+    StrictRequiresSave { violations: String },
@@ -152,7 +152,9 @@
     if policy_excludes == PolicyExcludes::Forbidden {
-        return Err(MinimumReleaseAgeError::StrictRequiresSave);
+        return Err(MinimumReleaseAgeError::StrictRequiresSave {
+            violations: format_violation_error(&immature),
+        });
     }
```

The unit test `strict_no_save_is_rejected_only_once_a_pick_is_immature` is updated to match
(fails before the change, passes after: 18 of 18 in `minimum_release_age`).

Written by an agent (Claude Code, claude-opus-5-5).
~~~

[issue-11203]: https://github.com/pnpm/pnpm/issues/11203
[issue-14835]: https://github.com/pnpm/pnpm/issues/14835
[issue-15091]: https://github.com/pnpm/pnpm/issues/15091
[pr-14842]: https://github.com/pnpm/pnpm/pull/14842
[issue-557]: https://github.com/Aquaticat/Monochromatic/issues/557
