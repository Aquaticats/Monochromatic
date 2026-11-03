# Chromium platform, renderer storage, and lifecycle disk-blocking evidence

## Scope and evidence at 2026-11-03T08:15:09+00:00

This is a static source investigation of Chromium `156.0.8078.73`,
commit `2f9a11097447f226133bc1a31c787291a5eeab98`.
The local checkout is `${HOME}/temp/agent/chromium-disk-audit-20261103`.
`git rev-parse HEAD` verified the commit.
The initially fetched evidence files were compared byte-for-byte against the completed checkout:
`19` compared files matched.

Scoped `rg` passes inspected blocking allowances and synchronous-disk comments in
`chrome/android`, `android_webview`, `chrome/browser` Java/Kotlin,
`ios/chrome`, `ios/web`, `content/renderer`, `cc`, `components/viz`,
Blink storage, and the DOM-storage service.
Raw outputs are `chromium-platform-candidates.txt`, `chromium-platform-allowances.txt`,
and `chromium-platform-browser-android-allowances.txt` in the scratch directory.
The parent audit owns whole-tree enumeration and candidate accounting.
These passes are not an exhaustive call-graph proof.

No Android or iOS device tests were run.
No browser latency was measured by this subtask.
Each finding is a source-proven synchronous dependency with conditional UI impact,
not a measured freeze or a claim that every call reaches physical storage.
Disk-cache residency, feature selection, and call-site thread matter.
Source links in this evidence draft were checked against local pinned source;
the parent must perform the required browser selection-link checks before publishing them.

## Renderer cold localStorage access synchronously waits for database reads

Priority: investigate during live page stalls on every Chromium renderer platform.
This can stop page JavaScript and main-thread rendering;
it does not establish that browser controls stop responding.

The DOM-facing [`StorageArea::getItem`, `storage_area.cc:113-119`][storage-get-item]
calls the cache synchronously:

```cpp
// third_party/blink/renderer/modules/storage/storage_area.cc
return cached_area_->GetItem(key);
```

[`CachedStorageArea::GetItem`, `cached_storage_area.cc:69-71`][cache-get-item]
calls `EnsureLoaded()`.
If the in-renderer map is absent,
[`EnsureLoaded`, `cached_storage_area.cc:615-633`][cache-load]
waits for the result of a Mojo call:

```cpp
// third_party/blink/renderer/modules/storage/cached_storage_area.cc
if (map_)
  return;
// ...
remote_area_->GetAll(receiver_.BindNewPipeAndPassRemote(), &data);
```

The interface explicitly declares the call synchronous at
[`storage_area.mojom:134-139`][storage-sync]:

```mojom
// third_party/blink/public/mojom/dom_storage/storage_area.mojom
[Sync]
GetAll(pending_remote<StorageAreaObserver>? new_observer)
    => (array<KeyValue> data);
```

The service defers its response until the map is loaded:
[`StorageAreaImpl::GetAll`, `storage_area_impl.cc:504-521`][storage-service-load].

```cpp
// components/services/storage/dom_storage/storage_area_impl.cc
if (map_state_ != MapState::LOADED_KEYS_AND_VALUES) {
  LoadMap(OnLoadCompleteTask(
      base::BindOnce(&StorageAreaImpl::GetAll, weak_ptr_factory_.GetWeakPtr(),
                     std::move(new_observer), std::move(callback)),
      AccessMode::ReadOnly));
  return;
}
```

[`LoadMap`, `storage_area_impl.cc:569-606`][storage-read-map]
can commit a pending keys-only cache before reloading,
then calls `database_->ReadMapKeyValues(...)`.
[`AsyncDomStorageDatabase::ReadMapKeyValues`, `async_dom_storage_database.cc:90-112`][storage-db-read]
dispatches the read to the database sequence, but the renderer is still waiting for the reply.
The backing implementations contain actual database reads:

- [`LocalStorageLevelDB::ReadMapKeyValues`, `local_storage_leveldb.cc:178-181`][storage-leveldb]
  calls `leveldb_->GetMapKeyValues(...)`.
  [`dom_storage_database_leveldb.cc:242-260`][storage-leveldb-read]
  iterates key/value records via `ForEachWithPrefix`.
- [`LocalStorageSqlite::ReadMapKeyValues`, `local_storage_sqlite.cc:154-173`][storage-sqlite]
  starts a SQL transaction and obtains the map entries.
  [`MapEntriesTable::GetMapKeyValues`, `map_entries_table.cc:42-65`][storage-sql-query]
  executes `SELECT key, value_compression_type, value FROM map_entries WHERE map_id = ?`.

Warm cached access returns from `EnsureLoaded()` without this IPC.
An in-memory backend does not establish disk I/O.
This finding concerns the cold disk-backed localStorage branch,
not every localStorage operation or every use of the shared class for sessionStorage.

## Android active-tab restoration blocks the UI in both store generations

Priority: startup and user-requested restoration stalls.

The legacy store explicitly documents UI execution in
[`TabPersistentStoreImpl.java:717-757`][legacy-restore]:

```java
// chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java
// restoreTab is used to preempt async queue and restore immediately on the UI thread.
StrictMode.ThreadPolicy oldPolicy = StrictMode.allowThreadDiskReads();
// ...
if (restoredTabId == tabToRestore.id && mPrefetchTabStateActiveTabTask != null) {
    return mPrefetchTabStateActiveTabTask.get();
}
// Necessary to do on the UI thread as a last resort.
return TabStateFileManager.restoreTabState(
        getStateDirectory(), tabToRestore.id, mCipherFactory);
```

There are both a wait-for-prefetch path and a direct synchronous file-read fallback.
The state reader uses `FileInputStream`, stream reads, or a read-only file mapping at
[`TabStateFileManager.java:381-422`][tab-state-read].
The current FlatBuffer reader also opens the file and maps it at
[`TabStateFileManager.java:584-619`][tab-flatbuffer-read].

This is not only a legacy-store concern.
[`TabStateStore.java:342` and `618-624`][new-store-cache]
loads the cached active tab through `ActiveTabCache`.
[`ActiveTabCache.java:100-103`][active-cache-ui]
asserts UI execution and calls the shared tab cache.
[`TabCache.java:158-185`][new-cache-read] contains the same dependence:

```java
// chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/TabCache.java
assertOnUiThread();
FutureTask<@Nullable LoadedTabState> task = mPreloadTasks.remove(key);
if (task != null && !task.isCancelled()) {
    try {
        return task.get();
    } catch (ExecutionException | InterruptedException e) {
        // Ignore and attempt to restore on the UI thread.
    }
}
try (StrictModeContext ignored = StrictModeContext.allowDiskReads()) {
    return restoreTab(key);
}
```

`restoreTab` checks file existence and calls `TabStateFileManager.restoreTabStateInternal`.
Finished prefetch avoids the wait; an unfinished or absent prefetch does not.

## Android legacy tab persistence writes and flushes synchronously during lifecycle transitions

Priority: activity stop or persistent-state flush with unsaved tabs.
The implementation is conditional on the selected store or a legacy shadow store.

[`ChromeTabbedActivity.java:1811-1823`][activity-save]
calls `saveState()` from `onStopWithNative()` when its feature condition holds,
and from `flushPersistentState()`.
The activity delegates to its orchestrator at
[`ChromeTabbedActivity.java:5763-5764`][activity-delegate].
[`TabModelOrchestrator.java:163-174`][orchestrator-save]
invokes the authoritative and optional shadow stores synchronously.

[`TabPersistentStoreImpl.java:411-483`][legacy-save]
temporarily permits disk writes, saves metadata, then saves unsaved tabs:

```java
// chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java
StrictMode.ThreadPolicy oldPolicy = StrictMode.allowThreadDiskWrites();
// ...
saveListToFile(extractTabMetadata());
// ...
// Synchronously save any remaining unsaved tabs (hopefully very few).
for (Tab tab : mTabsToSave) {
    // ...
    TabStateFileManager.saveState(
            getStateDirectory(), state, id, incognito, mCipherFactory);
}
```

[`TabStateFileManager.java:643-668`][tab-save-dispatch]
calls `saveStateInternal` directly;
only legacy-file cleanup is posted to a background task.
[`TabStateFileManager.java:804-879`][tab-atomic-save]
writes the FlatBuffer through `AtomicFile.startWrite()` and channel writes,
then calls `atomicFile.finishWrite(fileOutputStream)`.
The source explicitly states that this finalization calls `FileUtils.sync` (`fsync`).
This report did not independently audit the Android framework implementation.

Do not generalize this synchronous save to the newer store.
[`TabStateStore.java:290-306`][new-store-save]
states that mutations are queued to the DB thread and boosts its priority instead of blocking on a flush.
[`TabPersistentStoreFactory.java:74-106`][store-selection]
selects the legacy or new implementation from migration state;
the same file has a conditional legacy shadow-store branch.
A device's actual store selection was not measured.

## Android history submenu falls back to a synchronous metadata read

Priority: live interaction, when the closed-window tab cache is cold.

[`HistoryItemBuilder.java:388-399`][history-submenu]
builds a closed-window submenu by requesting its tabs.
[`RecentlyClosedEntriesManager.java:709-724`][history-read]
returns a cached list when available, but otherwise explicitly allows UI-thread disk I/O:

```java
// chrome/android/java/src/org/chromium/chrome/browser/RecentlyClosedEntriesManager.java
// Fallback to synchronous I/O if cache is cold. It's not ideal to perform I/O on UI thread,
// but this should seldom happen.
try (StrictModeContext ignored = StrictModeContext.allowDiskReads()) {
    infoList = TabPersistentStoreImpl.getTabListForClosedWindow(instanceId);
}
```

The synchronous overload at
[`TabPersistentStoreImpl.java:2254-2275`][closed-window-file]
checks the file, opens a `FileInputStream`, and reads saved metadata.
The asynchronous overload and prefetch exist, but this fallback does not wait asynchronously.
The source's frequency comment is not a measured rate in this audit.

## Android legacy first-run tab ID repair enumerates directories on the UI thread

Priority: first relevant initialization with the computed-ID preference absent.

[`TabPersistentStoreImpl.java:1255-1307`][tab-id-scan]
checks `TABMODEL_HAS_COMPUTED_MAX_ID` and returns when already set.
Otherwise it deliberately performs the work on the UI thread:

```java
// chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java
// This is done on the UI thread because it is on the
// critical patch to initializing the TabIdManager with the correct max tab ID.
StrictMode.ThreadPolicy oldPolicy = StrictMode.allowThreadDiskReads();
try {
    File[] subDirectories = TabStateDirectory.getOrCreateBaseStateDirectory().listFiles();
    // ... listFiles on each subdirectory and FileInputStream on metadata files ...
}
```

`loadState()` invokes this at line `552`.
The runtime work scales with persisted directories and files,
and includes metadata parsing in addition to directory reads.
This branch is not an ongoing every-startup scan once its preference is set.

## Android WebView initialization opens, locks, and writes its data lock file on UI

Priority: first WebView creation or startup in an embedding app.

[`StartupTasks.java:111-117`][webview-lock-caller]
asserts the UI thread and calls `AwDataDirLock.lock`.
[`WebViewChromium.java:221-225`][webview-constructor-lock]
also invokes the lock even in the nonblocking-constructor feature branch.
[`AwDataDirLock.java:67-92`][webview-lock-open]
opens the lock file synchronously:

```java
// android_webview/java/src/org/chromium/android_webview/AwDataDirLock.java
try (DualTraceEvent e1 = DualTraceEvent.scoped("AwDataDirLock.lock");
        StrictModeContext ignored = StrictModeContext.allowDiskWrites()) {
    // ...
    sLockFile = new RandomAccessFile(lockFile, "rw");
}
```

[`AwDataDirLock.java:134-182`][webview-lock-retry]
calls `tryLock()`, writes process information after acquisition,
and sleeps on retries.
The constants at lines `37-39` are `LOCK_RETRIES = 16` and `LOCK_SLEEP_MS = 100`.
The documented `1.5 s` retry budget bounds the deliberate sleeps,
not the filesystem open, metadata, lock operation, or write latency.
An already-held process lock returns early.
This is a WebView app path, not proof about desktop Chromium.

## Android WebView synchronous cookie APIs can make an app UI wait for storage

Priority: embedding apps calling `CookieManager.flush()` on their UI thread.
The Chromium API implementation preserves synchronous caller semantics;
it does not enforce that the caller is UI.

[`CookieManagerAdapter.java:222-227`][webview-cookie-api]
delegates `flush()` to `AwCookieManager.flushCookieStore`.
[`cookie_manager.cc:776-790`][webview-cookie-flush]
uses `ExecCookieTaskSync`, which posts work then waits at
[`cookie_manager.cc:248-255`][webview-cookie-wait]:

```cpp
// android_webview/browser/cookie_manager.cc
WaitableEvent completion(base::WaitableEvent::ResetPolicy::AUTOMATIC,
                         base::WaitableEvent::InitialState::NOT_SIGNALED);
ExecCookieTask(
    base::BindOnce(std::move(task), SignalEventClosure(&completion)));
base::ScopedAllowBaseSyncPrimitivesOutsideBlockingScope wait;
completion.Wait();
```

The provisional-store branch calls `CookieMonster::FlushStore`,
which calls the persistent store at
[`cookie_monster.cc:513-521`][cookie-monster-flush].
[`sqlite_persistent_store_backend_base.cc:226-233`][cookie-db-flush]
executes `Commit()` on the DB worker before posting completion.
[`sqlite_persistent_cookie_store.cc:1444-1447`][cookie-db-commit]
commits the SQL transaction.
The network-service branch instead calls its cookie-manager Mojo flush;
this subtask did not re-trace that branch to its storage backend.
Pending writes and a persistent initialized store are necessary for actual flush work.
Calling the API off UI avoids directly blocking the UI caller.

## iOS session restoration and lazy tab realization synchronously read files

Priority: startup and first showing a lazily restored tab.

[`session_restoration_service_impl.mm:320-339`][ios-session-design]
explicitly documents synchronous main-sequence reads alongside background writes.
Startup [`LoadSession`, lines `407-429`][ios-session-load]
calls `ios::sessions::LoadSessionStorage` synchronously.
Its main-thread blocking duration is recorded at lines `478-480`.
[`session_loading.mm:206-217`][ios-session-metadata]
parses the session metadata file.

Lazy per-tab restoration also retains synchronous readers.
[`session_restoration_service_impl.mm:65-105`][ios-tab-readers]
passes `LoadWebStateStorage` and `LoadWebStateSession` callbacks to
`WebState::CreateWithStorage`.
[`WebStateImpl::WasShown`, `web_state_impl.mm:632-634`][ios-tab-shown]
calls `RealizedState()`.
[`RealizedState`, lines `1077-1084`][ios-realized-state]
forces realization when absent.
[`ForceRealizedWithPolicy`, lines `562-567`][ios-tab-realize]
calls `saved->LoadStorage()` synchronously:

```cpp
// ios/web/web_state/web_state_impl.mm
pimpl_->InitWithProto(saved->GetBrowserState(), saved->GetLastActiveTime(),
                      saved->GetTitle(), saved->GetVisibleURL(),
                      saved->GetFaviconStatus(), saved->LoadStorage(),
                      saved->TakeNativeSessionFetcher());
```

[`web_state_impl_serialized_data.mm:70-73`][ios-load-callback]
runs the storage callback inline.
The underlying file reader is explicit at
[`session_internal_util.mm:162-165`][ios-file-read]:

```objc
// ios/chrome/browser/sessions/model/session_internal_util.mm
return [NSData dataWithContentsOfFile:filename options:0 error:nil];
```

Already realized tabs avoid this load path.
The native session fetcher is also file-backed,
but its full call-site chain was not independently traced here;
the protobuf load alone establishes the synchronous tab-realization dependency.

## iOS profile creation synchronously checks and creates directories

Priority: profile initialization.

[`profile_ios_impl.mm:169-208`][ios-profile-dirs]
explicitly justifies main-thread directory creation,
then performs `PathExists` and `CreateDirectory` calls for profile,
off-the-record stash, and cache paths.

```cpp
// ios/chrome/browser/profile/model/profile_ios_impl.mm
base::ScopedAllowBlocking allow_blocking_to_create_directory;
if (!base::PathExists(state_path)) {
  if (!base::CreateDirectory(state_path)) {
    return Result::Failure();
  }
}
```

The constructor invokes this helper before preference setup at lines `248-251`.
The existence checks still touch filesystem metadata when the directories already exist.
The source calls the operation lightweight;
no latency measurement in this audit supports that adjective on a slow filesystem.

## Browser shutdown waits for disk workers even though ordinary saves are asynchronous

Priority: delayed process exit or restart; separate from normal interactive UI stalls.

[`BrowserMainLoop::ShutdownThreadsAndCleanUp`, `browser_main_loop.cc:1199-1216`][shutdown-allow]
allows blocking and synchronization during teardown.
At [`browser_main_loop.cc:1301-1306`][shutdown-pool]
it calls the thread pool's `Shutdown()`.
[`thread_pool_impl.cc:425-450`][shutdown-tracker]
starts shutdown and calls `CompleteShutdown()`.
[`task_tracker.cc:235-247`][shutdown-wait]
waits on an event until blocking tasks complete:

```cpp
// base/task/thread_pool/task_tracker.cc
// Now block the thread until all tasks are done.
TS_UNCHECKED_READ(shutdown_event_)->Wait();
```

There are concrete disk-writing task sequences with this shutdown policy.
[`BookmarkStorage`, `bookmark_storage.cc:181-195`][bookmark-runner]
uses `MayBlock`, `BEST_EFFORT`, and `BLOCK_SHUTDOWN` for its writer.
[`ImportantFileWriter::WriteNowWithBackgroundDataProducer`, `important_file_writer.cc:463-478`][important-post]
posts the actual write to that sequence.
The atomic write performs file writes and `tmp_file.Flush()` at
[`important_file_writer.cc:310-342`][important-flush].
Therefore an outstanding slow bookmark disk write can delay thread-pool shutdown.
This does not make ordinary bookmark saves synchronous on UI.
`BrowserMainRunnerImpl` calls `RunMainMessageLoop()` and later its shutdown method;
this finding should be categorized as teardown, not browsing-time responsiveness.

## False positives and limits retained

- `StrictModeContext.allowDiskReads/Writes` only relaxes Android diagnostics;
  it does not itself perform or defer I/O.
  The findings trace actual operations inside or after the allowance.
- `MostVisitedSitesMetadataUtils.restoreFileToSuggestionListsOnUiThread` has a concerning name
  and direct file-reading implementation,
  but the scoped `chrome/android` caller search found its declaration and tests,
  not a production call to this UI-named helper.
  It remains an unconfirmed reachability candidate, not a confirmed live UI path.
- New `TabStateStore.saveState()` queues DB work and explicitly avoids waiting for a flush.
  Its active-tab cache read remains synchronous as described in the restoration finding.
- iOS session writes are posted to a background sequence.
  A comment about time spent saving on main includes serialization overhead;
  it does not establish that those writes happen on main.
- `ImportantFileWriter::WriteNow` means schedule a write now;
  it does not imply the file write executes on its caller.
- The WebView lock sleep budget is not a deadline on disk operations.
- No source patch, synthetic host load, runtime browser restart, or upstream filing was performed.

[storage-get-item]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/modules/storage/storage_area.cc#L113-L119
[cache-get-item]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/modules/storage/cached_storage_area.cc#L69-L71
[cache-load]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/renderer/modules/storage/cached_storage_area.cc#L615-L633
[storage-sync]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/third_party/blink/public/mojom/dom_storage/storage_area.mojom#L134-L139
[storage-service-load]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/storage_area_impl.cc#L504-L521
[storage-read-map]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/storage_area_impl.cc#L569-L606
[storage-db-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/async_dom_storage_database.cc#L90-L112
[storage-leveldb]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/leveldb/local_storage_leveldb.cc#L178-L181
[storage-leveldb-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/leveldb/dom_storage_database_leveldb.cc#L242-L260
[storage-sqlite]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/sqlite/local_storage_sqlite.cc#L154-L173
[storage-sql-query]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/services/storage/dom_storage/sqlite/map_entries_table.cc#L42-L65
[legacy-restore]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java#L717-L757
[tab-state-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/tabpersistence/android/java/src/org/chromium/chrome/browser/tabpersistence/TabStateFileManager.java#L381-L422
[tab-flatbuffer-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/tabpersistence/android/java/src/org/chromium/chrome/browser/tabpersistence/TabStateFileManager.java#L584-L619
[new-store-cache]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/TabStateStore.java#L618-L624
[active-cache-ui]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/ActiveTabCache.java#L100-L103
[new-cache-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/TabCache.java#L158-L185
[activity-save]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/ChromeTabbedActivity.java#L1811-L1823
[activity-delegate]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/ChromeTabbedActivity.java#L5763-L5764
[orchestrator-save]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/TabModelOrchestrator.java#L163-L174
[legacy-save]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java#L411-L483
[tab-save-dispatch]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/tabpersistence/android/java/src/org/chromium/chrome/browser/tabpersistence/TabStateFileManager.java#L643-L668
[tab-atomic-save]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/tabpersistence/android/java/src/org/chromium/chrome/browser/tabpersistence/TabStateFileManager.java#L804-L879
[new-store-save]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/TabStateStore.java#L290-L306
[store-selection]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/app/tabmodel/TabPersistentStoreFactory.java#L74-L106
[history-submenu]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/tabbed_mode/HistoryItemBuilder.java#L388-L399
[history-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/RecentlyClosedEntriesManager.java#L709-L724
[closed-window-file]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java#L2254-L2275
[tab-id-scan]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/android/java/src/org/chromium/chrome/browser/tabmodel/TabPersistentStoreImpl.java#L1255-L1307
[webview-lock-caller]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/java/src/org/chromium/android_webview/StartupTasks.java#L111-L117
[webview-constructor-lock]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/glue/java/src/com/android/webview/chromium/WebViewChromium.java#L221-L225
[webview-lock-open]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/java/src/org/chromium/android_webview/AwDataDirLock.java#L67-L92
[webview-lock-retry]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/java/src/org/chromium/android_webview/AwDataDirLock.java#L134-L182
[webview-cookie-api]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/glue/java/src/com/android/webview/chromium/CookieManagerAdapter.java#L222-L227
[webview-cookie-flush]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/browser/cookie_manager.cc#L776-L790
[webview-cookie-wait]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/android_webview/browser/cookie_manager.cc#L248-L255
[cookie-monster-flush]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/net/cookies/cookie_monster.cc#L513-L521
[cookie-db-flush]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/net/extras/sqlite/sqlite_persistent_store_backend_base.cc#L226-L233
[cookie-db-commit]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/net/extras/sqlite/sqlite_persistent_cookie_store.cc#L1444-L1447
[ios-session-design]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/chrome/browser/sessions/model/session_restoration_service_impl.mm#L320-L339
[ios-session-load]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/chrome/browser/sessions/model/session_restoration_service_impl.mm#L407-L429
[ios-session-metadata]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/chrome/browser/sessions/model/session_loading.mm#L206-L217
[ios-tab-readers]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/chrome/browser/sessions/model/session_restoration_service_impl.mm#L65-L105
[ios-tab-shown]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/web/web_state/web_state_impl.mm#L632-L634
[ios-realized-state]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/web/web_state/web_state_impl.mm#L1077-L1084
[ios-tab-realize]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/web/web_state/web_state_impl.mm#L562-L567
[ios-load-callback]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/web/web_state/web_state_impl_serialized_data.mm#L70-L73
[ios-file-read]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/chrome/browser/sessions/model/session_internal_util.mm#L162-L165
[ios-profile-dirs]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ios/chrome/browser/profile/model/profile_ios_impl.mm#L169-L208
[shutdown-allow]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/browser/browser_main_loop.cc#L1199-L1216
[shutdown-pool]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/browser/browser_main_loop.cc#L1301-L1306
[shutdown-tracker]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/task/thread_pool/thread_pool_impl.cc#L425-L450
[shutdown-wait]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/task/thread_pool/task_tracker.cc#L235-L247
[bookmark-runner]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/bookmarks/browser/bookmark_storage.cc#L181-L195
[important-post]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/files/important_file_writer.cc#L463-L478
[important-flush]: https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/base/files/important_file_writer.cc#L310-L342
