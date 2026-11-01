# Checkout de0fac2 and Node 26 service jobs fail before consumer execution

## Symptom

The frozen-source service profile encountered separate prerequisites before its real HTTP consumer.
The [prerequisite receipt](../audit/depot-container-prerequisites-2026-10-18.evidence.json)
keeps the failures distinct:

- GitHub run `39108872769` failed with zero jobs.
  Its source placed `job.services` in job-level `env`.
  The retained APIs do not recover its original hosted parser diagnostic.
- Job containers in `39109462308` printed checkout's diagnostics:

  ```text
  # actions/checkout at de0fac2e4500dabe0009e67214ff5f5447ce83dd
  The repository will be downloaded using the GitHub REST API
  To create a local Git repository instead, add Git 2.18 or higher to the PATH
  ```

  The containers did not complete that checkout before their bounds.
  This does not establish the cause of the REST download stall.
- After adding checkout tools,
  `39112162357` reached installation of the repository's Node `26.12.0`,
  then the dynamic loader reported:

  ```text
  # Node 26.12.0 loader stderr retained by mise install node
  error while loading shared libraries: libatomic.so.1: cannot open shared object file: No such file or directory
  ```

  Mise reported exit `127`.
  The base image's working Node `24.22.0` did not qualify this different executable.

## Source trace and limits

The inspected checkout source is commit `de0fac2e4500dabe0009e67214ff5f5447ce83dd`.
All captured source byte counts,
SHA-256 and Git blob hashes reproduce.
Fresh official Git trees bind the blobs to that commit;
the same files were read without modification in an existing upstream clone.

`actions/checkout` `src/git-source-provider.ts:77` selects the fallback when no command manager is returned:

```ts
// actions/checkout src/git-source-provider.ts:77, abridged.
if (!git) {
  core.info(`The repository will be downloaded using the GitHub REST API`)
}
```

Its `src/git-source-provider.ts:356` catches initialization errors without preserving their diagnostic
unless LFS requires propagation:

```ts
// actions/checkout src/git-source-provider.ts:366.
} catch (err) {
  if (settings.lfs) {
    throw err
  }
  return undefined
}
```

At `src/git-command-manager.ts:670`,
initialization finds Git and executes its version command:

```ts
// actions/checkout src/git-command-manager.ts:670, abridged.
this.gitPath = await io.which('git', true)
let gitOutput = await this.execGit(['version'])
```

The same method rejects an unparsable version at line `683`
and an insufficient version at line `688`.
The deciding checks are:

```ts
// actions/checkout src/git-command-manager.ts:683 to 691.
if (!this.gitVersion.isValid()) {
  throw new Error('Unable to determine git version')
}
if (!this.gitVersion.checkMinimum(MinimumGitVersion)) {
  throw new Error(
    `Minimum required git version is ${MinimumGitVersion}. Your git ('${this.gitPath}') is ${this.gitVersion}`
  )
}
```

Its minimum at line `15` is:

```ts
// actions/checkout src/git-command-manager.ts:15.
export const MinimumGitVersion = new GitVersion('2.18')
```

Thus the fallback log alone does not distinguish absent Git,
version/initialization failure or a later download problem.
The independent
[original-image control](../audit/depot-original-service-git-2026-10-18.evidence.json)
proves Git `ENOENT` and absent default-PATH candidates in that exact AMD64 image,
with working-Node and deliberately absent-command controls.
It does not directly observe the historical job's PATH or establish Arm64 absence.

The current official
[context availability table](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts)
excludes `job` from `jobs.<job_id>.env`
and includes it in `jobs.<job_id>.steps.env`.
This explains the context-placement correction without inventing the missing parser message.

The Node runtime correction belongs to the repository-owned derived image,
not checkout's Git initialization.
At producer `7590bcde15e5ceef4dc36f4e2c3d975e1c1b833f`,
`.depot/probe/platform-buildkit/ci-node26-context/Dockerfile:4`
contains this decoded `RUN` body:

```js
// .depot/probe/platform-buildkit/ci-node26-context/Dockerfile:4, decoded excerpt.
execFileSync('apt-get', ['update'], { stdio: 'inherit', env });
execFileSync('apt-get', ['install', '--assume-yes', '--no-install-recommends', 'libatomic1'], {
  stdio: 'inherit', env,
});
```

The actual consumer at `5485cc0d12805f330d568b358e5ff0bb988d4d37`,
`.depot/research/service/mise.toml:49`,
reads loaded library mappings and rejects a different Node version,
executable hash or missing `libatomic` mapping.
The actual admission condition is:

```js
// .depot/research/service/mise.toml:50.
if (process.version !== 'v26.12.0' || actualNodeSha256 !== tools.repositoryNode.binarySha256 || libatomicMappings.length === 0) throw new ServiceConsumerFailure('Actual repository Node binary or loaded libatomic mapping does not match the qualified image prerequisite');
```

The receipt retains those actual runtime measurements.

## Verification catalog

The complete owning harness is `.depot/research/service/mise.toml`
at `5485cc0d12805f330d568b358e5ff0bb988d4d37`.
Its actual consumer command is:

```sh
# In the immutable harness checkout with its workflow-provided service environment.
mise run //.depot/research/service:consume
```

The source workflow supplies the nonce,
expected architecture,
endpoint and host/job-container mode.
The service producer is `5cfc1d90f92154cf62fa95789fad9bd7deeda208`;
the frozen application marker is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Replaying a private-registry profile requires its appropriate disposable pull credentials;
their historical absence/cleanup is not permission to recreate account-wide settings.
This source/receipt review allocates no cloud run.

Passing catalog:

- Native `g02lgcbfgq`:
  four finished host/job-container consumers across x64 and Arm64.
- GitHub `39113547458`:
  six successful consumers across standard GitHub x64 and Depot-managed x64/Arm64.
- Every consumer:
  nonce/source/architecture-qualified health and nonce HTTP `200`,
  plus an actual missing-path HTTP `404`.
- Every job-container consumer:
  actual Node `v26.12.0`,
  matching executable hash,
  loaded `libatomic` and Git `2.39.5`.
  These are separate from the server/base Node `v24.22.0`.

Failing catalog:

- Unsupported context placement:
  workflow failure and no jobs,
  without a recovered original parser diagnostic.
- Original container checkout:
  REST fallback and incomplete download before the bound,
  with download cause unresolved.
- Earlier derived image:
  successful checkout followed by the distinct Node loader failure.

The [independent review](../audit/depot-independent-policy-process-mvm-review-2026-10-18.md)
checks the original ZIPs,
source/runtime joins and response/proof digests.
These functional fixtures do not execute the repository's application suites.

## Verified workarounds

The exact
[immutable workflow patch](depot-service-job-container-prerequisites.patch)
from `6b3c75517e798161e79ddb7e496b78cf1ffc4240`
to `1438ecde4b643a606e0a75f85067e3d0153e9bb5`
moves the dynamic endpoint to the consuming step.
It also binds the job-container endpoint at the consuming step.

The tradeoff is step-local availability;
other steps needing that value must bind it in their own supported context.
This correction does not repair the separate image prerequisites.

The verified derived image installs checkout tools and the Node runtime dependency,
then the workflow selects immutable digest
`sha256:853b99050d5420698a02d7cb10978be8824dc0999c9ace08ec453706da54ec31`.
The dependency correction's source recipe is retained at the producer path named in the source trace.
The tradeoff is ownership of this additional image,
its package inputs,
maintenance and source/digest admission.
Its Git `2.39.5` meets the exercised checkout boundary,
not the Git-policy suite's exact Git `2.56` requirement.

## What does not work

- Treating base Node 24 health as proof that repository Node 26 loads.
- Inferring a REST download root cause from checkout's fallback message.
- Generalizing an AMD64 default-PATH control to the historical environment or Arm64.
- Treating context placement,
  Git initialization and Node's dynamic loader as one incident.
- Repeating application checks or changing product source to hide a prerequisite failure.

## Upstream filing decision

The `.out-of-scope/` listing contains no checkout/service-specific exemption.
The upstream search for `repo:actions/checkout "REST API" git` located
[issue 2435](https://github.com/actions/checkout/issues/2435)
and [PR 2560](https://github.com/actions/checkout/pull/2560).
Their complete bodies and empty comment lists were read.
They concern omitted initialization diagnostics;
they do not diagnose this profile's REST download stall.
Nothing additive is proposed for those threads.

- Responsibility:
  unsupported context placement and missing image/runtime dependencies are owned configuration boundaries.
  The download cause is unresolved;
  the separate upstream diagnostic gap is already tracked.
- Fixability:
  the measured consumer corrections work;
  no source-backed stall repair has been established.
- Supported use:
  the documented step context and checkout fallback are supported;
  the profile must supply the binaries/libraries it actually selects.
- Contribution policy:
  `CONTRIBUTING.md` describes contribution steps,
  but the pinned `README.md:31` explicitly says the repository is not taking contributions.
  No new filing or comment is authorized or sent.
- Maintainer direction:
  the README directs support to Community Discussions;
  open diagnostic PR 2560 is not evidence that its patch is merged into the pinned version.
- Prototype:
  no upstream stall defect is established and the contribution gate is closed.
  No third-party source was changed or speculative upstream patch created.

The upstream artifact is this explicit nothing-to-add decision,
not a new issue draft.
