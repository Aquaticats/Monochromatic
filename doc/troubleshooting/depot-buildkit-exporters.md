# Depot CLI v2.102.23 rejects save combined with OCI or local exports

## Symptom

`depot bake --save` failed with:

```text
multiple outputs currently unsupported
```

The selected targets already had `type=oci` or `type=local` outputs.
The CLI launched both architecture builders,
then rejected the combination before executing the Dockerfile.
The observed build was `7wjrw9szj0` in isolated research project `27m5m3vjlv`.

## Root cause

The installed release is `v2.102.23`,
with [CLI source][cli] pinned to commit `1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
The source was retrieved and separately checked in a shallow clone at that commit.
Exact source is retained in the [platform source archive][sources].

The Bake command calls `registry.WithDepotSave` for save requests.
`pkg/buildx/commands/bake.go:141` contains:

```go
// depot/cli:pkg/buildx/commands/bake.go
if in.save {
    opts := registry.SaveOptions{
        // Other project, build and credential fields omitted.
        AddTargetSuffix: true,
        RequestedTargets: requestedTargets,
    }
    buildOpts = registry.WithDepotSave(buildOpts, opts)
}
```

`pkg/registry/cli.go:39` identifies image exporters:

```go
// depot/cli:pkg/registry/cli.go
for i, export := range buildOpt.Exports {
    if export.Type == "image" {
        imageExportIndices = append(imageExportIndices, i)
    }
}
```

When no image exporter exists,
`pkg/registry/cli.go:53` appends one:

```go
// depot/cli:pkg/registry/cli.go
if len(imageExportIndices) == 0 {
    exportImage := client.ExportEntry{
        Type: "image",
        Attrs: map[string]string{"push": "true"},
    }
    exports = append(exports, exportImage)
}
```

An OCI or local exporter is retained alongside the new image exporter.
The subsequent option conversion rejects that list.
`pkg/buildx/build/build.go:433` contains:

```go
// depot/cli:pkg/buildx/build/build.go
switch len(opt.Exports) {
case 1:
    // valid
case 0:
    // Driver-specific default handling omitted.
default:
    return nil, nil, nil, errors.Errorf("multiple outputs currently unsupported")
}
```

This conclusion concerns the installed Depot CLI path.
It does not establish a restriction for every BuildKit version or exporter implementation.

## Verification

The [campaign receipt][campaign] preserves source identity,
arguments,
terminal outcomes,
and the registry consumer.
The rejected combination used fixture source `d9cf7458443c5c4b7d169ba18cdb80a7fcf34cbf`.
Its native command exited one after 10.986 seconds.
The request guard did not fire.

The failing catalog contains:

- An OCI output plus `--save`.
- A local output plus `--save`.

The successful catalog contains:

- Native Bake print resolution with one image exporter.
- An image-only service target plus `--save`,
  which completed as build `fxzj58r8m2` in 28.003 seconds.
- Registry reads of the resulting multi-platform OCI index,
  configuration blobs,
  provenance,
  and SBOM subjects.

The successful fixture commit is `5cfc1d90f92154cf62fa95789fad9bd7deeda208`.
Its task invocation was:

```sh
mise run //.depot/probe/platform-buildkit:probe:build-service
```

The task is deliberately single-use per attempt.
Do not repeat its cloud request to repair a later artifact assertion.

## Verified workaround

For registry save,
the exercised invocation replaced the OCI output with one image exporter:

```sh
depot bake --file docker-bake.hcl --project 27m5m3vjlv \
  --save --save-tag monochromatic-platform-buildkit-20261018-v1 \
  --set service.output=type=image --metadata-file metadata.json --progress plain service
```

This produces the saved registry image.
It does not simultaneously produce an OCI archive or local filesystem export.
Those consumers need separate single-export requests and their own receipts.
The separate archive and local export cases remain part of the campaign.

## What does not work

- Keeping the OCI or local output and adding `--save` creates multiple exporters.
- Successful `bake --print` proves option resolution,
  not remote export or artifact consumption.
- Changing a post-build tag assertion does not require rebuilding a successfully saved image.

The image-list response's `tag` is a full registry reference.
The original bare-tag comparison was our harness error,
recorded separately from exporter rejection.

## Upstream filing decision

The `.out-of-scope/` inventory was checked;
no Depot exemption applies.
A public issue and pull-request search for `multiple outputs` found [CLI pull request 627][upgrade].
Its complete description and comments were read.
The open change describes this exact error and replaces the older Buildx implementation.

1.  The current behavior is an explicit exporter restriction;
    a new upstream defect is not established by the rejected combination.
2.  Upstream can change the implementation;
    the open upgrade contains such work.
3.  The combination is addressed in that upgrade,
    but the installed release does not provide its proposed behavior.
4.  No new contribution is proposed,
    so contribution-policy permission is not claimed.
5.  The open change shows active work,
    without establishing a release date or acceptance guarantee.
6.  No separate prototype is needed for an already implemented upstream candidate;
    our verified consumer-side workaround is retained instead.

There is nothing additive to file from this reproduction.
No upstream issue or comment was sent.

[campaign]: ../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json
[cli]: https://github.com/depot/cli
[sources]: ../audit/depot-platform-surface-sources-2026-10-18.evidence.json.gz
[upgrade]: https://github.com/depot/cli/pull/627
