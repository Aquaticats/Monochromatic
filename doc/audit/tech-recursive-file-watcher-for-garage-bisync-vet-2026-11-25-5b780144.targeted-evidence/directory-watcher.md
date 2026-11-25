# Targeted evidence: directory-watcher 3.2.1

Status:
 targeted evidence for one candidate of the vet report
 `doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`;
 gathered by a Pi run on 2026-11-25;
 not reviewed,
 not approved by the owner,
 not a score,
 not a ranking and not a recommendation.

## What this record covers

The vet report asks targeted evidence for the candidates it sends on,
against the hard constraints in its "Hard constraints" section
and the design decisions in
`doc/planning/garage-bisync-run-on-file-change.md`
sections "Owner answers, round 1" to "Owner answers, round 4".
This file answers that for directory-watcher 3.2.1 only.

Read-only rule for this run:
the candidate's source was cloned and read,
and nothing of it was installed,
built,
tested,
scripted,
or run.
No binary and no container of it was started.
Every record below therefore comes from source,
from release metadata,
or from the release workflow,
never from watching the candidate behave.

Common context for every record:

- Access date: 2026-11-25.
- Host: CachyOS, Linux 7.3.5-2-cachyos, x86_64, 2 CPUs.
- Working directory: `/home/user/Monochromatic`, branch `main`.
- Clone path:
  `/home/user/temp/agent/directory-watcher-source-for-garage-watcher-vet-2026-11-25`.
- Source paths in this file are relative to that clone root unless they start with `doc/` or `package/`,
  in which case they belong to this repository.
- Files of the fsnotify dependency (`README.md`, `backend_inotify.go`, `backend_kqueue.go`,
  `backend_windows.go`, `fsnotify.go`) are that library's files at tag `v1.10.1`,
  fetched read-only from `https://raw.githubusercontent.com/fsnotify/fsnotify/v1.10.1/`;
  a record that quotes one says so.
- Tools: `gh` 2.105.0, plain `git` (the repository's policy wrapper, reported as
  `monochromatic-git-policy-cli 0.6.1-alpha.44, shadowing git at /usr/bin/git`).

## Clone and release identity

The clone command was the one this run's instructions fix:

```shell
gh repo clone seungyeop-lee/directory-watcher \
  "/home/user/temp/agent/directory-watcher-source-for-garage-watcher-vet-2026-11-25" \
  -- --depth 1 --branch v3.2.1
```

- Exit status 0.
  Git warned `refs/tags/v3.2.1 5622bdfe6d4ddad01cf21f3d63e605266cdae0b3 is not a commit!`,
  which is what an annotated tag looks like to a one-branch clone,
  and checked out `7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- `git rev-parse HEAD`:
  `7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- `git rev-parse 'HEAD^{tree}'`:
  `c61dd635e132fe56d5d96d539fa0ffa6ee7c4ed0`.
- Tag object (`git tag -l --format='%(objecttype) %(objectname) -> %(*objectname)'`):
  `tag 5622bdfe6d4ddad01cf21f3d63e605266cdae0b3 -> 7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- Tag contents (`git cat-file -p v3.2.1`):
  tagger `github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>`,
  timestamp `1793891352 +0000` (2026-11-05T15:09:12Z), message `v3.2.1`.
- Tagged commit (`git log -1`):
  `7286f534dbeb3854607b3a48875c4cf0bbb5e672`,
  committed 2026-11-06T00:00:08+09:00 (2026-11-05T15:00:08Z),
  author `seungyeop-lee <xbeast7@gmail.com>`.
- The shallow clone holds one commit and 257 tracked files,
  and `git status --porcelain` was empty after the reads,
  so the tree read here is the tag's tree unchanged.
- `gh api repos/seungyeop-lee/directory-watcher/commits/master` returns the same
  `7286f534dbeb3854607b3a48875c4cf0bbb5e672`,
  so the tagged commit is also the default branch tip at the access date.
- The requested release is `v3.2.1`, published 2026-11-05T15:10:54Z by `github-actions[bot]`,
  and it is the newest release of the repository.
  Its `target_commitish` field is the branch name `master`, not a commit id;
  the annotated tag above is what actually pins the commit.

## Evidence records

### GW-01 License text and file path

- Version: 3.2.1, commit `7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- Claim: the released version is MIT licensed, with the license text in the repository.
- Gate: hard constraint "open source with inspectable source";
  skill gate "license and dependency surface".
- Status: pass.
- Evidence: `LICENSE:1-3` reads `MIT License`, blank line,
  `Copyright (c) 2020 Seungyeop Lee`;
  the file is the unmodified 21-line MIT text
  (SHA-256 `171b222a3a4c2da1660a5980a4c5c037f4661403341a38dd44e42a49bacb3fb7`).
  `gh api repos/seungyeop-lee/directory-watcher` reports `"license":"MIT"`, `"archived":false`.
  `README.md:184` reads
  `Versions before 3.0.0 were released under GPL-3.0.`,
  so the MIT grant covers 3.2.1 and nothing here depends on the older GPL terms.
- Access date: 2026-11-25.
- Outcome: license confirmed at the released commit, with no added clause,
  and the earlier GPL versions are outside the range in use.

### GW-02 Inspectable source for the released version

- Version: 3.2.1, commit `7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- Claim: the source of the released version is public and readable,
  and the clone is that source.
- Gate: hard constraint "open source with inspectable source";
  skill gate "production source".
- Status: pass.
- Evidence: the clone at the path in "Clone and release identity" is at the tag's commit,
  `main.go:5-7` contains the whole entry point
  (`func main() { cmd.Execute() }`),
  and the shipped program is `cmd/` plus `internal/` in Go
  (`go.mod:1` `module github.com/seungyeop-lee/directory-watcher/v3`).
  No generated source and no vendored third-party code sits in the tree
  (`git ls-files` lists 257 files: Go source, tests, documentation, a JSON schema and promo assets,
  with no vendor directory).
- Access date: 2026-11-25.
- Outcome: the code behind every claim below was read at the released commit.

### GW-03 Artifact provenance: the tag is built by the release workflow

- Version: 3.2.1.
- Claim: each release asset is built by the project's own workflow
  from the commit the tag points to,
  and not uploaded by hand from an unknown build.
- Gate: prebuilt overlay, "source-to-artifact mapping" and "release workflow";
  hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64" (artifact availability).
- Status: pass.
- Evidence, in the order the workflow runs:
  - `.github/workflows/ci.yml:3-9` triggers on `push` (`branches-ignore: master`) and `pull_request`;
    `:20-26` runs the `test` job on `ubuntu-latest`, `windows-latest` and `macos-latest`;
    `:124-130` adds the `release` job with `needs: test`
    and `if: github.event_name == 'push' && startsWith(github.ref, 'refs/heads/v')`,
    calling `uses: ./.github/workflows/goreleaser.yml`.
  - `.github/workflows/goreleaser.yml:92-120` creates the annotated tag in the release job itself:
    `git tag -a "v$VERSION" -m "v$VERSION" "$HEAD_SHA"` then `git push origin "$push_ref"`,
    with `HEAD_SHA: ${{ github.sha }}` at `:76`.
  - `.github/workflows/goreleaser.yml:132-140` runs
    `goreleaser/goreleaser-action@v7` with `version: 'v2.19.0'`,
    `args: release --clean --release-notes ...` and `GORELEASER_CURRENT_TAG: v${{ env.VERSION }}`.
  - `.goreleaser.yaml:9-17` builds `goos: linux, windows, darwin` with `CGO_ENABLED=0`
    and `ldflags: -s -w -X .../cmd.version=v{{.Version}}`;
    `:19-34` names the archives and writes `checksums.txt`.
  - Run evidence for this tag:
    `gh api 'repos/seungyeop-lee/directory-watcher/actions/runs'` lists run `42050842995`,
    name `ci`, `head_branch v3`, `head_sha 7286f534dbeb`, `event push`, `conclusion success`,
    started 2026-11-05T15:02:08Z and updated 2026-11-05T15:11:04Z.
    Its jobs were `test (ubuntu-latest)`, `test (windows-latest)`, `test (macos-latest)`,
    `release / version` and `release / publish`,
    all `success`, with `release / publish` completed 2026-11-05T15:11:03Z.
    The tag object above is dated 2026-11-05T15:09:12Z,
    and the release 15:10:54Z,
    both inside that run.
  - `docs/release.md:5-7` states the same flow in the project's own words:
    a push to `v<major>` runs the three-OS tests and the `release` job publishes that commit,
    making the annotated tag and the GitHub Release.
- Access date: 2026-11-25.
- Outcome: the artifacts come from a workflow build of the tagged commit,
  and the tag itself is created by that same run,
  which is the strongest mapping available without rebuilding the binary.
  A byte-level rebuild against the published asset was not attempted
  (this run may not build the candidate).

### GW-04 Registry packages point at the same tag and the same hashes

- Version: 3.2.1.
- Claim: what a user gets from Homebrew or Scoop on each platform
  is the artifact named by this release,
  pinned by version and SHA-256.
- Gate: prebuilt overlay, "checksum or signature";
  hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64" (installation path).
- Status: pass.
- Evidence:
  - Homebrew cask `seungyeop-lee/homebrew-tap` `Casks/directory-watcher.rb:3` `version "3.2.1"`;
    `:7-8` `sha256 "d90ccc89815f919e894ebf62537f498c3840ddd8bbf1be73d2fe37c830e52be5"`
    with URL `.../download/v#{version}/directory-watcher_Darwin_arm64.tar.gz`;
    `:11-12` Darwin x86_64 hash `226e9704...`;
    `:21-22` `sha256 "eb07d000d21069f528511a16d56ee529542f70daa1da527016cf74f8d9ba437a"`
    with URL `.../directory-watcher_Linux_x86_64.tar.gz`.
  - Scoop manifest `seungyeop-lee/scoop-bucket` `directory-watcher.json:2` `"version": "3.2.1"`;
    `:11-16` 64bit URL `.../download/v3.2.1/directory-watcher_Windows_x86_64.zip`
    with `"hash": "6fd9c034cdcc4c30d85deb1d39d5f0cd9452c2947ecd4e5bd32b3fc96b94ff26"`.
  - The release asset list
    (`gh api repos/seungyeop-lee/directory-watcher/releases/tags/v3.2.1`)
    holds nine files for version 3.2.1:
    `checksums.txt` and one archive per platform
    (`Darwin_arm64`, `Darwin_x86_64`, `Linux_arm64`, `Linux_i386`, `Linux_x86_64`,
    `Windows_arm64`, `Windows_i386`, `Windows_x86_64`).
  - The archives this design needs are named above: Linux x86_64 for this PC now,
    Darwin arm64 for the macOS client, and Windows x86_64 for the Windows client,
    the last of which the Scoop manifest also names.
- Access date: 2026-11-25.
- Outcome: each of the three required targets has a published artifact,
  and every package manifest that names an artifact pins the same SHA-256
  the release checksum file lists.

### GW-05 Integrity artifacts, and what is missing from them

- Version: 3.2.1.
- Claim: the release publishes SHA-256 checksums,
  and publishes no signature and no build attestation.
- Gate: prebuilt overlay, "checksum or signature";
  skill gate "checksum or signature", recorded as a limit.
- Status: pass with a stated limit.
- Evidence:
  - `checksums.txt` was fetched read-only from the release download URL
    (`curl -sSL --output checksums.txt .../releases/download/v3.2.1/checksums.txt`),
    HTTP 200, 824 bytes, SHA-256 `4e193f36901ee872417fd5cbff8e708ca10e3d27b808b2f45cf95ff84e82cd24`.
    It lists one line per archive:
    the Linux x86_64 line is `eb07d000d21069f528511a16d56ee529542f70daa1da527016cf74f8d9ba437a`
    followed by two spaces and `directory-watcher_Linux_x86_64.tar.gz`.
  - GitHub's own digest for each upload matches that file:
    the API reports `"digest":"sha256:eb07d000..."` for `directory-watcher_Linux_x86_64.tar.gz`,
    `sha256:d90ccc89...` for `Darwin_arm64`, `sha256:226e9704...` for `Darwin_x86_64`
    and `sha256:6fd9c034...` for `Windows_x86_64`.
  - The asset list holds no `.sig`, `.pem`, `.intoto.jsonl` or SBOM file,
    and `rg -n 'signs|sbom|attest|notariz' .goreleaser.yaml` returns nothing (exit 1),
    so no signing step is configured.
  - `.goreleaser.yaml:44-59` shows the macOS install hook removes the quarantine attribute
    (`/usr/bin/xattr -dr com.apple.quarantine`) because the binary is not signed or notarized,
    as its comment says.
- Access date: 2026-11-25.
- Outcome: integrity rests on SHA-256 values from the same release and on GitHub's storage digests.
  There is no signature, no attestation and no notarization,
  so a checksum published in the same release cannot show that the release itself was not replaced.
  Recorded as a limit of the provenance, not as a failure of the mapping in GW-03.

### GW-06 Dependency surface and compiled form

- Version: 3.2.1.
- Claim: the program depends on a short list of Go modules,
  ships no lifecycle script,
  and needs no interpreter, native extension or C toolchain.
- Gate: skill gate "license and dependency surface";
  prebuilt overlay.
- Status: pass.
- Evidence: `go.mod:5-18` requires
  `github.com/bmatcuk/doublestar/v4 v4.10.4`,
  `github.com/fsnotify/fsnotify v1.10.1`,
  `github.com/moby/term v0.5.2`,
  `github.com/spf13/cobra v1.10.2`,
  `github.com/spf13/pflag v1.0.10`,
  `golang.org/x/sys v0.49.0`,
  `gopkg.in/yaml.v3 v3.0.1`,
  plus `github.com/Azure/go-ansiterm` and `github.com/inconshreveable/mousetrap` as indirect.
  `go.sum` pins each module by hash and there is no vendor directory.
  No package in `cmd/`, `internal/` or `main.go` runs at install time,
  because a Go module has no install hooks;
  `go.mod:3` fixes `go 1.27`.
  `.goreleaser.yaml:10-11` sets `CGO_ENABLED=0`, so the published binaries are static Go builds.
  The network client, HTTP server and socket code that exist in the tree
  belong to `promo/scenes/` demo modules and the `e2e/` test harness,
  not to the shipped program (see GW-22).
- Access date: 2026-11-25.
- Outcome: the dependency surface a user must accept is these nine modules,
  each pinned by hash, with fsnotify as the only one that touches the operating system.

### GW-07 Watching backend on Linux

- Version: 3.2.1.
- Claim: on Linux the watcher uses inotify, through fsnotify.
- Gate: hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64";
  hard constraint "watches every folder under the root recursively".
- Status: pass.
- Evidence: `internal/app/service/runner_watch_target.go:26`
  `var inotifyOS = runtime.GOOS == "linux"`;
  `:60` `watcher, err := fsnotify.NewWatcher()`;
  `internal/app/service/runner_watch_target_tree.go:210`
  `if err := addWatch(t.watcher.Add, p); err != nil {`.
  The library is `github.com/fsnotify/fsnotify v1.10.1` (`go.mod:7`),
  whose README at that tag lists `inotify` for Linux (`README.md:13`),
  and whose Linux backend calls `unix.InotifyAddWatch` (`backend_inotify.go:275`).
  Linux-specific event handling in the project confirms the backend:
  `internal/app/service/runner_watch_target.go:279`
  `if isDir && inotifyOS && ev.Op.Has(fsnotify.Rename) {`.
- Access date: 2026-11-25.
- Outcome: Linux support is inotify and nothing else,
  so one watch per folder is the resource model (see GW-10 and GW-20).

### GW-08 Watching backend on macOS

- Version: 3.2.1.
- Claim: on macOS the watcher uses kqueue, through fsnotify.
- Gate: hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64".
- Status: pass.
- Evidence: `internal/app/service/runner_watch_target.go:23`
  `var kqueueOS = slices.Contains([]string{"darwin", "dragonfly", "freebsd", "netbsd", "openbsd"}, runtime.GOOS)`;
  `:224` comment on the kqueue implementation (translation:
  fsnotify's kqueue implementation follows folder links inside the target by itself,
  and reports events for paths outside the target);
  `internal/app/service/runner_watch_target_tree.go:235-244` retries `Add` on `EBADF` on kqueue only,
  which is a kqueue-specific defect it works around.
  fsnotify at v1.10.1 lists `kqueue` for `BSD, macOS` (`README.md:14`)
  and its macOS entry point is `func (w *kqueue) Add(name string) error` (`backend_kqueue.go:275`).
  The macOS artifact is published as `directory-watcher_Darwin_arm64.tar.gz` (GW-04).
- Access date: 2026-11-25.
- Outcome: macOS support is kqueue,
  which costs one open file descriptor per watched folder and one per file inside it (GW-21).

### GW-09 Watching backend on Windows

- Version: 3.2.1.
- Claim: on Windows the watcher uses the Windows change notifications through fsnotify,
  with Windows-specific handling of its quirks.
- Gate: hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64".
- Status: pass.
- Evidence: `internal/app/service/runner_watch_target_tree.go:205-211`
  comments that fsnotify's Windows implementation keeps a moved folder's watch on the old path,
  then `if runtime.GOOS == "windows" { _ = t.watcher.Remove(p) }`;
  `:581-584` `pumpEvents` exists because fsnotify's Windows implementation handles
  `Add` and `Remove` on the goroutine that sends events;
  `internal/app/service/runner_watch_target.go:289` comment
  (translation: Windows sends the folder's `Write` when an entry inside it changes).
  `cmd/console_windows.go:1` is a `//go:build windows` file that sets the console code page to UTF-8,
  and `internal/process/process_windows.go` is the Windows process-tree implementation.
  fsnotify at v1.10.1 lists `ReadDirectoryChangesW` for Windows (`README.md:15`).
  The Windows artifact is published as `directory-watcher_Windows_x86_64.zip` (GW-04),
  and CI runs the test suite on `windows-latest` (GW-03, `ci.yml:25`).
- Access date: 2026-11-25.
- Outcome: Windows support is present in source, in the release assets, and in CI.
  Its runtime behaviour was not exercised by this run.

### GW-10 Recursion at startup, including every existing folder

- Version: 3.2.1.
- Claim: at startup the watcher walks the whole target tree
  and adds one watch per folder, skipping excluded folders.
- Gate: hard constraint "watches every folder under the root recursively,
  including folders created after it starts".
- Status: pass.
- Evidence: `internal/app/service/runner_watch_target_tree.go:110-135`
  `add()` resolves a folder link, then `return t.addDirTree(t.root)` when `subDir` is on;
  `:177-226` `addDirTree` uses `filepath.WalkDir(root, ...)`,
  records each entry (`:195` `t.addEntry(p, d.IsDir())`),
  returns `filepath.SkipDir` for an excluded folder (`:200-202`),
  and calls `Add` for every folder (`:210`).
  `subDir` defaults to true: `internal/config/converter/sub_dir.go:3`
  `const defaultSubDirValue bool = true`, documented at `docs/guide/configuration.md:58`.
  The plan record measured the target tree at 1454 folders and 10332 files
  (`doc/planning/garage-bisync-run-on-file-change.md`, section "Facts measured").
- Access date: 2026-11-25.
- Outcome: a start on `~/Plain` adds one watch per folder, about 1454 of them,
  minus any folder whose name or pattern is excluded (GW-19).
  Not measured by this run: the watcher was not started.

### GW-11 Watches for folders created after start

- Version: 3.2.1.
- Claim: when a folder appears under the target while the watcher runs,
  the watcher adds a watch for it and for any folders already inside it.
- Gate: hard constraint "watches every folder under the root recursively,
  including folders created after it starts".
- Status: pass.
- Evidence: `internal/app/service/runner_watch_target.go:252-261`,
  in the `fsnotify.Create` branch of `handleEvent` (`:235`):
  `isDir = err == nil && info.IsDir()`, then
  `if err := r.tree.added(ev.Name, isDir); err != nil { return err }`.
  `internal/app/service/runner_watch_target_tree.go:159-172` `added()`:
  `t.addEntry(path, isDir)`, `if !isDir || !t.subDir || t.isExcluded(path) { return nil }`,
  `err := t.addDirTree(path)`.
  `addDirTree` (`:177-226`) applies the same walk and the same exclusion rules
  as at startup, so a folder created later is watched exactly like one that was already there
  (its comment says so at `:173-176`, translation:
  startup and a folder that appears or moves in while watching use this one function,
  so the same rules apply whenever the folder appeared).
  A rename or move into the target arrives as a `Create` of the new path
  (`docs/guide/events.md:7-16`), so it takes the same path.
- Access date: 2026-11-25.
- Outcome: the recursion requirement holds for folders created after start,
  by the same code path as startup.

### GW-12 Content that a new folder already holds

- Version: 3.2.1.
- Claim: files that come into existence before their folder's watch is added produce no file events;
  the folder's own event is what reaches the hook.
- Gate: hard constraint "watches every folder under the root recursively";
  recorded as an event-coverage limit, not a failure of the watch setup.
- Status: pass with a stated limit.
- Evidence: `addDirTree` records existing entries with `t.addEntry` (`runner_watch_target_tree.go:195`)
  and adds watches, but emits no event for them;
  the only event produced for such a folder is the `Create` of the folder itself.
  `docs/guide/events.md:45-46` states the resulting contract:
  `A folder C means that the folder appeared together with everything in it.`
  and `The watcher sends no file events for the files a folder brings with it, even when the folder is moved in.`
  Folder events are delivered by default: `internal/config/converter/watch_event.go:35`
  sets the folder-event fallback to `CD`, and `docs/guide/configuration.md:62` documents `dirEvent`
  defaulting to `C` and `D`.
- Access date: 2026-11-25.
- Outcome: a `mkdir -p ~/Plain/x` followed by `cp note.txt ~/Plain/x/` reaches the hook as one folder `C`,
  not as file events, and the folder `C` is enough to start a sync.
  A target configured with `watch.dirEvent: none` would see nothing for such a folder,
  so that setting would break the requirement and is not what this candidate needs.
  This is a source-level reading, not a measurement.

### GW-13 How a hook command is executed

- Version: 3.2.1.
- Claim: the configured command is a shell string, run by `sh -c` on Linux and macOS
  and by `cmd.exe` or Windows PowerShell on Windows,
  with the working directory set to the watch target,
  standard input connected to the terminal,
  and the watcher's environment inherited.
- Gate: hard constraint "runs an arbitrary command,
  here `systemctl --user start rclone-bisync-garage.service`".
- Status: pass.
- Evidence: `internal/app/domain/command/shell.go:27-34`:
  `name: "sh"`, `return exec.CommandContext(ctx, "sh", "-c", script)`;
  `:36-44` `cmd.exe` and `:46-56` `powershell`;
  `:90-97` `DefaultShell()` is `cmd` on Windows and `sh` elsewhere
  (`internal/app/domain/command/shell_unix.go:10` `const defaultShellName = "sh"`,
  `shell_windows.go:11` `const defaultShellName = "cmd"`).
  `internal/app/domain/command/single_cmd.go:31-40`:
  `cmd := c.Shell.command(ctx, script)`,
  `cmd.Env = append(os.Environ(), c.eventEnvironment(event)...)`,
  `cmd.Dir = runDir.String()`,
  `cmd.Stdin, _, _ = term.StdStreams()`,
  `process.SetupForOs(cmd)`.
  `runDir` is the watch target when a command has no `dir`
  (`internal/app/service/hook_runner.go:171-172` passes `r.target`).
  `docs/guide/configuration.md:105-115` states the same three shells and the Windows default.
- Access date: 2026-11-25.
- Outcome: the command for this design is written as one YAML string
  (`systemctl --user start rclone-bisync-garage.service`),
  runs through `sh -c` with working directory `~/Plain`,
  and nothing about it needs argv splitting.
  The shell string means the YAML author controls shell syntax,
  which is the intended behaviour of a hook runner and not an injection path from file names (GW-14).

### GW-14 How changed paths reach the command, and how they are quoted

- Version: 3.2.1.
- Claim: changed paths are available as template variables that the watcher replaces
  with references to environment variables,
  so a file name never enters the command string and never becomes shell syntax;
  paths are not passed on standard input,
  and a command that names no variable receives no path at all.
- Gate: hard constraint "runs an arbitrary command";
  security boundary of the design (file names under `~/Plain` contain Japanese and Chinese characters,
  spaces, parentheses, brackets and quotes).
- Status: pass.
- Evidence: `internal/app/domain/command/template.go:14-33` defines the eight variables
  and their environment names, for example
  `{"Path", "DW_PATH", func(e *domain.Event) string { return e.Path.String() }}`
  and `{"IsDir", "DW_IS_DIR", ...}`.
  `internal/app/domain/command/single_cmd.go:47-50` comment (translation):
  the executable command replaces each variable with a reference to the environment variable
  that holds the value, so a file name is never interpreted as a shell command,
  while the printed line fills the value in.
  `:61-74` builds two strings: the script from `c.templateRefs()` and the printable `filled` from the values.
  `:94-103` puts each value in `DW_*` and, for `cmd` and `powershell`, an `_ARG` variant.
  `internal/app/domain/command/shell.go:33` builds the `sh` reference from the environment variable name:
  `templateRef` returns the name in double quotes after a dollar sign,
  which yields `"$DW_PATH"` at run time;
  `:40` uses `!VAR!` for `cmd` and `:48` `${env:VAR}` for `powershell`.
  `docs/guide/configuration.md:217-227` states the consequence in plain words:
  a file name never enters the command string, so a name that contains a semicolon,
  an ampersand or a command substitution never runs as a command;
  that a variable outside quotes is passed as one argument even with spaces,
  and that inside double quotes in `sh` and `cmd` the reference ends up outside quotes,
  so the value can split at spaces.
  `internal/app/domain/command/single_cmd.go:36` connects standard input to the terminal,
  so nothing is fed through standard input.
- Access date: 2026-11-25.
- Outcome: the quoting is sound for names with spaces, brackets, quotes and non-ASCII characters,
  provided a variable is written outside quotes as the guide says.
  For this design the command names no variable, so no path is passed at all,
  and the run is triggered by the event without path data.

### GW-15 Busy behaviour by default, and the settings that change it

- Version: 3.2.1.
- Claim: by default a change that arrives while a command runs is **queued**,
  one run at a time, and nothing runs concurrently and nothing is killed;
  concurrency, interruption and the queue length are the three settings that change this.
- Gate: hard constraints "queues one more run of its command natively when events arrive while the command runs",
  "never interrupts a running command; each run finishes",
  "after a change that lands while a sync runs, one more sync starts after that run ends".
- Status: pass.
- Evidence:
  - Defaults: `internal/config/converter/run.go:10` `const defaultConcurrency = 1`;
    `internal/config/converter/interrupt.go:29-31` returns `domain.InterruptNone` when unset;
    `internal/config/converter/queue.go:10` `const defaultQueueSize = 2048`.
  - Waiting instead of running or killing: `internal/app/service/hook_runner.go:116-119`
    `func (r *hookRunner) full() bool { return len(r.runs) >= r.policy.Concurrency || `
    `len(r.runs) > 0 && r.runs[0].start }`
    (a running `onStartWatch` alone fills every slot);
    `:95-114` `admit` waits on `r.ended.Wait()` until a slot is free, then calls `r.start`;
    no branch of it cancels a run under `InterruptNone`.
  - The queue is FIFO and bounded by `queue.size`:
    `internal/app/service/runner_watch_target_queue.go:92-107` `first` blocks until a run exists or watching stops;
    `:306-321` `dispatcher.run` takes the first run, calls the hook, then `d.queue.pop()`.
  - The only kills are opt-in: `hook_runner.go:122-138` `toCancel` returns runs only for
    `domain.InterruptSamePath` or `domain.InterruptOldest`;
    `:161-169` `cancel` cancels every run and is reached through `stopAccepting`
    (`internal/app/service/runner_watch_target.go:353-356`),
    which `Stop` and a fatal error call.
  - Documentation of the same behaviour: `docs/guide/running.md:137-152`
    (concurrency default one run at a time, `interrupt` default `none`,
    `oldest` cancels the run that started first, `samePath` cancels the run of the same path)
    and `:48-52` (collecting goes on while a hook runs, and each finished wait adds its runs to the queue).
  - `interrupt: oldest` is what `README.md:41` and `USAGE.md:26` describe
    (`Each save stops the running server and the processes it started in the hook's process group`),
    which is a documented mode and not the default.
- Access date: 2026-11-25.
- Outcome, one label per behaviour with the setting that selects it:
  - queue: default, `run.concurrency: 1` with `run.interrupt: none`;
  - run concurrently: `run.concurrency` above 1;
  - kill: `run.interrupt: samePath` or `run.interrupt: oldest`;
  - drop: no setting drops a single run;
    when the queue is full the whole watcher stops (GW-16).
  With the defaults the mid-run requirement holds natively:
  events collected during a run become runs that start after it ends, in order.

### GW-16 A full queue, and what "full" costs

- Version: 3.2.1.
- Claim: a queue overflow is an explicit fatal error,
  not a silent drop of one run,
  and it runs the global `onError` hook once before the watcher exits with code 1.
- Gate: hard constraint "queues one more run of its command natively";
  capability "what it does when a watch limit runs out" (queue side).
- Status: pass.
- Evidence: `internal/app/service/runner_watch_target_queue.go:75-88`:
  `if len(q.runs) == q.capacity {`, then
  `return fmt.Errorf("dispatch queue full (capacity %d): %w", q.capacity, fsnotify.ErrEventOverflow)`;
  `:290-303` `dispatcher.enqueue` returns that error to the watch loop;
  `internal/app/service/runner.go:42-56` logs it, sends it on the error channel,
  and calls `r.global.CallOnError(err)` for anything that wraps `fsnotify.ErrEventOverflow`;
  `internal/app/service/runner_global.go:62-72` fills `DW_ERROR_*` with
  `DW_ERROR_KIND=event_loss` and `DW_ERROR_POSSIBLE_LOSS=true`;
  `cmd/cmd.go:46-51` turns the returned error into `os.Exit(1)`.
  `docs/guide/running.md:93-95` and `:193-196` state the same:
  a run that does not fit stops processing with `dispatch queue full (capacity <size>)`,
  the watcher does not wait for room in the queue, and the waiting runs are discarded, not saved.
- Access date: 2026-11-25.
- Outcome: the queue is a bounded native queue of 2048 runs by default,
  and its overflow is fail-stop with one notification.
  For this design a sync of a few seconds against a 100 ms quiet period
  would need a sustained arrival rate far above what `~/Plain` shows (GW-18, plan record),
  but the number is fixed and the failure mode is the whole watcher stopping.

### GW-17 Whether queueing can ever stop a running command

- Version: 3.2.1.
- Claim: queueing itself never stops a running command;
  a running command is stopped only by `run.interrupt`, by a shutdown signal, or by a fatal watcher error,
  and on Linux that stop is SIGTERM to the hook's process group followed by SIGKILL after 20 seconds.
- Gate: hard constraints "never interrupts a running command; each run finishes",
  "after a change that lands while a sync runs, one more sync starts after that run ends".
- Status: pass with the three exceptions named.
- Evidence: `internal/app/service/hook_runner.go:95-114` waits and never cancels under `InterruptNone`;
  `:122-138` is the only place that selects runs to cancel;
  `:161-169` `cancel()` cancels every run and is called from `runFinishHook` (`:79-91`)
  and from `stopAccepting` (`internal/app/service/runner_watch_target.go:353-356`),
  which `Stop` runs on SIGINT or SIGTERM (`cmd/cmd.go:121-134`)
  and the first fatal error runs (`internal/app/service/runner.go:41-56`);
  `internal/process/process_unix.go:12-23` sends `SIGTERM` to the process group;
  `internal/process/context.go:64` `const shutdownGrace = 20 * time.Second`
  and `process_unix.go:56-67` sends `SIGKILL` after that;
  `process_unix.go:69-74` sets `Setpgid: true`, so the group is the hook and its children.
  `docs/guide/running.md:205-214` documents the same for shutdown and replacement.
- Access date: 2026-11-25.
- Outcome: with the defaults, a change mid-run cannot interrupt the run.
  A SIGTERM to the watcher does end the hook's process group,
  which for `systemctl --user start` ends the client only;
  the design record measured that killing the client leaves the sync unit running
  (`doc/planning/garage-bisync-run-on-file-change.md`, section "Owner answers, round 4").
  A shutdown also discards the runs still waiting in the queue
  (`docs/guide/running.md:205`), so a stop is the one way pending work is lost on purpose.

### GW-18 Quiet period and batching defaults

- Version: 3.2.1.
- Claim: events are merged with a 100 ms quiet period capped by a 1000 ms maximum,
  and folder events have their own 100 ms and 1000 ms pair.
- Gate: capability "merges a burst of events into one run (a quiet period)";
  design work on trigger rate.
- Status: pass, values recorded.
- Evidence: `internal/config/converter/batch_wait.go:11`
  `const defaultWaitMillisecond, defaultMaxWaitMillisecond domain.Millisecond = 100, 1000`,
  applied by `:17-29`: `maxWait = max(defaultMaxWaitMillisecond, wait)`;
  used for `hookBatch` (`internal/config/converter/hook_batch.go:20-29`)
  and `dirBatch` (`internal/config/converter/dir_batch.go:19-25`).
  `hookBatch.deliver` defaults to `last`
  (`internal/config/converter/deliver.go:28-31`),
  which turns the collected events of one wait into a run for the last changed path only.
  `docs/guide/running.md:50-52` and `docs/guide/events.md:59-63` document the pair,
  including that `0` turns waiting off, and `docs/guide/configuration.md:63-69` lists both groups.
- Access date: 2026-11-25.
- Outcome: a burst of saves within 100 ms runs the command once,
  and with `deliver: last` a burst that touches several paths still runs it once per wait.
  Several waits during one long run each add a run, which is the native queueing of GW-15.

### GW-19 Path exclusion

- Version: 3.2.1.
- Claim: paths can be excluded with `**`-style globs relative to the target,
  `!` re-includes a path, a matched folder is not watched at all,
  and names starting with `.` or `_` are excluded without any configuration.
- Gate: capability "excludes paths by pattern";
  hard constraint "watches every folder under the root recursively" (what is not watched).
- Status: pass, with the default exclusion named because it changes what is watched.
- Evidence: `internal/app/domain/exclude.go:27-49`
  (`Excludes` checks every parent segment, `lastMatchExcludes` applies the last matching pattern
  and treats a `!` prefix as re-include);
  `internal/app/service/runner_watch_target_tree.go:295-307` `isExcluded` returns true for
  `domain.Path(name).IsDefaultExcludeFile()` and for a pattern match,
  and is used before `Add` at `:200-202` (`filepath.SkipDir`).
  `internal/app/domain/path.go:46-49`:
  `return strings.HasPrefix(name, ".") || strings.HasPrefix(name, "_")`.
  `docs/guide/events.md:77-82` documents default exclusion and that
  the folder set as `path` is watched regardless of its name,
  and `:84-110` documents the pattern syntax and the `!` limits.
- Access date: 2026-11-25.
- Outcome: exclusion is per target and predictable.
  For `~/Plain`, any hidden or underscore-named folder is not watched and its changes trigger nothing,
  which is the documented macOS `._` behaviour and also hides `.git` and similar names.
  A target whose hook writes inside the watched tree can exclude its own output,
  as section "Stopping a hook from rerunning itself" in `docs/guide/events.md:113-120` describes.

### GW-20 What happens when the inotify watch limit runs out

- Version: 3.2.1.
- Claim: an `Add` that fails for lack of watches is a fatal error:
  the watcher logs `watch target "<path>": <cause>` and exits with code 1,
  with no polling fallback and no `onError` notification.
- Gate: capability "what it does when the inotify watch limit runs out";
  vet report ledger entry (which records a notification command for this case).
- Status: pass on the source reading; the outcome below refutes the ledger's wording.
- Evidence: `internal/app/service/runner_watch_target_tree.go:235-244` `addWatch` returns the `Add` error,
  retrying only on `EBADF` and only on kqueue;
  `:210-218` returns it from `addDirTree` unless the folder vanished,
  which is the startup path too, and `:120-122` returns it at startup for a target with `subDir: false`;
  `:159-171` `added` passes it up during watching;
  `internal/app/service/runner_watch_target.go:258-259` returns it from `handleEvent`,
  which `Run` turns into `&FatalError{Path: r.path, Cause: err}`
  (`:188-193` `exitError`, called from the event path at `:149`);
  `internal/app/service/runner.go:42-56` logs `err.Error()` and calls `CallOnError`
  only when the error wraps `fsnotify.ErrEventOverflow`,
  which an `ENOSPC` or `EMFILE` from `Add` does not;
  `internal/app/service/runner_global.go:68` hard-codes `DW_ERROR_KIND` as `event_loss`;
  `docs/guide/running.md:201-204` documents the class:
  a target that fails to start watching is also a fatal error,
  which happens when its `path` does not exist, its folder link cannot be resolved,
  or a folder cannot be added to the watch list.
  The library states the limit itself: fsnotify v1.10.1 `README.md:152-156`
  `Reaching the limit will result in a "no space left on device" or "too many open files" error.`
  `rg -n 'ENOSPC|EMFILE|ENFILE|Rlimit|RLIMIT'` over the non-test tree returns nothing,
  and `rg -ni 'poll'` over `cmd/`, `internal/` and `main.go` returns only the Windows
  Job Object cleanup interval, so there is no polling fallback.
  Scale for this host: the plan record measured 1454 folders and
  `fs.inotify.max_user_watches = 524288`, `fs.inotify.max_user_instances = 1024`;
  one target creates one fsnotify watcher (`runner_watch_target.go:60`)
  and temporarily a second during a rebuild (`runner_watch_target_tree.go:515-516`).
- Access date: 2026-11-25.
- Outcome: watch-limit exhaustion is fail-stop with a log line and exit code 1,
  and no notification hook runs, because only event overflow and a full dispatch queue are
  treated as `event_loss`.
  The arithmetic of 1454 watches against 524288 leaves a wide margin on this PC,
  and the same margin covers the single instance against 1024.
  The vet report ledger entry, which says a watch-limit condition "runs a notification command",
  is refuted for the watch-limit case and confirmed only for the event-loss case (GW-16).

### GW-21 The macOS descriptor cost of a watched tree

- Version: 3.2.1.
- Claim: on macOS the kqueue backend opens one file descriptor for the watched folder
  and one for every file inside it, the watcher watches every folder of the tree,
  and it never raises its descriptor limit,
  so this tree of 1454 folders and 10332 files needs about 11786 descriptors.
- Gate: hard constraint "runs on Linux x86-64, macOS arm64 and Windows x64";
  runtime limit of the later macOS client.
- Status: open, because it was not measured and needs a run on the Mac.
- Evidence: fsnotify v1.10.1 `README.md:207-215`:
  `kqueue requires opening a file descriptor for every file that's being watched;`
  `so if you're watching a directory with five files then that's six file descriptors.`
  `You will run in to your system's "max open files" limit faster on these platforms.`
  Its macOS backend opens the added path with `unix.Open` (`backend_kqueue.go:397-403`)
  and, for a directory, watches every entry inside it with `watchDirectoryFiles` (`:582-616`),
  each entry going through `internalWatch` and its own descriptor.
  `internal/app/service/runner_watch_target_tree.go:210` adds one watch per folder of the tree,
  the counts 1454 folders and 10332 files come from the plan record's "Facts measured",
  and `rg -n 'Rlimit|RLIMIT'` over the shipped tree finds no limit handling.
- Access date: 2026-11-25.
- Outcome: on macOS this tree would need about 11786 open descriptors,
  the sum of the folder and file counts.
  Whether that fits depends on the job's soft `RLIMIT_NOFILE` and on `kern.maxfilesperproc`,
  and this run read neither, on this host or in Apple's documentation;
  the Mac must be measured, including whether its job can be given a limit that high.
  This is an arithmetic inference from source and the library's documentation, not a measurement:
  if the limit cannot be raised to cover both counts, this candidate would fail on macOS for this tree,
  which would be a runtime limit rather than a missing platform.
  Nothing in this record says the tool fails on macOS in general.

### GW-22 Network, listeners, telemetry, privileges, and file writes

- Version: 3.2.1.
- Claim: the shipped program opens no socket, runs no server, checks for no update,
  sends no telemetry, requests no privilege, and writes no file.
- Gate: security boundary of the design (runs as the desktop user, reads names and events only);
  skill gate "security boundary required by the use".
- Status: pass.
- Evidence, over `cmd/`, `internal/` and `main.go`, excluding `_test.go`:
  - `rg -n '"(net|net/http|net/url|net/http/pprof)"'`: no hit.
    The only `net` imports in the tree are `promo/scenes/1-restart/server/main.go:6-7`
    (a demo server the promos run), `promo/scenes/4-forward/receiver/main.go:8`
    and `e2e/windows/utm.go:10` (the test harness driving a UTM VM).
  - `rg -ni 'telemetry|analytics|checkforupdate|update check|listen\(|socket|dial\(|http\.Get|http\.Post'`:
    the only hits are `promo/scenes/1-restart/server/main.go:16` `net.Listen("tcp", ":8090")`
    and a Windows process-enumeration struct in `e2e/windows/guest/main.go:220`.
  - `rg -n 'os\.(OpenFile|Create|WriteFile|Mkdir|MkdirAll|Remove|Rename|Chmod)'` over the shipped tree:
    no hit, so the program creates no file, log, cache or state.
  - `rg -ni 'sudo|setuid|setgid|chroot|syscall\.Set'`: no hit.
  - `docs/guide/installation.md:42-60` shows the published image only mounts a config file and the watched folder.
  - `SECURITY.md:5-15` states that only the latest release receives security fixes
    and that reports go through GitHub private vulnerability reporting.
- Access date: 2026-11-25.
- Outcome: the only external effect is the command the configuration names,
  which for this design is `systemctl --user start rclone-bisync-garage.service`.
  The trust boundary in the vet report's fingerprint ("reads names and events under ~/Plain;
  runs one fixed command; no credentials pass through it") matches the source.

### GW-23 What the hook process inherits

- Version: 3.2.1.
- Claim: the hook process inherits the watcher's whole environment and runs in the watched folder,
  in its own process group, and is stopped as a group.
- Gate: security boundary and the trigger design
  (a `systemctl --user` call needs the session environment, and cancelling the client must not kill the sync).
- Status: pass, recorded because it decides how the trigger unit must be started.
- Evidence: `internal/app/domain/command/single_cmd.go:33` `cmd.Env = append(os.Environ(), ...)`;
  `:35` `cmd.Dir = runDir.String()`, where `runDir` is the target path
  (`internal/app/service/hook_runner.go:171-172`);
  `internal/process/process_unix.go:69-74` `Setpgid: true`;
  `:12-23` SIGTERM to the group and `internal/process/context.go:64` a 20-second grace before SIGKILL.
- Access date: 2026-11-25.
- Outcome: if the watcher runs as a systemd user unit,
  the hook inherits that unit's environment, which includes `XDG_RUNTIME_DIR` and the session bus address,
  so `systemctl --user start` works without extra configuration.
  A cancellation stops the client process group only;
  the design record measured that killing the client leaves the sync unit running.

### GW-24 Tests and CI behind these claims

- Version: 3.2.1.
- Claim: the behaviour above is exercised by unit tests and by end-to-end scenarios
  on all three operating systems, with no fuzzing and no published coverage.
- Gate: skill gate "tests and CI", supporting the code-behaviour records.
- Status: pass, with the absences named.
- Evidence: 67 `*_test.go` files and 288 `func Test` functions in the tree
  (`git ls-files '*_test.go' | wc -l`, and a count of `^func Test` lines in those files);
  scenario tests under `e2e/scenarios/` with per-scenario documents in `e2e/docs/scenarios/`;
  `.github/workflows/ci.yml:20-39` runs `go build ./...`, `go vet ./...` and
  `go test -count=1 -race ./...` on `ubuntu-latest`, `windows-latest` and `macos-latest`,
  plus a Linux `make smoke-example` and Windows-only lifecycle and diagnostic checks (`:43-113`).
  `docs/release.md:31-33` requires `make e2e-linux` and `make e2e-windows-all` before a release,
  the latter in a UTM Windows VM.
  `rg -n 'func Fuzz'` returns nothing, and no coverage profile or coverage gate appears
  in `Makefile` or the workflows.
- Access date: 2026-11-25.
- Outcome: the code paths this record cites have tests and a three-OS CI,
  and the absences (no fuzzing, no coverage number, no Windows run in this audit) are stated.

## Maintenance audit

Method: the checks the skill's "Maintenance audit" section lists,
run read-only against GitHub on 2026-11-25.
Tracker sample rule: with at most 20 matching items, inspect all.

- Issues and pull requests in the last 12 months:
  none, and none in the repository's whole history.
  `gh api -X GET 'repos/seungyeop-lee/directory-watcher/issues' -f state=all -f since=2025-11-25T00:00:00Z`
  returned an empty list, and the same call without `since` also returned `0`,
  both for `issues` and for `pulls`.
  Positive control: the same `since` query against `repos/cli/cli/issues` returned 5 rows,
  so the query shape does work.
  Repository metadata also reports `open_issues_count: 0`.
- Why there are no pull requests:
  `CONTRIBUTING.md:3` reads
  `Bug reports and feature ideas are welcome as GitHub issues.`
  then `Pull requests are not accepted: the repository settings allow only collaborators to open them.`
  and `README.md:178` repeats it.
  Zero pull requests is therefore a policy,
  and maintainer-authored work is the only path into the code.
- Maintainer concentration:
  `gh api 'repos/seungyeop-lee/directory-watcher/contributors?anon=1'` returns one entry,
  `seungyeop-lee`, 359 contributions,
  which matches the repository's whole commit count
  (`Link: ... page=359; rel="last"` for `commits?per_page=1`).
  Every recent commit read (`commits?per_page=8`) is authored by `seungyeop-lee`,
  with Korean subjects such as `fix: Unix에서 취소된 hook의 자손을 신호부터 20초 안에 정리함`.
  One maintainer holds all of the code, the releases and the tags.
- Release cadence:
  30 published releases and 33 tags in total.
  v2.0.0 (2023-01-07) through v2.9.4 (2024-11-24) are 26 releases over about 22 months.
  Then nothing until v3.0.0 on 2026-11-04,
  followed by v3.1.0 (2026-11-05T05:40Z), v3.2.0 (2026-11-05T06:55Z)
  and v3.2.1 (2026-11-05T15:10Z).
  So the current line is four releases in two days after a two-year quiet period,
  and the release this audit reads is the newest of them.
- Whether maintenance reaches a release:
  yes for this release.
  The tagged commit is the default branch tip (`commits/master` = `7286f534...`),
  and v3.2.1 is the fourth release of the current line, so no fix sits unreleased behind the tag.
- Changelog and release notes:
  `CHANGELOG.md:1-18` is Keep a Changelog and SemVer,
  with a `## [3.2.1] - 2026-11-05` section.
  Release notes come from the changelog through `scripts/releasenotes` (`docs/release.md:41`),
  and the changelog holds a section for each 3.x release read here (3.0.0, 3.1.0, 3.2.0, 3.2.1).
- Stale unanswered bugs, compatibility failures, security reports, abandonment comments:
  none to inspect, because the tracker holds no item at all.
  Nothing is unanswered and nothing is stale,
  and equally there is no tracker evidence of a responsive maintainer beyond the release flow itself.
  `SECURITY.md:5` says only the latest release receives security fixes.
- Maintainer-only practices that bear on an adopter:
  the code comments, commit messages, `AGENTS.md`, `docs/release.md` and project skills are in Korean
  (`CONTRIBUTING.md:15`),
  and the project states an agent-assisted workflow (`.claude/skills/versionup/SKILL.md`, `AGENTS.md`).
  Reading the released source needs Korean for the comments that explain the platform workarounds.
- Reading of the signals, in the skill's terms:
  this is "active releases with weak public issue support" and "low-signal tracker activity
  in a project evaluated through source, release, ownership and test evidence",
  not an abandoned project:
  the current line was published days before this audit,
  the release job builds and tests on all three targets,
  and the source shows maintained platform handling.
  The concentration risk is real: one person owns everything,
  and a stop of that person stops the project and the security fixes.

## Where this record disagrees with the vet report's ledger entry

- The ledger entry says a "watch-limit or event-loss" condition stops the watcher
  "and run[s] a notification command".
  The notification path is guarded by `errors.Is(err, fsnotify.ErrEventOverflow)`
  (`internal/app/service/runner.go:53-55`),
  and a failed `Add` from the watch limit is not that error,
  so the watcher stops with a log line and exit code 1 and runs no notification (GW-20).
  The ledger's statement holds for event loss only (GW-16).
- The ledger entry lists `run.concurrency` and `run.interrupt` as choosing overlap or interruption
  but gives no defaults.
  The defaults are `1` and `none` (GW-15),
  and they are what makes the "never interrupts" and "queues natively" constraints hold without configuration.
- The ledger entry's gate 2 record quotes the README line about `interrupt: oldest`
  stopping a running server, which is one opt-in mode,
  rather than the mechanism that actually runs the command.
  The deciding evidence is `configuration.md:105-115` with `shell.go:27-34` and `single_cmd.go:31-40` (GW-13).
- The ledger entry's gate 4 record is one sentence about recursive targets.
  The deciding evidence is `addDirTree` and `added` (GW-10, GW-11),
  together with the limit that a new folder's own contents produce no file events (GW-12).
- The ledger entry's gate 3 record lists asset names.
  This record adds the workflow-to-tag mapping and the hashes (GW-03, GW-04)
  and the absence of any signature (GW-05).
- The ledger entry records the macOS backend as covered by a published asset.
  On kqueue the backend opens one descriptor per watched file as well as per folder,
  which for this tree is about 11786 descriptors (GW-21).
  That is a runtime limit the ledger does not record, and it can block the macOS client
  without any missing platform support.
- The vet report's "Hard constraints" section lists seven constraints and no provenance or security gate,
  while this run's instructions ask for license, provenance and security records.
  Those appear here under the skill's own gates and are marked as overlay gates,
  so they are not counted as vet-report hard constraints in the summary.

## Summary against the hard constraints

Each line names the verdict and the record that decides it.
No verdict here is a score, a ranking or a recommendation.

- Open source with inspectable source: pass, GW-01 and GW-02.
- Runs an arbitrary command, here `systemctl --user start rclone-bisync-garage.service`:
  pass, GW-13 and GW-14.
- Runs on Linux x86-64, macOS arm64 and Windows x64:
  pass, GW-03 (artifacts built for all three), GW-04 (packages and hashes),
  GW-07, GW-08 and GW-09 (backend in source).
  The macOS descriptor cost, about 11786 descriptors for this tree, is an open item in GW-21.
  It does not change this verdict, because the platform support is present in source, assets and CI;
  it may still block that client at runtime if the limit cannot be raised that far.
- Watches every folder under the root recursively, including folders created after it starts:
  pass, GW-10 and GW-11;
  the event-coverage limit for content a new folder already holds is GW-12.
- After a change that lands while a sync runs, one more sync starts after that run ends:
  pass natively, GW-15 and GW-16 (a queued run starts when the current one ends, in order).
- Never interrupts a running command; each run finishes:
  pass with the defaults, GW-15 and GW-17;
  the three ways a run is stopped are `run.interrupt`, a shutdown signal, and a fatal watcher error.
- Queues one more run of its command natively when events arrive while the command runs:
  pass, GW-15 (queue, `run.concurrency: 1`, `run.interrupt: none`, `queue.size` 2048) and GW-16.

Overlay and other gates:

- License and dependency surface: pass, GW-01 and GW-06.
- Prebuilt provenance and checksums: pass on mapping, with the limit that nothing is signed
  and no rebuild was attempted (GW-03, GW-04, GW-05).
- Security boundary: pass, GW-22 and GW-23.
- Maintenance audit: recorded, not a hard gate.
  Active releases with a single maintainer, no issue or pull request history,
  and a security policy that covers only the latest release.
- Tests and CI: pass, GW-24.

## Open items and limits of this record

- Not executed, by this run's rule:
  no install, build, test, script, binary or container of the candidate.
  Every behavioural statement is a source reading,
  including the busy behaviour, the queue, the watch-limit failure and the exclusion rules.
- Not verified on the real tree:
  a start on `~/Plain` with its Japanese and Chinese names, spaces, parentheses, brackets and quotes
  was not run, so GW-14's quoting conclusion is from source and the project's own guide,
  not from an end-to-end execution.
- Not attempted: a reproducible rebuild of any release asset.
  GW-03 maps artifacts to the tagged commit through the workflow and the bot-created tag,
  which is strong but is not a byte-for-byte reproduction.
- Not measured: the macOS descriptor cost (GW-21) and any Windows runtime behaviour.
  Both need a run on those machines; GW-21's number is arithmetic, not a measurement.
- Not in scope of this run: how the watcher behaves inside a systemd user unit,
  its interaction with `rclone-bisync-garage.service`, and the 5-minute timer.
  Those belong to the design record and to finalist validation.
- Translations in this file are mine.
  The source comments and commit messages quoted above are Korean,
  and each translation is marked where it appears.
- The account of `directory-watcher`'s own docs in the vet report was written from documentation only.
  This record confirms the doc-derived claims about queueing, exclusion, notification and duplication
  against source, and adds the limits GW-12, GW-20 and GW-21.
