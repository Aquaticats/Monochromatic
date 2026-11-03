# Chromium mapped-file disk risks at 2026-11-03T08:10:39+00:00

Source revision:
`2f9a11097447f226133bc1a31c787291a5eeab98`,
Chromium `156.0.8078.73`.
The complete shallow checkout is `${HOME}/temp/agent/chromium-disk-audit-20261103`.
Initial pinned raw-source downloads are in sibling `chromium-mapped-source`.
This is static source tracing except for the explicitly attributed prior BrowserMetrics capture.
No browser latency experiment or source patch was performed.
The cited source selections were opened in a disposable browser session,
with selected text compared against the pinned checkout.
The [verification record](core-link-checks.json) is shared with the overview's source checks.

## BrowserMetrics writes can fault on the browser UI thread

Priority:
strongest evidence,
browser-wide responsiveness boundary.
The prior runtime finding is in [`helium-window-freezes.md`](../helium-window-freezes.md):
a matching main-thread write fault landed in the active `BrowserMetrics/*.pma` mapping,
with `btrfs_page_mkwrite` and writeback/transaction waits.
That report,
rather than this source pass,
supplies the measured stall evidence.

The installed source initializes mapped histograms before field trials in
[`chrome/app/chrome_main_delegate.cc:1745-1765`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/app/chrome_main_delegate.cc#L1745-L1765).
[`components/metrics/persistent_histograms.cc:124-164`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/metrics/persistent_histograms.cc#L124-L164)
constructs the active path and calls `GlobalHistogramAllocator::CreateWithFile`.
[`base/metrics/persistent_histogram_allocator.cc:759-797`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/metrics/persistent_histogram_allocator.cc#L759-L797):

```cpp
// base/metrics/persistent_histogram_allocator.cc
File file(file_path, flags);
success = mmfile->Initialize(std::move(file), MemoryMappedFile::READ_WRITE);
```

New files use `READ_WRITE_EXTEND`;
both modes reach a shared writable mapping in
[`base/files/memory_mapped_file_posix.cc:74-101`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/files/memory_mapped_file_posix.cc#L74-L101):

```cpp
// base/files/memory_mapped_file_posix.cc
int flags = MAP_SHARED;
// READ_WRITE and READ_WRITE_EXTEND select PROT_READ | PROT_WRITE.
mmap(nullptr, map_size, prot, flags, file_.GetPlatformFile(), map_start)
```

Histogram counts are `DelayedPersistentAllocation` storage
in `persistent_histogram_allocator.cc:624-649`.
`PersistentSampleVector` mounts that allocation at `base/metrics/sample_vector.cc:697-706`.
The normal recording path at
[`base/metrics/sample_vector.cc:136-162`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/metrics/sample_vector.cc#L136-L162)
writes synchronously on its caller:

```cpp
// base/metrics/sample_vector.cc
Count32 new_bucket_count =
    subtle::NoBarrier_AtomicIncrement(&counts_at(bucket_index), count);
IncreaseSumAndCount(strict_cast<int64_t>(count) * value, count);
```

This path needs no explicit `fsync` syscall at the sample call site.
An atomic memory access still touches a file-backed page.
It is not evidence that every histogram write waits or that every Chromium-derived application uses this allocator.

## UI resource packs expose mapped bytes during synchronous string and image lookup

Priority:
browser UI candidate,
cold or reclaimed file pages;
no runtime stall measured here.
[`ui/base/resource/data_pack.cc:245-268`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/base/resource/data_pack.cc#L245-L268)
opens the pack,
maps it,
and retains the mapping for an uncompressed pack.
Its data source returns the mapped span at `data_pack.cc:114-128`.
Its resource lookup returns a view into that data at
[`ui/base/resource/data_pack.cc:443-474`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/base/resource/data_pack.cc#L443-L474):

```cpp
// ui/base/resource/data_pack.cc
return GetStringViewFromOffset(target->file_offset, next_entry->file_offset,
                               data_source_->GetData());
```

The synchronous UI chain is `SimpleMenuModel::AddItemWithStringId`
at `ui/menus/simple_menu_model.cc:86-97`,
`l10n_util::GetStringUTF16` at `ui/base/l10n/l10n_util.cc:358-360`,
and `ResourceBundle::GetLocalizedStringImpl` at
[`ui/base/resource/resource_bundle.cc:1266-1325`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/base/resource/resource_bundle.cc#L1266-L1325):

```cpp
// ui/base/resource/resource_bundle.cc
data = locale_data->GetStringView(static_cast<uint16_t>(resource_id));
```

The function copies/converts the mapped text to UTF-16 while holding `locale_resources_data_lock_`.
Concrete user-triggered caller:
`BrowserAppMenuButton::ShowMenuWithFlags`
constructs `AppMenuModel` at `chrome/browser/ui/views/toolbar/browser_app_menu_button.cc:83-112`;
`AppMenuModel::Init` calls `Build` at `chrome/browser/ui/toolbar/app_menu_model.cc:1263-1264`;
`Build` adds localized menu entries at `app_menu_model.cc:2123-2129`.
The image path also reads resource-pack bytes in
`ui/base/resource/resource_bundle.cc:1182-1207` when decoding a bitmap.

Limits:
a cache hit can avoid loading an image;
resident pages do not need disk reads;
gzip-compressed packs are expanded into `StringDataSource` at `data_pack.cc:258-266`,
so their later resource bytes are not borrowed from the pack mapping.
The finding is specifically that uncompressed mapped resources remain on synchronous UI paths.

## Font misses synchronously await disk work, then retain file mappings for rendering

Priority:
renderer responsiveness candidate during layout or new font use.
The active Linux default is `kFontDataServiceLinux`,
enabled at `content/common/features.cc:311`.
`FontDataServiceForCSSLocalFonts` is disabled by default at `features.cc:332-334`,
so CSS `local()` has an additional legacy route.

The service contract is explicit in
[`components/services/font_data/public/mojom/font_data_service.mojom:93-111`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/font_data/public/mojom/font_data_service.mojom#L93-L111):

```text
// components/services/font_data/public/mojom/font_data_service.mojom
// Called synchronously from layout and asynchronously by the renderer's font prewarmer.
[Sync]
MatchFamilyName(string family_name, TypefaceStyle style) =>
   (MatchFamilyNameResponse response);
```

`FontDataManager::onMatchFamilyStyle` uses that synchronous call on a cache miss at
`content/child/font_data/font_data_manager.cc:187-227`.
The service has a `MayBlock`,
`USER_BLOCKING` sequenced worker at
`components/services/font_data/font_data_service_impl.cc:70-74`,
but the renderer awaits its result.
Before responding,
`CreateMatchFamilyNameResult` opens the font stream and obtains a file handle
at `font_data_service_impl.cc:475-486`.
The actual file open is at
[`components/services/font_data/font_data_service_impl.cc:125-150`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/font_data/font_data_service_impl.cc#L125-L150):

```cpp
// components/services/font_data/font_data_service_impl.cc
auto font_file =
    base::File(font_file_path, base::File::FLAG_OPEN | base::File::FLAG_READ |
                                   base::File::FLAG_WIN_EXCLUSIVE_WRITE);
```

The renderer then maps the file and gives Skia a direct stream into it at
[`content/child/font_data/font_data_manager.cc:494-523`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/child/font_data/font_data_manager.cc#L494-L523):

```cpp
// content/child/font_data/font_data_manager.cc
if (new_mapping->Initialize(std::move(
        match_result->typeface_data->get_font_file()->file_handle))) {
  mapped_files_entry = std::move(new_mapping);
}
typeface = onMakeFromStreamArgs(
    SkMemoryStream::MakeDirect(font_data.data(), font_data.size()), args);
```

That adds direct file-page exposure after the synchronous service dependency.
The alternative shared-memory response at `font_data_manager.cc:525-559` is not classified as a disk mapping.
The cache and font prewarming can avoid some misses;
font prewarming is disabled by default at `content/common/features.cc:329-330`.
No font latency or default-configuration override was measured in the running browser.

The legacy Linux route is also blocking:
`components/services/font/public/cpp/font_loader.cc:56-82` obtains a mapped font stream;
`font_service_thread.cc:142-155` posts the open and then calls `done_event.Wait()`;
`mapped_font_file.cc:21-34` maps the returned font file and exposes its bytes without copying.
CSS `local()` reaches this route through
`third_party/blink/renderer/platform/fonts/linux/font_unique_name_lookup_linux.cc:17-36`
and `fonts/skia/sktypeface_factory.cc:21-31`.

## Hunspell dictionaries remain mapped on synchronous editing paths

Priority:
renderer responsiveness candidate when focusing an editable field,
checking text,
or requesting spelling suggestions.
[`components/spellcheck/renderer/hunspell_engine.cc:62-69`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/spellcheck/renderer/hunspell_engine.cc#L62-L69):

```cpp
// components/spellcheck/renderer/hunspell_engine.cc
bdict_file_ = std::make_unique<base::MemoryMappedFile>();
if (bdict_file_->Initialize(std::move(file_))) {
  hunspell_ = std::make_unique<Hunspell>(bdict_file_->bytes());
}
```

Initialization is delayed until needed at `hunspell_engine.cc:120-135`.
`SpellCheckProvider::FocusedElementChanged` invokes initialization for the on-demand feature
at `spellcheck_provider.cc:271-276`.
`SpellCheckProvider::CheckSpelling` synchronously calls `SpellCheckWord`
at `spellcheck_provider.cc:310-344`;
`SpellCheck::SpellCheckWord` calls `InitializeIfNeeded` at `spellcheck.cc:285-295`.
Actual spelling and suggestions call Hunspell at `hunspell_engine.cc:89,110-111`.

The vendored reader retains the original span at
[`third_party/hunspell/google/bdict_reader.cc:777-793`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/hunspell/google/bdict_reader.cc#L777-L793):

```cpp
// third_party/hunspell/google/bdict_reader.cc
bdict_data_ = bdict_data;
NodeReader reader(bdict_data_, header_.dic_offset, 0);
return reader.FindWord(base::as_byte_span(word), affix_indices);
```

Posting delayed paragraph checking does not prove a worker-thread escape:
`spellcheck.cc:530-538` uses `SingleThreadTaskRunner::GetCurrentDefault()`.
The feature/build selection is `components/spellcheck/renderer/spelling_engine.cc:20-38`.
This finding applies to the Hunspell implementation,
not every platform-native spellchecker.

## External V8 snapshots and ICU data retain startup and cold-page dependencies

Priority:
startup,
renderer creation,
and cold data access candidates.
`content/app/content_main_runner_impl.cc:1025-1033` initializes ICU,
then loads the V8 snapshot when external startup data is configured.
`gin/v8_initializer.cc:155-182` opens the file synchronously;
the `FILE_ERROR_IN_USE` branch even sleeps between bounded retries.
`gin/v8_initializer.cc:142-150` maps the file.
`GetMappedFileData` points V8 directly at it at
[`gin/v8_initializer.cc:78-82`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/gin/v8_initializer.cc#L78-L82):

```cpp
// gin/v8_initializer.cc
data->data = reinterpret_cast<const char*>(mapped_file->bytes().data());
data->raw_size = static_cast<int>(mapped_file->bytes().size());
```

The pointer is passed to `v8::V8::SetSnapshotDataBlob` at `gin/v8_initializer.cc:539-543`.
Context creation uses `v8::Context::FromSnapshot` at
`third_party/blink/renderer/bindings/modules/v8/v8_context_snapshot_impl.cc:346-350`.
The V8 dependency itself is a separate checkout,
pinned in `DEPS:344` to
`235050a0df6db2c9d92a419af2b72fbf3b2dd73b`;
its mapped-data consumption is traced in the
[dependency evidence](dependencies.md#v8-and-icu-preserve-the-mapped-data-dependencies).
Do not claim a measured ongoing V8 stall from this integration trace.

For external ICU data,
`base/i18n/icu_util.cc:186-197` opens `icudtl.dat`;
[`base/i18n/icu_util.cc:241-248`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/i18n/icu_util.cc#L241-L248)
maps it and passes its pointer to ICU:

```cpp
// base/i18n/icu_util.cc
(*out_error_code) = U_ZERO_ERROR;
udata_setCommonData((*out_mapped_data_file)->bytes().data(), out_error_code);
```

`ICU_UTIL_DATA_STATIC` is a distinct build branch at `icu_util.cc:447-450`.
This subtask traced the external-data integration,
not every ICU consumer or its dependency internals.

## Legacy GPU disk-cache mapping is not a proven steady-state UI fault

`gpu/ipc/host/gpu_disk_cache.cc:681-686` creates a `net::SHADER_CACHE` backend.
`net/disk_cache/blockfile/mapped_file_posix.cc:26-27` maps its files writable and shared.
However,
normal blockfile operations use a dedicated `CacheThread_BlockFile`
defined at `net/disk_cache/blockfile/backend_impl.cc:132-155`.
[`net/disk_cache/blockfile/in_flight_backend_io.cc:663-666`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/net/disk_cache/blockfile/in_flight_backend_io.cc#L663-L666):

```cpp
// net/disk_cache/blockfile/in_flight_backend_io.cc
background_thread_->PostTask(
    from_here, base::BindOnce(&BackendIO::ExecuteOperation, operation));
```

GPU-facing cache reads in `gpu/command_buffer/service/dawn_caching_interface.cc:57-82`
read a memory cache.
`GrShaderCache::WriteToDisk` at `gpu/command_buffer/service/gr_shader_cache.cc:253-266`
forwards to `client_->StoreShader`;
its name alone is not a disk syscall.
Therefore,
the existence of `GPUCache`/`Dawn*Cache` mappings in a browser process
does not identify a UI-thread fault.

There is a real synchronous teardown edge:
[`net/disk_cache/blockfile/backend_impl.cc:198-223`](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/net/disk_cache/blockfile/backend_impl.cc#L198-L223)
posts `FinalCleanupCallback` and waits:

```cpp
// net/disk_cache/blockfile/backend_impl.cc
base::ScopedAllowBaseSyncPrimitivesOutsideBlockingScope allow_wait;
done.Wait();
```

Cleanup writes the mapped crash flag at `backend_impl.cc:365-368`,
closes files,
and flushes the index at `backend_impl.cc:380-382`.
The invoking owner/teardown thread was not fully traced in this subtask,
so classify this as a pending lifetime-specific candidate rather than a confirmed ordinary-browsing UI stall.
The [persistent-cache finding](../chromium-disk-responsiveness.md#gpu-persistent-cache-misses-synchronously-read-sqlite-backed-files)
concerns a separate implementation and must not inherit this exclusion.

## Coverage and exclusions

This subtask searched mapped-file references across `chrome`,
`content`,
`components`,
`ui`,
`gin`,
`gpu`,
`skia`,
and `third_party/blink` production `.cc`/`.mm` files,
then traced the listed candidates.
It does not certify every mapped call site in the full source tree.
The [overview and complete-tree ledger](../chromium-disk-responsiveness.md#verification-and-coverage)
supply the overall scan scope.
Wayland keymaps and DMA buffers were not classified as regular disk files merely because they use `mmap`.
Off-main translation/model-loading helpers were not promoted to UI-thread findings without a blocking caller chain.
No synthetic host load,
browser state changes,
background poller,
or source edits were used.
All download/scan scopes were transient and completed.
