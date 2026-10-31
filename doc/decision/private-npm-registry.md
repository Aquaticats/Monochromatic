# Private npm registry for workspace packages

## Status

Accepted 2026-09-14.
Owner-authorized during the grilling recorded in `doc/planning/private-npm-registry.md`,
 which keeps the measured facts,
 rejected options,
 and the order in which answers changed.
Extends `doc/decision/npm-publishing.md`;
 public npmjs releases through changesets are unchanged.

## Context

Another owner project,
 `Aquaticat/labwc-config`,
 needs `oxlint-plugin-tsdoc`,
 `config-typescript`,
 and `module-or-throw`,
 which the owner judges below public-release quality.
The owner widened the goal to every workspace package.
On 2026-09-14 the workspace held 154 packages under `package/`,
 120 of them `"private": true`,
 and only `module-logger` and `module-fs-path` release to npmjs.

## Decisions

### Registry: pnpr on the Coolify host

- pnpr from the owner's fork `Aquaticat/pnpm`,
   branch `pnpr-oidc-workload-discovery`,
   whose `.github/workflows/pnpr-patched-image.yml` publishes static musl binaries as a GitHub release
   and a multi-arch image at `ghcr.io/aquaticat/pnpr` on every push.
   `package/config/pnpr/Containerfile` builds on that image pinned by digest.
   Owner decision (2026-09-15),
    after `@pnpm/pnpr@0.1.0-alpha.11` answered every GitHub Actions workload publish with HTTP 500
    (`doc/troubleshooting/pnpr-oidc-github-actions-discovery.md`)
    and upstream `main` had no fix.
   It supersedes installing `@pnpm/pnpr` at npm dist-tag `next`.
   Rejected:
    a temporary stored publish token (reverses the OIDC decision),
    waiting for upstream (the registry stays empty),
    and compiling the patch on the Coolify host at every rebuild.
   Return to upstream releases once one verifies GitHub workload tokens.
- Served at `https://pnpr.c.aquati.cat/`,
   behind the owner's self-managed Caddy.
- One hosted npm registry named `monochromatic-dev` claims `@monochromatic-dev/*`.
   There is no npmjs upstream,
   and the install-accelerator surface is disabled (`--disable-resolver`),
   because it fetches upstream indexes on clients' behalf.
- Reads are anonymous;
   publishes need authentication.
- Storage is a Coolify volume with no backup;
   losing historical versions is acceptable,
   and recovery republishes current versions.

### Consumers route only the scope

```ini
# consumer .npmrc
@monochromatic-dev:registry=https://pnpr.c.aquati.cat/~monochromatic-dev/
```

Moved 2026-10-21:
 the route is `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/`
 from the `0.1.0-alpha.16` base,
 which mounts npm under `/npm`
 (`doc/troubleshooting/pnpr-base-move.md`);
 the block above records the `0.1.0-alpha.11` root surface.

Through this route,
 npmjs-only versions and `@monochromatic-dev/mcp-nvim` are not installable.
pnpm 11 and later delay fresh versions by `minimumReleaseAge`,
 so consumers add `@monochromatic-dev/*` to `minimumReleaseAgeExclude`.

### Published set

- Every workspace package except those under `package/test-fixture/`,
   private ones included.
   Selection lives in `generatePnprConfig` in `file-enforcer.config.ts`
   (moved 2026-10-30 to `package/config/pnpr/src/generated-config-selection.ts`, which the root file now calls);
   the publish workflow reads the generated name list instead of re-deriving it.
- A package publishes only if its manifest still declares `exports`,
   `main`,
   `module`,
   or `bin` after `./ts` export subpaths are stripped;
   126 of 155 workspace packages qualified on 2026-09-14
   (`package/config/pnpr/config.yaml`),
   which drops `config-tofu` (only `./ts/*` exports)
   and `config-pnpr` (no entry points).
- `module-logger` and `module-fs-path` also publish here,
   packed separately from the npmjs release,
   so the same version's tarball bytes can differ between registries.
- A reviewed exclusion list silences packages that cannot build on GitHub-hosted runners.
   Since `pnpr-publish` run 34917976384 (2026-09-15) it holds
    `desktop-daemon-hall-monitor` (its build runs `bun build --compile`,
     and the job installs only node and pnpm)
    and `webapp-productivity-doodle-widget` (rolldown cannot resolve jspdf's optional `canvg` import);
   the generated config then lists 121 packages,
    all of which were published that day.
   Packages lacking a `version` field are skipped too.
- Owner decision (2026-09-15):
   a package is meant to publish when it exposes something others can import.
   Apps exposing nothing importable,
    including both excluded packages,
    do not count.
   Blocked importable packages each get an issue and the minimum manifest change:
    `cli-markdown-lint` (#522),
    `config-oxlint` (#523),
    and `config-stylelint` (#524) gained a `version`,
    `config-stylelint` also ships the modules `index.mjs` imports,
    and `module-dom` (#525) exports its built bundle;
   the config then lists 125 packages.
   Classification of the other unlisted packages:
    `doc/handover/pnpr-publish-remaining-packages.md`.
- Since #537,
   selection ignores `main` and `module` when `exports` exists,
   and ignores `main`,
   `module`,
   and bin targets that are TypeScript source;
   ten packages whose only entries were such paths left the config (115 of 155 on 2026-09-15).
   Their already published versions stay on pnpr.
- #521 verified installability on 2026-09-15:
   every published package installed alone from pnpr into a `node:26-slim` npm consumer,
   with every declared file present,
   Node-resolvable exports imported,
   bare imports resolved,
   typed exports type-checked under `nodenext`,
   and every bin run with `--help`.
   Findings and fixes are listed in the handover.
- Owner decision (2026-09-15, #543):
   the hook plugins bundle `claude-code-plugin-source`,
   so they list it as a development dependency,
   and it is excluded from pnpr because it exports only TypeScript source;
   the config lists 114 of 155 packages.

### Tarball shape

- Packed with pnpm in a clean CI checkout holding no secrets.
- `"private": true` is removed from the packed manifest.
- `./ts` export subpaths are removed,
   the same rule as `doc/decision/npm-publishing.md`.
- `main`,
   `module`,
   and `bin` entries naming TypeScript source are removed
   (#537,
    #540),
   because Node refuses to load TypeScript from `node_modules`;
   `bin` disappears when no loadable entry remains.

### Versions change only by manual bumps

- The publish workflow publishes every version pnpr does not have;
   a published version is never overwritten.
- `latest` moves only to a greater version.
- A cli-git commit policy bumps dependents by patch,
   transitively,
   in the same commit as a hand bump.
   An edge counts when the dependent reaches the bumped package through `dependencies`,
    `peerDependencies`,
    or `optionalDependencies`,
   or imports it from non-test source as a bundled `devDependency`.
   cli-git's commit transaction cannot do this today:
    it patches only paths already staged in the commit
    (`package/git-policy/cli/src/policy-engine/apply-policy-patches.ts` lines 99 to 112 reject other targets),
    and its fixes never reach the worktree.
   Owner decision (2026-09-15):
    extend the engine so a policy pass can add paths to the commit
    and write those fixes to the worktree when the worktree copy still matches `HEAD`,
    with SPEC,
    `doc/decision/cli-git-policies-platform.md`,
    and trust fixtures updated for every commit mode.
   Rejected:
    a read-only check plus a `version:bump` command (two steps when forgotten),
    and adding paths without updating the worktree (leaves reverted versions in `git status`).
- The changesets `version` job in `npm-release.yml` runs the same ripple
  before committing the Version Packages pull request.
   Root task `changeset:version` runs `changeset version`,
    then `mise run //package/git-policy/repository:bump:dependents`,
    then `pnpm install --lockfile-only`.
   The runner (`package/git-policy/repository/src/bump-dependents.ts`)
    compares worktree manifests with `HEAD`
    and shares its planning module (`dependent-bump-workflow.ts`) with the cli-git policy,
    so both apply the same edge and publishability rules.
   It imports only Node built-ins and sibling source files,
    so the job needs no build step.
   Evidence (2026-09-15, disposable clone of `main`):
    hand-bumping `@monochromatic-dev/or-throw-core` and running the runner planned 19 dependent bumps,
    and `git diff --shortstat` reported `20 files changed, 20 insertions(+), 20 deletions(-)`,
    version lines only.
- No CI backstop catches commits that bypass cli-git.

### Publishing workflow

- A dedicated GitHub Actions workflow on `main`,
   filtered to `**/package.json`,
    the generated pnpr config,
    and the workflow file,
   with `workflow_dispatch` for retries.
- Authentication is pnpr's OIDC workload credential;
   no token is stored.
   pnpr accepts only exact package names for OIDC trust
   (`pnpr/crates/pnpr/src/server/oidc.rs` at pnpm `f38f11e`),
   so the trust config lists every published name.
- The version-missing check reads anonymously,
   because the workload credential cannot read.
- Packages publish in dependency order;
   a build failure blocks that package and its dependents,
   the rest still publish,
   and the run fails naming the package.

### Configuration and deployment

- file-enforcer generates the pnpr config,
   including the package-name list and the exclusion list
   (since 2026-10-30 the generator and its check live in `package/config/pnpr`,
   per `doc/decision/file-enforcer-rust-rewrite.md`, "Elsewhere").
- The deployment lives in `package/config/pnpr/`
   and Coolify redeploys it on push through a GitHub App source,
   limited by Watch Paths to `config.yaml`,
    `Containerfile`,
    and `compose.yaml` under `package/config/pnpr/`,
   so edits to the publish script beside them do not redeploy.
   A public-repository resource was tried first and has no Watch Paths field:
    Coolify renders it only for `is_github_based() && !is_public_repository()`.
   Without Watch Paths every push would redeploy:
    Coolify deploys when `isWatchPathsTriggered($changed_files) || blank($application->watch_paths)`
    (`app/Http/Controllers/Webhook/Github.php` lines 136 to 137 in coollabsio/coolify),
    and `main` received 1492 commits in the 30 days before 2026-09-14,
    of which 7 added or removed a package manifest.
   A new package's first publish can fail until the redeploy lands;
   a later qualifying run retries it.
- The Coolify build runs no package manager:
   the patched base image already carries `ca-certificates`,
   installed by upstream's `pnpr/docker/Dockerfile` in the fork's GitHub Actions run.
   Before that image,
    the `node:24-slim` build could not apt-install `ca-certificates` on the Coolify host
    (`apt-get` exited with code 100 before and after
     `deb.debian.org` was added to the port-80 egress allowlist in `package/config/tofu/hetzner.tf` on 2026-09-15;
     the cause is unconfirmed)
    and wrote Node's bundled root certificates instead.
   The allowlist entry stays.
- The owner performs the Njalla A record,
   Coolify resource,
   and Caddy site block from `doc/runbook/deploy-pnpr-registry.md`.

### Packaging fix

`oxlint-plugin-tsdoc` moves its bundled workspace packages,
 `oxlint-plugin-shared` and `ownership-marker-foreign-borrowed`,
 to `devDependencies`.

### Verification

From a throwaway consumer:

- Anonymous install and use of `oxlint-plugin-tsdoc` (loaded by oxlint),
   `config-typescript` (extended by `tsc`),
   and `module-or-throw` (called).
- Install of `module-logger` from pnpr.
- A publish without the OIDC credential is rejected,
   and the workflow's OIDC publish succeeds.

Results on 2026-09-15 against `https://pnpr.c.aquati.cat`:

- `pnpr-publish` run 34917839821 published `config-typescript@0.0.5`,
   `module-or-throw@0.0.1`,
   and `oxlint-plugin-tsdoc@0.0.1` with the OIDC workload credential.
- The throwaway consumer installed all three with the lockfile resolving the scope from pnpr;
   `nonNullishOrThrow(42)` returned `42` and threw on `null`,
   `tsc` reported `TS2322` under the extended config,
   and oxlint reported the `tsdoc` rule.
- `@monochromatic-dev/logger-core@0.4.0` installed from pnpr and logged through `tagged`.
- An unauthenticated `PUT` returned
   `Authentication required for package "@monochromatic-dev/or-throw-core" 401`,
   and a forged GitHub-issuer token returned `401 Authentication required for OIDC credentials`.

GitHub issue #521 tracks checking installability of the remaining published packages.

## Consequences

- A hand bump rewrites,
   in the same commit,
   the manifest of every dependent reached through runtime or bundled edges.
- pnpr is alpha software pinned by image digest,
   so upstream changes arrive only when the fork branch is rebased,
   its workflow publishes a new image,
   and the `Containerfile` digest moves.
- Each consumer adds the scope route itself;
   nothing here wires `labwc-config`.
- A push that adds names to `config.yaml` starts the Coolify redeploy and `pnpr-publish` together,
   so the first publish of a new name can reach the old registry process and fail with `E403`.
   Run 34922532212 (2026-09-15) got `403 Forbidden` for three new names at 02:48:23 to 02:48:29
    and published the fourth at 02:48:43.
   Since commit `fad49abcc`,
    `publishWithForbiddenRetry` (`package/config/pnpr/src/publish-retry.ts`) retries an `E403` publish
    every 20 seconds for 5 minutes from run start,
    with a fresh workload token per attempt;
    other failures,
    and refusals after that window,
    still fail at once.
   Manual recovery,
    if a redeploy ever outlasts the window,
    is `gh workflow run pnpr-publish.yml --raw-field only='<names>'`.
