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
   starts from the digest-pinned `ghcr.io/aquaticat/pnpr` image,
   bakes in `config.yaml`,
   and disables the install-accelerator surface.
   That image is pnpr 0.1.0-alpha.11 with a fix for GitHub Actions OIDC publishes
   (`doc/troubleshooting/pnpr-oidc-github-actions-discovery.md`),
   published by `.github/workflows/pnpr-patched-image.yml`
   on branch `pnpr-oidc-workload-discovery` of `Aquaticat/pnpm`.
- `compose.yaml`:
   the Coolify Docker Compose resource.
   It publishes `127.0.0.1:7677` for the host's Caddy,
   requires `PNPR_SECRET` from Coolify,
   and keeps storage in the unbacked `pnpr-registry-storage` volume.
- `config.yaml`:
   generated and checked by this package
   (`src/generated-config-sync.ts`, command line `src/generate-config.ts`).
   Run `mise run //package/config/pnpr:generate:config` after adding,
    removing,
    or changing the entry points of a workspace package;
   `mise run sync:files` does the same through the root `file-enforcer.config.ts`,
   which calls this package's generator.
   `mise run //package/config/pnpr:check:config` writes nothing and exits non-zero,
   naming the file and every package that differs,
   when the checked-in file is not what the generator renders.
   Never edit it by hand.

## Which packages publish

`generatePnprConfig` (`src/generated-config-selection.ts`) lists every workspace package that:

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

`.github/workflows/pnpr-publish.yml` runs `mise run //package/config/pnpr:publish` on pushes to `main`
 that change a manifest,
 `config.yaml`,
 the script,
 or the workflow.
`src/publish-missing-versions.ts` then:

1.   reads the trusted names from `config.yaml`,
2.   reads each package's published versions anonymously,
3.   builds (`mise run //<dir>:build` when declared),
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

## Deploy and operate

Deployment steps live in
 [`doc/runbook/deploy-pnpr-registry.md`](../../../doc/runbook/deploy-pnpr-registry.md).
Coolify redeploys only when files under `package/config/pnpr/` change,
 through its Watch Paths setting.

To move to a newer pnpr,
 rebase `pnpr-oidc-workload-discovery` in `Aquaticat/pnpm` onto upstream and push it,
 then copy the `tag@digest` from that run's **pnpr image** notice into the `Containerfile` `FROM` line.
A base newer than 0.1.0-alpha.11 also needs the generated `config.yaml` schema change in the same commit,
 because Coolify bakes the config into the rebuilt image:
 pnpr 0.1.0-alpha.16's schema names the web UI block `ui` (`enabled`) where our config had `web` (`enable`),
 and the parser rejects unknown fields.
A base at 0.1.0-alpha.16 or newer also gates the deploy in two ways,
 both measured in
 [`doc/troubleshooting/pnpr-base-move.md`](../../../doc/troubleshooting/pnpr-base-move.md):

- The startup layout gate aborts on the deployed store until `pnpr upgrade-storage` converts it
  on the Coolify host,
  with the container stopped and the volume backed up.
  The section "Move to a newer pnpr base" in
  [`doc/runbook/deploy-pnpr-registry.md`](../../../doc/runbook/deploy-pnpr-registry.md)
  is that procedure.
  The converted store cannot be read by the 0.1.0-alpha.11 base,
  so rollback after that step means restoring the snapshot.
- The npm surface answers only under `/npm`,
  so move the consumer route,
  the publish tooling URLs in `package/config/pnpr/src/`
  (`publish-steps.ts` and `registry-read.ts`),
  and every doc example in that same commit.

Once an upstream release verifies GitHub workload tokens,
 return to the upstream image instead.

## Local check

```bash
# package/config/pnpr
mise run //package/config/pnpr:start:image
curl http://127.0.0.1:17677/-/ping
```

A local container cannot complete an OIDC publish,
 because GitHub issues workload tokens only inside Actions;
 unauthenticated and malformed-token publishes return HTTP 401.
