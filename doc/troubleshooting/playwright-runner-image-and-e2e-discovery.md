# Playwright 1.64.0 with a 1.63.0 image misses browser executables before test bodies

## Symptom

Frozen application `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`
locks `@playwright/test` to 1.64.0,
while `playwright.Dockerfile` selects the digest-qualified 1.63.0 image.
In GitHub Actions run `39123458425`,
the actual `package/test-harness/core` browser owner enumerated two cases per engine,
then each engine failed at launch with Playwright's diagnostic:

```text
Executable doesn't exist at ...
Looks like Playwright was just updated to 1.64.0.
Please update docker image as well.
```

The six enumerated cases receive zero executed-body credit.
All 42 shared build invocations passed.
This is independent of the [rootless Podman identity failure](depot-rootless-podman-group-identity.md).

## Root cause

The immutable Playwright source checkout is
`microsoft/playwright` tag `v1.64.0`,
commit `606ecd8b34bac100db32561b6278ffd76e5eb3cd`,
read at `~/temp/agent/playwright-1.64-image-20261018` on 2026-10-18.
No upstream source was changed.

`packages/playwright-core/src/server/registry/index.ts:520`
checks the selected executable and compares the image's recorded version with the runner:

```typescript
// packages/playwright-core/src/server/registry/index.ts:520
if (!canAccessFile(e)) {
  const currentDockerVersion = readDockerVersionSync();
  const preferredDockerVersion = currentDockerVersion ? dockerVersion(currentDockerVersion.dockerImageNameTemplate) : null;
  const isOutdatedDockerImage = currentDockerVersion && preferredDockerVersion && currentDockerVersion.dockerImageName !== preferredDockerVersion.dockerImageName;
```

The same function emits the failure at `registry/index.ts:555`:

```typescript
// packages/playwright-core/src/server/registry/index.ts:555
throw new Error(`Executable doesn't exist at ${e}\n${wrapInASCIIBox(prettyMessage, 1)}`);
```

The frozen recipe itself states that its image version must match the lockfile.
The official [Docker guidance][docker-doc] requires the same agreement.
The OCI-aware registry request returned the 1.64.0 image index digest
`sha256:06a9939e57531807f8d5fd76ce44b53165ffb7d7501d87ab10e285c20b1e971f`.
Its amd64 child is
`sha256:bc72a8df40831ded821e3cbc79c318c21b44d6e51dff9fd89ace7a8249c5b462`.

## Verification

The failing catalog is the original three-engine owner run `39123458425`,
harness `6328647d8b7113a920d777044944612f3bc47a58`.
It retains the original recipe and all six launch failures.

The working image control is run `39126466632`,
harness `552831830ef2f059b7efa5149fe20f053428c220`.
The control copies the original Docker recipe into its private receipt directory
and changes only its base-image token to the digest-qualified matching image.
It builds the same `monochromatic-playwright` tag.
The original `prepare:playwright` task observes that existing image;
the original root owners and frozen configurations execute unchanged.
A separate Podman boundary wrapper adds 2 GiB RAM and 2 CPU caps to the actual containers.
No source fix,
CI variable propagation,
or assertion change is applied.

All eight browser specification files execute in each configured engine:

- Chromium passes 28 cases.
- Firefox passes 28 cases.
- WebKit passes 26 cases,
  fails the authored OPFS availability assertion with false versus true,
  and skips the authored fs-path OPFS write case.

The OPFS assertion is a real body outcome and remains red.
Both product page color-scheme screenshot assertions pass in all three engines.
The retained JSON reports include per-case status,
diagnostics,
stdout,
project configuration,
and exact source file identity.
The complete source-bound commands and logs are in
[`depot-complete-coverage-artifact-browser-receipts.json.gz`](../planning/depot-complete-coverage-artifact-browser-receipts.json.gz).

## Verified workaround

The measured consumer control selects this base in a copied recipe:

```dockerfile
# Copied research recipe; original repository recipe remains unchanged.
FROM mcr.microsoft.com/playwright:v1.64.0-noble@sha256:06a9939e57531807f8d5fd76ce44b53165ffb7d7501d87ab10e285c20b1e971f
```

The immutable workflow at `552831830ef2f059b7efa5149fe20f053428c220`
records the runnable copied-recipe task and actual root consumers.
The control qualifies matching-image execution,
not synchronization automation for future upgrades.
The existing WebKit assertion still fails.

## What does not work

- Counting test discovery as proof that browser bodies ran.
- Treating the successful image build as proof that the runner can find its own executable revisions.
- Crediting a WebKit skip as a passing OPFS write assertion.
- Treating this image control as a repair for rootless Podman UID/GID rejection.

## E2E discovery includes an inactive package

The same run invokes the unchanged root E2E owner for Chromium and Firefox.
Both fail before any E2E body with:

```text
Error: Cannot find package '@monochromatic-dev/or-throw-core' imported from /work/package-paused/webapp-edu/paper2vn/src/paper2vn.e2e.test.ts
```

The frozen `playwright.e2e.config.ts` matches `**/*.e2e.test.ts`
and declares no `testDir` or inactive-root exclusion.
Playwright `packages/playwright/src/common/config.ts:174`
selects the configuration directory when neither project nor configuration supplies `testDir`:

```typescript
// packages/playwright/src/common/config.ts:174
const testDir = takeFirst(pathResolve(configDir, projectConfig.testDir), pathResolve(configDir, config.testDir), fullConfig.configDir);
```

`config.ts:195` defaults `testIgnore` to an empty list:

```typescript
// packages/playwright/src/common/config.ts:195
testIgnore: takeFirst(projectConfig.testIgnore, config.testIgnore, []),
```

This preserves an actual repository discovery defect,
distinct from browser-image launch and WebKit OPFS behavior.
An additional source-owner control explicitly selects the four active DOM specification paths.
Run [`39126926119`](https://github.com/Aquaticats/Monochromatic/actions/runs/39126926119)
now retains that exact source-owner control:
17 Chromium cases and 17 Firefox cases pass with no skips or retries.
All four active DOM specifications execute.
The unfiltered root failure remains a separate observed discovery result.
The source-bound logs and parsed cases are retained in
[`depot-complete-coverage-finite-owner-receipts.json.gz`](../planning/depot-complete-coverage-finite-owner-receipts.json.gz).

## Upstream filing decision

Nothing is filed upstream.
The image mismatch is already tracked in repository issue [#1045][local-issue].
The E2E discovery scope is repository configuration.
The upstream source follows its declared configuration and executable checks.

- Upstream fault is not established.
- Upstream could change defaults,
  but no change is required to select a matching image or explicit specifications.
- The documented Docker integration supports matching versions.
- No upstream contribution is proposed,
  so contribution-policy and acceptance claims are not made.
- Existing repository evidence receives the measured consumer results;
  no upstream defect claim follows from them.
- No upstream fix prototype is applicable to this integration mismatch.

[docker-doc]: https://playwright.dev/docs/docker
[local-issue]: https://github.com/Aquaticats/Monochromatic/issues/1045
