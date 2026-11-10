# pnpm 11.11.0 pack rejects an unlinked workspace development dependency before catalog export

## Symptom

Packing `@monochromatic-dev/oxlint-plugin-no-restricted-syntax` directly emitted:

```text
[ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL] Cannot resolve workspace protocol of dependency
"@monochromatic-dev/config-typescript" because this dependency is not installed. Try running "pnpm install".
```

The package's production artifact is bundled and does not load this development-only TypeScript configuration,
but `pnpm pack` still validates and transforms the development dependency.

The semantic rule later moved to
`@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type`.
Its moved external-consumer test retains the same staged production-manifest workaround;
the original package name remains here because it identifies the artifact used for this diagnosis.

Using `npm pack` avoids that validation but leaves `catalog:` production dependency specifications unchanged.
Installing that tarball externally then emits:

```text
[ERR_PNPM_SPEC_NOT_SUPPORTED_BY_ANY_RESOLVER] "@oxlint/plugins@catalog:" isn't supported by any available resolver.
```

## Root cause

pnpm tag `v11.11.0`,
 commit `8e1e4c0aaece387dfda16377b2ba31f7a1a2602c`,
 transforms all three dependency
fields in
`pnpm11/releasing/exportable-manifest/src/index.ts:61`:

```ts
await Promise.all((['dependencies', 'devDependencies', 'optionalDependencies'] as const).map(async (depsField) => {
  const deps = await makePublishDependencies(dir, originalManifest[depsField], {
    modulesDir: opts?.modulesDir,
    convertDependencyForPublish,
  })
```

A `workspace:*` dependency is resolved by reading its installed manifest.
The exact diagnostic comes from
`pnpm11/releasing/exportable-manifest/src/index.ts:134`:

```ts
if (!manifest?.name || !manifest?.version) {
  throw new PnpmError(
    'CANNOT_RESOLVE_WORKSPACE_PROTOCOL',
    `Cannot resolve workspace protocol of dependency "${depName}" ` +
      'because this dependency is not installed. Try running "pnpm install".'
  )
}
```

Catalog conversion occurs in the same export path through `resolveCatalogProtocol` at
`pnpm11/releasing/exportable-manifest/src/index.ts:147`.
`npm pack` does not execute pnpm's exportable-manifest conversion,
 so it cannot substitute this workspace's catalog
ranges.

The earlier idea that a production-only filter would skip development dependency conversion was wrong.
`createExportableManifest` maps `devDependencies` independently of package-selection filters.

## Verification

The behavior was reproduced with pnpm `11.11.0` against this package after
`mise run prepare:pnpm:install`.
`package/oxlint-plugin/no-restricted-syntax/node_modules/@monochromatic-dev/config-typescript` was absent while the
workspace-root link existed.

### Commands that fail

```sh
pnpm pack --pack-destination /tmp/consumer
pnpm pack --filter-prod . --pack-destination /tmp/consumer
pnpm --filter '@monochromatic-dev/oxlint-plugin-no-restricted-syntax...' pack --recursive
```

All three commands report `ERR_PNPM_CANNOT_RESOLVE_WORKSPACE_PROTOCOL` for
`@monochromatic-dev/config-typescript`.

An `npm pack` tarball installs unsuccessfully outside the workspace because its production manifest retains
`@oxlint/plugins: "catalog:"`.

### Path that works

`package/oxlint-plugin/no-restricted-syntax/src/external-consumer.unit.test.ts` creates a disposable publication
workspace containing:

`package/oxlint-plugin/no-restricted-syntax/src/external-consumer.unit.test.ts` is not in the
working tree and no commit in this repository carries it.

- built `dist/final` artifacts;
- production manifest fields;
- local catalog ranges for `@oxlint/plugins` and TypeScript;
- no development-only workspace protocols.

`pnpm pack` converts those catalog entries.
A disposable external consumer installs the resulting tarball and launches the bundled rule's TypeScript 7 native
bridge successfully.

## Verified workarounds

Use a disposable production staging directory for consumer acceptance.
Retain the authored workspace manifest for development,
but stage only production dependency fields and built files before `pnpm pack`.
This verifies catalog conversion and external runtime resolution without mutating workspace `node_modules`.

The tradeoff is that this acceptance path does not prove conversion of development-only workspace specifications.
Those specifications are irrelevant to package consumers but still need a separate direct-pack check before actual
publication.

Use `publishConfig.exports` to expose only `dist/final` from the packed package.
The workspace can retain `./ts` exports for source-level cross-package imports without advertising absent source files
to registry consumers.

## What does not work

- `--filter-prod` controls package selection;
   it does not remove `devDependencies` from exportable-manifest conversion.
- Recursive packing still validates the selected package's installed workspace links.
- `npm pack` does not perform pnpm catalog substitution.
- Re-running the root install did not create the package-local `config-typescript` link in this checkout.
- A platform-filtered install did not expose `tsdown` when it was only a development dependency of the shared tsdown
  configuration package.
  The invoking package now declares `tsdown` directly.
  A subsequent missing-command failure came from the separately diagnosed
  [`mise-action` Windows shim defect](mise-action-windows-shims.md),
  not pnpm dependency conversion.
- Manually creating a workspace link would mutate installation state and would not represent a reproducible consumer
  boundary.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** No. pnpm intentionally validates every published dependency field and requires an
    installed manifest for `workspace:*` conversion.
2.  **Can upstream fix it?
    ** No defect has been established.
     A production-only packing feature would be a separate
    request.
3.  **Are they supporting this use case?
    ** pnpm documents catalog and workspace conversion during publishing,
     not
    omission of unresolved development dependencies.
4.  **Would the repo welcome our contribution?
    ** Not evaluated because no upstream defect was found.
5.  **Will they likely fix it?
    ** Not applicable without a defect or accepted feature request.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** No upstream change is appropriate.
     The
    disposable production staging path is verified locally.

Nothing should be filed upstream.
