# Depot CLI 2.102.23 built-in Semgrep misses the selected Dockerfile control

## Symptom

The optional built-in `--lint` path did not report the selected missing-user Dockerfile control.
Hadolint's independent positive control worked.
This is not evidence that the built-in linter detects no problems,
or that Semgrep's current release has the same behavior.

The [BuildKit evidence](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json)
preserves the cloud observations and separate mount-free scanner controls.
The source and selected runtime boundaries are also recorded in the
[platform audit](../planning/depot-platform-surface-audit.md).

## Root cause

At upstream CLI source `1f96d6d488f938de97ea640b115e9567ebfd6fbf`,
`pkg/buildx/commands/lint.go:30` selects the old scanner image:

```go
// Upstream pkg/buildx/commands/lint.go:30
Semgrep = "returntocorp/semgrep:1.34.1"
```

`RunSemgrep` at `pkg/buildx/commands/lint.go:352` selects a mutable registry ruleset
and an absolute executable path:

```go
// Upstream pkg/buildx/commands/lint.go:353
args := []string{"/usr/local/bin/semgrep", "scan", "--config=p/dockerfile", "--json", "--quiet", "--disable-version-check", dockerfile.Filename}
```

The old scanner did not match the canonical Dockerfile with the retained rules,
while current Semgrep `1.181.0` matched it.
The suffixed `Dockerfile.semgrep` control also exposed filename-selection behavior.
These are distinct boundaries:
upgrading the engine is not automatically a filename-selection fix.

The old scanner itself emitted:
`Versions prior to 1.76.0 are no longer supported by Semgrep.dev, please upgrade.`
The separately resolved current image is
`docker.io/semgrep/semgrep@sha256:81474464c1f07483ba0682a44787fd93c91397419762a04abe2bafaae396304b`.
Merely replacing the image while retaining `/usr/local/bin/semgrep` failed with executable-not-found status `127`.
A mount-free current-image `/bin/busybox which semgrep` control returned `/usr/bin/semgrep`.

There are additional source-level concerns that the scanner comparison does not isolate.
`RunImage` at `pkg/buildx/commands/lint.go:338` discards the process wait error:

```go
// Upstream pkg/buildx/commands/lint.go:338
_ = proc.Wait()
```

`CaptureOutput.Write` at `pkg/buildx/commands/lint.go:365` splits each individual write at newlines.
It does not preserve incomplete fragments between writes.
The downstream unmarshal functions skip messages that fail JSON decoding.
These are source observations,
not established causes of the cloud finding's absence.
Do not call a scanner-image update a verified repair of the complete BuildKit wrapper.

## Verification

The public BuildKit evidence retains:

- The working Hadolint control and cloud Semgrep non-detection.
- Old-engine canonical and suffix targets with both fetched and retained rule bytes.
- Old-engine direct pattern controls.
- Current-engine version and help,
  its initial incorrect absolute-path failure,
  and corrected canonical finding/direct-pattern controls.

A further disposable CLI source prototype paired the engine,
its actual executable,
and explicit file selection against missing-user and user-present controls.
That local command-boundary prototype passed 14 controls:

- `CMD` and `ENTRYPOINT` without `USER` each produce the selected finding for canonical and suffixed filenames.
- User-present `CMD`,
  user-present `ENTRYPOINT`,
  and no-command inputs produce no findings,
  with every target actually scanned.
- Omitting `--scan-unknown-extensions` on the same current image/rules keeps the canonical positive
  but excludes each selected suffix target.

Each invocation was mount-free,
network-disabled,
limited to 2 CPU and 2 GiB,
and given a 60-second container lifetime.
The executable and arguments were extracted from the original/patched Go source.
The local process launcher replaces the BuildKit gateway;
the modified Go package was not compiled in this prototype.
It does not allocate another remote builder or establish BuildKit process-status propagation.

The [invocation evidence](../audit/depot-semgrep-invocation-2026-10-18.evidence.json)
and its linked source archive retain the exact patch,
fixtures,
rules,
image digest,
actual invocations,
raw JSON,
and local issue draft.

## Verified workarounds

Run the separately pinned current scanner for the retained canonical input and rules.
The retained local control's `--entrypoint semgrep` invocation found the selected rule.
This supplies independent findings for that input;
it does not make the built-in `--lint` command fail closed or qualify the entire ruleset.
Retaining the fetched rules makes this comparison reproducible;
continuing to fetch `p/dockerfile` can change rule contents independently of the engine.

## What does not work

Replacing the image without resolving the executable is insufficient.
Counting old-engine zero findings as a successful security check is also insufficient.
A canonical filename alone did not restore the old-engine positive control.
The process-wait and chunk-decoding source observations do not prove the incident's causal path.

## Upstream filing artifact

### Upstream filing decision

1.  Upstream ownership:
    the CLI selects the engine and invocation.
    A defect in the current Semgrep engine is not established.
2.  Fixable integration:
    yes.
    The CLI can change its engine,
    executable and filename selection.
    Process-status and capture behavior require separate evidence.
3.  Supported use case:
    the CLI explicitly exposes Dockerfile linting,
    including the Semgrep invocation.
4.  Contributions:
    the upstream README welcomes PR contributions;
    the fresh disposable clone's contribution-policy filename search and all-state AI tracker search found no ban.
5.  Maintainer refusal:
    not established.
    The all-state `semgrep` issue search returned no matches.
    Path history shows Semgrep integration on `2023-08-01`
    and the last listed source change on `2024-06-05`.
    Neither proves future maintainer acceptance.
6.  Prototype:
    the source-derived invocation candidate passed the 14 local controls.
    This qualifies its scanner/image/filename boundary;
    complete BuildKit gateway,
    process failure,
    output chunking,
    and compiled integration behavior remain unqualified.

No upstream issue or PR has been sent.
No current Semgrep bug report is justified by a reproduction confined to the unsupported old engine.
