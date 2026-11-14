# Chromium 156.0.8078.73 synchronous disk dependencies can delay UI and rendering

## Audit status at 2026-11-03T03:43:41-05:00

The audit found direct browser-UI file operations,
renderer waits for storage services,
synchronous graphics-cache reads,
and file-backed memory accesses that can fault on latency-sensitive threads.
An asynchronous writer does not make a synchronous reader or waiting caller nonblocking.

The baseline is Chromium `156.0.8078.73`,
commit `2f9a11097447f226133bc1a31c787291a5eeab98`,
matching the installed version recorded in
[`helium-window-freezes.md`](helium-window-freezes.md).
The tag was verified through GitHub's ref API and the checkout through `git rev-parse HEAD`.
This is a source audit,
not a fresh runtime responsiveness comparison.
Only the previously captured Helium `BrowserMetrics` write fault has incident-level attribution here.

The supporting evidence is split by the affected execution path:

- [Desktop UI and profile paths](chromium-disk-responsiveness/desktop-ui.md).
- [Mapped metrics, resources, fonts, and dictionaries](chromium-disk-responsiveness/mapped-files.md).
- [Renderer storage, mobile UI, and lifecycle paths](chromium-disk-responsiveness/mobile-renderer-lifecycle.md).
- [Runtime dependencies and their Chromium callers](chromium-disk-responsiveness/dependencies.md).
- [Machine-readable scan results](chromium-disk-responsiveness/scan-summary.json)
  and candidate inventory (`chromium-disk-responsiveness/candidates.jsonl.gz`).
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

The complete-tree searches are finished;
the inventory is not an exhaustive proof of every reachable operation in every Chromium dependency.

## Symptom boundaries

Blocking the browser UI thread can stop browser controls and window interaction.
Blocking a renderer main thread can stop that page's JavaScript,
layout,
and main-thread painting while browser controls remain usable.
Blocking graphics work can delay presentation without proving that the browser UI event loop itself stopped.
Startup and teardown waits affect readiness or exit,
and must not be relabeled as arbitrary browsing-time freezes.

All new flags describe possible blocking paths.
A source call can be satisfied from memory caches;
its presence does not measure how often physical storage is reached or how long it takes.
Feature defaults in this report are source defaults,
not measurements of the current browser's experiment configuration.

## Findings to prioritize for desktop Linux

### Diagnostic histogram writes touch a writable file mapping

The [mapped-file evidence](chromium-disk-responsiveness/mapped-files.md#browsermetrics-writes-can-fault-on-the-browser-ui-thread)
traces browser startup through `InstantiatePersistentHistograms`,
`GlobalHistogramAllocator::CreateWithFile`,
and a shared writable mapping.
Histogram sample recording writes directly into that storage on the recording thread.

The prior [incident capture](helium-window-freezes.md#metrics-write-faults-block-the-main-thread)
matched a Helium main-thread write fault to its `BrowserMetrics/*.pma` mapping
and Btrfs writeback or transaction waits.
This mechanism needs no explicit `fsync` at the metric-recording call site.
The existing local-memory workaround removes this specific mapping,
not every storage dependency.

### GPU persistent-cache misses synchronously read SQLite-backed files

This is a rendering-path risk rather than direct proof of browser-UI-thread I/O.
[`SharedContextState::InitializeSkia`][gpu-install]
installs the cache into Skia and the current GL context:

```cpp
// gpu/command_buffer/service/shared_context_state.cc:529-532,564
options.fPersistentCache = persistent_cache_.get();
BindCacheToCurrentOpenGLContext(persistent_cache_.get());
```

[`GpuPersistentCache::LoadImpl`][gpu-load]
returns memory hits immediately,
but a miss with an initialized disk cache calls its disk loader inline:

```cpp
// gpu/command_buffer/service/gpu_persistent_cache.cc:710-711
CacheLoadResult disk_load_result =
    disk_cache_->Load(key, wrapped_buffer_provider);
```

[`DiskCache::Load`][gpu-find]
first waits for other cache use,
then calls the persistent backend synchronously:

```cpp
// gpu/command_buffer/service/gpu_persistent_cache.cc:284-299
if (!TimedWait(cache_in_use_cond_var_, kDiskOpWaitTimeoutMs,
               [this]() { return cache_in_use_; })) {
  return CacheLoadResult::kMissTimeout;
}
// After leaving the synchronization scope:
result = cache_->Find(base::as_byte_span(key), buffer_provider);
```

The [constant is `20 ms`][gpu-timeout],
despite a nearby comment saying `10 ms`.
That timeout bounds waiting for another operation,
not `Find()` or the underlying storage read.
[`SqliteBackendImpl::FindImpl`][gpu-sql]
steps a SQL query and reads the selected blob.
The [SQLite VFS][gpu-vfs] performs the file read:

```cpp
// components/sqlite_vfs/sandboxed_file.cc:118
std::optional<size_t> bytes_read = opened_file_.Read(offset, data);
```

[`kGpuPersistentCache`][gpu-feature] is enabled by default outside Windows in this revision.
Graphite has a separate cache feature with a different platform default;
do not infer the running Graphite configuration from the Ganesh feature.
[Production disk writes use a worker][gpu-writes],
so the finding is the synchronous read and its serialization dependency,
not a claim that every graphics-cache write runs on the rendering caller.
The generic cache also supports synchronous writes when no worker is supplied;
that branch alone does not establish the production configuration.

### UI resources can incur file-page faults while constructing menus or images

[Resource-pack tracing](chromium-disk-responsiveness/mapped-files.md#ui-resource-packs-expose-mapped-bytes-during-synchronous-string-and-image-lookup)
connects browser menu construction to localized-string lookup and direct views into uncompressed `.pak` mappings.
Image decoding also consumes pack bytes synchronously on cache misses.
Opening or mapping a file ahead of time does not prove that all its pages remain resident.
Compressed packs that were expanded into owned memory are a distinct branch.

### Font and spelling cache misses retain synchronous file dependencies

[Browser font creation](chromium-disk-responsiveness/dependencies.md#browser-ui-fonts-use-synchronous-skia-file-access-under-a-shared-manager-lock)
calls Skia's Linux font manager synchronously.
On a cache miss,
that manager holds its mutex while opening and mapping a font file.
The browser's font service also uses that manager,
so worker-side loading can create a lock dependency for a concurrent UI font miss.
This is a possible contention path,
not a measured lock wait.

[Font lookup](chromium-disk-responsiveness/mapped-files.md#font-misses-synchronously-await-disk-work-then-retain-file-mappings-for-rendering)
uses a synchronous service call from layout on a cache miss.
The service opens the font on a worker,
but the renderer waits for its response and can subsequently read a mapped font file.
The legacy font loader also explicitly waits for its worker.

[Hunspell spelling](chromium-disk-responsiveness/mapped-files.md#hunspell-dictionaries-remain-mapped-on-synchronous-editing-paths)
maps `.bdic` files and retains their bytes for synchronous checks and suggestions.
These are page responsiveness risks during layout or editing;
they are not new measurements of a whole-window freeze.

[PDF substitute fonts](chromium-disk-responsiveness/dependencies.md#pdf-substitute-font-reads-run-synchronously-in-the-renderer-callback)
also use synchronous `fstat` and `pread` in Chromium's Linux font callback.
The normal PDF document-byte path instead uses the custom document loader;
upstream PDFium file helpers alone do not establish disk reads during normal document rendering.

### Parked strings and images are synchronously read back from disk

[`ParkableStringImpl::ToString`][park-string-access]
unparks on demand.
[`UnparkInternal`][park-string-read]
reads the stored compressed bytes before decompressing them:

```cpp
// third_party/blink/renderer/platform/bindings/parkable_string.cc:677-685
if (is_on_disk_no_lock()) {
  // Allocation precedes this call.
  manager.data_allocator().Read(*metadata_->on_disk_metadata_,
                                *metadata_->compressed_);
}
```

[`DiskDataAllocator::DoRead`][park-disk-read]
explicitly describes the main-thread tradeoff and performs the file read inline:

```cpp
// third_party/blink/renderer/platform/disk_data_allocator.cc:205-214
// This happens on the main thread, which is typically not allowed.
base::ScopedAllowBlocking allow_blocking;
std::optional<size_t> read = file_.Read(offset, data);
```

The comment justifies the design as preferable to swapping arbitrary memory under pressure.
That is an upstream rationale,
not a measured guarantee that this read is rare or bounded on the user's filesystem.
[`kCompressParkableStrings`][park-feature]
is enabled by default on Linux and Android and disabled by default on macOS and Windows.
Actual disk unparking additionally requires that the data was parked to a usable temporary file.

[`ParkableImageImpl::Unpark`][park-image-read]
likewise builds its buffer using an inline callback to the disk allocator.
Its consumer can be an image-decoding or rendering thread;
this audit does not classify every image unpark as renderer-main-thread work.
In both cases,
background writes do not remove synchronous reads when the data is needed again.

### Cold localStorage reads block the renderer on its storage service

The [full call chain](chromium-disk-responsiveness/mobile-renderer-lifecycle.md#renderer-cold-localstorage-access-synchronously-waits-for-database-reads)
is `StorageArea::getItem` to `CachedStorageArea::EnsureLoaded`,
a synchronous Mojo `GetAll`,
then service-side map loading from the selected LevelDB or SQLite backend.
The service can also commit a pending keys-only cache before reloading.
The renderer waits even though the database work is dispatched to another sequence.
Warm renderer-cache reads and in-memory stores do not establish disk access.

### File pickers and profile loading perform direct UI filesystem operations

[Linux picker setup](chromium-disk-responsiveness/desktop-ui.md#linux-file-pickers-synchronously-inspect-filesystem-metadata)
calls `DirectoryExists` through an explicitly UI-thread helper.
The GTK preview callback also executes `stat` before posting content reads to a worker.
Slow local metadata or a remembered network-mounted directory can therefore hold up that UI path.

[Profile switching](chromium-disk-responsiveness/desktop-ui.md#switching-to-an-unloaded-profile-still-does-synchronous-filesystem-work)
reaches `CreateProfileDirectory` synchronously even through `CreateProfileAsync`.
The operations include `PathExists`,
`CreateDirectory`,
and `GetFileInfo`.
Existing directories still require metadata access.

[Theme loading and migration](chromium-disk-responsiveness/desktop-ui.md#theme-packs-and-theme-migration-read-files-on-the-ui-thread)
read pack or image files on UI while initializing a themed profile.
[Extension manifest reload](chromium-disk-responsiveness/desktop-ui.md#extension-manifest-reload-during-profile-startup-reads-on-ui)
does synchronous reads for selected cases such as unpacked extensions or relocalization.
Ordinary theme changes use a worker and are not the same path.

### Enterprise scan teardown can wait on background file reads

The [enterprise evidence](chromium-disk-responsiveness/desktop-ui.md#enterprise-file-scan-cancellation-waits-for-file-reading-workers)
traces handler destruction to `JobHandle::Cancel()`
and workers opening or hashing files.
The code explicitly says cancellation can block the UI until the workers return.
An atomic cancellation flag cannot interrupt a filesystem read already in progress.
Enterprise analysis must be active,
and experiment flags affect the precise teardown branch.
This is not an unconditional consumer-browser finding.

## Other platform and lifecycle flags

The linked evidence files retain the exact callers,
source excerpts,
and feature gates for each flag:

- macOS drag-out synchronously creates destination files and changes permissions.
  Native file-panel setup also checks directory metadata in its UI process.
- Windows drag-out synchronously creates a temporary directory.
  Profile crash acknowledgment synchronously deletes its marker file;
  profile lifecycle tracking has additional file operations.
- Android active-tab restoration can wait for unfinished prefetch or read state on UI,
  in both the legacy and newer store generations.
- Android legacy tab saves synchronously write remaining state during selected lifecycle calls.
  The newer store's ordinary save method queues work instead.
- Android closed-window history menus can synchronously read uncached metadata.
  Legacy initial tab-ID repair enumerates persisted directories on UI when its preference is absent.
- Android WebView startup opens,
  locks,
  and writes its data-directory lock file on UI.
  Its bounded retry sleeps do not bound filesystem-operation latency.
- Android WebView synchronous cookie flush waits for storage completion on the API caller.
  It blocks UI when the embedding application calls it there.
- iOS session loading and lazy tab realization synchronously read session files.
  Profile construction also checks and creates directories on the main thread.
- Clean-exit beacons synchronously read or write diagnostic state during startup,
  shutdown,
  and mobile foreground/background transitions.
- Configured NetLog or TLS key logging synchronously opens files when the network service is created.
- External ICU data and V8 snapshot loading have startup file opens and mapped-data dependencies.
- With renderer-side audio processing,
  stopping an enabled WebRTC echo-cancellation dump waits on the renderer main thread
  for queued file writes to finish.
  This is a diagnostic-feature teardown path.
- ANGLE and Dawn synchronously load native libraries in selected initialization branches.
  These can delay graphics readiness;
  a browser-main-thread caller was not established for those loader branches.
- Browser teardown waits for outstanding `BLOCK_SHUTDOWN` work,
  including atomic bookmark writes with a flush.
  The end-session preference-write rundown has a separate timed wait.

See [desktop evidence](chromium-disk-responsiveness/desktop-ui.md),
[mobile and lifecycle evidence](chromium-disk-responsiveness/mobile-renderer-lifecycle.md),
and [external data evidence](chromium-disk-responsiveness/mapped-files.md#external-v8-snapshots-and-icu-data-retain-startup-and-cold-page-dependencies).
The [dependency evidence](chromium-disk-responsiveness/dependencies.md)
also traces the echo-cancellation dump and native-library loaders.

## Verification and coverage

The Chromium checkout has `505,367` tracked entries:
`505,094` file entries and `273` separately versioned dependency gitlinks.
The primary search processed `505,073` filesystem files and `3,041,578,519` bytes.
Its `84,855` matched lines occur in `19,920` files.
The remaining `21` tracked file entries are symlinks,
enumerated in [the symlink inventory](chromium-disk-responsiveness/symlinks.json).
No untracked files entered the primary scan.
Git internals were excluded;
hidden and ignored source files were included.
Ripgrep's binary-file detection remained active;
this is a text-source search rather than an exhaustive search of binary assets.

The [search patterns](chromium-disk-responsiveness/patterns.txt)
cover explicit blocking allowances,
POSIX and Chromium file APIs,
managed-language file access,
memory maps,
database operations,
synchronous waits,
and thread/disk comments.
The candidate inventory preserves path,
line,
matched categories,
and source text.
Its `production-looking` label is a path/extension heuristic,
not proof of reachability or UI affinity.
`21,154` lines have that label.
Tests,
tools,
comments,
and declarations remain in the full inventory for inspection.

A supplementary pass (`chromium-disk-responsiveness/supplemental-raw.jsonl.gz`)
adds `base::File` construction,
object `Read`/`Write`/`Flush` calls,
and managed metadata or commit calls.
It processed the same `505,073` files,
with dependency gitlinks explicitly excluded,
and matched `58,646` lines.
These passes overlap;
their counts must not be summed as distinct operations or defects.
The [supplementary summary](chromium-disk-responsiveness/supplemental-summary.json)
and [patterns](chromium-disk-responsiveness/supplemental-patterns.txt) are retained.
`supplemental-raw.jsonl.gz` left the tree on 2026-11-13.
`doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

A native-library-loader pass (`chromium-disk-responsiveness/loader-raw.jsonl.gz`)
searched the same core files and found `371` matching lines in `226` files.
Its [patterns](chromium-disk-responsiveness/loader-patterns.txt)
and [summary](chromium-disk-responsiveness/loader-summary.json) are retained separately.
The [inventory verification](chromium-disk-responsiveness/scan-verification.json)
recounts the compressed primary artifact,
checks its distinct-file total,
and confirms named source positive controls plus a non-matching declaration control.
`loader-raw.jsonl.gz` left the tree on 2026-11-13.
`doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

The first primary-scan attempt rejected ripgrep's base64 representation for a non-UTF-8 text line.
That parser was corrected and the complete scan rerun successfully.
Only the completed run supplies the counts in this report.
The scanner is lexical,
not a compiler-based whole-program analysis.
It can miss wrappers,
macro-generated operations,
platform-library internals,
and dependencies not checked out.
It also cannot detect arbitrary executable-page or swapped-anonymous-memory faults from a file-API search.

### Dependency coverage

The pinned V8,
Skia,
ANGLE,
Dawn,
PDFium,
ICU,
LevelDB,
SQLite,
and WebRTC repositories were also acquired and scanned.
Together they contribute `144,427` regular tracked files,
`1,229,036,005` searched bytes,
and `17,752` matching lines after adding named dependency file helpers.
The original core patterns match `17,166` of those lines.
A separate library-loader pass found `191` matching lines;
it overlaps the main dependency scan.

The other `264` top-level gitlinks were not populated by this audit.
The selected dependencies also contain `148` nested gitlinks and a symlink that were inventoried,
not traversed.
Build-time downloads,
generated output,
private Chrome code,
OS implementations,
and proprietary graphics or printer drivers are outside the inspected source corpus.
The [top-level gitlink manifest](chromium-disk-responsiveness/gitlinks.json)
retains their exact paths and revisions.
The [dependency manifest](chromium-disk-responsiveness/dependencies/summary.json)
and combined dependency candidates (`chromium-disk-responsiveness/dependencies/dependencies-candidates.jsonl.gz`)
retain the selected repositories' results.
The core and selected dependency scans together cover `649,500` files.
This scope is broader than a Chromium-directory keyword search,
but it is not a complete audit of the transitive dependency universe.
`dependencies-candidates.jsonl.gz` left the tree on 2026-11-13.
`doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

### Browser-validation resource limit

The first disposable citation browser ran with `TasksMax=128`.
Its cgroup recorded `pids.events` value `max 100`,
while its memory limit and OOM counters remained zero.
`agent-browser 0.41.0` reported
`Operation timed out. The page may still be loading or the element may not exist.`
and a later probe reported
`Failed to read: Resource temporarily unavailable (os error 11)`.
No failed navigation was counted as a verified citation.

Closing that owned session and relaunching the same browser with `TasksMax=512`
loaded the same GitHub source selection successfully.
A second validation session measured `176` live tasks with `pids.events max 0`
after the corrected launch.
The source scans retained their own lower task bounds.
This is evidence that the validation task budget was insufficient;
it is not evidence that each browser error had the same immediate cause.

No Chromium build,
device latency test,
feature-toggle experiment,
or new browser stall reproduction was performed.
No runtime flags or profile files were changed for this audit.
The delegated source passes used the same model vendor and are not independent reviews.

### Reproducing the lexical search

The primary scan used the invocation in this excerpt from a fresh top-level checkout,
before separately versioned dependencies were populated.
Its pattern file is committed with this report.
Populated dependency directories must be excluded to reproduce the core-only counts;
the supplemental scans used exclusions generated from `gitlinks.json`.

```sh
# From the pinned Chromium checkout, with AUDIT_DIR naming this report's companion directory.
systemd-run --user --scope \
  --property=CPUQuota=200% --property=MemoryMax=6G \
  --property=TasksMax=128 --property=IOWeight=10 \
  rg --json --hidden --no-ignore --glob '!.git/**' \
  --file "${AUDIT_DIR}/patterns.txt" .
```

The JSON records expose path,
line,
and matched text.
The committed compressed inventory adds categories and a filename-based triage label.
Rendering is reproducible with `mise run verify:markdown`
from `doc/troubleshooting/chromium-disk-responsiveness`.

All `131` external source-selection URLs in the overview and evidence documents
were opened in disposable browser sessions.
The selected anchors and source excerpts were compared with the pinned checkouts.
The [validation summary](chromium-disk-responsiveness/validation-summary.json)
checks that every cited URL has a successful verification record.
Full records retain failed attempts separately;
browser DOM blank-line and trailing-whitespace differences were normalized for comparison.

## What does not establish a UI disk stall

- `ScopedAllowBlocking` and Android StrictMode allowances change checks;
  they do not dispatch work to another thread.
  Conversely,
  an allowance alone does not prove that a disk operation occurs.
- `MayBlock` worker work is not UI-thread blocking unless a synchronous caller,
  shared lock,
  or required result connects it to the UI path.
- A `GPUCache` mapping in a browser process does not identify a UI-thread fault.
  Legacy blockfile operations normally run on a dedicated worker;
  its destructor's wait remains a separate lifetime-specific candidate.
- `/proc` reads,
  anonymous shared memory,
  Wayland keymaps,
  and DMA mappings are not regular disk files merely because file APIs or `mmap` appear.
- Windows printing wrappers expose blocking driver calls,
  but this audit has not distinguished storage from network or other driver waits.
- ChromeOS diagnostic WebUI handlers with direct filesystem calls remain separately labeled candidates
  where full thread-affinity tracing was not completed.
- A source comment describing an operation as quick,
  rare,
  or harmless is not a latency measurement.

## Comparison with Firefox ESR at 2026-11-03T03:50:48-05:00

The [Firefox ESR audit](../audit/firefox-esr-disk-responsiveness.md)
does not establish that Firefox is significantly more resistant to disk-induced UI stalls overall.
Both audits identify possible blocking paths rather than comparable stall rates or latency distributions.
Their match counts and finding counts are not a responsiveness score:
the patterns,
dependency layouts,
and manually traced surfaces differ.

Firefox has a concrete design advantage in the checked ordinary Glean counter path:
updates are dispatched to a worker instead of writing Chromium's shared disk-backed histogram mapping
on the recording thread.
That distinction matters because the prior Helium capture attributed a main-thread stall to this mapping.
The mapping workaround already removes that particular path from the recorded modified Helium setup,
so it does not establish an ongoing advantage against that modified setup.

The broader Firefox audit supplies substantial counterevidence to a general isolation claim:

- [Startup-cache readers can wait on a mutex held across background file writes](../audit/firefox-esr-disk-responsiveness.md#main-thread-startup-cache-lock-held-by-disk-writer).
- [Glean memory reporting can wait on the global mutex during persistence](../audit/firefox-esr-disk-responsiveness.md#glean-memory-reports-can-wait-for-telemetry-database-persistence).
  Ordinary queued metric recording and main-thread reporting are different paths.
- [Certificate-state task submission can acquire a lock held by a disk-working task](../audit/firefox-esr-disk-responsiveness.md#certificate-state-async-apis-share-a-lock-held-across-disk-work).
- [Classic History/Places interactions execute synchronous SQLite queries](../audit/firefox-esr-disk-responsiveness.md#places-history-interaction-executes-sqlite-statements-on-the-parent-main-thread).
  Downloads,
  file pickers,
  extension installation,
  and other UI actions also have direct filesystem operations.

Both browsers have synchronous shader-cache reads on rendering paths,
font-loading and mapped-resource dependencies,
and cold localStorage waits on content main threads.
These source similarities do not prove equal frequency or severity.

The supported conclusion is a specific Firefox advantage against the observed unmodified metrics path,
with no demonstrated overall winner or effect size.
The user's reported smoother Firefox experience is relevant to that user's workload,
but the source audits alone do not explain all of it or establish a general browser ranking.
This comparison concerns disk-related responsiveness,
not browser quality across unrelated criteria.

## Verified workarounds

No new workaround was implemented or tested by this audit.
The previously verified metrics-mapping workaround and its persistence tradeoff are documented in
[`helium-window-freezes.md`](helium-window-freezes.md#verified-workarounds).
This report does not recommend disabling caches,
durability,
or browser features based solely on the static flags.

## Upstream filing decision

The user requested an audit,
not fixes or an upstream report.
No source patch,
issue,
or external comment was produced.

- Responsibility is established for the listed source-level synchronous dependencies;
  newly measured regressions are not established.
- Moving work or changing persistence semantics may be feasible,
  but this audit does not contain implementation or correctness proofs.
- Supported-use scope varies by platform,
  feature,
  profile state,
  and embedding API caller.
- Contribution-policy and trust-system checks for a new filing were not run.
- Duplicate,
  in-progress-fix,
  and maintainer-intent searches for every candidate were not run.
- No upstream fix prototype was authorized or tested.

The correct artifact is this local audit and its evidence inventory.
There is no recommendation to file a new issue from an unmeasured candidate.

[gpu-install]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/gpu/command_buffer/service/shared_context_state.cc#L529-L564
[gpu-load]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/gpu/command_buffer/service/gpu_persistent_cache.cc#L648-L715
[gpu-find]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/gpu/command_buffer/service/gpu_persistent_cache.cc#L268-L300
[gpu-timeout]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/gpu/command_buffer/service/gpu_persistent_cache.cc#L43-L45
[gpu-sql]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/persistent_cache/sqlite/sqlite_backend_impl.cc#L254-L307
[gpu-vfs]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/sqlite_vfs/sandboxed_file.cc#L105-L120
[gpu-feature]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/gpu/config/gpu_finch_features.cc#L441-L488
[gpu-writes]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/viz/service/gl/gpu_service_impl.cc#L163-L177
[park-string-access]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/platform/bindings/parkable_string.cc#L402-L410
[park-string-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/platform/bindings/parkable_string.cc#L670-L690
[park-disk-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/platform/disk_data_allocator.cc#L205-L214
[park-feature]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/common/features.cc#L355-L377
[park-image-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/platform/graphics/parkable_image.cc#L438-L476
