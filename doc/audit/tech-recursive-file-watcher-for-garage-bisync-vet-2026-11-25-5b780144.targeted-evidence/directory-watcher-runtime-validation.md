# Runtime validation of directory-watcher 3.2.1

Status:
 runtime validation of the vet report's only finalist,
 run by a Pi session on 2026-11-25;
 not reviewed,
 not approved by the owner,
 not final,
 not a score,
 not a ranking and not a recommendation.

## What this file is

The vet report
`doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`
carries directory-watcher 3.2.1 as its only hard-gate-confirmed candidate
and its only finalist (section "Owner decisions on the results").
Its source-reading record,
`directory-watcher.md` in the `targeted-evidence` folder beside this file,
records GW-01 to GW-24 and ran nothing of the candidate.
This file is the run the vet report's "Finalist validation" step needs:
the upstream CI-equivalent path,
the Linux consumer boundary in the shape
`doc/planning/garage-bisync-run-on-file-change.md`
settled in its "Owner answers, round 4" section,
and a record of the platforms this host cannot exercise.

The governing skill is `.claude/skills/choosing-technology/SKILL.md`;
its "External execution gate" and "Finalist validation" sections decide the shape of this file.

Everything below was measured on this host,
in throwaway containers,
on 2026-11-25 between 18:54 and the time the last section records.
Nothing was measured on `~/Plain`,
on the owner's Mac,
or on any Windows machine.

## Candidate identity, revision and artifacts

- Candidate: directory-watcher 3.2.1, `seungyeop-lee/directory-watcher`.
- Tag: annotated tag `v3.2.1`, tag object `5622bdfe6d4ddad01cf21f3d63e605266cdae0b3`,
  pointing at commit `7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- Clone:
  `gh repo clone seungyeop-lee/directory-watcher <scratch>/source -- --depth 1 --branch v3.2.1`,
  exit 0, `git rev-parse HEAD` = `7286f534dbeb3854607b3a48875c4cf0bbb5e672`,
  which is the commit the source-reading record read.
- Release assets, downloaded from
  `https://github.com/seungyeop-lee/directory-watcher/releases/download/v3.2.1/`:
  - `checksums.txt`, 824 bytes, SHA-256 `4e193f36901ee872417fd5cbff8e708ca10e3d27b808b2f45cf95ff84e82cd24`;
  - `directory-watcher_Linux_x86_64.tar.gz`, 2409961 bytes,
    SHA-256 `eb07d000d21069f528511a16d56ee529542f70daa1da527016cf74f8d9ba437a`,
    which is the line the checksum file holds for that name;
  - the GitHub API reports the same digest for both assets
    (`sha256:4e193f36...` for `checksums.txt`, `sha256:eb07d000...` for the archive),
    so the download matches the release record and the release's own checksum file.
- Binary inside the archive: `directory-watcher`, 5869728 bytes,
  SHA-256 `9b1fd474ae39e3dcd50a5a125335ffaa6d779c1495df05f82f97bee8a726fcdd`.
- Container images, pulled by digest and used by every tree below:
  - `docker.io/library/golang:1.27`,
    digest `sha256:e65b9442d27a8cbb2fde9128832d75154664528d8ab6b5b6e951800d613cc12a`,
    image ID `7bbeef4f5aa5dc3193e87e61e6c27d2b13b8dc589fc9ff97958a4c5a0e429edb`;
  - `docker.io/cachyos/cachyos:latest`,
    digest `sha256:96535268a984b889df0623a37b893e226bf615c918e3ad6f836b8f67373e7b3e`,
    image ID `fdf6cfa512cc96b76017440919f639091b50140bb4197ab7b830f36dacb4a4f3`.
- Scratch folder:
  `/home/user/temp/agent/watcher-finalist-validation-directory-watcher-scratch-2026-11-25`
  (rule `ATH`), created with `mkdir --parents` and `chmod 700`.

## Isolation defaults for every command tree

Unless a tree's own manifest section names a deviation:

- `--cpus 2 --memory 2g --pids-limit 512 --blkio-weight 10 --rm`;
- no ambient credentials of any kind;
- no mount of the real home directory,
  of `~/Plain`,
  or of the repository `/home/user/Monochromatic`;
- the candidate source is mounted from the scratch clone, never from the repository;
- a private bind-mounted scratch folder inside the scratch folder holds every input and output;
- `--network=none` for every container that runs code,
  after the separately declared fetch phase in tree 1;
- the container boundary, not an external `timeout` command,
  is the wall-clock ceiling (rule `TMO`);
- containers that run agents' own commands are removed when the tree ends.

Deviation from the skill's default, recorded as it asks:
the candidate source is mounted read-write in tree 2,
because the CI job's own smoke step creates `test/dir1` and `test/dir2` in the clone
(`Makefile` target `example-watch-dirs`)
and the Go build writes its module and build caches.
The mount is the disposable clone in the scratch folder, not any repository.

## Execution manifest, tree 1: fetch and inspection

Purpose: obtain the pinned source, the release artifact and its checksum file,
and the images, before any candidate code runs.

Top-level commands, on the host:

- `gh repo clone seungyeop-lee/directory-watcher <scratch>/source -- --depth 1 --branch v3.2.1`
  (clone origin `https://github.com/seungyeop-lee/directory-watcher`, HTTPS);
- `curl -sSL --output <scratch>/checksums.txt` and
  `curl -sSL --output <scratch>/directory-watcher_Linux_x86_64.tar.gz`,
  both from the release download URL above;
- `sha256sum` over both files, compared with each other and with
  `gh api repos/seungyeop-lee/directory-watcher/releases/tags/v3.2.1`;
- `podman pull docker.io/library/golang:1.27` and
  `podman pull docker.io/cachyos/cachyos:latest`;
- `podman image inspect` on both, to record digest and image ID.

Statically discovered commands reached by tree 1:
`gh` (HTTPS to `api.github.com` and `codeload.github.com`),
the clone's own `git`,
`curl`,
`sha256sum`,
`tar --extract --gzip`,
`podman pull` (registry `docker.io`, `registry-1.docker.io` and `production.cloudflare.docker.com`),
`podman image inspect`.

Files inspected to establish the trees below, all inside the clone:

- `.github/workflows/ci.yml` (the `test` job and its three-OS matrix);
- `.github/workflows/goreleaser.yml` (only to confirm the release's build mapping, already GW-03);
- `Makefile` (every target, to see which ones CI reaches);
- `scripts/smoke-example.sh` (what `make smoke-example` runs);
- `config.example.yml` (the configs the smoke step and the example hooks use);
- `go.mod`, `go.sum` (module set and the `go 1.27` line);
- `e2e/harness/harness.go`, `e2e/harness/signal_unix.go`, `e2e/hookrec/main.go`,
  `e2e/scenarios/*.go` (the subprocesses the test binaries start);
- `e2e/worktree/worktree.go` (it runs `git`, reached only by the darwin-only `e2e/linux` and `e2e/windows` commands);
- `docs/guide/configuration.md`, `docs/guide/events.md`, `docs/guide/running.md`,
  `config.example.yml` and `internal/config/converter/*`
  (the config keys tree 3 writes, and the event and `DW_*` semantics it reads);
- `promo/scenes/`, to confirm `go build ./...` reaches no graphics or network source.

Expected reads: GitHub API and release download over HTTPS, `docker.io` registry over HTTPS.
Expected writes: only inside the scratch folder.
Expected outputs: the four identities in "Candidate identity, revision and artifacts".
Success condition: clone HEAD equals the tag's commit,
archive SHA-256 equals the checksum file line and the API digest.
Failure condition: any digest mismatch, or a clone whose HEAD is another commit.
Stop condition: a mismatch stops the run, because the rest would validate a different artifact.

## Execution manifest, tree 2: upstream CI-equivalent

Purpose: run the Linux steps of the `test` job of `.github/workflows/ci.yml`
at tag `v3.2.1`, without network, in an image whose Go matches `go.mod`'s `go 1.27` line.

Top-level commands:

- fetch phase, network allowed:

```shell
podman run --rm --cpus 2 --memory 2g --pids-limit 512 --blkio-weight 10 \
  --volume <scratch>/source:/work:rw --volume dw-validation-gomodcache:/gomod \
  --env GOMODCACHE=/gomod --env GOFLAGS=-mod=mod --workdir /work \
  golang:1.27 go mod download
```

- test phase, `--network=none`, one container per step so each exit status and duration is its own:
  - `go version` (image probe, recorded as the Go the run used);
  - `go build ./...`;
  - `go vet ./...`;
  - `DW_E2E_DIAGNOSTICS=/tmp/dw-e2e go test -count=1 -race -v ./...`;
  - `make smoke-example`;
- each step mounts `<scratch>/source:/work:rw` and the module volume at `/gomod`,
  sets `GOMODCACHE=/gomod`, `GOPROXY=off` and `GOFLAGS=-mod=mod`,
  and runs `--workdir /work`;
- `--init` joins every test step after amendment 4:
  a container whose PID 1 is the test binary leaves orphaned hook processes as zombies,
  which a virtual machine's own PID 1 reaps, and which the harness's liveness check counts as alive.

Deviation from the CI file, recorded as the skill asks:
`go test` gains `-v`, which changes no test selection and no test code,
and is what records the test names this file's green-run claim rests on
(rule `TRV`).
CI's `actions/checkout@v7` and `actions/setup-go@v7` are replaced by the clone and the image;
CI's `echo "DW_E2E_DIAGNOSTICS=$RUNNER_TEMP/dw-e2e" >> "$GITHUB_ENV"` is replaced by the environment variable above.
The Windows-only steps of the job do not run on Linux and are recorded in "Platforms not run".

Statically discovered commands the tree reaches:

- `go build`, `go vet`, `go test`, and the `go` toolchain's own linker and compiler;
- `make`, which expands to `mkdir -p test/dir1 test/dir2` and
  `sh scripts/smoke-example.sh test/dir1 test/dir2`;
- `scripts/smoke-example.sh`: `mktemp -d`, `go build -o <tmp>/directory-watcher .`,
  runs that binary with `-c config.example.yml -l INFO`,
  writes `smoke-example.txt` in both watched folders,
  `kill -TERM` then `kill -KILL` on failure, `rm -f`, `rm -rf`, `sleep 0.1`, `awk`;
- the test binaries: `e2e/harness/harness.go` runs `go build` three times
  (the watcher, a watcher built with the `inject` tag, and `e2e/hookrec`),
  runs each once as a warm-up, and then starts the watcher per scenario;
- the hook commands the scenario tests write into their configs,
  which run through `sh -c` and name `hookrec`, `printf`, `echo`, `sh`, `sleep`,
  and the scenario's own files;
- no `git` call from any Linux test: `e2e/worktree` is reached only by `e2e/linux` and
  `e2e/windows`, and the latter is `//go:build darwin`;
- no network call from the shipped program (GW-22), so `--network=none` costs nothing.

Expected reads: the clone, the module volume.
Expected writes: `/work` (`test/dir1`, `test/dir2`, no tracked file),
`/gomod`, the container's Go build cache, `/tmp/dw-e2e`, and each test's own temp folder.
Expected outputs: one exit status and one duration per step, and the test names `-v` prints.
Success condition: every step exits 0 and the printed test names are present.
Failure condition: any non-zero step, or a step that exits 0 without printing its suite.
Stop condition: a step whose output shows an undeclared command, network attempt
(it would fail under `--network=none`), or native boundary stops the tree;
the manifest is amended, the path inspected, and only then the tree continues.

## Execution manifest, tree 3: consumer boundary under systemd

Purpose: run the shipped release binary on Linux
in the design's shape: a hook that starts a trigger oneshot,
which starts a sync oneshot ordered after itself.

Container:

- image `docker.io/cachyos/cachyos:latest` at digest `sha256:96535268...`,
  started with `--systemd=always` and `/usr/lib/systemd/systemd` as its command,
  the same limits as the defaults above,
  `--network=none` from the start,
  and `--volume <scratch>/container-share:/srv/share:rw` as its only host mount.
- system units stand in for user units, as this run's instruction fixes:
  the hook runs `systemctl start` (system scope) of the trigger,
  and the trigger's step is `systemctl start` of the sync.
  Nothing here reads or writes the host's units.

Inputs written into the container, from the share mount:

- `/usr/local/bin/directory-watcher`: the verified release binary,
  re-checked inside the container with `sha256sum` against the release checksum file.
- `/srv/Plain`: the tree under test, holding nested folders and names with
  Japanese characters, Chinese characters, spaces, parentheses, brackets,
  single quotes, double quotes and a fullwidth colon.
- `/srv/dw-config.yml`: the watcher config, from
  `docs/guide/configuration.md` and `config.example.yml`:
  `shell: sh`;
  one target `path: '/srv/Plain'` with `option.watch.subDir: true`,
  `option.watch.exclude: ['**/*.partial']`,
  `option.hookBatch.waitMillisecond: 2000`;
  `onStartWatch` and `onFinishWatch` log a marker line;
  `onChange` runs three commands in this order:
  `/usr/local/bin/dw-hook-begin.sh` (logs the eight `DW_*` values and a begin marker),
  `systemctl start garage-bisync-trigger-standin.service`,
  `/usr/local/bin/dw-hook-end.sh` (logs an end marker).
- `/etc/systemd/system/garage-bisync-standin.service`:
  `Type=oneshot`, two `ExecStart=` lines,
  each logging `SYNC-START pass=<n>` with `date +%s.%N`,
  sleeping 8 seconds,
  and logging `SYNC-END pass=<n>`, so one run is two passes like the settle pass the design settled;
- `/etc/systemd/system/garage-bisync-trigger-standin.service`:
  `After=garage-bisync-standin.service`, `Type=oneshot`,
  `ExecStart=/usr/bin/systemctl start garage-bisync-standin.service`.
- `/var/log/dw-measure.log`: the one log every marker and every `DW_*` dump goes to,
  appended with `date +%s.%N` timestamps.

Statically discovered commands the tree reaches:
`systemctl` (start, status, show, stop, list-units) talking to the container's own systemd over
`/run/systemd/private`;
`systemd` itself starting two oneshots;
`/bin/sh -c` for each `ExecStart=` and for each hook command;
`date`, `sleep`, `sha256sum`, `install`, `mkdir`, `find`, `touch`, `chmod`, `mv`, `rm`, `printf`, `cat`, `tail`;
the watcher's own inotify watch setup (one fsnotify instance, one watch per folder).
No network endpoint: the container runs with `--network=none` from start to finish.

Expected reads: `/srv/Plain` inode events, `/etc/systemd/system/*`, `/var/log/dw-measure.log`.
Expected writes: `/srv/Plain` by the measurement steps, `/var/log/dw-measure.log`,
systemd's own journal and state in the container.
Expected outputs: one timestamped log, plus systemd's own unit state history.
Success condition: every step's expected line sequence appears inside its own deadline,
and the watcher's own log holds no `[ERROR]` line.
Failure condition: a step whose lines do not appear,
a watcher error, or a sync that does not reach its second pass.
Stop condition: an undeclared process, socket, mount or file write stops the tree,
the manifest is amended and the path inspected before anything else runs.

Measurement steps inside that container, each started from the host with `podman exec`
and each bounded by its own deadline:

1. one edit gives exactly one trigger run and one sync;
2. an edit during a running sync gives exactly one more sync after it,
   and the running sync is never interrupted;
3. a sync started directly, the timer's path, plus an edit during it,
   gives one fresh sync after it;
4. a burst of 200 small files written over about 3 seconds:
   how many hook runs and how many syncs;
5. a new folder created with files inside, then an edit inside it:
   the change of each of the two starts a run;
6. a save done as "write a temporary file, rename it over the original": how many runs;
7. a metadata-only change, `touch` then `chmod` of an existing file: does it start a run;
8. a `.partial` file created, written, and renamed to its final name: what starts a run;
9. every odd file name: no error in the watcher log, and what the `DW_*` variables hold for it;
10. SIGTERM to the watcher while a hook runs: what happens to the hook process and to the sync.

Ceilings for tree 3:
2 CPUs, 2 GiB, 512 processes, block weight 10 (the defaults above);
whole-container life under 60 minutes;
each measurement step's wait has its own deadline, at most 90 seconds beyond the
expected 16-second sync;
logs expected to stay under 1 MiB.
The file-descriptor ceiling is the image default, measured inside the container and recorded,
because the design's real watch count (1454 folders) is far above this fixture's.

Cleanup for tree 3: the container is removed,
`podman ps --all` is checked for it,
and the share folder is deleted with the rest of the scratch folder.

## Ceilings, success, failure, cleanup and stop conditions shared by every tree

- Wall clock: tree 2's per-step ceilings are 10 minutes for `go version` and `go build`,
  10 minutes for `go vet`, 35 minutes for `go test`,
  and 5 minutes for `make smoke-example`.
  CI's own `timeout-minutes: 20` covers that whole job on a two-core runner,
  so a test step above 20 minutes is recorded as a parity deviation rather than truncated.
- Memory, CPU, process count and block weight: the isolation defaults.
- Disk: expected under 2 GiB in total (module volume plus images already pulled);
  no separate disk ceiling is imposed and the measured use is recorded.
- File descriptors: the image default, measured and recorded;
  no `--ulimit` is passed, which is a deviation from a strict reading of the gate
  and is recorded here as such.
- Credentials: none are passed, mounted or exported into a container.
- Environment: only the variables each manifest section names.
- Network: tree 1 only, plus the two image pulls; trees 2 and 3 run with `--network=none`
  except the single declared module-fetch step of tree 2.
- Success: every manifest section's success condition.
- Failure: every manifest section's failure condition.
- Cleanup: every container this run starts is removed,
  `podman ps --all` is read to prove it,
  the module volume is removed,
  and the scratch folder is deleted except its `keep/` subfolder.
- Stop: every manifest section's stop condition, plus the run's own rules
  (no synthetic host load, no host service-manager verb, no read of another process's environment).

## Amendment log

Append-only, in the order the deviations were found.
Each entry names the undeclared path, what was inspected, and the decision.

### Amendment 1: the test binary builds three helper programs

Found by reading, not by running, before tree 2 started:
`e2e/harness/harness.go` runs `go build` three times at test startup
(the watcher, the watcher with the `inject` build tag, and `e2e/hookrec`),
and every scenario runs the built watcher and its hooks.
Inspected: `harness.go`, `hookrec/main.go`, `scenarios/*_test.go`.
Decision: the container already holds the Go toolchain, the module cache and a writable `/tmp`,
so the path is bounded by the same ceilings; continue.

### Amendment 2: `go test -v` output is buffered per package

Found while the run was going:
the run's log shows a package's `-v` output only when that package finishes,
so the file's size says nothing about progress.
Inspected: the run's own log and `ps` inside the container (a `scenarios.test` process alive and busy).
Decision: none needed; the run was left to finish and its elapsed time is what it reports.

### Amendment 3: test diagnostics were not mounted out of the container

Found after tree 2 ended:
`DW_E2E_DIAGNOSTICS=/tmp/dw-e2e` sat inside a container started with `--rm`,
so the failure diagnostics the harness retained died with it.
Decision: the diagnostic rerun mounts the diagnostics folder from the scratch folder
and points `DW_E2E_DIAGNOSTICS` there,
so any retained diagnostics survive; recorded with the rerun.

### Amendment 4: the test containers had no init process

Found while diagnosing tree 2's nine failures.
The container's PID 1 was the `go test` process,
so the hook processes the harness orphans became zombies that it reaps never,
and the harness's liveness check (`kill(pid, 0)`) counts a zombie as alive.
Inspected: `e2e/harness/signal_unix.go:33` and `e2e/harness/process_records.go:167-195`,
the retained `cleanup.log`, and the project's own `e2e/linux/main.go:49`, which passes `--init`.
Decision: `--init` joins every test-phase container (declared in the tree 2 manifest above),
which restores the reaping a virtual machine's own PID 1 provides,
and the corrected run is the CI-equivalent this file reports.

### Amendment 5: the serial diagnostic was stopped

Started: `go test -race -v -parallel 1 ./e2e/scenarios` without `--init`,
to test whether intra-package parallelism caused the failures.
Stopped after four results, when `--init` already explained the pattern (amendment 4),
and its container was removed.
Recorded because a started experiment belongs in the record even when it is abandoned.

## Results, tree 1: fetch and inspection

All of it ran as the manifest describes.

- Clone: exit 0, `git rev-parse HEAD` = `7286f534dbeb3854607b3a48875c4cf0bbb5e672`.
- `checksums.txt`: 824 bytes, SHA-256 `4e193f36901ee872417fd5cbff8e708ca10e3d27b808b2f45cf95ff84e82cd24`,
  equal to the API digest for that asset.
- `directory-watcher_Linux_x86_64.tar.gz`: 2409961 bytes,
  SHA-256 `eb07d000d21069f528511a16d56ee529542f70daa1da527016cf74f8d9ba437a`,
  equal to the line in `checksums.txt` and to the API digest.
- Extracted binary: 5869728 bytes,
  SHA-256 `9b1fd474ae39e3dcd50a5a125335ffaa6d779c1495df05f82f97bee8a726fcdd`.
- Images pulled by tag and recorded by digest, as in "Candidate identity, revision and artifacts".
- The image's own Go, read inside the container before any build: `go version go1.27.4 linux/amd64`,
  the same minor version as `go.mod`'s `go 1.27` line,
  so no toolchain download was needed and `--network=none` could not hide one.
- The `cachyos/cachyos:latest` image was probed without systemd first:
  `/sbin/init` is a link to `../lib/systemd/systemd`, `/usr/bin/systemctl` exists,
  `systemd 262 (262.2-1-arch)`, and a shell's `nofile` limit is 1048576.

## Results, tree 2: upstream CI-equivalent

Image `docker.io/library/golang:1.27`, Go 1.27.4, `--network=none` for every step
(only `go mod download` had network), 2 CPUs, 2 GiB, 512 processes, block weight 10.

Per-step results, in the order the CI file runs them:

- `go mod download` (network phase): exit 0, no output.
- `go version`: exit 0, 22.5 s including container start, `go version go1.27.4 linux/amd64`.
- `go build ./...`: exit 0, 14.6 s, no output.
- `go vet ./...`: exit 0, 40.4 s, no output.
- `go test -count=1 -race -v ./...`: exit 1, 326.1 s, without `--init` (see amendments 4 and 5
  and the two sections after this one).
- `make smoke-example`: exit 0, 29.6 s, `smoke-example: PASS`.
  CI runs it after `go test`; this run made it after the failing test step and before the corrected one.

The test run's own numbers:

- 663 `=== RUN` lines, 231 top-level tests passed, 402 subtests passed, 2 skipped, 9 top-level tests failed.
- Every package except two passed:
  the root package, `cmd`, `e2e/harness`, `e2e/hookrec`, `e2e/windows/guestrun`, `e2e/worktree`,
  `internal/app/domain`, `internal/app/service`, `internal/config/yaml`, `internal/helper`,
  `internal/process`, `scripts/releasenotes`,
  with no test files under `e2e/linux`, `internal/config` and `internal/config/converter`.
- `e2e/scenarios` failed after 233.2 s, and `internal/app/domain/command` after 19.2 s.

The nine failures, each with the line that fired first in that test:

- `TestProcessLifecycle` (16.08 s), `e2e/scenarios`:
  `process_test.go:60 timeout: 2 ready process trees`,
  then `harness.go:300 process cleanup failed; inspect cleanup.log`.
- `TestConcurrentRunsShutdown` (16.13 s), `e2e/scenarios`:
  `process_test.go:158 timeout: watcher exit`,
  then the same `process cleanup failed` line.
- `TestStartHookShutdownModes`:
  `process_test.go:117 timeout: watcher exit` for `interrupt=none`,
  and `process_test.go:106 timeout: replacement tree ready` for `interrupt=oldest`.
- `TestOverlappingTargets`:
  `overlapping_targets_test.go:165 timeout: watcher exit` in
  `slow_parent_hook_does_not_block_child`.
- `TestRunInterrupt`:
  `process_test.go:180 timeout: second a.txt tree ready`,
  `process_test.go:220 timeout: second b.txt tree ready`,
  and `process_test.go:272 timeout: watcher exit`.
- `TestEventLossNotificationCLI`:
  `error_notification_test.go:174 timeout: all slow hook processes disappear`
  in its `missing-command`, `success-tree`, `timeout`, `failure` and `absent` cases.
- `TestInjectedLossNotificationCLI`:
  `error_notification_test.go:286 timeout: injected fatal error exits the CLI`
  in three cases, `error_notification_test.go:292` in one,
  and `timeout: a hook process tree running uncanceled with records unchanged for 200ms` in two.
- `TestFastInflowSlowHook`:
  `reliability_test.go:65 timeout: received inflow completes and settles`
  in `deliver=each/interrupt=oldest`,
  while its sibling case logged
  `inflow settled after 8.077709283s: received=92 records=184`
  for an inflow its own instrumentation recorded as `duration=323.934991ms` of writes.
- `TestCanceledHookPrintsAllOutputUntilPipeCloses` (15.07 s) in `internal/app/domain/command`:
  `cmd_manager_test.go:124 canceled hook did not finish`, a 15-second deadline.

None of the nine reported a wrong hook sequence, a wrong event variable, a missing line
or a wrong exit status.
Every one is a waited-for condition that did not clear in time,
and the next section measured which condition and why.

Host pressure while the run went on, measured by reading `/proc/loadavg` on this 3-CPU machine:
`45.67 37.64 33.71` during the test step, and between 47 and 73 for the rest of the hour.
The load was not this run's:
`btrfs check --readonly --mode=lowmem` (in its fourth hour, about 86 % of a CPU),
several `rustc` compilations (each about 80 % of a CPU),
and two other sessions' containers.
That pressure is recorded as context, not as the cause of those nine failures:
the section after this one measured what the cause was.

### Diagnosing the two failing packages

The first run kept no test diagnostics (amendment 3), so the diagnosis started by reproducing
the failure with the diagnostics folder mounted out of the container.

1. The eight failing scenario tests alone, same image, same flags, no `--init`:
   exit 1 after 321.0 seconds, the same eight failed, load `50.86` falling to `59.28`.
   27 diagnostic folders survived, because `DW_E2E_DIAGNOSTICS` pointed at the mount.
2. One retained `cleanup.log` reads `pid=3475 alive=false`, `pid=3482 alive=true`, `pid=3489 alive=false`:
   one recorded hook process still counted as alive after the harness had killed its group
   and waited its 3 seconds.
   That is what raised `process cleanup failed` and, in the notification tests,
   `timeout: all slow hook processes disappear`.
3. How liveness is decided: `e2e/harness/signal_unix.go:33` is
   `func (*processObserver) alive(pid int) bool { return syscall.Kill(pid, 0) == nil }`.
   A zombie process keeps its process-table entry, so `kill(pid, 0)` succeeds for it and it counts as alive.
4. Why there were zombies: the container ran without an init process,
   so the PID 1 of the container was the `go test` process itself,
   and the orphaned hook processes the harness kills are reparented to it.
   A Go program reaps only its own children, so those orphans stayed as zombies for the container's life.
   GitHub's `ubuntu-latest` runner is a virtual machine whose PID 1 is systemd, which reaps them at once,
   and the project's own container runner shows the same need:
   `e2e/linux/main.go:49` builds its container with `docker create --init`.
   The deviation was mine, not the candidate's.
5. The decisive check: the same eight tests, same image, same flags, with `--init` added:
   exit 0 after 77.8 seconds, eight of eight passed, load `72.54` falling to `57.56`.
   So load was not what decided them either: they passed under a heavier load than the failing run.
6. A stopped experiment, recorded because it was started: with `-parallel 1` and no `--init`,
   `TestConcurrentRunsShutdown` failed again at 16.21 seconds.
   That run was stopped after four results and its container removed,
   because the constant ~16 seconds it and the other two lifecycle tests reported
   (10-second stop wait plus two 3-second waits) fits a liveness check that never clears,
   not a wall-clock deadline that a slower host pushes past.

### The corrected CI-equivalent run

The CI-equivalent path, rerun with `--init` and nothing else changed:
`go test -count=1 -race -v ./...`, `--network=none`, 2 CPUs, 2 GiB, 512 processes, block weight 10.

- Exit status 0, elapsed 162.7 seconds, load `60.18` to `54.22`.
- 663 `=== RUN` lines, 240 top-level tests passed, 421 subtests passed, 0 failed, 2 skipped.
- Every package green:
  `<mod>` (1.386 s), `<mod>/cmd` (1.333 s), `<mod>/e2e/harness` (1.982 s),
  `<mod>/e2e/hookrec` (24.399 s), `<mod>/e2e/scenarios` (56.824 s),
  `<mod>/e2e/windows/guestrun` (1.188 s), `<mod>/e2e/worktree` (1.038 s),
  `<mod>/internal/app/domain` (1.028 s), `<mod>/internal/app/domain/command` (9.480 s),
  `<mod>/internal/app/service` (25.358 s), `<mod>/internal/config/yaml` (1.295 s),
  `<mod>/internal/helper` (1.073 s), `<mod>/internal/process` (41.179 s),
  `<mod>/scripts/releasenotes` (1.108 s),
  with no test files under `<mod>/e2e/linux`, `<mod>/internal/config`
  and `<mod>/internal/config/converter`.
- The two skips are kqueue-only tests, skipped on Linux by their own guard:
  `TestFolderRecreatedBeforeItsRemoveArrives`
  (`runner_watch_target_test.go:188`:
  "only fsnotify's kqueue backend drops the Create of a folder recreated at the same path")
  and `TestTargetRemovedAgainWhileReAddedDoesNotStopRunner`
  (`runner_watch_target_test.go:232`:
  "only on kqueue, handleEvent re-adds a watch target recreated before its Remove arrives").
- The tests that carry this design's claims all appear as passing top-level tests in that output,
  each in the package named above:
  `e2e/scenarios`: `TestGoldenPath`, `TestOverlappingTargets`, `TestFastInflowSlowHook`,
  `TestRunInterrupt`, `TestStartHookShutdownModes`, `TestProcessLifecycle`,
  `TestConcurrentRunsShutdown`, `TestEventLossNotificationCLI`,
  `TestInjectedLossNotificationCLI`, `TestFolderEvents`, `TestWatchOptions`, `TestBatchingOptions`,
  `TestQueueWarningCLI`, `TestHookWorkingDirectory`;
  `internal/app/service`: the runner, watch-target and queue suites;
  `internal/app/domain/command`: `TestCanceledHookPrintsAllOutputUntilPipeCloses`,
  `TestHookEndsSoonAfterCommandWhenLeftoverProcessHoldsOutput`,
  `TestHookOutputIsCompleteWhenRunReturns`.
  The whole list of 240 names is in the kept log,
  `keep/tree2-initfull.log`, whose line count of `=== RUN` lines is the 663 quoted here.
- `make smoke-example` passed separately, exit 0 in 29.6 s, before this corrected run:
  it printed `smoke-example: ok` for both watch starts, the `dir1 onChange`,
  the eight event values of `dir2 onChange`,
  the `dir:` hook's working directory, `exit status 0 after SIGTERM`,
  and both `onFinishWatch` hooks, then `smoke-example: PASS`.
  Its step order in CI is after `go test`; this run made it before the corrected `go test`,
  so the order above is the run's, not CI's.

### Suites of the repository this run did not run

Each line names the command, its purpose, and why it cannot change the claims above.

- `make e2e-linux` (`go run ./e2e/linux`): it creates a `golang:1.27-bookworm` container
  and runs `go test -count=1 -v ./e2e/scenarios` inside it.
  That is the same test code this run executed in a `golang:1.27` container
  (the `e2e/scenarios` package above), so the only difference is a nested docker daemon
  and the init process the project itself passes; the claims it covers are already measured.
- `make e2e-windows-all` / `make e2e-windows`: `e2e/windows/main.go` is `//go:build darwin`,
  drives a UTM VM through `utmctl` and `osascript`, and cannot build on this host.
  Its own tests (`e2e/windows/guestrun`) did run and passed.
- `make test-fast` (`go test -short ./...`): a strict subset;
  `-short` skips the scenario tests, which the corrected run executed in full.
- `make release-test`, `make release-test-snapshot`, `make rund-docker`:
  release and demo drivers that need goreleaser, tap credentials or a docker daemon;
  they build or run the same binary the release already publishes and this run exercised,
  and none of them asserts behaviour the suites above do not.
- `make run`, `runi`, `rund`: developer conveniences that start the watcher on `config.example.yml`;
  `make smoke-example` runs the same config with an assertion on its output and passed.


## Results, tree 3: the consumer boundary under systemd

The container, as the manifest describes it:

- `docker.io/cachyos/cachyos:latest` at digest `sha256:96535268...`,
  `--systemd=always` with `/usr/lib/systemd/systemd` as PID 1,
  `--network=none` from start to finish,
  2 CPUs, 2 GiB, 512 processes, block weight 10;
  `systemd 262 (262.2-1-arch)`, and a shell's `nofile` limit of 1048576.
- `is-system-running` reported `degraded` for the container's whole life,
  because `dbus-broker.service`, `systemd-resolved.service`, `systemd-homed.service`
  and `systemd-firstboot.service` fail in a container with no bus and no network.
  Nothing this run uses goes through those units:
  every `systemctl` call reached systemd over `/run/systemd/private`.
- The release binary at `/usr/local/bin/directory-watcher` hashed inside the container to
  `9b1fd474ae39e3dcd50a5a125335ffaa6d779c1495df05f82f97bee8a726fcdd`,
  the value the host download produced, and its archive line in the release checksum file is
  `eb07d000d21069f528511a16d56ee529542f70daa1da527016cf74f8d9ba437a`.
- The watcher's own startup dump of the resolved config, so the run's settings are the ones here:
  `FileEvent` create, update, delete; `DirEvent` create, delete (no update);
  `DirBatch` 100 ms and 1000 ms; `HookBatch` 2000 ms and 2000 ms; `Deliver` last (the default);
  `Queue.Size` 2048; `Run.Concurrency` 1; `Run.Interrupt` none (the default).
  So the measured behaviour is the shipped defaults plus the design's three settings:
  recursive, `**/*.partial` excluded, 2000 ms batch wait.
- At startup the watcher added watches for eight folders:
  `/srv/Plain` plus `日本語 フォルダ 1`, `中文目录 二`, `落合's フォルダ`, and the four folders of
  `深い/階層/の/フォルダ`.
- The fixture tree holds 19 entries:
  eight folders and eleven files. The eleven files are the odd names step 9 lists,
  plus the ones the later steps create.
- Watcher started 2026-11-25T19:09:16-05:00; the ten steps ran from 19:09:27 to 19:21:07.
  Steps 4 to 7 overlapped a tree 2 diagnostic rerun, so a second container of this run's own
  was using the same two-CPU share during them;
  every sync in those steps still took 16.05 to 16.16 seconds, the same as the rest.
- Deviations from the design's real hook, both declared in the manifest:
  the hook's first command dumps the eight `DW_*` values, and its third writes an end marker.
  The middle command is the one the design runs, `systemctl start` of the trigger.
  The three commands run in order inside one hook run, so `HOOK-BEGIN` to `HOOK-END` is the
  length of one run, and the trigger call's blocking is inside it.
- Across all ten steps: 24 hook runs, 25 sync runs,
  25 first passes started and ended, 25 second passes started and ended,
  so every sync that started ran both of its passes.
  23 hook runs reached their end line;
  the missing one is step 10's, whose process group the watcher signalled on SIGTERM.
- One sync run takes 16.05 to 16.16 seconds: two 8-second passes plus systemd's own start and stop.
- One edit reaches the hook 2.03 to 2.09 seconds after the write: the 2000 ms batch wait plus dispatch.
- The watcher's log holds 1065 lines and no `[ERROR]` and no `[WARN]` line, over the whole ten steps.

### Step 1: one edit, one trigger, one sync

Edit at 19:09:27.2 to `memo メモ 2026.txt`:

```text
19:09:27.222 STEP1-START
19:09:29.258 HOOK-BEGIN path=</srv/Plain/memo メモ 2026.txt> name=<memo メモ 2026.txt> ext=<.txt> parent=</srv/Plain> event=<U> isdir=<false> args=0
19:09:29.531 SYNC-START pass=1
19:09:37.552 SYNC-END pass=1
19:09:37.603 SYNC-START pass=2
19:09:45.636 SYNC-END pass=2
19:09:45.674 HOOK-END
```

One hook run, one trigger run, one sync run of both passes.
The hook's `systemctl start` blocked for the whole 16 seconds, so `HOOK-END` follows the sync's end.
Pass: one edit gives exactly one trigger and one sync.

### Step 2: an edit during a running sync

```text
19:10:05.050 STEP2-START
19:10:07.067 HOOK-BEGIN (run 1)
19:10:07.125 SYNC-START pass=1        (run A)
19:10:09.118 STEP2-EDIT-DURING-SYNC
19:10:15.134 SYNC-END pass=1
19:10:15.161 SYNC-START pass=2
19:10:23.173 SYNC-END pass=2          (run A finished, both passes)
19:10:23.213 HOOK-END (run 1)
19:10:23.246 HOOK-BEGIN (run 2)
19:10:23.371 SYNC-START pass=1        (run B)
19:10:39.494 SYNC-END pass=2          (run B finished, both passes)
19:10:39.569 HOOK-END (run 2)
```

The second edit landed 1.99 seconds into run A's first pass.
Run A produced both of its pass end lines, so nothing interrupted it.
Run B started 73 milliseconds after run 1's hook ended,
that is as soon as the single run slot freed, not 2 seconds later:
the queued run had already been collected while the hook was blocked.
Exactly one more sync, and one more only.
Pass: an edit during a running sync gives exactly one more sync after it,
and the running one is not interrupted.

### Step 3: a sync started directly, the timer's path, plus an edit during it

```text
19:11:09.284 STEP3-START
19:11:09.629 SYNC-START pass=1        (run A, started by systemctl start --no-block)
19:11:11.541 STEP3-EDIT-DURING-SYNC
19:11:13.611 HOOK-BEGIN
19:11:25.893 SYNC-END pass=2          (run A finished, both passes)
19:11:26.029 SYNC-START pass=1        (run B, started by the trigger)
19:11:42.196 SYNC-END pass=2          (run B finished, both passes)
19:11:42.238 HOOK-END
```

Run A was not started by the watcher at all.
Its edit still ran the hook, whose `systemctl start` of the trigger waited for run A to end
(the trigger's `After=` ordering) and then started run B 136 milliseconds after run A's last pass.
One fresh sync after the direct one, and run A finished both passes.
Pass: the timer's path plus an edit during it gives one fresh sync after it.

### Step 4: a burst of 200 small files over about three seconds

```text
19:11:54.965 STEP4-START
19:11:59.897 STEP4-WRITE-DONE        (200 files over 4.9 s including the loop's own sleeps)
19:11:57.049 HOOK-BEGIN  path=<burst 75.txt>  event=<C>   (run 1)
19:12:13.311 HOOK-BEGIN  path=<burst 159.txt> event=<C>   (run 2)
19:12:29.600 HOOK-BEGIN  path=<burst 200.txt> event=<C>   (run 3)
```

Three hook runs and three syncs for 200 files written over about five seconds.
The division is the batch settings:
the first 2000 ms window flushes on `maxWaitMillisecond`,
then each further 2000 ms window flushes 2000 ms after its last event,
and each flush's `deliver: last` names the last file of that window.
Runs 2 and 3 waited in the watcher's queue while the earlier hook ran.
Recorded as measured, without a judgement:
a burst of N seconds of writing costs one sync per batch window, not one sync per burst.

### Step 5: a new folder with files inside, then an edit inside it

```text
19:13:00.726 STEP5-START
19:13:02.894 HOOK-BEGIN path=</srv/Plain/新しい フォルダ (2) [x]> name=<新しい フォルダ (2) [x]> ext=<> isdir=<true> event=<C>
19:13:19.140 HOOK-END
19:13:27.160 STEP5-EDIT-INSIDE
19:13:29.204 HOOK-BEGIN path=</srv/Plain/新しい フォルダ (2) [x]/first ファイル.txt> event=<U>
```

The folder and its two files produced one hook run, a folder creation event with `isdir=true`,
and no file events for the two files written into it:
that matches the source record's GW-12.
The later edit inside the folder produced a second run with a file event.
Both changes start a run, and a folder created after start is watched.

### Step 6: a save done as write a temporary file, rename it over the original

```text
19:14:23.806 STEP6-START
19:14:25.865 HOOK-BEGIN path=</srv/Plain/atomic save.txt> event=<C>     (the file's first creation)
19:14:50.541 STEP6-ATOMIC-SAVE                                        (write atomic save.txt.tmp, mv over atomic save.txt)
19:14:52.617 HOOK-BEGIN path=</srv/Plain/atomic save.txt> event=<U>     (one run)
```

The atomic save produced exactly one run,
reported against the final name with event `U`.
The temporary file's own creation and write were in the same batch window and did not add a run.
Recorded without a judgement: one run per atomic save, and it names the final path.

### Step 7: a metadata-only change, `touch` and `chmod`

```text
19:15:20.678 STEP7-TOUCH
19:15:28.725 STEP7-CHMOD
19:15:40.897 STEP7-QUIET
```

No hook run, no sync, and no queued run either:
the log holds no `HOOK-BEGIN` between `STEP7-START` and `STEP7-QUIET`.
The watcher's own DEBUG log records the events and drops them:

```text
event: CHMOD         "/srv/Plain/memo メモ 2026.txt"
event: CHMOD         "/srv/Plain/memo メモ 2026.txt"
```

Neither line has the `has modified` or `has created` line that a delivered event gets,
which matches the source's `default: return nil` for an operation it does not map:
there is no `fsnotify.Chmod` branch in `handleEvent`.
Measured consequence for the design: a save that only touches timestamps or modes starts no sync,
which is what the design wants, and `touch` is a safe way to prove the watcher is alive.

### Step 8: a `.partial` file created, written, renamed to its final name

```text
19:15:45.639 STEP8-PARTIAL-CREATE       (create download target.txt.partial, write it twice)
19:15:53.658 STEP8-PARTIAL-WRITTEN
19:15:55.729 HOOK-BEGIN path=</srv/Plain/download target.txt> event=<C>  (only after the rename)
```

The watcher logged the excluded file's events and delivered none of them:

```text
event: CREATE        "/srv/Plain/download target.txt.partial"
event: WRITE         "/srv/Plain/download target.txt.partial"
event: RENAME        "/srv/Plain/download target.txt.partial"
event: CREATE        "/srv/Plain/download target.txt" ← "/srv/Plain/download target.txt.partial"
```

So the exclusion covers the temporary file's creation and writes,
and the rename to the final name starts one run, reported as a creation of the final path.
That is the shape rclone's download writes, which the design record measured on a throwaway Garage.

### Step 9: every odd file name

Eleven edits, one per name, each followed by a wait for quiet. One hook run and one sync per name,
eleven in all, and no error line. The `DW_*` values each run received, as the hook logged them
inside angle brackets so a split argument or a stripped character would show:

- `memo メモ 2026.txt`: path and abs path `/srv/Plain/memo メモ 2026.txt`,
  name `memo メモ 2026.txt`, ext `.txt`, parent `/srv/Plain`, event `U`, isdir `false`.
- `中文 文件 (final) [ok].txt`: name kept its parentheses and brackets, ext `.txt`.
- `全角：コロン と 半角:コロン.txt`: both the fullwidth colon U+FF1A and the ASCII colon survived.
- `田中's メモ.txt`: the single quote survived.
- `"quoted" メモ.txt`: both double quotes survived.
- `[bracket] (paren) メモ.txt`: brackets and parentheses survived.
- `日本語 フォルダ 1/保存 (draft) [v1].txt`: parent `/srv/Plain/日本語 フォルダ 1`, name `保存 (draft) [v1].txt`.
- `中文目录 二/说明 (draft) [v2].txt`: parent `/srv/Plain/中文目录 二`.
- `落合's フォルダ/it's inside (1).txt`: a single quote in the folder name and in the file name both survived.
- `深い/階層/の/フォルダ/nested deeper (1) [2].txt`: the four-level path arrived whole.
- `深い/階層/の/フォルダ/全角：コロン.txt`: fullwidth colon inside a deep path.

Every run received `args=0` in its first hook command, which takes no arguments:
a name is never split into shell words and never reaches a command line.
The folder event of step 5 shows the other shape: `ext=<>` and `isdir=<true>`.

### Step 10: SIGTERM while a hook runs

```text
19:20:43.134 STEP10-START
19:20:45.231 HOOK-BEGIN (run)
19:20:45.374 SYNC-START pass=1        (run A)
19:20:47.516 STEP10-PRE-SIGTERM watcher=176 hookclient=1974 sync=activating trigger=activating
19:20:47.554 WATCH-STOP               (the onFinishWatch hook ran)
19:20:47.605 STEP10-POLL watcher=176 hookclient=none sync=activating trigger=activating
19:20:48.669 STEP10-POLL watcher=none hookclient=none sync=activating trigger=activating
19:20:53.417 SYNC-END pass=1
19:20:53.470 SYNC-START pass=2
19:21:01.500 SYNC-END pass=2          (run A finished both passes)
```

The SIGTERM went to the watcher 2.14 seconds into the sync's first pass.
Within the next poll, 90 milliseconds later, the blocking `systemctl start` client of the trigger
was gone: the watcher signalled the hook's process group, as the source record's GW-17 describes.
The watcher itself exited within 1.15 seconds, so the 20-second grace was not needed.
The sync unit kept both of its passes and ended them normally,
and the trigger unit ended with `Result=success` and `NRestarts=0`.
The hook's third command never ran, and no `[ERROR]` line was logged for it,
which matches the documented rule that a command stopped by watching stopping is not a failure.
No queued run started after the watcher exited.

## Platforms this run could not exercise

### macOS arm64: not run

Reason: the only macOS machine is the owner's,
reached as `ssh m1`,
and it runs nothing third-party without the owner's approval;
this run's rules forbid touching it at all.
No other macOS host exists for this run.

What a run there would have to show, in the order that decides it:

- the released `directory-watcher_Darwin_arm64.tar.gz`,
  whose release checksum line is `d90ccc89815f919e894ebf62537f498c3840ddd8bbf1be73d2fe37c830e52be5`,
  installs and prints version 3.2.1;
- the kqueue descriptor cost against the job's own limit:
  the source record's GW-21 predicts one descriptor per watched folder and one per file inside it,
  which for the live tree's 1454 folders and 10332 files is about 11786 descriptors,
  and the read-only probe of the Mac recorded a soft `ulimit -n` of 256 for a login shell.
  The run must show the LaunchAgent's own limit and that the watcher starts with that tree,
  or record the largest tree it does start with;
- one edit in a watched folder runs the hook once, a second edit during that hook runs exactly one more,
  and no hook run is interrupted;
- the same eleven odd names, written by macOS,
  arrive with the same values in `DW_NAME` and `DW_PATH` and run the hook once each,
  including the case where the shell is `sh` on macOS rather than a GNU userland;
- a folder created after start is watched and its edit runs a hook,
  and `**/*.partial` is excluded the same way;
- a save done as write-then-rename runs the hook once,
  because macOS reports the new path before the old one and the watcher holds the pair up to a second;
- `launchctl` stopping the job (SIGTERM) leaves the sync child running to completion,
  which is the round 4 hard requirement on the client that does not use systemd.

### Windows x64: not run

Reason: no Windows machine is available to this run.
The project's own CI runs its suite on `windows-latest` (run `42050842995` at the tag),
and the release publishes `directory-watcher_Windows_x86_64.zip`
with release checksum `6fd9c034cdcc4c30d85deb1d39d5f0cd9452c2947ecd4e5bd32b3fc96b94ff26`,
but neither is a run on a machine this run can drive.

What a run there would have to show:

- the published zip installs and prints version 3.2.1;
- one edit runs the hook once through `cmd` (the Windows default shell),
  a second edit during that hook queues exactly one more run,
  and no hook run is killed;
- the odd names survive as far as NTFS allows them:
  spaces, parentheses, brackets, single and double quotes and CJK characters are legal there,
  and a fullwidth colon is legal because only the ASCII colon is reserved,
  so the same eleven names can be created; the run must confirm each `DW_NAME` value is intact,
  and that a name Windows refuses is recorded rather than silently dropped;
- a metadata-only change starts no run: `ReadDirectoryChangesW` reports attribute changes,
  so this needs the same `touch`-then-`chmod` check step 7 of this file made on Linux;
- the `.partial` exclusion behaves the same through `ReadDirectoryChangesW` renames;
- a save done as write-then-rename runs the hook once;
- stopping the job leaves the sync child running:
  Windows cancellation goes through Job Objects and `TASKKILL /T` in the project's own harness,
  so the round 4 requirement has to be shown there rather than inferred from Linux.

## Summary

Every verdict below names the step or record that decides it.
No verdict here is a score, a ranking or a recommendation.

### The vet report's hard constraints

- Open source with inspectable source: pass.
  Tree 1's clone is at the tag's commit `7286f534dbeb3854607b3a48875c4cf0bbb5e672`,
  and the source record's GW-02 reads that tree; the CI-equivalent run built and tested it.
- Runs an arbitrary command, here `systemctl start` of a trigger unit: pass.
  Steps 1 to 10 all ran the hook, and `make smoke-example` ran the same config end to end.
- Runs on Linux x86-64, macOS arm64 and Windows x64: open.
  Linux x86-64 is exercised twice, by tree 2 and tree 3;
  macOS arm64 and Windows x64 are not run, and "Platforms this run could not exercise" lists
  what a run there has to show.
- Watches every folder under the root recursively, including folders created after start: pass.
  The watcher added eight watches at startup of the fixture, and step 5's folder,
  created after start, was watched and its later edit ran a hook.
- After a change that lands while a sync runs, one more sync starts after that run ends: pass.
  Step 2 gave exactly one more sync, and step 3 gave one fresh sync after a directly started one.
- Never interrupts a running command; each run finishes: pass.
  All 25 sync runs of tree 3 completed both passes, including the run a SIGTERM interrupted
  in step 10.
- Queues one more run of its command natively when events arrive while the command runs: pass.
  Step 2's second run started when the first run's hook ended, and step 4's three runs serialized.

### The ten measured behaviours

- One edit, one trigger, one sync: pass, step 1.
- An edit during a running sync gives exactly one more sync after it,
  and the running one is never interrupted: pass, step 2.
- A sync started directly, the timer's path, plus an edit during it gives one fresh sync after it:
  pass, step 3.
- A burst of 200 small files written over about five seconds:
  measured, three hook runs and three syncs, step 4.
  No requirement fixes how many runs a burst may cause, so this is a number, not a verdict.
- A new folder created with files inside, then an edit inside it, both start a run:
  pass, step 5, one hook run for the folder and one for the later edit.
- A save done as write a temporary file, rename it over the original:
  measured, one hook run and one sync, step 6.
- A metadata-only change, `touch` and `chmod`: pass, step 7, no run for either.
- A `.partial` file created, written and renamed to its final name:
  pass, step 8, nothing for the temporary file and one run for the rename.
- Every odd file name: pass, step 9, no error line and every `DW_*` value intact.
- Stopping the watcher with SIGTERM while a hook runs:
  pass, step 10, the hook's process group is signalled, the sync finishes both passes,
  and the watcher exits in about a second.

### The upstream CI-equivalent path

Pass, on the corrected run: `go test -count=1 -race -v ./...` exited 0 in 162.7 seconds
with 240 top-level tests and 421 subtests passed, 2 kqueue-only skips, and every package green,
after `go build ./...`, `go vet ./...` and `make smoke-example` also exited 0.
The first attempt at the same path failed nine tests,
and "Diagnosing the two failing packages" shows that the cause was this run's missing init process,
not the candidate.

### Open items

- macOS arm64 and Windows x64: not run; see the section that names what they must show.
- The macOS descriptor arithmetic of the source record's GW-21 stays arithmetic until a Mac run.
- The system scope stood in for the user scope, as this run's instruction fixed.
  The user-scope path was not exercised:
  a real deployment starts a user unit over the session bus with `XDG_RUNTIME_DIR` set,
  and GW-23's claim that the hook inherits that environment is a source reading, not a measurement.
- A sustained burst was not measured.
  Step 4 shows three seconds of writing costing three runs against a 2048-run queue;
  what minutes of writing does to the queue and to the sync backlog is not measured.
- The design record's `TimeoutStartSec=3000` question belongs to that record and was not measured here.

### Surprises

Each line is something this run did not expect from the source record or the design record.

- The hook blocks for the whole sync, because `systemctl start` waits for the unit.
  So one hook run occupies the single run slot for the length of a sync,
  and a queued run starts the instant that slot frees, without the batch wait again (steps 1 and 2).
- A burst costs one sync per batch window rather than one per burst:
  200 files over five seconds made three runs, because `maxWaitMillisecond` follows
  `waitMillisecond` up to 2000 ms and each window delivers only its last path (step 4).
- A folder created with files inside produced one hook run and no file events for its contents:
  the source record's GW-12 limit is visible in the log and costs nothing, because the folder event
  alone starts the sync (step 5).
- An atomic save is one run, reported as `U` against the final name, not a delete plus a create (step 6).
- `touch` and `chmod` reach the watcher as `event: CHMOD` at DEBUG level and are dropped,
  so a `touch` is a safe liveness probe (step 7).
- A `.partial` file's creation and writes are logged and never delivered,
  and the rename delivers a creation of the final path (step 8).
- A SIGTERM to the watcher ends the hook's `systemctl` client but not the sync,
  and the watcher exits in about a second rather than using its 20-second grace (step 10).
- The container's `is-system-running` is `degraded` from start to finish
  (`dbus-broker`, `systemd-resolved`, `systemd-homed` and `systemd-firstboot` fail there),
  and nothing this run needed goes through the bus.
- The upstream suite needs an init process inside a container:
  without one, nine tests failed on liveness checks that count zombies as alive.
  The same tests passed under a heavier load once `--init` was added.

## Where this run departs from its instructions and from the earlier records

- The isolation defaults this run was given do not include `--init`.
  The CI-equivalent path needs it, for the reason amendment 4 records,
  so it was added and the manifest amended rather than the suite being declared failing.
- The instruction says "a private scratch volume under your scratch folder".
  The systemd container got a bind mount of `container-share/` inside the scratch folder
  instead of a named podman volume, so its inputs can be read and removed with the folder;
  the named volume was used only for the Go module cache.
- The burst was asked for as 200 files "over about 3 seconds";
  the loop's own `sleep 0.015` per file plus the host's load made it 4.9 seconds,
  so the numbers in step 4 are for a five-second burst.
- The instruction stands system units in for user units.
  That is what was measured, and the user-scope path stays open, as "Open items" says.
- Nothing found here contradicts the source record's GW-12, GW-13, GW-15, GW-17, GW-18 or GW-19;
  those readings are now measurements, with the numbers above.

## Where the evidence lives

Logs and measurements are kept in
`/home/user/temp/agent/watcher-finalist-validation-directory-watcher-scratch-2026-11-25/keep/`:

- `tree2-goversion.log`, `tree2-build.log`, `tree2-vet.log`, `tree2-smoke.log`;
- `tree2-test.log` and `tree2-test-status.txt`, the first, failing test step;
- `tree2-rerun-scenarios.log` and `tree2-rerun-load.txt`, the eight tests alone without `--init`;
- `tree2-init.log` and `tree2-init-load.txt`, the same eight with `--init`;
- `tree2-initfull.log` and `tree2-initfull-load.txt`, the corrected full run;
- `tree2-serial.log` and `tree2-serial-load.txt`, the stopped serial diagnostic;
- `tree3-measure.log` (the 212-line marker log), `tree3-watcher.log` (1065 lines),
  `tree3-config.yml`, `tree3-fixture-listing.txt` and `tree3-snapshot.txt`.

Retained test diagnostics from the diagnostic reruns are in the scratch folder's
`e2e-diag-scenarios/`, `e2e-diag-init/` and `e2e-diag-initfull/` while they exist;
they are not part of the kept set.
