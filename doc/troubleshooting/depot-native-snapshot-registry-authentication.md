# Depot native snapshot registry authentication

## Observed boundary

On 2026-10-18,
an exact owned-image `depot.registry.v1beta1.RegistryService/GetImageDetail` request
to `https://api.depot.dev` returned HTTP `401`,
code `unauthenticated`,
message `Invalid token`,
with the existing CLI login.
The same stored login returned HTTP `200` from the exact scoped native `CIService/GetWorkflow` request.
That distinguishes these observed request paths;
it does not establish the server-side reason for the registry RPC rejection.

The [sealed receipt](../audit/depot-native-snapshot-absence-2026-10-18.evidence.json)
and its [source archive](../audit/depot-native-snapshot-absence-2026-10-18.sources.json.gz)
retain the source contracts,
selected authentication controls,
creator identities,
timestamps,
and original response hashes.
The complete CI response remains private;
only the required response status and request identity are published.

## Exercised OCI authentication bridge

Current Depot registry documentation permits a user access token as the registry password,
with username `x-token`.
The exact OCI manifest request returned a Bearer challenge naming
`https://api.depot.dev/auth/registry/token`,
service `l4zncjjbj5.registry.depot.dev`,
and scope `repository:research-reuse-20261018:pull`.

The investigation followed the deciding CLI registry-authentication source
and its pinned containerd `v1.6.20` implementation.
An authenticated GET challenge exchange returned HTTP `200`
and a short-lived registry bearer with a `900`-second expiry.
The selected JWT audience matches that exact registry host.
Its only repository authorization names `research-reuse-20261018` with action `pull`.
No offline or refresh token was requested.
No persistent credential was created,
and token values remained only in process memory.
No token value was persisted or published.

With that bearer,
the three owned tags and their exact immutable digests each returned HTTP `404`,
code `MANIFEST_UNKNOWN`.
A deliberately invalid noncredential bearer returned HTTP `401` on the identical host and manifest path.
This response control distinguishes authentication rejection from the scoped missing-manifest responses.
The creator-to-host/repository/tag/digest joins are explicit in the receipt.

## Cleanup disposition and limits

The queried tags are `tools-r1`,
`tools-r2-git256`,
and `tools-r3-fuzz-codex`.
Their original producers are `xkmm08vpjs`,
`3jg1fmkc69`,
and `jf0c9llb1v`,
respectively.
The final producer's snapshot production is retained separately from its failed profile conclusion.

All queried tags and immutable digests were absent at the recorded read timestamps.
No deletion request or cleanup cloud job ran.
The investigation does not establish expiry as the cause,
when the references disappeared,
or storage reclamation.
The historical snapshot input `--max-age 6h` is not a demonstrated explanation for the current absence.
Local worktrees and source archives remain retained.

The ordinary `depot image rm` command is a different surface:
the captured CLI calls project-scoped `build.v1.RegistryService.DeleteImage`
and resolves project authentication.
It must not be inferred to delete an organization-hosted native snapshot.
The captured native registry schema instead declares repository/tag/image deletion RPCs.
Their mutation/authentication behavior was not exercised in this investigation.
The OCI distribution specification also makes manifest deletion optional;
its source contract alone does not prove that this registry implements deletion.

## Source identities and verification

The source archive retains current Depot custom-image and registry documentation,
CLI revision `1f96d6d488f938de97ea640b115e9567ebfd6fbf`,
proto revision `01dc47fdd504fe88697aec8b3033ce887588d302`,
containerd revision `2806fc1057397dbaeefbea0e4e17bddfbd388f38`,
and OCI distribution-spec revision `97274622c11112caa21efb8c52acca3c6b8fa7f1`.
The installed Depot CLI was `2.102.23`.
The relevant deciding source paths are `pkg/cmd/push/token.go`,
`pkg/api/rpc.go`,
`pkg/api/ci.go`,
`pkg/helpers/token.go`,
`pkg/cmd/image/rm.go`,
`proto/depot/registry/v1beta1/registry.proto`,
`remotes/docker/auth/fetch.go`,
and `spec.md` in their respective captured repositories.

No account-wide repository listing,
settings change,
broader token creation,
or new compute reservation was used.
