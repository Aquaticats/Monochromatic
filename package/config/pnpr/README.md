# `@monochromatic-dev/config-pnpr`

Deployment of the private pnpr npm registry at `https://pnpr.c.aquati.cat/`,
 which serves `@monochromatic-dev` workspace packages that are not public-release quality.
Design and rationale:
 [`doc/decision/private-npm-registry.md`](../../../doc/decision/private-npm-registry.md).

## Install from the registry

Reads are anonymous.
Route only the scope to pnpr;
 every other package keeps coming from npmjs,
 because pnpr proxies nothing.

```ini
# .npmrc
@monochromatic-dev:registry=https://pnpr.c.aquati.cat/npm/~monochromatic-dev/
```

That route is the `0.1.0-alpha.16` surface
 (moved 2026-10-21 from `@monochromatic-dev:registry=https://pnpr.c.aquati.cat/~monochromatic-dev/`,
 the `0.1.0-alpha.11` root surface);
 see "To move to a newer pnpr" and
 [`doc/troubleshooting/pnpr-base-move.md`](../../../doc/troubleshooting/pnpr-base-move.md).

pnpm 11 and later hold back versions younger than `minimumReleaseAge`,
 so exclude the scope or a fresh publish stays uninstallable for a day:

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - '@monochromatic-dev/*'
```

`@monochromatic-dev/config-typescript` sets `types: ["node"]`,
 so a project extending it also installs `@types/node`;
 without it `tsc` stops with `TS2688: Cannot find type definition file for 'node'`.

Through this route,
 versions that exist only on npmjs (older `module-logger` and `module-fs-path` releases,
 `@monochromatic-dev/mcp-nvim`) are not installable.

## Files

- `Containerfile`:
   starts from the digest-pinned upstream `ghcr.io/pnpm/pnpr:0.1.0-alpha.17` image,
   bakes in `config.yaml`,
   and disables the install-accelerator surface.
   Alpha.17 includes the GitHub Actions discovery fix that previously required
   the `ghcr.io/aquaticat/pnpr` fork.
   The isolated GitHub workload check passed in run `41941248129`.
- `compose.yaml`:
   the Coolify Docker Compose resource.
   It publishes `127.0.0.1:7677` for the host's Caddy,
   requires `PNPR_SECRET` from Coolify,
   and keeps storage in `pnpr-readable-registry-storage`.
   The previous `pnpr-registry-storage` volume stays declared and untouched for rollback.
   Recovery follows the owner's choice to rebuild and republish current versions.
- `config.yaml`:
   generated and checked by this package
   (`src/generated-config-sync.ts`,
  command line `src/generate-config.ts`).
   Run `mise run //package/config/pnpr:generate:config` after adding,
    removing,
    or changing the entry points of a workspace package;
   file-enforcer does not write this file.
   `mise run //package/config/pnpr:check:config` writes nothing and exits non-zero,
   naming the file and every package that differs,
   when the checked-in file is not what the generator renders.
   Never edit it by hand.

## Which packages publish

`generatePnprConfig` (selection in `selectPnprPublishedNames`,
`src/generated-config-selection.ts`)
lists every workspace package that:

- is outside `package/test-fixture/`,
- still declares `exports`,
   `main`,
   `module`,
   or `bin` after `./ts` export subpaths are stripped,
- and is not in `PNPR_EXCLUDED_PACKAGES` (`src/generated-config-settings.ts`),
   the reviewed exclusion list with one reason per package.

The list doubles as pnpr's OIDC trust list,
 because pnpr accepts only exact package names for workload publishing.
This package has no entry points,
 so it never publishes itself.

## Publishing

`.github/workflows/pnpr-publish.yml` runs `task-run //package/config/pnpr:publish` on pushes to `main`
 that change a manifest,
 `config.yaml`,
 the script,
 or the workflow.
`src/publish-missing-versions.ts` then:

1.   reads the trusted names from `config.yaml`,
2.   reads each package's published versions anonymously,
3.   builds (`task-run //<dir>:build` when the package's `task.ts` declares `build`;
      the CI has no mise,
      and `mise.toml` is read as TOML to decide that,
      so a package that declares `build` only in `mise.toml` fails with a diagnostic
      instead of publishing a tarball without its `dist`,
      as does a package whose `mise.toml` does not parse),
      packs with pnpm,
      and rewrites the packed manifest for every missing version in dependency order,
4.   publishes each tarball with a fresh OIDC workload token,
      moving `latest` only forward (older versions get `backfill`).

A failed package fails the run and blocks its dependents.
The job sets `pnpm_config_dedupe_direct_deps=false`,
 because pnpm 12.3.4 ignores the `--config.` form
 (`doc/troubleshooting/pnpm-pack-dedupe-direct-deps.md`).

Run a named subset from **Actions**,
 **pnpr-publish**,
 **Run workflow**,
 with a comma-separated **only** input.
Locally,
 `PNPR_REGISTRY_ORIGIN`,
 `PNPR_PUBLISH_TOKEN`,
 and `PNPR_PUBLISH_ONLY` point the same script at a test registry;
 `mise run //package/config/pnpr:publish:dry-run` builds and packs without publishing.

The workflow's `rebuild_check` input runs that same build-and-pack path against an empty disposable registry.
Its `verify_image` input tests a digest-pinned image with real GitHub workload credentials
and an isolated trust configuration.
That check also exercises anonymous-write refusal,
off-target workload refusal,
and exact archive download.
Neither mode publishes to production.

## Deploy and operate

Deployment steps live in
 [`doc/runbook/deploy-pnpr-registry.md`](../../../doc/runbook/deploy-pnpr-registry.md).
Coolify watches `config.yaml`,
`Containerfile`,
and `compose.yaml` under `package/config/pnpr/`.
Changes to those files on `main` trigger deployment.

The alpha.17 upgrade keeps the existing `/npm/~monochromatic-dev/` consumer URL.
It changes the generated registration field from `max_users` to `maxUsers`
and uses a fresh store because alpha.17 cannot read alpha.16's package paths.
The alpha.16 `upgrade-storage` command is absent from alpha.17.
The historical procedure in the runbook applies only to the alpha.11-to-alpha.16 move.

The owner selected repairing publishing and rebuilding current versions before this cutover.
The old volume remains available with the old image for rollback.
Rebuilt archives can differ from archives recorded in existing consumer lockfiles;
archive preservation was not the selected recovery path.
The upgrade assessment and verification record live in
[`doc/planning/pnpr-alpha17-upgrade.md`](../../../doc/planning/pnpr-alpha17-upgrade.md).

Future image changes require reviewing the release's configuration and storage format,
testing the pinned image with `verify_image`,
and verifying the selected storage transition before deployment.
Alpha.17 leaves public metrics disabled unless explicitly configured,
so the old `/-/metrics` version probe is not a deployment health check.

## Local check

```bash
# package/config/pnpr
mise run //package/config/pnpr:start:image
curl http://127.0.0.1:17677/-/ping
```

A local container cannot complete an OIDC publish,
 because GitHub issues workload tokens only inside Actions;
 unauthenticated and malformed-token publishes return HTTP 401.
