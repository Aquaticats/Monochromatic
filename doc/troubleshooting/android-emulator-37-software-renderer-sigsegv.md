# Android Emulator 37.1.11 on Linux crashes during API 37 headless boot when software rendering is selected

Tool under test:
Android Emulator 37.1.11.0,
build `15917651`,
on Fedora 44.
Surface trigger:
start an API 37 x86_64 AVD headlessly with `-gpu swiftshader_indirect` or `-gpu off`.
Failure mode:
`qemu-system-x86_64-headless` receives `SIGSEGV` before Android finishes booting.

## Rule for agents: headful only (2026-10-13)

Never launch an emulator with `-no-window`, under `xvfb-run`, or in a container, and never let a brief permit it.
Headless launches broke repeatedly: a container launch hung for 2.8 hours, two container boots on 2026-10-13 hung on
software rendering, and the human reports that headless emulators break frequently.
The only launch form for agent sessions is the one in "Working agent launch: headful, host GPU, no container":
a window on the host GPU, as its own systemd user unit with `MemoryMax`.
If that launch fails, stop and report the error; do not fall back to a headless form.
The headless commands below are the evidence of the original crash and stay as a record, not as instructions.

## Symptom

The process reaches the full-startup phase,
never changes from `adb` state `offline` to `device`,
and exits on signal 11.
Pi's process supervisor reported:

```text
Process "android-issue-460-pixel9-disposable-avd" ended after receiving SIGSEGV.
```

`coredumpctl info <pid>` identifies the exact emitter and signal:

```text
Executable: .../emulator/qemu/linux-x86_64/qemu-system-x86_64-headless
Signal: 11 (SEGV)
Message: Process ... (qemu-system-x86) of user 1000 dumped core.
```

The crashing thread enters an invalid executable address from the packaged SwiftShader GLES worker:

```text
Thread 1:
#0  0x000055eb4c48f070 in ?? ()
#1  .../emulator/lib64/gles_swiftshader/libGLESv2.so
#2  .../emulator/lib64/gles_swiftshader/libGLESv2.so
#3  .../emulator/lib64/gles_swiftshader/libGLESv2.so
```

Two alarming startup messages are not sufficient explanations:

```text
ERROR | Setting read-only feature 'GLAsyncSwap' to '0'
pc_memory_init: above 4g size: 40000000
```

The successful `-gpu host` positive control also printed the `pc_memory_init` line,
so that line does not distinguish the crash.
The `GLAsyncSwap` diagnostic appeared on software-renderer runs,
but the stripped build gives no evidence that this feature override caused the invalid call.

## Root cause

The observed boundary is the packaged software GLES renderer.
The exact defective SwiftShader routine is unresolved because Android Emulator 37.1.11 ships a stripped
`qemu-system-x86_64-headless`,
and its package metadata names only build `15917651`,
not a public source commit.

The closest public emulator source inspected was
`aosp-mirror/platform_external_qemu` branch `emu-master-dev` at
`ae9d18d2b6261179fbd57fffec720a04f7bfb053`.
It predates the installed build,
so it establishes the renderer-selection call chain,
not line-for-line identity with 37.1.11.

The GPU setup chooses a software Vulkan implementation when host Vulkan is not selected.
`android/android-ui/modules/aemu-gl-init/src/android/opengl/emugl_config.cpp:1060-1079` says:

```cpp
if (config->use_host_vulkan) {
    system->envSet("ANDROID_EMU_VK_ICD", NULL);
} else if (sGpuOption == "lavapipe") {
    system->envSet("ANDROID_EMU_VK_ICD", "lavapipe");
} else {
    system->envSet("ANDROID_EMU_VK_ICD", "swiftshader");
}
```

The same file adds a non-host backend directory to the dynamic-library search path,
while host mode returns without adding one.
`android/android-ui/modules/aemu-gl-init/src/android/opengl/emugl_config.cpp:1095-1112` says:

```cpp
if (strcmp(config->backend, "host") != 0) {
    std::string dir = sBackendList->getLibDirPath(
            use_swangle ? "angle" : config->backend);
    system->addLibrarySearchDir(dir);
}

if (!strcmp(config->backend, "host")) {
    return;
}
```

The selected renderer then supplies the GLES dispatch table.
`android/android-emu/android/opengles.cpp:258-265` and
`android/android-emu/android/opengles.cpp:409-424` say:

```cpp
sRenderLib->setRenderer(emuglConfig_get_current_renderer());

sRenderer = sRenderLib->initRenderer(width, height, gfxstreamFeatures,
                                     sRendererUsesSubWindow, sEgl2egl);
sGlesv2 = (const GLESv2Dispatch*)sRenderer->getGles2Dispatch();
```

The coredump places the faulting worker in `gles_swiftshader/libGLESv2.so`,
which matches this software-backend path.
The invalid program counter is outside the stripped emulator text addresses and is consistent with generated code,
but it is not proof of a specific JIT defect.
Current SwiftShader source at
`google/swiftshader` commit `694585a05946e1ed49b6bd577ca6537cbb57f025`
contains its JIT implementation under `src/Reactor/LLVMJIT.cpp` and executable-memory implementation under
`src/Reactor/ExecutableMemory.cpp`.
No source-to-binary revision mapping was available,
so attribution stops at the packaged SwiftShader GLES boundary.

## Verification

Installed version:

```text
Pkg.Revision=37.1.11
Pkg.BuildId=15917651
```

The release is Android Emulator 37.1.11 Stable from July 30,
 2026,
according to the [official release notes][emulator-releases].

### Failing catalog

Each software-renderer run reached `offline`,
then exited with `SIGSEGV` before `sys.boot_completed` became `1`.
The Pixel 9 Pro Fold command was retried after the user approved the emulator's OpenSnitch request;
it still produced the same `SIGSEGV`:

```bash
emulator -avd Pixel_9_Pro_Fold \
  -read-only -no-snapshot -no-window -no-audio -no-boot-anim \
  -gpu swiftshader_indirect
```

```bash
emulator -avd Pixel_9_Pro_Fold \
  -read-only -no-snapshot -no-window -no-audio -no-boot-anim \
  -gpu off -camera-back none -camera-front none
```

A fresh Pixel 9 AVD under a disposable `ANDROID_AVD_HOME` failed the same way:

```bash
ANDROID_AVD_HOME="$HOME/temp/agent/issue460-avd-home" \
  emulator -avd Issue460Pixel9 \
  -wipe-data -no-cache -no-snapshot -no-window -no-audio -no-boot-anim \
  -gpu swiftshader_indirect -camera-back none -camera-front none
```

On 2026-09-23,
 the same installed Fold AVD also crashed on SIGSEGV with the emulator's
implicit software renderer when started as follows:

```bash
emulator -avd Pixel_9_Pro_Fold -no-window -no-audio
```

The process logged `Graphics Adapter Android Emulator OpenGL ES Translator (Google SwiftShader)`;
`coredumpctl info 3324401` recorded signal 11 from `qemu-system-x86_64-headless` before
`adb` became ready.
 This additional invocation supports the failing catalog but does
not isolate `-no-window` itself as the cause.

### Working catalog

The installed Pixel 9 Pro Fold AVD booted read-only with host GPU rendering:

```bash
emulator -avd Pixel_9_Pro_Fold \
  -read-only -no-snapshot -no-window -no-audio -no-boot-anim \
  -gpu host -camera-back none -camera-front none
```

A disposable Pixel 9 AVD also booted with host GPU rendering:

```bash
ANDROID_AVD_HOME="$HOME/temp/agent/issue460-avd-home" \
  emulator -avd Issue460Pixel9 \
  -wipe-data -no-cache -no-snapshot -no-window -no-audio -no-boot-anim \
  -gpu host -camera-back none -camera-front none
```

The end-user probe returned:

```text
emulator-5554 device product:sdk_gphone16k_x86_64
boot_completed=1
```

On 2026-09-23,
 the existing Fold AVD also reached `sys.boot_completed=1` and exposed
`cmd device_state` with a windowed host-GPU invocation:

```bash
emulator -avd Pixel_9_Pro_Fold -no-audio -gpu host -no-snapshot-load
```

This comparison changes renderer and window mode together;
 the prior read-only
headless `-gpu host` positive control showed that a window is not required for host-GPU success on this host.
That is a fact about the crash only: agents still launch headful (see "Rule for agents").
 The music-player APK and test APK installed on that emulator.
The real Android instrumentation runner then executed the migration suite:

```text
dev.monochromatic.musicplayer.SessionStoreTest:..

Time: 0.008

OK (2 tests)
```

## Working agent launch: headful, host GPU, no container (2026-10-12)

An agent session drove the owned Pixel 9 Pro Fold AVD for hours with a headless container launch that passed
`-feature -Vulkan` under `xvfb-run` (software rendering, 2 CPUs and 6 GiB for a 4 GiB guest) and met repeated System UI
and Pixel Launcher ANRs, a `qemu` segmentation fault and an ANR loop.
The same AVD booted cleanly when launched headfully as its own systemd user unit, on the host GPU:

```bash
systemd-run --user --unit=agent-fold-headful --collect --property=MemoryMax=12G \
  --setenv=DISPLAY=:0 --setenv=WAYLAND_DISPLAY=wayland-0 --setenv=XDG_RUNTIME_DIR=/run/user/1000 \
  --setenv=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus --setenv=ANDROID_HOME=/home/user/Android/Sdk \
  --setenv=ANDROID_EMULATOR_WAIT_TIME_BEFORE_KILL=90 \
  /home/user/Android/Sdk/emulator/emulator -avd Pixel9ProFold_Fresh_5FXdOc -ports 5582,5583 \
  -no-snapshot -no-audio -no-boot-anim -gpu host -camera-back none -camera-front none -memory 4096 -cores 4 -no-metrics
```

(The owned AVD also needs its own `HOME`, `ANDROID_USER_HOME` and `ANDROID_AVD_HOME`, and a private adb server port, as
in the agent's notes.
Every `adb` call for that emulator, including `emu kill`, must run with the same `HOME` and port:
in one session the shell's real `HOME` showed the device as `unauthorized` and failed the console token check,
see `android-emulator-console-token-container-home.md`.)
Result: `sys.boot_completed=1` after 98 s, no ANR dialog and no crash, with the journal naming the AMD Radeon RX 7600
as the GL and Vulkan device, at a load average above 60 on 16 cores.
So the failures were launch flags and container limits, not host load.
Use this form for agent sessions: no `-no-window`, no `xvfb-run`, no container, Vulkan left at its default.
The window opens on the human's session; close it with `adb emu kill` after `sync`.

## Separate incident: SIGABRT in a gRPC callback of the headful launch (2026-10-14)

This happened with the working headful launch form above, so it is not the SwiftShader boot crash and not the llvmpipe
crash below.
It is recorded as a separate incident: the cause is not shown.

- Emulator 37.2.12, the owned Pixel 9 Pro Fold AVD copy, ports 5772 and 5773, `-gpu host`, as its own systemd user unit
  with `MemoryMax=12G`.
  The guest booted (about 35 s), a 35-track library was pushed, an APK was installed and the demo-mode broadcasts were
  sent; the emulator process then aborted about a minute later.
- `coredumpctl info` names the emitter and signal: `qemu-system-x86_64`, signal 6 (ABRT), raised by
  `__libcpp_verbose_abort` from a frame of the emulator binary inside `grpc::internal::CallbackWithSuccessTag::Proceed`,
  run by a gRPC event-engine worker thread of `libandroid-webrtc.so`.
- The journal line just before it, at the second the core was dumped, was
  `Netsim Wifi ipv6:... is gone due to CANCELLED` followed by `Got goaway [2] ... Cancelling all calls`.
  Another agent's emulator unit (`-ports 5592,5593`) was running when this one started and was gone when the
  relaunch was prepared, so a shared netsim daemon is one candidate; nothing here shows it.
- Workaround that held: a fresh reflink copy of the AVD and the same launch form booted again and ran the rest of the
  scripted captures with no abort; the session ended with `sync`, a wait of 8 s, `sync` and `adb emu kill`.
- Not done: no reproduction on purpose, no `-no-netsim` or other flag was tried, because the launch rule allows one
  form only.

## Separate incident: crash consent dialog blocks the next headful launch (2026-10-18)

Three headful launches (the working form above, a private reflink copy of the owned AVD, `-gpu host`) in one session:
the first crashed with SIGSEGV in `gfxstream::host::gl::TextureResize` right after boot,
the second aborted (SIGABRT, `grpc::internal::CallbackWithSuccessTag::Proceed`, the frame of the 2026-10-14 incident),
and the third printed `Showing crashdialog to get consent.` at start, then `Vulkan emulation initialized`,
and stayed alive for 15 minutes with no console port and no device in `adb devices`.
The two earlier reports sit in `/tmp/android-user/emu-crash-37.2.12.db`, and the emulator asks for consent before it
boots; the dialog is on the host session, which an agent cannot answer.
Not tested: whether `-no-metrics` is ignored by that dialog, or whether removing the report database avoids it.
The third launch was stopped with `systemctl --user stop` (the guest had not booted).

## Separate incident: mid-session SIGSEGV under llvmpipe in a container (2026-10-05)

This is recorded here as the nearest topic.
It is not shown to be the SwiftShader boot crash:
the emulator version,
renderer and timing all differ.

Android Emulator 37.2.12.0 (build_id 16428233) ran an owned Pixel 9 Pro Fold
AVD copy headlessly under Xvfb in `podman run --memory=6g --cpus=2`,
with `-gpu host -feature -Vulkan -memory 4096 -cores 2`.
Startup reported:

```text
# Android Emulator startup diagnostic
ERROR        | Your GPU cannot be used for hardware rendering. Consider using software rendering.
INFO         | Graphics Adapter Android Emulator OpenGL ES Translator (llvmpipe (LLVM 20.1.2, 256 bits))
```

The guest booted and worked for about 16 minutes,
through 24 screenshot captures,
then the emulator crashed during scripted touch input.
The container's shell printed `Segmentation fault (core dumped)` and the
emulator command returned `139`.
`coredumpctl info` names the emitter and signal:

```text
# coredumpctl info 2301587
Executable: .../emulator/qemu/linux-x86_64/qemu-system-x86_64-headless
Signal: 11 (SEGV)
Stack trace of thread 229:
#0  0x00007fc75dd89adc n/a (/usr/lib/x86_64-linux-gnu/libc.so.6 + 0x19badc)
```

Only that frame was resolved,
so the faulting emulator code is not identified.
The owner log's last lines before the fault were repeated
`gles_v2_imp.cpp:... error null ctx` messages from the GL translator.
An earlier visit with the same command that completed normally logged the
same message,
so those lines do not distinguish the crash.
Host load averaged about 40 to 50 at the time;
no out-of-memory event was looked for.

The same container command completed whole visits before and after this
crash,
so it is intermittent on this host.
No workaround was verified;
the visit was repeated.
A crash leaves the guest's changed settings and the AVD's lock in place;
see [the lock recurrence](android-emulator-37-disposable-avd-lock-after-hard-stop.md).

### Second occurrence the same day

The same container command crashed again later on 2026-10-05,
about a quarter of an hour after its start,
after twelve first-run study states had been captured.
The emulator's output again ended with `Segmentation fault (core dumped)`
and the owner exited with status 1.
`coredumpctl list --since 19:00` reported `No coredumps found`,
so no stack was read this time.
The host's one-minute load average had risen from under 1 to between 70
and 96 during the visit,
driven by other work on the host.
That both crashes came under heavy load is an observation,
not an established cause.

The next boot's first reading of `font_scale` was `2.0`,
the value the capture had set before the crash,
so a crash leaves the last written guest setting on disk.
The remedy that was used is not a fix for the crash:
the capture now keeps each finished state in one cohort folder and a later
boot continues from the first missing state,
with each view naming the boot that made it.

### Third occurrence the same day

A third crash followed that evening,
a few minutes into a visit,
after five views of another study.
Three of those views had the system keyboard open,
which the earlier crashed visits never did.
The emulator's output again ended with `Segmentation fault (core dumped)`.
The host's load average was between 21 and 30 this time,
well under the earlier two,
so heavy load is not required for the crash.
Whether the keyboard matters is not known from one case.

After this one,
visits are repeated by a bounded driver until the cohort is complete:
at most eight visits,
each in the same 6 GiB,
2 CPU container,
stopping early if two visits in a row add no view.
After each crash it checks for owners and moves the stale locks to a backup,
as [the lock record](android-emulator-37-disposable-avd-lock-after-hard-stop.md)
prescribes.

### Fourth and fifth occurrences the same day

Two more visits under that driver ended the same way,
each with `Segmentation fault (core dumped)` as the emulator's last line:
one after 19 views,
with a load average near 27 read a few minutes after it,
and one after 18 views,
with a load average of 53 read a few minutes after it.
During the second of these,
the agent was also running a browser check in a separate 2 CPU container.
Between them,
one visit captured 32 views and shut down without a crash.

So on this host the crash came after 5,
19 and 18 views in three visits and not at all in a fourth of 32 views.
That is too few visits to say whether load,
the keyboard or the number of views decides it.
The driver's answer stays the same:
keep every finished view,
record the crash,
move the stale locks after the owner checks,
and boot again.

### A visit that was killed, not a segmentation fault

Later that night a visit ended differently.
After 59 views in about 25 minutes,
the emulator's guest process was `Killed` and its command returned `137`.
The output has no segmentation fault line.
`podman events` for that container reports its end and no out-of-memory event,
and the kernel log could not be read without privileges.
So the cause is not established:
the 6 GiB bound of the container is a candidate and nothing more.
The stale locks it left were moved the same way and the next visit booted.
It is counted here because it interrupts a capture exactly as the crash does,
and must not be reported as one.

### After a reboot of the host

On 2026-10-06,
after the host had been restarted,
the first visit of a new capture ended with `Segmentation fault (core dumped)` after 25 views.
The host's load average read about 35 to 60 during that hour,
from other work on the machine.
The next visit lost no view to a crash but ended when one guest command exceeded its 60 second bound,
and the third completed the cohort and shut down cleanly.
A fresh boot of the host therefore did not stop the crash.

## Separate incident: headful host-GPU SEGV in gfxstream TextureResize (2026-10-13)

This is not the SwiftShader boot crash above: the launch was the recommended one in "Working agent launch: headful,
host GPU, no container" (`-gpu host`, own systemd unit, AMD Radeon RX 7600 through RADV), and the guest had booted.

Two agent runs on the same invented 10 000-track library each lost the emulator to `SIGSEGV` in
`qemu-system-x86_64` during long runs.
The first came at 13:46:59 during the third play-window run (`lf-emulator.service` exited with
`code=dumped, status=11/SEGV`); the second at 17:50:44, on the first cold start after a clean reinstall, about 53
minutes after the launch (core of 1.3 GB; its `coredumpctl info` is in the agent scratch
`lf/logs/crash2-coredumpctl-info.txt`).
The crashing thread in both was in `libgfxstream_backend.so`: `gfxstream::host::gl::TextureResize::TextureResize`
called from `ColorBufferGl::create` (first crash) and `EmulationGl::createColorBuffer` (second); in the first the top
frame was `__strlen_evex`.
Earlier on 2026-10-13 an Android gallery run saw one similar crash after its captures (a null-context SIGSEGV in the
same backend).
A fourth came on 2026-10-13 at 21:40:18 (local time, 01:40 UTC on the 14th), 24 minutes 39 seconds after the launch,
on the same 10 000-track library, during the cold start of the second trial of a play-window run; the stack was the same
(`__strlen_evex` under `TextureResize::TextureResize` under `EmulationGl::createColorBuffer`), the core 1.2 GB, and the
trial that was running was lost (its child hit the 20 minute timeout).
The two later launches from fresh copies ran 70 and 28 minutes without a crash.
A fifth came on 2026-10-14 at 08:30:17 (local time), 10 minutes after the launch
and 2 minutes into the first run on that
copy (the `pm clear`, install and cold start of a second variant); the stack was again `__strlen_evex` under
`TextureResize::TextureResize` under `EmulationGl::createColorBuffer`, the core 1.3 GB, and the journal showed
`glGenTextures ... error null ctx` lines from the backend two minutes earlier.
The relaunch from a fresh copy then ran about 1 hour 40 minutes (a 10 003-file push, 17 interleaved play windows, four
simpleperf runs and the controller checks) without a crash and shut down gracefully, and a third launch from another
fresh copy (three invented tones, transport checks) also ended without a crash.

A sixth and a seventh came on 2026-10-15 during the release-build measurements
(`doc/audit/music-player-release-build-measurements-2026-10-15.md`), from the same headful host-GPU launch
(own `systemd-run --user` unit, `MemoryMax=12G`, reflink copy of the owned AVD):

- `coredumpctl` entry 2569769, unit `relmeasure-emulator-1.service`, 13:02:34 local time, 1 hour 3 minutes 54 seconds
  after the launch, core 1.6 GB.
  The crash came during the 10 000-track scroll runs (the `dumpsys gfxinfo` windows), right after the first deep
  window of that size.
- `coredumpctl` entry 302782, unit `relmeasure-emulator-2.service`, 13:24:05 local time, 19 minutes 49 seconds after
  the launch, core 1.5 GB, again during the 10 000-track scroll runs.

Both stacks are the same as the fourth and fifth: `__strlen_evex` under
`gfxstream::host::gl::TextureResize::TextureResize` under `ColorBufferGl::create` under
`EmulationGl::createColorBuffer`.
The journal carried `glestranslator ... error null ctx` lines before each crash, as before the fifth.
The third launch, which ran only 1 000-track scroll windows and idle counts, ran to a graceful shutdown.
The driver then failed with `no scrollable node found` because the guest was gone, which is how the crash showed up
in the run.
Each was recovered by a fresh reflink copy and relaunch, at most twice as the task allowed, and the scroll windows
that ran in the two crashed sessions are kept in the audit with the crashes noted as a confound.

Two facts about the owned AVD copy found in the same session: the sdcard of the owned AVD already holds
`/sdcard/Music/RealAppTest` (private names), so delete it before pushing a library and before the volume scan, not
after; and on this emulator `content query ... --projection` takes columns separated by a colon (`_data:duration`), a
comma is rejected with `Invalid column`.

Recovery that worked: discard that AVD copy, make a fresh reflink copy of the owned AVD, relaunch headful, reinstall and
re-push the invented library.
The guest is already dead when the crash is seen, so `sync`, `adb emu kill` and the settings restore have nothing to
act on; compare settings only on the next boot.
Measurements in flight are lost, so run each scenario trial from a clean reinstall and redo the trial that was running.

Not established: what triggers it (the common point is a color buffer allocation in the backend),
whether another emulator build or `-gpu` mode avoids it, and whether it is related to the host load (20 to 36 during
those runs).
Do not fall back to a headless launch: it is banned (see "Rule for agents").

## Verified workarounds

Use `-gpu host` for this Linux host.
This bypassed the packaged SwiftShader GLES worker and booted both the installed Pixel 9 Pro Fold AVD and a
disposable Pixel 9 AVD from the installed API 37 x86_64 system image.

Tradeoffs:

- Host rendering depends on a usable host GPU and driver stack.
- Results are less renderer-independent than SwiftShader software rendering.
- This does not prove every host or headless CI worker can expose host GPU rendering.
- Keep AVD state disposable with a temporary `ANDROID_AVD_HOME` or `-datadir`;
  `-gpu host` does not itself isolate emulator state.

## What does not work

- `-gpu off` does not avoid the failing software stack for this image.
  The emulator reported Lavapipe plus SwANGLE,
  and the coredump still ended in `gles_swiftshader/libGLESv2.so`.
- Disabling both cameras did not prevent the crash.
- `-no-snapshot` and a fresh `-wipe-data` directory did not prevent the crash.
- `-read-only` was not the cause.
  A writable disposable Pixel 9 AVD also crashed under SwiftShader.
- Changing from Pixel 9 Pro Fold to Pixel 9 did not prevent the software-renderer crash.
- Approving the emulator's network request in OpenSnitch did not prevent the software-renderer crash.
  A host-GPU boot had already succeeded before approval,
  and the software command still received `SIGSEGV` after approval.
- Lowering requested RAM did not help the Fold AVD.
  The emulator raised it to its 4096 MB minimum.
- Treating `pc_memory_init: above 4g size: 40000000` as the cause was rejected by a positive control:
  the successful host-GPU boot printed the same line.

## Upstream filing decision

No `.out-of-scope/` entry covers Android Emulator or SwiftShader.
Searches for
`Android Emulator 37.1.11 SwiftShader SIGSEGV Linux`,
`libGLESv2.so SIGSEGV SwiftShader Linux headless`,
and
`software renderer segmentation fault 37.1`
found no issue that matched this version,
stack,
and host-GPU workaround.

1. **Really upstream's fault:**
    not established.
   The crashing code is packaged with Android Emulator,
   but the exact 37.1.11 source revision and the role of Fedora 44's host stack are unknown.
2. **Upstream can fix it:**
    yes if the defect reproduces in Google's matching build environment.
   They can update SwiftShader or alter software-renderer selection.
3. **Supported use case:**
    yes.
   The emulator help advertises `-no-window` and `swiftshader_indirect`,
   and the release notes describe Linux software rendering.
4. **Contribution welcome:**
    bug reports are welcome.
   The [Android bug-report guide][report-bugs] requests emulator version,
   host CPU,
   device configuration,
   diagnostics,
   and reproduction steps.
   No AI-assistance ban was found in the public mirror's `README.md` or the reporting guide.
5. **Likely fix:**
    unknown.
   No matching tracker signal was found.
6. **Minimal fix prototyped:**
    no.
   The installed binary is stripped,
   public emulator source predates build `15917651`,
   and no source revision maps to the crashing packaged SwiftShader library.
   A source patch would therefore target an unverified candidate rather than this incident.

Decision:
do not file the draft as-is.
Constraint 1 is unresolved and constraint 6 cannot pass the candidate-fix applicability gate.
The verified consumer-side workaround is enough to complete local emulator testing.

### New-issue draft, do not file as-is

~~~md
Title: Android Emulator 37.1.11 SIGSEGV in packaged SwiftShader GLES during Linux headless API 37 boot

Component: Android Emulator

Android Emulator 37.1.11.0, build 15917651, crashes on Fedora 44 before boot completes when an API 37
x86_64 AVD is started headlessly with `-gpu swiftshader_indirect` or `-gpu off`.

Reproduction:

```bash
emulator -avd Issue460Pixel9 \
  -wipe-data -no-cache -no-snapshot -no-window -no-audio -no-boot-anim \
  -gpu swiftshader_indirect -camera-back none -camera-front none
```

Observed:

- `adb devices` reaches only `offline`.
- `qemu-system-x86_64-headless` exits on SIGSEGV.
- The crashing thread's top library frames are in
  `emulator/lib64/gles_swiftshader/libGLESv2.so`.

Expected:

- The emulator reaches `sys.boot_completed=1`.

Workaround:

- `-gpu host` boots the same disposable Pixel 9 AVD and API 37 image.

Please advise which symbols or diagnostic bundle are needed to map build 15917651's stripped SwiftShader stack.
~~~

[emulator-releases]: https://developer.android.com/studio/releases/emulator#37.1.11
[report-bugs]: https://developer.android.com/studio/report-bugs#emulator-bugs
