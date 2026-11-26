# Runbook: self-hosted file sync (Garage S3 + rclone bisync)

Rebuild-from-scratch procedure for the Seafile replacement:
 a single-node Garage
S3 store on Coolify behind a self-managed Caddy,
 synced to one or more clients by
rclone bisync on a schedule (every five minutes and on every change on Linux, through systemd,
 hourly through launchd on macOS
 and through Task
Scheduler on Windows).

Personal infrastructure,
 no secrets in this file:
 credentials appear only as
placeholder names,
 never values.

## Client check at 2026-11-25T16:54:53-05:00

The CachyOS timer is enabled and waiting.
Its 16:01 run exited successfully after checking 20,664 entries with no changes.
An authenticated read-only `rclone lsf garage:files` returned `Plain/`;
TLS verification succeeded and the expected unauthenticated root request returned HTTP 403.

The local `main.kdbx` was saved at 16:48,
after that sync,
so the remote still held the preceding database at this check.
The next scheduled run is 17:00:30.
The four older `main.kdbx.conflict*` files remain on both sides;
they are historical artifacts,
not evidence that the latest sync failed.

## Sync on file change: live on this PC since 2026-11-25T22:01:09-05:00

This PC runs the design confirmed in `doc/planning/garage-bisync-run-on-file-change.md`:
directory-watcher starts a sync ten seconds after changes under `~/Plain` stop (at most a minute into a burst),
each sync runs bisync twice,
the timer runs every five minutes,
and alerts come on the first failure of a streak, hourly while it lasts, and once on recovery.
Names starting with `.` or `_` are not watched and sync on the timer (owner's decision).
The Mac's agent runs both passes with `--conflict-resolve newer` since 2026-11-25T21:15:56-05:00.

Open:

- The first change-triggered sync on live data ran at 2026-11-25T23:19:37-05:00,
  13 s after five files were deleted under `~/Plain`, and ended with both passes `Bisync successful`.
- Smaller review findings left open are listed in the design record, section "Coverage and review fixes merged".
- The five `main.kdbx.conflictN` copies were removed from this PC, the bucket and the Mac at 2026-11-25T23:19:24-05:00.
- The other 29 conflict copies were resolved by 2026-11-25T23:53:24-05:00, so `~/Plain`, the bucket and the Mac hold none.
  Seven were machine state (`Text/Reference/Legal/.git`, whose conflict ref pointed at a commit already in `main`, and two IDE files).
  Thirteen documents had no unique text or a few superseded lines (owner: delete).
  Seven were older versions of files later revised on purpose: `casl-spam-report-bell-2026-08.md.conflict2` equals the committed snapshot `a0248b6`,
  whose lines the commit `0ccd0d9` removed; the `clinical-summary` set, `emails-to-send-2026-08-26.md` and the `carleton-handover` sidecar
  each equal an earlier commit of the Health repository, and a named later commit removed their lines
  (for example `90aaa00`, "correct ten problems the translation pass found in the English").
  `Text/Notes/pcloud-console-client-mac-setup.md` was restored from `.conflict2`, a deliberate revision of `.conflict1`.
  Every removed copy is backed up with checksums in `~/temp/agent/garage-*-conflict-copies-removed-2026-11-25/` (four folders, 7.2 MB);
  the owner chose to keep them.
- `Text/Reference/Health` is a Git working tree whose Git directory lives outside `~/Plain`.
  Its pointer named the Bazzite path `/var/home/user/.local/share/health-records-git`, which does not exist on CachyOS;
  the directory was restored at 2026-11-26T00:00:19-05:00 from the old Bazzite NVMe (`old-nvme-p2`, mounted read-only)
  to `~/.local/share/health-records-git` (`git fsck` clean; last commit `cf05c3f`, 2026-09-25T03:46:26-04:00),
  and `Health/.git` now reads `gitdir: /home/user/.local/share/health-records-git`.
  206 of its 222 tracked files equal `HEAD`; 16 differ (14 Git LFS PDFs in `dist/`, `provider-requests.html`, `runbook-medication-continuity.md`).
- Git works again in `Text/Reference/Health` and `Text/Reference/Legal` on this PC:
  AppArmor's `git` profile had granted only project folders, and the twelfth batch of local rules
  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`) grants `~/Plain` and the Health Git directory.
  Each Git run leaves one lock-holder record in the repository's `.git/cli-git-lock-holders/`,
  removed by a later run (the Git wrapper's design); in Legal that folder is inside `~/Plain` and syncs.
- The Mac keeps the files it set aside before its resync in
  `/Users/user/temp/agent/mac-plain-pre-resync-backup-2026-11-25/` until the owner says to remove them.
- The files this PC ran before the change are in `~/temp/agent/garage-live-units-before-sync-on-change-2026-11-25/`
  (rollback steps: `doc/decision/garage-file-sync-change-watcher.md`).

## Client check (macOS, `ssh m1`) at 2026-11-25T18:51:49-05:00

Read-only over ssh; nothing was changed.
The LaunchAgent `cat.aquati.rclone-bisync-garage` is loaded
(`launchctl print`: 198 runs, last exit code 7, program `/opt/homebrew/bin/rclone`).
`~/Library/Logs/rclone-bisync-garage.log` starts at 2026-09-14 16:00
and holds 682 runs that each end with
`Bisync critical error: cannot find prior Path1 or Path2 listings, likely due to critical error on prior run`,
and no `Bisync successful` line at all.
The work folder `~/Library/Caches/rclone/bisync/` is empty.
So this client has not synced since its agent was installed:
its baseline listings either never existed at the agent's paths or were removed.
Recovering needs a `--resync`, which the owner has not authorised.

Read-only comparison at 2026-11-25T19:55:43-05:00, on the owner's go-ahead
(`rclone check garage:files/Plain /Users/user/Plain --filter-from <the agent's filter> --fast-list --size-only --combined`,
output in `~/temp/agent/mac-plain-vs-bucket-compare-2026-11-25/` on the Mac):
10009 files match;
214 exist only on the Mac, every one last modified before 2026-09-14,
208 of them in one top-level folder of game mods;
312 exist only in the bucket;
12 differ in size, `main.kdbx` among them
(the Mac's copy dated 2026-09-09, 901760 bytes; the bucket's dated 2026-11-25 18:01, 907268 bytes).
rclone's `docs/content/bisync.md` at `v1.76.0` says `--resync` implies `--resync-mode path1`,
so a default resync uploads the 214, downloads the 312,
and overwrites the 12 on the Mac with the bucket's versions.
The Mac's LaunchAgent gained `--conflict-resolve newer` and the settle pass at 2026-11-25T21:15:56-05:00
(owner's choice: both passes through `/bin/sh -c` in the plist);
its first run after the change ran both passes, each ending `Bisync successful`, and launchd recorded exit code 0.
The plist it replaced is kept on the Mac in `~/temp/agent/mac-bisync-agent-plist-before-two-passes-2026-11-25/`.
This PC's unit has one pass until the package `package/garage-file-sync/linux-client` goes live.

Recovered at 2026-11-25T20:04:52-05:00 on the owner's word
("Treat the bucket as the golden copy. I intentionally deleted the game mods from the bucket. Resync."):

1.  The 12 files that differed were copied,
    and the 214 Mac-only files moved,
    into `/Users/user/temp/agent/mac-plain-pre-resync-backup-2026-11-25/` on the Mac
    (`differing/` and `only-on-mac/`, with the two path lists);
    nothing was deleted.
2.  A fresh `rclone check --size-only` showed 0 Mac-only files.
3.  `rclone bisync garage:files/Plain /Users/user/Plain` with the agent's flags plus `--resync`
    exited 0 in 25.2 s, transferring 324 files (the 312 bucket-only and the 12 differing, bucket versions winning);
    log at `resync.log` in the backup folder.
4.  A second `rclone check --size-only`: 10333 match, nothing else.
5.  `launchctl kickstart` of the agent: `Bisync successful` in 4.7 s, last exit code 0.

The backup folder stays until the owner says to remove it.

## What this builds

- Garage v2.3.0 single-node S3 store on the Coolify host,
   bucket `files`,
  reachable at `https://garage.c.aquati.cat`.
- Caddy front proxy that terminates TLS and offers only HTTP/1.1 and HTTP/3 (no
  HTTP/2),
   because HTTP/2's flow-control window throttles uploads to about
  5 MB/s over the ~116 ms RTT to the server.
- An rclone `garage` S3 remote on each client,
   path-style,
   region `garage`.
- A bisync exclude filter (`garage-filter.txt`,
   rule `- **‛**`) passed by every
  client via `--filter-from`.
   Without it,
   bisync aborts on the stale double-encoded
  "phantom" keys that the bucket lists but cannot serve.
   See the phantom-key
  section below and
  [the phantom-keys troubleshooting doc](../troubleshooting/rclone-double-encode-phantom-keys.md).
- Two flags on every bisync invocation,
   needed because the link to the server has
  a high RTT:
   `--fast-list` (one bulk recursive S3 listing instead of a
  per-directory walk) and `--use-server-modtime` (compare on each object's
  LastModified from the listing instead of a per-object HEAD for mtime metadata).
  Without them a single run spends minutes HEADing every object.
   Trade-off:
   a
  client's local file mtimes become the server-side LastModified,
   not the original
  authoring time.
- A scheduler per client:
   a `user` systemd timer every five minutes plus a change watcher on Linux,
   a launchd
  LaunchAgent on macOS,
   a Task Scheduler job on Windows,
   each bisyncing
  `garage:files/Plain` against the client's local `Plain` directory.

## Artifacts referenced (do not duplicate)

In the personal notes folder (not this repo):

- `garage-coolify-compose.yml`:
   the full Garage compose,
   the secret-generation
  commands,
   the storage split,
   and the rclone client commands.
   Paste this into
  Coolify in Stage A;
   read its header for the env vars to set.
- `decision-file-sync.md`:
   why Garage over copyparty,
   Seafile,
   Nextcloud,
   OCIS,
  Syncthing.

In this repo:

- [rclone-double-encode-phantom-keys.md](../troubleshooting/rclone-double-encode-phantom-keys.md):
  why the filter exists,
   the full root cause,
   and every removal attempt that failed.

## Bridges tried (why some steps are manual)

- The client parts (rclone remote,
   scheduler units) are plain commands;
   on a
  fresh machine you run Stage C and the matching Stage D directly,
   no UI needed.
- The Garage deploy is a Coolify web-UI action.
   Coolify has an API,
   but the
  established workflow is the UI,
   so Stage A is written as UI steps.
- The Caddyfile is on the Coolify host and self-managed (Coolify's own Caddy
  management is disabled).
   Clients cannot reach the server's admin surface
  (Garage admin API on `:3903` is loopback-only on the host),
   so Stage B is done
  over SSH on the host.

## The phantom-key filter (read before Stage C)

The bucket lists a small set of keys (5 as of 2026-06) whose names carry rclone's
escape rune U+201B (`‛`),
 created once by a double-encode during the original
Seafile-to-Garage migration;
 each is a `…‛‛：…` name.
 Garage lists them but returns
`404` on HEAD/GET,
 and rclone bisync treats a single unreadable source object as
fatal:
 the whole run aborts with `Bisync aborted. Must run --resync to recover`.

They cannot be removed:
 an S3 `DELETE` is a silent no-op on them,
 and
`garage repair tables` does not reconcile them on a single node (both verified).
So every client excludes them with a filter instead.
 Nothing regenerates them
(the only writer is rclone on the default encoding with a clean local tree),
 so
this is a permanent,
 set-once measure.

The rule must be `- **‛**`,
 matching `‛` anywhere in the full path.
 The narrower
`- *‛*` matches only the base name,
 so it catches a phantom file
(`…‛‛：第2卷.pdf`) but misses one nested inside a phantom directory
(`…‛‛：第2卷.sdr/metadata.pdf.lua`,
 base name `metadata.pdf.lua`).
 Default-modtime
runs hid that gap:
 they HEAD every object while listing and silently drop the
un-HEAD-able phantom.
 With `--use-server-modtime` there is no per-object HEAD,
 so
the filter is the only thing keeping the phantom out.
 Use the full-path form on
every client.

Create the filter file on each client before its first bisync:

```bash
printf '%s\n' '- **‛**' > ~/.config/rclone/garage-filter.txt
# verify the bytes are 2d 20 2a 2a e2 80 9b 2a 2a 0a (the ‛ is a literal U+201B)
hexdump -C ~/.config/rclone/garage-filter.txt
```

On Windows the file lives beside the rclone config at
`%APPDATA%\rclone\garage-filter.txt`;
 the byte content is identical.

Pass it to every bisync invocation as `--filter-from ~/.config/rclone/garage-filter.txt`.
Changing the filter between runs makes bisync abort with "filters have changed",
so the first run after adding it must be a `--resync`.

## Setup

Status:
 TODO | DONE

Have these ready before starting:

- Login to **Coolify** for the target server.
- An **SSH** session to the Coolify host (for the data dir and the Caddyfile).
- The saved S3 credentials `GARAGE_DEFAULT_ACCESS_KEY` (format `GK` + 24 hex)
  and `GARAGE_DEFAULT_SECRET_KEY` (64 hex).
   If lost,
   you will mint new ones in
  Stage A and re-point rclone in Stage C.
- rclone on each client:
   `mise use -g rclone` on Linux,
   `brew install rclone` on
  macOS (Homebrew installs to `/opt/homebrew/bin/rclone`).

## Steps

Status:
 TODO | DONE

### Stage A: deploy Garage on Coolify

1.  On the Coolify host SSH shell,
     run `mkdir -p /mnt/storagebox/garage-data`.
    Expected:
     no output,
     or `File exists`.
2.  In **Coolify**,
     open the **Project**,
     click **+ New**,
     choose **Resource**,
    then **Docker Compose Empty**.
     Expected:
     a compose editor opens.
3.  Paste the entire contents of `garage-coolify-compose.yml` into the editor.
    Expected:
     the editor shows the `garage` service and a `configs:` block.
4.  Open **Environment Variables** and add the five secrets named in the compose
    header (`GARAGE_RPC_SECRET`,
     `GARAGE_ADMIN_TOKEN`,
     `GARAGE_METRICS_TOKEN`,
    `GARAGE_DEFAULT_ACCESS_KEY`,
     `GARAGE_DEFAULT_SECRET_KEY`),
     each generated with
    the command written beside it in that header.
     Expected:
     five variables saved.
    Keep the access key and secret key;
     they are your S3 login for Stage C.
5.  Click **Deploy**.
     Expected:
     the deploy log ends with the `garage` container
    running and no restart loop.

### Stage B: drop HTTP/2 at Caddy

1.  On the host,
     open your self-managed **Caddyfile** in an editor.
2.  In the global options block at the very top (the leading `{ ... }`),
     add:

     ```caddyfile
     {
         servers :443 {
             protocols h1 h3
         }
     }
     ```

    Expected:
     the file now contains a `servers :443` block with `protocols h1 h3`.
3.  Confirm the `garage.c.aquati.cat` site reverse-proxies to **127.0.0.1:3900**
    (the loopback port the compose publishes).
     Expected:
     the site block routes to
    `127.0.0.1:3900`.
4.  Reload Caddy:
     `systemctl reload caddy` (or `caddy reload --config <path>`).
    Expected:
     the command returns with no error printed.

### Stage C: configure rclone on a client

1.  Confirm rclone is installed:
     `rclone version` prints `rclone v1.74.3` or newer.
2.  Create the remote (substitute the Stage A credentials):

     ```bash
     rclone config create garage s3 \
       provider Other \
       access_key_id <GARAGE_DEFAULT_ACCESS_KEY> \
       secret_access_key <GARAGE_DEFAULT_SECRET_KEY> \
       endpoint https://garage.c.aquati.cat \
       region garage \
       force_path_style true
     ```

    Expected:
     `rclone lsf garage:files` returns `Plain/` (or an empty list on a
    fresh store),
     with no `directory not found` error.
3.  Create the phantom-key filter file (see the phantom-key section):

     ```bash
     printf '%s\n' '- **‛**' > ~/.config/rclone/garage-filter.txt
     ```

    Expected:
     `hexdump -C ~/.config/rclone/garage-filter.txt` shows
    `2d 20 2a 2a e2 80 9b 2a 2a 0a`.
4.  Establish the baseline,
     first run only,
     never repeat `--resync`:

     ```bash
     rclone bisync garage:files/Plain ~/Plain --resync \
       --filter-from ~/.config/rclone/garage-filter.txt \
       --fast-list --use-server-modtime \
       --transfers 16 --checkers 32 --s3-upload-concurrency 8 -v
     ```

    Expected:
     the log ends with `Bisync successful`.
     The first run pulls the full
    tree (here ~55 GB),
     so it is bounded by download bandwidth;
     `--fast-list` plus
    `--use-server-modtime` keep the listing and comparison to a few seconds,
     so
    later runs finish in well under a minute.

### Stage D (Linux): install the sync, its five-minute timer and the change watcher

Every file comes from [`package/garage-file-sync/linux-client`](../../package/garage-file-sync/linux-client/README.md),
whose README explains each unit and whose unit files carry the reasons for each setting.
The design and its measurements are in [`doc/planning/garage-bisync-run-on-file-change.md`](../planning/garage-bisync-run-on-file-change.md),
and the choice of watcher in [`doc/decision/garage-file-sync-change-watcher.md`](../decision/garage-file-sync-change-watcher.md).

1.  Install directory-watcher through the repository toolchain:
     `mise run toolchain:install`.
    Expected:
     `~/.local/share/monochromatic-dev/toolchain/bin/directory-watcher --version` prints `directory-watcher version v3.2.1`.
2.  Copy the units and the watcher config:
     `mise run //package/garage-file-sync/linux-client:install`.
    Expected:
     `copied 8 files; run systemctl --user daemon-reload before starting the units`,
    and `mise run //package/garage-file-sync/linux-client:check:installed` then prints
     `every installed copy under <home> matches this package`.
3.  Run `systemctl --user daemon-reload`.
     Expected:
     no output.
4.  Run `systemctl --user enable --now rclone-bisync-garage.timer`.
     Expected:
     `systemctl --user list-timers rclone-bisync-garage.timer` shows a `NEXT` on a five-minute boundary, up to 30 s later.
5.  Run `loginctl enable-linger $(id -un)` so the timer fires while logged out.
    Expected:
     `loginctl show-user $(id -un) -p Linger` then prints `Linger=yes`.
6.  Trigger one run:
     `systemctl --user start rclone-bisync-garage.service`.
    Expected:
     the journal shows two `Bisync successful` lines, one per pass,
    and `journalctl --user -u 'rclone-bisync-garage-recovered@*'` shows
     `rclone-bisync-garage.service: finished cleanly with no open streak; no alert.`
7.  Start the watcher:
     `systemctl --user enable --now rclone-bisync-garage-watch.service`.
    Expected:
     its journal ends with `rclone-bisync-garage-watch.service: finished cleanly with no open streak; no alert.`,
    and the watcher's inotify watch count
     (`cat /proc/<MainPID>/fdinfo/* | grep -c '^inotify wd:'`)
    equals the folders under `~/Plain` that are not inside a `.` or `_` name, plus one.
    Measured on this PC at 2026-11-25T22:01:17-05:00: 1236 watches for 1235 such folders.

`|@rclone` in the sync unit resolves the command through the account's default login shell on every start:
`|` selects the shell and `@` enables login-shell startup.
This was verified with systemd `262.1-1.fc45` and the account's Bash login shell.
The mpm shadow manages the toolchain;
the service follows the first `rclone` on the login shell's `PATH`.
The retired `%h/.local/share/mise/shims/rclone` path was replaced on `2026-11-07`;
its failure and recovery are recorded in
[the toolchain migration troubleshooting record](../troubleshooting/mpm-toolchain-migration.md).
The watcher's binary is named by absolute path instead,
because neither `/bin/sh -l` nor the fish login shell finds the toolchain's `bin/`.
`--conflict-resolve newer` preserves the winning file's original name;
see [the conflict diagnosis](../troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md).
The
`--filter-from`,
 `--fast-list`,
 and `--use-server-modtime` flags are mandatory
(phantom keys and the high-latency link);
 see "What this builds".

### Stage D (macOS): install the hourly launchd agent

1.  Write `~/Library/LaunchAgents/cat.aquati.rclone-bisync-garage.plist` (the
    local path and the `--config` path must be absolute;
     launchd does not expand
    `~`):

     ```xml
     <?xml version="1.0" encoding="UTF-8"?>
     <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
     <plist version="1.0">
     <dict>
         <key>Label</key>
         <string>cat.aquati.rclone-bisync-garage</string>
         <!-- Two passes: the second settles the mtimes the first upload leaves behind
              (doc/troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md).
              A failed first pass skips the second. -->
         <key>ProgramArguments</key>
         <array>
             <string>/bin/sh</string>
             <string>-c</string>
             <string>/opt/homebrew/bin/rclone bisync garage:files/Plain /Users/USERNAME/Plain --config /Users/USERNAME/.config/rclone/rclone.conf --filter-from /Users/USERNAME/.config/rclone/garage-filter.txt --fast-list --use-server-modtime --conflict-resolve newer --transfers 16 --checkers 32 --s3-upload-concurrency 8 --resilient --recover -v &amp;&amp; /opt/homebrew/bin/rclone bisync garage:files/Plain /Users/USERNAME/Plain --config /Users/USERNAME/.config/rclone/rclone.conf --filter-from /Users/USERNAME/.config/rclone/garage-filter.txt --fast-list --use-server-modtime --conflict-resolve newer --transfers 16 --checkers 32 --s3-upload-concurrency 8 --resilient --recover -v</string>
         </array>
         <key>StartCalendarInterval</key>
         <dict>
             <key>Minute</key>
             <integer>0</integer>
         </dict>
         <key>RunAtLoad</key>
         <false/>
         <key>ProcessType</key>
         <string>Background</string>
         <key>Nice</key>
         <integer>10</integer>
         <key>LowPriorityIO</key>
         <true/>
         <key>StandardOutPath</key>
         <string>/Users/USERNAME/Library/Logs/rclone-bisync-garage.log</string>
         <key>StandardErrorPath</key>
         <string>/Users/USERNAME/Library/Logs/rclone-bisync-garage.log</string>
     </dict>
     </plist>
     ```

    Replace `USERNAME` throughout.
     Expected:
    `plutil -lint ~/Library/LaunchAgents/cat.aquati.rclone-bisync-garage.plist`
    prints `... : OK`.
2.  Load it:
     `launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/cat.aquati.rclone-bisync-garage.plist`.
    Expected:
     `launchctl list | grep rclone` shows the label with exit status `0`.
3.  Trigger one run:
     `launchctl kickstart gui/$(id -u)/cat.aquati.rclone-bisync-garage`.
    Expected:
     `~/Library/Logs/rclone-bisync-garage.log` ends with two `Bisync successful` lines, one per pass.

    A LaunchAgent runs only while the user is logged in (the Aqua session);
     there
    is no `linger` equivalent.
     For a personal Mac that stays logged in,
     the
    `StartCalendarInterval` fires every hour and catches up once on wake.

### Stage D (Windows): install the hourly Task Scheduler job

The default shell is `cmd`,
 and a mise-installed rclone is on `PATH` only in
interactive shells,
 so the job calls rclone by absolute path
(`%LOCALAPPDATA%\mise\shims\rclone.exe`,
 the mise shim,
 stable across
`mise upgrade rclone`).
 Logging uses rclone's own `--log-file` because Task
Scheduler cannot redirect with `>`.

1.  Confirm rclone:
     `"%LOCALAPPDATA%\mise\shims\rclone.exe" version` prints
    `rclone v1.74.3` or newer.
2.  Put the rclone config and filter under `%APPDATA%\rclone\`:
     `rclone.conf` (the
    `[garage]` remote from Stage C) and `garage-filter.txt` (the `- **‛**` rule).
    Write the filter from PowerShell so no BOM or codepage corrupts the `‛`:

     ```powershell
     [IO.File]::WriteAllBytes("$env:APPDATA\rclone\garage-filter.txt",
       [byte[]](0x2d,0x20,0x2a,0x2a,0xe2,0x80,0x9b,0x2a,0x2a,0x0a))
     ```

    Expected:
     `"%LOCALAPPDATA%\mise\shims\rclone.exe" listremotes` prints `garage:`,
    and `Format-Hex "$env:APPDATA\rclone\garage-filter.txt"` shows
    `2D 20 2A 2A E2 80 9B 2A 2A 0A`.
3.  Establish the baseline (first run only):

     ```bat
     "%LOCALAPPDATA%\mise\shims\rclone.exe" bisync garage:files/Plain "%USERPROFILE%\Plain" --resync --config "%APPDATA%\rclone\rclone.conf" --filter-from "%APPDATA%\rclone\garage-filter.txt" --fast-list --use-server-modtime --transfers 16 --checkers 32 --s3-upload-concurrency 8 --resilient --recover -v
     ```

    Expected:
     the run ends with `Bisync successful`.
4.  Write the task definition to `%USERPROFILE%\rclone-bisync-garage.xml` (replace
    `USERNAME`;
     Task Scheduler does not expand `%USERPROFILE%` inside
    `<Command>`/`<Arguments>`):

     ```xml
     <?xml version="1.0" encoding="UTF-16"?>
     <Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
       <RegistrationInfo>
         <Description>Hourly rclone bisync of garage:files/Plain to the local Plain dir.</Description>
       </RegistrationInfo>
       <Triggers>
         <CalendarTrigger>
           <Repetition>
             <Interval>PT1H</Interval>
             <StopAtDurationEnd>false</StopAtDurationEnd>
           </Repetition>
           <StartBoundary>2026-01-01T00:00:00</StartBoundary>
           <Enabled>true</Enabled>
           <ScheduleByDay><DaysInterval>1</DaysInterval></ScheduleByDay>
         </CalendarTrigger>
       </Triggers>
       <Principals>
         <Principal id="Author">
           <LogonType>InteractiveToken</LogonType>
           <RunLevel>LeastPrivilege</RunLevel>
         </Principal>
       </Principals>
       <Settings>
         <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
         <StartWhenAvailable>true</StartWhenAvailable>
         <RunOnlyIfNetworkAvailable>true</RunOnlyIfNetworkAvailable>
         <AllowStartOnDemand>true</AllowStartOnDemand>
         <Enabled>true</Enabled>
         <ExecutionTimeLimit>PT2H</ExecutionTimeLimit>
         <Priority>7</Priority>
       </Settings>
       <Actions Context="Author">
         <Exec>
           <Command>C:\Users\USERNAME\AppData\Local\mise\shims\rclone.exe</Command>
           <Arguments>bisync garage:files/Plain "C:\Users\USERNAME\Plain" --config "C:\Users\USERNAME\AppData\Roaming\rclone\rclone.conf" --filter-from "C:\Users\USERNAME\AppData\Roaming\rclone\garage-filter.txt" --fast-list --use-server-modtime --transfers 16 --checkers 32 --s3-upload-concurrency 8 --resilient --recover --log-file "C:\Users\USERNAME\rclone-bisync.log" -v</Arguments>
         </Exec>
       </Actions>
     </Task>
     ```

    `MultipleInstancesPolicy` of `IgnoreNew` keeps a slow run from overlapping the
    next hour;
     `LogonType` `InteractiveToken` runs as the logged-in user with no
    stored password (like the Mac agent,
     it runs only while logged in).
5.  `schtasks` reads the XML as UTF-16 only;
     a UTF-8 file fails with
    `ERROR: The task XML is malformed. ... unable to switch the encoding`.
     Rewrite
    it as UTF-16 (LE BOM),
     then register:

     ```powershell
     $p = "$env:USERPROFILE\rclone-bisync-garage.xml"
     [IO.File]::WriteAllText($p, (Get-Content -Raw -Encoding UTF8 $p), [Text.Encoding]::Unicode)
     ```

     ```bat
     schtasks /create /tn "rclone-bisync-garage" /xml "%USERPROFILE%\rclone-bisync-garage.xml" /f
     ```

    Expected:
     `SUCCESS: The scheduled task "rclone-bisync-garage" has successfully been created.`
6.  Trigger one run:
     `schtasks /run /tn "rclone-bisync-garage"`.
     Expected:
     once it
    finishes,
     the tail of `%USERPROFILE%\rclone-bisync.log` contains
    `Bisync successful`,
     and
    `schtasks /query /tn "rclone-bisync-garage" /v /fo list` shows `Last Result: 0`.

## What to check

Status:
 TODO | DONE

Run each and match the exact string:

- Transport is no longer HTTP/2:
  `curl -sSI https://garage.c.aquati.cat` shows a first line `HTTP/1.1 403`,
  plus headers `Via: 1.1 Caddy` and `Alt-Svc: h3=":443"`.
   A first line of
  `HTTP/2 403` means the `protocols h1 h3` change in Stage B did not take.
- Garage is healthy (on the Coolify host):
  `docker exec $(docker ps --format '{{.Names}}' | grep -i garage) /garage bucket list`
  output contains `files`.
- rclone can reach the bucket:
  `rclone lsf garage:files` returns `Plain/` without `directory not found`.
- The filter file is correct on each client:
  `hexdump -C ~/.config/rclone/garage-filter.txt` shows `2d 20 2a 2a e2 80 9b 2a 2a 0a`
  (on Windows,
   `Format-Hex` of `%APPDATA%\rclone\garage-filter.txt` shows the same bytes).
- The sync ran clean (Linux):
  `journalctl --user -u rclone-bisync-garage.service -n 40` contains
  `Bisync successful` twice per run, one per pass.
- The sync ran clean (macOS):
  `tail ~/Library/Logs/rclone-bisync-garage.log` contains `Bisync successful`.
- The sync ran clean (Windows):
   the tail of `%USERPROFILE%\rclone-bisync.log`
  contains `Bisync successful`,
   and
  `schtasks /query /tn "rclone-bisync-garage" /v /fo list` shows `Last Result: 0`.
- No phantom files reached the local tree:
  `find ~/Plain -path '*‛*'` (or `~/Seafile/Plain`) prints nothing;
   on Windows,
  `Get-ChildItem -Recurse "$env:USERPROFILE\Plain" | Where-Object Name -match '‛'`
  prints nothing.
- The scheduler is armed (Linux):
  `systemctl --user list-timers rclone-bisync-garage.timer` prints a `NEXT`
  timestamp within five and a half minutes.
- The watcher runs (Linux):
  `systemctl --user is-active rclone-bisync-garage-watch.service` prints `active`.
- The installed files match the package (Linux):
  `mise run //package/garage-file-sync/linux-client:check:installed` prints `every installed copy under <home> matches this package`.
- The scheduler is armed (macOS):
  `launchctl print gui/$(id -u)/cat.aquati.rclone-bisync-garage` prints
  `state = waiting` (or `not running`) with the program path.
- The scheduler is armed (Windows):
  `schtasks /query /tn "rclone-bisync-garage"` prints `Ready` under `Status`.

## Restore

Status:
 TODO | DONE

Undo,
 from least to most destructive.
 Local files in the client's `Plain`
directory are never touched by any of these.

- Stop and remove the scheduler (Linux):
  `systemctl --user disable --now rclone-bisync-garage.timer`,
   then
  `rm ~/.config/systemd/user/rclone-bisync-garage.{service,timer}`,
   then
  `systemctl --user daemon-reload`.
   Then `loginctl disable-linger $(id -un)`.
- Stop and remove the scheduler (macOS):
  `launchctl bootout gui/$(id -u)/cat.aquati.rclone-bisync-garage`,
   then
  `rm ~/Library/LaunchAgents/cat.aquati.rclone-bisync-garage.plist`.
- Stop and remove the scheduler (Windows):
  `schtasks /delete /tn "rclone-bisync-garage" /f`,
   then
  `del "%USERPROFILE%\rclone-bisync-garage.xml"`.
- Force a fresh baseline next time (clears bisync state):
  `rm -rf ~/.cache/rclone/bisync/garage_files_Plain..*` on Linux,
  `rm -rf ~/Library/Caches/rclone/bisync/garage_files_Plain..*` on macOS,
   or
  `rmdir /s /q "%LOCALAPPDATA%\rclone\bisync"` on Windows.
   The next sync then
  needs `--resync` again.
- Remove the rclone remote and filter:
   `rclone config delete garage`,
   then
  `rm ~/.config/rclone/garage-filter.txt` (on Windows,
  `del "%APPDATA%\rclone\garage-filter.txt"`).
- Re-enable HTTP/2 at Caddy:
   delete the `servers :443 { protocols h1 h3 }` block
  from the Caddyfile and `systemctl reload caddy`.
- Tear down Garage:
   in **Coolify**,
   delete the compose resource and its
  `garage-meta` volume;
   optionally empty `/mnt/storagebox/garage-data` on the
  host.
   This deletes all uploaded objects.
