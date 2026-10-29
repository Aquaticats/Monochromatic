# cli-git 0.0.1 declared Node 22.18 support but its built artifact needed Node 24 language and APIs

## Resolution policy

Issue #429 aligned cli-git's declared floor with the latest published Node LTS line.
Issue #475 keeps that exact floor and adds the verified newer Node line.
`package/git-policy/cli/package.json` is the source of truth and carries one caret branch per supported line,
currently `^24.11.0 || ^26.0.0`.
The lowest branch is the maintained minimum supported runtime
and the build derives its `node24.11.0` transform target from it.

`package/git-policy/cli/src/runtime-contract.host-evidence.ts`
requires execution at the exact floor of any declared line,
imports the built public MJS without extra output,
and checks successful help plus invalid-usage behavior.
`.github/workflows/cli-git-trust.yml` built and ran unit and host evidence at every declared floor;
the workflow was deleted on 2026-10-13 in commit `642b6d72cb`, together with the TypeScript wrapper it tested.
A separate CI check compares the lowest declared branch with Node's official release index each day,
so a newly published LTS line that should advance the maintained minimum fails visibly.

### Landed verification

The runtime policy and scoped CI evidence landed in commits `6a43312ff`,
`e092d20bb`,
`b27889c38`,
and `1848ce73a`.
Node's official release index on 2026-08-28 reported Node 24.20.0 as latest LTS,
and its first release carrying the Krypton LTS marker was Node 24.11.0.
Issue #475 additionally verifies the Node 26 line,
so the declared range is now `^24.11.0 || ^26.0.0` and CI exercises both exact floors.

The following package-scoped commands passed with `MISE_NODE_VERSION=24.11.0`:

```sh
mise run //package/git-policy/cli:build
mise run //package/git-policy/cli:test:unit
mise run //package/git-policy/cli:verify:supported-runtime
```

The build accepted manifest-derived target `node24.11.0`.
Unit execution completed at Node 24.11.0,
and minimum-runtime host evidence confirmed:

- side-effect-free import of `dist/final/node/index.mjs` with `definePolicy` exported;
- management help exited `0` with authored help;
- invalid trust usage exited `2` with authored usage on stderr.

The guard had two positive controls after its first commit:

- running host evidence under repository Node 26.7.0 failed
  before artifact execution because it was not exact declared floor;
- temporarily restoring `^22.18.0`,
  rebuilding through derived `node22.18.0` target,
  and running on Node 22.18.0 failed representative invalid CLI behavior with exit `1`.

Restoring `^24.11.0`,
rebuilding,
and rerunning Node 24.11.0 host evidence passed.
Type checking passed.
Package Oxlint reported no finding in changed TypeScript files;
its package-wide task remained nonzero on existing `test-import(require-eventual-artifact)` diagnostics in other tests.
Repository-wide dprint likewise named only existing unrelated files.
The Markdown check passed for this record and package README.

Initial CI run `33218859206` built at Node 24.11.0 but did not reach host evidence:
the separate manual-push integration fixture reported that
`/usr/bin/git push --atomic origin --dry-run --verify` exited `128` on GitHub's runner.
That fixture is not part of runtime-contract acceptance.
Commit `1848ce73a` scopes minimum-runtime source execution to public authoring API and entry-point tests,
then runs dedicated built import and CLI host evidence.
The complete unit task remains package-scoped local evidence at Node 24.11.0.

Final CI run [33219232827][runtime-contract-ci] passed on commit `1848ce73a`.
Its runtime-policy,
latest-LTS,
minimum-runtime,
and Linux,
macOS,
and Windows trust jobs all completed successfully.
The minimum-runtime job built at Node 24.11.0,
ran public API and entry-point source tests,
imported built public MJS without side effects,
and exercised successful help plus invalid usage.

The remaining sections preserve the pre-resolution diagnosis and Node 22 evidence.

## Symptom

Before issue #429,
`package/git-policy/cli/package.json` declared:

```json
{
  "engines": {
    "node": "^22.18.0 || >=24.11.0"
  }
}
```

The unchanged built package cannot be imported by Node 22.18.0.
Node's ESM parser emits:

```text
SyntaxError: Unexpected identifier 'output'
```

The affected artifact line is an `await using` declaration:

```js
await using output = await open(snapshotPath, "w", PRIVATE_FILE_MODE$4);
```

A source-level cli-git import can get past that bundled parse boundary,
but then Node 22.18.0 emits another error:

```text
TypeError: Error.isError is not a function
```

CAC did not introduce either operation.
The unchanged cli-git baseline and the disposable CAC integration both fail the built import.

## Root cause

### Production source uses Node 24 language and API surfaces

Production cli-git contains explicit resource management declarations,
including:

- `package/git-policy/cli/src/management.ts` in `runManagementCommand`;
- `package/git-policy/cli/src/worktree-copy/entry-compare.ts`;
- `package/git-policy/cli/src/worktree-copy/lifecycle.ts`;
- trust registry,
  transaction,
  and direct-fix modules.

Production cli-git and shared caught-value code also call `Error.isError`,
including:

- `package/git-policy/cli/src/bin.ts`;
- `package/git-policy/cli/src/worktree-copy/entry-compare.ts`;
- `package/git-policy/cli/src/worktree-copy/journal-lock.ts`;
- `package/caught-value/core/src/index.ts`.

The [Node 24.0.0 release notes][node-24] identify explicit resource management and `Error.isError` as features
added with Node 24's V8 13.6 update.

### Build targeting follows repository Browserslist output

`package/config/rolldown/src/index.node.ts` resolves its transform target through
`browserslistTargets({ runtime: 'node' })` and gives that target to Rolldown.
The measured generated target for this build lists Node 26 releases.
Rolldown therefore preserves language and API use that Node 26 accepts;
it does not derive output compatibility from the consuming package's `engines.node` field.

The final artifact consequently retains `await using`.
No build step polyfills `Error.isError`.

### Manifest and artifact contracts diverged

The manifest still advertises a Node 22 branch while source and output use features first shipped in Node 24.
Package-manager engine metadata does not change parser support or build output.

## Verification

### Exact environment

The lower-bound probe used official image `docker.io/library/node:22.18.0-slim`:

```text
resolved digest: sha256:0d130e2ee18e88e1561375276daced6bff032539200173f2daf48c2e33f38ff5
image ID: 955b467cb9a2a941cb181f7cf1d2405c1dd24b4566a3598b7eae7ecca1a769d1
architecture: amd64
OS: linux
```

The container had no network,
a read-only root,
2 GiB memory,
2 CPUs,
a 128-process ceiling,
and a 1,024-file-descriptor ceiling.

### Runnable built-artifact harness

Build the package through its repository task:

```sh
mise run //package/git-policy/cli:build
```

Then import the public application artifact under Node 22.18.0:

```sh
podman run \
  --memory=2g \
  --cpus=2 \
  --pids-limit=128 \
  --ulimit nofile=1024:1024 \
  --rm \
  --network none \
  --read-only \
  --volume "$PWD:/work:ro" \
  --workdir /work \
  docker.io/library/node:22.18.0-slim \
  node --input-type=module --eval \
  "await import('file:///work/package/git-policy/cli/dist/final/node/index.mjs')"
```

Observed result:

```text
exit=1
SyntaxError: Unexpected identifier 'output'
```

The unchanged baseline stderr SHA-256 is
`37b72db335be0c716f8b24248cf7cddd0bb082017358e89c89b00af6d3f18a47`.
The CAC-integration artifact produced the same diagnostic at a shifted bundle line;
its stderr SHA-256 is
`02ca3774d5b3beadc967a14d6e88d842db192665a035b914549a7c2990327f1a`.

### Source-level confirmation

The 52-case CAC integration harness under Node 22.18.0 exited `1` in shared logger initialization:

```text
TypeError: Error.isError is not a function
```

This second failure confirms that removing only the `await using` syntax would not restore the declared runtime floor.

### Positive control

Linux/Node 26.7.0 imported the candidate artifact with no stderr,
exposed `definePolicy` as a function,
ran authored management and trust help,
and returned exit `2` for invalid trust,
check,
and fix options.
The maintained Linux/Node 24.18.0 built-trust consumer and lifecycle benchmark also executed the packed candidate
successfully.

### Working catalog

- Node 26.7.0 typed parity,
  candidate package import,
  built management help,
  invalid-usage contracts,
  type checking,
  and unit execution.
- Node 24.18.0 built-trust consumer and maintained lifecycle benchmark.

### Failing catalog

- Node 22.18.0 unchanged baseline package import:
  `await using` syntax error.
- Node 22.18.0 CAC candidate package import:
  same syntax error.
- Node 22.18.0 source-level CAC parity harness:
  missing `Error.isError`.

## Remediation options

### Raise cli-git's declared floor to Node 24.11.0 (recommended)

Pros:

- makes package metadata truthful for current source and built output;
- matches the verified Node 24.18.0 built consumer and benchmark plus the Node 26.7.0 import and command probes;
- avoids replacing widespread explicit resource management and error classification.

Cons:

- removes the currently advertised Node 22 branch;
- requires all consumers to use the Node 24 branch already present in the engine range.

This is the minimal verified alignment:
the unchanged packed artifact executes in the Node 24.18.0 lifecycle benchmark and fails import on Node 22.18.0.
The package is unpublished,
so changing its declared floor is a design correction rather than a published compatibility break.

### Preserve Node 22 with an engine-derived build and compatibility layer

Pros:

- retains the stated Node 22.18 consumer range;
- makes output targeting follow package support instead of the repository's current Node target.

Cons:

- requires proving Rolldown correctly downlevels every explicit resource management path;
- requires replacing or compatibility-wrapping every runtime `Error.isError` call;
- needs rebuilt import,
  CLI,
  cleanup,
  exception,
  trust,
  worktree,
  and platform tests on Node 22;
- is not validated by this investigation.

Ranking:
raising the floor is first because it matches the measured artifact and existing source design with a verified runtime.
Preserving Node 22 is second because it retains broader support
but requires an unbuilt cross-package compatibility change.

## What does not work

- Adding CAC does not fix cli-git's existing artifact syntax.
  CAC itself supports Node `>=20.19.0`.
- Changing only `engines.node` does not transpile output.
- Polyfilling only `Error.isError` does not make Node 22 parse `await using`.
- Removing only `await using` does not provide `Error.isError`.
- Node's TypeScript type stripping does not transform an already built MJS artifact.
- Upstream CAC Windows and Node 22 test passes do not validate the enclosing cli-git artifact.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   Node 22 rejects features that Node's release notes identify with Node 24,
   while cli-git advertises Node 22 and targets newer output.
2. **Can upstream fix it?**
   No external fix is required.
   The repository owns its engine declaration,
   build target,
   and compatibility choices.
3. **Are they supporting this use case?**
   Not applicable.
   Node 24 supports the used features;
   Node 22 is the package's own claimed floor.
4. **Would the repository welcome our contribution?**
   Not applicable because the relevant repository is this project.
5. **Will they likely fix it?**
   Not applicable to Node or CAC.
6. **Have we prototyped a minimal compatible fix?**
   The minimal metadata alignment is verified by the Node 22 failure,
   Node 24.18.0 built consumers,
   and Node 26.7.0 import and command probes.
   A true Node 22 compatibility implementation has not been prototyped.

Do not file an issue against Node or CAC.
Track any engine-floor or downleveling decision in this repository.

## Node 26.10.0 copied into a Node 24 fixture lacked a shared library

### Symptom and boundary

During the cli-git rewrite evaluation on 2026-10-04,
a disposable fixture copied the host's Node 26.10.0 executable over a Node 24.11.0 image's executable.
The Linux dynamic loader failed before any JavaScript or performance measurement ran:

```text
# Dynamic-loader stderr from the disposable policy fixture
node: error while loading shared libraries: libatomic.so.1: cannot open shared object file: No such file or directory
```

The container command exited 127.
This was an agent-authored fixture packaging error,
not the reported slow-commit incident and not a Node language compatibility failure.

### Evidence and verification

On the host,
`ldd` on the exact copied executable reported:

```text
# ldd on the host's Node 26.10.0 executable
libatomic.so.1 => /usr/lib64/libatomic.so.1
```

The failing fixture extended local image
`9b764e5240b8c4721640320ac499b3b3440612e3109481fb2df8ce48705f51e8`,
which was built on the Node 24.11.0 cli-git end-to-end runtime.
Copying the executable did not supply its shared-library dependencies.
This conclusion uses the loader diagnostic and the exact binary's dependency listing;
no upstream source defect is asserted.

The corrected fixture uses the locally installed `node:26-slim` image,
ID `8469107207880d917ddc60bb6aa76f35bb44965420f266840e1dea8758ff186b`,
and copies the same host Node executable into that runtime.
It then copies the existing Git executables,
installed cli-git package,
and benchmark inputs into the image.
That combination launched Node 26.10.0,
executed the Markdown CLI,
and reached cli-git's configuration validation.
A subsequent `Policy markdown/autofix options failed Valibot validation` error was a separate fixture-input error,
not a shared-library failure.

### Workaround and limits

Use a runtime image that supplies the copied executable's dependencies,
then verify the actual executable and consumer invocation.
Do not treat an executable file alone as the complete Node runtime distribution.
Changing the image also changes the userspace environment;
measure all comparison variants in that same corrected image.
The configuration error does not establish a passing performance run.

### Upstream filing decision

1.  Upstream fault:
    no;
    the evaluation fixture replaced a binary without its required runtime libraries.
2.  Upstream fix:
    unnecessary;
    the fixture image is repository-owned.
3.  Supported use:
    no claim is made that a copied executable supports an arbitrary older runtime image.
4.  Contribution policy:
    not applicable to this local fixture correction.
5.  Upstream willingness:
    not applicable;
    no upstream change is requested.
6.  Prototype:
    the corrected image launches the exact copied Node version and reaches application code.

No upstream filing or draft is warranted.
Reproduction inputs and subsequent measurement scope are recorded in
[`cli-git-rust-rewrite.md`](../planning/cli-git-rust-rewrite.md).

[node-24]: https://nodejs.org/en/blog/release/v24.0.0
[runtime-contract-ci]: https://github.com/Aquaticat/Monochromatic/actions/runs/33219232827
