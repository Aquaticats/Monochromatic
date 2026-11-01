# pnpm 11.5.1 to 11.15.1 strict `minimumReleaseAge` rejects immature packages in non-interactive installs

## Symptom

`mise run prepare:pnpm:install` can fail before dependency migration work completes when the lockfile already references
recent package versions and the session is non-interactive.

The observed failure was:

```text
[ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION] 4 lockfile entries failed verification
```

Affected entries in this run were:

```text
@earendil-works/pi-agent-core@0.78.1
@earendil-works/pi-ai@0.78.1
@earendil-works/pi-coding-agent@0.78.1
@earendil-works/pi-tui@0.78.1
```

The install succeeded with:

```bash
env PNPM_CONFIG_TRUST_LOCKFILE=true mise run prepare:pnpm:install
```

### Cached success verdict

pnpm 11.15.1 can report a successful cached verdict:

```text
✓ Lockfile passes supply-chain policies (verified 4d ago)
```

This does not mean pnpm checked different lockfile content four days ago.
The age describes a successful verdict for the same parsed lockfile content under policy
that every current verifier considers sufficient.
The current install still checks the lockfile identity and active policy before reusing that verdict.

### Fresh resolution without a lockfile

pnpm 11.15.1 also rejected an intentionally adopted dependency during fresh resolution:

```console
$ pnpm install --filter @monochromatic-dev/wg-cli-allowedips --lockfile=false --ignore-scripts
[ERR_PNPM_NO_MATURE_MATCHING_VERSION] 2 versions do not meet the minimumReleaseAge constraint:
  cidr-tools@12.1.3 ...
  ip-bigint@9.0.7 ...
```

The command explicitly disabled lockfile reads and writes,
so it had no reviewed resolution for `trustLockfile` to trust.
The accepted versions had already passed the source,
artifact,
runtime-graph,
and consumer validation recorded in
`doc/audit/tech-wg-allowedips-cidr-library-vet-2026-07-28.md`.
The matching bounded workaround is a version-specific exclusion for each exact audited release,
not a bare package-name exclusion.

## Root cause

The workspace enables `minimumReleaseAgeStrict`,
 so pnpm treats immature package resolutions as policy violations.
In pnpm 11.5.1,
 non-interactive strict installs cannot prompt,
 so the policy handler throws instead of adding
`minimumReleaseAgeExclude` entries.

`/var/home/user/.local/share/mise/installs/npm-pnpm/11/lib/node_modules/pnpm/dist/pnpm.mjs:167730`:

```javascript
const inCi = opts2.ci ?? import_ci_info2.isCI;
const canPrompt = !inCi && Boolean(process.stdin.isTTY);
return {
  handleResolutionPolicyViolations: async (violations) => {
    if (!strictMode)
      return;
    const immature = filterImmatureViolations(violations);
    if (immature.length === 0)
      return;
```

`/var/home/user/.local/share/mise/installs/npm-pnpm/11/lib/node_modules/pnpm/dist/pnpm.mjs:167742`:

```javascript
if (canPrompt) {
  await promptForApproval(immature);
} else {
  throw failOnImmature(immature);
}
```

The thrown error text comes from `failOnImmature`.

pnpm 11.15.1 retains that path at
`~/.local/share/mise/installs/pnpm/11.15.1/dist/pnpm.mjs:196967-196973`:

```javascript
function failOnImmature(immature) {
  const sorted = [...immature].sort((a2, b) => `${a2.name}@${a2.version}`.localeCompare(`${b.name}@${b.version}`));
  const list2 = sorted.map((v) => `  ${v.name}@${v.version} ${v.reason}`).join("\n");
  return new PnpmError("NO_MATURE_MATCHING_VERSION", `${sorted.length} ${sorted.length === 1 ? "version does" : "versions do"} not meet the minimumReleaseAge constraint:
${list2}`, {
    hint: "Run the install interactively to approve these picks, or add them to minimumReleaseAgeExclude in pnpm-workspace.yaml, or wait for the packages to mature past the configured cutoff."
  });
}
```

The original pnpm 11.5.1 source was at
`/var/home/user/.local/share/mise/installs/npm-pnpm/11/lib/node_modules/pnpm/dist/pnpm.mjs:167760`:

```javascript
function failOnImmature(immature) {
  const sorted = [...immature].sort((a2, b) => `${a2.name}@${a2.version}`.localeCompare(`${b.name}@${b.version}`));
  const list2 = sorted.map((v) => `  ${v.name}@${v.version} ${v.reason}`).join("\n");
  return new PnpmError("NO_MATURE_MATCHING_VERSION", `${sorted.length} ${sorted.length === 1 ? "version does" : "versions do"} not meet the minimumReleaseAge constraint:
${list2}`, {
    hint: "Run the install interactively to approve these picks, or add them to minimumReleaseAgeExclude in pnpm-workspace.yaml, or wait for the packages to mature past the configured cutoff."
  });
}
```

`PNPM_CONFIG_TRUST_LOCKFILE=true` maps to `trustLockfile`.
 The install path skips lockfile resolution verification when
that option is set.

`/var/home/user/.local/share/mise/installs/npm-pnpm/11/lib/node_modules/pnpm/dist/pnpm.mjs:162332`:

```javascript
if (!willDelegateToPacquet && !opts2.trustLockfile) {
  const cacheActive = opts2.cacheDir != null && opts2.resolutionVerifiers.length > 0;
  const wantedLockfilePath = cacheActive ? path128.resolve(ctx.lockfileDir, await getWantedLockfileName({
    useGitBranchLockfile: opts2.useGitBranchLockfile,
    mergeGitBranchLockfiles: opts2.mergeGitBranchLockfiles
  })) : void 0;
  try {
    await verifyLockfileResolutions(ctx.wantedLockfile, opts2.resolutionVerifiers, {
```

### Why an older cached verdict still verifies current content

Source audit used pnpm tag `v11.15.1`,
commit `331c26aa4bb8c12a2ce64ca989a9e3a73b571340`.
`pnpm11/installing/deps-installer/src/install/verifyLockfileResolutionsCache.ts:17-32`
defines parsed lockfile content as primary key and path metadata as shortcut:

```typescript
/**
 * On-disk cache of verifyLockfileResolutions results, keyed by lockfile
 * content hash. Lets repeat installs against an unchanged lockfile skip
 * the per-package registry round trips entirely — including across git
 * worktrees, where the same lockfile content lives at different paths.
 *
 * Two indexes share the same JSONL records:
 *
 * - **by content hash** — the primary index. Recognizing the same
 *   lockfile content regardless of where it sits on disk is what makes
 *   worktrees and lockfile copies hit.
 * - **by absolute path** — a same-machine stat shortcut.
 */
```

`pnpm11/installing/deps-installer/src/install/verifyLockfileResolutionsCache.ts:275-304`
shows that a stat miss hashes current content and accepts the cached record only when every active verifier
trusts cached policy:

```typescript
hash = key.hashLockfile()
const byHashRecord = indexes.byHash.get(hash)
if (!byHashRecord) return { hit: false, precomputed: { stat, hash } }
if (!everyVerifierTrustsCachedRun(byHashRecord, key.verifiers)) {
  return { hit: false, precomputed: { stat, hash } }
}

function everyVerifierTrustsCachedRun (record: CacheRecord, verifiers: readonly VerifierCacheIdentity[]): boolean {
  for (const verifier of verifiers) {
    if (!verifier.canTrustPastCheck(record.policy)) return false
  }
  return true
}
```

`pnpm11/installing/deps-installer/src/install/verifyLockfileResolutions.ts:133-157`
hashes parsed in-memory lockfile and emits cached verdict only after lookup succeeds:

```typescript
const hashLockfile = (): string => {
  if (cachedHash == null) cachedHash = hashObject(lockfile)
  return cachedHash
}
// ...
if (result.hit) {
  lockfileVerificationLogger.debug({
    status: 'cached',
    verifiedAt: result.verifiedAt,
    lockfilePath: options?.lockfilePath,
  })
  return
}
```

Finally,
`pnpm11/cli/default-reporter/src/reporterForClient/reportLockfileVerification.ts:32-35`
renders cache timestamp as successful policy result:

```typescript
if (log.status === 'cached') {
  return {
    msg: `${chalk.green('✓')} Lockfile${path_} passes supply-chain policies (${formatCachedVerdict(log.verifiedAt)})`,
  }
}
```

An earlier concern
that `verified 4d ago` meant newly introduced tarballs escaped current lockfile identity checking was wrong.
The source proves cache reuse requires same parsed lockfile content and acceptable cached policy.

## Verification

Versions under test:

```bash
mise exec -- pnpm --version
# 11.5.1 in original reproduction
# 11.15.1 in cached-verdict reproduction
```

Cached-verdict pattern:

```bash
mise run prepare:pnpm:install
# ✓ Lockfile passes supply-chain policies (verified 4d ago)
```

A disposable worktree containing identical lockfile content also reused the record by content hash:

```bash
pnpm install --frozen-lockfile --filter @monochromatic-dev/git-policy-cli...
# ✓ Lockfile passes supply-chain policies (verified 8m ago)
```

Both installs completed.
The second command used a different absolute path,
so the source-defined content-hash lookup rather than same-path stat shortcut explains the cache hit.

Failure pattern:

```bash
mise run prepare:pnpm:install
```

This fails in a non-interactive session when current lockfile entries violate the release-age verifier.

Working pattern used for the dependency migration:

```bash
env PNPM_CONFIG_TRUST_LOCKFILE=true mise run prepare:pnpm:install
```

This succeeded and produced the intended lockfile update for replacing `shell-quote` with `unbash`.

## Verified workarounds

### Trust the reviewed lockfile for one install

```bash
env PNPM_CONFIG_TRUST_LOCKFILE=true mise run prepare:pnpm:install
```

Tradeoff:
 pnpm skips lockfile resolution verification for that invocation.
 Use only when the lockfile diff is expected,
reviewed,
 and limited to the dependency change being made.

### Add explicit release-age exclusions

Add the rejected packages to `minimumReleaseAgeExclude` in `pnpm-workspace.yaml`,
 then rerun the normal install.

Use one entry per package,
 not one entry per version.
A bare name (`@scope/pkg`) exempts every version;
 a single `||` union (`@scope/pkg@1.0.0 || 1.1.0`) exempts the listed versions.
Listing the same name on multiple `name@version` lines does not work:
 pnpm honors only the first listed version per name and silently drops the rest,
 which is the exact shape `pnpm audit --fix` accumulates.
See `doc/troubleshooting/pnpm-minimum-release-age-exclude-first-match.md`.

Tradeoff:
 the exclusion is durable policy.
 Use it only when the immature version is intentionally accepted for future
installs too.

For an externally audited dependency,
prefer the exact form:

```yaml
minimumReleaseAgeExclude:
- 'cidr-tools@12.1.3'
- 'ip-bigint@9.0.7'
```

This admits only the inspected releases and leaves future versions behind the age gate.

### Wait for package maturity

Rerun the normal install after the package versions pass the configured release-age cutoff.

Tradeoff:
 this blocks urgent dependency work until registry age catches up.

## What does not work

Running the same install again in a non-interactive agent session does not help.
 pnpm cannot prompt,
 so it reaches
`failOnImmature` again.

Using `PNPM_CONFIG_TRUST_LOCKFILE=true` with `--lockfile=false` does not provide the reviewed-lockfile workaround.
The command has prohibited itself from reading a lockfile and must resolve the immature release again.

Using `--no-save` with strict release-age approval is not a substitute.
 The source has a separate guard for that case.

`/var/home/user/.local/share/mise/installs/npm-pnpm/11/lib/node_modules/pnpm/dist/pnpm.mjs:167738`:

```javascript
throw new PnpmError("STRICT_MIN_RELEASE_AGE_REQUIRES_SAVE", "minimumReleaseAgeStrict cannot be combined with --no-save: approval would require writing to minimumReleaseAgeExclude in pnpm-workspace.yaml, which --no-save prevents.", {
  hint: "Drop --no-save so the exclude list can be persisted, or set minimumReleaseAgeStrict: false to let the install proceed without prompting (the lockfile would still trigger the auto-collect on the next normal install)."
});
```

## Upstream filing decision

No entry in `.out-of-scope/` covers pnpm or lockfile verification.

Searches covered open and closed pnpm issues and pull requests for
`minimumReleaseAge`,
`trustLockfile`,
and lockfile verification cache behavior.
[pnpm issue 12324](https://github.com/pnpm/pnpm/issues/12324)
asked whether an update with `trustLockfile` had checked policy because it completed without output.
A maintainer explained that same-lockfile verification can reuse cached results.
[pnpm pull request 12326](https://github.com/pnpm/pnpm/pull/12326)
then added the visible cached-verdict message reproduced here.

1.  **Is it really upstream's fault?**
    No.
    Strict rejection enforces repository policy,
    while cached success is valid reuse keyed by current parsed lockfile content and acceptable policy.
2.  **Can upstream fix it?**
    Yes in general,
    and upstream already fixed the formerly silent cache reuse by displaying cached verdict age.
    No remaining defect is identified here.
3.  **Are they supporting this use case?**
    Yes.
    pnpm source tests cover same-path and cross-worktree content-hash hits,
    stricter-policy misses,
    and cached-verdict rendering.
4.  **Would the repository welcome our contribution?**
    Yes for a demonstrated defect with tests and a changeset.
    `CONTRIBUTING.md`,
    `.github/pull_request_template.md`,
    and `.github/ISSUE_TEMPLATE/bug-report.yaml` request those artifacts and contain no AI-assistance ban.
5.  **Will they likely fix it?**
    Not applicable to current behavior because no defect remains.
    Related cache and reporter fixes have already merged.
6.  **Have we prototyped a minimal compatible fix?**
    No prototype is appropriate because constraint one fails and the installed behavior matches upstream design.

There is nothing additive to post on issue 12324.
The maintainer comment and merged pull request already contain the source-backed explanation
and user-visible resolution.
No new issue or comment should be filed.

## pnpm 12.3.4: verification fetches `@jsr/*` metadata from `registry.npmjs.org` and fails with 404

Observed 2026-09-06 while installing a fresh `git worktree` of this repository
(main worktree installed under pnpm 11.21.0 per `node_modules/.modules.yaml`).
Both of these failed identically during "Verifying lockfile against supply-chain policies (781 entries)":

```bash
pnpm --dir "$WORKTREE" install --prefer-offline
pnpm --dir "$WORKTREE" install
```

```text
ERR_PNPM_META_FETCH_FAIL
ERR_PNPM_RESOLVING_NPM_RESOLVER_NETWORK_ERROR: Failed to fetch metadata from
https://registry.npmjs.org/@jsr%2Fstd__internal: HTTP status client error (404 Not Found)
```

The effective configuration maps the `@jsr` scope to `https://npm.jsr.io/`,
yet the verification step queried `registry.npmjs.org` for the scoped package.
`--config.minimum-release-age=0` failed the same way on `@jsr/std__path`.
`--prefer-offline` also failed earlier in the day with `ERR_PNPM_NO_OFFLINE_META` on `@esbuild/darwin-arm64`,
a platform-specific optional dependency whose metadata is not cached on Linux.

Install succeeded with the same skip the sections above use:

```bash
pnpm --dir "$WORKTREE" install --trust-lockfile
```

`pnpm install --help` describes `--trust-lockfile` as
"Skip verifying the lockfile against supply-chain policies".

Not investigated:
why the verifier ignores the scope registry mapping for `@jsr` packages under pnpm 12.3.4.
The pnpm source was not read for this section;
treat the registry attribution as an observation from the error text only.
