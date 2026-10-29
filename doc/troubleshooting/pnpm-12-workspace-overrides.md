# pnpm 12 ignores package.json overrides in an isolated packed consumer

## Symptom

The current-source pnpr producer in GitHub Actions run `39130630504`
actually built and packed 26 archives.
Its external consumer fixture put `pnpm.overrides` in `package.json`.
pnpm `12.14.0` emitted:

```text
The "pnpm" field in package.json is no longer read by pnpm.
The following keys were ignored: "pnpm.overrides".
```

The offline install then failed with `ERR_PNPM_NO_OFFLINE_META`
for the registry version of `@monochromatic-dev/numeric-format-core@0.0.1`.
No positive installed-consumer body ran in that invocation.
The actual packed CLI without its dependencies separately failed with `ERR_MODULE_NOT_FOUND`,
as required by the missing-dependency negative control.

## Configuration boundary

The current [pnpm settings documentation][pnpm-settings] places project configuration
in `pnpm-workspace.yaml`.
The [dependency-resolution documentation][pnpm-overrides] shows top-level `overrides`
in that file and says overrides can only be set at the project root.
These primary pages were fetched on 2026-10-18.

The corrected disposable fixture writes top-level dependencies in `package.json`
and top-level local-archive `overrides` in `pnpm-workspace.yaml`.
It restores the exact producer artifact,
verifies every retained archive,
and performs the isolated install before importing or calling installed packages.
Run `39134064943` at harness `c4c0d6fa8d3714849dc952f412102902eaf4d181`
is the bounded positive control;
its terminal result is recorded in
[the coverage research](../planning/depot-complete-coverage-research.md).
The source assertions and pnpr archives are unchanged.

The producer artifact is `12137807882`,
`7,522,668` bytes,
SHA-256 `d3dbd7e52fdc99e9008a17cbfaa4dba68725a359ae589aba03a679e6cba8fcf0`.
Archive hashes,
manifest hashes,
and materialized source-file hashes are separate receipt fields.
Git blob identities are not used as proof of materialized file bytes.

## Separate earlier Git flag rejection

Run `39129851779` stopped before any packing because the research harness invoked
unsupported `git ls-files --null`.
Git emitted `error: unknown option ` followed by `null`.
An actual local invocation verified the supported `git ls-files --stage -z` form
and its NUL-delimited output.
The corrected producer run then created the 26 real archives.
That Git harness failure does not establish a pnpm packaging failure.

[pnpm-settings]: https://pnpm.io/settings
[pnpm-overrides]: https://pnpm.io/settings/dependency-resolution#overrides

## Actual consumers and distinct research harness failures

The supported workspace configuration installs all 26 immutable archives in run `39134064943`.
Twenty-five packages expose runtime entries and import successfully.
The ownership marker declares only `types` entries; runtime import correctly raises
`ERR_PACKAGE_PATH_NOT_EXPORTED`.
An external compiler consumer, using the exact packed marker archive,
compiles both exported marker types and rejects an invalid field with TypeScript `7.0.2` diagnostic `TS2322`.
Its runtime-negative control confirms the intended absence of a runtime entry.

Run `39136177894` also passes all five actual functional consumers, including the installed mutation CLI
with supported `--package cli-fixture --dry-run src/value.ts`.
The CLI enumerates two mutants and a selected unit file without executing either.
The retained red aggregate is the research runtime-import assumption for the types-only package,
not a failed application assertion.
Run `39135404757` is separate: a truncated bootstrap activated an older plan.
Actual bootstrap execution plus decoded plan/task equality qualifies the replacement harness.
The [coverage research](../planning/depot-complete-coverage-research.md) retains exact boundaries and sealed receipts.
