# Depot CLI 2.102.22 Docker proxy does not expose cache occupancy through DiskUsage

## Investigation scope

The Depot trial needs account-wide storage evidence for its monthly cost model.
The available login runs builds,
but the documented organization API requires an organization token.
No aggregate occupancy was established.
This investigation checked whether the Docker-compatible proxy could supply an alternate read path.
No Docker integration setting was changed and no zero-usage result was observed.

## Source finding

The installed command reports `depot version 2.102.22 (2026-10-14)`.
Release tag `v2.102.22` resolves to commit `316bd631c66d863ab9004a369128656b0c9effae`.
Its `pkg/cmd/buildctl/proxy.go:424` returns an empty disk-usage response:

```go
// pkg/cmd/buildctl/proxy.go:424
func (p *ControlProxy) DiskUsage(ctx context.Context, in *control.DiskUsageRequest) (*control.DiskUsageResponse, error) {
    p.logger.Printf("DiskUsage called!\n")
    p.scheduleShutdown()
    return &control.DiskUsageResponse{}, nil
}
```

That method does not query cached records before returning.
An empty response from this source path is therefore not evidence of an empty remote cache.
This is a source inspection,
not an end-to-end invocation of Docker's disk-usage command.
[Depot CLI source](https://github.com/depot/cli),
[release](https://github.com/depot/cli/releases/tag/v2.102.22)

## Verification

The version and source were checked with:

```sh
# shell, from the Depot CLI source clone
depot --version
git show v2.102.22:pkg/cmd/buildctl/proxy.go
```

Evidence that exists:

- Remote build `14trfrrxg6` measured 5,719,616 KiB in `/target`
  and 700,660 KiB in `/cargo` inside its build step.
- The matching released proxy source explicitly returns an empty disk-usage response.

Evidence that remains unavailable:

- Complete BuildKit cache occupancy,
  registry and snapshot occupancy,
  and account-wide billable storage.
- An end-to-end Docker proxy disk-usage probe;
  the source finding was sufficient to reject it as an occupancy measurement strategy.

The build-directory counts omit layers,
other architectures,
other projects,
and registry or snapshot storage.
[Measured build](https://depot.dev/orgs/l4zncjjbj5/projects/72lp3rlfvw/builds/14trfrrxg6)

## Verified workarounds

None for aggregate account occupancy with the available credential.
The in-build directory measurements are usable only for their named directories.
The account's supported dashboard or organization API still needs authorized access.
No credential was created or exposed.

## What does not work as evidence

- Treating the proxy's empty response as a measurement of remote cache contents.
- Treating a project registry listing as a complete cache or snapshot inventory.
- Treating cache-retention configuration as actual occupancy or an aggregate spending cap.

## Upstream filing decision

The `.out-of-scope/` inventory contains no Depot-specific exemption.
The Depot CLI issue tracker searches for `disk usage` and `cache size` returned no matches.
No source-backed defect in a documented occupancy feature was established,
so there is nothing actionable to file.

- Upstream fault:
  unproven;
  the inspected method deliberately returns no records.
- Fixability:
  not evaluated because no incorrect implementation contract was established.
- Supported use case:
  Docker build compatibility is documented,
  but that alone does not promise account-wide billing introspection through this method.
- Contribution acceptance:
  not evaluated because no contribution is proposed.
- Likelihood of change:
  unknown;
  empty search results are not evidence of rejection or acceptance.
- Prototype:
  none because no applicable defect was established.

No issue,
comment,
or patch was submitted.
