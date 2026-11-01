# Changesets release tooling technology vet

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`.

- Status:
   recommended;
   adoption separately authorized by the owner on 2026-09-06
- Lifecycle phase:
   recommended
- Subject:
   changesets release tooling
- Scope:
   versioning and npm publishing tool for the Monochromatic pnpm workspace,
   first applied to `@monochromatic-dev/module-logger`
- Start date:
   2026-09-06
- Last updated:
   2026-09-06
- Governing skill commit:
   `a05818ad70a40e5769a36de669697ba109891b31`
- Governing skill SHA-256:
   `393eb68e2c7b5b16c8f7f90c100fb8be43eefa4501511360cd0572e4ae8087`
- Compatibility fingerprint:
   `9f3afc555940104cb6761c44be419f122f0f18c5037b8a7e1bc0776c6145b1e1`
- Active audit owner:
   current Claude Code session (`CLAUDE_CODE_SESSION_ID` set;
   value not recorded)
- Prior compatible report:
   none found (`doc/audit/` has no changesets,
   release-tooling,
   or versioning report)

## Context

The workspace has never published a package to npm.
Issue #159 asks for release versioning and publishing automation and names changesets as the recommended tool.
The owner stated during the module-logger release grilling
(`doc/planning/module-logger-release.md`)
that changesets was their standing intent.
This audit vets that single candidate against the skill's hard gates,
audits its source,
and validates it in a throwaway workspace.

## Scope correction

The owner instructed on 2026-09-06:
"If changeset fails then we do it the hard way.
 No need to research other candidates."
Alternatives discovery,
the query schedule,
and comparative scoring are therefore waived by owner instruction and recorded as such.
The skill's alternative-count invariant is not satisfied,
and this report says so rather than inventing alternatives.
If changesets fails a hard gate,
the terminal result is "no survivor" and the fallback is a hand-run publish
(the "hard way" the owner named),
not a different tool.

Measured repository context:

- No `.changeset/` directory,
   no `@changesets/*` dependency,
   and no changesets mention in `pnpm-workspace.yaml`,
   `package.json`,
   or `mise.toml`.
- `.github/workflows/publish.yml` publishes a hardcoded package list
   with a stored `NPM_TOKEN` that does not exist (`gh secret list`),
   has never run,
   and has no build step (issues #306,
   #307).
- pnpm 11.21.0 on the host;
   the workflow installs `pnpm = "latest"` through mise.
   pnpm has supported OIDC trusted publishing since 10.21.0.
- The owner decided that the first publish of any package is local and interactive,
   because `npm trust` requires the package to already exist on the registry.

## Classification

- Base category:
   inspectable open-source local technology (`@changesets/cli` and the `changesets/action` GitHub Action).
- High-trust overlay:
   applicable.
   The action and CLI run inside the CI publish credential boundary and drive `pnpm publish` under an OIDC id-token.
- Native,
   Wasm,
   and prebuilt overlay:
   applicable to the action only,
   whose release tags ship built `dist/*.js`.
- Replacement overlay:
   not applicable;
   no incumbent tool exists.
- Managed-service gates:
   not applicable;
   npm itself is outside this audit's subject.
- Sensitive-data overlay:
   not applicable.
- Multi-platform overlay:
   not applicable;
   the tool runs on Linux CI runners and the maintainer's Linux host.

## Hard constraints

- Inspectable open-source source for every executed component.
- Rewrites pnpm `workspace:` specifiers on publish (delegated to pnpm).
- Supports npm trusted publishing (OIDC) from GitHub Actions with no stored long-lived npm token.
- Independent per-package semver versions.
- Runs on the host and CI runtimes:
   Node 26,
   pnpm 11.

## Frozen score criteria

Single-candidate scoring cannot change an ordering,
so criteria are recorded for the sensitivity record only.
Each criterion has default weight 1:

- pnpm workspace fit.
- Trusted publishing fit.
- Maintenance activity.
- Operational surface.
- Source auditability.

## Query schedule

Waived by owner instruction (see "Scope correction").
No registry,
GitHub,
or aggregator queries were run for alternatives.

## Candidate ledger

- `@changesets/cli` 3.0.2 with `changesets/action` v2.1.1:
   discovered from issue #159 and the owner's stated intent;
   screened;
   hard-gate confirmed;
   finalist;
   validated in a throwaway workspace;
   recommended.

## Evidence records

Access date for every URL:
 2026-09-06.
Host for every command:
 Linux x86_64,
 Node v26.7.0,
 pnpm 11.21.0,
 no container.

### E1 License (hard gate, pass)

- `@changesets/cli@3.0.2`:
   `npm view @changesets/cli license` returns `MIT`.
- Repository `changesets/changesets`:
   GitHub API `license.spdx_id` is `MIT`,
   `archived: false`.
- Repository `changesets/action`:
   GitHub API `license.spdx_id` is `MIT`,
   `archived: false`.

### E2 Source availability and inspectability (hard gate, pass)

- Clone:
   `gh repo clone changesets/changesets "${HOME}/temp/agent/changesets-2026-09-06" -- --depth 1`,
   HEAD `d13a39e9126a3a77383788a4f3247b5385cc0e03`,
   `packages/cli/package.json` version `3.0.2`.
- Clone:
   `gh repo clone changesets/action "${HOME}/temp/agent/changesets-action-2026-09-06" -- --depth 1`,
   HEAD `e08fde7435de7db82496e0d3b7207e879b514117`.

### E3 Build provenance of the npm package (hard gate, pass)

- `npm view @changesets/cli dist.attestations --json` returns an attestations URL with
   `provenance.predicateType` `https://slsa.dev/provenance/v1`.

### E4 Build provenance of the action's prebuilt output (hard gate, pass with note)

- `action.yml` sub-actions run `../dist/*.js` (`pack/action.yml`,
   `publish/action.yml`).
- The `main` clone has no `dist/` directory;
   the v2.1.1 tag (commit `8488615a623b1b9c987934bb89eae8af6a946ac1`,
   dereferenced from tag object `fdf536a68c4154480c89b42547f8102cf0d8bc47`) contains
   `dist/index.js`,
   `dist/pack.js`,
   `dist/publish.js`,
   `dist/pr-comment.js`,
   `dist/pr-status.js`,
   and two chunk files (GitHub contents API).
- The action's own `.github/workflows/publish.yml:31-32` runs `pnpm build` (`rolldown -c`,
   `package.json` scripts) before tagging.
- Note:
   this audit did not rebuild `dist/` and diff it against the tag;
   the mapping rests on the publish workflow reading.
   Pinning the workflow to the commit SHA `8488615a623b1b9c987934bb89eae8af6a946ac1` freezes the audited bytes.

### E5 Security disclosure and advisories (hard gate, pass; low-signal)

- GitHub advisory database:
   `gh api '/advisories?affects=@changesets/cli'` returns no entries;
   the same query for `@changesets/apply-release-plan` returns none.
- `changesets/changesets` community profile reports no security policy file.
   Low-signal per the skill (a sparse tracker is not a failure);
   recorded as a scored concern.

### E6 Platform support (hard gate, pass)

- `npm view @changesets/cli engines`:
   `node ^22.11 || ^24 || >=26`,
   `pnpm >=10.0.0`,
   `npm >=10.9.0`,
   `yarn >=4.5.2`.
- Host Node v26.7.0 and pnpm 11.21.0 satisfy both.

### E7 Maintenance (scored)

- `changesets/changesets`:
   `pushed_at` 2026-09-06T16:09:35Z,
   261 open issues,
   latest releases 2026-09-04 (`@changesets/release-utils@1.0.1`,
   `@changesets/read@1.0.1`,
   `@changesets/git@4.0.1`).
- `changesets/action`:
   `pushed_at` 2026-09-04T18:13:04Z,
   60 open issues,
   latest release v2.1.1 on 2026-08-19.
- npm maintainers of `@changesets/cli`:
   two accounts.

### E8 Publish delegation to pnpm (hard-constraint evidence, pass)

- `packages/cli/src/commands/publish/getPublishTool.ts:20-38`:
   selects the pnpm publish tool when the workspace tool is pnpm,
   otherwise detects the package manager.
- `packages/cli/src/lib/pnpm.ts:149-181`:
   runs `pnpm publish --access <access> --tag <tag> --no-git-checks`,
   prepends `--json` when non-interactive,
   prepends a tarball path when publishing from a pack directory,
   and strips OTP environment variables.
   No provenance flag is passed;
   provenance therefore comes from each manifest's `publishConfig.provenance` and pnpm's OIDC path.
- `packages/cli/src/lib/pnpm.ts:124-139`:
   packs with `pnpm pack --out <path> --json`,
   so pnpm performs the `workspace:` rewrite and `publishConfig` overrides.
- `packages/cli/src/lib/pnpm.ts:213-222`:
   maps `ERR_PNPM_OTP_NON_INTERACTIVE` to a `failed:needs-2fa` result,
   which is the expected failure when a package has no trusted publisher yet.

### E9 Trusted publishing support in the action (hard-constraint evidence, pass)

- `README.md:20` recommends the individual sub-actions for trusted publishing;
   `README.md:28` and `publish/README.md` require `id-token: write`.
- `src/run.ts:160-215`:
   the publish step sets `CHANGESETS_OUTPUT` and runs `changeset publish [--from-pack-dir]` or a custom script;
   a custom script must forward that output file or tags and releases are skipped with a warning.
- Sub-actions:
   `select-mode` (outputs `mode` and a publish-plan artifact),
   `version`,
   `pack` (outputs a pack-dir artifact),
   `publish` (inputs `pack-dir-artifact-id`,
   `script`).

### E10 Throwaway validation (finalist validation, pass)

- Working directory:
   `/tmp/tmp.evyU0Ap6pM`,
   a fresh `git init` pnpm workspace with `@vetfixture/leaf` (public,
   `publishConfig.exports` override,
   `./ts` in `exports`) and private `@vetfixture/app` depending on `@vetfixture/leaf` via `workspace:*`.
- `pnpm dlx @changesets/cli@3.0.2 init` prompts interactively ("Should the GitHub integration be used for changelogs?")
   and exits 13 without a TTY;
   `init --help` exposes no non-interactive flag.
   `.changeset/config.json` was written by hand instead.
- `pnpm dlx @changesets/cli@3.0.2 status --verbose`:
   exit 0,
   "@vetfixture/leaf -> 0.1.0" from `.changeset/first.md` (`minor`).
- `pnpm dlx @changesets/cli@3.0.2 version`:
   exit 0;
   `packages/leaf/package.json` version `0.1.0`;
   `packages/leaf/CHANGELOG.md` created with the changeset text;
   the changeset file consumed;
   private `@vetfixture/app` kept `"@vetfixture/leaf": "workspace:*"` and version `0.0.1`.
- `pnpm publish -r --dry-run --no-git-checks`:
   exit 0,
   "Skip publishing @vetfixture/leaf@0.0.1 (dry run)" before versioning.
- `pnpm pack` of the leaf and `tar -xOzf … package/package.json`:
   `exports` collapsed to the `publishConfig.exports` override (the `./ts` entry removed),
   confirming the pnpm override path changesets relies on.
- `pnpm add -Dw @changesets/cli@3.0.2` then `pnpm ls --depth Infinity`:
   82 transitive packages installed for the CLI.
- Elapsed:
   each `pnpm dlx` invocation completed in seconds;
   not timed precisely.

### E11 Deferred consumer-boundary check (recorded)

- An OIDC publish cannot be reproduced outside CI and cannot run before a package exists on the registry.
   The first CI publish of a later version (0.1.1 or the next changeset) is the live check.
   Until then the trusted-publishing claim rests on E8,
   E9,
   and the pnpm release notes for 10.21.0.

### E12 Alternatives (excluded by owner instruction)

- release-please,
   semantic-release,
   and manual tag-triggered workflows were not researched.
   Owner instruction recorded in "Scope correction".

## Hard-gate outcomes

- Category mismatch:
   pass (E8,
   E9).
- Hard-constraint failure:
   pass (E6,
   E8,
   E9,
   E10).
- License:
   pass (E1).
- Required source unavailable:
   pass (E2).
- Proprietary or uninspectable high-trust code:
   pass (E2,
   E4).
- Prebuilt provenance unmapped:
   pass with note (E3,
   E4).
- Build provenance unknown:
   pass (E3,
   E4).
- Security boundary cannot be established:
   pass (E5,
   E9;
   `id-token: write` scoped to the publish job).
- Reproducible validation has no inspectable path:
   pass (E10),
   with the OIDC publish deferred (E11).
- Unsupported platform:
   pass (E6).

## Scores and sensitivity

Single finalist;
 ratings recorded for the record,
 confidence in parentheses:

- pnpm workspace fit:
   5 of 5 (high;
   E8,
   E10).
- Trusted publishing fit:
   4 of 5 (medium;
   E9 by source reading,
   E11 deferred).
- Maintenance activity:
   4 of 5 (high;
   E7,
   with a large open-issue count).
- Operational surface:
   3 of 5 (high;
   82 transitive packages,
   interactive `init`,
   E10).
- Source auditability:
   5 of 5 (high;
   E2,
   E8).

Sensitivity:
 no weight change can alter a one-candidate ordering.

## Pros and cons

Pros:

- Delegates packing and publishing to pnpm,
   so `workspace:` rewrite and `publishConfig` overrides come from the package manager already in use.
- Trusted publishing works through sub-actions with `id-token: write` scoped to the publish job.
- Independent per-package versions and changelogs;
   private packages are neither versioned nor tagged by default.
- Active maintenance on both repositories within the audit week.

Cons:

- 82 transitive packages for a release tool.
- `changeset init` is interactive with no flag;
   configuration is written by hand.
- No security policy file;
   the open-issue count is large.
- The live OIDC publish is unverifiable before the first registry entry exists.

## Ranking

One candidate;
 no adjacent ordering to justify.

## Recommendation

Adopt `@changesets/cli` 3.0.2 with `changesets/action` pinned to commit
`8488615a623b1b9c987934bb89eae8af6a946ac1` (v2.1.1),
using the `select-mode`,
`version`,
`pack`,
and `publish` sub-actions so that only the publish job holds `id-token: write`.
Write `.changeset/config.json` by hand with `access: public` and `baseBranch: main`.
Keep the owner's fallback:
 if the first CI publish fails at the trusted-publishing boundary,
 publish by hand from a pnpm-packed tarball.
The decision record is `doc/decision/npm-publishing.md`.
