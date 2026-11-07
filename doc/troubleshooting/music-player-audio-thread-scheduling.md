# Music player audio callback scheduling

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

## Linux issue disposition

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

## Cross-platform check at 2026-11-07T07:38:45-05:00

The follow-up asked about Windows,
macOS,
and Android.
These are source and configuration checks;
none of these platforms received a native runtime scheduling measurement in this session.
The configured Windows and Mac SSH probes returned `Could not resolve hostname`.
`adb devices -l` returned an empty device list.

### Windows: priority promotion is omitted from the build

Filed [#6263](https://github.com/Aquaticats/Monochromatic/issues/6263).
The issue records the evidence and native Windows completion checks;
no implementation change was made.

At repository commit `2b0c79cec67ee9c6554a03ade441488cf6372a76`,
`package/music-player/desktop-app/Cargo.toml:180` declares:

```toml
# package/music-player/desktop-app/Cargo.toml
[target.'cfg(not(target_os = "linux"))'.dependencies]
cpal = { workspace = true }
```

The workspace declaration supplies the version without features.
`Cargo.lock:1192` resolves CPAL `0.18.2`,
checksum `6f02e8d0327b42d3e2e4ab2119af397344eb9fc54a34bf0ddeaa1277af8681f1`.
Its dependency entry omits `audio_thread_priority`.
The manifest,
release workflow,
and desktop source contain no request for CPAL's `realtime` feature.

The installed crate and cloned tag `v0.18.2` identify commit
`e1612d5d98152f8dc2a62e1b51ef7cbf4f7f26b7`.
Its [feature declarations][cpal-features] have:

```toml
# CPAL Cargo.toml:17 and :22, intervening comments omitted.
default = []
realtime = ["dep:audio_thread_priority", "dep:alsa-sys"]
```

The [WASAPI output callback loop][cpal-priority] conditionally requests promotion:

```rust
// CPAL src/host/wasapi/stream.rs:661
#[cfg(feature = "realtime")]
if let Err(err) = boost_current_thread_priority(
    run_ctxt.stream.period_frames,
    run_ctxt.stream.config.sample_rate,
) {
    emit_error(error_callback, err);
}
```

That call is excluded by our feature selection.
This establishes a missing priority request,
not a measured Windows scheduling class or an observed underrun.
The same source selection includes the helper's realtime-promotion attempt and high-priority fallback.
Both require the feature.

Open and closed issue searches for `"cpal" "realtime"`,
`"WASAPI" "priority"`,
and the broader `"cpal"` found no matching report before filing.
This is our configuration issue;
CPAL documents the opt-in,
so no upstream defect or upstream patch is proposed.

### macOS: CoreAudio supplies the realtime render thread

Our `package/music-player/desktop-app/src/output_cpal.rs:586` calls `build_output_stream`.
CPAL's [macOS implementation][cpal-coreaudio] installs a CoreAudio render callback:

```rust
// CPAL src/host/coreaudio/macos/device.rs:888
audio_unit.set_render_callback(move |args: Args| unsafe {
```

The callback invokes `data_callback(&mut data, &info)` at line 927.
[Apple's audio-workgroup documentation][apple-workgroups] states:
"For every audio server I/O thread, its associated app also has its own real-time render thread."
It also says an app needs no additional work when it uses the realtime thread supplied by the audio frameworks.

No missing application-side scheduling opt-in was found for this path.
This source-supported conclusion does not substitute for sampling the running player on a Mac.

### Android: realtime-capable callback path requested, actual policy unmeasured

The production stream builder is in `package/music-player/android-app/rust/src/engine_worker.rs`,
not the standalone latency probe in `rust/src/output.rs`.
At line 1503 the production builder requests:

```rust
// package/music-player/android-app/rust/src/engine_worker.rs:1503
.performance_mode(AudioPerformanceMode::LowLatency)
```

At line 1526 it registers `.data_callback(Box::new(`,
delegating audio rendering to AAudio.
The [AAudio documentation][aaudio-callback] says it executes this callback in a higher-priority thread.

The Android framework source inspected in the LineageOS mirror is commit
`1c5914737a1a89d0c5b3e52e8289019eab3f8cbf`,
not a verified match to a connected device build.
Its [AAudio service registration][aaudio-priority] requests priority for the callback thread:

```cpp
// services/oboeservice/AAudioServiceStreamBase.cpp:951
int err = android::requestPriority(ownerPid, clientThreadId,
                                   priority, true /* isForApp */);
```

The [legacy output path][aaudio-fallback] derives the actual performance mode from granted stream flags,
starting with:

```cpp
// media/libaaudio/src/legacy/AudioStreamTrack.cpp:290
aaudio_performance_mode_t actualPerformanceMode = AAUDIO_PERFORMANCE_MODE_NONE;
```

It sets `LOW_LATENCY` only when the returned flags contain `AUDIO_OUTPUT_FLAG_FAST`.
Therefore the application's low-latency request is not proof that every device or output route grants it.
No Android scheduling failure was observed and no Android issue was opened.
Actual callback policy and granted performance mode remain unmeasured.

### Verification boundary

External source selections and documentation passages were opened in `agent-browser`
and their quoted text was checked after rendering.
GitHub source line containers initially existed with empty text;
waiting for the expected source text avoided treating that intermediate state as a missing match.
The issue body passed a GitHub Markdown rendering check.
No playback settings,
installed application state,
or thread priorities were changed.

[cpal-features]: https://github.com/RustAudio/cpal/blob/e1612d5d98152f8dc2a62e1b51ef7cbf4f7f26b7/Cargo.toml#L15-L27
[cpal-priority]: https://github.com/RustAudio/cpal/blob/e1612d5d98152f8dc2a62e1b51ef7cbf4f7f26b7/src/host/wasapi/stream.rs#L655-L728
[cpal-coreaudio]: https://github.com/RustAudio/cpal/blob/e1612d5d98152f8dc2a62e1b51ef7cbf4f7f26b7/src/host/coreaudio/macos/device.rs#L886-L930
[apple-workgroups]: https://developer.apple.com/documentation/audiotoolbox/understanding-audio-workgroups#:~:text=For%20every%20audio%20server%20I%2FO%20thread
[aaudio-callback]: https://developer.android.com/ndk/guides/audio/aaudio/aaudio#:~:text=AAudio%20executes%20the%20callback%20in%20a%20higher-priority%20thread
[aaudio-priority]: https://github.com/LineageOS/android_frameworks_av/blob/1c5914737a1a89d0c5b3e52e8289019eab3f8cbf/services/oboeservice/AAudioServiceStreamBase.cpp#L943-L960
[aaudio-fallback]: https://github.com/LineageOS/android_frameworks_av/blob/1c5914737a1a89d0c5b3e52e8289019eab3f8cbf/media/libaaudio/src/legacy/AudioStreamTrack.cpp#L286-L299
