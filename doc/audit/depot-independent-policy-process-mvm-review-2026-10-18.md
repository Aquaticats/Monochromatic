# Independent Depot policy, process and MVM evidence review

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/mvm` is now `package/mvm/cli`.

## Review boundary

This review independently decodes the original archive bundles for
[`job-policy-history`](depot-job-policy-history-2026-10-18.evidence.json),
[`process-timeouts`](depot-process-timeouts-2026-10-18.evidence.json)
and [`mvm-kvm`](depot-mvm-kvm-2026-10-18.evidence.json).
It also checks the combined-product pricing arithmetic and the coordinator's
[`expanded CI report`](../planning/depot-expanded-ci-evaluation.md).
The review excludes this reviewer's own Rust execution claims.
No new cloud allocation is admitted.

All 19 encoded original ZIPs decoded with strict base64 validation,
matched their recorded byte lengths and SHA-256,
and passed Python's independent ZIP CRC checks.
The ZIPs contain 566 file members totaling 802,946 uncompressed bytes;
the original ZIP bytes total 263,197.
Every member was decoded and individually hashed.
All member bodies were UTF-8 text;
there were no media or undecodable binary members.
The private independent catalog and input snapshots are retained under
`temp/agent/depot-independent-review-20261018` in the executing user's home.

## Findings

### Corrected: MVM file roundtrip does not independently exercise guest payload access

The initially reviewed coordinator report's MVM paragraph calls the transfer a 51-byte file roundtrip.
The stronger description of this as a guest payload roundtrip is not established.
All passing lifecycle receipts declare `consumer.meta.fileTransfer = "virtiofs"`.

At application source `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`,
`package/cli/mvm/src/file-transfer.ts:282` uses a host copy:

```ts
// package/cli/mvm/src/file-transfer.ts:282, abridged.
await copyFile(hostPath, sharedHostPath);
```

The matching shared-directory pull at `file-transfer.ts:409` reads that host file:

```ts
// package/cli/mvm/src/file-transfer.ts:409, abridged.
const content = await readFile(sharedHostPath);
return content;
```

Both paths select `basename(guestPath)`.
`package/cli/mvm/README.md:170` explicitly documents this shared-directory route
and distinguishes it from the guest-agent route.
This is not a newly inferred product defect.

The actual standard-GitHub consumer at harness
`64d6a666e63e4773ee56666148ae9c8ba371877a`,
`.depot/research/mvm/mise.toml:251` to `259`,
creates a guest directory,
then calls push and pull without a guest payload read or write:

```js
// .depot/research/mvm/mise.toml:256, abridged.
fs.writeFileSync(hostInput, payload);
requireSuccess(mvm('push', ['push', manifest.guest.name, hostInput, guestPath]));
requireSuccess(mvm('pull', ['pull', manifest.guest.name, guestPath, hostOutput]));
if (!payload.equals(fs.readFileSync(hostOutput))) {
  throw new LifecycleProbeFailure('Real guest file roundtrip bytes differ');
}
```

The retained commands and stdout agree with this source:
push reports `/mnt/shared/owned-<nonce>.bin`,
and pull reads the host `shared/` path.
The requested `/root/temp/agent/` directory does not strengthen the transfer assertion,
because the shared route uses only its basename.

Correct the report and transfer summary to:
“51-byte shared-directory CLI roundtrip;
guest payload visibility/write was not separately asserted.”
Keep the actual guest stdout,
exit status `2`,
QMP/KVM and private-domain/daemon cleanup credit.
A future separately admitted consumer can have the guest read and transform `/mnt/shared/` bytes
before pulling the changed result.
That consumer was not executed by this review.

The coordinator accepted this finding.
The current report explicitly calls the original result a shared-directory CLI roundtrip
and retracts separate guest payload visibility/write credit.
The original archive bytes remain authoritative;
the correction changes their qualification rather than their contents.

## Original archive and observation joins

The independent decoder used strict base64 decoding,
SHA-256 and byte-length comparison,
`zipfile.ZipFile.testzip()` and an explicit read of every non-directory member.
All 19 ZIP metadata joins and all 53 structured observation/member-hash joins pass.
The observations include the policy controls,
the process-timeout controls and every retained MVM diagnostic observation,
including prerequisite failures.

The original compressed bundles have these SHA-256 values:

- `depot-job-policy-history-2026-10-18.archives.json.gz`:
  `c96e692b6dacebaffac76c045c46286fbd49b92262b91b7395dd0bec1597ac6c`.
- `depot-process-timeouts-2026-10-18.archives.json.gz`:
  `ae7580dcf37185b3422befd024f4f1fd7fa342e79f3c4d92c3dffafceb899d4e`.
- `depot-mvm-kvm-2026-10-18.archives.json.gz`:
  `9ec38d2f8c3605e9400b0960a24d29196ad012c19f89fb9ce042c4b8e8d2a38f`.

Every decoded text member was included in credential-pattern inspection.
Manual inspection of every resulting candidate found QEMU help placeholders
(`password=password` and `password=<file>`)
and captured source code invoking `getent passwd`.
The broader context sweep also retained only system passwd identifiers,
key-file help paths and udev rule placeholders.
The dense QEMU logs name ephemeral master-key file paths;
they do not include those key files or their contents.
No credential material was found in this decoded-member review.
This is a bounded review of the retained members,
not a guarantee about unretained provider state.

## Job and step continuation

The raw receipts preserve a distinction between job continuation and step continuation:

- Native `kjf78stmkb`,
  source `f446852c42d4a382ba7931e6730a17cb3d8308e0`,
  has aggregate status `failed`.
  Its `allowed_failure` job is `failed`,
  while `consumer` and `history` are `finished`.
  The consumer's actual `needs.allowed_failure.result` is `success`.
- The paired GitHub run `39122132487` has workflow conclusion `success`,
  `allowed_failure` job conclusion `failure`,
  and consumer job conclusion `success`.
  This does not mean the failed GitHub job itself became successful.
- Native default-needs control `pfzfjrqdpq`,
  source `714a3cad36c20da75969454fe33f28dab7770e9b`,
  again has aggregate `failed`,
  failed allowed-failure job and finished consumer.
  The exact `.depot/workflows/ordinary-job-default-consumer.yml` source
  has `continue-on-error: true` on the failing job,
  an ordinary dependent consumer and no consumer `if` override.
- Native isolated step control `lrhfzwrwl2`,
  at the same `714a3cad36` source,
  has finished job and workflow.
  Its raw step observation reports outcome `failure` and conclusion `success`.
  `.depot/workflows/advisory-step-result.yml` puts continuation on the step.
  This verifies that bounded step workaround;
  it does not separately prove every GitHub App aggregate or required-check configuration.

The read-only history probe performed no uploads and executed no application tests.
Its original 34-candidate queries had no retained per-test timings.
The actual-class positive control found a timing;
the absent-class control did not.
These are API identity/timing controls,
not additional application case credit.

## Process deadline and advancement

All observations use harness `44b3303eca52a58806e28a6e8b681be07d43c278`.
The source task waits 70000 ms without a signal handler,
and the CI step limit is 60000 ms.
Every retained writer completed after 70009 ms.
Every timeout step has outcome `failure` and conclusion `success`.
These measurements establish deadline overrun on all three providers.

The boundary timestamps establish different advancement behavior:

- Standard GitHub writer completion is `2026-10-18T07:17:26.662Z`;
  observer-step start is `2026-10-18T07:17:26.6831188Z`.
  Completion precedes observer start.
- Depot-managed GitHub writer completion is `2026-10-18T07:17:24.042Z`;
  observer-step start is `2026-10-18T07:17:24.0592659Z`.
  Completion precedes observer start.
- Native writer completion is `2026-10-18T07:24:57.299Z`;
  observer-step start is `2026-10-18T07:24:46.969Z`.
  Completion follows advancement by 10.330 seconds.

The native same-VM observation also corroborates advancement:
the observer waits twelve seconds and records elapsed time 72027 ms from writer start,
while writer completion is at 70009 ms.
The observer necessarily began its wait before that completion.
The narrowed conclusion in
[`depot-step-timeout-process-boundary.md`](../troubleshooting/depot-step-timeout-process-boundary.md)
is supported.
These observations do not establish every descendant's kill path,
physical VM release,
billable lifetime or indefinite orphaning.

## Original MVM lifecycle checks

The passing native,
Depot-managed GitHub and standard-GitHub observations retain distinct harness sources
`6b3c75517e798161e79ddb7e496b78cf1ffc4240`,
`1438ecde4b643a606e0a75f85067e3d0153e9bb5`
and `64d6a666e63e4773ee56666148ae9c8ba371877a`.
They bind the frozen application source separately from those harness revisions.

For every passing consumer,
this review reconstructs the exact 51-byte payload from its guest-emitted nonce
using the harness's `depot-current-mvm-<nonce>\n` encoding.
Its independently computed SHA-256 matches the retained payload digest.
The raw receipts also retain actual guest stdout,
the deliberate missing-path stderr and exit status `2`,
QMP's enabled KVM result,
empty remaining private-domain inventory,
both owned daemons exited and no recorded cleanup-operation errors.
The daemon logs retain startup/guest-agent diagnostics;
empty cleanup-operation errors do not assert globally empty stderr.
The standard-GitHub replay's disposable child changes device-group membership
while preserving UID/GID.
None of these facts repairs the separate payload-access gap in the original consumer.

## Independent pricing recomputation

The reviewed
[`combined model`](depot-combined-product-pricing-2026-10-18.evidence.json)
has SHA-256 `933fa5192300d380bd01ba456709de4406ce2af7733579a937f4ff6e59d29b1d`.
Fresh unauthenticated requests at `2026-10-18T13:03:23Z` to `13:03:24Z` fetched
[pricing](https://depot.dev/pricing),
[native CI overview](https://depot.dev/docs/ci/overview.md)
and [GitHub runner types](https://depot.dev/docs/github-actions/runner-types.md).
Both documentation bodies exactly match the retained allowance-source snapshots.
Fresh pricing HTML differs in bytes,
but its normalized visible text exactly matches the retained pricing HTML.
The independent visible-text SHA-256 is
`f7fdd24fa61ba9b865bbe546e4f79614cab1171798562e2c5d442bfee8738284`.

The independent calculation uses decimal arithmetic and separately derives these inputs:

- Developer fee USD 20,
  500 build base minutes,
  2000 native two-CPU base minutes,
  2000 GitHub Linux two-CPU base minutes,
  25 GB combined average storage and 1000000 passing results.
- Startup fee USD 200,
  5000 build base minutes,
  20000 native two-CPU base minutes,
  20000 GitHub Linux two-CPU base minutes,
  250 GB combined average storage and 1000000 passing results.
- Overage rates:
  USD 0.04 per default 16-CPU/32-GB build base minute,
  USD 0.006 per native two-CPU base minute,
  USD 0.006 per GitHub Linux two-CPU base minute,
  USD 0.20 per combined GB-month and USD 0.000001 per additional passing result.

All line items and all 12 plan/scenario totals match.
The six scenarios produce Developer/Startup totals respectively:

- No overage:
  USD 20 / USD 200.
- Native allowance boundary:
  USD 128 / USD 200.
- Both separate Linux compute boundaries:
  USD 236 / USD 200.
- Build boundary:
  USD 200 / USD 200.
- Combined product boundary:
  USD 461 / USD 200.
- Explicit mixed illustration:
  USD 142 / USD 201.

These inputs are hypothetical monthly scenarios,
not a repository workload forecast,
invoice or spending cap.
Native and GitHub compute entitlements remain separate;
storage and passing results remain combined.
The model grants no unresolved generic-SDK or Mac allowance normalization credits.

## Report review and queued verification

The initially reviewed expanded report has SHA-256
`32ff533fe204730eac804b6f59b55dc385f52eff655a9901b38c2eb8f13d2430`.
Outside this reviewer's Rust claims,
its distinctions between current and historical source cohorts,
build production and real consumers,
APK production and device instrumentation,
passing browser cases and retained WebKit failure/skip,
API identity probes and application tests,
and hypothetical pricing versus a complete workload forecast are explicit.
No other material overstatement was established in this document-level review.
This is not a second independent execution of every linked application suite.

The MVM qualification finding was accepted and its current report wording rechecked.
The coordinator subsequently executed a finite three-pool consumer that reads pushed bytes in the guest,
creates a distinct initially absent output and pulls the guest-transformed result.
Independent decoding,
CRC/hash/outcome joins and credential inspection of those three new ZIPs were queued at this checkpoint
and are now complete in the Actual guest-transformation review section.
They must remain a separate cohort from the original eight MVM ZIPs.

The original-cohort review checkpoint `7d743ae989` passed the targeted
`mise run lint:monochromatic-lint -- doc/audit/depot-independent-policy-process-mvm-review-2026-10-18.md` task.
Its complete `marked` renderer output was inspected.
The original compressed archive hashes were rechecked unchanged after the coordinator's correction.

## Additional service and PR review

The coordinator assigned and this reviewer completed checks of these additional published cohorts:

- `depot-service-containers-2026-10-18.evidence.json` and its ten original ZIPs:
  original bytes,
  CRC/member hashes,
  consumer/source identity,
  actual Node/Git versus base-image runtime and credential privacy.
- `depot-runtime-pr-2026-10-18.evidence.json` and its eleven original ZIPs:
  original bytes,
  CRC/member hashes,
  PR App/source/merge identities,
  disposable-resource cleanup and credential privacy.
- `depot-container-prerequisites-2026-10-18.evidence.json` and its source bundle:
  prerequisite evidence and the pinned checkout-source reasoning.
  No generic REST-stall cause is presumed.

This work uses published/local evidence and read-only source inspection.
It admits no cloud allocation or repository/billing settings change.
The separate three-ZIP guest-transformation review was queued at this checkpoint
and is now complete in the Actual guest-transformation review section.

### Original additional archives

The additional 21 ZIPs contain 21 JSON members totaling 62099 uncompressed bytes;
their original ZIP bytes total 25758.
Strict base64,
recorded byte count/SHA-256,
ZIP CRC and complete member reads all pass.
Every member's independent SHA-256 is retained in the private additional catalog.
All 21 original JSON observations match the published structured observations.
All 17 separately retained member hashes match;
the remaining native service records retain member names and complete parsed observations instead.
All twelve GitHub ZIP digests match their advertised API digests.
All nine native ZIP sizes match API metadata;
fresh read-only artifact listings independently checked the five native PR ZIPs
and the cancelled native PR run's empty artifact list.

The additional compressed bundles have SHA-256 values:

- `depot-service-containers-2026-10-18.archives.json.gz`:
  `1d548493228ec9950b9de34d9b70b1107734a72a600e7ee9a7ad3218fc1a6b20`.
- `depot-runtime-pr-2026-10-18.archives.json.gz`:
  `fa455b4dd47abbfa7dce33d590a6ee1b0edf9d9d3657d35a829c0c6a1a062c1c`.

Credential-pattern and broader keyword/context inspection covered all additional members
and the decoded checkout-source files.
The only source-file keyword contexts concern credential-manager configuration and source identifiers.
No credential material was found in these retained members.
The service receipt's exact owned-secret absence flags are separate metadata;
these ZIPs do not independently prove secret deletion or opaque-token revocation.
The original receipt correctly sets `tokenRevocationAsserted: false`.

### Service and runtime boundaries

Native run `g02lgcbfgq` has four finished host/job-container consumers:
x64 and Arm64 in each mode.
GitHub run `39113547458` has six successful consumers:
standard GitHub x64 and Depot-managed x64/Arm64 in each mode.
All actual consumers identify harness `5485cc0d12805f330d568b358e5ff0bb988d4d37`.
The fixture server separately identifies producer `5cfc1d90f92154cf62fa95789fad9bd7deeda208`
and the frozen application marker `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
These are functional fixture controls,
not repository application-suite acceptance.

Each consumer records `/health` and `/nonce` responses with status `200`
and a nonce-qualified missing-path response with status `404`.
The review independently reconstructs and hashes all thirty response bodies
and checks their source,
nonce,
path and architecture fields.
Every server response records Node `v24.22.0`.

The five job-container consumers separately retain actual Node `v26.12.0`,
the hash of their running executable,
matching architecture-specific tool-manifest binary hash,
loaded `libatomic.so` mappings and actual `git version 2.39.5`.
Their job image is digest
`sha256:853b99050d5420698a02d7cb10978be8824dc0999c9ace08ec453706da54ec31`,
with image-tool producer `7590bcde15e5ceef4dc36f4e2c3d975e1c1b833f`.
All fifteen embedded input/builder/tool proof hashes reproduce from their exact source serialization.
The host HTTP observations do not independently hash their host Node executable.
The reported Node 24 fixture health therefore does not substitute for the separate Node 26 container proof.
Git 2.39.5 qualifies this exercised checkout boundary;
it does not satisfy the Git-policy suite's exact Git 2.56 fixture.

### Disposable PR App and source boundaries

Fresh read-only GitHub GETs at `2026-10-18T13:24:56Z` to `13:24:57Z`
independently bind both required check names to their exact Apps on all six recorded PR heads:
GitHub Actions App `15368` and Depot App `219785`.
Every check's head,
conclusion and actual provider workflow/job URL match the published run metadata.
The native run ID and its nested workflow ID are distinct;
the App URL matches the nested workflow/job rather than relying on similar run names.

All eleven archived gate observations reproduce the control JSON at their exact head commits.
Fresh commit-object GETs at `2026-10-18T13:27:14Z` to `13:27:15Z`
also verify all six distinct checkout merge commits have the exact recorded base and head parents.
Thus the synthetic merge checkout,
PR head and frozen application marker remain separate identities.
The gate fixture executes no application test suite.

The failed head `0f45cf8730753a3e986d567c27314cb2632ad668`
has both required checks failed,
administrator enforcement enabled,
merge state `BLOCKED` and an actual merge rejection with HTTP `405`.
The passing replacement `c61f59f6869d5f6c733d4024f32d8fca1e25cdde`
has both exact App checks successful and records a successful squash merge
`5c5fcad4acad9751c4e0cd06f51f847a75537461`.
Fresh PR lookup confirms PR `1766` is closed and merged at that exact merge SHA.
The disposable base lookup returns `Branch not found`,
HTTP `404`,
and a fresh main-protection GET exactly equals the retained baseline.
This review changes none of those settings.

The successful supersession proof is the separately admitted 180000-ms holder at head
`b043aa1d88d9f49fd8391ea05db7126826edce41`,
followed by the successfully pushed replacement.
Both providers report its gate cancelled;
the GitHub ZIP retains phase `started`,
while native retains its source-bound start observation from logs and has no artifact.
The replacement's own checks succeed.
The initial 90000-ms attempt remains inconclusive after a rejected replacement push.
Provider cancellation proves neither descendant termination nor VM/billing cessation.
The historical `protected` flag's cause remains unresolved;
it is not used instead of the fresh branch-absence/protection observations.

### Container prerequisite source reasoning

The reviewed prerequisite source bundle has SHA-256
`f713b99f1f6842cdf206ca2f447af351d6d05ff2ba638633c7a90f7ebadf8a76`.
All three decoded files reproduce their declared byte count,
SHA-256 and Git blob hash.
Fresh official repository tree GETs connect those exact blob IDs to
`actions/checkout` commit `de0fac2e4500dabe0009e67214ff5f5447ce83dd`.
Read-only inspection of that exact commit in the existing upstream clone agrees.

At `src/git-source-provider.ts:77`,
the `!git` branch prints the REST fallback diagnostic.
At `src/git-source-provider.ts:356`,
`getGitCommandManager` catches initialization failures and returns `undefined`
unless LFS requires the error to propagate.
Initialization at `src/git-command-manager.ts:670` locates Git,
executes/parses its version and checks the minimum at line `688`.
The minimum is 2.18.
The diagnostic establishes fallback selection,
not which initialization error occurred or what stalled the REST download.
The original-image Git-absence control is explicitly limited to AMD64's default PATH;
it does not directly observe the historical job environment or Arm64 absence.

The current official
[context availability table](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts)
was fetched at `2026-10-18T13:31:46Z`.
It excludes `job` from `jobs.<job_id>.env`
and includes it in `jobs.<job_id>.steps.env`.
This supports moving `job.services` to the consuming step.
The zero-job failed workflow record still does not recover its original hosted parser diagnostic.
The independent Node 26 loader failure explicitly names missing `libatomic.so.1`;
it is a separate incident from REST fallback selection and download lifetime.
The quoted source trace,
exact context-placement patch and qualified workaround catalog are retained in
[`depot-service-job-container-prerequisites.md`](../troubleshooting/depot-service-job-container-prerequisites.md).

No additional material correction was established in these cohorts.
Their original compressed bundles remain unchanged.
The independent private proof catalog,
fresh GET bodies and recomputation results are retained under
`temp/agent/depot-independent-review-20261018/additional` in the executing user's home.

### Additional review publication checks

The changed review section and complete prerequisite troubleshooting note pass a rendered CommonMark check.
Targeted `lint:monochromatic-lint` passes for both Markdown files.
The documented workflow patch also passes `git apply --check` and actual application in a disposable local fixture;
its resulting workflow bytes exactly equal the target immutable revision.
Patch SHA-256 is `ee242687e4ba3c5d96edb63f69960570be6da47c498b4f8d549d85e5e6580fff`.
The source workflow SHA-256 is `374acbf649e114477522738da6bbf9c2335841458d2838ff7333ba7cf5b9015f`;
the applied result is `4183e80c7f9bdbfd6bfd7dc8e0b7df068bf4fca5ad48a184843a5112ad60eb6a`.
Only the disposable fixture was patched.

At this publication checkpoint,
the separate three-ZIP guest-transformation review was queued.
Its initial attribution named GitHub run `39132577785`
at harness `3829831039cc43aa717c6c927f835b867e9313b6`.
Attribution correction on 2026-10-18:
those identify the cancelled generator control.
The successful cohort is native `25jpltfpkq` and GitHub `39134227406`
at harness `1373e624739fbc93502b0c8ca502820c6ff99b45`,
qualified in the Actual guest-transformation review section.
The original eight-ZIP qualification and unchanged bytes remain authoritative for their original boundary.
The coordinator also received the exact frozen-source scanner/Truepeak profile handoff,
recorded in `doc/planning/depot-rust-coverage-research.md`,
for its independently owned repeated Linux DAG.
No cloud run or settings mutation was performed by this reviewer.

The Rust handoff also passes its rendered check and the same targeted lint invocation.
Read-only publication verification found remote `main` at
`48d85b438492c186baf81a4c2171d583f61f0b82`,
containing the completed review and handoff;
the owned paths were clean.
The two additional archive bundles and prerequisite source bundle still reproduce their reviewed SHA-256 values.

## Actual guest-transformation review

The new three-ZIP cohort is published at `c65e3b8eeb0f1bc0e06bf555048b85635ec59202`
in `depot-mvm-guest-files-2026-10-18.evidence.json` and its linked original bundle.
The independent review passed for its original ZIP bytes,
strict base64 decoding,
CRC,
member sizes and SHA256 hashes.
The cohort contains 81,121 ZIP bytes,
198 members and 216,993 uncompressed member bytes.
Both GitHub API artifact digests match the original ZIP hashes;
the native advertised size matches its original ZIP.
The linked gzip SHA256 is `6f0bf7c1ea1891b24d8702b373fc9c30b8eb885a0827e6e056cb483cac0183d9`.

Native run `25jpltfpkq` and both jobs in GitHub run `39134227406`
join immutable harness `1373e624739fbc93502b0c8ca502820c6ff99b45`.
The harness has no diff from frozen `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`
in `package`,
`mise.toml`,
`mise.lock`,
`pnpm-lock.yaml` or `pnpm-workspace.yaml`.
Its owning lifecycle source was read independently,
including the actual payload assertions and private daemon cleanup.

All 85 command records match their embedded lifecycle records and retained stdout/stderr members.
For each pool,
the independently reconstructed nonce-bearing input is 51 bytes and its `guest:` transformation is 57 bytes.
Both reconstructed hashes match the actual guest `sha256sum` output and retained consumer hashes.
The recorded guest boot ID differs from the host boot ID.
The actual `mvm exec` sequence checks the distinct output is absent,
copies input to it,
changes it with `sed`,
then hashes it before `mvm pull`.
The immutable harness compares pulled bytes directly with the expected transformed buffer before reporting success.
This qualifies actual guest input visibility,
output creation and transformation through `/mnt/shared` for this cohort.
It does not upgrade the original eight-ZIP host shared-directory check.

Each guest reports KVM enabled and running;
QEMU command lines retain four vCPUs and `size=8388608k` memory.
The deliberate missing-path command returns status 2,
while every other recorded command succeeds.
Each final private domain listing is empty,
both owned daemon PIDs are observed exited and cleanup errors are empty.
These are execution and cleanup observations,
not VM billing-release measurements or application unit-test cases.

The separate cancelled GitHub run `39132577785` has corrupt `$ + value +` runner labels,
no assigned runner and no actual step execution.
Its queued `started_at` timestamp does not establish allocated compute.
The source records the failed generator separately from the successful guest consumers
and retains guard removal/restoration.

All decoded members were inspected as UTF8,
including dense lifecycle/command records and daemon logs.
Credential patterns found no token,
authorization value or private-key material.
Sensitive-word contexts contain passwd/GID diagnostics,
public repository package names,
QEMU references to private key files and libvirt help paths;
they contain no key-file bytes.
This closes the pending three-ZIP review without changing original archive bytes or allocating resources.

The completed guest-review section and reconciled Rust handoff pass targeted `lint:monochromatic-lint`.
Their rendered CommonMark output was inspected.
The new original gzip retains SHA256
`6f0bf7c1ea1891b24d8702b373fc9c30b8eb885a0827e6e056cb483cac0183d9`.

## Expanded graph local preflight queue

The coordinator requested independent review of frozen driver and plan
at `815591bf307b1237d31b1c4e644463d7c13654fa`
after all jobs of `39135886118` failed at the omitted configured `cargo-binstall` prerequisite.
The local-only queue is source review,
a fresh selected-tool installation probe if practical,
and actual driver execution against explicitly fake owning commands in a disposable 2 CPU,
2 GiB container.
No application test credit or new cloud allocation belongs to this preflight.
The coordinator owns workflow corrections and dispatch;
its graph worktree will not be edited.

The expanded graph local queue is complete.
The [source-guard/cancellation troubleshooting record](../troubleshooting/depot-expanded-graph-source-guard-cancellation.md)
and [independent receipt](depot-expanded-graph-preflight-2026-10-18.evidence.json)
retain original defects and corrected `234ccc92` controls.
The browser-image omission claim was retracted after complete plan enumeration.
Fresh anonymous selected-tool installation passed;
fake graph execution earns no application test credit.
