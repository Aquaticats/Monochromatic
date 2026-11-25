# Targeted evidence: Watchman 2026.11.23.00

Status:
 agent-written during the targeted-evidence run of 2026-11-25;
 read-only against the candidate (nothing was installed, built, tested or run);
 incomplete in the sense that no runtime validation happened;
 not approved by the owner,
 not final.

This file records evidence about one candidate for the watcher vet whose report is
`doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`.
The design it serves is `doc/planning/garage-bisync-run-on-file-change.md`.
It records no score, no ranking and no recommendation,
and it does not edit the vet report.

## Purpose and scope

The vet report lists seven hard constraints and asks each surviving candidate for
source-level confirmation of them, plus a maintenance audit.
This file answers that for Watchman 2026.11.23.00 with primary evidence:
the released tag's own source,
the release workflows committed at that tag,
the published release assets and their recorded digests,
the packaging scripts that build those assets,
the issue and pull-request history,
and every repository document that already recorded a verdict on Watchman.

Every claim below is either read from source (cited as `path:line` plus an excerpt)
or read from a GitHub API response (cited as the exact query and the fields used).
No behaviour in this file was measured at runtime.
The vet report's external execution gate and finalist validation remain open,
so nothing here stands in for them.

## Candidate identity and access

Candidate: Watchman 2026.11.23.00, the `watchman` daemon plus its CLI.
Repository: `facebook/watchman`, which GitHub now redirects to `meta/watchman`
(the API responses below use `meta/watchman`).
Release tag: `v2026.11.23.00`.
Access date for every URL and API query below: 2026-11-25,
in a session that ran from about 18:25 to 20:10 -05:00.

The clone was made exactly as instructed:

```console
$ gh repo clone facebook/watchman "/home/user/temp/agent/watchman-source-for-garage-watcher-vet-2026-11-25" -- --depth 1 --branch v2026.11.23.00
```

Clone path: `/home/user/temp/agent/watchman-source-for-garage-watcher-vet-2026-11-25`.
It is a shallow clone and is deleted when this run ends,
so every path cited below is relative to that clone root and can be re-created with the
command above.

What the tag points at:

- `gh api repos/facebook/watchman/git/ref/tags/v2026.11.23.00` reports
  `"ref":"refs/tags/v2026.11.23.00"` with
  `"object":{"sha":"389ebc81ee295451f01c0db330035416dd94b1e9","type":"commit"}`.
  The tag is lightweight, not annotated, so there is no tag object and no tagger identity.
- In the clone, `git rev-parse HEAD` prints `389ebc81ee295451f01c0db330035416dd94b1e9`,
  dated `2026-11-22T09:33:29-08:00`, subject `Updating hashes`.
- `gh api repos/facebook/watchman/releases/tags/v2026.11.23.00` reports
  `"published_at":"2026-11-23T16:28:51Z"`, `"draft":false`, `"prerelease":false`,
  `"immutable":false`, `"author":{"login":"github-actions[bot]"}` and `"body":null`.
  The release was published by the workflow, not by a person, and carries no notes.
- The clone is depth 1, so no history is available locally.
  All history claims in this file come from GitHub API queries, not from `git log`.

Working tree shape at the tag:
a CMake project (`CMakeLists.txt`) whose daemon source sits in `watchman/`,
with `watchman/watcher/` holding the platform backends,
`watchman/root/` holding the crawl and inotify-independent directories logic,
`watchman/cli/` holding the Rust `watchmanctl`,
`watchman/rust/` holding the Rust client crates,
`watchman/thirdparty/` holding vendored libraries,
`website/docs/` holding the documentation the vet report read,
and `build/` holding the Facebook build tooling and the dependency pin files.

### Sources outside the clone used for two claims

- `facebookexperimental/edencommon` at revision
  `4db211e753106e9ac9374558d4c34a4bdb504eee`
  (the pin in `build/deps/github_hashes/facebookexperimental/edencommon-rev.txt`),
  file `eden/common/telemetry/StructuredLoggerFactory.h`,
  fetched with `gh api repos/facebookexperimental/edencommon/contents/...`.
  Watchman vendors no copy of that file;
  the build fetches it, so the claim about default telemetry needs that source.
- `actions/runner-images` `README.md`, fetched the same way,
  to settle which architecture the `macos-15` and `windows-2022` labels mean
  (line 34: `macOS 15 Arm64 ... arm64 ... `macos-15``, line 33: `macos-15-large`/`macos-15-intel` are x64).

## How to read a record

Each record carries:

- the candidate and version;
- the claim and why it decides a gate;
- the gate it belongs to, and its status;
- the primary source as `path:line` (relative to the clone above) or a URL,
  with a short excerpt;
- the access date;
- the outcome.

Records marked as inference say so.
A record that rests on a documentation page records the page, not the code,
and says which.

## Hard-constraint records

### Licence and inspectable source

#### E01 Licence text and file path

- Candidate: Watchman 2026.11.23.00.
- Claim: the project is open source under a permissive licence.
- Gate: hard constraint 1 (open source with inspectable source), licence part.
- Status: pass.
- Source: `LICENSE:1-3`:

  ```text
  MIT License

  Copyright (c) Meta Platforms, Inc. and its affiliates.
  ```

  and `README.markdown:22-24`:
  "Watchman is made available under the terms of the MIT License."
- Access date: 2026-11-25.
- Outcome: MIT, permissive, compatible with this repository's use.
  Nuance for the dependency surface, not for the binary this job would run:
  `watchman/thirdparty/getopt/GetOpt.cpp:1-10` carries the Regents of the University of California
  and PostgreSQL BSD notices,
  and `watchman/cli/Cargo.toml:7` declares `license = "MIT"` for `watchmanctl`.
  The npm clients (`bser`, `fb-watchman`) had a licence-string mismatch,
  filed as issue 1349 and fixed in source by a maintainer (see E31);
  this job would not install them.

#### E02 The released version has inspectable source at the tag

- Claim: the exact version a user installs can be read as source.
- Gate: hard constraint 1 (inspectable source).
- Status: pass.
- Source: the clone above at tag `v2026.11.23.00`, commit `389ebc8`.
  Every behaviour cited in this file is a `path:line` inside that tree,
  and `website/docs/cmd/trigger.md:1-28` is the trigger documentation shipped at the same tag.
- Access date: 2026-11-25.
- Outcome: pass.
  The tree is the daemon, the CLI, the Rust and Python clients, the site and the build tooling.
  The build also fetches Meta dependencies from other repositories (see E08);
  that is a provenance caveat, not an inspectability failure.

### Provenance of what a user installs

The release tag is pushed, not hand-made:
`.github/workflows/release.yml:8-11` runs on `push` of tags matching `v*`,
and `:17-30` calls three reusable workflows with `release: ${{ github.ref_name }}`.
The `publish` job at `:34-54` needs all three, downloads their `*-packages` artifacts,
runs `gh release create "$TAG" --verify-tag ...` and then `gh release upload "$TAG" dist/*`.
So each asset below is produced from the tag's own commit by the workflow committed at that tag.

#### E03 Linux x86-64 artifacts: built from the tag by the committed workflow

- Claim: the Linux x86-64 binaries a user installs map to the tag.
- Gate: native or prebuilt overlay, provenance.
- Status: pass, with the dependency caveat in E08.
- Source:
  - `.github/workflows/linux.yml:31-35` builds two matrix entries:
    `runs-on: ubuntu-24.04` with `arch: x64`, and `runs-on: ubuntu-24.04-arm` with `arch: arm64`.
  - `:46` `- uses: actions/checkout@v6` (in a `workflow_call`, this checks out the caller's ref, the tag).
  - `:83-90` configures with
    `-DCMAKE_PROJECT_INCLUDE="$PWD/watchman/build/package/prefer-static.cmake"`
    and `-DFB_FETCH_AS_OF="$COMMIT_TIME"`.
  - `:105-108` runs `./watchman/build/package/package-linux.sh _build "$RELEASE" _packages`.
  - `watchman/build/package/package-linux.sh:27-41` names the x86-64 zip without an architecture suffix
    ("Only the x86-64 zip has no architecture in its name"),
    copies `bin/watchman` and `bin/watchmanctl`, and strips them;
    `:63` writes the zip, `:65-78` the `.deb`, `:80-87` the `.rpm`.
  - `gh api repos/facebook/watchman/releases/tags/v2026.11.23.00` lists
    `watchman-v2026.11.23.00-linux.zip` (8372671 bytes,
    `sha256:79ff42b247d89f0def5b231a8bb59a1cf28ccf1d070a3fc6d428eba9d623c997`),
    `watchman_v2026.11.23.00_amd64.deb` (6534170 bytes,
    `sha256:8b93b8a5f4d5367f21b537ff07d79c037bb68589f18030e0079f545d7d2f7c6d`),
    and `watchman-20261122.093329.0-1.x86_64.rpm` (8338064 bytes,
    `sha256:7e25eb70bb6316661ebe46ee3e50f278c72a50821322ded2b82adc46880efd99`).
- Access date: 2026-11-25.
- Outcome: pass.
  `website/docs/install.md:152-160` states the build links everything but glibc and the GCC runtime statically,
  and needs glibc 2.39 and the libstdc++ of GCC 13 or newer
  (Ubuntu 24.04, Debian 13, Fedora 40, RHEL 10 and newer),
  which this CachyOS host meets.
  The install page (`:96-112`) also has the user copy `bin/*` into `/usr/local/bin` by hand.

#### E04 macOS arm64 artifact: built from the tag by the committed workflow

- Claim: the macOS arm64 binary a user installs maps to the tag.
- Gate: native or prebuilt overlay, provenance; and the multi-platform constraint's macOS part.
- Status: pass, with the macOS floor and the signing gap in E06.
- Source:
  - `.github/workflows/macos.yml:28` `runs-on: macos-15`,
    which `actions/runner-images/README.md:34` maps to arm64.
  - `:38` `MACOSX_DEPLOYMENT_TARGET: "15.0"`, with the comment that Rust crates otherwise target a newer SDK.
  - `:41` `actions/checkout@v6`.
  - `:80-88` configures with the same `prefer-static.cmake` include and `FB_FETCH_AS_OF`.
  - `:101-104` runs `./watchman/build/package/package-macos.sh _build "$RELEASE" _packages`.
  - `watchman/build/package/package-macos.sh:26-31` copies `bin/watchman` and `bin/watchmanctl`;
    `:34-47` refuses an `LC_RPATH` and any library outside `/usr/lib` and `/System/Library`;
    `:56` writes the zip.
  - Release asset `watchman-v2026.11.23.00-macos.zip`, 7000225 bytes,
    `sha256:93cc4cbe22cca126d9ba16c4e391bfb837b8410d56102648d8ef94516110e414`.
- Access date: 2026-11-25.
- Outcome: pass for "macOS arm64 exists and maps to the tag".
  Two facts the later macOS client work inherits:
  the binary's deployment target is macOS 15.0, so it will not run on older macOS;
  and `website/docs/install.md:96-101` tells the user the binaries are not notarized
  and must have the quarantine attribute stripped with `xattr -dr com.apple.quarantine`.

#### E05 Windows x64 artifact: built from the tag by the committed workflow

- Claim: the Windows x64 binary a user installs maps to the tag.
- Gate: native or prebuilt overlay, provenance; and the multi-platform constraint's Windows part.
- Status: pass, with the bundled-runtime caveat below.
- Source:
  - `.github/workflows/windows.yml:28` `runs-on: windows-2022` (x64).
  - `:49` `actions/checkout@v6`.
  - `:125-138` configures with the vcpkg toolchain and `-DFB_FETCH_AS_OF=%COMMIT_TIME%`.
  - `:153-156` runs `./watchman/build/package/package-windows.ps1 _build $env:RELEASE _packages`.
  - `watchman/build/package/package-windows.ps1:29-40` copies `watchman.exe`, `watchmanctl.exe`,
    `eledo-pty-bridge.exe`, `watchman-diag.exe`, `watchman-make.exe`,
    `watchman-replicate-subscription.exe` and `watchman-wait.exe` into `bin/`;
    `:20-21` pins the Python runtime to `3.14.10` with
    `90905e64a840557d8c1bc4f5b74db3d7ee85e3ffb217611cac1c7813acea4159`;
    `:47-52` downloads `python-3.14.10-embed-amd64.zip` from python.org and checks that digest before
    unpacking it into `bin\python`;
    `:85` writes the zip.
  - Release asset `watchman-v2026.11.23.00-windows.zip`, 28533000 bytes,
    `sha256:c1fa4f93fec53a151e8e822d273db52b9be155819945067bc348225e0e5ba81e`.
- Access date: 2026-11-25.
- Outcome: pass for "Windows x64 exists and maps to the tag", with a documented exception to
  "built only from the tag": the zip embeds an official python.org embeddable runtime,
  fetched during packaging and verified against a hardcoded SHA-256.
  That runtime's provenance is python.org's, not Meta's, and the packaging step records it.

#### E06 No published checksum file, no signature, no notarization, no attestation

- Claim: whether a user can verify the artifact other than by trusting GitHub.
- Gate: native or prebuilt overlay (checksum or signature).
- Status: pass for provenance, because the artifacts do map to the tag (E03 to E05);
  no signature, no notarization, no published checksum file (recorded as a limit).
- Source:
  - The release's eight asset names are exactly the binaries in E03 to E05;
    none is a checksum, signature, or provenance file
    (query in E03).
  - `.github/workflows/release.yml:44-54` downloads artifacts and uploads them;
    there is no `sigstore`, `cosign`, `gpg`, or `attest` step in any of the four workflow files.
  - The API's per-asset `digest` field is a GitHub-computed `sha256:`,
    which is an integrity record held by the hosting platform,
    not a maintainer signature.
  - GitHub issue 1359, `macOS binaries are not signed.`, open since 2026-08-21,
    updated 2026-11-04, zero comments:
    `codesign -dvv /opt/homebrew/bin/watchman` shows `Signature=adhoc` and `TeamIdentifier=not set`.
  - `website/docs/install.md:104-112` says the same in the project's own words:
    "The Watchman binaries are not notarized by Apple".
- Access date: 2026-11-25.
- Outcome: provenance is known, so this is not the unknown-provenance hard failure.
  Verification stops at the hosting platform: no signature, no notarization, no published checksum
  file, no attestation.
  A user can pin the GitHub API digest or the release asset bytes and nothing stronger.
  The vet report's ledger entry for Watchman says "prebuilt binary boundary (weekly date releases, checksum files)";
  there are no checksum files. See the departures section.

#### E07 What the Linux package installs, and what it does not

- Claim: the installed footprint, for the privileged-operations question.
- Gate: security boundary; also informs E29.
- Status: pass (no service, no privileged unit).
- Source: `watchman/build/package/package-linux.sh:65-78` builds the `.deb` from
  `deb/usr/local/bin` plus `watchman-deb/DEBIAN`, and `:80-87` builds the `.rpm` from
  `prefix /usr/local/` and the same image.
  Nothing in either path installs a unit file, a timer, or a sysctl drop-in.
- Access date: 2026-11-25.
- Outcome: the Linux artifact is two binaries in `/usr/local/bin`.
  The daemon is spawned on demand by the client (E29), not by the system service manager.

#### E08 The build resolves Meta dependencies by commit time, not by the tag alone

- Claim: how much of the artifact the tag alone determines.
- Gate: native or prebuilt overlay, reproducibility.
- Status: open (documented, not hidden).
- Source:
  - `build/fbcode_builder/CMake/FBFetchFromManifest.cmake:19-25`:
    `FB_FETCH_AS_OF` "Fetch git dependencies' branches as they were at this time ... ignoring deps/github_hashes".
  - `:49-91` `_fb_fetch_commit_as_of` clones the dependency bare and takes
    `rev-list -1 --before=${FB_FETCH_AS_OF}`.
  - The three release workflows set it to the checkout's own commit time
    (`linux.yml:78-90`, `macos.yml:75-88`, `windows.yml:113-138`).
  - `build/deps/github_hashes/facebook/folly-rev.txt` and its siblings pin
    `Subproject commit 062cc44cc57a43c6f38d7c776732e94746e4f6e7` and five more,
    and those pins are bypassed when `FB_FETCH_AS_OF` is set, per `:99-108` of the same file.
- Access date: 2026-11-25.
- Outcome: the artifact is built from the tag's Watchman source plus folly, fizz, fbthrift, fb303,
  wangle, mvfst and edencommon as their default branches stood at the tag's commit time.
  That rule is committed and deterministic given those repositories' history,
  but the full build input is not a single recorded revision set.
  The vet report should carry this as the reproducibility limit of the prebuilt artifacts.

### Watching backends in source for the three target systems

Watcher selection is `WatcherRegistry::initWatcher` (`watchman/watcher/WatcherRegistry.cpp:75-164`):
it tries the configured name, then every registered watcher in descending priority
(`:109-115`), taking the first that initialises (`:118-128`).
Registrations and priorities:

- `watchman/watcher/inotify.cpp:534` `static WatcherRegistry reg("inotify", detectInotify);` (priority 0).
- `watchman/watcher/fsevents.cpp:873` `static RegisterWatcher<FSEventsWatcher> reg("fsevents");` (priority 0).
- `watchman/watcher/kqueue_and_fsevents.cpp:323` `... reg("kqueue+fsevents", makeKQueueAndFSEventsWatcher, 5);`.
- `watchman/watcher/kqueue.cpp:339-341` `... reg("kqueue", -1 /* last resort on macOS */);`.
- `watchman/watcher/win32.cpp:431` `static RegisterWatcher<WinWatcher> reg("win32");` (`#ifdef _WIN32` at `:25`).
- `watchman/watcher/eden.cpp:1662-1663` `... reg("eden", detectEden, 100 /* prefer eden above others */);`.

#### E09 Linux: inotify, one instance per root, one watch per directory

- Claim: Linux watching exists and is the default there.
- Gate: hard constraint 3 (platforms).
- Status: pass.
- Source:
  - `watchman/watcher/inotify.cpp:23-25` compiles only when `HAVE_INOTIFY_INIT`.
  - `:160-186` `InotifyWatcher::InotifyWatcher` calls `inotify_init1(IN_CLOEXEC)` and throws on failure.
  - `:33-37` the mask is `IN_ATTRIB | IN_CREATE | IN_DELETE | IN_DELETE_SELF | IN_MODIFY |
    IN_MOVE_SELF | IN_MOVED_FROM | IN_MOVED_TO | IN_DONT_FOLLOW | IN_ONLYDIR | IN_EXCL_UNLINK`.
  - `:133-135`: "we use one inotify instance per watched root dir".
- Access date: 2026-11-25.
- Outcome: pass, and `inotify` is the watcher reported for a root on Linux
  (`WatcherRegistry.cpp:63-72` logs "using watcher mechanism ...").

#### E10 macOS: FSEvents by default, with kqueue+fsevents and kqueue as alternatives

- Claim: macOS watching exists and is the default there.
- Gate: hard constraint 3 (platforms).
- Status: pass, with a selection detail worth recording.
- Source:
  - `watchman/watcher/fsevents.cpp:335-336` builds `FSEventStreamCreateFlags`;
    `:418` `latency = root->config.getDouble("fsevents_latency", 0.01)`;
    `:425-429` `flags = kFSEventStreamCreateFlagNoDefer | kFSEventStreamCreateFlagWatchRoot`
    (plus `kFSEventStreamCreateFlagFileEvents` when the watcher "hasFileWatching_"),
    then `FSEventStreamCreate(nullptr, fse_callback, &ctx, parray.get(), since, latency, flags)`.
  - `watchman/watcher/kqueue_and_fsevents.cpp:310-319` throws
    "Not using the kqueue+fsevents watcher as the "prefer_split_fsevents_watcher" config isn't set"
    unless that config is true.
  - `WatcherRegistry.cpp:79-163` therefore falls through from the higher-priority
    `kqueue+fsevents` to `fsevents` on a default configuration.
- Access date: 2026-11-25.
- Outcome: pass. A default macOS root is watched by one FSEvents stream on the root path,
  which Apple documents as recursive; `kqueue+fsevents` only runs when the config asks for it.
  The one-line selection detail matters if a later run compares watcher names in the log.

#### E11 Windows: ReadDirectoryChangesW with subtree watching

- Claim: Windows watching exists and is the default there.
- Gate: hard constraint 3 (platforms).
- Status: pass.
- Source:
  - `watchman/watcher/win32.cpp:69-82` opens the root with
    `CreateFileW(..., FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED, ...)`.
  - `:146-154` and `:178-186` call `ReadDirectoryChangesW((HANDLE)dir_handle.handle(), &buf[0], size,
    TRUE, filter, nullptr, &olap, nullptr)`;
    the `TRUE` argument is `bWatchSubtree`, so the OS reports the whole tree.
  - `:124-129` adds `win32_batch_latency_ms`, default 30, as artificial latency.
- Access date: 2026-11-25.
- Outcome: pass. Recursion on Windows is the OS's subtree flag, not a watch per directory.

### Recursion, including folders created after the watcher starts

#### E12 Linux: a new folder is crawled and gets its own watch

- Claim: folders created after start are watched, which is hard constraint 4's Linux half.
- Gate: hard constraint 4 (recursion), Linux.
- Status: pass.
- Source:
  - A created or deleted child sets the recursive flag:
    `watchman/watcher/inotify.cpp:358-360`
    `if (ine->mask & (IN_CREATE | IN_DELETE)) { pending_flags.set(W_PENDING_RECURSIVE); }`.
  - The iothread stats the path; an unknown or previously deleted directory sets `recursive = true`:
    `watchman/root/iothread.cpp:990-998` (`if (!file->exists) { ... recursive = true; }`)
    and `:1024-1028` (`if (st.isDir()) { if (dir_ent == nullptr) { recursive = true; } ... }`).
  - The directory is then queued for a recursive crawl:
    `:1048-1054`
    ```cpp
    if (recursive) {
      /* we always need to crawl if we're recursive, this can happen when a
       * directory is created */
      coll.add(pending.path, pending.now, desynced_flag | W_PENDING_RECURSIVE | W_PENDING_CRAWL_ONLY);
    }
    ```
  - Every directory the crawler opens gets a watch before it is read:
    `:507-518` (`osdir = watcher_->startWatchDir(root, path.c_str());`)
    and, on the parallel path, `:621-643` (`CrawlerFileSystem::openDir` delegates to
    `watcher_->startWatchDir(root_, path)`), used only when `enable_parallel_crawl` is set
    (`watchman/root/init.cpp:259`, default false).
  - `watchman/watcher/inotify.cpp:207-234` is that watch:
    ```cpp
    int newwd = inotify_add_watch(infd.fd(), path, WATCHMAN_INOTIFY_MASK);
    if (newwd == -1) { ... throw std::system_error(err, inotify_category(), "inotify_add_watch"); }
    ```
    with the mapping `wd -> name` recorded at `:224-229`.
- Access date: 2026-11-25.
- Outcome: pass. A folder created after start is crawled (its children enumerated) and watched
  before it is read, so a file written into it in the first moments is not missed by the watch itself.
  A folder moved into the tree takes the same path through `recursive = true`.
  Note for later: the direct "watch the target of a move" shortcut at
  `watchman/watcher/inotify.cpp:303` is dead code,
  because it compares `(mask & (IN_MOVED_TO | IN_ISDIR))` with `(IN_MOVED_FROM | IN_ISDIR)`,
  and `IN_MOVED_TO` (`0x80`) is not `IN_MOVED_FROM` (`0x40`).
  It is harmless because the crawl above covers the same case, and an external fix for it is
  pending as PR 1347 (see E32).

#### E13 macOS and Windows recursion, including folders created later

- Claim: hard constraint 4's macOS and Windows halves.
- Gate: hard constraint 4 (recursion).
- Status: pass.
- Source:
  - macOS: the single FSEvents stream is created on the root path (E10),
    so recursion is the stream's own contract, not per-directory watches;
    the FSEvents backend's `startWatchDir` exists only to enforce the order of open and watch
    (`watchman/watcher/fsevents.cpp:842-860`, which opens the directory and returns it).
  - Windows: `ReadDirectoryChangesW` with `bWatchSubtree = TRUE` (E11).
- Access date: 2026-11-25.
- Outcome: pass. The OS delivers subtree events on both systems;
  no per-directory watch needs adding when a folder appears.

### Running the command

#### E14 POSIX: an argv array, no shell. Windows: through `cmd.exe`

- Claim: how the trigger command is invoked, which decides hard constraint 2 and the quoting record.
- Gate: hard constraint 2 (runs an arbitrary command); native overlay.
- Status: pass, with a platform difference that decides E16.
- Source:
  - The trigger definition's `command` must be a JSON array:
    `watchman/TriggerCommand.cpp:331-335`
    ```cpp
    if (!cmd || !cmd->isArray() || !json_array_size(*cmd)) {
      throw CommandValidationError("invalid command array");
    }
    ```
    and `website/docs/cmd/trigger.md:79-81`:
    "It must be an array of string values; this will form the argv array of the trigger process."
  - It is spawned with `posix_spawnp(&pid_, argv[0], ..., &argv[0], envp.get())` in
    `watchman/ChildProcess.cpp:326-333`, after building `argv` from the JSON strings at `:283-294`.
  - On Windows that same call is Watchman's own `posix_spawn` emulation, and it always prefixes
    the command line with `cmd.exe /c "`:
    `watchman/portability/PosixSpawn.cpp:200` `#define CMD_EXE_PREFIX "cmd.exe /c \""`,
    used at `:221`, with `CreateProcess(...)` at `:467-478`.
- Access date: 2026-11-25.
- Outcome: pass for "runs an arbitrary command".
  On Linux and macOS the command is an argv array with no shell, so a target such as
  `systemctl --user start rclone-bisync-garage.service` is invoked exactly as given.
  On Windows the command string passes through `cmd.exe`, so shell syntax applies there (E16).

#### E15 Where the changed paths go: argv, stdin, or the environment

- Claim: the integration boundary for a `systemctl` command, and what a wrapper would see.
- Gate: hard constraint 2; capability record.
- Status: pass (recorded, not a gate for this job).
- Source:
  - `append_files` defaults to false (`watchman/TriggerCommand.cpp:337`)
    and, when true, appends the changed names to the argv array
    (`:205-230`) until the space computed from `ChildProcess::getArgMax()` runs out,
    setting `WATCHMAN_FILES_OVERFLOW` when it does (`:232`).
  - `stdin` defaults to `input_dev_null` (`:304`, `:349-351`), accepts a field list as JSON
    (`:352-354`) or the literal `NAME_PER_LINE` (`:359-362`),
    and truncates to `max_files_stdin` (`:370-375`, default 0 meaning no limit).
  - Environment: `WATCHMAN_ROOT`, `WATCHMAN_SOCK`, `WATCHMAN_TRIGGER` at `:381-384`;
    `WATCHMAN_CLOCK` and `WATCHMAN_SINCE` at `:185-194`;
    `WATCHMAN_RELATIVE_ROOT` at `:196-200`.
  - Working directory: root or `relative_root` (`:257-272`), overridable with `chdir`.
  - Output: stdout and stderr are inherited from the daemon unless redirected, so they land in the
    Watchman log (`:245-255`, and `website/docs/cmd/trigger.md:12-14`).
- Access date: 2026-11-25.
- Outcome: for this job's `systemctl` command, no paths need to be passed anywhere:
  the command takes no arguments from Watchman, and the trigger's value is "something changed".
  A wrapper around it can read the names from `NAME_PER_LINE` stdin if needed.

#### E16 Quoting, and names with spaces, brackets or quotes

- Claim: the owner's file names (Japanese, Chinese, spaces, parentheses, brackets, quotes) survive.
- Gate: hard constraint 2; capability record.
- Status: pass on POSIX with a documented limit; open on Windows.
- Source:
  - POSIX argv carries each name as its own NUL-terminated argument, so spaces, parentheses,
    brackets, quotes and non-ASCII bytes need no quoting
    (`watchman/ChildProcess.cpp:283-294`, `:326-333`).
  - `NAME_PER_LINE` frames by newline and documents no quoting:
    `website/docs/cmd/trigger.md:125-127`
    "No quoting will be applied to the names, and they may contain spaces."
    A name containing a newline therefore breaks the framing for that consumer.
  - JSON stdin needs valid UTF-8; issue 441 (open since 2017-03-09) records
    `input_json: failed to write json data to stream` for names that were not valid UTF-8,
    and the maintainer reply proposes BSER "so that we can just pass through the binary bytes".
  - Windows wraps the whole argv in `cmd.exe /c "` and re-quotes each argument by wrapping it in
    `"` and expanding an embedded `"` to `"""`:
    `watchman/portability/PosixSpawn.cpp:223-256`, with the source's own comment
    "Surely not complete here by any means" at `:218-219`.
- Access date: 2026-11-25.
- Outcome: on Linux, which is the only machine in scope for this session, names are passed as
  bytes in argv or on stdin and there is no quoting stage to break them.
  Valid UTF-8 (as Japanese and Chinese names are) is fine for JSON stdin as well.
  On Windows, names appended to the command line pass through `cmd.exe`'s parser with
  a hand-rolled quoting routine; the stdin routes avoid that parser, so a later Windows
  integration should prefer stdin over `append_files`.

### Busy behaviour, the queue, and never interrupting a run

This is where the discovery run's reading of the documentation and the released source disagree.
The documentation says the friendly thing; the code does the other thing.

#### E17 On POSIX, a new batch kills the running trigger command (source)

- Claim: what happens when changes arrive while the trigger's command runs.
- Gate: hard constraints 5, 6 and 7.
- Status: fail for 5, 6 and 7 on Linux and macOS.
- Source:
  - `watchman/TriggerCommand.cpp:462-527` `maybeSpawn` runs the since-query on every settled batch
    and calls `spawn_command` when it matches anything (`:513-516`).
  - `watchman/TriggerCommand.cpp:274-280` is the kill, immediately before the new spawn:
    ```cpp
    try {
      if (cmd->current_proc) {
        cmd->current_proc->kill();
        cmd->current_proc->wait();
      }
      cmd->current_proc = std::make_unique<ChildProcess>(
          json_array(std::move(args)), std::move(opts));
    ```
  - The trigger thread never waits for the child before evaluating the next batch:
    `:406-429` polls for the next settle and calls `maybeSpawn` again;
    `waitNoIntr` (`:529-537`) only reaps an already-terminated child and is called once (`:426`).
  - `kill` is `::kill(pid_, signo)` with `signo = SIGTERM` and no process-group signal:
    `watchman/ChildProcess.cpp:420-431`, `watchman/ChildProcess.h:155-159`;
    then `wait()` blocks until the child exits (`ChildProcess.cpp:399-418`).
  - The same kill runs when the daemon shuts down: `watchman/TriggerCommand.cpp:431-434`.
- Access date: 2026-11-25.
- Outcome: exactly one of queue, drop, kill or run concurrently applies, and it is **kill**:
  the trigger process is SIGTERM'd, waited for, and replaced by a run whose query was evaluated
  against the clock from the previous spawn.
  `wait()` after SIGTERM means a child that ignores SIGTERM stalls the trigger thread indefinitely.

#### E18 The documentation claims the opposite, and an open issue confirms the kill

- Claim: whether E17 is the known, intended behaviour.
- Gate: hard constraints 5 to 7; evidence quality.
- Status: fail; documentation and tracker disagree with the code.
- Source:
  - `website/docs/cmd/trigger.md:23-28`:
    "Watchman will only run a single instance of the trigger process at a time. ...
    When the process terminates, watchman will re-evaluate the trigger criteria based on the clock
    at the time the process was last spawned".
  - Issue 703, `Race condition with triggers`, open since 2019-05-16, last comment 2019-06-17,
    filed against release 4.9.0 and `master` on macOS with FSEvents, by a user driving `rsync`:
    "I see that Watchman tries to ensure that only one instance of each trigger's command is
    running at a time, which I like. As part of enforcing this, however, it sends SIGTERM to the
    currently running instance of a command when it's time to start the next, which I don't like.
    I don't want any of my `rsync` invocations to be aborted mid-transfer."
  - Issue 447, answered by the original author in 2017: "Neither `trigger` nor `watchman-make`
    kill off the running process."
    The code at this tag does kill, so that answer no longer describes `trigger`.
- Access date: 2026-11-25.
- Outcome: the code is the deciding evidence; the shipped documentation and the 2017 answer are stale,
  and issue 703 is an open report of the exact behaviour this job's owner forbids, with maintainer
  engagement in 2019 and no fix or closure since.
  For this job the consequence is concrete: on macOS the later client would have its `rclone`
  killed mid-run, and on Linux the kill lands on `systemctl`, whose own SIGTERM handling
  (design record, Owner answers round 4) does not stop the unit, so the observed effect differs
  by platform even though the watcher's behaviour does not.

#### E19 On Windows, the trigger thread blocks instead, which serialises without a queue

- Claim: whether Windows behaves differently.
- Gate: hard constraints 5 to 7 (platform-wide requirement).
- Status: fail as a native queue; Windows does serialise, but by blocking.
- Source:
  - `watchman/ChildProcess.cpp:420-431` is compiled with the `_WIN32` branch empty,
    so `kill()` is a no-op there.
  - In `watchman/TriggerCommand.cpp:274-278`, the `kill()` then `wait()` pair therefore waits for
    the running child on Windows before the new one is created.
  - The wait blocks the trigger thread, which is the only consumer of settle notifications
    (`:406-429`), and the publisher keeps items for the slowest subscriber
    (`watchman/PubSub.cpp:57-107` walk to the subscriber's serial; `:120-136` collect garbage
    only below the minimum serial).
- Access date: 2026-11-25.
- Outcome: on Windows a change during a run waits for the running command and then produces one more
  run, which satisfies hard constraints 5 and 6 there and not 7 (nothing is queued by the watcher;
  the batching is a side effect of blocking).

#### E20 A trigger fires once at registration, for the files already there

- Claim: operational side effect at setup, recorded because it decides what the first run does.
- Gate: capability record.
- Status: pass (recorded).
- Source:
  - `watchman/integration/test_trigger.py:57-60` asserts exactly this:
    `message="both triggers fire at start"`.
  - Issue 440 is the feature request for a way to turn it off
    ("[RFE] First trigger invocation lists all files, optionally turn this behavior off", open).
  - `maybeSpawn` with no saved `since` clock runs a query with no `since` and matches everything
    (`watchman/TriggerCommand.cpp:473-516`).
- Access date: 2026-11-25.
- Outcome: registering the trigger would start one sync immediately for the existing tree
  (10332 files), not only on later changes.

#### E21 The trigger and its watch come back when the daemon starts

- Claim: whether the setup survives a daemon restart or a reboot.
- Gate: capability record; security-adjacent (autostart).
- Status: pass (recorded).
- Source:
  - `watchman/state.cpp:168-171` writes each watched root's `path` and
    `root->triggerListToJson()` under `"triggers"` in the state file.
  - `watchman/state.cpp:77-116` `w_state_load` reads that file unless `--no-save-state`,
    and calls `w_root_load_state`; `watchman/main.cpp:208` calls it during daemon startup.
  - `watchman/state.cpp:182-235` re-resolves every saved root
    (`root_resolve(filename, true, &created)`) and re-creates each trigger
    (`new TriggerCommand(getInterface, root, tobj)` then `cmd->start(root)`).
  - `website/docs/cmd/trigger.md:30-33`: "Unless `no-save-state` is in use, triggers are saved and
    re-established across a Watchman process restart."
  - `watchman/Options.cpp:125-131` defines `--no-save-state`,
    "Don't save state between invocations".
- Access date: 2026-11-25.
- Outcome: a daemon start restores both the watch and the trigger from the state file,
  without a client asking for it.
  Combined with the macOS LaunchAgent (E29), that means the setup resumes at login on macOS;
  on Linux it resumes when anything starts the daemon again, and `--no-save-state` turns it off.

### Further capability records

#### E22 Quiet period: `settle`, default 20 ms

- Claim: the debounce option and its default.
- Gate: capability record.
- Status: pass.
- Source:
  - `watchman/root/init.cpp:21-23` `inline constexpr json_int_t kDefaultSettlePeriod = 20;`
    and `:260` `trigger_settle(int(config.getInt("settle", kDefaultSettlePeriod)))`.
  - `website/docs/config.md:65-70` "Specifies the settle period in milliseconds. This controls how
    long the filesystem should be idle before dispatching triggers. The default value is 20 milliseconds."
  - The settle itself fires in the iothread: `watchman/root/iothread.cpp:106-119`
    enqueues `{"settled": true}` once nothing is pending.
- Access date: 2026-11-25.
- Outcome: `settle` in the root's `.watchmanconfig`, default 20 ms.
  Inference from where the value is read (the `Root` constructor): changing it takes effect when the
  root is created again, not live.
  Two platform latencies sit underneath it: `fsevents_latency` 0.01 s (E10) and
  `win32_batch_latency_ms` 30 (E11).

#### E23 Path exclusion: `ignore_dirs`, `ignore_vcs`, and the trigger expression

- Claim: whether paths can be excluded.
- Gate: capability record.
- Status: pass, with the exact shape recorded.
- Source:
  - `watchman/root/init.cpp:200-219` reads `ignore_dirs` as an array of names, joins each to the root
    (`w_string::pathCat({root_path, name})`) and adds it as a full ignore.
  - `watchman/IgnoreSet.cpp:11-24` records `FULL_IGNORE` for those paths, and `:28-89`
    `isIgnored` matches the path and everything below it, requiring a directory separator after the
    prefix so `build` does not match `builder`.
  - `website/docs/config.md:172-192`: "Dirs that match are completely ignored by watchman ...
    On Linux systems, `ignore_dirs` is respected at the OS level; the kernel simply will not tell
    watchman about changes to ignored dirs."
  - `website/docs/config.md:163-170`: `ignore_vcs` defaults to `[".git", ".hg", ".svn", ".jj"]`
    and is a shallow watch, not a full ignore.
  - `watchman/TriggerCommand.cpp:310-320` and `:103-106` of the trigger page: the trigger's own
    `expression` (a query expression, including `pcre`) decides which changed files can fire it.
- Access date: 2026-11-25.
- Outcome: directories only, by root-relative path, plus a query expression that can filter files.
  There is no file-name glob exclusion inside `ignore_dirs`, so a rule such as
  "ignore `*.partial`" belongs in the trigger expression, not in `ignore_dirs`.

#### E24 Linux inotify watch limit exhaustion: a loud, global poison state

- Claim: what happens when `fs.inotify.max_user_watches` runs out.
- Gate: capability record.
- Status: pass (error, no silent loss, no fallback).
- Source:
  - `watchman/watcher/inotify.cpp:218-227` throws `std::system_error(err, inotify_category(), "inotify_add_watch")`
    when `inotify_add_watch` fails, for example with `ENOSPC`.
  - `watchman/Errors.cpp:47-49` maps `ENOSPC` to
    "The user limit on the total number of inotify watches was reached; increase the
    fs.inotify.max_user_watches sysctl".
  - `watchman/Errors.cpp:114-123` makes `ENOSPC`, `ENOMEM`, `EMFILE` and `ENFILE`
    equivalent to `error_code::system_limits_exceeded`.
  - `watchman/root/warnerr.cpp:30-35` turns that into `set_poison_state(...)` and records
    `root.failure_reason`; `watchman/Poison.cpp:16-41` writes the reason once and refuses to overwrite it.
  - `watchman/Client.cpp:127-131` answers every command that is not `CMD_POISON_IMMUNE` with the
    poison text, so the failure is visible to the next client.
  - The moved-in directory path also poisons directly on `ENOSPC`/`ENOMEM`
    (`watchman/watcher/inotify.cpp:310-317`).
- Access date: 2026-11-25.
- Outcome: an error, not silent loss and not a fallback watcher.
  The daemon stays up, the affected root is marked failed, and every later request returns the
  poison message until the daemon is restarted with a raised limit.
  A separate overflow path exists for the kernel's event queue: `IN_Q_OVERFLOW` schedules a recrawl
  (`watchman/watcher/inotify.cpp:255-259`, "we missed something, will need to re-crawl").

#### E25 The daemon keeps a trigger's watch alive past `idle_reap_age_seconds`

- Claim: whether a quiet tree loses its watch, which the earlier editord handover worried about.
- Gate: capability record.
- Status: pass (recorded, and it corrects an earlier repository claim).
- Source: `watchman/root/reap.cpp:14-33`:
  ```cpp
  if (now > inner.last_cmd_timestamp.load(...) + idle_reap_age &&
      (triggers.rlock()->empty()) && (now > inner.last_reap_timestamp) &&
      !unilateralResponses->hasSubscribers()) {
  ```
  Default age `86400 * 5` at `watchman/root/init.cpp:21`.
- Access date: 2026-11-25.
- Outcome: a registered trigger keeps the root from being reaped, because the condition requires
  `triggers` to be empty.
  The editord handover's "leaves a watch ... until a 5-day idle window" holds only without a trigger
  or subscription; with one, the watch persists while the daemon runs.

### Security record

#### E26 Sockets and listeners

- Claim: what listens, and who can talk to it.
- Gate: security boundary.
- Status: pass.
- Source:
  - `watchman/sockname.cpp:18-26` returns `flags.unix_sock_name` on POSIX and
    `flags.named_pipe_path` on Windows; the names are computed in
    `watchman/main.cpp:644-651` (`\\.\pipe\watchman-<user>` on Windows, a Unix socket elsewhere).
  - `watchman/listener.cpp:67-104` creates a `PF_LOCAL` `SOCK_STREAM` socket, binds it,
    and `chmod`s it with the permissions from `cfg_get_perms("sock_access", true, false)`;
    an optional `secondary_sock_group` ACL exists for Linux at `:105-120`.
  - `watchman/main.cpp:597-613` disables the Unix socket on Windows if a `PF_LOCAL` socket cannot be made.
  - There is no TCP listener: the only network-ish option is a hidden `--inetd`
    "Spawning from an inetd style supervisor" flag (`watchman/Options.cpp:33-38`),
    which reuses a supervisor-provided descriptor (`watchman/listener.cpp:52-64`).
  - `grep -rn "AF_INET\|getaddrinfo" watchman/` finds nothing outside tests and client `connect()`
    calls that target the local socket.
- Access date: 2026-11-25.
- Outcome: one local socket or named pipe per user, permissions configurable, no TCP.
  Anything that can write to that socket can register triggers and run commands as the user;
  that is the tool's trust boundary and it is the same user boundary as a shell.

#### E27 State directory permissions and ownership checks

- Claim: whether another local user can hijack the daemon's state.
- Gate: security boundary.
- Status: pass.
- Source:
  - Default state directory: `watchman/UserDir.cpp:195-219`,
    `<TMPDIR or XDG state or WATCHMAN_STATE_DIR>/<user>-state`.
  - `watchman/PathUtils.cpp:154-184` creates it with `chmod(state_dir, 0700)` when it creates it.
  - `watchman/PathUtils.cpp:32-72` `verify_dir_ownership` refuses a state directory whose owner is not
    the effective user ("the owner of ... doesn't match your euid"), and forces group inheritance with
    `S_ISGID` at `:45-48`.
  - `website/docs/install.md:106-112` and `:176-182` tell a user installing by hand to
    `chmod 2777 /usr/local/var/run/watchman`, the shared parent from
    `CMakeLists.txt:114-117`; the per-user subdirectory is the 0700 one that is verified.
- Access date: 2026-11-25.
- Outcome: pass. The shared parent is world-writable by design for a system install;
  the per-user directory under it is owner-checked.

#### E28 No network access, no update check, and telemetry off by default

- Claim: whether the daemon phones home.
- Gate: security boundary.
- Status: pass.
- Source:
  - The structured logger is constructed with the `scribe-cat` and `scribe-category` config strings:
    `watchman/telemetry/WatchmanStructuredLogger.cpp:129-150`.
  - In `facebookexperimental/edencommon` at `4db211e`,
    `eden/common/telemetry/StructuredLoggerFactory.h:28-37` returns
    `std::make_shared<NullStructuredLogger>()` when the binary string is empty, which is the default
    (`cfg_get_string("scribe-cat", "")`), and only builds a `SubprocessScribeLogger` when both
    settings are present.
  - The Meta-internal Xplat logger is compiled only under `WATCHMAN_FACEBOOK_INTERNAL`
    (`watchman/telemetry/WatchmanStructuredLogger.cpp:17-24`, `:108-118`).
  - No update check exists: `grep -rn "curl\|http://\|https://" --include=*.cpp watchman/` finds only
    documentation URLs (for example the trouble-shooting link at `watchman/WatchmanConfig.cpp:392`),
    and `watchman/main.cpp` has no HTTP client.
- Access date: 2026-11-25.
- Outcome: no network egress by default and no update check.
  A user who sets `scribe-cat` and `scribe-category` opts into a subprocess delivery path.

#### E29 What the daemon does at startup, and what the macOS build installs

- Claim: privileged operations, persistence, and autostart.
- Gate: security boundary.
- Status: pass (recorded).
- Source:
  - Linux: the client forks and daemonizes (`watchman/main.cpp:267-300`, `:855-859`),
    after dropping to `/` and redirecting stdio to a 0600 log (`:139-162`).
    No unit, no `systemd` call anywhere in the tree.
  - Windows: the client spawns a detached `--foreground` child (`watchman/main.cpp:310-343`);
    no service or scheduled task registration.
  - macOS: the client writes `~/Library/LaunchAgents/com.github.facebook.watchman.plist`
    with `RunAtLoad` and `KeepAlive` `Crashed`
    (`watchman/main.cpp:442-548`), `chmod 0644` (`:546`), then runs
    `/bin/launchctl load -F <plist>` (`:549-553`), falling back to plain daemonizing (`:563`).
  - Windows elevation: the client de-elevates only when
    `should_deelevate_on_startup` is set (`watchman/main.cpp:881-891`),
    with the comment that an elevated daemon cannot be reached from a non-elevated client.
- Access date: 2026-11-25.
- Outcome: no privileged operations on Linux; a user-level LaunchAgent on macOS that restarts a
  crashed daemon at login; a detached process on Windows.

## Repository precedent (counterevidence)

Every verdict this repository already recorded about Watchman, found by searching `doc/` and package
docs for the name, with whether it bears on this job.

- `package-paused/desktop-daemon/editord/HANDOVER.custom-dev-server-watcher.md:143-163`
  rates it for the editord dev loop:
  "Fit: would work, but not justified for this case",
  "watchman is architecturally a long-lived daemon designed to amortise its cost across many tools
  and many directories. We have one tool watching one directory.",
  and adds that its "DSL is metadata-only (no content access)".
  The same file's section "Can we just use watchman?" (`:368-470`) spells the reasoning out,
  and its "When this should be revisited" list (`:448-462`) names a larger tree, several dev servers,
  `scm-query` needs, or an existing Python toolchain.
  Bearing on this job: the scale argument is the same shape (one tool, one folder),
  and the tree here is 1454 folders and 10332 files, not a tens-of-files directory.
  Two of the recorded costs do not apply to the trigger route:
  `watchman-make` and `pywatchman` are not needed to register a trigger (the JSON protocol is enough),
  and E25 shows a registered trigger prevents the 5-day reaping the handover describes.
  The handover's verdict is a cost judgement, not a hard-gate result, so it is counterevidence
  rather than a failure.
- `package/watch-restart/cli/README.md:331-335` repeats it for the restart tool's watcher choice:
  "watchman (daemon designed to amortise across many tools; one project is the wrong scale)".
  Same bearing.
- `package/watch-restart/cli/HANDOVER.implementation-state.md:29-31` points back at that analysis.
  No independent verdict.
- `doc/audit/tech-monorepo-manager-vet-2026-09-16/screening-registry-chunk-3.md:1161-1199`
  exits Watchman at gate G3 "coverage (c)" for that decision:
  "no extension or plugin mechanism page exists".
  Bearing on this job: none. The requirement that drove it (a monorepo manager's extension surface)
  is not a requirement here.
- `doc/audit/tech-monorepo-manager-vet-2026-09-16.md:850-855` exits it again at a re-screen for
  a documentation confusion trigger:
  the install page said the Linux binaries build on Ubuntu 20.04 while the release workflow used
  `ubuntu-24.04`, and kept a macOS 10.6 section.
  Bearing on this job: the stale text is gone at this tag.
  `website/docs/install.md:145` now says the `.deb` works on "Ubuntu 24.04, Debian 13 and newer"
  and `:281` still has the macOS 10.6 section, marked "Only applicable on macOS 10.6 and earlier".
- `doc/planning/monorepo-manager-route-research/stack-node-libraries.md:237-252` measured
  "The latest Watchman release `v2026.09.14.00` has 0 assets; the newest release with assets is
  `v2026.07.27.00`" and `:744-760` ranks Watchman fifth of eight for that decision, disqualifying it
  for "an external daemon the user must install and keep running; no current release assets, the
  newest RPM targets Fedora 42 while the host is Fedora 44, and `mise` has no `watchman` entry".
  Bearing on this job: the asset claim has changed.
  On 2026-11-25 the newest three releases have assets, and `v2026.11.23.00` is the first in the last
  30 with macOS and Windows zips (E33). The "no `mise` entry" and "external daemon" costs remain.
- `doc/planning/monorepo-manager-route-research/from-scratch-inputs.md:663-700` reads the trigger
  documentation as a design input and quotes `website/docs/cmd/trigger.md:23-24`
  ("Watchman will only run a single instance of the trigger process at a time").
  Bearing on this job: it is the same documentation claim E17 and E18 refute at source level,
  and the vet report's own screening step inherited it.
- `doc/decision/ios-iphone-x-vet-report/vet-react-native.md:337` and
  `doc/handover/ios-framework-vet.md:398` name "Watchman (recommended)" and
  "React Native community CLI + Watchman/Metro" as the React Native toolchain.
  Bearing on this job: none. Those are toolchain requirements of another product, not verdicts on
  Watchman as a change-to-command watcher.
- `doc/planning/monorepo-manager-from-scratch-design.md:3080` and
  `doc/planning/monorepo-manager-route-research/stack-rust.md:509` cite "Watchman clocks" as a design
  precedent for a clock model.
  Bearing on this job: none, and no verdict.
- `doc/planning/cli-git-no-self-invocation-dispatcher.md:171` mentions
  `hook.fsmonitor-watchman.enabled=false` as a git config to avoid a hook loop.
  Bearing on this job: none; it is about git's fsmonitor integration, not this candidate.
- `doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`
  "Repository precedent missed by discovery" itself records the editord "not justified" verdict and
  says Watchman "stays, with its 'not justified' verdict recorded as counterevidence".
  This file adds the source-level records that make that verdict checkable.

## Maintenance audit

Query set, all run on 2026-11-25 against `meta/watchman`:
`gh api repos/meta/watchman` for repository state;
`gh api -X GET search/issues -f q='repo:meta/watchman is:issue updated:>2025-11-25'` (and the `is:pr`,
`is:open`, `is:merged` variants) for volume;
`gh api -X GET repos/meta/watchman/commits -f since=2025-11-25T00:00:00Z --paginate` for authors;
`gh api repos/meta/watchman/releases` for releases and assets.

#### E30 Tracker volume and repository state

- Claim: how much public activity the tracker shows.
- Gate: maintenance audit.
- Status: recorded.
- Numbers:
  - repository: `archived:false`, `stargazers_count:13773`, `open_issues_count:260`
    (220 issues plus 40 pull requests by the search counts), `pushed_at:2026-11-25T20:09:29Z`,
    `license.spdx_id:MIT`.
  - 33 issues updated in the last 12 months; 43 pull requests updated in the last 12 months;
    220 open issues; 40 open pull requests.
  - 864 commit objects in the last 12 months by the commits API (863 by the search API), of which
    531 carry a human author name.
- Access date: 2026-11-25.
- Outcome: the volume of code change is high; the volume of public issue traffic is low,
  because the repository is a mirror (E34).

#### E31 The ten most recently updated issues

- Claim: maintainer responsiveness on the public tracker.
- Gate: maintenance audit.
- Status: recorded; two of ten got a maintainer reply.
- The ten, newest update first, all updated inside 12 months:
  1363, 1372, 1366, 1359, 1364, 1349, 1354, 1299, 1361, 1355.
- Findings:
  - Three of the ten are closed by `meta-codesync[bot]` with `state_reason: completed`:
    1363 and 1372 with zero comments, and 1349 after a maintainer reply.
    The bot closure mirrors a closure inside Meta's internal tracker and leaves the public reporter
    no explanation of the outcome.
  - Maintainer comments exist on two: genevievehelsel on 1366 (cookies left in a Git worktree's CWD),
    and MichaelCuevas on 1349, where he writes
    "Thanks for flagging! I've updated the licenses in source, and I'll publish new NPM package
    versions soon."
  - The other eight have no maintainer comment, and none of the ten carries a label:
    1359 (macOS binaries unsigned, zero comments since 2026-08-21),
    1364 (Windows drive root, zero comments),
    1354 (recrawl loop, four comments, all the reporter's and one other user's),
    1299 (prebuilt binaries missing, six comments, all external, open since 2025-08-21),
    1361 (serde_bser release, two external comments),
    1355 (build error, one external comment).
- Access date: 2026-11-25.
- Outcome: active development with weak public issue support.
  Bugs that touch this job sit unanswered: unsigned macOS binaries (1359),
  missing prebuilt binaries in many weeks (1299), a Windows indexing bug (1364).

#### E32 The ten most recently updated pull requests

- Claim: whether outside fixes land, and how review works.
- Gate: maintenance audit.
- Status: recorded; one merge in 12 months, and it was a bot re-sync.
- The ten, newest update first:
  1376, 1371, 1375, 1373, 1340, 1365, 1370, 1369, 1368, 1362; plus 1347 because it fixes E12's dead branch.
- Findings:
  - `search/issues q='repo:meta/watchman is:pr is:merged merged:>2025-11-25'` returns exactly one
    result: 1376 "Re-sync with internal repository", opened and merged by `meta-codesync[bot]` on
    2026-11-24.
  - 1371, 1373, 1375, 1340 and 1370 were closed unmerged, all by the bot;
    1370 did receive `CHANGES_REQUESTED` from genevievehelsel before closure.
  - 1365 (symlink hashing), 1369, 1368, 1362, 1358 and 1347 are open with no maintainer review;
    1347 "Fix never-true IN_MOVED_TO mask check in inotify watcher" has been open since 2026-05-17
    with no review comment.
  - Reviews recorded: one, on 1370. Everything else has no review; 1347 and 1358 return zero reviews
    as well.
  - `CONTRIBUTING.md:1-8` states the model plainly:
    "Watchman is currently developed in Meta's internal repositories and then exported out to GitHub
    by a Meta team member; however, we invite you to submit pull requests as described below."
- Access date: 2026-11-25.
- Outcome: the public pull-request path is not how code lands.
  A fix a user wants cannot be expected to arrive through a public PR, and one that fixes a watcher bug
  in this job's area has sat unreviewed for about six months.

#### E33 Release cadence and whether assets exist

- Claim: weekly releases, and whether the three target platforms ship.
- Gate: maintenance audit; provenance.
- Status: recorded.
- Numbers from `gh api repos/meta/watchman/releases -f per_page=30`:
  - weekly tags, all `draft:false` and `prerelease:false`, named `vYYYY.MM.DD.00`,
    and published on the Monday in the name (the target release's tag commit is dated
    2026-11-22, and it was published 2026-11-23).
  - of the 30 most recent, 14 published no assets at all, including 13 consecutive weeks from
    2026-08-03 to 2026-10-26.
  - the target release `v2026.11.23.00` has 8 assets, and is the only release in those 30 with a macOS
    zip, a Windows zip, and a Linux arm64 zip.
    The two before it ship only the Linux x86-64 zip, an amd64 `.deb` and an x86_64 `.rpm`;
    earlier asset-bearing weeks ship an `fc42` x86_64 `.rpm` and the Linux x86-64 zip.
  - `website/docs/install.md:162-167` warns about exactly this:
    "That process is in a beta state; some or all of the binaries may not be present for any given tag."
  - Issue 1299 (open since 2025-08-21) is users hitting that;
    PR 1360 "Restore macOS release binaries on Apple Silicon" is open since 2026-08-21.
- Access date: 2026-11-25.
- Outcome: a weekly cadence that is real, with an artifact pipeline that is intermittent.
  For this candidate and this tag the three platforms are present, which is what the gate needs,
  but a later pin to a newer weekly tag may find no macOS or Windows artifact.

#### E34 Maintainer concentration

- Claim: who does the work, and whether the repository is a mirror.
- Gate: maintenance audit.
- Status: recorded.
- Numbers, from 864 commits in the last 12 months:
  - 323 are authored by "Open Source Bot" and 10 by "GitHub Workflows Support";
    531 carry a human author name.
  - Of those 531: Genevieve (Genna) Helsel 76, Richard Barnes 73, Victor Zverovich 27,
    Muir Manders 20, Michel Lind 20, Xiaowei Lu 17, Alan Frindell 17, Indu Suresh 14,
    Davis Rollman 11, David Tolnay 10.
    The top two are 149 of 531 (28 per cent), the top ten are 285 of 531 (54 per cent).
  - Every recent commit's committer is `meta-codesync[bot]`;
    the eight newest commits on `main` on 2026-11-25 all have that committer.
  - All GitHub issue closures inspected in E31 came from the same bot.
- Access date: 2026-11-25.
- Outcome: a Meta team of roughly a dozen people maintains it actively, exported through one bot.
  Concentration is moderate by headcount and total by authority: the public side has no merge path
  (E32), and one bot authors or commits the mirror.

#### E35 Security reporting, stale bugs, and abandonment signals

- Claim: whether a user with a security or core-behaviour problem has a route.
- Gate: maintenance audit.
- Status: recorded.
- Findings:
  - No `SECURITY.md` at the tag (`ls -a` of the root shows `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`,
    `LICENSE`, no security file).
  - `CONTRIBUTING.md:20-24` sends security reports to Meta's bounty programme:
    "Meta has a bounty program ... for the safe disclosure of security bugs. In those cases, please
    go through the process outlined on that page and do not file a public issue."
  - Stale, unaddressed core bugs found in the issue search:
    703 (triggers SIGTERM the running command, open since 2019-05-16, last activity 2019-06-17),
    441 (non-UTF-8 names break JSON trigger stdin, open since 2017-03-09),
    440 (no way to suppress the first full trigger run, open since 2017-03-09),
    1359 (unsigned macOS binaries, open since 2026-08-21).
  - No abandonment comment or deprecation notice was found in the inspected items,
    and the weekly tags are current.
- Access date: 2026-11-25.
- Outcome: active releases with weak public issue support, and one core behaviour this job depends on
  has been reported and ignored since 2019.

## What this run did not do

- No install, build, test, or execution of Watchman or any of its scripts, binaries, or containers.
  The packaging scripts and workflows in E03 to E05 were read, not run.
- No clone of `facebookexperimental/edencommon`; only one file was fetched by API for E28.
- No runtime measurement of the kill behaviour in E17 to E19.
  It is read from source and corroborated by issue 703, which is a user report, not a reproduction
  performed here.
  The skill's external execution gate and finalist validation remain for a later run.
- No review of the C++ source beyond the paths that decide the constraints and capabilities above.
  Large parts of `watchman/` were not read: the SCM query integration, the Python and Node clients,
  the Windows de-elevation helper, the saved-state format, and the test suite outside the trigger tests.
- No inspection of distro or AUR packages, which are third-party builds and not the tag's artifacts.
- No score, no ranking, no recommendation, and no edit to the vet report.

## Where this file departs from the prompt or the vet report

- The vet report's ledger entry says the prebuilt boundary comes with "checksum files".
  There are none: the eight assets are binaries only, and the only digests are GitHub's own
  `sha256:` fields (E06).
- The vet report records as capability that "triggers run one process at a time and re-evaluate after
  it exits", and its round-4 screening sends Watchman on with the queue requirement "met natively".
  The released source kills the running trigger command before the next spawn (E17),
  the documentation's claim to the contrary is stale (E18), and open issue 703 reports the kill
  as the behaviour since at least 4.9.0.
  This changes the outcome of three hard constraints, not just the capability note.
- The vet report describes Watchman as "one process at a time" queueing; on Windows there is no kill
  and the trigger thread blocks instead (E19), so the platform behaviours differ while neither is a queue.
- The vet report's install-page claim ("Windows 10 64-bit and up") was not re-checked here;
  this file records what the tag's source and assets support and leaves the OS floor to a later run.
- `AGENTS.md` asks for long-form options and no external `timeout`; both were kept.
  The one deliberate deviation from the prompt's instruction set is that nothing in the candidate was
  executed, which the prompt itself required.

## Summary: each hard constraint as pass, fail or open

Vet report hard constraints, in its order, with the record that decides each.

- **Open source with inspectable source**: pass. E01 (MIT, `LICENSE:1-3`), E02 (tag `v2026.11.23.00`
  is commit `389ebc8` and the whole tree is readable).
- **Runs an arbitrary command, here `systemctl --user start rclone-bisync-garage.service`**: pass.
  E14 (JSON argv array through `posix_spawnp` on Linux and macOS; `cmd.exe /c` on Windows),
  with E16 recording the quoting limits when paths are appended.
- **Runs on Linux x86-64, macOS arm64 and Windows x64**: pass.
  E03, E04 and E05 map each artifact to the tag through the committed workflows and the release assets;
  E09, E10 and E11 name the backend in source for each system.
  Limits recorded: no signature or notarization (E06), macOS 15.0 deployment target (E04),
  dependency resolution by commit time (E08), and an intermittent artifact pipeline (E33).
- **Watches every folder under the root recursively, including folders created after it starts**: pass.
  E12 for Linux (new directory triggers a recursive crawl and `inotify_add_watch` per directory),
  E13 for macOS (FSEvents stream on the root) and Windows (`bWatchSubtree = TRUE`).
- **After a change that lands while a sync runs, one more sync starts after that run ends**:
  fail. E17 (on Linux and macOS the running trigger command is SIGTERM'd and waited for before the
  next spawn), E18 (the documentation says otherwise; issue 703 reports the kill).
  On Windows E19 serialises by blocking, so that platform would satisfy this;
  the hard requirement is platform-wide and fails on the Linux machine in scope.
- **Never interrupts a running command; each run finishes**: fail.
  E17, with E18 as corroboration.
  On Linux the killed process is `systemctl`, whose SIGTERM does not stop the unit,
  so the visible effect there differs from macOS, where the kill would land on `rclone` itself.
- **Queues one more run of its command natively when events arrive while the command runs**: fail.
  E17 (there is no queue; the running child is killed and replaced), E19 (Windows blocks instead of
  queueing).

Result: three of the seven hard constraints fail on the released source, so this candidate does not
reach finalist validation in this vet on the evidence in this file.
