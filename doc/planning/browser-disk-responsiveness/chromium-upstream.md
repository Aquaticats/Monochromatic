# Chromium contribution targets for storage-related UI stalls

## Status checked at 2026-11-03T04:06:28-05:00

This is a proposal,
not a source patch or a filing recommendation.
Installed-version evidence uses `2f9a11097447f226133bc1a31c787291a5eeab98`.
Current upstream source was fetched at `e436fd72f52814c1228fb16a8c8f7cae8253fb99`
(`main` commit time `2026-11-03T08:44:16Z`).
No prototype,
upstream post,
fork,
or browser configuration change was made.

## Recommended work order

### Establish an upstream regression case for the observed metrics write fault

Start with a sanitized reproduction and regression test for the confirmed
`BrowserMetrics` mapped-write stall.
Retain the decisive relationship:
browser-main instruction pointer plus write-fault address inside `.pma`,
and a matching `btrfs_page_mkwrite`/writeback stack.
Do not substitute a generic `fsync` benchmark or the unrelated GPU cache path.
The useful contribution is a reproducible invariant:
blocking diagnostic persistence must not stop UI input processing.

The current normal-browser startup path still selects mapped histograms before field trials,
so a normal post-initialization feature toggle is not a sufficient remedy.
Headless mode is explicitly excluded from this startup branch.
`components/metrics/persistent_histograms.cc` is byte-identical between the installed revision and checked main.
Source:
[early mapped-metrics choice][metrics-init].

Pros:
directly addresses measured evidence,
gives maintainers a testable failure boundary,
and supports evaluating alternative designs.
Cons:
the existing freeze trace alone is not a portable reproduction;
page faults bypass ordinary file-I/O mocks and blocking annotations.

### Remove optional GPU-cache waits from latency-critical callbacks

Build on the existing memory cache and asynchronous preload machinery.
A practical first candidate is to let a memory hit proceed without first waiting for disk-cache initialization.
For a memory miss,
design a bounded background fetch that owns its result buffer;
the synchronous shader callback returns a cache miss if data is not ready.
Never retain the callback's borrowed destination buffer after return.
This is a proposed design,
not yet a proved patch.

Current source still calls `cache_->Find` synchronously after a timed gate.
The `20 ms` gate limits waiting for another operation,
not the subsequent filesystem read.
Current `EnsureDiskCacheInitialized()` also calls an unbounded `Wait()`;
`LoadImpl` calls that before checking its memory cache.
Sources:
[disk lookup][gpu-read],
[initialization join][gpu-init-wait],
[memory-lookup ordering][gpu-load-order].

The cache contract explicitly allows eviction and misses:
entries are best effort,
and any process with the file handles may access the store.
Preserve that sandbox/file-handle model,
cross-process database synchronization,
transient versus permanent error handling,
GPU-restart lifetime,
memory-pressure budgets,
and Vulkan monolithic-entry exclusions.
Source:
[persistent-cache contract][cache-contract].

Pros:
miss semantics already exist,
and a narrow change can reuse owned worker state and the existing memory cache.
Cons:
more misses can move latency into shader compilation;
background reads can outlive the consumer,
and memory-cache locks must never cover storage work.
Measure cold and warm startup,
first presentation,
cache-hit rate,
compilation time,
memory use,
and GPU restarts together.

### Redesign metrics persistence after choosing its crash-loss contract

The durable direction is to keep live histogram mutation in RAM-backed memory
and perform storage persistence outside latency-critical execution.
An in-process snapshot writer may lose the tail of metrics on process death;
an external collector retaining a shared-memory handle can survive browser death
but adds lifetime,
sandbox,
resource,
and crash-recovery obligations.
Neither provides a guarantee against all swap or system-wide memory stalls.

Do not implement this as a late allocator swap:
histogram objects keep pointers into allocator memory and the global allocator is not destroyed.
Existing `LocalMemory`,
shared-memory histogram allocation,
and atomic snapshot-writing APIs are useful precedent,
not proof that periodic concurrent snapshots preserve current telemetry semantics.
Preserve sparse and ordinary sample counts,
system-profile association,
cross-process imports,
early-startup/early-exit diagnostics,
consent/deletion behavior,
format compatibility,
and collection after abnormal termination.
Source locations:
`base/metrics/persistent_histogram_allocator.h:480-522`,
`base/metrics/histogram_shared_memory.cc:149`,
and `components/metrics/persistent_histograms.cc:310-329` in the installed checkout.

Pros:
removes the specific filesystem-backed write-fault mechanism from every histogram caller.
Cons:
this is a telemetry architecture change,
with a real crash-diagnostics tradeoff requiring metrics-owner agreement and a design document.
Preallocation,
prefaulting,
moving explicit flushes to a worker,
or lowering I/O priority does not by itself prevent later mapped-page write faults.

Ranking:
metrics reproduction/test > GPU callback isolation > metrics persistence redesign,
because the reproduction anchors work in an observed failure;
GPU caches already have a best-effort miss contract,
whereas metrics persistence needs an explicit crash-loss and lifetime design before implementation.
The metrics redesign remains the causal long-term target for the confirmed mapped-write mechanism.

## Existing upstream work to coordinate with

- [Merged GPU initialization work][gpu-async-commit],
  commit `db4fa7da45bbffd5a2fcc2b4eb6ca267e41d0522` on `2026-10-26`,
  moves initialization off GPU I/O to a worker to avoid blocking browser-main channel establishment.
  Its feature is disabled by default in checked main.
  It deliberately retains a lookup-time initialization wait.
- [Merged metadata/preloading work][gpu-preload-commit],
  commit `f2fc41911596344ad1399b9035f375f1413514e5` on `2026-06-04`,
  already preloads recorded keys into memory.
  `kGpuPersistentCacheMetadata` is also disabled by default.
  Source:
  [feature defaults][gpu-feature-defaults].
- [Open DSE preloading prototype][gpu-dse-wip],
  CL `8412185`,
  explicitly discusses synchronous SQLite misses on GPU main and startup task priorities.
  It remained `NEW` in the Gerrit response,
  last updated `2026-09-16`.
  Avoid duplicating generic preloading work;
  its scope is Android Default Search Engine rendering,
  not a universal no-disk-wait contract.
- [Existing cache timeout precedent][gpu-timeout-commit],
  commit `076b8b99b9729d4266e43df22fc96d06eb34421a`,
  explicitly trades waiting on cache writes for a cache miss and recompilation.
  Extending this principle to actual I/O needs an asynchronous owner,
  not a timer around an uninterruptible read on the same thread.
- [Open BrowserMetrics segment-reference change][metrics-segments],
  CL `8707393`,
  remained `NEW`,
  last updated `2026-11-02`.
  Its description attributes main-thread shutdown hangs to cold mapped-page reads while scanning unrelated allocations.
  It caches system-profile segment references.
  This is closely related evidence but does not establish a fix for the observed mapped write faults.
- [Open sparse-merge change][metrics-sparse],
  CL `8688624`,
  remained `NEW`,
  last updated `2026-10-30`.
  It moves allocator scans outside the sparse-histogram lock and batches imports to reduce UI/IO contention.
  Again,
  distinguish lock-mediated stalls from live mapped writes.
- Historical upstream commit `2868e616e219500919fde8f95f0642267dd8c245`
  moved independent metrics loading to a background thread in `2018`,
  expressly because memory-mapped file reads caused UI jank.
  It is precedent for the principle,
  not a current fix for recording samples into the browser's active mapping.

## Tests that would make a contribution reviewable

- Positive-control reproduction:
  baseline visibly stalls at the identified boundary;
  the same fixture and event sequence continue processing UI tasks after the intended change.
  Use a disposable constrained VM/filesystem or injectable blocking backend,
  never synthetic load on the shared host.
- GPU:
  hold a fake backend read or initialization unresolved;
  verify memory hits and chosen miss policy complete without waiting;
  release the backend and verify later hits,
  deduplicated fetches,
  cancellation,
  teardown,
  corruption,
  quota exhaustion,
  memory-pressure eviction,
  and recreation after GPU death.
- Metrics:
  concurrent ordinary/sparse recording,
  importing and persistence;
  force termination at each persistence boundary;
  check no double counting,
  corrupted snapshots,
  lost system-profile associations,
  or unbounded queued memory.
  State the allowed loss window explicitly rather than silently changing crash diagnostics.
- Performance:
  alternate baseline and candidate with unchanged-input controls;
  record input-to-presentation and long-tail stalls alongside throughput,
  memory,
  cache hit rate,
  and retained diagnostics.
  No improvement percentage is established by this research.

## Ownership and contribution constraints

`components/metrics/OWNERS` includes `base/metrics/OWNERS`,
which asks contributors to prefer `chromium-metrics-reviews@google.com` for reviewer assignment.
GPU and persistent-cache changes involve `gpu/command_buffer/OWNERS`
and `components/persistent_cache/OWNERS`,
with recent relevant reviewers including Geoff Lang,
Kyle Charbonneau,
and Zhenyao Mo.
These identify future review routes;
no one was contacted.

Chromium asks for early owner discussion and a design document for critical-path changes,
with corresponding automated tests and review through Gerrit.
Source:
[contribution process][contributing].
Its AI policy permits assistance but requires the human author to understand and self-review the work;
human feedback in bugs or reviews must receive a human reply.
Source:
[AI policy][ai-policy].
This means a later contribution needs a human-maintained review commitment,
not an unattended agent posting loop.

## Search scope and uncertainty

Checked latest source,
recent path-specific GitHub commit history,
and public Gerrit queries for open changes touching the GPU-cache implementation,
persistent-histogram setup,
and histogram allocator,
then widened to `message:GpuPersistentCache` and `message:BrowserMetrics`.
Responses had no `_more_changes` marker.
Read descriptions and messages for the named open CLs.
General web searches included mapped metrics UI jank,
BrowserMetrics freezes,
and GPU-cache timeouts/prewarming.
Unrelated directory-growth reports were not treated as this freeze mechanism.

Issue pages `577796100` and `399642827` returned only a sign-in shell through the fetch tool;
their private/current discussion is not assessed.
Public issue `439845637` discusses shader-compilation and cache-startup performance,
consistent with the cited cache commits.
No claim is made that no matching issue or unpublished fix exists.
No runtime check of current-main feature rollout or new source behavior was performed.
All cited source ranges,
commit descriptions,
open-change subjects,
and policy ranges were browser-verified at `2026-11-03T09:07:44.700Z`.
The verification record is [chromium-links.json](chromium-links.json);
all 14 cited URLs passed and the named browser session closed successfully.

[metrics-init]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/chrome/app/chrome_main_delegate.cc#L1803-L1825
[gpu-read]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/gpu/command_buffer/service/gpu_persistent_cache.cc#L281-L326
[gpu-init-wait]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/gpu/command_buffer/service/gpu_persistent_cache.cc#L530-L543
[gpu-load-order]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/gpu/command_buffer/service/gpu_persistent_cache.cc#L710-L745
[cache-contract]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/components/persistent_cache/README.md?plain=1#L1-L27
[gpu-feature-defaults]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/gpu/config/gpu_finch_features.cc#L473-L490
[gpu-async-commit]: https://github.com/chromium/chromium/commit/db4fa7da45bbffd5a2fcc2b4eb6ca267e41d0522
[gpu-preload-commit]: https://github.com/chromium/chromium/commit/f2fc41911596344ad1399b9035f375f1413514e5
[gpu-timeout-commit]: https://github.com/chromium/chromium/commit/076b8b99b9729d4266e43df22fc96d06eb34421a
[gpu-dse-wip]: https://chromium-review.googlesource.com/c/chromium/src/+/8412185
[metrics-segments]: https://chromium-review.googlesource.com/c/chromium/src/+/8707393
[metrics-sparse]: https://chromium-review.googlesource.com/c/chromium/src/+/8688624
[contributing]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/docs/contributing.md?plain=1#L20-L43
[ai-policy]: https://github.com/chromium/chromium/blob/e436fd72f52814c1228fb16a8c8f7cae8253fb99/agents/ai_policy.md?plain=1#L6-L33
