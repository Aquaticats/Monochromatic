# git-policy-cli 0.0.1 stopped a package-split commit in dependent-version-bump decoding

## Symptom

During the `module-test` extraction,
`git commit` stopped before forwarding with exit code 2:

```json
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"plugin-threw","message":"The \"list\" argument must be an instance of SharedArrayBuffer, ArrayBuffer or ArrayBufferView.","trigger":"pre-forward","policyId":"mono/dependent-version-bump"}
```

The selected changes included the existing runner manifest and new manifests for
`module-test-expect`,
`module-test-sandbox`,
and `module-test-diagnostic`.
This was an engine failure,
not a `dependent-version-stale` policy finding.

## Source evidence and diagnosis limits

The repository manifest identifies `@monochromatic-dev/git-policy-cli` as version `0.0.1`.
The exact trusted policy snapshot revision was not captured,
so source inspection does not prove which decoder input caused this invocation to fail.

The current policy reads current and previous manifest content in
`package/git-policy/repository/src/dependent-version-bump-policy.ts:82`:

```ts
// package/git-policy/repository/src/dependent-version-bump-policy.ts
async function manifestFileOf(file: TrackedFile,): Promise<WorkspaceManifestFile> {
  const headBytes = await file.headBytes();
  return {
    path: file.path,
    text: DECODER.decode(await file.bytes(),),
    ...(headBytes === ABSENT_GIT_VALUE ? {} : { baseText: DECODER.decode(headBytes,), }),
  };
}
```

The generated counterpart is
`package/git-policy/cli/src/optional/repository-policy/dependent-version-bump-policy.ts`.
This identifies a relevant decoding path,
not a confirmed root cause.
Do not conclude that missing-manifest sentinels
or stale snapshots caused the error without inspecting the executing inputs.
See [trusted snapshot behavior](cli-git-trusted-snapshot-stale-policy-code.md).

## Verification record

Observed failing case:
a pathspec-scoped commit introducing the extracted package manifests,
without any policy escape.
The diagnostic named `mono/dependent-version-bump` and no commit landed.

Observed successful cases:

- The extraction committed as `79a3565fc` with only that policy escaped.
- The existing-file sandbox-test correction committed as `2aea88562`
  without any escape.

These are recorded invocations,
not a minimized reproducer.
Repeating the original command after the new manifests are already committed does not recreate the same candidate state.

## Verified workaround

The CLI documents policy-specific one-invocation escapes in
`package/git-policy/cli/README.md:729`.
The successful extraction used:

```sh
# From the repository root, with the extraction changes selected.
git commit --no-enforce-mono/dependent-version-bump \
  --message 'refactor(*): split module-test responsibilities into packages' \
  -- package/test-harness/core package/test-harness/core-expect package/test-harness/core-sandbox \
  package/test-harness/core-diagnostic .changeset/config.json doc/planning/module-test-split.md
```

The actual commit also supplied its package-by-package body.
The escape skips the named policy for that invocation;
other checks remain enabled.
The tradeoff is that the dependent-version check did not validate this commit.
No existing package version was intentionally bumped by the extraction.
No policy source,
trust record,
or persistent enforcement configuration was changed.

## What does not establish a fix

A later successful existing-file commit does not prove new-manifest handling is fixed.
The narrower input differs.
No CLI rebuild,
trust refresh,
or policy implementation change was attempted for this task.

## Upstream filing decision

No external upstream issue was filed.
This is a repository-owned policy,
and the observation has not been reduced to a confirmed cause or patch.
The external filing gates are not satisfied:
external attribution and an upstream target are absent;
upstream fixability,
support,
contribution policy,
and willingness were not assessed;
no candidate fix was prototyped.
The durable artifact is this local incident record,
not an upstream allegation.

## Second reproduction: multi-package commits crash the same policy

During the search-fetch MCP extraction (2026-10-05),
two `git commit` invocations stopped before forwarding with the identical engine failure:

```json
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"plugin-threw","message":"The \"list\" argument must be an instance of SharedArrayBuffer, ArrayBuffer or ArrayBufferView.","trigger":"pre-forward","policyId":"mono/dependent-version-bump"}
```

Both invocations named more than one package in one commit:

- the core extraction, spanning `package/search-fetch/core`,
  `package/search-fetch/pi-plugin`,
  and `pnpm-lock.yaml`
- the server package, spanning `package/search-fetch/mcp`,
  `package/search-fetch/pi-plugin/package.json`,
  and `pnpm-lock.yaml`

Verified workaround without any policy escape:
split the commit per package and let `pnpm-lock.yaml` ride with one of them.
Every single-package commit passed all policies unchanged,
including `dependent-version-bump`.
This is preferable to the `--no-enforce-mono/dependent-version-bump` escape above
whenever the change genuinely spans packages,
because the policy still runs on each commit.

Repeated observation only;
it does not establish a cause.
