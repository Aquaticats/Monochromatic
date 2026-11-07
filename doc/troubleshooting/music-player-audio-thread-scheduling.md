# Music player 0.1.13: the Linux audio callback has realtime scheduling

## Scheduling check at 2026-11-07T05:10:54-05:00

The running Linux player has a realtime audio data loop.
The requested condition for opening a GitHub issue,
an audio thread without realtime scheduling,
was not observed,
so no issue was opened.

## Why thread names can mislead

The player process has a thread named `music-player-au`,
the truncated name of its `music-player-audio` control loop.
That thread uses `SCHED_OTHER`.
The audio output node instead names `data-loop.0` as its loop,
and that thread uses `SCHED_RR` with priority `20`.

The current checkout explicitly requests the realtime processing path in
`package/music-player/desktop-app/src/output_pipewire.rs:1427`:

```rust
// package/music-player/desktop-app/src/output_pipewire.rs
stream
    .connect(
        Direction::Output,
        None,
        StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
        &mut params,
    )
```

Source inspected at `687074ecf2cb5ccec0369c885f2b3ba0c21bbf5e`.
The installed executable reports `music-player 0.1.13`;
its build commit was not established.
The runtime scheduling observation stands separately from the source inspection.

## Verification

Read-only commands against the existing player process:

```bash
# Live process and thread identifiers from this observation, not stable identifiers.
ps -L -p 1805131 -o pid,tid,comm,cls,rtprio,ni,stat
chrt --pid 2180568
chrt --pid 1805294
```

The data loop returned:

```text
# chrt --pid 2180568
pid 2180568's current scheduling policy: SCHED_RR|SCHED_RESET_ON_FORK
pid 2180568's current scheduling priority: 20
```

The control loop returned `SCHED_OTHER` with priority `0`.
These different results show that the probe distinguishes ordinary and realtime scheduling.
`/proc/1805131/task/2180568/comm` returned `data-loop.0`,
and its `sched` file independently reported policy `2`.

`pw-dump --no-colors` identified client `73` as process `1805131`,
application `music-player`.
Audio output node `87` belonged to that client,
had state `running`,
media class `Stream/Output/Audio`,
and `node.loop.name` equal to `data-loop.0`.
The inspection did not change playback or scheduling.

## Limits and workarounds

No workaround was needed or tested.
Looking only at the process leader or the thread named `music-player-audio`
does not describe the measured audio data loop.
This check covers the running Linux PipeWire output;
Android,
Windows,
macOS,
and the ALSA fallback were not exercised.
Realtime scheduling alone does not prove that every callback operation is safe or that underruns cannot occur.

## Issue disposition

No new filing or additive comment is warranted by this scheduling check.
The repository search for `"realtime" "music"` found existing callback-allocation reports
[#5171](https://github.com/Aquaticats/Monochromatic/issues/5171) and
[#3662](https://github.com/Aquaticats/Monochromatic/issues/3662).
Those concern work performed inside the callback,
a separate question from its scheduling policy.
The search for `"audio" "priority"` found closed decoder-priority and CI-session reports,
not a failure in the running audio data loop.

No upstream defect was established.
Upstream fix feasibility,
contribution policy,
willingness,
and prototype requirements therefore do not apply;
no upstream filing artifact or fix is proposed.
