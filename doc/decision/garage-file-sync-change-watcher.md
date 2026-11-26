# The Garage sync's change watcher: directory-watcher 3.2.1 on this PC

## Status

Adopted by the owner;
live on this CachyOS PC since 2026-11-25T22:01:09-05:00.
Not final: the first change-triggered sync of a real save had not happened when this was written (2026-11-25T22:02:52-05:00),
and the macOS and Windows clients have no watcher yet.

## Adopted

directory-watcher 3.2.1:
repository `seungyeop-lee/directory-watcher`, tag `v3.2.1`, commit `7286f534dbeb3854607b3a48875c4cf0bbb5e672`;
release archive `directory-watcher_Linux_x86_64.tar.gz`, sha256 `eb07d000d21069f528511a16d56ee529542f70daa1da527016cf74f8d9ba437a`
(`toolchain.lock.json`, commit `d78abba1c1`);
installed binary `~/.local/share/monochromatic-dev/toolchain/bin/directory-watcher`, sha256 `9b1fd474ae39e3dcd50a5a125335ffaa6d779c1495df05f82f97bee8a726fcdd`.

## Who decided, and when

- The request: "Make my Garage setup trigger a manual sync whenever something changes on relevant dirs."
- Round 7 of the design (2026-11-25T19:51:58-05:00): "Adopt it".
- Round 8 (2026-11-25T20:41:14-05:00): "Keep directory-watcher",
  accepting that names starting with `.` or `_` are not watched.
- The design was confirmed at 2026-11-25T20:54:22-05:00 ("Yes, confirmed")
  and built on the request "Do it."

The design and every owner answer are in [`doc/planning/garage-bisync-run-on-file-change.md`](../planning/garage-bisync-run-on-file-change.md).

## Evidence

- Vet report: [`doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`](../audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md)
  (last changed in `5b00a650fd`),
  governed by `.agents/skills/choosing-technology/SKILL.md` at `64138ba3ba`.
- Runtime validation of the binary: that report's `targeted-evidence/directory-watcher-runtime-validation.md`.
- The unit stack on this PC's user manager: the design record's section "Proof on this PC's user manager".

## Hard constraints and weights

- Open source with inspectable source.
- Runs an arbitrary command.
- Runs on Linux x86-64, macOS arm64 and Windows x64.
- Watches every folder under the root recursively, including folders made after it starts;
  the owner accepted one exception: names starting with `.` or `_`, which directory-watcher never watches
  (its `docs/guide/events.md:82` and `:108` at `v3.2.1`); they reach the bucket on the five-minute timer.
- A change that lands during a sync gets one more sync after that run ends.
- Never interrupts a running command.
- Queues one more run natively.

No weights were frozen: one finalist survived, so no weight could change an order.

## Ranking and rejected alternatives

1.  directory-watcher 3.2.1: the only candidate confirmed on every hard gate.
2.  onchange 7.1.0: exited by the owner (2026-11-25T18:51:49-05:00) on the intent of the follow-up requirement:
    it queues one job per event, unbounded and never merged, with no quiet period.
3.  modd 0.8: exited on provenance (no release workflow, checksum or signature ties its archives to the tag);
    building it from source was declined.
4.  Watchman 2026.11.23.00: exited on three hard constraints;
    it kills the running trigger command before starting the next one (`watchman/TriggerCommand.cpp`).
5.  watchexec: excluded by the owner (2026-11-25T18:18:47-05:00), "Vet 3p, no watchexec",
    with the repository's earlier rejection recorded.
6.  The repository's own `package/watch-restart/cli`: the fallback only if nothing else works,
    because, in the owner's words, "watch-restart has its own critical problems we'd need to fix if we use it".

The Sonnet gate review noted that rounds 4 to 6 judged queueing and interruption under a blocking trigger,
and that with the adopted `--no-block` trigger those two constraints no longer separate candidates on Linux;
the owner kept directory-watcher with that stated (round 8).

## Integration boundary

[`package/garage-file-sync/linux-client`](../../package/garage-file-sync/linux-client/README.md) owns every file.
The watcher unit `rclone-bisync-garage-watch.service` runs the binary by absolute path with
`--config-path ~/.config/directory-watcher/garage-plain.yml`;
its only change hook is `systemctl --user start --no-block rclone-bisync-garage-after-current.service`,
so directory-watcher never runs rclone and never waits on a sync.
systemd merges the trigger's requests into at most one follow-up run.

## Migration

The hourly timer became a five-minute timer, the sync unit gained a second pass,
and the alert script moved from `~/.local/bin/rclone-bisync-garage-alert.ts` into the package.
The files they replaced are kept in `~/temp/agent/garage-live-units-before-sync-on-change-2026-11-25/`.

## Exit and rollback

1.  `systemctl --user disable --now rclone-bisync-garage-watch.service` stops change-triggered syncs;
    the five-minute timer keeps syncing.
2.  For the hourly setup, copy the four files from the backup folder over
    `~/.config/systemd/user/`, put `rclone-bisync-garage-alert.ts` back in `~/.local/bin/`,
    remove `rclone-bisync-garage-after-current.service`, `rclone-bisync-garage-recovered@.service` and the watch unit,
    then `systemctl --user daemon-reload` and `systemctl --user restart rclone-bisync-garage.timer`.

## Revisit when

- `toolchain:update` moves directory-watcher past 3.2.1:
  rerun the runtime validation, the dot-name check included, before installing it.
- The macOS or Windows client gets a watcher:
  there the follow-up relies on the watcher's own queue, not systemd's merge,
  and macOS's kqueue needs about 11786 descriptors for this tree against launchd's default soft limit of 256.
- A file under a `.` or `_` name needs to sync sooner than the timer.
- The watcher fails at its inotify watch limit: it exits without its own notice, and the unit's failure alert is the only signal.
