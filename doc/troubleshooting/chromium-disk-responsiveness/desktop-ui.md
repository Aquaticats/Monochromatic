# Chromium desktop UI disk-blocking source audit

## Scope and evidence status

Source:
Chromium `156.0.8078.73`,
commit `2f9a11097447f226133bc1a31c787291a5eeab98`.
The full checkout is `${HOME}/temp/agent/chromium-disk-audit-20261103`.
This is static source evidence of synchronous operations and their callers,
not a runtime latency measurement or proof that any new finding caused the reported Helium freeze.
All source ranges refer to this commit.
Every cited source-selection URL was opened in a disposable browser session.
Highlighted line ranges and their text matched the pinned checkout.
The [verification record](desktop-link-checks.json) retains failures and successful retries separately.

The production-source searches covered `chrome/browser`,
`ui`,
`content/browser`,
and `components/remote_cocoa` for blocking allowances and thread/disk comments.
A broader filesystem-API search covered desktop UI,
profiles,
themes,
downloads,
extensions,
sessions,
printing,
shell dialogs,
GTK,
and drag/drop.
Tests and mocks were filtered for triage,
then the actual candidate callers were read.
The searches are discovery aids,
not a whole-program proof of all blocking paths.

Retained discovery output:

- `${HOME}/temp/agent/chromium-ui-allowances.txt`
- `${HOME}/temp/agent/chromium-ui-disk-comments.txt`
- `${HOME}/temp/agent/chromium-ui-disk-apis.txt`

## Interactive paths with direct UI-thread disk evidence

### Linux file pickers synchronously inspect filesystem metadata

`ui/shell_dialogs/select_file_dialog_linux.cc:34-38` explicitly allows blocking and calls `DirectoryExists`:

```cpp
// ui/shell_dialogs/select_file_dialog_linux.cc
bool SelectFileDialogLinux::CallDirectoryExistsOnUIThread(
    const base::FilePath& path) {
  base::ScopedAllowBlocking scoped_allow_blocking;
  return base::DirectoryExists(path);
}
```

`ui/shell_dialogs/select_file_dialog_linux.h:51-55` explicitly describes this as access on the UI thread.
`base/files/file_util_posix.cc:905-912` performs `File::Stat` synchronously.
The portal dialog calls it before posting its next step in
`ui/shell_dialogs/select_file_dialog_linux_portal.cc:286-301`.
The GTK dialog calls it when setting up open/save dialogs and accepting paths,
at `ui/gtk/select_file_dialog_linux_gtk.cc:472`,
`571`,
`628`,
and `656`.

GTK 3 preview updates add a raw synchronous `stat` in
`ui/gtk/select_file_dialog_linux_gtk.cc:686-729`:

```cpp
// ui/gtk/select_file_dialog_linux_gtk.cc
if (stat(filename, &stat_buf) != 0 || !S_ISREG(stat_buf.st_mode) ||
    static_cast<uint64_t>(stat_buf.st_size) > kMaxPreviewFileSize.InBytes()) {
```

The file-content read in the same callback is correctly posted to the thread pool.
The metadata access remains on the dialog callback path.
These checks can delay the browser UI when a selected or remembered path requires slow storage,
network-mount metadata,
or filesystem transaction progress.
This is a recurring live-operation candidate,
not merely startup work.

Sources:
[Linux helper](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/shell_dialogs/select_file_dialog_linux.cc#L34-L38),
[portal call](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/shell_dialogs/select_file_dialog_linux_portal.cc#L286-L301),
[GTK preview](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/ui/gtk/select_file_dialog_linux_gtk.cc#L686-L729).

### Switching to an unloaded profile still does synchronous filesystem work

`chrome/browser/profiles/profile_window.cc:222-233` routes `SwitchToProfile` to `CreateProfileAsync`.
`chrome/browser/profiles/profile_manager.cc:835-879` asserts the UI thread and directly calls
`CreateProfileAsyncHelper` when the profile is not loaded.
That helper calls `Profile::CreateProfile` in asynchronous mode at `profile_manager.cc:1322-1324`.
Despite that mode,
`chrome/browser/profiles/profile_impl.cc:375-379` immediately calls `CreateProfileDirectory`.
The latter permits blocking and executes `PathExists`,
`CreateDirectory`,
and `GetFileInfo` at `profile_impl.cc:300-336`:

```cpp
// chrome/browser/profiles/profile_impl.cc
ScopedAllowBlockingForProfile allow_io_to_create_directory;
if (base::PathExists(path.Append(chrome::kReadmeFilename)))
  return GetCreationTimeForPath(path);
if (base::CreateDirectory(path)) {
```

The code comment expressly justifies running these operations on the main thread.
An existing profile also performs metadata calls;
the risk is not limited to creating a new directory.
The asynchronous README write does not remove the synchronous prerequisite.
This affects Linux as well as other desktop platforms.

Sources:
[UI caller](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/profiles/profile_manager.cc#L835-L879),
[filesystem work](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/profiles/profile_impl.cc#L300-L336),
[asynchronous-mode dispatch](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/profiles/profile_impl.cc#L362-L395).

### Enterprise file-scan cancellation waits for file-reading workers

`chrome/browser/enterprise/connectors/analysis/content_analysis_delegate.cc:269-278`
explicitly documents UI-thread teardown waiting for background file opens and hashes.
The handler destructor invokes `MaybeCancelAndReport`,
which calls `file_opening_job_->Cancel()` when interrupted,
in `chrome/browser/enterprise/connectors/analysis/files_request_handler.cc:155-180`.

`components/enterprise/connectors/core/cloud_content_scanning/file_opening_job.cc:89-97` contains the wait:

```cpp
// components/enterprise/connectors/core/cloud_content_scanning/file_opening_job.cc
// JobHandle::Cancel() blocks the UI thread until every worker has returned
// from ProcessNextTask().
file_opening_job_handle_.Cancel();
```

`ProcessNextTask` calls `request->OpenFile(&is_cancelled_)` at `file_opening_job.cc:179-183`.
`file_analysis_request_base.cc:188-195` opens the file;
the hashing loop calls `ReadAtCurrentPos` at `file_analysis_request_base.cc:146-147`.
The loop's own comment identifies slow reads from AV/EDR,
network shares,
and cloud placeholders as concerns at `file_analysis_request_base.cc:118-127`.

Scope qualification:
enterprise content analysis must be active.
Both `kNonBlockingFileOpeningJobCancel` and `kEnableCancelUploadOnContentAnalysis`
are disabled by default in `components/enterprise/connectors/core/features.cc:44-51`.
`Cancel()` itself returns early if the latter feature is off;
the destructor still calls `CancelJobHandle` unconditionally at `file_opening_job.cc:147-149`.
The nonblocking feature signals before teardown,
but a running synchronous read is still not interrupted by an atomic cancellation flag.
The exact active experiment state and teardown timing require a runtime check before claiming this occurs in a user session.

Sources:
[UI wait explanation](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/enterprise/connectors/analysis/content_analysis_delegate.cc#L269-L278),
[job wait and destructor](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/enterprise/connectors/core/cloud_content_scanning/file_opening_job.cc#L89-L150),
[blocking reads](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/enterprise/connectors/core/cloud_content_scanning/file_analysis_request_base.cc#L112-L154),
[default features](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/enterprise/connectors/core/features.cc#L44-L51).

### macOS drag-out creates files before returning to native UI

`content/browser/web_contents/web_contents_view_mac.mm:639-667`
handles `DragPromisedFileTo` on the UI thread.
Its comment explicitly says the filesystem must be queried before returning,
which blocks the UI thread.
It calls `CreateFileForDrop` and then changes file permissions:

```cpp
// content/browser/web_contents/web_contents_view_mac.mm
base::ScopedAllowBlocking allow_blocking;
base::File file(content::CreateFileForDrop(out_file_path));
```

`content/browser/download/drag_download_util.cc:26-53` synchronously tries to create a destination file,
retrying suffixed filenames on failure.
The `kMaxSeq = 99` bound permits the original name plus suffix attempts.
Actual file-content writing is then posted to a worker at `web_contents_view_mac.mm:680-685`.
This is a live drag/drop metadata and creation risk,
not evidence that all dragged bytes are written by the UI thread.

Sources:
[UI drag callback](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/browser/web_contents/web_contents_view_mac.mm#L639-L685),
[file creation loop](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/browser/download/drag_download_util.cc#L26-L53).

### Windows drag-out creates a temporary directory synchronously

`content/browser/web_contents/web_contents_view_aura.cc:205-248`,
under `IS_WIN`,
prepares a drag-download payload:

```cpp
// content/browser/web_contents/web_contents_view_aura.cc
ScopedAllowBlockingForViewAura allow_file_operations;
base::FilePath temp_dir_path;
if (!base::CreateNewTempDirectory(FILE_PATH_LITERAL("chrome_drag"),
                                &temp_dir_path))
  return;
```

The same path registers file/directory deletion after reboot.
`PrepareDragData` calls this helper,
and browser `StartDragging` calls `PrepareDragData` on the normal drag event path.
Network download and bulk file contents are separate work;
the confirmed synchronous operation is temporary-directory setup.

Source:
[drag-download setup](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/browser/web_contents/web_contents_view_aura.cc#L205-L248).

### Windows crash acknowledgment deletes a profile file synchronously

`chrome/browser/sessions/exit_type_service.cc:210-234`
checks whether the user acknowledged a crash and calls `profile_->AckCrashForTracking()`.
`ProfileImpl::AckCrashForTracking` forwards to the tracker at `profile_impl.cc:1675-1678`.
`chrome/browser/profiles/profile_load_tracker_win.cc:188-201` permits blocking and deletes the crash-ack file:

```cpp
// chrome/browser/profiles/profile_load_tracker_win.cc
base::ScopedAllowBlocking allow_blocking;
base::FilePath waiting_for_crash_ack_path =
    profile_dir_.Append(kWaitingForCrashAckFileName);
base::DeleteFile(waiting_for_crash_ack_path)
```

The tracker also creates or opens lock/dirty/ack files and checks their existence during profile loading,
and deletes the dirty file plus closes a delete-on-close lock during unloading,
at `profile_load_tracker_win.cc:54-183`.
This adds interactive profile lifecycle metadata operations on Windows.

Sources:
[acknowledgment caller](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/sessions/exit_type_service.cc#L210-L234),
[tracker operations](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/profiles/profile_load_tracker_win.cc#L170-L201).

### macOS file pickers inspect the default directory synchronously

`components/remote_cocoa/app_shim/select_file_dialog_bridge.mm:350-362`
runs `DirectoryExists(default_path)` under `ScopedAllowBlocking`
while setting up the native open/save panel.
The comment explicitly accepts the `stat` because the dialog will perform more metadata operations.
This can delay the dialog's UI process or browser process,
depending on the app-shim hosting configuration.
Do not label every remote-Cocoa invocation as the browser main thread without checking that configuration.

Source:
[native panel setup](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/remote_cocoa/app_shim/select_file_dialog_bridge.mm#L350-L362).

## Profile loading and startup paths

### Theme packs and theme migration read files on the UI thread

`chrome/browser/themes/browser_theme_pack.cc:726-750`
asserts `BrowserThread::UI`,
permits blocking,
and calls `ui::DataPack::LoadFromPath`:

```cpp
// chrome/browser/themes/browser_theme_pack.cc
DCHECK_CURRENTLY_ON(BrowserThread::UI);
base::ScopedAllowBlocking scoped_allow_blocking;
if (!data_pack->LoadFromPath(path)) {
```

`ThemeService::InitFromPrefs` invokes it at `chrome/browser/themes/theme_service.cc:1008-1018`.
If the pack is absent or unusable,
`MigrateTheme` explicitly builds the replacement on UI at `theme_service.cc:1073-1093`.
`BrowserThemePack::LoadRawBitmapsTo` permits blocking at `browser_theme_pack.cc:1355`
and calls `ReadFileData` at `1376`;
that helper opens,
gets length,
and reads the file synchronously at `browser_theme_pack.cc:349-365`.
The code's stated tradeoff is avoiding a temporary default-theme appearance.
This is relevant when loading a themed profile,
including startup.
`ThemeService::HandlePolicyColorUpdate` also calls `InitFromPrefs` when a policy theme is removed
at `theme_service.cc:1243-1253`,
so the mapped pack load is reachable after startup through a policy change;
ordinary theme changes use `ThreadPool` at `theme_service.cc:1100-1116`
and must not be reported as the same UI read path.

Sources:
[pack load](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/themes/browser_theme_pack.cc#L726-L750),
[migration versus ordinary theme change](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/themes/theme_service.cc#L1073-L1116),
[synchronous image read](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/themes/browser_theme_pack.cc#L349-L367).

### Extension manifest reload during profile startup reads on UI

`chrome/browser/extensions/installed_loader.cc:351-385`
asserts the UI thread,
then calls `file_util::LoadExtension` synchronously if `ShouldReloadExtensionManifest` returns true:

```cpp
// chrome/browser/extensions/installed_loader.cc
base::ScopedAllowBlocking allow_blocking;
scoped_refptr<const Extension> extension(
    file_util::LoadExtension(info.extension_path, info.extension_location,
                            GetCreationFlags(&info), &error));
```

The comment expressly acknowledges disk reads on UI and cites `crbug.com/40366546`.
The predicate at `installed_loader.cc:117-139` includes unpacked extensions,
relocalization,
and cached manifests with the checked corruption pattern.
An absent cached manifest returns false from this predicate.
This is profile-load work,
not proof that every installed extension is re-read on every UI event.

Source:
[manifest reload](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/extensions/installed_loader.cc#L351-L385).

### Clean-exit beacon performs synchronous diagnostic file I/O

`components/metrics/clean_exit_beacon.cc:229-239`
reads the beacon during initialization via `MaybeGetFileContents`.
`WriteBeaconValue` invokes `WriteBeaconFile` for supported profiles at `clean_exit_beacon.cc:297-317`.
`WriteBeaconFile` serializes JSON and performs this direct write at `clean_exit_beacon.cc:432-446`:

```cpp
// components/metrics/clean_exit_beacon.cc
base::ScopedAllowBlocking allow_io;
success = base::WriteFile(beacon_file_path_, json_string);
```

Startup invokes it via `MaybeExtendVariationsSafeMode`,
`components/variations/service/variations_field_trial_creator.cc:162-177`.
Desktop shutdown invokes it through `MetricsService::LogCleanShutdown`,
`components/metrics/metrics_service.cc:731-734`,
called by `BrowserProcessImpl::EndSession` at `chrome/browser/browser_process_impl.cc:888-891`.
Android/iOS foreground and background transitions also call the beacon through
`metrics_service.cc:640` and `692`.
This is an additional diagnostic-write path beyond persistent histogram memory mappings.
It is not evidence of a periodic desktop UI `fsync`:
the periodic live-timestamp task only changes preferences in memory,
and this source uses `WriteFile`,
not an explicit durability flush.

Sources:
[beacon write](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/metrics/clean_exit_beacon.cc#L432-L446),
[startup caller](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/components/variations/service/variations_field_trial_creator.cc#L162-L177).

### Debug logging opens files synchronously when the network service is created

`content/browser/network_service_instance_impl.cc:605-621`
defines a deliberately blocking file-open helper.
`GetNetworkService` calls it for `--log-net-log` at `network_service_instance_impl.cc:713-714`
and for `--ssl-key-log-file` or `SSLKEYLOGFILE` at `752-754`.

```cpp
// content/browser/network_service_instance_impl.cc
base::ScopedAllowBlocking allow_blocking;
return base::File(path, file_flags);
```

The helper's comment says opening asynchronously would lose startup debug data.
Network-service startup or reconnection on the UI path therefore waits for these configured opens.
This is conditional debugging behavior,
not an unconditional normal-browsing path or evidence that all subsequent log writes happen on UI.

Source:
[debug file setup](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/content/browser/network_service_instance_impl.cc#L605-L621).

### Shutdown waits for asynchronous preference writes

`chrome/browser/browser_process_impl.cc:870-900`
requests profile and Local State preference writes with rundown callbacks.
It then synchronously waits for rundown at `browser_process_impl.cc:923-944`:

```cpp
// chrome/browser/browser_process_impl.cc
rundown_counter->TimedWait(kEndSessionTimeout);
```

`kEndSessionTimeout` is `base::Seconds(10)` at `browser_process_impl.cc:308`.
The platform branch covers Windows,
Ozone,
and macOS.
This is an explicit join from the main-thread shutdown path to disk work;
it can delay exit but is not evidence of an ordinary mid-session freeze.

Source:
[shutdown write and wait chain](https://github.com/chromium/chromium/blob/2f9a11097447f226133bc1a31c787291a5eeab98/chrome/browser/browser_process_impl.cc#L870-L944).

## Candidates requiring more dependency or runtime attribution

### Windows printing invokes blocking driver calls on UI

`chrome/browser/ui/webui/print_preview/local_printer_handler_default.cc:50-64`
selects the UI task runner on Windows for the in-process path,
while CUPS gets a worker task runner.
Enumeration,
default-printer lookup,
and capability lookup permit blocking at `81`,
`125`,
and `148`.
`PrinterQuery::UpdatePrintableArea`,
`UpdatePrintSettings`,
and `UseDefaultSettings` assert UI and permit driver blocking at
`chrome/browser/printing/printer_query.cc:268-287`,
`332-371`,
and `447-459`.
Out-of-process printing bypasses some in-process calls,
but `PrintBackendServiceManager::SetCrashKeys` still invokes driver lookup locally at
`chrome/browser/printing/print_backend_service_manager.cc:158-170`.

This is confirmed blocking-driver exposure,
but the traced Chromium wrapper ends at Winspool APIs.
Disk I/O,
network I/O,
and driver-internal waits are not distinguished by the inspected source.
Do not list this as a confirmed disk-backed UI stall without a Windows trace or deciding driver source.

### ChromeOS internal diagnostic pages contain direct filesystem calls

`chrome/browser/ui/webui/bluetooth_internals/bluetooth_internals_handler.cc:280-320`
performs `PathExists` checks and raw `open(..., O_WRONLY | O_CREAT | O_TRUNC, 0644)`
when stopping a Bluetooth trace.
`chrome/browser/ui/webui/ash/dlp_internals/dlp_internals_page_handler.cc:301-315`
directly `stat`s the selected Downloads path for `GetFileInode`.
Both are WebUI handler callback paths without a worker dispatch in the handler.
These are diagnostic-page candidates,
not generic desktop-Linux paths.
Thread-affinity tracing beyond the WebUI construction boundary was not completed by this worker.

## False positives checked

- Ordinary extension-theme changes dispatch `BuildFromExtension` to a `MayBlock` worker
  in `chrome/browser/themes/theme_service.cc:1100-1116`.
- GTK file-preview bytes are read on a worker at `ui/gtk/select_file_dialog_linux_gtk.cc:725-729`;
  the synchronous issue is the preceding `stat`.
- GTK print spool creation/writes run on the print worker or service document task runner,
  expressly documented and checked in `ui/gtk/printing/print_dialog_gtk.cc:515-550`.
- Download image copying posts `StartDecoding` from UI to the supplied task runner,
  then performs file-size/read work in `chrome/browser/download/download_commands.cc:63-88`.
- Omnibox selected-file reads are posted to `ThreadPool` at
  `chrome/browser/ui/views/location_bar/omnibox_popup_file_selector.cc:253-265`.
- Screen AI DLC existence checks are posted to a worker at
  `chrome/browser/screen_ai/screen_ai_dlc_installer.cc:133-146`.
- Web-app internals directory enumeration is posted to a worker at
  `chrome/browser/ui/webui/web_app_internals/web_app_internals_handler.cc:315`.
- On-device internals model-file setup is posted to a worker at
  `chrome/browser/ui/webui/on_device_internals/on_device_internals_page_handler.cc:79-84`.
- Omnibox Everywhere shortcut work is invoked through a sequence-bound helper's `AsyncCall`
  at `chrome/browser/ui/omnibox/omnibox_everywhere/omnibox_everywhere_ui_manager.cc:268-272`.
- Drive internals service-log reads and their `stat` call run inside a posted worker task at
  `chrome/browser/ui/webui/ash/drive_internals/drive_internals_ui.cc:681-687`.
- `chrome/browser/ui/webui/print_preview/print_preview_ui_untrusted.cc:68-89`
  reads a PDF only for the special `test.pdf` request path,
  not ordinary Print Preview content.
- `/proc` reads in memory-pressure and renderer-priority helpers are kernel pseudo-files;
  a blocking allowance there is not by itself evidence of disk access.

## Limitations and priorities

Prioritize Linux file-picker metadata,
profile creation/loading,
theme loading,
and conditional enterprise scan cancellation for additional runtime tracing.
Keep the [metrics mapping](mapped-files.md#browsermetrics-writes-can-fault-on-the-browser-ui-thread)
as a separate mechanism from beacon writes.
Startup and shutdown findings should be tested with startup/exit timing,
not presented as explanations of arbitrary steady-state freezes.
Platform-driver calls remain candidates until the actual wait is attributed.
No source edits,
feature changes,
browser restarts,
stress load,
or upstream filings were made by this worker.
