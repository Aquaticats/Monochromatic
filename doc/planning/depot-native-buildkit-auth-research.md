# Native CI job authentication for container builds

Native run `pxx75mxnw2` passed the actual frozen-source queue producer and exported-file consumer
using only its automatically injected job credential.
The initial exporter fixture failure is retained separately.
Native run `38z4wlq9d3` also passed a separate image-only build,
project-registry token exchange,
complete OCI graph/blob validation,
and actual extracted-source execution using its injected job token.
All three native runs are terminal;
the solely owned project and saved image tags were removed with fresh absence controls.

## Scope and admission

This follow-up closes a cross-product boundary identified during the complete documentation audit.
The existing BuildKit campaign used the existing local user login.
Its successful builds and subsequent pull-token consumers do not establish that a native CI job can
build or read registry content using its automatically injected credentials.

Application inputs remain frozen at `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Experiment source belongs in a new isolated worktree.
Production workflows and the parent's repeated DAG inputs remain unchanged.

The parent admitted this finite follow-up on `2026-10-18`:

- One native `2 CPU`,
  `8 GiB` job with a five-minute server timeout.
- One default `16 CPU`,
  `32 GiB` AMD64 builder with a five-minute project build timeout.
- One newly owned temporary project,
  with autoscaling disabled,
  `keepGb: 25` and `keepDays: 1`,
  subject to the existing verified schema minimum.
- An additional `USD 1` campaign guard.
  Reserving the complete admitted windows at the public rates gives
  `5 × USD 0.006 + 5 × USD 0.04 = USD 0.23` of timestamp-compute arithmetic.
  This is not an observed invoice or a guarantee covering uninspected account usage and storage.
- No persistent token or secret,
  organization setting,
  broad credential,
  paid model,
  production image,
  or account-wide listing.
- Source review,
  local positive and rejected-input controls,
  and a recorded admission precede allocation.
  A bounded observer must retain terminal status and owned-resource cleanup.

The parent confirmed its native slot is free and will hold the native DAG matrix until this job is terminal.
No cloud resource has been created for this follow-up at the time of this admission record.

## Deciding source

The complete deployed [native CI container-build integration][native-builds] says trusted native jobs
receive a short-lived `DEPOT_TOKEN` for builds and the organization's registry.
Fork pull requests do not receive it.
The name is reserved,
and the injected token replaces a secret with that name during preparation.
The guide explicitly says these jobs need no project OIDC trust relationship or stored Depot token.

The installed CLI source at `1f96d6d488f938de97ea640b115e9567ebfd6fbf` resolves the explicit token,
then `DEPOT_TOKEN`,
then stored login in `pkg/helpers/token.go`.
Its later fallback also considers OIDC,
`DEPOT_JIT_TOKEN`,
and `DEPOT_CACHE_TOKEN`.
The experiment must isolate the CLI home and configuration and remove those alternatives from its
child-process environment,
rather than treating token presence as proof of the credential actually used.
The source is retained in the [platform source archive][sources].

`pkg/buildx/commands/build.go` authenticates before beginning a build.
`pkg/registry/cli.go:WithDepotSave` preserves a local exporter and appends an image exporter for `--save`.
The initial admission incorrectly concluded this permitted both outputs from one request.
That conclusion is retracted:
`pkg/buildx/build/build.go:443` rejects multiple exporters in the installed release.
The existing [exporter troubleshooting record][exporters] already established this behavior.
Successful export,
build authorization,
and a subsequent registry request remain distinct assertions.

The [GitHub Actions integration][github-builds] requires a project trust relationship for its
recommended OIDC route.
That is a different credential contract from the native job injection.
This follow-up does not create a GitHub trust relationship or qualify the GitHub OIDC route.

## Planned consumer and evidence

### Preventing account listing on an authentication failure

The complete `pkg/helpers/build.go:BeginBuild` error path calls onboarding on `NotFound`.
`pkg/helpers/project.go:InitializeProject` retrieves the project list before checking terminal mode.
That catch also surrounds existing-build attachment,
so precreating a build would not itself prevent a later listing.

The current fixture therefore retains the normal CLI credential path and uses the source-supported
`DEPOT_API_URL` override from `pkg/api/rpc.go` with an owned loopback HTTP/2 relay.
Only `CreateBuild`,
`GetBuildKitConnection`,
`ReportBuildHealth`,
`ReportStatusStream`,
and `FinishBuild`
may reach `https://api.depot.dev`.
The relay admits one creation containing the exact owned project identifier and requires its authorization
header to match the injected job credential.
Project,
organization,
and list operations are rejected before forwarding.
Builder connections still use the ordinary returned BuildKit credentials.
The relay is a research boundary,
not an adopted production API proxy.

Local preflight at `2026-10-18T14:45:16.436Z` retained these controls:

- The actual installed CLI received a local `NotFound` response and reached its onboarding branch.
  The relay rejected `ProjectsService/ListProjects` with `403`;
  no request reached Depot.
- Wrong credential,
  wrong project,
  and duplicate creation inputs were rejected.
- Local HTTP/2 unary and streamed bodies passed through the actual relay unchanged,
  with the expected authorization header.
  These controls used a loopback fixture,
  not Depot infrastructure.
- A network-disabled,
  mount-free `2 CPU`,
  `2 GiB` container executed the copied first-party queue consumer
  under the exact retained Node `24.22.0` image.
  FIFO execution,
  clearing,
  rejection,
  and the source hash passed.
  The owned preflight image was removed and its absence verified.
- The actual Mise workload prefix rejected the local non-native profile before running any command or API,
  and its saved JSON parsed successfully.

The preflight source is `1f5f9f51da475543beca0fe6e80116e484422cfa`.
The initial local generator failed during Mise template rendering before creating a worktree or resource.
It now uses the recorded
[GitHub-expression delimiter workaround](../troubleshooting/mise-task-github-expression-templating.md).
Neither the local relay controls nor the local Node consumer establish native job-token authorization.

The narrowly scoped project creation returned `200` at `2026-10-18T14:47:45.122Z`:
owned project `ng17dl3xrn`,
`HARDWARE_16X32`,
five-minute build timeout,
`keepBytes: 26843545600`,
`keepGb: 25`,
and `keepDays: 1`.
No autoscaling threshold was set.
Creation allocated no builder.
Harness `21494877fd` binds this project identifier into the fixture;
native dispatch remains a separate operation.

### Native consumer

The producer will copy the exact frozen first-party
`package/upstream-fork/p-limit/src/task-queue.ts` into a bounded context.
A pinned Node base will execute a queue consumer and export its source binding and observed results.
The native job must read and independently validate the resulting files.
This is a narrow first-party consumer,
not the package's complete unit-test owner or a performance sample.

A missing-token control must fail before build admission with an isolated home and no alternate credential.
The positive build receives only the injected job token as its Depot credential.
Registry requests must use that same credential and target only exact admitted references.
Local user authentication cannot be used to turn a failed registry control into a passing job-token claim.
The parent controller may use the existing local login only to create and clean the solely owned project
and retrieve this campaign's own status and artifacts.

The receipt must retain the committed harness,
project creation bounds,
native job and attempt identities,
build identity,
exporter and image digests,
raw artifact checks,
negative-control output,
independent consumer results,
and deletion or absence responses.
Failures remain evidence and do not authorize an unrecorded replacement allocation.

## First native result

Committed harness `e0bd5030fb0efde9b4b3982583229e10b74525fc` was dispatched at
`2026-10-18T14:54:05.816Z` as native run `dzsf27t0xx`.
Fresh status confirmed `failed` at `2026-10-18T14:54:55.510Z`.
The attempt was `2r16tj84qh`,
job `l1qjrcg80f`,
and sandbox `j14m9v5vk3w06g81t85l`.
The terminal watcher did not invoke its watchdog.
The native slot was released to the next coordinated profile.

The original `profile.json` records actual `x64`,
two available CPUs,
`8336855040` memory bytes,
and an injected token with no alternative Depot credential.
The missing-token command exited one with
`missing API token, please run depot login`.
The positive ordinary CLI request then obtained `200` responses for one `CreateBuild`,
BuildKit connection lookup,
health/status reporting,
and `FinishBuild`.
Its actual relay admitted the original injected credential,
forwarded twenty permitted requests,
and forwarded no account or project listing.
The CLI reported an AMD64 machine launch and connection.
This establishes build admission and connection authorization,
not a successful exported build.

Our `--save` plus `--output type=local` fixture repeated the known incompatible exporter combination.
The CLI exited one after `7948` milliseconds with `multiple outputs currently unsupported`.
The Dockerfile consumer did not execute,
build metadata was not produced,
and registry reads were not reached.
This is a fixture defect and a failed preflight scope,
not evidence that native job credentials lack build authorization.

The original seven-member ZIP is `2758` bytes,
with SHA-256 `2455167ace5a106c67f332a4579208265a4b7622ac48f6257de9d3d3a35dba03`.
Its bytes match the artifact API size and Action log digest;
all member CRCs pass.
The artifact API did not return a vendor digest.
The [campaign receipt][receipt] preserves the raw source and controls.

A correction must use a single exporter,
retain this failure,
and separately admit another native request before allocation.
The proposed local-only consumer would reserve a further `USD 0.23`,
for `USD 0.46` cumulative full-window timestamp-compute arithmetic under the admitted `USD 1` guard.
Registry image-read authorization would remain a distinct boundary if no image were exported.
At that failed checkpoint the solely owned project remained pending cleanup or an admitted correction.
The qualified corrections and final cleanup are retained in this record.

## Qualified local-export correction

Harness `ddf656ea992952559a975e8693668b2005583499` removed `--save` and retained only the local exporter.
Native run `pxx75mxnw2` finished successfully,
with fresh status at `2026-10-18T15:13:40.272Z`.
Its attempt was `r0vm5jjk75`,
job `wzvqphjg1w`,
sandbox `kp101dvl5mw0wnl41726`,
and build `xwq746tv4z`.
The isolated missing-token control failed before admission;
the positive ordinary CLI path used the injected token,
created one build,
forwarded no account-listing request,
and completed `FinishBuild` with `200`.

The pinned Node `24.22.0` producer executed the frozen first-party queue under an actual AMD64 builder
with sixteen available CPUs and `32997224448` memory bytes.
The native two-CPU job independently read the exported result and source,
verified source SHA-256 `e791bea71a4c59e1d6fb17e7d2213a52f55937fe5999329864ddf76b8c0c59a5`,
and checked FIFO execution,
clearing/rejection,
and the empty queue.
The CLI command took `8301` milliseconds in this single sample.
This is not a whole-package unit task or a performance comparison.

The original ten-member ZIP is `6295` bytes,
with SHA-256 `6049cbcdb3fa451ab0e1f269b2e5e134a9f3da51296a452bb9be4dd29199b205`.
Its CRCs,
API length,
Action log digest,
and independently retrieved frozen source bytes match.
The artifact API supplies no digest.
Our first local collector expected a colon in the digest message;
the retained Action actually says `SHA256 digest of uploaded artifact zip is`.
Correcting that selector required no cloud rerun or artifact replacement.

## Separately admitted project-registry consumer

The parent authorized a further image-only request before its unchanged graph samples.
The same owned project will receive one `--save` build,
with no local exporter and no separately named organization repository.
The native job will use the organization registry hostname and the project-scoped repository,
then validate the complete returned OCI index,
child manifests,
configuration and layer descriptors,
decompressed runnable layer digest,
and the exact first-party files.
It must import and execute the extracted queue source.
This is an OCI file consumer,
not an ordinary container-runtime pull claim.

Preflight at `2026-10-18T15:27:34.195Z` established:

- The current source outside the research fixture remains identical to the frozen application.
- The generated program parses,
  and a malformed JavaScript control fails the checker.
- The organization-hosted registry responds with the exact standard Bearer challenge,
  realm `https://api.depot.dev/auth/registry/token`,
  service `l4zncjjbj5.registry.depot.dev`,
  and scope `repository:ng17dl3xrn:pull`.
- An invented credential is rejected with `401`.
- The existing local user credential obtains a repository-scoped pull token in process,
  then receives `404` for the known absent saved image.
  This validates the preflight transport and scope,
  not native job authorization or existing image content.
- The actual source bytes from the successful cloud export import and execute under the local consumer.

The native runtime receives only its new injected job credential.
Its anonymous and invented-token controls remain separate from the positive native token exchange and content reads.
Registry credentials stay in process and are excluded from receipts.
Redirects drop authorization when crossing origins,
and every response is bounded to the declared tiny fixture size.

The additional full-window reservation is `USD 0.23`,
bringing conservative cumulative reservation to `USD 0.69` under the same `USD 1` campaign guard.
The admitted shape remains one two-CPU/eight-GiB native job and one sixteen-CPU/thirty-two-GiB AMD64 builder,
each with a five-minute server guard.
No registry allocation had occurred at that admission checkpoint.

## Qualified injected-token registry consumer

Harness `f0865228da0123433face17054ca7f7f62538c9b` ran as `38z4wlq9d3`.
Its workflow was `ptf21f5lhg`,
job `833d5l1vdh`,
attempt `4fbdgxpspd`,
and sandbox `whvmz0ljglpkk547x1tx`.
Build `20l2bllb5n` finished successfully;
fresh native status confirmed completion at `2026-10-18T15:33:29.579Z`.

The anonymous manifest request and invented-token exchange each returned `401`.
The native injected token obtained a repository-scoped bearer in process,
then read the saved index,
both child manifests,
and every referenced configuration/layer blob with `200`.
Authorization was removed on cross-origin redirects;
no signed URL or credential value is published.

The saved index is
`sha256:6af2ea8c461c034d875f805cc973972b7e2977ca324690be65f9133db30f5469`.
Its runnable AMD64 manifest is
`sha256:003e7caae307e87f0e77b8ab77e9e08518aa35449fcd77e96e448f3758582c0d`.
The native consumer and independent local verifier checked every descriptor's byte count and SHA-256,
the decompressed runnable layer's configuration `diff_id`,
and the exact frozen first-party source bytes.
The native consumer imported and executed that extracted source,
checking FIFO behavior,
clear/rejection,
and the empty queue.
This qualifies the actual project-repository OCI/file consumer,
not an arbitrary named organization repository or an ordinary container-runtime pull.

The attestation child carries a SLSA provenance `v0.2` statement whose subjects bind the runnable manifest.
Its complete graph was read and rehashed.
This is generated provenance evidence,
not signature verification or a claim about a trusted signer.

The original artifact has seventeen members and is `15759` bytes,
with SHA-256 `1b8e0b6fae36cd890b4c592cf759c57381caae4b834dcd25f43ade48ed4e7417`.
ZIP CRCs,
API byte length,
and the upload Action's log digest match.
The API does not return an artifact digest.
The [campaign receipt][receipt] and its linked source archive retain all three original ZIPs,
the complete OCI graph,
exact source,
raw logs,
and independently fetched workflow metadata.

## Exact owned cleanup and allocation limits

Project-scoped build metadata retained these request intervals:

- Failed exporter fixture `j9r23brwtx`:
  `14:54:39.892` to `14:54:47.759` UTC.
- Qualified local export `xwq746tv4z`:
  `15:13:27.293` to `15:13:35.527` UTC.
- Qualified registry image `20l2bllb5n`:
  `15:33:16.579` to `15:33:24.324` UTC.

These are request timestamps,
not billed builder allocation intervals.
The core `GetBuild` endpoint rejected the existing user token with `401`,
explicitly requiring an organization token.
No broader token was created.
The scoped CLI build-list response retained the known IDs and its nonempty pagination cursor;
it is not claimed as an exhausted account-wide history.

Both successful jobs measured two native CPUs and `8336855040` memory bytes,
and sixteen builder CPUs and `32997224448` memory bytes.
The failed first request did not reach the hardware consumer.
The image CLI command took `7817` milliseconds in one sample.
No performance ratio or complete package-test claim follows.

At `2026-10-18T15:38:26.135Z`,
the exact saved manifest was readable with `200`.
Only the two owned image tags were removed;
the same manifest then returned `404`.
The solely owned project `ng17dl3xrn` was deleted,
a fresh project query returned `not_found`,
and the manifest remained `404` at `15:38:29.592Z`.
These controls establish logical absence,
not physical erasure or completed storage garbage collection.
The native slot was released to the parent after terminal collection and cleanup.

The cumulative three-window reservation remains `USD 0.69` under the admitted `USD 1` guard.
It is conservative admission arithmetic,
not an invoice,
unseen account usage,
or a measured combined-platform bill.

[native-builds]: https://depot.dev/docs/container-builds/integrations/depot-ci
[github-builds]: https://depot.dev/docs/container-builds/integrations/github-actions
[sources]: ../audit/depot-platform-surface-sources-2026-10-18.evidence.json.gz
[exporters]: ../troubleshooting/depot-buildkit-exporters.md
[receipt]: ../audit/depot-native-buildkit-job-auth-2026-10-18.evidence.json
