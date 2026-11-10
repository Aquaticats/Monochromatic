# GitHub Actions macOS arm64 job remains queued under runner-image branch label

## Symptom

A platform matrix used `macos-15-arm64` to request Apple Silicon.
Linux and Windows jobs started immediately,
but every macOS job remained queued until its workflow run was cancelled.
No setup step or repository code ran.

## Root cause

`macos-15-arm64` is a runner-image release branch and release-tag prefix,
not a standard GitHub-hosted workflow label.
The existence of releases such as `macos-15-arm64/20260706.0213` in `actions/runner-images` does not make that string a
runner selector.

GitHub's hosted-runner reference retrieved on 2026-07-13 lists these standard public macOS labels:

- Intel:
   `macos-15-intel` and `macos-26-intel`;
- arm64 M1:
   `macos-latest`,
   `macos-14`,
   `macos-15`,
   and `macos-26`.

The same reference identifies `macos-15` as three-core M1 arm64 with seven gigabytes of memory.

## Resolution

Use `macos-15` for an explicit macOS 15 arm64 host:

```yaml
strategy:
  matrix:
    os:
    - ubuntu-latest
    - macos-15
    - windows-latest
```

`macos-latest` also currently selects the standard arm64 image,
but `macos-15` records the operating-system generation required by this acceptance workflow.
The tradeoff is that the workflow will not automatically advance to a later macOS generation.

## Verification

After replacing `macos-15-arm64` with `macos-15`,
the new matrix run entered `in_progress` instead of retaining a queued macOS job under an unmatched label.
Completion still requires the plugin build,
TypeScript 7 lifecycle probe,
and external-consumer probe to pass on that host.

## What does not work

- Inferring workflow labels from `actions/runner-images` branch or release names is unreliable.
- Waiting longer does not provision a standard runner for an unsupported label.
- `macos-15-intel` verifies macOS but not the required arm64 native binary.

## Upstream filing artifact

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** No.
    The workflow used a runner-image branch name instead of a documented runner label.
2.  **Can upstream fix it?
    ** No product defect was established.
3.  **Are they supporting this use case?
    ** GitHub supports macOS arm64 through `macos-15` and the other documented labels.
4.  **Would the repo welcome our contribution?
    ** Not applicable because the deciding documentation is correct.
5.  **Will they likely fix it?
    ** No upstream change is needed.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** The workflow label replacement is the minimal
    fix and started a standard runner job.

Nothing should be filed upstream.
