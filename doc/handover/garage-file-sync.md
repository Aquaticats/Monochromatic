# Runbook: self-hosted file sync (Garage S3 + rclone bisync)

Rebuild-from-scratch procedure for the Seafile replacement:
 a single-node Garage
S3 store on Coolify behind a self-managed Caddy,
 synced to one or more clients by
rclone bisync on an hourly schedule (systemd on Linux,
 launchd on macOS,
 Task
Scheduler on Windows).

Personal infrastructure,
 no secrets in this file:
 credentials appear only as
placeholder names,
 never values.

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
- An hourly scheduler per client:
   a `user` systemd timer on Linux,
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

### Stage D (Linux): install the hourly systemd timer

1.  Write `~/.config/systemd/user/rclone-bisync-garage.service`:

     ```ini
     [Unit]
     Description=Bisync ~/Seafile/Plain with Garage S3
     Documentation=https://rclone.org/bisync/
     After=network-online.target
     Wants=network-online.target

     [Service]
     Type=oneshot
     WorkingDirectory=%h
     TimeoutStartSec=3000
     ExecStart=|@rclone bisync garage:files/Plain %h/Seafile/Plain --filter-from %h/.config/rclone/garage-filter.txt --fast-list --use-server-modtime --conflict-resolve newer --transfers 16 --checkers 32 --s3-upload-concurrency 8 --resilient --recover -v
     Nice=10
     IOSchedulingClass=best-effort
     IOSchedulingPriority=6
     ```

    `|@rclone` resolves the command through the account's default login shell on every start:
    `|` selects the shell and `@` enables login-shell startup.
    This was verified with systemd `262.1-1.fc45` and the account's Bash login shell.
    The mpm shadow manages the toolchain;
    the service follows the first `rclone` on the login shell's `PATH`.
    The retired `%h/.local/share/mise/shims/rclone` path was replaced on `2026-11-07`;
    its failure and recovery are recorded in
    [the toolchain migration troubleshooting record](../troubleshooting/mpm-toolchain-migration.md).
    `--conflict-resolve newer` preserves the winning file's original name;
    see [the conflict diagnosis](../troubleshooting/rclone-bisync-use-server-modtime-phantom-conflicts.md).
    The
    `--filter-from`,
     `--fast-list`,
     and `--use-server-modtime` flags are mandatory
    (phantom keys and the high-latency link);
     see "What this builds".
2.  Write `~/.config/systemd/user/rclone-bisync-garage.timer`:

     ```ini
     [Unit]
     Description=Run Garage bisync hourly

     [Timer]
     OnCalendar=hourly
     Persistent=true
     RandomizedDelaySec=120

     [Install]
     WantedBy=timers.target
     ```

3.  Run `systemctl --user daemon-reload`.
     Expected:
     no output.
4.  Run `systemctl --user enable --now rclone-bisync-garage.timer`.
     Expected:
    `Created symlink ... timers.target.wants/rclone-bisync-garage.timer`.
5.  Run `loginctl enable-linger $(id -un)` so the timer fires while logged out.
    Expected:
     `loginctl show-user $(id -un) -p Linger` then prints `Linger=yes`.
6.  Trigger one run:
     `systemctl --user start rclone-bisync-garage.service`.
    Expected:
     the run finishes and the journal shows `Bisync successful`.

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
         <key>ProgramArguments</key>
         <array>
             <string>/opt/homebrew/bin/rclone</string>
             <string>bisync</string>
             <string>garage:files/Plain</string>
             <string>/Users/USERNAME/Plain</string>
             <string>--config</string>
             <string>/Users/USERNAME/.config/rclone/rclone.conf</string>
             <string>--filter-from</string>
             <string>/Users/USERNAME/.config/rclone/garage-filter.txt</string>
             <string>--fast-list</string>
             <string>--use-server-modtime</string>
             <string>--transfers</string>
             <string>16</string>
             <string>--checkers</string>
             <string>32</string>
             <string>--s3-upload-concurrency</string>
             <string>8</string>
             <string>--resilient</string>
             <string>--recover</string>
             <string>-v</string>
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
     `~/Library/Logs/rclone-bisync-garage.log` ends with `Bisync successful`.

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
  `Bisync successful`.
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
  timestamp.
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
