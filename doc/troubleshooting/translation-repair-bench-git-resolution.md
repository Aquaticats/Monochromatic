# Node 26.7.0 exhausts its heap when corpus bench reads repeatedly resolve Git

## Symptom

Writer-calibration preparation on frozen `45e64e411` failed in a Podman 5.8.4 container
limited to 2 GiB memory,
2 CPUs and 512 PIDs,
with no network and read-only corpus/source mounts.
The container's Node process reported:

```text
FATAL ERROR: Ineffective mark-compacts near heap limit Allocation failed - JavaScript heap out of memory
```

The GC trace was near 1023 MiB of V8 heap,
and the native stack included `StringDecoder::DecodeData`.
The inner container exited 139.
That exit code is not the diagnosis by itself;
the emitted fatal error identifies heap exhaustion.
No writer model call had started.

A logger warning also occurred:

```text
logger internal error: file sink verification failed: EROFS: read-only file system
```

Its full path was the source worktree's `node_modules/.monochromatic` log destination.
This warning also occurred in passing controls and after the heap fix.
It was a separate output-mount problem,
not established as the heap failure's cause.

## Root cause

The batch in `package/translation-repair/bench/src/corpus-run/bench-sample.ts:200`
launches entry reads concurrently:

```ts
// package/translation-repair/bench/src/corpus-run/bench-sample.ts
const sliced = await Promise.all(
  entryIds.map(async function sliceOne(entryId,): Promise<readonly BenchSlice[]> {
```

Each entry starts its source and English file reads together at that file's line 93.
Without an explicit executable in the pin,
`package/translation-repair/corpus-source/src/corpus-source.ts:359`
resolves Git for each text read:

```ts
// package/translation-repair/corpus-source/src/corpus-source.ts
const gitPath = pin.gitPath ?? await resolveGit();
```

Resolution is not merely a PATH existence check.
`package/git-policy/cli/src/resolve-git.ts:193`
reads each candidate as UTF-8 to detect a wrapper pointing back to cli-git:

```ts
// package/git-policy/cli/src/resolve-git.ts
const content = await readFile(
  candidatePath,
  'utf8',
);
return [...SELF_SHIM_MARKERS,].some(function hasSelfShimMarker(marker,) {
  return content.includes(marker,);
},);
```

In the measured image,
`/usr/bin/git` was 4537488 bytes and decoded to 4475298 UTF-16 code units.
The whole corpus's 184 page files totalled only 841615 bytes.
The concurrent repeated executable decoding,
not a giant corpus page,
was the load-bearing path.

A minimized case needs no parsing or slicing:
186 simultaneous reads of the same 3731-character pinned file reproduce the heap failure.
Supplying the already verified native Git path makes those same reads complete.
The unchanged full sampler likewise succeeds with that explicit path.
This narrows the cause beyond the initial hypothesis of generic corpus I/O concurrency.

## Fix

`d6be6e978` resolves the executable once at the owned sampling boundary,
then passes the same resolved pin to listing and every entry read.
At `bench-sample.ts:184`:

```ts
// package/translation-repair/bench/src/corpus-run/bench-sample.ts
const resolvedPin: CorpusPin = {
  ...pin,
  gitPath: pin.gitPath ?? await resolveGit(),
};
```

An explicitly supplied path is preserved,
the caller's pin is not mutated,
and the batch owns its revision before asynchronous reads begin.
No process-wide executable cache was added.
No shim-marker check,
intrinsic-object flag,
no-fetch protection,
source byte or sample-selection rule was weakened.

The old comment calling resolution negligible was removed because the measured path disproved that description.

## Verification

All heap probes used the resource-limited container,
not an unbounded host stress run.
The private reproducible harness is
`~/temp/agent/v41-writer-20260911.sEDtepyo/run-minimize.mts`.
Its modes preserve the same container envelope.

Working controls:

- Importing the frozen bundle.
- Listing all 93 entry directories.
- Reading one pinned `gqt` source file.
- One hundred sequential reads of that file without retaining its texts.
- One hundred concurrent reads of that file.
- The 186-read minimized case with `pin.gitPath` supplied.
- The old full sampler with that explicit path.
- The fixed default sampler without an explicit path.

Failing controls:

- The original full sampler,
  reproduced twice with the same heap diagnostic.
- The 186-read minimized case without `pin.gitPath`.

The executable-read regression in `bench-sample-draw.unit.test.ts`
observes actual `fs/promises.readFile` calls in a separate process against an invented Git fixture.
The pre-fix build read the executable three times for one sample;
the fixed build reads it once.
An explicit pin reads it zero times,
returns the same sample and leaves the caller's pin unchanged.
The red is `cf982b708` and `v41-bench-resolution-red-20260911.out`.

The full bounded after-state compared every returned field of all forty slices
with the old sampler using the verified explicit path.
They were identical.
The comparison returned with heap usage 90086072 bytes and no cgroup OOM event.
`fixed-sample-verification.json` retains the complete comparison,
and `v41-writer-fixed-sample-20260911.out` retains its metadata.
The missing `tdor` page was skipped in both cases under the existing sample policy.

Package build,
types and Oxlint pass.
`v41-bench-resolution-final-unit-20260911.out` ends `unit exit 0` at line 9169.
The exact frozen `67823bc55` artifact was then sampled in the same bounded container,
with the private log overlay.
It returned the same forty rows,
reported the configured memory/CPU/PID limits,
and recorded no OOM or PID-limit event.
`v41-writer-frozen-preflight-20260911.out` and the owned workspace's `writer-plan.json` retain that consumer check.

## Verified workarounds and container setup

For an older artifact,
resolve native Git once and supply it through `CorpusPin.gitPath` for the batch.
The measured Linux probe used `/usr/bin/git`;
production code uses the existing platform-aware resolver.
Tradeoff:
the selected executable path is held for the operation rather than rediscovered per file.

The local terminal image supplied Git 2.52.0,
and an actual invocation accepted `--no-lazy-fetch`.
It did not supply Node on PATH.
The mounted Node 26.7.0 runtime needed a read-only `libatomic.so.1` mount and
`LD_LIBRARY_PATH=/lib-extra`;
`ldd` identified that dependency and the dynamic-loader error disappeared after the mount.

Label-enabled execution of the mounted runtime was denied.
An otherwise matched version probe with `--security-opt=label=disable` succeeded.
Podman v5.8.4's `docs/source/markdown/options/security-opt.md:17`
defines that option as turning off label separation.
Tradeoff:
SELinux process-label separation is disabled for this container;
user namespaces,
read-only mounts and resource limits remain.
Shared source trees were not relabeled.
This is not a claim of full confidential isolation.

The logger's read-only-path warning is handled separately by mounting the owned run's
`runtime-logs` directory over the container's source-worktree `node_modules/.monochromatic` path.
The `fixed-logs` control preserves the same forty slices and completes without that warning.
Tradeoff:
logs are writable only in the owned output overlay,
not the real source worktree's log directory.

## What does not work

- Treating the exit code alone as proof of an arbitrary native crash.
  The retained V8 diagnostic names heap exhaustion.
- Blaming the logger warning merely because it preceded the heap error.
  The same warning appears in passing controls.
- Inferring a giant input from heap size.
  Git's pinned blob metadata disproves that hypothesis.
- Treating one hundred successful parallel reads as proof that the larger batch is safe.
  The 186-read case crosses the observed failure boundary.
- Increasing host memory or removing resource bounds instead of fixing repeated resolution.
  Neither was used as the remedy.
- Globally caching executable selection or bypassing wrapper detection.
  Neither is needed when the batch can own and pass its resolved path.

Proposed `AGENTS.md` clarification to `QJ1`,
not applied:

```text
QJ1: Measure quantitative claims. Include delegated I/O and setup costs; a wrapper call is not a cost model.
Never call work negligible, fast, or small without evidence.
```

The expected action was performed:
the delegated resolver was traced,
its executable read was measured,
and the batch now passes the ownership-known primitive rather than repeating the setup.

## Upstream filing decision

No external upstream filing is warranted.
`.out-of-scope/` was checked;
no Node or Podman exemption is needed to explain this decision.
The defect is in this repository's composition of its sampler and Git resolver.

- Upstream fault:
  not established for Node or Podman;
  the memory cap and missing-library/label refusals behaved as observed boundaries.
- Ability to fix:
  the owned caller was fixed without changing those tools.
- Supported use:
  `CorpusPin.gitPath` already supported an explicitly resolved executable.
- Contribution policy:
  no external contribution is proposed.
- Expected resolution:
  the owned fix is committed and verified.
- Compatible prototype:
  explicit-pin controls,
  the executable-read regression and the unchanged forty-slice comparison establish the remedy.

The source commits and this record are the filing artifact.
No external issue or comment should be sent from these findings.
