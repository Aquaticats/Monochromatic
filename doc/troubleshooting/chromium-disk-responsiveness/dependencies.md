# Chromium runtime dependency disk audit at 2026-11-03T08:40:07+00:00

This extends the source audit of Chromium `156.0.8078.73`,
commit `2f9a11097447f226133bc1a31c787291a5eeab98`.
Findings are static responsiveness risks,
not measured latency regressions.
The source came from the dependency URLs recorded in Chromium's `.gitmodules`,
at the exact gitlink revisions.
GitHub mirrors are navigation copies;
source content is compared with those pinned local checkouts.

## Coverage

The non-recursive native `git submodule update --init --depth 1 --jobs 3 -- <explicit paths>` succeeded.
All selected dependencies match their gitlink commits.
The scan submitted every regular tracked file explicitly to `rg`,
including ignored and hidden tracked files.
It excluded `148` nested gitlinks and one symlink;
their paths and revisions are in each inventory.
No nested test/tool dependency checkout was requested.

Across `144,427` regular tracked files,
the scan retained `17,752` matching lines in `2,572` files.
Of those lines,
`17,166` matched the core audit's original patterns.
Additional wrapper patterns cover Skia,
LevelDB,
and ICU file helpers.
A separate native-library-loading pass found `191` matching lines in `87` files.
After checking for shared locations,
the combined artifact contains
`17,943` distinct matching lines across `2,634` files.

- V8:
  `19,826` regular files,
  `1,778` matches,
  revision `235050a0df6db2c9d92a419af2b72fbf3b2dd73b`.
- Skia:
  `12,431` regular files,
  `5,513` matches,
  revision `a65b5cc03a5131808aec172085b7074fc66625b9`.
- ANGLE:
  `4,037` regular files,
  `658` matches,
  revision `127e783de33425c27a70a3f48da9d9202d6a0745`.
- Dawn:
  `85,594` regular files,
  `1,823` matches,
  revision `0caf3649477f7c3ffbd4b1ca4764a81b469b1582`.
- PDFium:
  `5,440` regular files,
  `208` matches,
  revision `fa4d5f8fbd89ea07273529daf9fd24fe1c7f6b9a`.
- ICU:
  `7,119` regular files,
  `947` matches,
  revision `2a51536dfb3b448bdf3dae70c782ea2879c90904`.
- LevelDB:
  `152` regular files,
  `335` matches,
  revision `7ee830d02b623e8ffe0b95d59a74db1e58da04c5`.
- SQLite:
  `2,218` regular files,
  `5,080` matches,
  revision `ba8c54c83337618a9f15dd368a267e46b672fc84`.
- WebRTC:
  `7,610` regular files,
  `1,410` matches,
  revision `19576e37870bfe5a0a910a32f98ff411f722f43a`.

Counts include declarations,
comments,
tests,
utilities,
and false positives such as geometric `close()` calls.
The `production-looking` field is a filename heuristic,
not a validated production call graph.
The complete matches,
patterns,
inventories,
and summaries are retained in
[the committed dependency evidence](dependencies/summary.json).
The loader pass is in that directory's `loaders/` child.
The scanner is `${HOME}/temp/agent/chromium-dependency-disk-scan.ts`.
The combined artifact is `dependencies-candidates.jsonl.gz`,
measuring `298,922` bytes;
`dependency-audit-summary.json` retains per-dependency totals.
`known-source-controls.json` confirms that the scanner retained the inspected Skia/PDFium mappings,
WebRTC file writer,
and ANGLE/Dawn loader calls.
`dependencies-candidates.jsonl.gz` left the tree on 2026-11-13.
`doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

## Browser UI fonts use synchronous Skia file access under a shared manager lock

This extends the mapped-font finding beyond the renderer's synchronous font-service request.
Chromium's [`CreateSkTypeface` at `ui/gfx/platform_font_skia.cc:59-84`][ui-font]
calls `skia::MakeTypefaceFromName` directly:

```cpp
// ui/gfx/platform_font_skia.cc
typeface = skia::MakeTypefaceFromName(family->c_str(), sk_style);
```

The [Linux default font manager at `skia/ext/font_utils.cc:91-130`][font-manager]
uses `SkFontMgr_New_FCI` and invokes its `legacyMakeTypeface` synchronously.
Skia's [`SkFontMgr_FCI::onMatchFamilyStyle` at `src/ports/SkFontMgr_FontConfigInterface.cpp:178-205`][fci-match]
holds the manager mutex across matching and file opening:

```cpp
// third_party/skia/src/ports/SkFontMgr_FontConfigInterface.cpp
SkAutoMutexExclusive ama(fMutex);
// On a cache miss:
sk_sp<SkTypeface> realTypeface = fScanner->MakeFromStream(
        std::unique_ptr<SkStreamAsset>(fFCI->openStream(identity)),
        SkFontArguments().setCollectionIndex(identity.fTTCIndex));
```

The [direct FCI opener at `src/ports/SkFontConfigInterface_direct.cpp:750-751`][fci-open]
calls `SkStream::MakeFromFile`.
[`src/core/SkStream.cpp:913-935`][skia-stream] opens the file and tries a memory mapping,
then falls back to `SkFILEStream`:

```cpp
// third_party/skia/src/core/SkStream.cpp
FILE* file = sk_fopen(path, kRead_SkFILE_Flag);
auto data = SkData::MakeFromFILE(file);
// If mapping fails:
auto stream = std::make_unique<SkFILEStream>(path);
```

The browser font service uses the same `skia::DefaultFontMgr()` at
`components/services/font_data/font_data_service_impl.cc:105`.
Therefore a service worker opening a new font can also hold the manager mutex needed by a UI font miss.
This is a source-supported contention possibility,
not a measured lock wait.
Caches and resident pages can avoid disk work.
The source search for manager overrides found child-process overrides and a Windows override,
not a Linux browser override of this default path.

## PDF substitute-font reads run synchronously in the renderer callback

Chromium's [`pdf/pdfium/pdfium_font_linux.cc:133-184`][pdf-font-callback]
gets the substitute font through `MatchFontWithFallback`,
then uses `GetFontTable` from `GetFontData`:

```cpp
// pdf/pdfium/pdfium_font_linux.cc
font_loader->MatchFontWithFallback(
    desc.family.Utf8(),
    desc.weight >= blink::WebFontDescription::kWeightBold, desc.italic,
    charset, desc.generic_family, font_file.get());
if (!GetFontTable(platform_file, table_tag, buffer, &size)) {
```

[`GetFontTable` at `pdf/pdfium/pdfium_font_linux.cc:51-106`][pdf-font-read]
does `fstat` and `pread` directly:

```cpp
// pdf/pdfium/pdfium_font_linux.cc
ssize_t n = HANDLE_EINTR(pread(fd, output, data_length, data_offset));
```

Trigger:
rendering or measuring PDF text that needs a substitute system font.
`pdf/pdf_view_web_plugin.cc:210` initializes the SDK with `FontMappingMode::kBlink`;
`pdf/pdfium/pdfium_engine.cc:1015` installs this Linux mapper.
This can delay the PDF renderer;
it does not establish a whole-browser UI stall.

The document-byte path is a useful exclusion.
[`pdf/pdfium/pdfium_document.cc:109-117`][pdf-document] uses `FPDF_LoadCustomDocument`;
[`pdf/loader/document_loader_impl.cc:203-211`][pdf-chunks] reads `chunk_stream_`:

```cpp
// pdf/loader/document_loader_impl.cc
return chunk_stream_.ReadData(
    gfx::Range(position, addition_result.ValueOrDie()), buf);
```

Consequently,
PDFium's standalone file-opening and `fsync` helpers alone do not prove that Chromium's
normal PDF document reads block on disk.
Font callbacks provide the concrete synchronous path here.

## Stopping an enabled AEC dump waits for file writes on renderer main

This path requires renderer-side audio processing and enabled diagnostic AEC dumping.
`third_party/blink/renderer/modules/mediastream/processed_local_audio_source.cc:243-276`
has distinct audio-service and renderer-side processing branches.
It is not a claim about every microphone session or calls with AEC dumping disabled.
[`MediaStreamAudioProcessor::Stop` and `OnStopDump`][aec-main]
at `third_party/blink/renderer/modules/mediastream/media_stream_audio_processor.cc:91-114`
assert the main-thread runner and call `audio_processor_->OnStopDump()`.
`media/webrtc/audio_processor.cc:410-418` calls `media::StopEchoCancellationDump`,
which invokes `DetachAecDump` at `media/webrtc/helpers.cc:141-142`.

WebRTC explicitly [documents the blocking destructor in `modules/audio_processing/audio_processing_impl.cc:1791-1800`][aec-detach].
[`AecDumpImpl::~AecDumpImpl` at `modules/audio_processing/aec_dump/aec_dump_impl.cc:78-84`][aec-wait]
queues a marker behind pending writes and waits without a timeout:

```cpp
// third_party/webrtc/modules/audio_processing/aec_dump/aec_dump_impl.cc
worker_queue_->PostTask([&thread_sync_event] { thread_sync_event.Set(); });
thread_sync_event.Wait(Event::kForever);
```

The [queued writes at `modules/audio_processing/aec_dump/aec_dump_impl.cc:265-286`][aec-write]
call `debug_file_.Write`;
[`rtc_base/system/file_wrapper.cc:115-124`][webrtc-file] uses `fwrite` and closes with `fclose`.
The stream teardown waits for the worker's earlier file work,
and closing the dump can flush buffered bytes.
No AEC recording was enabled or stopped during this audit.

## V8 and ICU preserve the mapped-data dependencies

The V8 pin was inspected,
resolving the initial mapped-source subtask's dependency limitation.
[`src/snapshot/snapshot.cc:204-218`][v8-context] reads the isolate snapshot blob,
extracts the context slice,
and deserializes synchronously.
[`src/snapshot/snapshot.cc:716-733`][v8-slice] points that slice into the original blob:

```cpp
// v8/src/snapshot/snapshot.cc
const uint8_t* context_data =
    reinterpret_cast<const uint8_t*>(data->data + context_offset);
return base::Vector<const uint8_t>(context_data, context_length);
```

The context creation path therefore consumes Chromium's externally mapped snapshot bytes.
Snapshot compression changes the subsequent representation but still reads the input bytes to decompress them.
This is a new-context/startup exposure,
not a claim that ordinary execution continuously reads the snapshot file.

ICU's [`source/common/udata.cpp:878-898`][icu-common]
passes the supplied common-data pointer to `UDataMemory_setData`.
[`source/common/udatamem.cpp:86-88`][icu-pointer] retains it as `pHeader`:

```cpp
// third_party/icu/source/common/udatamem.cpp
This->pHeader = UDataMemory_normalizeDataPointer(dataAddr);
```

The source confirms that handing ICU a mapped `icudtl.dat` pointer does not copy the entire data file into
independent heap storage.
The integration and build-mode limitations are in the [mapped-file report](mapped-files.md).

## Native-library loading is an additional conditional GPU dependency

The first matcher did not cover dynamic-loader names.
The supplemental pass covers `LoadLibrary*`,
`dlopen`,
`dlmopen`,
and Chromium library-loading helpers.
It is separate because loader waits must not be counted as an established UI-thread disk call without tracing callers.

ANGLE's [`src/libANGLE/renderer/d3d/HLSLCompiler.cpp:116-143`][angle-loader]
loads the compiler DLL during first initialization if no preloaded module is found:

```cpp
// third_party/angle/src/libANGLE/renderer/d3d/HLSLCompiler.cpp
mD3DCompilerModule = LoadLibraryA(D3DCOMPILER_DLL_A);
```

Dawn's [`src/dawn/native/vulkan/BackendVk.cpp:355-379`][dawn-loader]
loads the selected Vulkan library while initializing the instance.
Its [`src/dawn/common/DynamicLib.cpp:83-104`][dawn-dlopen]
performs synchronous `LoadLibraryExA` or `dlopen(..., RTLD_NOW)`:

```cpp
// third_party/dawn/src/dawn/common/DynamicLib.cpp
mHandle = dlopen(filename.c_str(), RTLD_NOW);
```

These can delay GPU/backend readiness or the operation awaiting initialization.
They are not validated browser-main-thread call chains here.
Preloaded libraries and warm executable pages reduce this exposure.
Optional shader graph dumps and command-line tooling were not treated as default browsing I/O.

## Database and logging false-positive boundaries

LevelDB's upstream default `Env` has mmap and synchronous file operations,
but Chromium supplies its own environment.
`third_party/leveldatabase/env_chromium.cc:967-985` returns Chromium random-access file wrappers,
while `ChromiumWritableFile::Sync` at `env_chromium.cc:375-401` invokes `file_.Flush()`.
Responsiveness therefore depends on the Chromium caller's thread and whether another thread waits on it;
the upstream `Env` implementation alone does not identify the affected UI surface.

SQLite contains direct file operations and duplicates them in generated amalgamations.
`sql/statement.cc:133` enters `sqlite3_step` on its caller,
and `sql/database.cc:2446-2456` controls mapping size per database.
The [overview's storage and persistent-cache call-chain analysis](../chromium-disk-responsiveness.md) determines which synchronous database uses matter.
The scan does not classify every SQLite call as a UI stall.

WebRTC's regular dump writes are queued.
The AEC finding is specifically the renderer-main teardown wait,
not an assertion that the writer task itself runs on renderer main.
The separate `RtcEventLogImpl::StopLogging()` also waits for a logging task at
`third_party/webrtc/logging/rtc_event_log/rtc_event_log_impl.cc:140-165`,
but this subtask did not trace a Chromium main-thread caller of that overload.
It remains a candidate.

## Verification and cleanup

Acquisition and both scans completed successfully.
`git submodule status -- <explicit paths>` reports the exact pinned commits without dirty/mismatch prefixes.
No source edits,
builds,
browser performance experiments,
or synthetic host load were performed.
The acquisition's surviving Git fsmonitor daemons were identified in its cgroup and stopped.
Both scan scopes also reached inactive state.
All `21` cited selections passed browser verification:
exact URL,
selected anchor,
anchor in the viewport,
and source text matching the pinned local checkout.
The checks are recorded in [the source-selection record](dependency-link-checks.json).
GitHub layout readiness was awaited before extracting `textContent`;
immediate `innerText` reads had produced empty strings while source text existed in the DOM.
The initial checks are retained in `chromium-dependency-link-checks-before-scroll.json`.
The probe's later navigation returned `agent-browser` diagnostic `outcome_unknown`
with `Resource temporarily unavailable (os error 11)`.
Its final close succeeded;
`doctor --offline --quick` then reported no failures,
and the complete pass with viewport readiness waits succeeded.
No cause is attributed to that transient transport failure.
The named `chromium-audit-dependency-links` browser closed,
and its final scope is inactive with no tasks remaining.

[ui-font]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/gfx/platform_font_skia.cc#L59-L84
[font-manager]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/skia/ext/font_utils.cc#L91-L130
[fci-match]: https://github.com/google/skia/blob/a65b5cc03a5131808aec172085b7074fc66625b9/src/ports/SkFontMgr_FontConfigInterface.cpp#L178-L205
[fci-open]: https://github.com/google/skia/blob/a65b5cc03a5131808aec172085b7074fc66625b9/src/ports/SkFontConfigInterface_direct.cpp#L750-L752
[skia-stream]: https://github.com/google/skia/blob/a65b5cc03a5131808aec172085b7074fc66625b9/src/core/SkStream.cpp#L913-L935
[pdf-font-callback]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/pdf/pdfium/pdfium_font_linux.cc#L133-L184
[pdf-font-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/pdf/pdfium/pdfium_font_linux.cc#L51-L106
[pdf-document]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/pdf/pdfium/pdfium_document.cc#L109-L117
[pdf-chunks]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/pdf/loader/document_loader_impl.cc#L203-L211
[aec-main]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/modules/mediastream/media_stream_audio_processor.cc#L91-L114
[aec-detach]: https://webrtc.googlesource.com/src/+/19576e37870bfe5a0a910a32f98ff411f722f43a/modules/audio_processing/audio_processing_impl.cc#1791
[aec-wait]: https://webrtc.googlesource.com/src/+/19576e37870bfe5a0a910a32f98ff411f722f43a/modules/audio_processing/aec_dump/aec_dump_impl.cc#78
[aec-write]: https://webrtc.googlesource.com/src/+/19576e37870bfe5a0a910a32f98ff411f722f43a/modules/audio_processing/aec_dump/aec_dump_impl.cc#265
[webrtc-file]: https://webrtc.googlesource.com/src/+/19576e37870bfe5a0a910a32f98ff411f722f43a/rtc_base/system/file_wrapper.cc#115
[v8-context]: https://github.com/v8/v8/blob/235050a0df6db2c9d92a419af2b72fbf3b2dd73b/src/snapshot/snapshot.cc#L204-L218
[v8-slice]: https://github.com/v8/v8/blob/235050a0df6db2c9d92a419af2b72fbf3b2dd73b/src/snapshot/snapshot.cc#L716-L733
[icu-common]: https://github.com/chromium-full-mirror/chromium_deps_icu/blob/2a51536dfb3b448bdf3dae70c782ea2879c90904/source/common/udata.cpp#L878-L898
[icu-pointer]: https://github.com/chromium-full-mirror/chromium_deps_icu/blob/2a51536dfb3b448bdf3dae70c782ea2879c90904/source/common/udatamem.cpp#L86-L88
[angle-loader]: https://github.com/google/angle/blob/127e783de33425c27a70a3f48da9d9202d6a0745/src/libANGLE/renderer/d3d/HLSLCompiler.cpp#L116-L143
[dawn-loader]: https://github.com/chromium-full-mirror/experimental_external_gob_dawn_dawn/blob/0caf3649477f7c3ffbd4b1ca4764a81b469b1582/src/dawn/native/vulkan/BackendVk.cpp#L355-L379
[dawn-dlopen]: https://github.com/chromium-full-mirror/experimental_external_gob_dawn_dawn/blob/0caf3649477f7c3ffbd4b1ca4764a81b469b1582/src/dawn/common/DynamicLib.cpp#L83-L104
