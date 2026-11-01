# V4.1 writer calibration preflight

Note of 2026-09-16:
the task-38 block named below is lifted with the teardown recorded in
[the teardown decision](../decision/translation-repair-provider-free-layer-teardown-2026-09-16.md);
the intended measurement stands and is the next thing between passes.

Task 37's heap fix and bounded verification are complete.
Task 35 is now blocked by task 38's independently discovered benchmark unit-scope defect.
The heap fix's unchanged-output proof is not writer-admission evidence.
No writer generation or image-reading calibration has run.
Judge admission is verified independently and remains seated.

## Intended measurement

Use the existing `producer-calibrate 40 --candidates deepseek-v4.1-flash` instrument
and pooled-null writer rule,
with the current measured roster,
preserved self-vote/contributor handling and no extra generation rounds.
The pre-fix post-judge runtime is frozen at `45e64e411`.
The resolved-batch implementation is now checked and frozen at `67823bc55`.
The source corpus stays at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379` and is not edited.

`bench-sample.ts` reads all pinned entries before selecting the deterministic source-size spread.
Its `Promise.all` launches entry reads concurrently.
Preparation is therefore being exercised in a resource-limited container,
not by an uncapped host probe.

## Container probe facts

The attempted envelope is:

- 2 GiB memory.
- 2 CPUs.
- PID limit 512.
- No network during preflight.
- Read-only corpus and source mounts.
- Only the owned writer workspace writable.

Owned workspace:
`~/temp/agent/v41-writer-20260911.sEDtepyo`.

The available Node 22 Bookworm image has Git 2.39.5.
The local terminal image has Git 2.52.0,
and its actual invocation accepts `--no-lazy-fetch`.
It has no `node` executable on its default PATH.
Binding the installed Node 26.7.0 runtime required its `libatomic.so.1` dependency;
with that read-only library mount,
`node --version` succeeds.
A prior bind-mount execution was denied before the dynamic-loader probe.
SELinux labeling and library availability remain distinct boundaries,
not interchangeable explanations.

The container disables SELinux process labeling instead of relabeling shared source trees.
Its user namespace,
read-only mounts and resource limits remain active.
This is a tradeoff to document with the completed source trace,
not a claim of full confidential isolation.
No provider keys were supplied to the preflight.

## Pre-fix failure

`proc_248e` ran the original sample preflight and exited 139 after twelve seconds.
`v41-writer-container-preflight-20260911.out` records:

```text
logger internal error: file sink verification failed: EROFS: read-only file system
FATAL ERROR: Ineffective mark-compacts near heap limit Allocation failed - JavaScript heap out of memory
```

The full file-sink diagnostic names the read-only source worktree's
`node_modules/.monochromatic` output path.
V8's trace reports heap collection around 1023.5 MiB and includes `StringDecoder::DecodeData`.
These are separate symptoms until a probe establishes a relationship.
An exit-code label alone is not the diagnosis;
the observed fatal diagnostic is heap exhaustion.

Import,
listing all 93 entries and reading the pinned `gqt` source file succeed under the same envelope.
The logger EROFS warning occurs in those passing controls too.
A second full-sampling attempt reproduces the same heap exhaustion after the listing checkpoint.

Ranked hypotheses used to distinguish the failure:

- Concurrent reads exceed the working set.
  Serial reads of the same bounded input should remain within the limit.
- A completed read retains memory.
  Serial repetition should continue growing and fail too.
- A particular entry or slicing operation causes the growth.
  Repeated read-only controls should pass,
  while an isolated parse/slice path reproduces it.

The first distinguishing control reads the same pinned 3731-character file one hundred times sequentially,
without retaining its texts,
inside the same resource-limited container.
`minimize.mts` and `run-minimize.mts` retain the same bounded container envelope.
No larger host run or production source change has been used to evade this failure.
## Verified cause and fix

The 186-read minimized case reproduces heap exhaustion while reading one bounded file,
without any parsing or slicing.
Supplying its already resolved Git path makes the same case complete.
The image's Git executable is 4537488 bytes and decodes to 4475298 UTF-16 units.
Each default corpus read called `resolveGit`,
whose self-shim check decodes the candidate executable.

`d6be6e978` resolves the executable once before the bench batch and passes that owned pin to all reads.
The existing protections and sample-selection policy remain unchanged.
Regression `cf982b708` observed three executable reads before the fix and one after it;
an explicit pin observes zero and returns identical output without mutating caller state.

The actual fixed default sampler completes in the same 2 GiB container.
All forty slices are field-for-field identical to the old sampler with a resolved pin.
No cgroup OOM event occurred.
The logger warning was separately removed by an owned writable output overlay,
without making the corpus or source writable.
The exact frozen `67823bc55` consumer check then returned the identical forty-row sample.
Its digest is `sha256-tree-v1:d0f505681c37619bf92114d706d93d74bac4b2a1d7e5bd023dd6dc01895188c3`.
The cgroup reported 2147483648-byte memory,
2 CPU quota,
512 PIDs,
and no OOM or PID-limit event.
`writer-plan.json` in the owned workspace is the preflight sample/provenance record.

Build,
types and Oxlint pass;
`v41-bench-resolution-final-unit-20260911.out` ends `unit exit 0` at line 9169.
The complete source trace,
controls and container tradeoffs are in
[the troubleshooting record](../troubleshooting/translation-repair-bench-git-resolution.md).
The initial red command also triggered automatic mise tool preparation after mise changed to 2026.9.5;
subsequent checks explicitly used `--no-deps --skip-tools`.
Unrelated `mise.lock` drift remains unstaged and is not part of this fix.

## Separate benchmark scope blocker

Full sample reading and a compiled validator probe found a concrete cross-row obligation:
`windward0032#14` supplies only a source heading,
but its incumbent floor requires the body paragraphs whose source is in sibling `#15`.
A heading-only output is rejected while the full incumbent passes;
the `lintong#0` heading-only positive control succeeds.
The old launch plan is superseded for calibration,
and its launcher now refuses paid `run` mode.
No writer generation occurred.

[Writer unit scope](translation-repair-writer-unit-scope-2026-09-11.md)
records the evidence,
withdrawn conditional advice and constraints for task 38.
The production archive floor and existing forty-round pooled-null policy are not weakened.
