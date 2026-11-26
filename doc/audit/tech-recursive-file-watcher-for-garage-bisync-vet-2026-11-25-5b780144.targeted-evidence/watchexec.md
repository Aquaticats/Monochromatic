# Targeted evidence: watchexec 2.8.0

> Stopped at 2026-11-25T18:18:47-05:00 by the coordinating session,
> when the owner excluded watchexec from the vet
> (`doc/planning/garage-bisync-run-on-file-change.md`, "Owner answers, round 5").
> The run never wrote its final summary, so this record is partial;
> it is kept as evidence on an excluded candidate, not as a basis for any pick.

Status:
 agent-written during the targeted-evidence run of 2026-11-25;
 read-only against the candidate (nothing was installed, built, tested or run);
 incomplete,
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
This file answers that for watchexec 2.8.0 with primary evidence:
the released tag's own source,
the release workflow committed at that tag,
the published artifacts and their checksums,
the dependency crates pinned in `Cargo.lock`,
and the repository's issue and pull-request history.

Every claim below is either read from source (cited as `path:line` plus an excerpt)
or read from a GitHub API response (cited as the exact command and the fields used).
Where a claim rests on a dependency's source rather than watchexec's own,
the record says which crate,
which version,
and how that version was pinned.
No behaviour in this file was measured at runtime.
The vet report's "External execution gate" and "Finalist validation" sections remain open,
so nothing here can stand in for them.

## Candidate identity and access

Candidate: watchexec 2.8.0, CLI crate `watchexec-cli` 2.8.0.
Repository: `watchexec/watchexec`.
Release tag: `v2.8.0`.
Access date for every URL and API query below: 2026-11-25,
in a session that ran from 18:02 to 18:40 -05:00.

The clone was made exactly as instructed:

```console
$ gh repo clone watchexec/watchexec "/home/user/temp/agent/watchexec-source-for-garage-watcher-vet-2026-11-25" -- --depth 1 --branch v2.8.0
```

Clone path: `/home/user/temp/agent/watchexec-source-for-garage-watcher-vet-2026-11-25`.
It is a shallow clone and is deleted when this run ends,
so every path cited below is given relative to that clone root and can be re-created with the
command above.

What the tag points at
(`git rev-parse`, `git cat-file -t`, and `gh api repos/watchexec/watchexec/git/tags/<sha>`):

- `v2.8.0` is an annotated tag object `f8029889f11575d35e224c300a5dc8e1468738d1`,
  message `chore: Release package watchexec-cli version 2.8.0`,
  pointing at commit `07e2b99e833da655571fc4001ae47b03f44742c7`.
- That commit is dated `2026-10-05T16:33:05Z`,
  subject `release: update CLI metadata`,
  and its tree is `5c557c9d485d3d2fc746b969000cfbf649f3039b`.
- The release commit on `main` is `a5be9f092e44a68d199bd103a059e5e3130252c4`
  (`release (#1143)`, dated `2026-10-06T17:01:51Z`),
  whose second parent is `07e2b99` and whose tree is the same
  `5c557c9d485d3d2fc746b969000cfbf649f3039b`.
- `gh api repos/watchexec/watchexec/releases/tags/v2.8.0` reports
  `"published":"2026-10-06T17:04:04Z"`, `"draft":false`, `"prerelease":false`.

Working tree layout at the tag:
a Cargo workspace (`Cargo.toml:1-14`) with `crates/cli` (the `watchexec` binary),
`crates/lib` (the `watchexec` library crate, version 8.4.4 at this tag),
`crates/supervisor`, `crates/events`, `crates/signals`,
`crates/filterer/globset`, `crates/filterer/ignore`, `crates/ignore-files`,
`crates/project-origins`, `crates/bosion`, `crates/test-socketfd`,
`doc/watchexec.1.md` and the generated `doc/watchexec.1`.

Version and licence fields read at the tag:
`crates/cli/Cargo.toml:2-6` is `name = "watchexec-cli"`, `version = "2.8.0"`,
`license = "Apache-2.0"`.
Library versions wired into the CLI (`crates/cli/Cargo.toml:81-98`):
`watchexec` 8.4.4, `watchexec-events` 6.1.3, `watchexec-signals` 5.1.1,
`watchexec-filterer-globset` 8.0.6, all as `path` dependencies.

### Dependency sources used for backend claims

The watching backend is chosen by the `notify` crate, not by watchexec's own code.
`Cargo.lock:2214-2217` pins `notify` 8.2.0 with
`checksum = "4d3d07927151ff8575b7087f245456e549fea62edf0ec4e565a5ee50c8402bc3"`.
`Cargo.lock:2246-2249` pins `notify-types` 2.1.0 with
`checksum = "42b8cfee0e339a0337359f3c88165702ac6e600dc01c0cc9579a92d62b08477a"`.

Both crate archives were downloaded read-only from crates.io into this run's scratch folder and
their archive checksums recomputed, and both matched the `Cargo.lock` values:

```console
$ sha256sum notify-8.2.0.crate
4d3d07927151ff8575b7087f245456e549fea62edf0ec4e565a5ee50c8402bc3  notify-8.2.0.crate
$ sha256sum notify-types-2.1.0.crate
42b8cfee0e339a0337359f3c88165702ac6e600dc01c0cc9579a92d62b08477a  notify-types-2.1.0.crate
```

Paths written `notify-8.2.0/src/...` and `notify-types-2.1.0/src/...` below are relative to those
two extracted archives, reproducible with
`curl -sSL https://static.crates.io/crates/notify/notify-8.2.0.crate`.
Nothing from either crate was built or run.

## How to read a record

Each record gives the fields the governing skill's "Evidence records" section asks for:
candidate and exact version,
the claim and why it decides a gate,
the gate or overlay it bears on,
a status,
the primary evidence with an excerpt and access date,
corroboration or counterevidence where material,
and an outcome.
Records E01 to E23 decide hard constraints or the capabilities the vet report records for every
candidate; records E24 to E29 are the maintenance audit.

Status words are the skill's: pass, fail, open, not applicable.
"Open" means this read-only run could not settle it,
never that the candidate failed.

## Hard-constraint records

### Licence and inspectable source

#### E01 Licence text and file path

Candidate and version: watchexec 2.8.0.
Claim: the licence is Apache-2.0, with the licence text in the repository at the released tag.
Gate: hard constraint "open source with inspectable source";
skill section "License and dependency surface".
Status: pass.
Primary evidence: `LICENSE:1-3` in the clone at tag `v2.8.0` reads
`Apache License` / `Version 2.0, January 2004` / `http://www.apache.org/licenses/`;
the file is 201 lines, the complete Apache-2.0 text.
`crates/cli/Cargo.toml:6` declares `license = "Apache-2.0"`.
Access date: 2026-11-25.
Corroboration: `gh api repos/watchexec/watchexec` reports `"license":"Apache-2.0"`;
the crates.io record for `watchexec-cli` 2.8.0 reports `"license":"Apache-2.0"`.
Outcome: pass. No compatibility question arises for a tool that is run, not linked into this
repository's code.

#### E02 Released version has inspectable source at the tag

Candidate and version: watchexec 2.8.0.
Claim: the source that produced the released artifacts is readable at the release tag, and the
crate published to crates.io carries the same tree.
Gate: hard constraint "open source with inspectable source";
skill section "Production source".
Status: pass.
Primary evidence: the clone above is at tag commit `07e2b99`, tree `5c557c9d`,
and contains the code paths this file cites
(`crates/cli/src/config.rs`, `crates/lib/src/sources/fs.rs`,
`crates/lib/src/sources/fs/recursor.rs`, `crates/supervisor/src/command/conversions.rs`).
Corroboration for the crates.io tarball:
`curl -sS https://crates.io/api/v1/crates/watchexec-cli/2.8.0` returns
`"checksum":"a9e73398f1d43104592c8b3aed966fac990773962dc3fd11939ba4cee1d23745"`,
`"created_at":"2026-10-06T17:04:00.987432Z"`, and
`"trustpub_data":{"provider":"github","repository":"watchexec/watchexec", ... "sha":"a5be9f092e44a68d199bd103a059e5e3130252c4"}`,
which is the release commit whose tree equals the tag's tree (see the identity section).
Access date: 2026-11-25.
Outcome: pass. The registry package and the tag carry the same source tree.

### Provenance of what a user installs

#### E03 Linux x86-64 artifact: built from the tag by the committed workflow

Candidate and version: watchexec 2.8.0, `x86_64-unknown-linux-gnu`.
Claim: the archive a Linux x86-64 user installs is built from the `v2.8.0` tag by a workflow file
committed at that tag, and its bytes are identified by a published checksum.
Gate: prebuilt overlay, "Unknown build provenance is a hard failure"
(skill section "Native, Wasm, and prebuilt overlay").
Status: pass.
Primary evidence, all read at the tag:

- `.github/workflows/release-cli.yml:5-8` triggers on `push: tags: v*.*.*`,
  so the workflow that runs is the copy committed at the pushed tag.
- `.github/workflows/release-cli.yml:40-44` declares the matrix row
  `- name: linux-amd64-gnu`, `os: ubuntu-22.04`,
  `target: x86_64-unknown-linux-gnu`, `cross: false`, `experimental: false`.
- `.github/workflows/release-cli.yml:200-207` builds it with
  `cargo build -p watchexec-cli --release --locked --target x86_64-unknown-linux-gnu`
  (the `${{ matrix.cross && 'cross' || 'cargo' }}` form resolves to `cargo`).
- `.github/workflows/release-cli.yml:225-227` archives it with
  `tar cavf "$dst.tar.xz" "$dst"`, and `.github/workflows/release-cli.yml:228-237`
  adds the `.deb` and `.rpm` packages via `cargo-deb 3.7.0` and `cargo-generate-rpm`.
- `.github/workflows/release-cli.yml:280-291` computes `B3SUMS`, `SHA512SUMS`,
  `SHA256SUMS` and a per-file `.b3`, `.sha256` and `.sha512`.
- `.github/workflows/release-cli.yml:294-311` publishes those files on the release
  named `CLI v${{ version }}` for `tag_name: v${{ version }}`.

Corroboration, three independent sources agreeing on one hash:

- `gh api repos/watchexec/watchexec/releases/tags/v2.8.0` lists the asset
  `watchexec-2.8.0-x86_64-unknown-linux-gnu.tar.xz` (2940160 bytes) plus its
  `.b3`, `.sha256` and `.sha512` files and the bulk `SHA256SUMS` file.
- The published `SHA256SUMS` line is
  `21839e9840ef592b4d571885ccbd692f54ae5bd4231f3caf29fcf31fa7723afe  watchexec-2.8.0-x86_64-unknown-linux-gnu.tar.xz`,
  and the asset's own `.sha256` file contains the same value.
- This repository installs watchexec from the same release:
  `toolchain.lock.json:393-396` records
  `https://github.com/watchexec/watchexec/releases/download/v2.8.0/watchexec-2.8.0-x86_64-unknown-linux-gnu.tar.xz`
  with `sha256: 21839e9840ef592b4d571885ccbd692f54ae5bd4231f3caf29fcf31fa7723afe`,
  which equals the published value.

Access date: 2026-11-25.
Outcome: pass. The chain tag, workflow, asset, checksum and this repository's installed binary
all agree.

#### E04 macOS arm64 artifact: built from the tag by the committed workflow

Candidate and version: watchexec 2.8.0, `aarch64-apple-darwin`.
Claim: same as E03, for the macOS arm64 archive.
Gate: prebuilt overlay provenance.
Status: pass, with one provenance caveat recorded in E06.
Primary evidence at the tag:

- `.github/workflows/release-cli.yml:118-121` declares
  `- name: mac-arm64`, `os: macos-15`, `target: aarch64-apple-darwin`, `cross: false`.
- `.github/workflows/release-cli.yml:200-207` builds it with
  `cargo build -p watchexec-cli --release --locked --target aarch64-apple-darwin`
  on that runner (a native build, not a cross build).
- `.github/workflows/release-cli.yml:225-227` archives it as `.tar.xz`.

Corroboration: the release lists `watchexec-2.8.0-aarch64-apple-darwin.tar.xz`
(2069904 bytes) with `.b3`, `.sha256` and `.sha512` files;
the published `SHA256SUMS` line is
`8da8bdbfb8568138616a082c05a87224e08d5cb2994feb956426291cf0c6fe10  watchexec-2.8.0-aarch64-apple-darwin.tar.xz`.
No lock entry in this repository pins this archive, because this repository does not install it.
Access date: 2026-11-25.
Outcome: pass for provenance; see E06 for the missing signature and the missing notarisation.

#### E05 Windows x64 artifact: built from the tag by the committed workflow

Candidate and version: watchexec 2.8.0, `x86_64-pc-windows-msvc`.
Claim: same as E03, for the Windows x64 archive.
Gate: prebuilt overlay provenance.
Status: pass, with the caveat in E06.
Primary evidence at the tag:

- `.github/workflows/release-cli.yml:124-127` declares
  `- name: windows-x86-64`, `os: windows-latest`, `target: x86_64-pc-windows-msvc`,
  `cross: false`.
- `.github/workflows/release-cli.yml:200-207` builds it with
  `cargo build -p watchexec-cli --release --locked --target x86_64-pc-windows-msvc`.
- `.github/workflows/release-cli.yml:239-241` archives it with
  `7z a "$dst.zip" "$dst"`.

Corroboration: the release lists `watchexec-2.8.0-x86_64-pc-windows-msvc.zip`
(3574886 bytes) with its `.b3`, `.sha256` and `.sha512` files;
the published `SHA256SUMS` line is
`0c872fc18c8e801f21614c6daf2504a5826d5c705bb67f7a583d879f5410d9ea  watchexec-2.8.0-x86_64-pc-windows-msvc.zip`.
Access date: 2026-11-25.
Outcome: pass for provenance; see E06.

#### E06 No signature, no attestation, no notarisation, and floating action refs

Candidate and version: watchexec 2.8.0.
Claim: the artifacts are identified by checksums only; the pipeline has no signing step, and some
of its actions are referenced by moving tags rather than by commit.
Gate: prebuilt overlay ("checksum or signature", "release workflow", "reproducible verification").
Status: pass on the gate as written (a checksum exists), recorded as a caveat rather than a
failure.
Primary evidence:

- `rg -n -i 'sign|cosign|sigstore|attest|notariz|codesign' .github/` returns no workflow match
  (the three hits are issue-template `assignees:` lines and a `crates/signals` path);
  there is no signing, attestation or notarisation step in either workflow.
- `gh api repos/watchexec/watchexec/releases/tags/v2.8.0` lists no `.sig`, `.asc`, `.pem`
  or attestation asset.
- `.github/workflows/release-cli.yml:20`, `:146`, `:262` use `actions/checkout@v7`;
  `:225` is `actions/upload-artifact@v7`; `:262` is `actions/download-artifact@v8`;
  `Swatinem/rust-cache@v2` is used at `:191`. These are moving major-version refs.
  By contrast `taiki-e/install-action@v2.87.20` is pinned by version (`:196`, `:265`)
  and `softprops/action-gh-release@efb35369e0ad2afab669f228072c1b0d510eae64`
  is pinned by commit (`:294`).
- `crates/cli/build.rs:5` calls `bosion::gather()`, which records build-time git information;
  `Cargo.toml:23-27` sets release profile `lto = true`, `codegen-units = 1`,
  `strip = "symbols"`, and `debug = 1`, which is not a bit-reproducible configuration by itself.

Access date: 2026-11-25.
Outcome: pass with caveats. A reader reproduces integrity against the published checksums, not a
signature; a third party rebuilding the tag would not be expected to produce identical bytes from
these settings (this was not attempted).

#### E07 The crates.io package maps to the tag through an identical tree

Candidate and version: `watchexec-cli` 2.8.0 on crates.io.
Claim: a user installing from the registry (`cargo install --locked watchexec-cli`, or
`cargo binstall`) gets source identical to the tag, built on their own machine.
Gate: prebuilt overlay, source-to-artifact mapping for the registry route.
Status: pass.
Primary evidence:

- `https://crates.io/api/v1/crates/watchexec-cli/2.8.0` reports
  `"checksum":"a9e73398..."` and
  `"trustpub_data":{"provider":"github","repository":"watchexec/watchexec","run_id":"37500360873","sha":"a5be9f092e44a68d199bd103a059e5e3130252c4"}`.
- `gh api repos/watchexec/watchexec/commits/a5be9f0` reports tree
  `5c557c9d485d3d2fc746b969000cfbf649f3039b`, equal to the tag commit's tree.
- `release-plz.toml:1-14` shows the release route: `git_release_enable = true`,
  `git_tag_enable = true`, `git_tag_name = "v{{ version }}"` for `watchexec-cli`.
- `README.md:52` documents `cargo install --locked watchexec-cli` and
  `cargo binstall watchexec-cli`.

Access date: 2026-11-25.
Outcome: pass. The registry route and the prebuilt route both map to the tag.

### Watching backends in source for the three target systems

#### E08 Linux: inotify through notify's recommended watcher

Candidate and version: watchexec 2.8.0 with `notify` 8.2.0.
Claim: on Linux the watcher is inotify.
Gate: hard constraint "runs on Linux x86-64"; capability record for watch semantics.
Status: pass.
Primary evidence:

- `crates/lib/src/sources/fs/backend.rs:104-118`: `create_notify` calls
  `notify::RecommendedWatcher::new(handler, config)` for the native kind.
- `crates/lib/src/sources/fs.rs:63-69`: the default `Watcher` variant is
  `#[default] Native`, documented as "The Notify-recommended watcher on the platform".
- `notify-8.2.0/src/lib.rs:403-404`:
  `/// The recommended Watcher implementation for the current platform`
  followed by `#[cfg(any(target_os = "linux", target_os = "android"))]`
  `pub type RecommendedWatcher = INotifyWatcher;`.
- `notify-8.2.0/src/inotify.rs:489-497` (`add_single_watch`) sets the inotify mask to
  `ATTRIB | CREATE | OPEN | DELETE | CLOSE_WRITE | MODIFY | MOVED_FROM | MOVED_TO`.

Access date: 2026-11-25.
Outcome: pass. One inotify instance per watcher, with one watch per registered directory (E11).

#### E09 macOS: FSEvents stream from notify

Candidate and version: watchexec 2.8.0 with `notify` 8.2.0.
Claim: on macOS the watcher is FSEvents.
Gate: hard constraint "runs on macOS arm64".
Status: pass.
Primary evidence:

- `notify-8.2.0/src/lib.rs:405-407`:
  `#[cfg(all(target_os = "macos", not(feature = "macos_kqueue")))]`
  `pub type RecommendedWatcher = FsEventWatcher;`, and
  `notify-8.2.0/Cargo.toml:68-70` has `default = ["macos_fsevent"]`
  with `macos_fsevent = ["fsevent-sys"]`, while watchexec's
  `crates/lib/Cargo.toml:20` declares `notify = "8.2.0"` with no feature changes,
  so the default feature set (FSEvents, not kqueue) is what ships.
- `notify-8.2.0/src/fsevent.rs:293-305` builds the stream with
  `flags: fs::kFSEventStreamCreateFlagFileEvents | fs::kFSEventStreamCreateFlagNoDefer`
  and `latency: 0.0`.
- `crates/lib/src/sources/fs.rs:125-133` maps `notify::WatcherKind::Fsevent` to
  watchexec's `RecursionStrategy::RecursiveRoots`.

Access date: 2026-11-25.
Outcome: pass. See E12 for what RecursiveRoots means for coverage and for folders created later.

#### E10 Windows: ReadDirectoryChangesW through notify

Candidate and version: watchexec 2.8.0 with `notify` 8.2.0.
Claim: on Windows the watcher is `ReadDirectoryChangesWatcher`.
Gate: hard constraint "runs on Windows x64".
Status: pass.
Primary evidence:

- `notify-8.2.0/src/lib.rs:409-411`:
  `#[cfg(target_os = "windows")]`
  `pub type RecommendedWatcher = ReadDirectoryChangesWatcher;`.
- `notify-8.2.0/src/windows.rs` is the backend module, exported at
  `notify-8.2.0/src/lib.rs:217-218`.
- `crates/lib/src/sources/fs.rs:125-133` maps
  `notify::WatcherKind::ReadDirectoryChangesWatcher` to `RecursionStrategy::PerDirectory`,
  the same managed per-directory recursion Linux uses.

Access date: 2026-11-25.
Outcome: pass.

### Recursion, including folders created after the watcher starts

#### E11 Linux: a new folder gets its own watch, before it is scanned

Candidate and version: watchexec 2.8.0.
Claim: with inotify, watchexec walks the tree itself and adds one non-recursive watch per
directory, and when a directory is created later it registers a watch for it and then walks into
it, so nested new folders are covered too.
Gate: hard constraints "watches every folder under the root recursively, including folders
created after it starts" and "runs on Linux x86-64".
Status: pass, from source, with the repository's own tests as corroboration.
Primary evidence, the chain from a create event to a new watch:

- `crates/lib/src/sources/fs.rs:125-133`: inotify maps to
  `RecursionStrategy::PerDirectory`; `crates/lib/src/sources/fs.rs:113-121` shows
  `PerDirectory` "owns" the logical tree, so watchexec, not notify, owns recursion.
- `crates/lib/src/sources/fs.rs:794-801`: `topology_messages` turns every
  `EventKind::Create(_)` path into `Topology::Create(path)`.
- `crates/lib/src/sources/fs.rs:511`: `Topology::Create(path) => recursor.topology_create(path)`.
- `crates/lib/src/sources/fs/recursor.rs:511-519` `topology_create`, then
  `crates/lib/src/sources/fs/recursor.rs:525-539` `topology_create_one_with_removal`,
  which queues `Work::Candidate { root, path, transient_retries }` for the new path.
- `crates/lib/src/sources/fs/recursor.rs:1932-1939`: `step_candidate` calls
  `self.add_logical(path, identity, root, RecursiveMode::NonRecursive, result)`,
  then queues `Work::Scan` for the same path (`:1941-1947`).
- `crates/lib/src/sources/fs/recursor.rs:2151`: `match self.backend.watch(&path, mode)`,
  which reaches `notify::Watcher::watch` in
  `crates/lib/src/sources/fs/backend.rs:64-70`.
- `crates/lib/src/sources/fs/recursor.rs:1993-2004`: `step_scan` pushes a
  `Work::Candidate` for every entry found inside a directory, so a folder created with
  contents gets watches for its subfolders as well, recursively.

Ordering and pruning:

- `crates/lib/src/sources/fs/recursor.rs:2902` is the unit test
  `watches_each_directory_before_scanning_it`, which pins the order that makes the
  create-then-scan sequence lossless.
- `crates/lib/src/sources/fs/recursor.rs:2070-2084` `candidate_allowed` consults
  `self.filter.check_dir(&candidate)` for each path component, so an ignored folder is
  never registered at all.
- `crates/lib/tests/fs_worker_real.rs:484` is the integration test
  `directories_created_or_moved_in_after_readiness_are_covered`, run against the native
  backend and the poll backend (`managed_watcher_cases` at `:38-60`),
  which writes into a directory created after readiness and asserts the event arrives.
- `crates/lib/tests/fs_worker_real.rs:1045-1086` `assert_linux_inotify_registration`
  reads `/proc` inotify fdinfo for the running process and asserts that a given directory is
  or is not registered, which is direct evidence that per-directory inotify watches exist.

Access date: 2026-11-25.
Outcome: pass. On Linux the watch count for the target tree is one watch per directory plus one
inotify instance, both read from source, not measured here.

#### E12 macOS and Windows recursion, including folders created later

Candidate and version: watchexec 2.8.0.
Claim: on Windows the same per-directory recursion applies; on macOS watchexec registers one
recursive FSEvents stream per root and tracks the logical tree itself, so new subfolders are
reported by the stream without any additional registration.
Gate: hard constraint "watches every folder, including folders created later", on the other two
platforms.
Status: pass for Windows by E11's shared code path; pass for macOS on a reading of notify's
FSEvents backend, with the caveat that FSEvents behaviour was not measured here.
Primary evidence:

- Windows: `crates/lib/src/sources/fs.rs:125-133` maps
  `ReadDirectoryChangesWatcher` to `PerDirectory`, the same strategy and the same recursor
  code as Linux (E11).
- macOS: `crates/lib/src/sources/fs.rs:125-133` maps `Fsevent` to `RecursiveRoots`;
  `crates/lib/src/sources/fs/recursor.rs:2221-2248` `add_virtual_logical` records the
  root as a logical watch without registering descendants, and
  `crates/lib/src/sources/fs/recursor.rs:3282-3310` and `:3316-3362` are tests showing
  FSEvents batches coverage at the root and reconciles only the logical tree.
- FSEvents itself: `notify-8.2.0/src/fsevent.rs:388-408` appends the watched path to a
  `CFArray` and records only whether the request was recursive
  (`self.recursive_info.insert(canonical_path, recursive_mode.is_recursive())`);
  there is no per-directory registration, and the stream carries
  `kFSEventStreamCreateFlagFileEvents` (`:301`), which reports events for files in the
  whole subtree.

Access date: 2026-11-25.
Outcome: pass, with the macOS half resting on notify's backend source rather than a measurement.
This run executed nothing, so no runtime confirmation exists for any platform.

### Running the command

#### E13 The command is a shell string by default, with an argv mode

Candidate and version: watchexec 2.8.0.
Claim: watchexec joins the trailing command words with single spaces and hands the result to a
shell with `-c` (or `/C` for CMD.EXE), unless `--shell=none` is given, in which case it execs the
first word with the rest as arguments.
Gate: hard constraint "runs an arbitrary command, here `systemctl --user start
rclone-bisync-garage.service`".
Status: pass. That command contains no shell metacharacters, so both modes run it identically.
Primary evidence:

- `crates/cli/src/args.rs:95-107`: the help text for the trailing command says
  "every part of the command is joined together into one string with a single ascii space
  character, and given to the shell"; the field is `pub program: Vec<String>` with
  `trailing_var_arg = true`.
- `crates/cli/src/config.rs:1111` `interpret_command_args` clones `args.program`;
  `crates/cli/src/config.rs:1113-1148` picks the shell
  (`--shell`, else `$SHELL`, else `sh` on Unix; `pwsh`, `powershell` or `cmd` on Windows);
  `crates/cli/src/config.rs:1149` maps `none` to no shell.
- `crates/cli/src/config.rs:1169-1178`:
  `Program::Shell { shell, command: cmd.join(" "), args: Vec::new() }`, else
  `Program::Exec { prog: cmd.remove(0).into(), args: cmd }`.
- `crates/supervisor/src/command/conversions.rs:16-49` turns that into a
  `tokio::process::Command`: `Program::Shell` runs the shell program with the command string
  as an argument, `Program::Exec` runs the program directly with an argument vector.
- `crates/supervisor/src/command/conversions.rs:126-128`: on Unix, when the string is a single
  simple command, the shell is given `exec <command>`, so the shell replaces itself and the
  spawned child is the real command. `shell_command_is_execable` (`:150-190`) refuses that
  rewriting for `;`, `&`, `|`, newlines, assignments, reserved words and subshells.
  For `systemctl --user start rclone-bisync-garage.service` the rewrite applies.
- Process wrapping (`crates/cli/src/config.rs:1182-1196`): default `WrapMode::Auto` means a new
  process group on Unix (a session on macOS) and a Job Object on Windows.

Access date: 2026-11-25.
Outcome: pass. `systemctl --user start ...` arrives at `sh -c "exec systemctl --user start
rclone-bisync-garage.service"`.

#### E14 Changed paths never reach the command line

Candidate and version: watchexec 2.8.0.
Claim: changed paths are not substituted into the command line. They are emitted, if at all, into
environment variables, into the command's stdin, or into a temporary file, and the default at
2.8.0 is not to emit them at all.
Gate: capability record ("whether changed paths are put into the command line, the environment or
stdin, and how they are quoted"), plus the robustness question for names containing Japanese and
Chinese characters, spaces, parentheses, brackets and quotes.
Status: pass: nothing is quoted into a shell, because nothing is inserted into the command line.
Primary evidence:

- No substitution feature exists: `rg -n 'placeholder|"\{}"' crates/cli crates/lib
  crates/supervisor doc/watchexec.1.md` returns only Rust format strings and a test fixture
  named `get_placeholder_data`, and the manual documents no such feature.
- `crates/cli/src/args/events.rs:303-308`: `--emit-events-to` has
  `default_value = "none"`, and `doc/watchexec.1.md:420-425` says
  "It defaults to 'none', which doesn't emit event information at all."
- Modes that do emit: `crates/cli/src/config.rs:1637-1690` `emit_events_to_command`
  switches on `EmitEvents::Environment` (environment variables),
  `Stdio` and `JsonStdio` (the command's stdin), `File` and `JsonFile`
  (`$WATCHEXEC_EVENTS_FILE` pointing at a temporary file), and `None` (nothing).
- Environment mode: `crates/cli/src/emits.rs:12-18` builds `WATCHEXEC_<KIND>_PATH`
  variables from `summarise_events_to_env`, which
  (`crates/lib/src/paths.rs:144-166`) sorts paths and joins them with `PATH_SEPARATOR`
  (`:` on Unix, `;` elsewhere, `crates/lib/src/paths.rs:13-16`) into an `OsString`.
  Values are not shell-parsed, so spaces, quotes, brackets and non-ASCII names survive
  unchanged; a file name containing the separator character would be ambiguous, which the
  manual itself calls out as a limitation of this mode
  (`doc/watchexec.1.md:481-492`).
- Stdin and file text modes go through `crates/cli/src/emits.rs:23-57`, which formats
  `kind:path` lines and uses `path.to_string_lossy()` (`:35`, `:51`), so those two modes are
  lossy for paths that are not valid UTF-8 (the target tree's names are UTF-8).
- The manual's warning on the environment mode: `doc/watchexec.1.md:455-462` calls it
  "the legacy mode, is deprecated", limited and truncation-prone for large bursts.

Access date: 2026-11-25.
Outcome: pass. A trigger command needs no path information, and the default emits none.

### Busy behaviour, the queue, and never interrupting a run

#### E15 Default busy behaviour is to drop the events

Candidate and version: watchexec 2.8.0.
Claim: with no flag, events that arrive while the command runs are ignored; the default is
`do-nothing`, which is the drop behaviour.
Gate: capability record ("busy behaviour by default"), and the hard constraints "queues one more
run" and "never interrupts a running command", which the default does not meet.
Status: pass as a record of behaviour; the default fails the queue requirement, which is why
`--on-busy-update=queue` exists (E16).
Primary evidence:

- `crates/cli/src/args/events.rs:29-36`: `--on-busy-update` with
  `default_value = "do-nothing"`, documented as
  "Default is to 'do-nothing', which ignores events while the command is running".
  The `OnBusyUpdate` enum at `crates/cli/src/args/events.rs:370-377` derives
  `#[default] Queue`, but the clap attribute at `:31` overrides it for the CLI.
  A library user constructing `OnBusyUpdate::default()` would get `Queue`; the shipped
  binary's default is `do-nothing`.
- `crates/cli/src/config.rs:631-632`: `OnBusyUpdate::DoNothing => {}`, an empty arm.
- `doc/watchexec.1.md:538-549` repeats the same default and the same description.

Access date: 2026-11-25.
Outcome: drop.

#### E16 `--on-busy-update=queue` queues exactly one more run, natively

Candidate and version: watchexec 2.8.0.
Claim: with `--on-busy-update=queue`, a change that lands while the command runs causes one more
run after that run ends, and multiple changes during one run coalesce into that single run.
Gate: hard constraints "after a change that lands while a sync runs, one more sync starts after
that run ends" and "queues one more run of its command natively".
Status: pass, natively, with no wrapper.
Primary evidence:

- `crates/cli/src/args/events.rs:29-36` documents `queue` as
  "run the command once again when the current run has finished if any events occur while
  it's running"; `doc/watchexec.1.md:539-547` says the same.
- `crates/cli/src/config.rs:289`: `let queued = Arc::new(AtomicBool::new(false));`,
  one flag per process.
- `crates/cli/src/config.rs:618-624`: at action time the handler reads
  `let is_running = matches!(context.current, CommandState::Running { .. });`.
- `crates/cli/src/config.rs:680-716` is the whole queue arm:
  `let already_queued = queued.fetch_or(true, Ordering::SeqCst);` and, when it was not
  already queued, a spawned task that does `job.to_wait().await;` (`:694`),
  `job.start();` (`:696`), `job.run(...)` (`:697-712`) and then
  `queued.store(false, Ordering::SeqCst);` (`:715`).
- The waiting meaning of those calls: `crates/supervisor/src/job/job.rs:231-233`
  (`to_wait` sends `Control::NextEnding`, which parks until the child finishes,
  `crates/supervisor/src/job/task.rs:437-446`), while
  `crates/supervisor/src/job/job.rs:245-247` (`run`) queues a `Control::SyncFunc`, which the
  job task executes immediately and then raises the ticket
  (`crates/supervisor/src/job/task.rs:448-453`, `:494-495`).

What the code implies for a burst, stated precisely:

- At most one run is pending at any time; there is no growing queue, because the pending state is
  a single boolean and the second, third and later events during the same run hit
  `if already_queued { debug!("next start is already queued, do nothing"); }` (`:683-685`).
- The flag clears as soon as the queued run has been spawned and its run callback has executed,
  not when that run finishes (`:715` runs right after the non-blocking `job.run` ticket). A
  change that lands during the queued run therefore queues a further run, so a long
  back-to-back sequence is handled, but two changes inside the short window between
  `job.start()` and `queued.store(false)` would collapse to one.
  No runtime measurement of that window was taken in this run.

Access date: 2026-11-25.
Outcome: pass. The vet report's ledger entry ("met natively, `--on-busy-update=queue`") is
confirmed and sharpened: the queue is a coalescing single-slot flag, not a FIFO.

#### E17 Queue mode never stops a running command

Candidate and version: watchexec 2.8.0.
Claim: in queue mode nothing in the busy-update path signals or kills the running command, and the
remaining ways a run can end early are outside the file-event path.
Gate: hard constraint "never interrupts a running command; each run finishes".
Status: pass for queue mode, with the stop paths enumerated below.
Primary evidence:

- `crates/cli/src/config.rs:631-632` (`DoNothing`) and `:680-716` (`Queue`) contain no
  `job.signal`, `job.stop`, `stop_with_signal`, `restart` or `restart_with_signal` call.
  The kill and signal behaviours live only in the `Signal` arm (`:632-639`) and the two
  `Restart` arms (`:639-679`), which are the modes the design would not use.
- Remaining stop paths, all outside the busy-update decision:
  - quitting watchexec: `crates/cli/src/config.rs:381-397` (`quit_gracefully` with
    `--stop-signal` then `--stop-timeout`), reached from SIGINT/SIGTERM, from `--stdin-quit`
    EOF, and from `--interactive` keys (`:461-520`);
  - `--timeout`: `crates/cli/src/config.rs:1422-1450` selects between the child ending and a
    timer, and on the timer stops the child with `--stop-signal`/`--stop-timeout`;
  - `--once`: `crates/cli/src/config.rs:418-456`, a debug mode that runs once and quits;
  - `--exit-on-error`: `:408-413` quits after a failed run.
- Process-group wrapping (`crates/cli/src/config.rs:1182-1196`,
  `crates/supervisor/src/command/conversions.rs:52-70`) means those stops reach the whole group,
  which is what a wrapper around `systemctl --user start` would have to survive.

Access date: 2026-11-25.
Outcome: pass in queue mode. With the flags the design would use (no `--restart`, no `--signal`,
no `--timeout`, no `--once`), a file event can never end a run.

### Further capability records

#### E18 Debounce: `--debounce`, default 50 ms, window measured from the first event

Candidate and version: watchexec 2.8.0.
Claim: events are coalesced into one action per window; the option is `--debounce` (short `-d`),
default 50 ms; the window starts at the first event of a batch and does not wait for a trailing
quiet period.
Gate: capability record ("merges a burst of events into one run (a quiet period)").
Status: pass, with the naming correction that it is a fixed window, not a trailing quiet period.
Primary evidence:

- `crates/cli/src/args/events.rs:117-118`: `default_value = "50ms"`, documented at
  `:105-125` as "Time to wait for new events before taking action" and
  "The default is 50 milliseconds".
- `crates/cli/src/config.rs:178` `config.throttle(args.events.debounce.0);`.
- `crates/lib/src/action/worker.rs:113-207` `throttle_collect`: when the set is empty it waits
  indefinitely (`:126-131`), and on the first accepted event it resets the window
  (`:180-183`, `last = Instant::now()`), then keeps collecting until
  `last.elapsed() >= throttle` (`:192-197`). Events after that point go into the next batch.
  An `Urgent` priority event bypasses the wait (`:158-159`, `:188-196`).

Access date: 2026-11-25.
Outcome: pass. A 50 ms default coalesces a save burst; a longer value is a single flag.

#### E19 Path exclusion

Candidate and version: watchexec 2.8.0.
Claim: paths can be excluded with glob patterns and filter files, plus a default ignore list, and
excluded directories are pruned so they are not watched at all.
Gate: capability record ("excludes paths by pattern", the rclone `*.partial` case).
Status: pass.
Primary evidence:

- `crates/cli/src/args/filtering.rs:329-335`: `--ignore` / `-i`, "Filename patterns to filter
  out ... events for files matching the pattern will be excluded".
- `crates/cli/src/args/filtering.rs:344-353`: `--ignore-file`, one pattern per line,
  also settable through `$WATCHEXEC_IGNORE_FILES`.
- `crates/cli/src/args/filtering.rs:199-213`: `--exts` and `--filter` (allow list),
  `:301-317`: `--filter-prog` with a jq expression evaluated in process.
- Ignore discovery switches: `--no-vcs-ignore` (`:96`), `--no-ignore` for project files
  (`:123`), `--no-global-ignore` (`:146`), `--no-default-ignore` (`:158`),
  `--no-discover-ignore` (`:171`), `--ignore-nothing` (`:184`).
- `crates/cli/src/filterer.rs:81-86` loads project ignore files unless discovery is off;
  `crates/cli/src/filterer.rs:90-108` adds a default ignore list that includes
  `.DS_Store`, `watchexec.*.log`, `*.py[co]`, editor swap names, and every VCS directory name
  from `ignore_files::VCS_DIR_NAMES` (`.git`, `.hg`, and so on) with its contents.
- Registration pruning: `crates/lib/src/sources/fs/recursor.rs:2070-2084` `candidate_allowed`
  checks `self.filter.check_dir(&candidate)` for each component before a watch is added,
  and `crates/cli/src/filterer.rs:34-36` delegates `check_dir` to the globset filterer.
  The unit test at `crates/lib/src/sources/fs/recursor.rs:2950`
  (`ignored_descendant_is_pruned_but_explicit_root_is_not`) pins that behaviour.
- Patterns are matched relative to the working directory or the pattern file's directory
  (`crates/cli/src/filterer.rs:127-131`), so an exclusion for rclone's `*.partial` names is a
  one-line pattern.

Access date: 2026-11-25.
Outcome: pass.

#### E20 When the Linux inotify watch limit runs out

Candidate and version: watchexec 2.8.0 with `notify` 8.2.0.
Claim: the failure is reported once as a non-fatal diagnostic naming `inotify.max_user_watches`,
further watch registrations are skipped until something frees a watch or a new scan epoch
starts, and there is no automatic fallback to the poll watcher.
Gate: capability record ("what it does when the inotify watch limit runs out").
Status: pass as a record; behaviour is report-and-degrade, not fail-fast and not silent.
Primary evidence:

- The kernel error: `notify-8.2.0/src/inotify.rs:449-452` maps
  `e.raw_os_error() == Some(libc::ENOSPC)` to `Error::new(ErrorKind::MaxFilesWatch)`,
  and `notify-8.2.0/src/inotify.rs:381-392` stops adding watches and reports once the limit is
  hit.
- The classification: `crates/lib/src/sources/fs/backend.rs:137-152` treats
  `notify::ErrorKind::MaxFilesWatch`, and on Linux also `raw_os_error() == Some(28)`, as
  `ResourceError::Watches`, which becomes `FsWatcherError::TooManyWatches` (`:128-133`).
- The message: `crates/lib/src/error/specialised.rs:17-26` reads
  "failed to instantiate: too many watches" with the Linux help
  "you will want to increase your inotify.max_user_watches, see inotify(7) and
  https://watchexec.github.io/docs/inotify-limits.html".
- The latch: `crates/lib/src/sources/fs/recursor.rs:2186-2201` sets
  `self.resource_latched = true` and pushes that error once
  (`if !self.resource_reported`), returning `AddResult::Skipped` for the path;
  `crates/lib/src/sources/fs/recursor.rs:2147` then short-circuits every later
  registration while the latch is set. The latch clears in
  `crates/lib/src/sources/fs/recursor.rs:817-819` (`schedule_retries`, called from
  `:1669` when a watch is removed and a handle is freed) and in
  `crates/lib/src/sources/fs/recursor.rs:746-753` (`begin_epoch`).
- What the user sees: `crates/lib/src/watchexec.rs:232-249` logs every runtime error and leaves
  it non-fatal unless an error handler upgrades it; the CLI's handler
  (`crates/cli/src/config.rs:139-166`) prints
  `eprintln!("[[Error (not fatal)]]\n{}", Report::new(err.error))` and de-duplicates
  identical messages within a one second window (`RUNTIME_ERROR_SAMPLER`,
  `crates/cli/src/config.rs:66-77`).
- No automatic poll fallback: `crates/lib/src/sources/fs/backend.rs:104-118` only builds a
  `PollWatcher` when the caller asked for `Watcher::Poll`, which the CLI sets only from
  `--poll` (`crates/cli/src/config.rs:180-182`). The error's own help text
  (`crates/lib/src/error/specialised.rs:16`) suggests "perhaps retry with the poll watcher",
  which is a manual remedy.
- Neighbouring failure mode: an inotify queue overflow is turned into a rescan request, not a
  silent loss (`notify-8.2.0/src/inotify.rs:212-214` emits `Flag::Rescan`;
  `crates/lib/src/sources/fs.rs:796-798` maps `event.need_rescan()` to `Topology::Rescan`,
  handled at `crates/lib/src/sources/fs.rs:514`; `need_rescan` is
  `notify-types-2.1.0/src/event.rs:648-650`).

Access date: 2026-11-25.
External corroboration for the kernel semantics only (a documentation page, not a measurement):
`https://watchexec.github.io/docs/inotify-limits.html` states that a recursive watcher establishes
one inotify watch per subdirectory, that the error for exhaustion is ENOSPC (errno 28), and that
reaching `max_queued_events` drops the whole inotify instance.
Outcome: pass. For the target tree the relevant limit is `fs.inotify.max_user_watches = 524288`
against one watch per directory; the design record measures 1454 folders.

#### E21 A run starts at startup unless `--postpone`

Candidate and version: watchexec 2.8.0.
Claim: the command runs once immediately when watchexec starts.
Gate: capability record; it matters because a watcher service restarted for any reason would
start a sync.
Status: pass as a record.
Primary evidence: `crates/cli/src/lib.rs:45-50`
(`if !args.events.postpone { wx.send_event(Event::default(), Priority::Urgent).await?; }`),
the `--postpone` flag at `crates/cli/src/args/events.rs:170-179`, and the manual sentence at
`doc/watchexec.1.md:116-119`: "It's run when events pass filters and the debounce period (and
once at startup unless '--postpone' is given)".
Access date: 2026-11-25.
Outcome: one run at startup by default; `--postpone` suppresses it.

#### E22 What an error or a dead watcher costs

Candidate and version: watchexec 2.8.0.
Claim: filesystem errors do not stop the process; the process keeps running and prints them,
so a supervisor is what notices a broken watcher, not the exit status.
Gate: capability record; relevant because the design keeps a timer as the safety net.
Status: pass as a record.
Primary evidence: `crates/lib/src/watchexec.rs:232-249` logs and continues
(`if matches!(err, RuntimeError::Exit)` is the only upgrade to a critical error);
`crates/cli/src/config.rs:139-166` prints `[[Error (not fatal)]]` with a miette report;
`crates/lib/src/error/specialised.rs:9-14` shows that only watcher *creation* failure
(`FsWatcherError::Create`) is described as a critical error path.
Access date: 2026-11-25.
Outcome: non-fatal by design; exit status alone does not report a degraded watch.

### Security record

#### E23 Sockets, network access, privileged operations, telemetry and temporary files

Candidate and version: watchexec 2.8.0.
Claim: with the flags the design would use, the process opens no sockets, makes no network
connections, performs no privileged operation, and has no update check or telemetry; the
security-relevant surfaces are the command it runs, an opt-in socket mode, an opt-in interactive
stdin mode, and its log and temporary files.
Gate: security boundary per the skill's "Screening hard gates" and "Production source";
the vet report's overlays record no high-trust data passing through the watcher.
Status: pass, with the opt-in surfaces listed.
Primary evidence:

- No network client: `rg -n 'reqwest|ureq|curl::|isahc' crates` returns nothing;
  `rg -n -i 'telemetry|analytics|update check|update-check|phone home' crates doc README.md`
  returns nothing. The only hyper/tonic crates in `Cargo.lock` (for example `:1561-1578`,
  `:3442-3470`) come in through `console-subscriber`, which is
  `optional = true` and gated behind the `dev-console` feature
  (`crates/cli/Cargo.toml:65-67`, `:169-171`), and the release workflow builds with default
  features only (`--release --locked`, `.github/workflows/release-cli.yml:200-207`).
- Listeners are opt-in only: `crates/cli/src/state.rs:22-30` creates a socket set only when
  `args.command.socket` is non-empty, which is the `--socket` option
  (`crates/cli/src/args/command.rs:263-306`). With it, Unix binds the requested TCP or UDP
  address and passes file descriptors plus `LISTEN_FDS`
  (`crates/cli/src/socket/unix.rs:45-78`, `:31-43`), and Windows binds a loopback
  handoff listener at `127.0.0.1:0` (`crates/cli/src/socket/windows.rs:40-48`).
  A user-requested `HOST:PORT` can be a wildcard address, so this mode is a real listener when
  enabled, which is the user's choice, and `rg -l 'TcpListener|UdpSocket|UnixListener' crates`
  finds only `crates/cli/src/socket/windows.rs`.
- Privileged operations: none. There is no `setuid`, capability or namespace call.
  `crates/cli/src/main.rs:15-19` starts the `pid1` crate (default feature `pid1`,
  `crates/cli/Cargo.toml:161-164`), and `pid1-0.1.6/src/lib.rs:92-105` shows that
  `launch()` is a no-op unless the process id is 1; when it is 1, it relaunches the process as a
  child and reaps orphans. `pid1` 0.1.6 is pinned at `Cargo.lock:2470-2473`.
- stdin is only read for events when `--interactive`/`-I` or `--stdin-quit` is given
  (`crates/cli/src/config.rs:179`, `crates/cli/src/args/events.rs:140-160`); `-I` puts the
  terminal into raw mode, which the manual warns about (`doc/watchexec.1.md:495-503`).
- Temporary files are written only in the `--emit-events-to=file|json-file|stdio|json-stdio`
  modes, through `tempfile::NamedTempFile` in the system temp directory or
  `$WATCHEXEC_TMPDIR` (`crates/cli/src/state.rs:49-71`); the default mode creates none.
- Log files are written only with `--log-file`, to the path given
  (`crates/cli/src/args/logging.rs:56`, `:119-130`); the default logs to the terminal.
- The command is executed as the user, in a new process group on Unix (E13), and its
  environment receives `--env` values, the socket variables when `--socket` is used, and the
  event variables only in the emission modes (E14).

Access date: 2026-11-25.
Outcome: pass. The design's use (no `--socket`, no `-I`, no `--emit-events-to`, no `--log-file`)
touches none of the opt-in surfaces; the security boundary that remains is `systemctl --user
start` and whatever that unit runs.

## Maintenance audit

Method: the skill's "Maintenance audit" section, applied on 2026-11-25 against
`watchexec/watchexec`. Issue counts come from `gh api -X GET search/issues`; the twelve month
window starts 2025-11-25.

#### E24 Tracker volume and repository state

Candidate and version: repository at `main`, at tag `v2.8.0` for the source records.
Claim: the project is active, not archived, and its tracker sees steady traffic.
Gate: maintenance audit.
Status: pass.
Primary evidence:

- `gh api repos/watchexec/watchexec` returns
  `"archived":false`, `"disabled":false`, `"pushed_at":"2026-11-21T05:24:53Z"`,
  `"stars":7339`, `"forks":215`, `"open_issues":42` (that count includes pull requests),
  `"created_at":"2016-09-18T20:41:31Z"`.
- Search counts for work updated since 2025-11-25:
  issues 48, pull requests 179, open issues 29, open pull requests 13.
- Open issue labels available: `bug`, `improvement`, `new feature`, `question`,
  `need info`, `need repro`, `has fix`, `has workaround`, plus `OS: Linux`, `OS: macOS`,
  `OS: Windows` (`gh api repos/watchexec/watchexec/labels`).

Access date: 2026-11-25.
Outcome: active project with a small but real open backlog.

#### E25 The ten most recently updated issues

Candidate and version: repository tracker.
Claim: recent issues get maintainer attention; the ten most recently updated were all answered or
closed, with a single maintainer doing the answering.
Gate: maintenance audit ("if at most 20 exist, inspect all; otherwise inspect the 10 most
recently updated without cherry-picking"; maintainer comments counted separately from actions).
Status: pass.
Primary evidence: the ten most recently updated issues in the window, in order of update
(`gh api -X GET search/issues -f q='repo:watchexec/watchexec is:issue updated:>=2025-11-25'
-f sort=updated -f order=desc -f per_page=10`):

- #1185 (open, `bug`, macOS pathset can miss an event): opened 2026-11-24, one maintainer comment
  by `passcod` on 2026-11-25. The label was applied by the reporter, not by a maintainer
  (`gh api repos/.../issues/1185/timeline` shows `labeled Jac0b-Shi bug`).
- #1159 (closed): external report, maintainer reply, closed 2026-11-03.
- #1152 (open, macOS shutdown hang since 8.4.3): six comments including three by `passcod`
  and three by `injust`, the last on 2026-10-28.
- #863 (closed): old bug closed 2026-10-14 with a maintainer reply.
- #1131 (closed): maintainer reply then closure.
- #655 (open since 2023, PTY support): seventeen comments, maintainer replies through
  2026-10-06, the most recent comment on the issue.
- #874 (closed), #1112 (closed), #1113 (closed): each closed by `passcod` after replies;
  #1113's timeline shows `labeled passcod bug`, then `closed passcod`.
- #1107 (open, periodic-run feature request): maintainer reply the same day.

Maintainer comments, counted separately from actions: of the sampled issues, eleven comments are
by `passcod`, none by any other maintainer account; `injust` commented three times on #1152 and
is not otherwise identified as a maintainer by these queries. Actions (labels, closures,
references) that the timeline shows are all `passcod`'s, except the reporter self-label on #1185.
Access date: 2026-11-25.
Outcome: responsive on recent issues; no stale unanswered bug among the ten.

#### E26 The ten most recently updated pull requests

Candidate and version: repository tracker.
Claim: external pull requests are reviewed by the maintainer before merge, and the merge latency
for small external fixes is days.
Gate: maintenance audit ("measure maintainer-authored work, reviews, merge latency, unreviewed
external fixes").
Status: pass.
Primary evidence: the ten most recently updated pull requests
(`gh api -X GET search/issues -f q='repo:watchexec/watchexec is:pr updated:>=2025-11-25'
-f sort=updated -f order=desc -f per_page=10`), with reviews from
`gh api repos/watchexec/watchexec/pulls/<n>/reviews`:

- #1183 "Fix allowlist gitignores at any level", external (`noc7c9`), 3 files, +189/-8,
  opened 2026-11-19, approved by `passcod` 2026-11-21T04:57Z, merged by `passcod`
  2026-11-21T05:14Z, so about two days.
- #1180 "fix(cli): trim the extensions given to --exts", external (`HarpeLm`), one file,
  +59/-5, opened 2026-11-13, review comments exchanged 2026-11-17, approved 2026-11-21T05:05Z,
  merged 2026-11-21T05:19Z, so about eight days including a review round.
- #1146 "release", by `passcod`, open, the release-plz pull request, updated 2026-11-21.
- #1147 "feat: add --initial-events=create to report existing paths on first run", external
  (`Boulea7`), open, review state `CHANGES_REQUESTED` from `passcod`.
- #1182 "fix: make events on watched directory roots still match filters", external
  (`akesson`), +120/-60, opened 2026-11-18, approved 2026-11-21T05:01Z, merged
  same day, about three days.
- #1181 "fix(cli): reject a zero --poll interval", external (`HarpeLm`), open, approved by
  `passcod`, not merged.
- #1178, #1184, #1177, #1179: `dependabot[bot]` bumps of `taiki-e/install-action` and
  `softprops/action-gh-release`; the older ones were closed as superseded rather than merged.

Unreviewed external fixes: none among the sampled set; every external code pull request carries a
`passcod` review before merge. The two open external pull requests carry an approval and a
changes request respectively, so the queue is being used.
Access date: 2026-11-25.
Outcome: reviewed merges, merge latency from days to a week, no unreviewed external merge in the
sample.

#### E27 Release cadence, and how far `main` runs ahead of the tag

Candidate and version: repository releases.
Claim: releases are frequent, and the newest release is about seven weeks older than `main`, so a
user on 2.8.0 does not have the fixes merged after 2026-10-06.
Gate: maintenance audit ("inspect latest publish, releases, changelog links, and whether
maintenance merges reach a release").
Status: pass as a record; the lag is a fact, not a failure.
Primary evidence:

- `gh api 'repos/watchexec/watchexec/releases?per_page=30'` gives fifteen non-prerelease
  releases inside the twelve month window: v2.8.0 (2026-10-06), v2.7.4 (2026-10-02),
  v2.7.3 (2026-09-15), v2.7.2 (2026-09-06), v2.7.1 (2026-09-03), v2.7.0 (2026-08-24),
  v2.6.1 and v2.6.0 (2026-08-22), v2.5.1 (2026-03-30), v2.5.0 (2026-02-25),
  v2.4.3 and v2.4.2 (2026-02-24), v2.4.1 (2026-02-22), v2.4.0 (2026-02-22),
  v2.3.3 (2026-01-20). No prerelease flags are set on any of them.
- `gh api repos/watchexec/watchexec/compare/v2.8.0...main` reports
  `"status":"ahead"`, `"ahead_by":29`, so 29 commits are merged and unreleased as of the
  access date. Those include the reviewed external fixes above (#1183, #1182, #1180).
- Changelog route: `release-plz.toml:12-17` sets `changelog_update = false` for
  `watchexec-cli` and `git_release_enable = true`, so the CLI's changelog is the generated
  release body. The v2.8.0 body lists the fixes it contains (#1140, #1138, #1139), and
  `crates/lib/CHANGELOG.md` exists for the library crate.
- Crates.io publish for this version came through trusted publishing from the release commit
  (E02, E07).

Access date: 2026-11-25.
Outcome: monthly-ish cadence with bursts (four releases in February, four in late August and
September), and a seven-week gap between the tag and `main` at the access date.

#### E28 Maintainer concentration

Candidate and version: repository `main`.
Claim: one maintainer authors most commits.
Gate: maintenance audit ("measure maintainer concentration").
Status: pass.
Primary evidence: all commits since 2025-11-25, paginated to exhaustion
(`gh api 'repos/watchexec/watchexec/commits?since=2025-11-25T00:00:00Z&per_page=100&page=N'`
for pages one to three; page four returned zero, so the 283 commits below are the complete set):

- `passcod` 203 commits (about 72 per cent).
- `dependabot[bot]` 23 and `github-actions[bot]` 13.
- The remaining 44 commits are spread over 25 distinct accounts, the largest being 7
  (`VXNCXNX`), 4 (`meop`), 4 (`domenkozar`), 3 (`fzakaria`), and ones and twos.

Access date: 2026-11-25.
Outcome: one active maintainer (`passcod`), with bot automation and drive-by contributors; no
second regular committer in the window.

#### E29 Bug handling, security reporting and abandonment signals

Candidate and version: repository tracker.
Claim: recent bug reports get a first response within days, there is no security policy file, and
long-standing open feature requests are acknowledged rather than abandoned.
Gate: maintenance audit ("stale unanswered bugs, compatibility failures, security reports,
abandonment comments").
Status: pass as a record.
Primary evidence:

- Time to first maintainer reply on the sampled recent bugs: #1112 (reported 2026-09-08,
  replied the same day), #1113 (2026-09-10, replied 2026-09-11), #1185 (2026-11-24,
  replied 2026-11-25), #1159 (2026-10-25, replied 2026-10-27).
- Two open macOS reports touch the parts this design cares about: #1152 (shutdown hangs on
  macOS since 8.4.3 with keyboard events, open) and #1185 (a `pathset` change can miss an event
  on macOS, open as of the access date).
- Oldest open issues, least recently updated first: #101 (2019, one comment),
  #224 (2021, two comments), #234 (2022, zero comments, opened by `passcod`),
  #33 (2017, five comments), #456 (2022, five comments). These are unaddressed feature
  requests, not unanswered bug reports.
- Security: `gh api repos/watchexec/watchexec/contents/SECURITY.md` returns 404, so there is no
  security policy file at the repository root; there is no `security` label
  (`gh api repos/watchexec/watchexec/labels`) and no advisory-driven release in the window.
  Dependency updates arrive weekly per ecosystem from `.github/dependabot.yml:3-44`.

Access date: 2026-11-25.
Outcome: no abandonment signal; no formal security disclosure route was found.

## Summary: each hard constraint as pass, fail or open

The hard constraints are the vet report's "Hard constraints" list, plus the two round 4
requirements it folds into that list.

Open source with inspectable source: pass, by E01 (licence text at `LICENSE`) and
E02 (source at the tag, crates.io tree equal to the tag's).

Runs an arbitrary command, here `systemctl --user start rclone-bisync-garage.service`: pass, by
E13 (shell string through `sh -c`, with the `exec` rewrite applicable to this command).

Runs on Linux x86-64, macOS arm64 and Windows x64: pass, by E03, E04 and E05 for the artifacts and
their provenance, and E08, E09 and E10 for the watching backend on each system.

Watches every folder under the root recursively, including folders created after it starts:
pass, by E11 for Linux (create event, candidate work, per-directory `NonRecursive` watch, then a
scan that registers nested folders) and E12 for Windows (same code) and macOS (one recursive
FSEvents stream, reconciled logically).
Not measured at runtime in this run; the repository's own tests at
`crates/lib/tests/fs_worker_real.rs:484` and `:1045` corroborate the Linux mechanism.

After a change that lands while a sync runs, one more sync starts after that run ends: pass
natively, by E16 (`--on-busy-update=queue`); no wrapper is needed.

Never interrupts a running command; each run finishes: pass in queue mode, by E17, provided no
`--restart`, `--signal`, `--timeout` or `--once` is used.

Queues one more run of its command natively when events arrive while the command runs: pass, by
E16, with the correction that the pending run is a single coalescing slot rather than a FIFO.

Two further items the vet report asks to be recorded for every candidate, which the prompt also
asked for:

The quiet-period option and its default: `--debounce`, 50 ms, a fixed window from the first event
of a batch, not a trailing quiet period (E18).

Path exclusion: `--ignore`, `--ignore-file`, `--filter`, `--exts`, `--filter-prog`, plus a
default ignore list and ignore-file discovery; excluded directories are pruned before
registration (E19).

inotify watch-limit exhaustion: one non-fatal diagnostic naming `max_user_watches`, then further
registrations are skipped until a watch is freed or a new epoch starts; no automatic poll
fallback (E20).

Security-relevant surfaces: none active by default, no network, no telemetry, no update check,
sockets and interactive stdin opt-in only (E23).

## What this run did not do, and what stays open

- Nothing from the candidate was installed, built, tested or run, as instructed. Consequently no
  behaviour in this file is measured: the watch count for the 1454-folder tree, the queue timing
  window in E16, and every macOS and Windows claim are source readings.
- The candidate's own test suites were read, not executed
  (`crates/lib/tests/fs_worker_real.rs`, `crates/lib/src/sources/fs/recursor.rs` test module,
  `crates/cli/tests/`, `crates/cli/integration/*.sh`).
- The vet report's "External execution gate", "Finalist validation", scoring and recommendation
  remain untouched and open.
- The clone at `/home/user/temp/agent/watchexec-source-for-garage-watcher-vet-2026-11-25` and the
  scratch folder `/home/user/temp/agent/watcher-targeted-evidence-watchexec-scratch-2026-11-25`
  are deleted when this run ends, so every `path:line` above is meant to be read against a fresh
  clone made with the command in "Candidate identity and access".

## Where this file departs from the prompt or the vet report

- The vet report's ledger entry for watchexec says the paths that changed are provided "in
  environment variables or STDIN". That is the README feature bullet, but at 2.8.0 the default
  for `--emit-events-to` is `none` (`crates/cli/src/args/events.rs:303-308`), so paths reach
  neither the environment nor stdin unless asked for (E14).
- The vet report's ledger entry says watch-limit exhaustion "is not documented in the manual page
  read". That holds for the man page, but the source carries a dedicated diagnostic with a link
  to a project documentation page, and the behaviour is report-and-degrade (E20), which is more
  than "not documented".
- The vet report quotes `--on-busy-update=queue` as "run the command once again when the current
  run has finished". The code adds a detail the sentence hides: at most one run is pending, and
  several changes during one run collapse into that single run (E16).
- The release record on GitHub has `target_commitish: "main"` while the tag object points at
  `07e2b99` on the release branch; the tag ref, not that field, is what the workflow builds, and
  the crate publish came from the merge commit with an identical tree (identity section, E02,
  E07).
- The prompt asked for the license, provenance, platform backends, recursion, command execution,
  busy behaviour, debounce, exclusion, inotify limits and security. It did not ask for the
  startup run (E21) or the error-reporting behaviour (E22); both are recorded because they change
  what a service wrapper has to do.
