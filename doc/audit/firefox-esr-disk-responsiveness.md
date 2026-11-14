# Firefox ESR disk operations that can affect UI responsiveness

## Result and baseline at 2026-11-03T03:42:34-05:00

Firefox ESR retains direct filesystem work on the browser main thread,
main-thread waits for background disk work,
render-thread disk reads,
and file mappings consumed during UI work.
The detailed findings identify the trigger,
affected thread,
blocking operation,
and relevant configuration limits.
These are source-confirmed dependencies and explicitly marked candidates,
not reproduced freezes or measured latency.

The baseline is `153.6.0esr`,
the release reported as `FIREFOX_ESR` by
[Mozilla's product metadata](https://product-details.mozilla.org/1.0/firefox_versions.json).
[The release tag](https://api.github.com/repos/mozilla-firefox/firefox/git/ref/tags/FIREFOX_153_6_0esr_RELEASE)
resolves to Git commit `a34c4160e3f75ab9918fbe4669ff7ae56f1977c8`.
The checkout's `browser/config/version_display.txt` agrees.
The installed ESR's `application.ini` also reports version `153.6.0`;
this audit uses the immutable release source,
without asserting that every installed binary or system library is byte-identical to the checkout.
The separate legacy `115.44.0esr` track is outside this selected baseline.

The most actionable findings for a responsiveness investigation are:

- UI actions directly touching files:
  Downloads commands and filename reservation,
  file-picker previews/completion,
  large clipboard transfers,
  Save Page,
  printing,
  local extension installation,
  certificate import/export,
  and updater capability probes.
- Shared locks carrying disk delays back to the UI:
  startup-cache writes,
  certificate-state persistence,
  Glean memory reporting,
  and cache memory reporting.
- First-use waits:
  Places,
  cookies,
  permissions,
  HSTS state,
  bounce-tracking storage,
  NSS,
  handlers,
  chrome modules,
  localization,
  and fonts.
- Other visible boundaries:
  WebRender shader reads can delay frame production;
  cold `localStorage` access can block a page's main thread;
  final cache/preferences/telemetry work can delay quitting.

Parent-main-thread stalls can stop browser UI handlers.
Content-main-thread stalls affect the involved content process.
A blocked render thread can stop producing frames while input handling continues elsewhere.
An event loop that spins while awaiting shutdown is a completion dependency,
not automatically an unresponsive event loop.
The report keeps these boundaries separate.

## Coverage and retained evidence

The complete shallow release checkout contains `468,910` tracked regular files,
with no gitlinks or symlinks in its tracked inventory.
An unignored discovery pass found exactly the same files;
there were no missing tracked regular files.
The complete-tree search included hidden files,
vendored libraries,
tests,
build tooling,
and mobile sources.
Only Git metadata was excluded from discovery.

Ripgrep reported `468,910` searches,
`82,115` matching lines,
and `22,108` files with matches.
The retained patterns cover native file APIs,
Gecko file and SQLite APIs,
file mappings,
synchronous waits,
and comments identifying main-thread I/O.
The raw counts include non-runtime code and false positives.
They are not counts of blockers.

Evidence artifacts:

- [Scan parameters, counts, and directory coverage](firefox-esr-disk-responsiveness.scan.json).
- Tracked paths and Git object identities (`firefox-esr-disk-responsiveness.inventory.json.gz`).
- Complete matching lines and ripgrep events (`firefox-esr-disk-responsiveness.candidates.jsonl.gz`).
- Pinned source selections supporting the citations (`firefox-esr-disk-responsiveness.sources.json.gz`).
- Browser checks of Firefox source selections (`firefox-esr-disk-responsiveness.browser.json.gz`).
- [Native-library source selections and browser checks](firefox-esr-disk-responsiveness.external.json).
- [Markdown rendering receipts](firefox-esr-disk-responsiveness.render.json).
  `firefox-esr-disk-responsiveness.browser.json.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `firefox-esr-disk-responsiveness.candidates.jsonl.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `firefox-esr-disk-responsiveness.inventory.json.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.
  `firefox-esr-disk-responsiveness.sources.json.gz` left the tree on 2026-11-13.
  `doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

The scan searched every tracked file;
the reachability audit followed the candidate families described in this report.
It did not manually adjudicate every matching line or prove an exhaustive call graph.
Binary contents,
code generated only during a build,
external operating-system/framework implementations,
and undocumented indirect dispatch can conceal additional dependencies.
No-match results are not proofs that a subsystem is disk-independent.
Mobile source was included in discovery,
but Android application lifecycle paths were not classified as desktop ESR behavior.

The source passes covered desktop UI and profiles,
storage and security,
graphics/resources/fonts/telemetry,
and saving/printing/native maintenance.
They are same-vendor parallel source passes,
not independent reviews.
No Firefox source,
live browser profile,
or browser setting was changed.
No synthetic host load or runtime stall benchmark was run.

## Reading the findings

A synchronous file call is a potential disk wait even when a warm filesystem cache usually satisfies it.
Conversely,
an asynchronous API can still block its caller while constructing a task or acquiring a shared lock.
The findings name those intermediate steps instead of treating every background write as harmless
or every occurrence of `read` as a UI freeze.

The source citations are pinned to the audited release.
Line numbers without a repeated path belong to the file named in the same paragraph or finding.
Configuration guards refer to source defaults,
not a probe of the user's live preferences.

## Browser UI, extensions, profiles, and updates

### Downloads perform filesystem checks while updating commands and beginning a drag

Confirmed direct browser UI path,
all desktop platforms.

[`browser/components/downloads/content/downloads.js:1095-1105`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/downloads/content/downloads.js#L1095-L1105)
handles the download context menu and calls `DownloadsViewController.updateCommands()`.
`DownloadsViewItem.isCommandEnabled()` at lines `1179-1205`
creates `nsIFile` objects and calls `exists()` synchronously to enable Open and Show actions.
It may test both the final target and the `.part` path.
The DOM drag handlers at [`downloads.js:1115-1125`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/downloads/content/downloads.js#L1115-L1125)
and [`browser/components/downloads/content/allDownloadsView.js:827-842`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/downloads/content/allDownloadsView.js#L827-L842)
also call `exists()` synchronously.
Their own comment says this is synchronous because the call occurs in a DOM event.

On Unix,
[`xpcom/io/nsLocalFileUnix.cpp:1890-1897`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/io/nsLocalFileUnix.cpp#L1890-L1897) implements `Exists()` with `access(path, F_OK)`.
On Windows,
[`xpcom/io/nsLocalFileWin.cpp:2927-2937`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/io/nsLocalFileWin.cpp#L2927-L2937) calls `ResolveAndStat()`,
whose metadata path reaches `GetFileAttributesExW` at line `695`.
These metadata operations can wait for an uncached or remote filesystem before the event returns.
No cache-hit assumptions or latency measurements were made.

### Creating a download blocks before the background saver takes over

Confirmed parent-process download setup and helper-dialog paths,
all desktop platforms.

[`uriloader/exthandler/ExternalHelperAppParent.cpp:108-115`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/ExternalHelperAppParent.cpp#L108-L115)
receives the parent-side start message and calls the stream listener's `OnStartRequest()`.
[`nsExternalHelperAppService.cpp:1696-1701`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/nsExternalHelperAppService.cpp#L1696-L1701)
returns early in content processes and calls `SetUpTempFile()` in the parent.
`SetUpTempFile()` at lines `1450-1535`:

- Finds and validates the initial download directory.
- Calls `dummyFile->CreateUnique()` at `1484`.
- Calls `IsExecutable()` and deletes the dummy file at `1488-1489`.
- Calls `mTempFile->CreateUnique()` for the `.part` file at `1498`.
- Only then creates/configures the background saver and calls its `SetTarget()` at `1535`.

The background data writer therefore does not move these initial filesystem operations off the parent event path.
`GetInitialDownloadDirectory()` at `400-424` delegates to directory helpers.
Its temporary-directory helper performs synchronous existence/creation checks
at [`nsExternalHelperAppService.cpp:230-246`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/nsExternalHelperAppService.cpp#L230-L246)
on non-macOS Unix when `browser.download.start_downloads_in_tmp_dir` selects that helper.
The preferred-download-directory helpers contain additional synchronous checks at `268-375`.
`SetDownloadToLaunch()` reserves the final filename with another `CreateUnique()` at line `2694`.

The helper-dialog JS has another directly visible path:
[`toolkit/mozapps/downloads/HelperAppDlg.sys.mjs:306`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/downloads/HelperAppDlg.sys.mjs#L306) and `402`
call `validateLeafName()` from automatic-save and picker-completion code.
Its directory guard at `46-49` uses `exists()`,
`isDirectory()`,
and `isWritable()`.
`validateLeafName()` at `459-479` calls `DownloadPaths.createNiceUniqueFile()`.
[`toolkit/components/downloads/DownloadPaths.sys.mjs:66-82`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/downloads/DownloadPaths.sys.mjs#L66-L82)
performs repeated synchronous existence probes for filename collisions,
with loop condition `i < 10000`,
then synchronously creates the file.
The exact loop bound is source evidence,
not a measured normal workload.

### Opening a downloaded file probes executability synchronously

Confirmed direct parent JS path,
with Unix disk behavior established.

[`browser/components/downloads/DownloadsCommon.sys.mjs:579-584`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/downloads/DownloadsCommon.sys.mjs#L579-L584)
delegates the UI Open action to `download.launch()`.
[`toolkit/components/downloads/DownloadIntegration.sys.mjs:789-826`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/downloads/DownloadIntegration.sys.mjs#L789-L826)
constructs `nsIFile` and calls `file.isExecutable()` before awaiting the confirmation prompt.
An `async` declaration does not move this initial call to another thread.
[`xpcom/io/nsLocalFileUnix.cpp:1941-1947`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/io/nsLocalFileUnix.cpp#L1941-L1947) checks symlink status,
and line `2007` checks `access(path, X_OK)`.
The exact native Launch/Reveal implementation is not counted as a proven disk stall here,
because its platform service may hand work to another process.

### File pickers retain synchronous filesystem work around asynchronous dialogs

Confirmed direct callback paths.

On GTK,
the non-portal picker connects `update-preview` to `UpdateFilePreviewWidget`
at [`widget/gtk/nsFilePicker.cpp:710-711`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/gtk/nsFilePicker.cpp#L710-L711).
The callback performs `stat()` at `111`,
`gdk_pixbuf_get_file_info()` at `118`,
and synchronous `gdk_pixbuf_new_from_file_at_size()` or `gdk_pixbuf_new_from_file()` at `130-133`.
Selecting an image in this picker therefore loads it inside the GTK callback.
This preview finding is limited to the non-portal picker:
`Open()` chooses the portal at `420-426`,
and portal failure can fall back to the non-portal path at `453-463`.
The `stat()` guard rejects non-regular files but does not make regular-file reads asynchronous.

Both GTK portal and non-portal completion reach `DoneCommon()` at `948`.
Save-mode completion calls `Exists()` at `956`;
open-mode completion calls `WarnForNonReadableFile()` at `962`,
which calls `IsReadable()` at `840`.
Unix `IsReadable()` reaches `access(path, R_OK)` at [`xpcom/io/nsLocalFileUnix.cpp:1921`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/io/nsLocalFileUnix.cpp#L1921).
Initial directory validation uses [`widget/nsBaseFilePicker.cpp:416-437`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/nsBaseFilePicker.cpp#L416-L437):
`IsDirectory()` and `IsReadable()` outside its content-process/macOS/Flatpak/Snap bypasses.

Windows picker completion explicitly runs on `GetMainThreadSerialEventTarget()`
at [`widget/windows/nsFilePicker.cpp:783-784`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/windows/nsFilePicker.cpp#L783-L784)
and checks save-target `file->Exists()` at `804`.
The native Windows dialog itself is implemented through asynchronous/remote helpers,
so the finding is the completion metadata check,
not the entire dialog.

macOS [`widget/cocoa/nsFilePicker.mm:604-638`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/cocoa/nsFilePicker.mm#L604-L638)
checks `NSFileManager fileExistsAtPath` in the save-panel completion block;
`PresentSavePanel()` asserts the main thread at line `522`.

### Large clipboard transfers spill synchronously to a temporary file

Confirmed parent-process transferable behavior and browser clipboard entry points.

[`widget/nsTransferable.h:45`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/nsTransferable.h#L45) defines `kLargeDatasetSize = 1000000` bytes.
[`widget/nsTransferable.cpp:60-76`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/nsTransferable.cpp#L60-L76) sends larger primitive data to `WriteCache()`
when running in the parent process and the transfer is not private.
`WriteCache()` calls `NS_OpenAnonymousTemporaryFile()` at `129`
and `PR_Write()` at `138`.
`ReadCache()` performs `PR_GetOpenFileInfo`,
`PR_Seek64`,
and `PR_Read`
at lines `155-168` when the data is requested again.
There is no worker dispatch in these operations.

The browser clipboard helper calls `SetTransferData()` directly
at [`widget/nsClipboardHelper.cpp:89`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/nsClipboardHelper.cpp#L89).
Content clipboard messages reach
[`dom/ipc/ContentParent.cpp:3247-3278`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/ipc/ContentParent.cpp#L3247-L3278),
which creates and fills a parent transferable before handing it to the clipboard.
The direct main-thread content-analysis clipboard path is explicitly asserted
at [`widget/ClipboardContentAnalysisParent.cpp:27`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/ClipboardContentAnalysisParent.cpp#L27) and calls transferable Get/Set at `93-96`.

This finding excludes private/sensitive transfers and content-process `SetData()`.
An anonymous temporary file is file-backed I/O;
whether its backing store is physical disk or memory is host-dependent and was not measured.
It is not a claim that every clipboard operation accesses physical disk.

### Lazy handler settings can synchronously load handlers.json

Confirmed browser Settings/download/protocol path,
conditional on first use racing or preceding preload.

[`uriloader/exthandler/ExtHandlerService.sys.mjs:53-74`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/ExtHandlerService.sys.mjs#L53-L74)
initializes a `JSONFile` for `handlers.json` and calls `ensureDataReady()`.
The comment at `64-65` explicitly preserves this synchronous fallback even if
`asyncInit()` has started but has not completed.
`asyncInit()` at `351-370` offers a real asynchronous preload;
the finding does not assume that preload always loses.

[`toolkit/modules/JSONFile.sys.mjs:300-322`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/modules/JSONFile.sys.mjs#L300-L322)
opens a file input stream,
asks for all available bytes,
and reads/parses them synchronously.
Corrupt-file recovery at `346-362` creates a unique backup path,
removes its placeholder,
and moves the original.
The optional backup-file recovery calls `copyTo()` at `365` before reading the backup at `373-390`.

A directly visible caller is Settings:
[`browser/components/preferences/config/downloads.mjs:797-798`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/preferences/config/downloads.mjs#L797-L798)
iterates `gHandlerService.enumerate()`,
whose [`ExtHandlerService.sys.mjs:373-378`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/ExtHandlerService.sys.mjs#L373-L378) accesses `_store.data.mimeTypes`.
That module is shared with the older Settings view through
[`browser/components/preferences/main.js:1827-1837`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/preferences/main.js#L1827-L1837) and `2105-2106`.
The download helper also calls the handler service in its parent start path
at [`nsExternalHelperAppService.cpp:1774`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/nsExternalHelperAppService.cpp#L1774).

Every saved local application path deserialized through
[`ExtHandlerService.sys.mjs:627-633`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/uriloader/exthandler/ExtHandlerService.sys.mjs#L627-L633) receives a synchronous `file.exists()` check,
independent of whether the JSON preload already completed.
The live implementation is `ExtHandlerService.sys.mjs`;
no RDF handler-store path was found in this implementation.

### Settings application menus check local executables on selection

Confirmed direct UI event path.

[`browser/components/preferences/main.js:2110-2128`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/preferences/main.js#L2110-L2128)
handles a selected application row by calling `rebuildActionsMenu()`.
The menu loops possible application handlers at `2582-2583`.
`isValidHandlerApp()` at `2749-2755` calls `_isValidHandlerExecutable()`
for local handlers;
that function calls `exists()` and `isExecutable()` at `2783-2784`.
The corresponding current settings module has the same checks at
[`browser/components/preferences/config/downloads.mjs:1813-1814`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/preferences/config/downloads.mjs#L1813-L1814).
This can touch several saved application paths while populating the menu.

### Extension installation hashes and changes files synchronously

Confirmed parent JS installation path.

The `about:addons` file-picker callback calls `AddonManager.getInstallForFile`
at [`toolkit/mozapps/extensions/content/aboutaddons-utils.mjs:340-358`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/extensions/content/aboutaddons-utils.mjs#L340-L358).
[`toolkit/mozapps/extensions/internal/XPIInstall.sys.mjs:3383-3393`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/extensions/internal/XPIInstall.sys.mjs#L3383-L3393)
constructs `LocalAddonInstall` for a local XPI and calls `init()`.
`init()` at `2291-2325` checks existence and file size,
then calls `getHashForFile()` before its first `await` at `2336`.
`getHashForFile()` at `836-844`
opens `FileInputStream` and calls `crypto.updateFromStream(fis, file.fileSize)`.
That is a synchronous whole-file read/hash for local installation.
The helper is also called on the staged add-on at line `2079`.

The final on-disk install path at `1958` awaits
`this.location.installer.installAddon(...)`,
but the directory installer's `installAddon()` at `3582-3645` is synchronous.
It moves/copies files through `SafeInstallOperation`,
sets modification time,
and removes the trash directory synchronously.
`SafeInstallOperation` performs `copyTo()` / `moveTo()` at `1144-1151`.
`recursiveRemove()` at `1027-1066` changes permissions,
calls recursive `remove(true)`,
and can enumerate/recurse into directory contents on failure.
This is a source-level responsiveness exposure while installing/replacing/removing extensions,
not just add-on startup.

### Extension startup and sideload scans synchronously load and inspect add-ons

Confirmed startup path,
with an additional explicit rescan entry point.

[`toolkit/mozapps/extensions/internal/XPIProvider.sys.mjs:3344`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/extensions/internal/XPIProvider.sys.mjs#L3344)
calls `XPIStates.scanForChanges()` during `checkForChanges()`.
The scan at `1529-1530` loads extension state via
`loadExtensionState():1494 -> aomStartup.readStartupData()`.
[`toolkit/mozapps/extensions/AddonManagerStartup.cpp:488-496`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/extensions/AddonManagerStartup.cpp#L488-L496)
reads `addonStartup.json.lz4` through `ReadFileLZ4()`;
that helper reaches `URLPreloader::ReadFile()` at `207-210`.
If enabled add-ons need modification checks,
[`AddonManagerStartup.cpp:438-462`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/extensions/AddonManagerStartup.cpp#L438-L462) performs `Exists()`,
file-type tests,
and `GetLastModifiedTime()` synchronously.

The JS scan walks installation directories at [`XPIProvider.sys.mjs:1665`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/extensions/internal/XPIProvider.sys.mjs#L1665),
and `iterDirectory()` at `397-403` is a synchronous `directoryEntries` iterator.
The scope is conditional:
build ID changes set `SCOPE_ALL` at `1550-1557`,
and missing/stale state can broaden the scan at `1563-1628`.
Ordinary unchanged starts can skip locations at `1639-1642`.
`getNewSideloads()` at `3438-3439` calls `scanForChanges(false)`
before awaiting database work.

### Preferences and profile state retain startup and forced-save disk dependencies

Confirmed startup and shutdown/explicit save paths;
normal deferred preference writes are excluded.

[`modules/libpref/Preferences.cpp:4202-4203`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/Preferences.cpp#L4202-L4203)
reads saved preferences and `user.js` during `InitializeUserPrefs()`.
`ReadSavedPrefs()` at `4660-4687` calls `openPrefFile()`
and queries the saved file's modification time.
Invalid preferences cause synchronous backup removal/copy at `4700-4723`.
`openPrefFile()` reaches `URLPreloader::ReadFile()` at `4917`.

[`js/xpconnect/loader/URLPreloader.cpp:502-509`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/js/xpconnect/loader/URLPreloader.cpp#L502-L509)
explicitly falls back to synchronous reads before profile initialization.
`ReadOrWait()` at `657-675` waits on the preload monitor if a read is pending,
or does a synchronous read if cached data was forgotten.
The file implementation reaches synchronous open at
[`xpcom/build/FileLocation.cpp:146-149`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/build/FileLocation.cpp#L146-L149),
metadata at `165-169`,
and `PR_Read()` at `186-190`.
The preloader therefore moves work ahead of demand but does not guarantee a non-blocking consumer.

The default `preferences.allow.omt-write` is `true`
at [`modules/libpref/init/StaticPrefList.yaml:17210-17213`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L17210-L17213).
Normal dirty/default-file writes use the background queue.
Explicit `SavePrefFile(aFile)` with a non-null file forces blocking save
at [`Preferences.cpp:4771-4774`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/Preferences.cpp#L4771-L4774).
`WritePrefFile()` dispatches a `SyncRunnable` at `4874-4875` for that case;
disabling off-main-thread saving writes directly at `4888-4898`.
Actual write operations use safe output streams and `Finish()` at `3414-3464`.
Shutdown/suspend force a blocking save at `4238` and `4246`.
An already-clean forced save can instead spin the event loop while outstanding writes complete
at `3473-3488`;
this is a completion dependency,
not proof of a totally unresponsive event loop.

A non-test explicit-file caller is Firefox profile migration:
[`browser/components/migration/FirefoxProfileMigrator.sys.mjs:202-207`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/migration/FirefoxProfileMigrator.sys.mjs#L202-L207).
That migrator also copies profile data synchronously in its `migrate()` callback at `178-182`,
with more copies at `282`,
`374`,
and `417`.
These are refresh/import/startup operations,
not normal page navigation.

### Update checks and maintenance include synchronous directory writes

Confirmed UI-reachable updater capability probe.

[`browser/base/content/aboutDialog-appUpdater.js:39-75`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/base/content/aboutDialog-appUpdater.js#L39-L75)
creates an `AppUpdater` and starts a check.
After finding an applicable update,
[`toolkit/mozapps/update/AppUpdater.sys.mjs:350`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/update/AppUpdater.sys.mjs#L350) reads `aus.canApplyUpdates`.
[`UpdateService.sys.mjs:4123-4134`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/update/UpdateService.sys.mjs#L4123-L4134)
calls `getCanApplyUpdates()`;
its line `709` calls `testWriteAccess()`.
The latter synchronously checks/removes/creates/removes a file at `548-558`.
Non-Windows/non-macOS platforms additionally test the application directory
through `isAppBaseDirWritable()` at `656-680`.
Staging capability has synchronous test-directory creation/removal at `768-783`;
macOS elevation checks can recursively inspect writable directory entries at `578-586` and `613`.

Update maintenance also synchronously moves log files and recursively removes patch files
at [`UpdateService.sys.mjs:1321-1396`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/update/UpdateService.sys.mjs#L1321-L1396)
and enumerates/removes download-directory contents at `1423-1435`.
These maintenance sinks are confirmed in the parent JS service,
but the exact frequency and UI-state timing of every caller were not measured.

File update logging is conditional,
not a default behavior:
[`toolkit/mozapps/update/UpdateLog.sys.mjs:113-126`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/update/UpdateLog.sys.mjs#L113-L126)
writes and flushes synchronously when `app.update.log.file` is enabled.
That preference is read with default `false` at `60`
and automatically turned off by module startup at `139-168`.
The module's own comment acknowledges startup I/O for this cleanup.

### Other confirmed narrow UI filesystem operations

- [`browser/base/content/browser.js:1362-1377`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/base/content/browser.js#L1362-L1377):
  the remembered open-file directory getter/setter synchronously checks existence/type.
- [`toolkit/mozapps/downloads/DownloadLastDir.sys.mjs:132`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/mozapps/downloads/DownloadLastDir.sys.mjs#L132):
  the remembered download-directory path receives a synchronous existence check.
- [`browser/components/extensions/parent/ext-tabs.js:1445-1458`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/extensions/parent/ext-tabs.js#L1445-L1458):
  `tabs.saveAsPDF` opens/creates/truncates and closes the picked file in the picker callback
  before starting print output.
- [`browser/components/backup/BackupService.sys.mjs:2609-2618`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/backup/BackupService.sys.mjs#L2609-L2618) and `2776-2785`:
  backup extraction opens and seeks the archive file synchronously,
  even though subsequent extraction is asynchronous.
  This establishes open/seek exposure,
  not a synchronous whole-archive copy.
- [`toolkit/profile/nsToolkitProfileService.cpp:1040-1100`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/profile/nsToolkitProfileService.cpp#L1040-L1100):
  profile initialization probes and reads profile/install INI files.
  Synchronous `Flush()` at `2958-2967` calls `FlushData()`;
  `WriteFile()` at `260-278` opens and writes the INI file,
  then finishes its safe output stream.
  The service also provides distinct async flush APIs at `2663` and `2746`;
  those must not be conflated with this synchronous API.

### Rejected overclaims and scope limits

- [`browser/components/sessionstore/SessionFile.sys.mjs:198-220`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/sessionstore/SessionFile.sys.mjs#L198-L220)
  uses `await IOUtils.readUTF8()`;
  its writer path at `422-483` is asynchronous and registers a shutdown blocker.
  The source read does not establish direct UI-thread disk I/O for ordinary session writes.
- [`browser/components/topsites/TopSites.sys.mjs:1269-1276`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/topsites/TopSites.sys.mjs#L1269-L1276)
  calls `readInputStream()` only inside `NetUtil.asyncFetch` completion.
  [`browser/extensions/newtab/lib/RemoteRenderer.sys.mjs:754-766`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/extensions/newtab/lib/RemoteRenderer.sys.mjs#L754-L766) has the same structure.
  Merely finding a stream read there does not establish a disk-blocking event handler.
- [`browser/components/backup/BackupService.sys.mjs:2338-2351`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/backup/BackupService.sys.mjs#L2338-L2351)
  uses `DEFER_OPEN` and `NetUtil.asyncCopy` for extracted-file output.
  Do not report that output stream as a synchronous whole-file write.
- `Services.prefs.savePrefFile(null)` in BrowserGlue/Sanitizer uses the normal asynchronous path
  under the verified default;
  the non-null file and forced-save cases are distinct.
- Presence of a synchronous API establishes a blocking opportunity,
  not a measured stall duration or a claim that a particular user's filesystem was uncached.
- Candidate searches included test files in the first pass;
  production findings were read individually.
- No browser profile,
  preference,
  Firefox source,
  or repository source was modified.
  No runtime stress,
  synthetic load,
  or user-state mutation was performed.

## Storage and security

### Places history interaction executes SQLite statements on the parent main thread

- Trigger:
  opening/searching the classic History sidebar or opening a Places query container.
- Thread:
  browser parent main thread,
  reached from browser chrome JS.
- Chain:
  [`browser/components/places/content/historySidebar.js:118-167`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/places/content/historySidebar.js#L118-L167) creates query options and calls
  `gHistoryTree.load(query, options)`;
  [`treeView.js:66-69`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/browser/components/places/content/treeView.js#L66-L69) opens the root container.
  [`toolkit/components/places/nsNavHistoryQuery.cpp:882`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/places/nsNavHistoryQuery.cpp#L882) defaults `mAsyncEnabled` to false.
  [`nsNavHistoryResult.cpp:529-535`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/places/nsNavHistoryResult.cpp#L529-L535) therefore takes `OpenContainer`,
  `:1594-1603` calls `FillChildren`,
  and `:1715-1727` calls `GetQueryResults`.
  [`nsNavHistory.cpp:1590-1650`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/places/nsNavHistory.cpp#L1590-L1650) gets a synchronous `mozIStorageStatement` and calls `ResultsAsList`;
  `:2274-2287` loops over `ExecuteStep` synchronously.
- Disk endpoint:
  SQLite reads of `places.sqlite` and potentially its WAL;
  mozStorage's `Connection::stepStatement` reaches `sqlite3_step`
  in [`storage/mozStorageConnection.cpp:1760`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/storage/mozStorageConnection.cpp#L1760).
- Important limit:
  bookmark folder result nodes have a separate async path,
  and newer history UIs need their own caller trace.
  This finding is specifically the still-present synchronous Places query-container path.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/places/nsNavHistory.cpp#L1590-L1650>

### Places first connection, migrations, and corruption recovery perform main-thread disk work

- Trigger:
  first Places use during startup or a later first consumer.
- Thread:
  `Database::EnsureConnection` explicitly asserts main thread at
  [`toolkit/components/places/Database.cpp:572-573`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/places/Database.cpp#L572-L573).
- Chain:
  `:602-605` checks the database file;
  `:630-632` calls `OpenUnsharedDatabase`,
  `:665-669` calls `SetupDatabaseConnection` and `InitSchema`,
  `:1076-1184` runs synchronous pragmas and attaches favicons,
  and `:1188` starts synchronous schema initialization/migrations.
  Corruption branches invoke `BackupAndReplaceDatabaseFile` at `:656` and `:708-713`.
- Disk endpoint:
  `places.sqlite`,
  `favicons.sqlite`,
  journal/WAL and recovery files.
  [`storage/mozStorageConnection.cpp:1197-1226`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/storage/mozStorageConnection.cpp#L1197-L1226) contains the SQLite open calls.
- Limit:
  schema migration and corruption replacement are conditional;
  ordinary database opening and initialization are not dependent on corruption.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/places/Database.cpp#L561-L738>

### Cookies can wait for background loading and then reopen SQLite on the parent main thread

- Trigger:
  first non-private cookie operation before initialization finishes.
  [`netwerk/cookie/CookieService.cpp:1738-1759`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cookie/CookieService.cpp#L1738-L1759) calls `EnsureInitialized` when picking persistent storage;
  enumeration and clearing also call it at `:692-706`.
- Thread:
  `CookiePersistentStorage::EnsureInitialized` asserts main thread at `:2033`.
- Chain:
  [`CookiePersistentStorage.cpp:2038-2043`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cookie/CookiePersistentStorage.cpp#L2038-L2043) monitor-waits while the Cookie thread initializes;
  its background task at `:882-925` calls `TryInitDB` and notifies only after completion.
  `TryInitDB` opens the DB at `:969`,
  and `Read` executes the load query at `:1869-1992`.
  Back on main,
  `EnsureInitialized` calls `InitDBConn` at `:2069`;
  `InitDBConnInternal` asserts main at `:2169`,
  opens another SQLite connection at `:2171-2173`,
  then executes synchronous setup pragmas.
- Disk endpoint:
  `cookies.sqlite` and associated files.
- Existing instrumentation:
  `sqlite_cookies_block_main_thread`,
  `sqlite_cookies_time_to_block_main_thread`,
  and `CookiePersistentStorage::InitDBConn` profiler markers.
- Limit:
  after initialization,
  cookie lookups use in-memory state and ordinary persistence uses async statements.
  The presence of async writes does not eliminate this initialization dependency.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cookie/CookiePersistentStorage.cpp#L2032-L2191>

### Permission queries can wait for permissions.sqlite initialization

- Trigger:
  a permission query or update reaches the manager before its initial DB load finishes.
- Thread:
  parent main thread for ordinary permission-manager callers.
- Chain:
  [`extensions/permissions/PermissionManager.cpp:907-958`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/extensions/permissions/PermissionManager.cpp#L907-L958) dispatches initialization to its DB thread.
  That task holds `mMonitor` while `TryInitDB` opens,
  migrates and reads `permissions.sqlite`.
  `:884-901` opens the DB off-main;
  `:3446-3512` reads persisted permission rows.
  Main-thread APIs call `EnsureReadCompleted` (for example `:1969-1974`);
  `:4009-4014` waits on the monitor while state is `eInitializing`.
  Acquiring that same monitor can itself wait behind the initializing task before reaching the explicit wait.
- Limit:
  ready-state permission checks use the loaded data.
  Do not label every permission check a disk read.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/extensions/permissions/PermissionManager.cpp#L907-L958>
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/extensions/permissions/PermissionManager.cpp#L4009-L4039>

### Bounce tracking storage can block main-thread user-activation handling on its first disk load

- Trigger:
  recording user activation or other storage access before bounce-protection initialization completes.
- Chain:
  [`toolkit/components/antitracking/bouncetrackingprotection/BounceTrackingProtection.cpp:396-435`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/antitracking/bouncetrackingprotection/BounceTrackingProtection.cpp#L396-L435)
  records user activation;
  [`BounceTrackingStateGlobal.cpp:30-67`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/antitracking/bouncetrackingprotection/BounceTrackingStateGlobal.cpp#L30-L67) calls `UpdateDBEntry`.
  [`BounceTrackingProtectionStorage.cpp:173-184`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/antitracking/bouncetrackingprotection/BounceTrackingProtectionStorage.cpp#L173-L184) asserts main thread and calls `WaitForInitialization`;
  `:578-584` monitor-waits.
  Its background task `:713-736` holds the monitor through `CreateDatabaseConnection`
  and `LoadMemoryStateFromDisk`;
  `:739-748` opens the SQLite database.
- ESR default exposure:
  [`modules/libpref/init/StaticPrefList.yaml:17986-17992`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L17986-L17992) sets non-Nightly mode to `3`.
  [`nsIBounceTrackingProtection.idl:12-40`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/antitracking/bouncetrackingprotection/nsIBounceTrackingProtection.idl#L12-L40) defines `3` as `MODE_ENABLED_DRY_RUN`:
  activation collection and classification run;
  tracker purging is simulated.
  It is not fully disabled and it is not standby mode.
- Limit:
  private data is not persisted by `UpdateDBEntry`;
  fully disabling mode is a separate configuration.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/antitracking/bouncetrackingprotection/BounceTrackingProtectionStorage.cpp#L578-L591>

### HSTS and other security DataStorage reads can synchronously await their backing-file load

- Concrete main-thread trigger:
  speculative connection.
  [`netwerk/protocol/http/nsHttpHandler.cpp:2480-2515`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/protocol/http/nsHttpHandler.cpp#L2480-L2515)
  asserts main thread and directly calls `IsSecureURI`.
- Chain:
  [`security/manager/ssl/nsSiteSecurityService.cpp:722-753`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsSiteSecurityService.cpp#L722-L753) checks host HSTS state;
  `:833` calls `mSiteStateStorage->Get`.
  [`security/manager/ssl/data_storage/src/lib.rs:757-760`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/data_storage/src/lib.rs#L757-L760) calls `wait_for_ready`;
  `:824-829` waits on a condition variable.
  Initialization is dispatched in `:981-988`,
  holds the data mutex in `:834-840`,
  and reads `SiteSecurityServiceState.bin` or its older text format in `:294-328` and `:417-425`.
- Additional exposure:
  the memory reporter at `:842-853` takes the same data mutex,
  so memory reporting can wait behind initialization's disk read too.
- Important mitigation:
  [`netwerk/base/nsNetUtil.cpp:3087-3139`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/base/nsNetUtil.cpp#L3087-L3139) explicitly offloads a cold-storage HSTS check
  when it has the socket transport service and a result callback.
  Do not claim every navigation performs this cold HSTS wait on main.
  The speculative-connect caller is direct,
  and the fallback at `:3141` is also synchronous.
- Limit:
  data is resident in Rust maps/vectors after initialization;
  steady-state writes are queued to background tasks.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/data_storage/src/lib.rs#L757-L840>
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/protocol/http/nsHttpHandler.cpp#L2480-L2515>

### Certificate-state async APIs share a lock held across disk work

- Trigger:
  remote security settings submit certificate/revocation work while another certificate-storage task
  owns the security-state lock,
  or a memory report runs during that work.
- Main-thread entry example:
  [`security/manager/ssl/cert_storage/src/lib.rs:1320-1337`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/cert_storage/src/lib.rs#L1320-L1337)
  requires main thread for `HasPriorData`,
  then constructs `SecurityStateTask`.
  Remote settings calls that API in [`security/manager/ssl/RemoteSecuritySettings.sys.mjs:119-129`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/RemoteSecuritySettings.sys.mjs#L119-L129).
- Chain:
  `SecurityStateTask::new` takes `security_state.write()` at
  `security/manager/ssl/cert_storage/src/lib.rs:1200`.
  Worker `run` takes the same write lock at `:1216` and keeps it through `open_db` and `task_action`
  at `:1221-1229`.
  `SecurityState::open_db` at `:216-239` opens/migrates the store.
  Persisting tasks call `writer.commit()` (for example `:380-401`).
  The selected backend is `rkv::SafeMode` at `:43`,
  `:74` and `:1043-1057`.
  [`third_party/rust/rkv/src/backend/impl_safe/transaction.rs:184-193`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/rkv/src/backend/impl_safe/transaction.rs#L184-L193) calls `write_to_disk` on commit;
  [`environment.rs:266-278`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/rkv/src/backend/impl_safe/environment.rs#L266-L278) performs filesystem metadata,
  serialized-file write and rename.
- Memory-report variant:
  [`cert_storage/src/lib.rs:1698-1701`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/cert_storage/src/lib.rs#L1698-L1701) obtains a read lock;
  the report is dispatched to main by [`xpcom/base/nsMemoryReporterManager.cpp:1983-2006`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/base/nsMemoryReporterManager.cpp#L1983-L2006).
- Consequence:
  an async-looking API can synchronously wait on the main thread while a prior worker task
  holds the lock over filesystem work.
  This can recur after startup.
- Limit:
  overlap is required;
  no runtime contention duration was measured.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/cert_storage/src/lib.rs#L1193-L1241>
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/rkv/src/backend/impl_safe/environment.rs#L266-L278>

### NSS profile initialization opens security databases on the parent main thread

- Trigger:
  NSS initialization for the parent process.
- Chain:
  [`security/manager/ssl/nsNSSComponent.cpp:1631-1645`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsNSSComponent.cpp#L1631-L1645) asserts both main thread and parent process,
  then calls `InitializeNSS`.
  `:1534` calls `InitializeNSSWithFallbacks`;
  `:1333-1345` attempts read/write then read-only NSS initialization for the profile.
  [`security/certverifier/NSSCertDBTrustDomain.cpp:1543-1584`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/certverifier/NSSCertDBTrustDomain.cpp#L1543-L1584) explicitly requires the parent main thread,
  constructs a `sql:` profile path,
  and calls `NSS_Initialize`.
  [`security/nss/lib/softoken/sdb.c:730`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/nss/lib/softoken/sdb.c#L730) opens SQLite;
  `:2547` and `:2564` initialize certificate and key databases.
- Additional metadata I/O:
  [`nsNSSComponent.cpp:1157-1182`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsNSSComponent.cpp#L1157-L1182) invokes `statfs` on Linux profile storage;
  Windows uses volume-path/drive queries in the same helper.
- Guards:
  `security.nocertdb` defaults false ([`StaticPrefList.yaml:18543-18546`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L18543-L18546)).
  Utility-process PKCS#11 support does not remove the parent's certificate/key DB initialization;
  [`NSSCertDBTrustDomain.cpp:1568-1577`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/certverifier/NSSCertDBTrustDomain.cpp#L1568-L1577) distinguishes module DB and certificate DB flags.
- Limit:
  built-in roots loading has its own background task;
  do not describe all certificate work as main-thread.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/certverifier/NSSCertDBTrustDomain.cpp#L1543-L1584>

### Certificate manager import, export, and list operations can block the parent UI

- Trigger:
  opening Certificate Manager;
  importing a CA/email certificate;
  importing or backing up a personal cert.
- JS entry:
  [`security/manager/pki/resources/content/certManager.js:322`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/pki/resources/content/certManager.js#L322) calls `certdb.getCerts()`;
  `:666` exports PKCS#12,
  `:757` imports PKCS#12,
  and `:855` imports a certificate file.
- Direct file import:
  [`security/manager/ssl/nsNSSCertificateDB.cpp:760-791`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsNSSCertificateDB.cpp#L760-L791)
  opens an NSPR file descriptor,
  stats it,
  reads the entire file via `PR_Read`,
  then closes it synchronously.
- PKCS#12:
  `:811-851` explicitly requires main thread and calls `nsPKCS12Blob`.
  [`nsPKCS12Blob.cpp:198`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsPKCS12Blob.cpp#L198) opens output synchronously;
  `:241-249` opens and reads input;
  `:330` calls `PR_Write` for output.
- Listing:
  [`nsNSSCertificateDB.cpp:1173-1186`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsNSSCertificateDB.cpp#L1173-L1186) waits for loadable certs and calls `PK11_ListCerts`.
  [`nsNSSComponent.cpp:554-566`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsNSSComponent.cpp#L554-L566) implements the loadable-cert monitor wait.
- Limit:
  listing may be satisfied from loaded state;
  import/export have explicit filesystem endpoints.
  This finding applies to these certificate UI operations,
  not ordinary TLS verification generally.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsNSSCertificateDB.cpp#L760-L851>

### Certificate override service synchronously loads cert_override.txt

- Trigger:
  first service initialization/profile change,
  potentially on first consumer.
- Chain:
  [`security/manager/ssl/nsCertOverrideService.cpp:158-175`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsCertOverrideService.cpp#L158-L175)
  rejects non-main-thread initialization and invokes its profile-change observer.
  `:184-198` holds `mMutex` and calls `Read`.
  `:227-257` opens a local file input stream and loops over synchronous `ReadLine` calls.
- Limit:
  writes are not the same finding:
  `:347-356` dispatches a `WriterRunnable` to `mWriterTaskQueue`.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsCertOverrideService.cpp#L158-L257>

### localStorage cold access blocks the content main thread on quota and SQLite work

- Trigger:
  a page calls `localStorage.getItem`,
  `setItem`,
  `length`,
  or similar before its datastore is ready.
- Chain:
  [`dom/localstorage/LSObject.cpp:507-609`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/localstorage/LSObject.cpp#L507-L609) calls `EnsureDatabase`;
  `:908-915` sends a datastore-preparation request through `DoRequestSynchronously`.
  `:838-856` explicitly documents synchronously blocking the owning thread;
  `RequestHelper::AssertIsOnOwningThread` asserts main thread at `:170`.
  `StartAndReturnResponse` dispatches to `RemoteLazyInputStreamThread` and monitor-waits at `:1048-1100`.
  Parent [`dom/localstorage/ActorsParent.cpp:7009-7046`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/localstorage/ActorsParent.cpp#L7009-L7046) sends preparation to quota IO;
  `:7043-7323` checks directories and prepares the datastore,
  then `LoadDataOp::DoDatastoreWork` at `:7822-7863` reads `SELECT ... FROM data` on the connection thread.
- Scope:
  this is normally a page/content-process main-thread stall,
  not proof that browser chrome freezes.
- Limits:
  an existing datastore is reused;
  private browsing skips disk preparation at `dom/localstorage/ActorsParent.cpp:7025-7028`.
  Writes can be buffered after loading;
  do not claim every `setItem` waits for a physical disk flush.
  The cancellation deadline is `50000` ms ([`LSObject.cpp:65`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/localstorage/LSObject.cpp#L65)),
  but cancellation can then wait indefinitely at `:1087-1091`,
  so it is not a hard upper bound on a stall.
- Default:
  the unsupported legacy implementation pref defaults false ([`StaticPrefList.yaml:5300-5304`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L5300-L5304)).
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/localstorage/LSObject.cpp#L1048-L1100>

### Cache memory reporting waits synchronously for the cache IO thread

- Trigger:
  a memory report,
  including an `about:memory` report.
- Thread:
  `nsMemoryReporterManager::DispatchReporter`
  dispatches `CollectReports` onto main ([`xpcom/base/nsMemoryReporterManager.cpp:1983-2006`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/base/nsMemoryReporterManager.cpp#L1983-L2006)).
- Chain:
  [`netwerk/cache2/CacheStorageService.cpp:2423-2428`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheStorageService.cpp#L2423-L2428)
  calls `CacheFileIOManager::SizeOfIncludingThis` while holding its service lock.
  [`CacheFileIOManager.cpp:4633-4642`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheFileIOManager.cpp#L4633-L4642) calls `SizeOfHandlesRunnable::Get`.
  `:4580-4597` dispatches to the cache IO thread and monitor-waits for completion.
- Disk dependency:
  the target is the cache IO thread,
  whose work includes synchronous file operations;
  `ReadInternal` calls `PR_Read` at `:2071` in non-async builds,
  and file opens/metadata operations remain on that thread.
  A report can queue behind a blocked operation even though measuring handle memory does not itself read disk.
- Limit:
  queue overlap is necessary;
  runtime stall duration has not been measured.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheFileIOManager.cpp#L4568-L4600>

### Cache shutdown waits for IO and can fall back to main-thread recursive deletion

- Trigger:
  shutdown,
  with additional deletion when clear-cache-on-shutdown is configured.
- Chain:
  [`netwerk/cache2/CacheFileIOManager.cpp:1226-1255`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheFileIOManager.cpp#L1226-L1255) asserts main thread,
  invokes `ShutdownEvent::PostAndWait`,
  then `CacheIOThread::Shutdown`.
  [`CacheFileIOManager.cpp:623-648`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheFileIOManager.cpp#L623-L648) spins an event loop waiting for the IO thread's shutdown event;
  [`CacheIOThread.cpp:261-274`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheIOThread.cpp#L261-L274) calls `PR_JoinThread`.
- Distinction:
  the `SpinEventLoopUntil` phase pumps events;
  it delays shutdown completion but is not equivalent to a non-pumping UI freeze.
  The later join is a thread wait.
- Clear-on-shutdown:
  `SyncRemoveAllCacheFiles` at `:4378-4426` attempts background-process purge plus a
  synchronous cache-directory rename.
  If background purge is disabled or the attempt fails,
  `:4429-4455` calls `SyncRemoveDir` for entries/doomed/trash;
  `SyncRemoveDir` at `:4305-4338` performs recursive removal.
- Limit:
  mainly exit responsiveness.
  Do not generalize this to mid-session browsing.
- Source: <https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cache2/CacheFileIOManager.cpp#L4378-L4455>

### Preference-gated candidates, excluded from default ESR findings

#### Unused-permission expiration

[`PermissionManager.cpp:4933-4987`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/extensions/permissions/PermissionManager.cpp#L4933-L4987) asserts main thread and synchronously dispatches a full
`moz_origin_interactions` SELECT to its worker via `SyncRunnable::DispatchToThread`.
This runs from the idle-daily maintenance continuation (`:1707-1741`).
However,
`permissions.expireUnused.enabled` is `@IS_NIGHTLY_BUILD@`
in [`StaticPrefList.yaml:17027-17030`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L17027-L17030),
so it is off by default in ESR.

#### Identity credential storage

[`toolkit/components/credentialmanagement/IdentityCredentialStorageService.cpp:216-222`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/credentialmanagement/IdentityCredentialStorageService.cpp#L216-L222)
waits on main-thread initialization;
`:190-211` opens/loads its disk DB on a worker holding the monitor.
APIs including `GetState` (`:665-674`) and `Clear` (`:841-845`) reach the wait.
The public identity feature defaults false ([`StaticPrefList.yaml:5019-5022`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L5019-L5022)).
Clear-data consumers are also guarded by that pref
([`toolkit/components/cleardata/ClearDataService.sys.mjs:2045-2066`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/cleardata/ClearDataService.sys.mjs#L2045-L2066)).
Do not claim clearing ordinary data enables this path in a default ESR profile.

#### SQLite at-rest encryption

[`storage/SQLiteEncryption.cpp:407-456`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/storage/SQLiteEncryption.cpp#L407-L456) holds a process-wide state mutex while opening its keystore
and retrieving/creating database keys.
The master switch `security.storage.encryption.sqlite.enabled` defaults false
([`StaticPrefList.yaml:18927-18934`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libpref/init/StaticPrefList.yaml#L18927-L18934)) and is explicitly marked internal/do-not-enable in source.
This is not default-ESR disk work.

### Async paths that should not be promoted from grep hits alone

- [`dom/indexedDB/IDBFactory.cpp:785-808`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/indexedDB/IDBFactory.cpp#L785-L808) returns an `IDBOpenDBRequest`;
  `:851` sends the background factory request.
  Worker SQLite I/O alone does not demonstrate a UI-thread wait.
- Cookie persistence uses async statements after loading
  ([`CookiePersistentStorage.cpp:714-774`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/cookie/CookiePersistentStorage.cpp#L714-L774)),
  despite the initialization findings.
- Certificate override writes dispatch to a writer queue ([`nsCertOverrideService.cpp:347-356`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/security/manager/ssl/nsCertOverrideService.cpp#L347-L356)).
- The normal cold HSTS upgrade check has the callback-based mitigation described in the HSTS finding.
- LocalStorage normal datastore reuse/private paths must remain distinct from first persistent load.
- Cache shutdown's event-loop-spinning phase is a completion dependency,
  not an unconditional frozen event loop.

## Graphics, resources, fonts, and telemetry

### Main-thread startup-cache lock held by disk writer

Confirmed source dependency,
parent and other processes using StartupCache.
[`startupcache/StartupCache.cpp:386`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/startupcache/StartupCache.cpp#L386) asserts that `GetBuffer` runs on the main thread;
line 393 acquires `mTableLock` with a blocking lock.
`MaybeWriteOffMainThread`,
lines 796 to 805,
dispatches a worker which holds that same mutex for `WriteToDisk`.
`WriteToDisk`,
lines 525 to 620,
opens the profile startup cache with `PR_WRONLY | PR_CREATE_FILE | PR_TRUNCATE`,
compresses and writes entries,
seeks,
writes the header,
and closes while the caller retains the mutex.
Thus a main-thread cache lookup can wait for disk work even though the write was dispatched to a worker.
`HasEntry` also takes the same blocking lock at lines 375 to 378.

This can occur after the startup write timer fires and another resource/module is requested,
and during shutdown.
The current source itself documents the shutdown manifestation:
lines 685 to 690 say a starved writer holding `mTableLock` blocks main-thread `GetBuffer`
and cite Mozilla bug `2037923`.
The release prevents low I/O priority for the shutdown write;
it does not remove the shared-lock dependency.
`PutBuffer` is deliberately different:
lines 470 to 476 use a try-lock and fail instead of waiting.

### Startup-cache and omnijar file-backed page faults

Confirmed mappings and synchronous consumers;
physical disk faults are conditional.
[`startupcache/StartupCache.cpp:273`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/startupcache/StartupCache.cpp#L273) asserts `LoadArchive` is main-thread-only.
It calls `mCacheData.init(mFile)` at line 277,
then reads the mapped magic/header/index at lines 291 to 357.
[`js/xpconnect/loader/AutoMemMap.cpp:31-86`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/js/xpconnect/loader/AutoMemMap.cpp#L31-L86) performs
`OpenNSPRFileDesc`,
`PR_GetOpenFileInfo64`,
`PR_CreateFileMap`,
and `PR_MemMap`.
`GetBuffer`,
[`StartupCache.cpp:415-442`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/startupcache/StartupCache.cpp#L415-L442),
decompresses directly from mapped compressed bytes on the main thread.

[`modules/libjar/nsZipArchive.cpp:187-246`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/libjar/nsZipArchive.cpp#L187-L246) opens the archive,
queries its size,
and creates a read-only file mapping.
Unix `MADV_WILLNEED` at lines 218 to 223 requests prefetch;
it does not make subsequent accesses independent of backing-file residency.
[`xpcom/build/Omnijar.cpp:80`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/build/Omnijar.cpp#L80) opens the `omni.ja` archive with `nsZipArchive::OpenArchive`.
Synchronous module,
stylesheet,
and localization readers consume its mapped data.

These are read-only file mappings,
unlike a hypothetical continuously updated disk-backed metrics mapping.
A warm resident page does not require disk I/O on every access.
Cold or reclaimed mapped pages can fault when the UI thread reads them.
The source does not establish that a major fault occurred on this machine.

`StartupCache::InvalidateCache`,
lines 623 to 667,
also blocks for prefetch completion,
acquires the writer mutex,
removes the cache file,
and reloads it.
`EnsureShutdownWriteComplete`,
lines 693 to 715,
can synchronously write the cache as a fallback.
These are separate invalidation/shutdown triggers rather than evidence of continuous browsing writes.

### Synchronous chrome ES module loading

Confirmed direct main-thread file/resource access on cache misses.
[`js/xpconnect/loader/mozJSModuleLoader.cpp:721-764`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/js/xpconnect/loader/mozJSModuleLoader.cpp#L721-L764)
contains the `ChromeUtils.importESModule` synchronous load path.
The off-main-thread branch returns at lines 731 to 733;
the remaining branch calls `GetScriptForLocation` on the main thread.
`GetScriptForLocation`,
lines 870 to 989,
tries the script preloader and startup cache,
then either maps a real file or calls `ReadScript`.
`ReadScript`,
lines 830 to 856,
uses synchronous channel `Open`,
`Available`,
and `Read`.

Trigger:
first use of a chrome module,
including a module lazily imported by a UI feature,
when its in-memory/bytecode cache does not satisfy the request.
Packaged builds normally read chrome resources from `omni.ja`;
the real-file mapping branch is especially relevant to local/unpacked builds.
The current source intentionally skips bytecode caching for modules outside the app/gre omnijars
to avoid stale built-in-addon bytecode,
lines 903 to 907.
The separately implemented `AsyncOpen` module load path is not evidence that these synchronous imports are async.

### Script-preloader startup metadata, mappings, and decode waits

Confirmed startup disk work and main-thread dependency on mapped decode input.
[`js/xpconnect/loader/ScriptPreloader.cpp:478-511`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/js/xpconnect/loader/ScriptPreloader.cpp#L478-L511)
creates the startup-cache directory,
checks the cache file,
renames `.bin` to `-current.bin`,
and maps it.
The parent initialization path calls `OpenCache` at line 538;
the child initialization path maps the inherited file at line 590.
`InitCacheInternal`,
lines 597 to 674,
reads the mapped header and assigns mapped XDR ranges to cached scripts.

`WaitForCachedStencil`,
lines 1113 to 1162,
can either decode a small script immediately on the caller or wait on a monitor until a background decode finishes.
The worker's required input is the file mapping,
so a background page fault can extend this synchronous wait.
The source explicitly calls it a blocked main thread in the notification callbacks at lines 1166 to 1190.
This is conditional on the cache,
decode state,
and off-thread parsing configuration.
It is not evidence that all waits are disk-bound;
decoding CPU work is another dependency.

### Synchronous localization and stylesheet reads through URLPreloader

Confirmed main-thread localization I/O;
synchronous internal-style and Fluent paths also exist.
[`intl/strres/nsStringBundle.cpp:425-475`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/intl/strres/nsStringBundle.cpp#L425-L475)
asserts main-thread initialization of a string bundle,
calls `URLPreloader::ReadURI`,
and falls back to synchronous channel `Open`.
[`js/xpconnect/loader/URLPreloader.cpp:480-492`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/js/xpconnect/loader/URLPreloader.cpp#L480-L492)
directly reads when the startup preload cache is unavailable or startup has finished.
`URLEntry::ReadOrWait`,
lines 657 to 675,
waits on a monitor for a pending background read,
or synchronously reads again if the entry's data was discarded.

The file sink is [`xpcom/build/FileLocation.cpp:146-151`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/build/FileLocation.cpp#L146-L151)
(`OpenNSPRFileDesc`),
lines 165 to 181 (size query),
and lines 186 to 202 (`PR_Read` loop).
Archive entries instead read from the mapped zip archive.
[`URLPreloader.cpp:642-651`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/js/xpconnect/loader/URLPreloader.cpp#L642-L651) requests the size and copies the whole resource.

[`intl/l10n/L10nRegistry.cpp:344-359`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/intl/l10n/L10nRegistry.cpp#L344-L359) has a separate `LoadSync` Fluent-resource path.
Its ordinary `Load` method at lines 326 to 340 uses a stream loader and should not be labeled synchronous.
[`layout/style/Loader.cpp:1169-1215`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/style/Loader.cpp#L1169-L1215)
uses `NewStyleSheetChannel(..., UsePreload::Yes, ...)` for synchronous stylesheet loads;
the preload read is at lines 1134 to 1143.
These paths matter when opening UI that first requires localized strings or internal styles.
They do not mean all web-page CSS or all Fluent loads block the browser UI on disk.

### WebRender shader-cache reads block the render thread

Confirmed direct frame-critical disk reads;
not a browser-main-thread operation.
[`gfx/webrender_bindings/RenderThread.cpp:1269`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/webrender_bindings/RenderThread.cpp#L1269) asserts the render thread for `InitDeviceTask`.
At lines 1282 to 1284 it constructs `WebRenderProgramCache` when program-binary disk caching is enabled.
The constructor,
lines 1602 to 1612,
calls `wr_program_cache_new` and `wr_try_load_startup_shaders_from_disk` synchronously.

[`gfx/webrender_bindings/src/program_cache.rs:313-325`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/webrender_bindings/src/program_cache.rs#L313-L325)
creates the `shader-cache` directory.
Lines 222 to 254 read the `startup_shaders` file and enumerate the cache directory.
`deserialize_program_binary`,
lines 27 to 30,
opens a shader binary and reads the entire file.
The load budget is `MAX_LOAD_TIME_MS = 400` at line 17,
but lines 257 to 269 check elapsed time only after each completed shader load.
It does not interrupt a single slow open,
directory read,
or file read.

This is also an on-demand rendering path:
[`gfx/wr/webrender/src/device/gl.rs:2545-2553`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/wr/webrender/src/device/gl.rs#L2545-L2553)
asks the cache observer to load a missing shader from disk when linking a program.
[`program_cache.rs:295-299`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/webrender_bindings/src/program_cache.rs#L295-L299) forwards that call to the synchronous reader.
The initial timeout clears the remaining disk-filename list,
but when startup loading stays within its budget the remaining entries are available for demand loading.
Source proves that frame production can wait here,
not that input-event dispatch itself waits on disk.

Availability is gated by `Feature::WEBRENDER_SHADER_CACHE`
([`gfx/thebes/gfxPlatform.cpp:2649-2650`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxPlatform.cpp#L2649-L2650)),
the `gfx.webrender.program-binary-disk` preference,
and the graphics blocklist ([`gfx/config/gfxConfigManager.cpp:289-299`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/config/gfxConfigManager.cpp#L289-L299)).
Shader writes and startup-whitelist writes use Rayon workers at
[`program_cache.rs:136-170`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/webrender_bindings/src/program_cache.rs#L136-L170) and `189` to `194`;
they are not the direct render-thread read finding.

### Linux first-use system-font loading

Confirmed synchronous file-oriented font creation during font selection/metrics/shaping.
[`gfx/thebes/gfxFcPlatformFontList.cpp:372-384`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxFcPlatformFontList.cpp#L372-L384)
extracts `FC_FILE` and `FC_INDEX` and calls `Factory::NewSharedFTFace` with the filename.
`GetFTFace`,
lines 1080 to 1092,
does this lazily on the first request.
It is used from font instance creation around lines 993 to 1076,
table detection at line 634,
and table copying at line 1191.
The text-run path calls `gfxFontEntry::FindOrMakeFont`
([`gfx/thebes/gfxTextRun.cpp:2078`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxTextRun.cpp#L2078),
`2162`,
and `4031`),
which calls the platform `CreateFontInstance` on a cache miss
([`gfx/thebes/gfxFontEntry.cpp:242-250`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxFontEntry.cpp#L242-L250)).
These operations are reachable while browser chrome or content text is being laid out;
they are not limited to font enumeration at startup.

[`gfx/2d/Factory.cpp:536-552`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/2d/Factory.cpp#L536-L552)
holds the global FreeType mutex while calling `FT_New_Face` with the font filename.
The in-tree FreeType file implementation
([`modules/freetype2/src/base/ftsystem.c:242-282`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/modules/freetype2/src/base/ftsystem.c#L242-L282))
opens the file and seeks to determine its length;
its read callback at lines 222 to 235 seeks and reads synchronously.
System-FreeType builds depend on the linked library implementation,
but Firefox still makes the synchronous filename-based call.
The shared FreeType lock can also transmit a worker's file-open delay to another thread needing that lock.

[`gfx/2d/UnscaledFontFreeType.cpp:23-49`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/2d/UnscaledFontFreeType.cpp#L23-L49)
is an additional explicit open/stat/mmap/callback/unmap path for serializing a font file.
Do not claim it occurs for every text draw:
[`gfx/layers/wr/WebRenderBridgeChild.cpp:317-318`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/layers/wr/WebRenderBridgeChild.cpp#L317-L318)
prefers `GetFontDescriptor` and only falls back to `GetFontFileData` when no descriptor is available.

Windows DirectWrite and macOS CoreText calls have synchronous font-creation/table-access boundaries
([`gfx/thebes/gfxDWriteFontList.cpp:764-790`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxDWriteFontList.cpp#L764-L790),
`gfx/thebes/CoreTextFontList.cpp`),
but their OS-internal disk activity was not fully traced.
Treat those as native-library candidates,
not equivalent source-confirmed file reads.

### Linux periodic fontconfig checks on the parent thread

Confirmed Firefox synchronous native-library boundary;
disk implementation verified in current upstream fontconfig.
[`gfx/thebes/gfxFcPlatformFontList.cpp:1496-1508`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxFcPlatformFontList.cpp#L1496-L1508)
creates the parent's repeating font-update timer when fontconfig's rescan interval is nonzero.
`CheckFontUpdates`,
lines 2887 to `2907`,
calls `FcInitBringUptoDate`,
then can rebuild the font list and force a global reflow.
[`gfxPlatformFontList.cpp:730-732`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxPlatformFontList.cpp#L730-L732)
asserts that font-list reinitialization runs on the main thread.

Current first-party fontconfig source:
<https://github.com/fontconfig/fontconfig/blob/b6cf4b270c42b13347a4012cdceb9bbc3d2e8f32/src/fcinit.c#L303>
checks the rescan interval and calls `FcConfigUptoDate`,
then `FcInitReinitialize` when stale.
<https://github.com/fontconfig/fontconfig/blob/b6cf4b270c42b13347a4012cdceb9bbc3d2e8f32/src/fccfg.c#L253>
shows `FcConfigNewestFile` issuing `FcStat` across config files,
config directories,
font directories,
and application fonts.
Thus periodic metadata work can occur even without a visible font picker.
`rpm --query fontconfig gtk3 freetype` reports installed x86_64 and i686 packages
`fontconfig-2.18.3-2.fc45`,
`gtk3-3.24.52-6.fc45`,
and `freetype-2.14.3-3.fc45`.
This is package inventory,
not proof of which library a running browser mapped.
The system rescan interval and distribution-specific library patches were not measured;
do not assign a fixed timer period or claim this path executes on every installation.

Initial font-list loading can be off-main-thread under
[`gfxPlatformFontList.cpp:290-304`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxPlatformFontList.cpp#L290-L304) preference/safe-mode conditions.
That mitigation does not make subsequent main-thread font-list reinitialization async.

### Linux native-theme changes can synchronously load GTK files

Confirmed Firefox main-thread GTK calls,
conditional upstream GTK file I/O.
[`widget/gtk/nsLookAndFeel.cpp:1753-1784`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/gtk/nsLookAndFeel.cpp#L1753-L1784)
asserts main-thread initialization and invokes theme restoration and alternate-theme configuration.
Lines 1413 to 1415 and 1454 to 1477
synchronously set `gtk-theme-name` and/or `gtk-application-prefer-dark-theme`.

Current GTK `gtk-3-24` source confirms the external boundary:
<https://github.com/GNOME/gtk/blob/bac4a949b151eedada8bc60a97429d2176b0b28b/gtk/gtksettings.c#L2154>
routes these property notifications to `settings_update_theme`;
line 3321 calls `_gtk_css_provider_load_named`,
then probes and may load per-theme `settings.ini`.
<https://github.com/GNOME/gtk/blob/bac4a949b151eedada8bc60a97429d2176b0b28b/gtk/gtkcssprovider.c#L2123>
first checks built-in resources;
the external-theme branch probes theme directories,
loads `gtk.gresource`,
and calls `gtk_css_provider_load_from_path` at lines 2153 to 2166.
Its `load_internal` calls `gtk_file_load_bytes` at line 1742.

Trigger:
initial native-theme setup or theme/color-scheme changes that require loading a disk-installed GTK theme.
Built-in GTK resource themes follow a resource branch;
cached files can already be resident.
The precise file behavior depends on the linked GTK version and selected theme,
not solely the Firefox commit.
Both external repositories' branch heads were resolved through their GitLab commit APIs;
each cited file was fetched again by immutable commit and compared byte-for-byte with the inspected copy.

### Custom ICC display profile read during graphics initialization

Confirmed conditional direct main-thread file read.
[`gfx/thebes/gfxPlatform.cpp:808`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxPlatform.cpp#L808) asserts main-thread graphics initialization;
line 987 calls `InitializeCMS`.
`GetPrefCMSOutputProfileData`,
lines 2094 to `2112`,
checks `gfx.color_management.display_profile`
and calls `qcms_data_from_path` when it names a file.
The GTK platform requests this override before system-profile lookup
([`gfx/thebes/gfxPlatformGtk.cpp:510-514`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/gfxPlatformGtk.cpp#L510-L514)).
[`gfx/qcms/src/c_bindings.rs:201-212`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/qcms/src/c_bindings.rs#L201-L212) uses `fopen`,
and the reader at lines 135 and 160 calls `fread`.

This is a custom-profile/startup path,
not an unconditional read on every paint.
The default empty preference returns immediately.
System-profile retrieval differs by platform and is not all file I/O.

### Glean memory reports can wait for telemetry database persistence

Confirmed shared-lock dependency from main-thread reporting to off-thread disk writes.
Ordinary Glean metric updates are dispatched:
[`third_party/rust/glean-core/src/metrics/counter.rs:113-115`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/metrics/counter.rs#L113-L115)
calls `launch_with_glean`;
[`glean-core/src/lib.rs:182`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/lib.rs#L182) runs it on the dispatcher.
However,
the closure executes while holding the global Glean mutex
([`glean-core/src/core/mod.rs:81-87`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/core/mod.rs#L81-L87)).
Ping-lifetime persistence runs within such a closure
([`glean-core/src/lib.rs:774-779`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/lib.rs#L774-L779)).
[`glean-core/src/database/mod.rs:826-847`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/database/mod.rs#L826-L847)
writes the ping store and commits it.

Glean uses `rkv::backend::SafeModeEnvironment`
([`glean-core/src/database/mod.rs:51-56`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/database/mod.rs#L51-L56)),
not an LMDB disk mapping.
[`third_party/rust/rkv/src/backend/impl_safe/transaction.rs:184-193`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/rkv/src/backend/impl_safe/transaction.rs#L184-L193)
commits via `write_to_disk`;
[`impl_safe/environment.rs:266-277`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/rkv/src/backend/impl_safe/environment.rs#L266-L277)
does metadata lookup,
writes a temporary file,
and renames it over the database.

The UI-visible contender is a memory report:
[`xpcom/base/nsMemoryReporterManager.cpp:1995-2005`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/base/nsMemoryReporterManager.cpp#L1995-L2005)
dispatches reporters to the main thread.
[`toolkit/components/glean/xpcom/FOG.cpp:686-693`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/glean/xpcom/FOG.cpp#L686-L693)
calls `fog_collect_reports`;
[`toolkit/components/glean/api/src/lib.rs:79`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/glean/api/src/lib.rs#L79) calls `glean::alloc_size`;
[`glean-core/src/lib.rs:615-618`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/lib.rs#L615-L618)
calls `with_opt_glean`,
which blocks on the same global mutex
([`glean-core/src/core/mod.rs:105-112`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/core/mod.rs#L105-L112)).

Trigger:
memory reporting,
such as measuring memory,
overlaps Glean persistence or other disk work
inside a Glean-dispatcher closure.
This establishes a lock chain,
not a reproduced hang or a claim that each metric update writes on the UI thread.
It is not a Chromium `BrowserMetrics` writable-mapping equivalent.

### Glean shutdown waits and final synchronous persistence

Confirmed shutdown-only dependency.
[`toolkit/components/glean/xpcom/FOG.cpp:99-120`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/glean/xpcom/FOG.cpp#L99-L120)
registers the shutdown callback;
`FOG::Shutdown`,
lines 124 to `128`,
calls `fog_shutdown`.
[`toolkit/components/glean/src/lib.rs:45-46`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/glean/src/lib.rs#L45-L46)
calls `glean::shutdown`.
[`third_party/rust/glean-core/src/lib.rs:678-763`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/lib.rs#L678-L763)
waits for initialization/dispatcher work,
shuts down the dispatcher/uploader,
and synchronously calls `persist_ping_lifetime_data` at lines 759 to `762`.

The queue waits include a ten-second timeout and the uploader wait has a thirty-second timeout;
these are individual source constants,
not a measured total or a bound on final persistence.
The final persistence reaches the SafeMode file write/rename described in the Glean memory-report finding.
This can delay quitting;
it does not prove steady-state browsing jank.

### Rejected and bounded candidates

- Ordinary Glean startup database initialization occurs on the `glean.init` thread
  ([`third_party/rust/glean-core/src/lib.rs:408`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/third_party/rust/glean-core/src/lib.rs#L408)),
  and ordinary metric writes are dispatched.
  There is no evidence here of a continuously written UI-thread disk mmap.
- `fog_apply_serverknobs` performs a direct file read,
  but its call site is `ApplyInterestingServerKnobs` for test/artifact-build behavior
  ([`toolkit/components/glean/xpcom/FOG.cpp:163-169`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/components/glean/xpcom/FOG.cpp#L163-L169),
  `toolkit/components/glean/bindings/jog/JOG.cpp`).
  Do not elevate it to an ordinary packaged-ESR browsing finding.
- Telemetry client-state and stored-ping JS reads use awaited `IOUtils` operations.
  `Telemetry.cpp`'s failed-lock-count reader runs in `nsFetchTelemetryData::Run`
  and dispatches only result processing back to main.
  A synchronous file call inside that worker is not itself UI blocking.
- WebRender capture/replay file readers in `gfx/wr/webrender/src/renderer/mod.rs`
  and `capture.rs` are debug/capture workflows,
  not normal rendering inputs.
- `/proc` reads,
  `/dev` opens,
  and DMA-buf/shared-memory mappings are not automatically persistent-disk I/O.
  They can have other blocking dependencies but need a separately established disk link.
- Native dynamic-library calls such as
  [`gfx/gl/GLLibraryEGL.cpp:511-564`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/gl/GLLibraryEGL.cpp#L511-L564),
  [`widget/gtk/nsSound.cpp:145`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/gtk/nsSound.cpp#L145),
  and [`widget/cocoa/nsMenuX.mm:83`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/cocoa/nsMenuX.mm#L83)
  remain cold-library-load/page-fault candidates.
  The source proves synchronous loader boundaries;
  OS loader/cache residency and the active calling thread must be established for a specific interaction.
- Font,
  GTK,
  shader,
  and startup cache prefetching reduces potential cold reads;
  the inspected code does not prove permanent residency or a maximum disk wait.

## Saving, printing, and native maintenance

### Save Page performs parent-thread file creation, enumeration, and failure cleanup

Confirmed browser UI path on desktop.
[`toolkit/content/contentAreaUtils.js:511-550`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/toolkit/content/contentAreaUtils.js#L511-L550) calls `persist.saveDocument` from the browser save workflow.
[`dom/webbrowserpersist/nsWebBrowserPersist.cpp:720-741`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/webbrowserpersist/nsWebBrowserPersist.cpp#L720-L741) checks the destination with `Exists`
and calls `MakeOutputStream` during document serialization.
`MakeOutputStreamFromFile` at `2247-2269` calls `fileOutputStream->Init` with behavior flags `0`.
[`netwerk/base/nsFileStreams.cpp:686-700`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/netwerk/base/nsFileStreams.cpp#L686-L700) only defers opening when `DEFER_OPEN` is set,
so this caller opens the output immediately.
The Windows branch also calls `MaybeWriteFileZoneIdSync`.

The accompanying `_files` directory is checked and created synchronously at
[`dom/webbrowserpersist/nsWebBrowserPersist.cpp:1800-1821`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/webbrowserpersist/nsWebBrowserPersist.cpp#L1800-L1821).
`EndDownload` explicitly asserts main thread at `2322`
and schedules `EndDownloadInternal` on the current serial event target.
Successful-save cleanup at `2370-2408` checks the directory,
enumerates entries,
and tests stale entries with `IsDirectory` before dispatching their deletion to a background task.
The comment saying deletion is asynchronous does not cover the synchronous enumeration that precedes it.
Failure cleanup at `1890-1965` additionally checks metadata,
removes files,
walks directories,
and removes empty directories on this completion path.

Scope:
saving a complete page and its cleanup,
especially a destination whose metadata operations wait on storage.
This finding does not turn every downloaded resource write into a main-thread write;
[`nsWebBrowserPersist.cpp:979-986`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/webbrowserpersist/nsWebBrowserPersist.cpp#L979-L986) explicitly describes the separate background `OnDataAvailable` path.

### Printing uses temporary files and synchronous recorded-page reads in the parent

Confirmed desktop print path with an additional GTK output-file path.
[`docshell/base/CanonicalBrowsingContext.cpp:1097-1104`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/docshell/base/CanonicalBrowsingContext.cpp#L1097-L1104)
creates `RemotePrintJobParent` and attaches it to the browser's content-process manager.
[`layout/printing/ipc/PRemotePrintJob.ipdl:20-28`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/printing/ipc/PRemotePrintJob.ipdl#L20-L28)
defines the asynchronous initialize/page messages;
asynchronous IPC delivery does not dispatch the receiver's filesystem work to a worker.
[`RemotePrintJobParent.cpp:75-105`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/printing/ipc/RemotePrintJobParent.cpp#L75-L105) initializes the parent print device and prepares a temporary file.
Its accessibility readiness continuation explicitly targets the main thread at `69-73`.

[`layout/printing/ipc/RemotePrintJobParent.cpp:152-163`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/printing/ipc/RemotePrintJobParent.cpp#L152-L163)
calls `NS_OpenAnonymousTemporaryFile` for page recordings.
[`xpcom/io/nsAnonymousTemporaryFile.cpp:82-129`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/io/nsAnonymousTemporaryFile.cpp#L82-L129)
creates a unique temporary file and opens it with delete-on-close semantics.
[`RemotePrintJobParent.cpp:167-240`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/printing/ipc/RemotePrintJobParent.cpp#L167-L240)
seeks the recording,
calls `PrintPage`,
and translates its recorded drawing commands inline.
[`layout/printing/PrintTranslator.cpp:28-72`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/printing/PrintTranslator.cpp#L28-L72)
reads the recording in a loop;
[`layout/printing/DrawEventRecorder.h:97-107`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/layout/printing/DrawEventRecorder.h#L97-L107)
implements those reads with `PR_Read`.
`PageDone` prepares another temporary file at `246-261`.

GTK printing also calls `g_file_open_tmp`,
sets permissions,
and opens a spool output stream in
[`widget/gtk/nsDeviceContextSpecG.cpp:71-115`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/gtk/nsDeviceContextSpecG.cpp#L71-L115).
The Cairo PDF writer calls that stream's `Write` synchronously in
[`gfx/thebes/PrintTargetPDF.cpp:21-36`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/PrintTargetPDF.cpp#L21-L36).
The alternate Skia PDF writer likewise calls `mStream->Write` in
[`gfx/thebes/PrintTargetSkPDF.cpp:170-180`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/gfx/thebes/PrintTargetSkPDF.cpp#L170-L180).

Scope:
active printing,
including temporary-file creation and recorded-page replay.
Actual disk involvement depends on the backing filesystem for temporary/spool/output files.
No claim is made that print preview continuously writes to disk,
or that a temporary file must reside on a physical disk.

### Windows idle cleanup recursively removes anonymous temporary files on the observer path

Confirmed Windows-only maintenance path.
[`xpcom/build/XPCOMInit.cpp:439-441`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/build/XPCOMInit.cpp#L439-L441) registers `CreateAnonTempFileRemover` on Windows.
[`xpcom/io/nsAnonymousTemporaryFile.cpp:248-258`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/xpcom/io/nsAnonymousTemporaryFile.cpp#L248-L258) restricts it to the parent process.
The timer registers an idle observer;
the idle callback at `190-205` directly calls `RemoveAnonTempFileFiles`.
That function at `228-237` obtains the Mozilla temporary directory and calls `Remove(true)`.
There is no background dispatch between notification and recursive removal.
[`widget/nsUserIdleService.cpp:795-803`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/widget/nsUserIdleService.cpp#L795-L803) delivers the observer notification inline.

The source schedules registration after `3` minutes and uses a `30` second idle threshold.
These are scheduling constants,
not an I/O deadline and not proof that the user remains idle throughout cleanup.
This is a maintenance exposure after startup,
not a Linux Firefox finding.

### Other boundaries and rejected media candidates

[`ipc/glue/FileDescriptorUtils.cpp:32-53`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/ipc/glue/FileDescriptorUtils.cpp#L32-L53) intentionally closes a file descriptor on the current thread
when dispatching the normal background close fails.
Its comment accepts a possible main-thread I/O hit rather than leaking the descriptor.
This is an error-path candidate;
no normal UI trigger or runtime frequency was established.

[`dom/media/FileBlockCache.cpp:354-363`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/media/FileBlockCache.cpp#L354-L363) drops `mDataMutex` before disk writes,
and `431-442` drops it before disk reads.
[`dom/media/MediaCache.cpp:2222-2233`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/media/MediaCache.cpp#L2222-L2233) sends main-thread stream close work to the owner thread with `InvokeAsync`.
Those examined operations do not prove a parent UI thread waiting on media disk I/O.
The older OMX comment in [`VideoFrameContainer.cpp:143-149`](https://github.com/mozilla-firefox/firefox/blob/a34c4160e3f75ab9918fbe4669ff7ae56f1977c8/dom/media/VideoFrameContainer.cpp#L143-L149) describes a deadlock-avoidance measure;
it is not evidence that this release retains that deadlock.

## Verification and remaining limits

Release identity and complete-tree inventory were verified against the pinned checkout.
Cited source selections are retained in the accompanying source evidence artifact.
Browser checks matched `205` Firefox source selections and the cited GTK/fontconfig selections.
Pandoc rendered the report and accompanying documentation;
the scoped Markdown formatting task passed.
The handover and evidence artifacts record these checks.

Runtime stall duration,
frequency,
filesystem-cache residency,
active driver behavior,
and contention overlap remain unmeasured.
A follow-up runtime investigation should correlate thread stacks,
file paths,
major faults,
and lock ownership while exercising the named interactions with a disposable profile.
CPU decoding,
font shaping,
GPU-driver work,
and background disk latency need separate attribution.
The audit does not establish that Firefox ESR is more or less responsive than Chromium.
No upstream issue or patch was submitted.
