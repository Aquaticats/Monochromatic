# Podman 5.8.7 omits Docker healthcheck extensions from OCI image inspection

## Symptom

The owned Depot service image answered `/health` and `/nonce` with HTTP 200,
and a missing-path control with 404,
under a bounded local Podman container.
Its raw OCI configuration contains a Docker `Healthcheck` extension.
`podman healthcheck run` nevertheless returned 125:

```text
# Observed Podman diagnostic, variable container identifier omitted.
container <owned-container> has no defined healthcheck
```

The original failed aggregate remains in the
[BuildKit evidence receipt](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json).
Successful HTTP responses alone did not satisfy its native-healthcheck assertion.

## Root cause

The installed `podman version --format json` reports version `5.8.7`
and Git commit `c593b672bf3db1173aebea565ebf1a724ea196dc`.
That commit matches the upstream `v5.8.7` tag.

At that commit,
`vendor/go.podman.io/common/libimage/inspect.go:172` handles OCI manifests,
while line 183 begins the Docker schema 2 case.
Only the Docker case extracts healthcheck metadata:

```go
// podman:vendor/go.podman.io/common/libimage/inspect.go
case manifest.DockerV2Schema2MediaType:
    // Other configuration decoding omitted from this excerpt.
    data.HealthCheck = dockerConfig.ContainerConfig.Healthcheck
    if data.HealthCheck == nil && dockerConfig.Config != nil {
        data.HealthCheck = dockerConfig.Config.Healthcheck
    }
```

The OCI case reads annotations and history without assigning `HealthCheck`.
`pkg/specgen/generate/container.go:148` then populates an unspecified container healthcheck
from that inspection result:

```go
// podman:pkg/specgen/generate/container.go
if s.HealthConfig == nil || len(s.HealthConfig.Test) == 0 {
    if err := applyHealthCheckOverrides(s, inspectData.HealthCheck); err != nil {
        return nil, err
    }
}
```

`libpod/healthcheck.go:85` emits the observed diagnostic when the resulting configuration is absent.
The raw Docker extension's presence in the OCI blob therefore does not establish that this Podman version imports it.

## Verification

The immutable original index is
`sha256:4f2c182b14f816dfe9ab08f531a8d49378b5f76767b8f56f7c9154f05d0ff3e7`
in owned project `27m5m3vjlv`.
Registry consumers checked both runtime configurations and their hashes.
Both configurations contain the same healthcheck program.
The local container ran the original image command,
returned the bound source,
architecture,
and runtime nonce,
then stopped with exit zero and was removed.
Its native Podman healthcheck failed as recorded.

The independent GitHub Docker consumer in workflow run `39109462308`,
job `117622633314`,
imported the same image index and progressed from starting to healthy.
This is a different runtime result for the same OCI metadata.
That job's later Git checkout failure belongs to a separate missing-tool incident.

## Verified consumer route

The GitHub Docker service consumer honored the original embedded check without changing the image.
The separate derived CI job image deliberately has no active service healthcheck,
because its job process is a workspace command rather than the original HTTP service.
Neither result repairs or converts the original local Podman image.

Runtime overrides and local Docker-format rebuilding have not been exercised for this fixture.
They receive no verified-workaround credit here.

## What does not work

Repeating the original local `podman healthcheck run` does not import the omitted image metadata.
The raw extension and successful HTTP responses are separate evidence from a configured native check.
The derived image's absent `Config.Healthcheck` is an accepted disabled state;
expecting a literal `Test: ["NONE"]` was a separate harness assertion defect.

## Upstream filing decision

The narrow Podman issue search for `OCI healthcheck` returned no entries.
A broader search for the exact OCI-format warning found the merged upstream
[healthcheck image-format test correction][podman-healthcheck-test].
Its body records the Docker-format requirement and an actual Docker-format positive control.
That is relevant precedent for Podman's metadata handling,
but its producer-side test does not replace this audit's raw OCI import observation.

No new upstream regression has been established and no external message was sent.
The pinned implementation and existing upstream precedent are sufficient to record this compatibility boundary.

[podman-healthcheck-test]: https://github.com/podman-container-tools/podman/pull/29631
