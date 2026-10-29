# Depot CLI 2.102.23 collection: logger final guards and worker completion gaps

## Symptom

The frozen logger replay records original check outcomes but lacks complete final provenance.
The collector used Depot CLI `2.102.23`.
The GitHub CLI version inside the affected CI job was not retained.
These are separate incidents in cohort `complete-20261017T111555806Z`,
run `8nntjcgx9b`,
workflow `pm7ck11p16`:

- [Issue #1180](https://github.com/Aquaticats/Monochromatic/issues/1180):
  the final common-manifest fetch reports `gh: Bad credentials (HTTP 401)`.
- [Issue #1181](https://github.com/Aquaticats/Monochromatic/issues/1181):
  static and unit builds change tracked distribution outputs;
  the clean-tree guards reject those changes.
- [Issue #1182](https://github.com/Aquaticats/Monochromatic/issues/1182):
  a generated-consistency sandbox terminates before worker completion and publishes no archive.

The [retained evidence](../audit/depot-coherent-logger-2026-10-17.evidence.json)
distinguishes source-command observations from paired source and common-cohort completion.
The historical source and frozen harness were not changed.

## Root cause and established boundaries

### Final authenticated lookup

Native TOML mutation job `nrmthwzvnz`,
attempt `cnhrgd3dct`,
finishes its original source task with exit zero and retains equal local source identities.
Its initial authenticated common-manifest lookup succeeds at `11:19:38.790Z`.
The final lookup fails at `12:54:38.162Z` with HTTP 401.
All times describe 2026-10-17 in UTC.
The reason the credentials were rejected is unknown.
Elapsed time alone does not prove token expiry.

At frozen harness `3d976fe06d22ae01c54aa206cf021a78374a3b06`,
`package/logger/core.fuzz/mise.toml:614` invokes the authenticated fetch:

```ts
// package/logger/core.fuzz/mise.toml:614, frozen harness 3d976fe06d22
const object=JSON.parse(execFileSync('gh',['api','--method','GET','--header',
 'Accept: application/vnd.github.object+json',endpoint],
 {encoding:'utf8',maxBuffer:4*1024*1024,timeout:30000}));
```

The failed child process throws before the completion record is written.
The write occurs at `package/logger/core.fuzz/mise.toml:667`:

```ts
// package/logger/core.fuzz/mise.toml:667, frozen harness 3d976fe06d22
writeFileSync(join(directory,`logger-cohort-${phase}-results.json`),
 JSON.stringify(receipt,null,2)+'\n',{flag:'wx',mode:0o600});
```

The frozen workflow supplies the Actions token at
`.depot/workflows/depot-complete-logger-combined.yml:1280`:

```yaml
# .depot/workflows/depot-complete-logger-combined.yml:1280, frozen harness 3d976fe06d22
GH_TOKEN: ${{ github.token }}
```

The TOML-case final guard in
[issue #1179](https://github.com/Aquaticats/Monochromatic/issues/1179)
has no retained diagnostic.
It is not evidence of the same authentication failure.

### Tracked generated outputs

Static job `xn7ft1tmjw` changes tracked plugin bundles,
declarations and the IntelliJ JAR.
Unit job `8t3kxfvzcc` changes the tracked stop-reminder bundle.
The retained porcelain paths have positive matching and command-header rejection controls.

The source guard rejects tracked differences before collecting configured-input hashes,
at `package/monorepo-task/core-exec/mise.toml:1187`:

```ts
// package/monorepo-task/core-exec/mise.toml:1187, frozen harness 3d976fe06d22
const trackedStatus=git(['status','--porcelain','--untracked-files=no']);
assert.equal(trackedStatus,'','Coherent replay changed tracked source');
```

The common guard independently rejects them at `package/logger/core.fuzz/mise.toml:660`:

```ts
// package/logger/core.fuzz/mise.toml:660, frozen harness 3d976fe06d22
const sourceStatus=git(source,['status','--porcelain','--untracked-files=no']);
assert.equal(sourceStatus,'','Cohort source has tracked modifications');
```

These guards enforce their declared clean-tree contract.
Changed generated outputs do not establish a change to authored task definitions.
Restoring the outputs solely to make a guard pass would erase relevant evidence.

### Terminated generated-consistency worker

Job `vkv8rwn4l1`,
attempt `5d611nw71d`,
reports `Sandbox terminated before worker reported completion`.
Its raw log retains partial command records:
root generation and one sidecar generator exit zero,
then another sidecar's container adapter receives `SIGKILL`.
The external driver and final source guard subsequently report shell exit code `137`.
The common after-guard starts,
but no completed receipt or artifact is retained.

Command exit records do not recover the missing generated-difference records,
underlying output files or final bindings.
The initial inference from a missing archive to no available command observations was too broad;
the provider log preserves those observations.

The declared runner is `16x64`.
Its allocation lasts `713890` milliseconds,
from `11:18:57.762Z` to `11:30:51.652Z`.
The provider retains 133 memory samples without downsampling,
with peak utilization `0.9975022728697344` and average `0.7365051365670434`.
SIGKILL and utilization near the declared limit do not establish an OOM kill.
The available evidence does not identify the killer or worker termination cause.

## Verification

The immutable source is `ef01490f5e960f8ca0b424c12242caeb3c5a2b5e`.
The collector's unmodified Depot CLI source was checked against tag `v2.102.23`.
The actual read-only collection uses these command forms:

```sh
# Read-only diagnostics for the completed logger attempts
depot ci diagnose --attempt 5d611nw71d --org l4zncjjbj5 --output json
depot ci logs cnhrgd3dct --org l4zncjjbj5 --output json
depot ci logs 5d611nw71d --org l4zncjjbj5 --output json
```

The working catalog includes 94 local source pairs and 93 common-cohort pairs,
equal archived before/after identities,
and independently verified hashes for 650 configured source inputs.
The failing catalog contains the HTTP 401 after-lookup,
the static and unit tracked-output rejections,
and the generated worker's incomplete artifact and completion chain.
The generic primary registry contains 613 passes,
105 failures and one check skipped because its prerequisite build failed.
Only 199 logical primary checks have an observation with both completion pairs.

## Verified collection workaround

The API JSONL has stable `step_key` values but lacks `step_name` in these actual records.
Filtering only by step names incorrectly selected zero lines.
The corrected collector identifies the actual executed before/after group headers,
requires both phases,
then retains all records with each selected after-step key.
It recovers 219 guard records without pretending that missing metadata names imply missing diagnostics.
This repairs evidence extraction only;
it does not repair authentication or recreate missing completion receipts.

The CLI serialization path preserves both fields at `pkg/cmd/ci/logs.go:314`:

```go
// pkg/cmd/ci/logs.go:314, Depot CLI v2.102.23
StepKey:     line.GetStepKey(),
StepID:      line.GetStepId(),
StepName:    line.GetStepName(),
```

## What does not work

- Treating exit-zero mutation output as proof of the final authenticated common binding.
- Treating rebuilt tracked outputs as proof of edits to authored task definitions.
- Searching absent step names and concluding that the provider retained no diagnostics.
- Treating a missing archive as proof that no original commands ran.
- Inferring token expiry or OOM from elapsed time or utilization alone.
- Repeating the resource-consuming generated job without inspecting its partial execution and memory evidence.

## Upstream filing decision

The `.out-of-scope/` inventory has no Depot-specific exemption.
The current-repository duplicate search found no matching incident;
issues `#1180`,
`#1181` and `#1182` retain the separate findings.
No upstream issue or comment was sent.

The upstream filing constraints remain:

- Fault attribution:
  neither credential rejection nor sandbox termination is traced to an upstream defect.
  The tracked-output guards enforce this repository's own contract.
- Fixability:
  no architectural impossibility is established;
  the unidentified cause prevents choosing a justified upstream patch.
- Supported use:
  the exercised CLI paths collect completed CI logs,
  diagnostics and metrics.
  This does not establish the lifetime or issuer of the job credential.
- Contribution policy:
  no upstream contribution is proposed,
  so no maintainer willingness or AI-report acceptance claim is made.
- Maintainer direction:
  the previously inspected CLI logging changes do not establish a fix for these actual incidents.
  No refusal or promised fix is inferred.
- Prototype:
  the corrected consumer-side log selector is verified;
  no cause-specific upstream fix is established or presented as tested.

There is nothing attributable to add upstream yet.
The current-repository findings retain the observations needed for further diagnosis.
