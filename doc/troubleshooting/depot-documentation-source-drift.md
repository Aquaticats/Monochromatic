# Depot docs e40bfe1: source mirror yields obsolete capability exclusions

## Symptom

On 2026-10-18,
reading the public `depot/docs` main checkout alone
incorrectly excluded deployed Depot CI and Sandbox capabilities.
The checkout was `e40bfe1683ae561156d415513f2a648f89e82f12`,
whose latest commit was dated 2026-09-11.
The [deployed documentation][docs] gave different answers when fetched directly over HTTP.

The stale exclusions concerned cross-repository reusable workflows,
Sandbox public-beta availability and custom runtimes,
and default Depot Code standalone CI triggers.
This was a research evidence-selection failure,
not a demonstrated failure to execute those features.

## Root cause and evidence

The mistaken chain was:
shallow-clone the public docs repository,
read its current branch as the complete deployed contract,
and treat historical restrictions as present exclusions.
Direct deployed-page HTTP 200 responses and rendered text disproved that chain.

At the cited checkout,
`content/ci/compatibility.mdx:197` says:

```md
<!-- depot/docs/content/ci/compatibility.mdx:197 at e40bfe1683ae561156d415513f2a648f89e82f12 -->
The `uses` key for referencing workflows from other repositories is not supported.
```

The [deployed compatibility page][compatibility] instead says:
"You can call a reusable workflow from another repository using a job-level `uses` reference".
It documents full commit SHAs/version tags and private-source access restrictions.

The source checkout's `content/api/overview.mdx:31` says:

```md
<!-- depot/docs/content/api/overview.mdx:31 at e40bfe1683ae561156d415513f2a648f89e82f12 -->
The Sandbox SDK is in private beta.
```

The [deployed API page][api] calls it public beta.
The [deployed SDK reference][sandbox] also documents an `imageRef` runtime,
while `content/api/sandbox-sdk-reference.mdx:97` in the checkout says custom runtimes are unavailable.

The source checkout's `content/code/overview.mdx:97` says:

```md
<!-- depot/docs/content/code/overview.mdx:97 at e40bfe1683ae561156d415513f2a648f89e82f12 -->
Depot CI triggers are currently disabled by default for standalone repos
```

The [deployed Code page][code] instead says standalone repositories and CI push triggers
are enabled by default for all organizations.

[Docs PR 187][sync-pr] is an open automated update from `depot/app`.
A [contributor comment on PR 188][source-authority] says:
"docs changes are authored in app and synced here."
Together these are evidence that the public repository is a synchronization target.
The precise deployment and synchronization schedule was not established.

The earlier private-beta,
opt-in,
and unsupported-reusable-workflow conclusions were retracted.
This does not imply every deployed capability is available on Developer:
the live runner plan matrix still puts macOS on Startup and Business.

## Verification

The [research evidence][evidence] retains source revisions,
direct-page HTTP statuses and SHA-256 digests,
and exact text matches for the conflicting claims.
Pages were also converted with Pandoc and their actual rendered text inspected.

The verification task compared pinned public source text against live rendered documentation.
Its complete runnable Mise definition,
`verify-deployed-versus-mirrored-docs`,
is retained in the evidence's `harnesses` field.
Its working controls were the current public-beta statement,
cross-repository workflow support,
and enabled standalone triggers.
The failing readings were the corresponding exclusions in the pinned source snapshot.
This checks the reading boundary;
it does not itself execute reusable workflows or create a Code repository.
The [separate SDK probes][sdk-evidence] exercised default amd64 and Arm64 sandboxes.

## Verified workaround

Fetch the deployed document that owns the capability,
then exercise a bounded consumer when that fact determines the design.
Retain the fetched date,
body digest,
source revision,
and actual outcome separately.
The generic SDK's [registry restriction][runtime-boundary] demonstrates why
a current page or an installed type comment alone still cannot replace the real consumer probe.

The tradeoff is that live documentation can change after collection.
Pinned source remains useful historical evidence and a path into implementation details;
it cannot be assumed to represent a different deployed revision.

## What does not work

- Treating a repository's `main` name or "Edit on GitHub" link as a freshness guarantee.
- Treating a public-beta label as proof of paid-plan eligibility or complete API behavior.
- Extending one corrected restriction to unrelated features without fetching their deciding sources.
- Treating absence of a maintenance guarantee as a prohibition on requested investigation or work.

## Upstream filing decision

The `.out-of-scope/` files were searched for Depot and documentation mirrors;
no matching exemption was found.
Open and closed issue/PR listings were read,
including the automated synchronization PR and the redirected snapshot documentation PR.
No message,
issue,
or comment was sent upstream.

1.  Upstream fault:
    no deployed product failure was established.
    The corrected failure was our choice to infer current capabilities from a historical mirror.
2.  Fixability:
    synchronizing documentation is possible in principle,
    but the private `depot/app` publishing implementation was not inspected.
3.  Supported use case:
    the repository README describes documentation hosted at `depot.dev/docs`.
4.  Contribution path:
    the contributor comment identifies `depot/app` as the authoring location;
    editing this public mirror would not establish a durable publishing fix.
    No prohibition on AI-assisted research is inferred.
5.  Upstream movement:
    PR 187 already contains a synchronization proposal,
    and PR 188 was redirected to the authoring repository.
6.  Prototype:
    no product fix was prototyped because no product defect was demonstrated.
    The consumer-side evidence comparison was implemented and run instead.

Nothing fileable is proposed.
The existing source-authority comment already supplies the relevant routing information;
repeating it upstream would add no missing diagnosis.

[docs]: https://depot.dev/docs
[compatibility]: https://depot.dev/docs/ci/compatibility
[api]: https://depot.dev/docs/api/overview
[sandbox]: https://depot.dev/docs/api/sandbox-sdk-reference
[code]: https://depot.dev/docs/code/overview
[sync-pr]: https://github.com/depot/docs/pull/187
[source-authority]: https://github.com/depot/docs/pull/188#issuecomment-5797275767
[evidence]: ../audit/depot-developer-platform-research-2026-10-18.evidence.json
[sdk-evidence]: ../audit/depot-developer-sandbox-sdk-2026-10-18.evidence.json
[runtime-boundary]: depot-sandbox-runtime-registry-boundary.md
