# @monochromatic-dev/deps-update-cli

Runs `pnpm update --recursive --no-save`
 and explains the one refusal pnpm leaves unexplained.

```shell
# /var/home/user/Monochromatic
mise run deps:update
```

## Why it exists

With `minimumReleaseAgeStrict: true`,
 pnpm refuses `update --no-save` whenever it picks a version younger than `minimumReleaseAge`
 that `minimumReleaseAgeExclude` does not cover:

```text
Error: ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE
```

pnpm 12.4.2 names no package in that error.
The usual culprit is a same-day dependency of an excluded package.
See `doc/troubleshooting/pnpm-update-no-save-strict-release-age.md`.

## What it does

1.  Runs the real update with pnpm's output shown live.
2.  On success,
     exits 0.
3.  On any failure other than `ERR_PNPM_STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE`,
     exits with pnpm's output already shown.
4.  On that refusal,
     copies the resolution inputs
     (root `package.json`,
     `pnpm-workspace.yaml`,
     `pnpm-lock.yaml`,
     `.npmrc`,
     `.pnpmfile*`,
     and every workspace project's `package.json`)
     into a temporary directory,
     and runs `pnpm update --recursive --lockfile-only --config.minimum-release-age-strict=false` there.
     In loose mode pnpm appends each immature pick to that copy's `minimumReleaseAgeExclude`.
5.  Reports each appended version with its registry publish time,
     when it passes the age gate,
     and its direct dependents (`pnpm why --depth 1`).
6.  Ends with the two choices:
     add the package name or scope glob to `minimumReleaseAgeExclude`,
     or rerun after the latest maturity time.

The real checkout's manifests are never modified by the diagnosis;
 the temporary copy is deleted afterwards.

Example report:

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
```

When the loose resolution also fails after recording picks
 (for example `ERR_PNPM_PEER_DEP_ISSUES`),
 the report names that failure too,
 since the real update will stop on it next.

## Limits

- Publish times come from the registry packument,
   using `@scope:registry` or `registry` from pnpm config.
   Registry authentication is not sent,
   so private registries needing a token fail the lookup.
- `pnpm view <name>@<version> time` is not used:
   on pnpm 12.4.2 it returned a `time` map missing recent versions.

## Development

```shell
mise run //package/deps-update/cli:test:unit
mise run //package/deps-update/cli:lint
```
