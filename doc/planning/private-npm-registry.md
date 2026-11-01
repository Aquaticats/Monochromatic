# Private npm registry for unpolished packages

Status:
 grilling in progress (started 2026-09-14);
 nothing implemented.
Owner decisions are recorded here as they land;
 this file is canonical for the effort.

## Goal

Publish `@monochromatic-dev/oxlint-plugin-tsdoc`,
 `@monochromatic-dev/config-typescript`,
 and `@monochromatic-dev/or-throw-core`
 somewhere another owner project can install them with pnpm.
The owner judges them below public-release quality.
The Hetzner server managed by Coolify is available but not a constraint.

Scope widened by the owner on 2026-09-14:
 the registry immediately reflects every change of every workspace package,
 not only those three.

## Measured facts (2026-09-14)

- `Aquaticat/Monochromatic` is public on GitHub (`gh repo view`),
   so a separate registry buys unlisting and version freedom,
   not source confidentiality.
- `@monochromatic-dev/logger-core` is already public on npmjs (`0.1.0` to `0.4.0`,
   maintainer `aquaticat`),
   released through changesets and trusted publishing per `doc/decision/npm-publishing.md`.
- All three target packages are in the `ignore` list of `.changeset/config.json`.
- A scoped `.npmrc` route (`@monochromatic-dev:registry=`) sends the whole scope to one registry,
   so a second registry must proxy npmjs for the scope
   or consumers lose access to the npmjs-published scope members.
- `oxlint-plugin-tsdoc`:
   - `dist/final/node/index.mjs` imports only `@oxlint/plugins` at runtime;
      `oxlint-plugin-shared` and `ownership-marker-foreign-borrowed` are bundled.
   - `dependencies` still lists both bundled workspace packages,
      which `pnpm pack` would rewrite to `0.0.1` versions no registry serves.
   - Retracted 2026-09-14:
      an earlier reading said `dist/final/node/index.d.mts` imports `defineConfig` from `oxlint`.
      That line sits inside the TSDoc `@example` block copied from `src/index.ts` line 50;
      the search `rg 'import|from'` matched comment text.
      The only declaration import is `Plugin` from `@oxlint/plugins`.
- `module-or-throw` has no runtime or declaration imports.
- `config-typescript` is JSON only,
   extends only its own files,
   and is `"private": true`,
   which blocks publishing.
- No `package.json` within depth 4 of the home directory,
   outside Monochromatic worktrees and scratch clones,
   references any of the three packages.
- The consumer is `Aquaticat/labwc-config` (public).
   At `ca229f1` it holds shell scripts,
    Python helpers,
    libvirt XML,
    and labwc configs;
   it has no `package.json`,
    no TypeScript,
    and no `.github/workflows`.
   The local checkout is `~/labwc-vm-test`.
- pnpm `12.3.4` (the version `mise.toml` resolves from `pnpm = "latest"`)
   routes a single dependency to another registry:
   `pnpm-workspace.yaml` declares `registries: { <url>: { prefix: <name> } }`
   and the manifest specifier is `<name>:@scope/pkg@<range>`.
   Probe:
    `mirror:@monochromatic-dev/logger-core@^0.4.0` installed through `https://registry.npmmirror.com/`;
    pointing the same prefix at `https://registry.invalid/` failed metadata fetch for that package only.
   Lockfile keys become registry-qualified (`@monochromatic-dev/logger-core@mirror:0.4.0`).
   Docs: https://pnpm.io/settings/dependency-resolution (`registries`, `prefix` since 11.23.0).
- Workspace size (`pnpm list --recursive --depth -1 --json`):
   154 packages under `package/` plus the root;
   120 are `"private": true`.
   Categories include `test-fixture` (11 packages),
    `webapp-productivity` (6),
    and `desktop-app` (3).
- Publish volume for one week of `main` (2026-09-07 to 2026-09-13,
   script `publish-volume.ts` in the session scratchpad,
   commits from `git log --name-only`):
   - 328 commits,
      of which 75 touched a workspace package directory.
   - Republishing only directly touched packages:
      111 publishes.
   - Adding every dependent through `dependencies`,
       `peerDependencies`,
       and `optionalDependencies`:
      449 publishes.
   - Adding dependents through `devDependencies` as well:
      3216 publishes,
      up to 131 packages in one commit;
      `config-typescript` has 140 transitive dependents that way
       and `module-logger` has 130.
- Daily commit counts on `main` that week peaked at 172 (2026-09-09).
- `pnpm add 'mirror:@monochromatic-dev/logger-core@^0.4.0'` fails on `12.3.4` with `ERR_PNPM_INVALID_DEPENDENCY_NAME`
   (`dependency with an invalid name: "mirror:"`),
   although the docs show `pnpm add work:@corp/lib@^2.0.0`;
   writing the specifier into `package.json` and running `pnpm install` works.

## Settled decisions

- Registry:
   a registry other than npmjs
   (owner chose it over public npmjs at 0.x and over tarball URLs).
- Install environments:
   owner dev machine,
   GitHub Actions CI,
   and another owner machine such as `m1`;
   not a Coolify build.
   Consequence:
   the registry must be reachable from GitHub-hosted runners.
- Publish cadence (superseded 2026-09-14):
   automatic snapshot per push that touches a package
   (owner chose it over changesets releases and manual local publish).
   Replaced by the change-detection decision:
   versions change only by manual bumps.
- Consumer:
   `Aquaticat/labwc-config`.
- Read access:
   anonymous install;
   publishing still authenticates
   (owner chose it over token-gated install).
- Budget:
   no new spend;
   the already-paid Hetzner server and free tiers qualify.
- Scope routing:
   consumers route the whole `@monochromatic-dev` scope to the private registry,
   which proxies npmjs for scope members it does not hold
   (owner chose it over pnpm `prefix` specifiers and over a proxy-less scope route).
   Consequence:
   the registry product must support an npmjs uplink for the scope.
- Package set:
   every workspace package except the 11 under `package/test-fixture/`,
   private ones included
   (owner chose it over all 154 and over only the 35 non-private).
   npm packages only;
   Rust crates and Kotlin artifacts keep their own pipelines.
- Trigger:
   a GitHub Actions workflow on each push to `main`
   (owner chose it over a local post-commit hook plus CI and over a working-tree watcher).
- Refs:
   `main` only
   (owner chose it over per-branch dist-tags).
- Change detection:
   a package republishes only when its `package.json` `version` changes by a manual bump
   (hand edit or changesets);
   CI on `main` publishes every version the private registry lacks.
   Owner chose it over CI-computed snapshot versions and committed automatic bumps,
    knowingly dropping "every change is reflected";
   "immediately" now means as soon as a bumped version lands on `main`.
   Rejected along the way:
    tarball content diff,
    runtime dependency ripple,
    and directly touched packages as republish triggers.
- Consumer scope:
   publish side only;
   verification installs from a throwaway consumer,
   and `labwc-config` stays untouched
   (owner chose it over full and minimal consumer wiring).
- npmjs overlap:
   the private registry publishes all 143 packages,
   including `module-logger` and `module-fs-path`,
   which `npm-release.yml` also releases to npmjs
   (owner chose it over skipping them and over one pack uploaded to both registries).
   Consequence presented with the choice:
   the same version packed by two workflows can differ byte for byte,
   so lockfile integrity depends on which registry served it.
- Dependents:
   when a version is bumped by hand,
   a commit-time hook finds workspace dependents that must bump too and bumps them in the same commit
   (owner's own answer,
    chosen over a CI stale-pin check,
    changesets for every package,
    and hand bumps with no check).
- Registry product:
   pnpr on the Coolify host
   (owner chose it over Verdaccio 6,
    GitLab.com,
    and Nexus CE).
- Ripple edges:
   a dependent bumps when it reaches the bumped package through `dependencies`,
    `peerDependencies`,
    or `optionalDependencies`,
   or imports it from source as a `devDependency` that the build bundles
   (owner chose it over all workspace edges and over runtime edges only).
- Backstop:
   none;
   commits that bypass cli-git can publish stale pins
   (owner chose it over a failing CI check and over CI bump commits).
- Front proxy:
   the Coolify host runs the owner's self-managed Caddy
   (Coolify's own proxy management is disabled,
    per `doc/handover/garage-file-sync.md`),
   so the Traefik `%2F` caveat does not apply.
- Durability:
   losing historical package versions is acceptable,
   so registry storage needs no backup;
   recovery republishes current versions.
- pnpr distribution:
   the npm package `@pnpm/pnpr` at dist-tag `next`
   (owner instruction,
    superseding an adopted exact image pin).
   Measured 2026-09-14:
    `next` is `0.1.0-alpha.11` and `latest` is the older `0.0.0-26070301` (`npm view`);
    `ghcr.io/pnpm/pnpr` lists version tags only,
    with no `next` tag (`podman search --list-tags`).
   Consequence:
    the deployment installs `@pnpm/pnpr@next` itself,
    and each rebuild picks up whatever `next` points to.
- Upstream:
   pnpr has no npmjs upstream;
   its hosted registry claims `@monochromatic-dev/*` outright,
   and consumers route only that scope to pnpr.
   Through the scope route,
    `mcp-nvim` and npmjs-only historical versions become uninstallable,
    which the owner accepted.
   History:
    the owner first dropped the upstream,
    then after the confirmation summary asked for an npmjs proxy covering everything pnpr does not host,
    then withdrew it ("let's not make it proxy")
    after seeing that a full proxy is an open public mirror whose cache has no documented eviction.
   In the same round the owner picked "pnpr as default registry" while a proxy was still assumed;
    without a proxy that routing breaks every non-scope install,
    so the scope route stands.
- Publish authentication:
   pnpr OIDC workload credential from the GitHub Actions publish workflow,
   no stored token
   (owner chose it over a stored CI user token).
- Config delivery:
   file-enforcer generates the pnpr config,
    including the exact package-name list,
   into the repo,
   and Coolify deploys the compose and config from the public GitHub repo on push
   (owner chose it over a runbook with Coolify UI edits and over an SSH push from the workflow).
   Accepted consequence:
   a new package's first publish can fail until the redeploy lands,
   and a later qualifying run retries it.
- `./ts` subpaths:
   stripped from every tarball published to pnpr,
   the same rule as `doc/decision/npm-publishing.md`
   (owner chose it over keeping them).
- Entry-less packages:
   a package publishes only if its manifest has `exports`,
    `main`,
    `module`,
    or `bin`,
   which removes 15 packages and leaves 128
   (owner chose it over a name-suffix exclusion and over publishing them).
- Server-side steps:
   the owner performs the Njalla A record,
    Coolify resource,
    and Caddy site block
   from a runbook at `doc/runbook/deploy-pnpr-registry.md`;
   no host,
    Coolify,
    or DNS credentials are shared with agents
   (owner chose it over a mixed split and over full agent access).
- Build failures on GitHub-hosted runners:
   the run fails and names the package,
   until it is fixed or added to a reviewed exclusion list in the generated config
   (owner chose it over failing once per version and over warnings on a green run).
- Verification:
   from a throwaway consumer,
    anonymous install and use of `oxlint-plugin-tsdoc` (loaded by oxlint),
    `config-typescript` (extended by `tsc`),
    and `module-or-throw` (called);
    install of `module-logger` from pnpr;
    a publish without the OIDC credential is rejected;
    the workflow's OIDC publish succeeds.
   A GitHub issue tracks checking installability of the remaining published packages
   (owner's answer,
    chosen over installing all 128 now and over a registry-level check).

## Adopted without asking (veto welcome)

- Hostname `pnpr.c.aquati.cat`
   (owner instruction at confirmation,
    replacing the adopted product-neutral `npm.c.aquati.cat`),
   following the host's `<service>.c.aquati.cat` convention (`garage.c.aquati.cat`).
- pnpr storage is a Coolify volume on local disk,
   not the Garage S3 store on the same host,
   which would add a dependency without adding durability.
- The publish job's version-missing check reads anonymously,
   because a pnpr workload credential "cannot read packages"
   (https://pnpm.io/pnpr/oidc).
- Scoped publish through Caddy is verified at implementation time,
   since the encoded slash in `PUT /@scope%2fname` must reach pnpr intact.
- The publish workflow is its own file,
   because the OIDC `workflow_ref` claim pins one workflow path,
   and its `push` filter matches `**/package.json`,
    the generated pnpr config,
    and the workflow itself,
   because under manual bumps only a manifest change can create a missing version;
   `workflow_dispatch` stays for manual retries.
- The pnpr hosted registry is named `monochromatic-dev`,
   so consumers route `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/~monochromatic-dev/`.
  Moved 2026-10-21:
   the route is `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/`
   from the `0.1.0-alpha.16` base
   (`doc/troubleshooting/pnpr-base-move.md`).
- Packing reuses the `npm-release.yml` install override `--config.dedupe-direct-deps=false`
   (`doc/troubleshooting/pnpm-pack-dedupe-direct-deps.md`).
   Superseded 2026-09-15:
    pnpm 12.3.4 ignores that flag,
    so both workflows set `pnpm_config_dedupe_direct_deps=false` job-wide.
- The dependent-bump policy reads the staged `version` of every publishable manifest,
   and treats a `devDependency` as bundled when non-test source files import it.
- The pnpr deployment lives in a workspace directory under `package/config/`,
   beside `package/config/tofu`,
   with a `README.md` documenting consumer setup;
   it has no entry points,
   so it never publishes itself.
- Tarballs are packed in a clean CI checkout holding no secrets,
   so their content is the public repo plus build output;
   ignored local files such as `package/config/tofu/terraform.tfstate` and `hetzner.auto.tfvars.json`
   are untracked (`.gitignore` lines 15 and 16) and never reach CI.
- Packaging fixes land regardless of registry:
   bundled workspace packages move to `devDependencies`
   (the rule in `doc/decision/npm-publishing.md`).
   The earlier `oxlint` peer dependency item is withdrawn with the retracted declaration-import fact.
- `config-typescript` keeps `"private": true`;
   the publish step removes `"private": true` from every packed manifest,
   because `npm publish` refuses private packages.
   This supersedes the earlier adopted item that dropped the flag in git.
- Published versions are immutable:
   a version already in the registry is never overwritten.
- The first workflow run publishes the current version of every package in the set,
   since all of them are missing from a new registry.
- Publishing runs in dependency order;
   a package that fails to build blocks itself and its dependents,
   the rest still publish,
   and the run reports failure.
- `latest` moves only when the published version is greater than the current `latest`.
- The dependent-bump hook is a cli-git commit policy,
   not a raw Git hook or hk step:
   hk and Pkl were retired (issue `#357`),
   and cli-git's commit transaction already applies policy patches to a private index
   (`doc/planning/final-newline-normalization.md`,
    `package/git-policy/cli/src/policy-engine/final-newline-policy.ts`).
- Dependents bump by patch and the ripple is transitive,
   matching `updateInternalDependencies: patch` in `.changeset/config.json`.
- The changesets `version` job in `npm-release.yml`
   runs the same ripple before committing the Version Packages pull request,
   because that commit is authored in CI where no local hook runs,
   and changesets does not version the ignored dependents of `module-logger` or `module-fs-path`.
- Consumer note for later wiring:
   pnpm 11 and later default `minimumReleaseAge` to 1440 minutes,
   so consumers exclude `@monochromatic-dev/*` or new versions stay uninstallable for a day
   (https://pnpm.io/settings/dependency-resolution).

## Registry candidates (research 2026-09-14)

Settled gates:
 anonymous read,
 npmjs proxy for the scope,
 no new spend,
 reachable from GitHub-hosted runners.

### Pass the gates

- Verdaccio 6 (6.10.3,
   2026-09-05,
   MIT):
   per-pattern `access: $all`,
    `publish: $authenticated`,
    `proxy: npmjs`;
   metadata carries `time`;
   standard tarball URLs.
   No OIDC,
    so CI publishes with a stored token.
   No Coolify template;
    deploy `verdaccio/verdaccio:6` as custom compose.
   Sources:
    https://github.com/verdaccio/verdaccio,
    https://verdaccio.org/docs/packages.
- pnpr (0.1.0-alpha.11,
   2026-09-10,
   PolyForm Shield 1.0.0,
   from the pnpm team):
   anonymous read,
    npmjs upstream,
    GitHub OIDC publish without a stored token.
   `check_workload_request` rejects non-PUT requests carrying the OIDC token
    (`pnpr/crates/pnpr/src/server/oidc.rs` lines 147 to 167, read not tested).
   Sources:
    https://pnpm.io/pnpr/configuration,
    https://pnpm.io/pnpr/oidc.
- GitLab.com project package registry:
   free,
    excluded from the storage quota,
    anonymous read for a public project,
    missing packages redirect to npmjs.
   CI publishes with a stored deploy token;
    pnpm needs `serverType: artifactory`.
   Source:
    https://docs.gitlab.com/user/packages/npm_registry/.
- Sonatype Nexus Community Edition (3.96.1):
   npm proxy and group repositories,
    Coolify template with a 2703m JVM heap.
   Caps:
    40,000 components and 100,000 requests a day,
    cached npmjs packages count.
   Hetzner server capacity not measured
    (no SSH alias for it in `~/.ssh/config`).
   Source:
    https://help.sonatype.com/en/ce-onboarding.html.

### Fail a gate

- GitHub Packages:
   install needs a token even for public packages,
   and the scope must match the owner.
- Forgejo 16 and Gitea 1.27:
   no npmjs proxy.
- npmjs restricted packages:
   paid.
- AWS CodeArtifact and Azure Artifacts:
   always authenticated.
- npflared:
   reads need a token.
- vsr:
   anonymous read unresolved (docs and code disagree),
   release candidate only.
- Cloudsmith,
   Gemfury,
   Buildkite:
   anonymous npm install or arbitrary scope support unverified.

### Cross-cutting caveats

- Coolify pins `traefik:v3.6` (`bootstrap/helpers/proxy.php` line 282).
   Traefik 3.6.4 to 3.6.6 rejected `%2F` by default,
   which broke scoped `npm publish` behind Coolify
   (https://github.com/coollabsio/coolify/issues/7631);
   the running proxy version on the Hetzner server is unmeasured.
- `@changesets/cli` 3.0.3 runs `pnpm info <name> --json` without `--registry`
   (https://github.com/changesets/changesets/blob/main/packages/cli/src/lib/pnpm.ts).
- The npmjs org `monochromatic-dev` also holds `mcp-nvim`,
   which is not a workspace package,
   so the scope proxy still matters after all workspace packages publish privately.

### pnpr facts that shape the remaining choices

- OIDC trust is configured per named hosted registry and per exact package name,
   and validates `repository_id`,
    `repository_owner_id`,
    `workflow_ref`,
    and the `subject` claim.
   The workload credential permits only `PUT` publications of the configured packages
   (https://pnpm.io/pnpr/oidc).
- A router resolves a package to "the first listed source whose `packages:` keys claim its name,
   authoritatively";
   package patterns include `@scope/*` and exact names
   (https://pnpm.io/pnpr/configuration).
- Storage is authoritative and needs a durable volume;
   the upstream cache is disposable.
- Nothing in the npmjs `@monochromatic-dev` org depends on the upstream except
   `mcp-nvim` (`0.1.0`,
    source directory `packages-deprecated/mcp/nvim`,
    not a workspace package)
   and npmjs-only historical versions of `module-logger` and `module-fs-path`.

- Source check at pnpm `main` `f38f11e` (sparse clone in `~/temp/agent/pnpm-pnpr-20260914`):
   `pnpr/crates/pnpr/src/server/oidc.rs` lines 133 to 142 reject any OIDC workload package that is not an exact name:

   ```rust
   // pnpr/crates/pnpr/src/server/oidc.rs
   if !matches!(
       pnpr_registry::PackagePattern::parse(package, pnpr_registry::Ecosystem::Npm),
       Ok(pnpr_registry::PackagePattern::Exact(_)),
   ) {
   ```

   and `check_workload_request` (lines 147 to 167)
   allows only `PUT` to `{base}/~{registry}/{package}`
   after percent-decoding the path.
   So the OIDC trust config must list all publishable package names,
   and publishes target the named-registry path `/~<registry>/`.

### Package shapes (measured 2026-09-14)

Script `build-shape.ts` in the session scratchpad,
 over the 143 packages in the set:

- 93 export or bin into `dist/` (92 have a `build` task).
- 31 have other entry points (JSON configs,
   shims,
   plugins,
   Electron apps).
- 4 export only TypeScript source
   (2 `claude-code-plugin`,
    1 `config`,
    `ownership-marker-foreign-borrowed`).
- 15 have no `exports`,
   `main`,
   `module`,
   or `bin` at all:
   `config-cosign`,
   `config-dotfiles`,
   `intellij-plugin-islands-black`,
   `module-css-edit.bench`,
   `module-css-edit.conformance`,
   `module-css-edit.fuzz`,
   `module-jsonc-edit.bench`,
   `module-jsonc-edit.conformance`,
   `module-jsonc-edit.fuzz`,
   `module-logger.fuzz`,
   `module-toml-edit.fuzz`,
   `runtime-error-bun`,
   `ssg-aquati.cat`,
   `webapp-productivity-done`,
   `webapp-productivity-done-postcss`.

### Server-side facts (measured 2026-09-14)

- `garage.c.aquati.cat` resolves to `135.181.104.96`;
   `npm.c.aquati.cat` and a random `*.c.aquati.cat` name do not resolve,
   so there is no wildcard record and a new A record is needed (`getent ahostsv4`).
- `aquati.cat` nameservers are Njalla (`dig +short NS aquati.cat`).
- No SSH alias for the Coolify host exists in `~/.ssh/config`
   (aliases:
    `m1`,
    `x13-win`).

## Open questions

None.
The owner confirmed shared understanding on 2026-09-14;
the accepted design moves to `doc/decision/private-npm-registry.md`.

Context recorded while the proxy was reconsidered:
 the Coolify host is about 116 ms RTT from the owner's clients (`doc/handover/garage-file-sync.md`),
 and pnpr docs describe only a packument TTL (`--packument-ttl-secs`),
 with no npm upstream cache eviction found in `pnpr/crates/pnpr/README.md` or `pnpr/npm/pnpr/README.md`.
