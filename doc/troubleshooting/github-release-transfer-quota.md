# GitHub CLI 2.102.0 masks release quota failures and misses transferred release attestations

## Symptom

On 2026-10-15 local time,
or 2026-10-16 UTC,
the repository's first Depot-backed forbidden-strings scan failed while installing its published scanner.
The repository had moved from `Aquaticat/Monochromatic` to `Aquaticats/Monochromatic` on 2026-10-15.
These are separate failures at different boundaries:

- `gh release view forbidden-strings-v0.5.0 --repo Aquaticats/Monochromatic` returned
  `release not found` and exit status `1`,
  although the public release and asset were present.
- Repository-scoped attestation discovery returned HTTP `404` for the scanner archive,
  while the former owner's user-scoped endpoint still returned its signed bundles.

The [diagnostic cloud job][diagnostic-job] used `gh version 2.102.0 (2026-09-30)`.
Its retained log,
`completed-116744386136.log:425` through `439`
under the session's `depot-trial-2026-10-15` scratch directory,
records authenticated release listing returning `403`,
anonymous listing returning `200`,
anonymous tag lookup returning `200`,
asset download returning `200`,
and the original `gh release view` invocation still returning `release not found`.

The contemporaneous Scorecard job emitted
`RequestError [HttpError]: API rate limit exceeded for installation`.
Its response had `x-ratelimit-limit: 5000`,
`x-ratelimit-used: 5000`,
`x-ratelimit-remaining: 0`,
and `x-ratelimit-resource: core`.
The recorded request was
`GET /repos/Aquaticats/Monochromatic/actions/runs/38828872347/jobs?filter=latest`.
This proves exhausted installation REST quota at that time.
It does not identify which consumers spent the quota or establish that Scorecard spent it alone.

## Root cause

### Release lookup discards a more informative error

The audited upstream source is `cli/cli` tag `v2.102.0`,
commit `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`.
A separate read-only checkout of current upstream commit
`3d7967df5a94052b0554898ffa02526f0f90b8c9` contains the same selector.

In upstream `pkg/cmd/release/shared/fetch.go:184`,
the published lookup uses the REST release-by-tag route:

```go
// pkg/cmd/release/shared/fetch.go:184
publishedPath, err := safeurl.JoinPath("repos", repo.RepoOwner(), repo.RepoName(), "releases", "tags", tagName)
```

The calls at `pkg/cmd/release/shared/fetch.go:194` and `200`
invoke the published and draft lookups:

```go
// pkg/cmd/release/shared/fetch.go:194, 200
release, err := fetchReleasePath(cc, httpClient, repo.RepoHost(), publishedPath)
release, err := fetchDraftRelease(cc, httpClient, repo, tagName)
```

The draft lookup can return the not-found sentinel even when GraphQL found a published release.
At `pkg/cmd/release/shared/fetch.go:259`:

```go
// pkg/cmd/release/shared/fetch.go:259
if query.Repository.Release == nil || !query.Repository.Release.IsDraft {
    return nil, ErrReleaseNotFound
}
```

The REST helper preserves non-404 errors at `pkg/cmd/release/shared/fetch.go:277`:

```go
// pkg/cmd/release/shared/fetch.go:277
if err != nil {
    if isNotFound(err) {
        return nil, ErrReleaseNotFound
    }
    return nil, err
}
```

`isNotFound` checks HTTP status `404` at `pkg/cmd/release/shared/fetch.go:297`:

```go
// pkg/cmd/release/shared/fetch.go:297
httpErr, ok := errors.AsType[api.HTTPError](err)
return ok && httpErr.StatusCode == http.StatusNotFound
```

After both lookups fail,
the original selector at `pkg/cmd/release/shared/fetch.go:220`
prefers a not-found result over the other error:

```go
// pkg/cmd/release/shared/fetch.go:220, unmodified v2.102.0
// Both lookups failed; prefer reporting the release as not found.
if errors.Is(second.error, ErrReleaseNotFound) {
    return nil, second.error
}
return nil, first.error
```

Consequently,
REST `403` plus draft not-found becomes `release not found` in either completion order.
The prototype demonstrates this error-selection defect directly.
The cloud incident is consistent with that path,
but the retained cloud probe did not capture both internal responses from the same `gh release view` call.
Do not describe the exact internal cloud request sequence as directly observed.

### Attestation discovery and signed identity are different boundaries

Upstream `pkg/cmd/attestation/api/client.go:117` and `123`
route repository lookup through `repos` and owner lookup through `orgs`:

```go
// pkg/cmd/attestation/api/client.go:117, 123
u, err = safeurl.JoinPath("repos", owner, name, "attestations", params.Digest)
u, err = safeurl.JoinPath("orgs", params.Owner, "attestations", params.Digest)
```

Those branches do not try the personal-account `users` route.
GitHub documents a separate [user attestation API][user-attestations].
For the scanner's digest,
both repository names and the new organization's endpoint returned `404` during investigation,
whereas `users/Aquaticat/attestations/<digest>` returned two bundles.
This disproves the hypothesis that the original attestation evidence was lost.

The verified statement identifies repository `710240121`,
source commit `22a1ff40b3d2a0a649507d01924e26a5e2720d6f`,
and the original signer
`Aquaticat/Monochromatic/.github/workflows/cargo-publish.yml@refs/heads/main`.
Its invocation was run `38410341575`,
attempt `1`.
The original name remains part of that signed historical identity;
the repository's current name is `Aquaticats/Monochromatic` after the 2026-10-15 transfer.

The public client source explains which routes the CLI requests.
It does not explain GitHub's server-side indexing after a repository transfer.
The backend reason for the differing endpoint results remains unproven.

## Verification

### Observed working and failing cases

- Public anonymous release-by-tag lookup,
  release-by-ID lookup,
  and asset download returned `200` for
  `forbidden-strings-v0.5.0`,
  `monochromatic-lint-v0.1.3`,
  `monochromatic-git-policy-cli-v0.1.4`,
  and `monochromatic-nested-wayland-session-v0.6.0`.
- Authenticated release listing returned `403` in the same diagnostic job.
  The original `gh release view` returned `release not found`.
- Original scanner bytes plus the recovered bundle and original signer policy verified successfully.
- A copy with one byte changed failed verification with exit status `1`.
  An unrelated repository and signer policy also failed with exit status `1`.
  Both negative controls emitted `Error: verifying with issuer "sigstore.dev"`.
- The repaired consumer completed [cloud job `116746800272`][repaired-job],
  including scanner installation and the full forbidden-strings scan.

### Isolated error-selector prototype

The [prototype patch][prototype] contains the selector change,
a TypeScript generator,
and `mise` tasks.
It applies to the audited `v2.102.0` checkout.
The generator extracts the actual production selector before and after the patch,
removing only channel receive and cancellation statements so fixtures can supply both completion orders.
Generated Go programs retain the source's success and failure selection branches.

The test environment used Node `24.21.0`,
Go `1.27.2`,
and the existing `node:24-trixie-slim` container image.
Each container had no network,
a `2 GiB` memory limit,
and a `2 CPU` limit.
Only the Go installation and generated fixture directory were mounted read-only.
No repository credentials or this repository's working tree were mounted.
The tasks disable SELinux labeling for these read-only mounts;
without that setting the image entrypoint could not execute the mounted compiler.

The patch records the installed Go path under the current user's home directory.
The version selects the measured toolchain for this reproduction.
From a disposable clone at the audited commit,
apply the patch and run its tasks:

```sh
# Disposable cli/cli checkout at v2.102.0, after applying github-release-transfer-quota.patch.
mise run repro:generate
mise run repro:before
mise run repro:after
```

The pre-patch task failed four assertions:

```text
# repro:before, exit status 1 from mise
FAIL quota then missing: got release not found, want HTTP 403: API rate limit exceeded
FAIL missing then quota: got release not found, want HTTP 403: API rate limit exceeded
FAIL server then missing: got release not found, want HTTP 500: server error
FAIL missing then server: got release not found, want HTTP 500: server error
PASS both missing
PASS found then quota
PASS quota then found
PASS different failures preserve first
```

The patched task returned exit status `0`:

```text
# repro:after
PASS quota then missing
PASS missing then quota
PASS server then missing
PASS missing then server
PASS both missing
PASS found then quota
PASS quota then found
PASS different failures preserve first
```

The fix switches the sentinel test from `second.error` to `first.error`.
If the first error is only not-found,
the second error is retained;
otherwise the first error is retained.
Success precedence and the both-not-found case remain covered by controls.
This verifies selector behavior,
not a complete patched CLI build,
HTTP transport integration,
or the full upstream suite.

## Verified workarounds

### Remove Scorecard's per-run REST history requests

The repository-side query repair in commit `f598404b31`
replaces the gate's history requests with a nested GraphQL query.
The workflow resolves its URL and requests batches of run records at
`.github/workflows/scorecard.yml:58`:

```graphql
# .github/workflows/scorecard.yml:58, query excerpt
resource(url: $workflowUrl) {
  ... on Workflow {
    runs(first: 100, after: $cursor, orderBy: {field: CREATED_AT, direction: DESC}) {
```

The complete runnable patch is the committed
[Scorecard workflow](../../.github/workflows/scorecard.yml).
Its [production gate][scorecard-repaired-job] succeeded,
visited 972 historical runs with ten GraphQL requests and ten points,
and emitted `no REST history requests`.
It then skipped the dependent analysis at the eight-per-day count.
The counting step ran from `00:51:04` to `00:51:29` UTC on October 16,
2026,
according to the jobs API.
The [retained evidence][scorecard-production-evidence] records the complete run identity
and selected credential-free log lines.

Tradeoffs:
the query uses GraphQL quota and refuses admission when history is incomplete,
the page bound is reached,
or remaining points fall below its reserve.
It retains the old workflow-creation-time,
latest-attempt,
cancellation,
and absent-analysis accounting semantics.
The [scheduling evaluation](../planning/depot-rolling-budget-evaluation.md) records those boundaries
and the passing policy fixtures.
This consumer repair removes these REST history requests;
it does not prove which consumers exhausted the original installation quota,
eliminate every API call in the workflow,
or establish that GitHub's quota enforcement is defective.

### Download public releases without the exhausted installation token

The consumer repair in commit `e5af1daa71` uses the known public release asset URL directly.
It avoids authenticated REST metadata lookup for the expected release.
The cloud diagnostic verified anonymous asset retrieval,
and the repaired cloud job verified the actual consumer path.

Tradeoffs:
this path requires a known tag and asset naming convention and applies to public assets.
The consumer's exact-release `404` fallback still uses GraphQL release discovery.
Future versions still use current-repository attestation verification.
The repair therefore does not promise zero API requests for every future version or fallback.

### Recover and verify the original signed attestation

The original archive's SHA-256 is
`a6e82699d8fcdd2b82af811b680a473e5068aa1b01ce53eeb415c9f5ae015116`.
The successful recovery used this endpoint and bundle extraction:

```sh
# Bundle recovery into a disposable verification directory.
gh api users/Aquaticat/attestations/sha256:a6e82699d8fcdd2b82af811b680a473e5068aa1b01ce53eeb415c9f5ae015116 \
  --jq '.attestations[].bundle' > forbidden-strings-0.5.0.sigstore.jsonl
gh attestation verify forbidden-strings-0.5.0-x86_64-unknown-linux-gnu.tar.gz \
  --bundle forbidden-strings-0.5.0.sigstore.jsonl \
  --repo Aquaticat/Monochromatic \
  --signer-workflow Aquaticat/Monochromatic/.github/workflows/cargo-publish.yml \
  --format json
```

The [CLI verification manual][verify-manual] documents bundle input and signer policy.
The consumer now carries the original bundle at
`package/forbidden-strings/cli/provenance/0.5.0-x86_64-unknown-linux-gnu.sigstore.jsonl`.
The local `verify:provenance` task in the session's `binstall-org-repair` scratch directory
recorded the positive and negative results in `provenance-controls.json`.

Tradeoffs:
the exception is tied to this archive version,
target,
digest,
and historical signer.
It does not authorize arbitrary old-owner artifacts or replace signature verification with endpoint success.
The negative controls demonstrate that changed bytes and unrelated signer identity remain rejected.

## What does not work

- Interpreting `release not found` as proof that the release or tag was deleted.
  Anonymous tag lookup and asset retrieval succeeded during the same cloud probe.
- Renaming the attestation repository argument to the current repository name alone.
  The tested repository endpoints returned `404`,
  while the original user's endpoint returned the bundles.
- Concluding that repository transfer destroyed attestations.
  The recovered original bundle verified against the actual archive.
- Treating upstream [PR `#13723`][success-fix] as an absent fix for this incident.
  Its success-first selector is already in `v2.102.0`.
  It addresses one lookup succeeding while the other fails;
  this prototype addresses two failed lookups with different information.
- Inferring the server-side transfer cause from CLI route construction.
  The source establishes the request paths,
  not GitHub's backend indexing behavior.

## Upstream filing artifact

### Upstream filing decision

The `.out-of-scope/` inventory was searched for GitHub CLI,
release,
and attestation exemptions;
none matched this incident.
Upstream searches covered open and closed issues and pull requests using
`"release not found"`,
`"release not found" "rate"`,
`release "403"`,
`"release" "rate limit"`,
and `attestation transfer`.
The related [PR `#13723`][success-fix],
including its comments and reviews,
was read in full.
Its discussion does not contain the both-failed quota-error reproduction.
No matching attestation-transfer issue or pull request was returned by those scoped searches.
That result does not establish absence of private GitHub support incidents.

1.  Upstream fault:
    yes for discarding an actionable error in the release selector.
    The attestation backend explanation is unproven and is not folded into this finding.
2.  Fixable:
    yes;
    the local selector change preserves the informative error in both completion orders.
3.  Supported use case:
    yes;
    `FetchRelease` explicitly supports published and draft lookup,
    and PR `#13723` documents public release downloads and CI use.
4.  Contribution welcome:
    yes for an issue or additive discussion comment.
    `.github/CONTRIBUTING.md:5` encourages those contributions,
    while restricting external pull requests to `help wanted` issues with acceptance criteria.
    The README,
    bug template,
    PR template,
    `AGENTS.md`,
    and related maintainer discussion were checked.
    No ban on AI-assisted reports was found.
    The PR template explicitly includes agent-authored contributions;
    its human follow-up question concerns PR creation,
    which is not being performed here.
5.  Likely fix:
    soft yes;
    maintainers accepted the related selector repair in PR `#13723`.
    No documented refusal of this failure-selection behavior was found.
6.  Prototype:
    yes for the isolated selector.
    The patch and eight-case reproduction demonstrate the defect and correction;
    full CLI integration remains outside the measured prototype.

The artifact is an additive comment for the existing related PR,
not a proposed unsolicited pull request.
Nothing was posted externally.
The attestation endpoint discrepancy has no separate fileable source-level defect claim in this document.

The relevant contribution policy at `.github/CONTRIBUTING.md:5` says:

```text
# .github/CONTRIBUTING.md:5, cli/cli v2.102.0
We encourage issues and discussion posts for all other contributions.
```

### Additive comment draft for PR `#13723`

~~~md
<!-- Additive comment draft for cli/cli#13723. -->
There is a remaining error-selection case in `gh` 2.102.0:
both lookups fail, one with an HTTP quota error and the other with `ErrReleaseNotFound`.

At `pkg/cmd/release/shared/fetch.go:259`, a GraphQL result for a published release
returns `ErrReleaseNotFound` from the draft lookup:

```go
if query.Repository.Release == nil || !query.Repository.Release.IsDraft {
    return nil, ErrReleaseNotFound
}
```

At `pkg/cmd/release/shared/fetch.go:221`, the failure selector prefers that sentinel:

```go
if errors.Is(second.error, ErrReleaseNotFound) {
    return nil, second.error
}
return nil, first.error
```

A source-extracted selector harness on `v2.102.0`,
commit `fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`,
returns `release not found` for HTTP 403 plus not-found in either completion order.
HTTP 500 plus not-found has the same result.
Changing the tested operand to `first.error` preserves the actionable error.
Both-not-found, either lookup succeeding, and two different non-not-found errors remain covered.

Reproduction and local prototype:
apply the accompanying `github-release-transfer-quota.patch` to that tag,
then run `mise run repro:generate`, `mise run repro:before`, and `mise run repro:after`.
The original selector fails four of eight assertions; the patched selector passes all eight.
The prototype extracts selection logic and does not claim a full patched CLI integration test.

An actual CI probe also observed authenticated REST release listing returning 403,
anonymous release and asset lookup returning 200,
and `gh release view` returning `release not found`.
It did not capture both internal responses for that specific invocation,
so the isolated reproduction is the direct evidence for this selector defect.

This report and prototype were prepared with AI assistance.
The source extraction, pre/post assertions, cloud results, and consumer workaround were checked by the agent;
no independent human verification is claimed.
~~~

[diagnostic-job]: https://github.com/Aquaticats/Monochromatic/actions/runs/38829393247/job/116744386136
[repaired-job]: https://github.com/Aquaticats/Monochromatic/actions/runs/38830142505/job/116746800272
[prototype]: github-release-transfer-quota.patch
[success-fix]: https://github.com/cli/cli/pull/13723
[user-attestations]: https://docs.github.com/en/rest/users/attestations
[verify-manual]: https://cli.github.com/manual/gh_attestation_verify
[scorecard-repaired-job]: https://github.com/Aquaticats/Monochromatic/actions/runs/38831778986/job/116751956364
[scorecard-production-evidence]: ../audit/depot-scorecard-graphql-production-2026-10-15.evidence.json
