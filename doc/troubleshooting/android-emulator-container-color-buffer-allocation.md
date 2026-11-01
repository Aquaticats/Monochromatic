# Android Emulator 37.2.12 host rendering aborts during an API37 container color-buffer probe

## Symptom

An owned disposable `Fold_No_Hardware_Probe` launch under Fedora44 with
`podman run --memory=6g --cpus=2`,
`-gpu host` and Xvfb exited before guest readiness.
The emulator emitted:

```text
# Android Emulator startup diagnostic
ERROR        | Failed to allocate ColorBuffer with Vulkan backing.
ERROR        | Failed to setup memory type index test ColorBuffer.
FATAL        | Failed to find memory type for ColorBuffers.
```

The owning process returned the emulator's abort status `134` through
Node's failed `execFileSync` call.
This is not a music-player APK failure,
a demonstrated host out-of-memory event or the prior packaged
SwiftShader `SIGSEGV` incident.
No native application frame or fresh guest settings snapshot was retained
from this attempt.

## Source-traced failure boundary

Read-only `google/gfxstream` source was inspected at
`07ee40efb0e7037a9a9b7fe59071e7c4997e7cbe`.
It matches the diagnostic sequence,
but is not mapped to the installed emulator's binary revision.
Source findings are comparable implementation evidence only.

`host/vulkan/vk_common_operations.cpp:3233` to `3240`
reports a false result from the external-memory allocator:

```cpp
// host/vulkan/vk_common_operations.cpp
bool allocRes = allocExternalMemory(vk, &infoPtr->memory,
                                    deviceAlignment, kNullopt, dedicatedImage, infoPtr);
if (!allocRes) {
    GFXSTREAM_ERROR("Failed to allocate ColorBuffer with Vulkan backing.");
    return false;
}
```

`findRepresentativeColorBufferMemoryTypeIndexLocked` at lines `5307` to `5319`
creates a `64` by `64` RGBA probe with device-local memory properties.
Failure returns no representative result:

```cpp
// host/vulkan/vk_common_operations.cpp
if (!createVkColorBufferLocked(kArbitraryWidth, kArbitraryHeight,
                               GfxstreamFormat::R8G8B8A8_UNORM,
                               kArbitraryHandle, true, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
                               kArbitraryMipLevels)) {
    GFXSTREAM_ERROR("Failed to setup memory type index test ColorBuffer.");
    return std::nullopt;
}
```

The initialization path at lines `1751` to `1754` turns that missing probe
result into the fatal:

```cpp
// host/vulkan/vk_common_operations.cpp
if (!representativeInfo) {
    GFXSTREAM_FATAL("Failed to find memory type for ColorBuffers.");
}
```

The final message does not prove the physical GPU lacks every memory type.
The comparable path can reach it after an allocation failure.
The installed allocator's deciding failure and driver/device exposure
remain unresolved;
no ABI,
libunwind,
RAM-cap or shader defect is attributed from nearby messages alone.

## Verification and reproduction boundary

The current installed `emulator/source.properties` reports
`Pkg.Revision=37.2.12` and `Pkg.BuildId=16428233`.
An initial draft carried the historical `37.1.11` version from an older
incident;
that attribution is withdrawn.
The freshly read installed package metadata is the current version evidence.
The private startup harness uses:

```bash
# Disposable reproduction; requires the already owned AVD and inspected Xvfb files.
podman run --memory=6g --cpus=2 --rm \
  --name fold-light-feedback-avd \
  --security-opt label=disable --device /dev/kvm \
  --volume /usr:/usr:ro \
  --volume "${HOME}/Android/Sdk:${HOME}/Android/Sdk:ro" \
  --volume "${HOME}/temp/agent/fold-no-hardware-avd:${HOME}/temp/agent/fold-no-hardware-avd:rw" \
  --volume "${HOME}/temp/agent/fold-no-hardware-avd/Xvfb-container:/opt/Xvfb:ro" \
  --volume "${HOME}/temp/agent/fold-no-hardware-avd/xvfb-run-fedora44:/opt/xvfb-run:ro" \
  --env HOME=/tmp --env PATH=/opt:/usr/bin:/usr/sbin \
  --env LIBGL_DRIVERS_PATH=/usr/lib64/dri \
  --env "ANDROID_AVD_HOME=${HOME}/temp/agent/fold-no-hardware-avd" \
  registry.fedoraproject.org/fedora:44 \
  /usr/bin/sh /opt/xvfb-run -d --error-file=/dev/stderr \
  "${HOME}/Android/Sdk/emulator/emulator" \
  -avd Fold_No_Hardware_Probe -ports 5580,5581 \
  -no-snapshot -no-audio -no-boot-anim -no-window \
  -gpu host -memory 4096 -cores 2
```

The current SDK binary's `-help` and `-help-feature` were invoked in a
2 GiB/2 CPU container before the configuration probe.
The feature help documents force-enabling or disabling a named feature
with `-feature`.
The installed `lib/advancedFeatures.ini:249` to `251` names `Vulkan`:

```ini
# emulator/lib/advancedFeatures.ini
# Vulkan------------------------------------------------------------------------
# If enabled, the guest Vulkan HAL (if installed) will activate.
Vulkan = off
```

That default text is not a read-back of effective runtime graphics state.
The failing log separately reports initialized host Vulkan emulation.
Guest HAL activation and host graphics allocation must not be conflated.

### Failing catalog

- The invocation without a feature override reached the quoted color-buffer
  allocation fatal before guest readiness.
- Adding only `-feature -Vulkan` then reached a distinct multiple-instance
  fatal before the graphics result could be compared.
  This does not prove the override fixes or preserves the allocation failure.

### Passing controls

- Current SDK help and feature-help invocations returned successfully in
  their bounded container.
- `lsof` and `fuser` identified an intentionally held disposable control file.
  Empty exact-lock probes therefore had a demonstrated owner-detection
  positive control before any lock was moved.
- These controls are not passing guest boots or application-rendering
  acceptance.

## Workaround status and tradeoffs

No graphics cause or renderer-independent workaround is established.
The post-quarantine runtime reached a rendered launcher and later an actual
ADB authorization prompt;
its guest transport became usable after direct approval of the matching
owned key.
The successful attempt also changed identity persistence and launch options,
so it does not isolate which change affects the initial allocation failure.
An explicit guest-Vulkan override is a consumer-side experiment,
not an installed-source patch or proof that host allocation is disabled.
Changing the graphics backend also changes the capture environment;
any successful cohort must retain its exact effective environment and
cannot claim renderer-independent evidence.

The separate restart rejection followed the multiple-instance branch.
Matching container/process,
`lsof`,
`fuser` and `lslocks` checks found no owner of the exact disposable locks.
The two named files were jointly moved into a private backup,
not deleted,
before a bounded retry.
The retry passed the lock rejection and remained running under the same
6 GiB/2 CPU caps.
Its bounded readiness watch exited because ADB reported
`device unauthorized`,
not because a successful shell read proved guest boot incomplete.
No guest boot-completion or fresh settings read has yet succeeded.
The owning-container console's `help` command was accepted,
so the running runtime and console remain reachable.
Guest ADB authorization and console-token authorization are distinct
boundaries.

### Guest authorization recovery

The subsequent `-skip-adb-auth` attempt remained unauthorized during its
bounded watch.
An independently captured screen showed `System UI isn't responding`,
not an RSA prompt.
A console mouse command returned `OK` without the intended visible state
change;
authenticated local gRPC touch then removed that ANR dialog.
That is an input positive control,
not proof of the ANR's cause or guest authorization.
Container-local `adb reconnect offline` and one server restart still
reported unauthorized transport.

The server's inspected environment used `HOME=/tmp` and the container
network was isolated `pasta`.
Only its generated key pair was retained in a private,
mode-restricted identity directory for one retry.
No original host key or original AVD was read or changed.
The retry without the ineffective skip option displayed the actual
`Allow USB debugging?` prompt.
The displayed fingerprint matched the runtime-owned public key.
After `Always allow from this computer` and `Allow` were selected through
local authenticated input,
`adb devices -l` reported `device`,
`getprop sys.boot_completed` returned `1` and the AVD identity matched.
The fresh settings snapshot then succeeded.
This verifies recovery of this owned guest's debug transport,
not a storage grant,
application fit or universal emulator authorization mechanism.
Generated gRPC metadata path/port/token were rediscovered for the current
runtime;
previous credentials and PID paths were not treated as restart invariants.
See [the lock lifecycle boundary](android-emulator-37-disposable-avd-lock-after-hard-stop.md).
This lock recovery does not diagnose the initial graphics failure.

## Separate post-readiness renderer failure

After verified guest authorization,
the host-renderer capture visit retained nine inner/100% frames before the
owning emulator received `SIGSEGV`.
The wrapper reported status `139`;
the capture task separately timed out on `am force-stop`.
The requested `-gpu host` configuration actually reported:

```text
# Android Emulator 37.2.12 runtime adapter diagnostic
Graphics Adapter Android Emulator OpenGL ES Translator (llvmpipe (LLVM 22.1.8, 256 bits))
```

Its final diagnostics included `gles_v2_imp.cpp` functions reporting
`error null ctx`.
Comparable `host/gl/glestranslator/common/include/common/gles_macros.h:28`
to `43` obtains the current context and routes a missing context through
`FAIL_IF`.
The guard's source at lines `18` to `21` prints and returns:

```cpp
// host/gl/glestranslator/common/include/common/gles_macros.h
#define FAIL_IF(condition, description) if((condition)) { \
        fprintf(stderr, "%s:%s:%d error %s\\n", __FILE__, __FUNCTION__, __LINE__, description); \
        return; \
    }
```

This identifies a diagnostic guard,
not the crash's deciding instruction or whether the messages preceded
teardown.
No usable matching coredump was found in the local `coredumpctl` listing.
No memory-cap,
LLVM,
Xvfb or driver defect is established from these messages alone.
This is distinct from both the pre-guest allocation fatal and the older
37.1.11 SwiftShader incident.

The current 37.2.12 GPU help was exercised and lists `swiftshader` as a
software renderer for GLES and Vulkan.
That renderer's current-37.2.12 probe reached authorized `device`
transport,
boot-completion `1` and a separately retained startup settings record under
the same owned AVD and 6 GiB/2 CPU bounds.
The runtime explicitly reported OpenGL ES Translator over Google SwiftShader.
A new capture cohort is in progress.
Boot success alone does not establish a graphics-crash fix or complete
application evidence;
it is not ruled out by the older version's failure.
New startup settings and renderer provenance remain separate from the
original restoration snapshot and interrupted cohort.

## Separate SwiftShader visit memory-limit termination

The current-37.2.12 SwiftShader visit completed the 40-state initial cohort
and three native Undo-intent/Dismiss controls before its QEMU process was
killed.
The owning wrapper reported status `137` and the next validated hierarchy
request rejected `adb: device offline`.
This is not the host/llvmpipe `SIGSEGV`.

A source of the termination is now measured:
the kernel journal names the exact owning libpod memory cgroup and emits
`CONSTRAINT_MEMCG` with:

```text
# Kernel journal, identifiers redacted; exact owning container matched privately.
Memory cgroup out of memory: Killed process <qemu-pid> (qemu-system-x86)
```

Podman's terminal event for that owning container separately records exit
`137`.
The original `journalctl --kernel` command was invalid;
the installed help names `--dmesg`,
and that corrected invocation returned the cgroup OOM evidence.
No empty result from the invalid command counts as kernel-log absence.
The evidence establishes a contained memory-limit termination,
not an isolated renderer leak,
a click-specific cause or general host memory exhaustion.
The 6 GiB/2 CPU caps are not raised.
A fresh bounded visit retains the granted owned identity for the remaining
native controls and explicit original-settings restoration/read-back;
its startup snapshot remains separate from the restoration target.

## Separate fresh-guest startup and capture-transport observations

A fresh API37 guest and container-native Ubuntu libraries did not isolate
one universal failure cause.
The [Xvfb PID 1 readiness boundary](xvfb-run-container-init.md)
prevented the first native-library attempt from launching QEMU at all.
Adding the verified init configuration passed that boundary;
a later SwiftShader attempt still ended on `SIGSEGV` before readiness.
ANGLE/SwiftShader booted but a cover capture rejected a launcher ANR.
The host-renderer request with a render node exposed actually reported
`llvmpipe (LLVM 20.1.2, 256 bits)`,
not hardware acceleration.
That configuration completed the cover lifecycle controls.
It also had a later cold-start System UI failure,
so it is not recorded as a general crash/ANR fix.

A separate no-input startup observation recorded 23 samples through guest
uptime `135.65` seconds.
No debug-app launch,
settings write or touch was issued during that interval.
The full retained event buffer contains process-start records but no
music-player process entry.
It reports:

```text
# API37 event buffer; process identifiers omitted.
am_anr: com.android.systemui, executing service com.android.systemui/.keyguard.KeyguardService, waited 20023ms
```

Other image packages also failed startup or broadcast deadlines.
The System UI ANR window subsequently owned focus.
These events separate the observed startup failure from feedback scene
changes;
they do not identify a CPU,
memory,
image or library cause.
A short run of clean focus samples was not sufficient readiness evidence.
The same guest was retained,
the named startup dialog was closed once through ADB,
and verified application focus preceded capture.

Later acquisition interruptions were `spawnSync podman ETIMEDOUT`,
including a hierarchy dump and a filtered log read.
Completed records were retained;
timed-out dumps were not accepted as fresh hierarchy evidence.
Process inspection found no remaining dump/read command,
and the guest still answered `device`.
A client entering only the owned container's user/network namespaces
connected to its existing ADB server without per-command OCI execution:

```bash
# Owned disposable runtime only; no host/original-AVD server is selected.
nsenter --target "${owned_container_pid}" --user --net \
  "${HOME}/Android/Sdk/platform-tools/adb" -s emulator-5582 get-state
```

The actual capture wrapper pins the generated runtime home and clears
inherited ADB vendor-key/server overrides.
The emulator retains its 6 GiB/2 CPU cgroup;
the namespace client is not an emulator restart or cap increase.
The remaining cover poses completed through that bridge.
This is a verified alternative client route,
not proof that Podman's internals caused every observed timeout.
The 48 held images remain separate from automatic-expiry and native-action
results.

## What does not work

- Treating the feature-override attempt's lock fatal as a graphics result.
- Inferring host RAM exhaustion from the word `memory` in the final fatal.
- Attributing the failure to nearby libunwind or XKEYBOARD warnings without
  a distinguishing control.
- Using `-read-only` to bypass a writable fixture's stale-lock rejection.
- Modifying the original AVD or deleting arbitrary lock files.

## Upstream filing decision

Nothing is filed or drafted as ready.
The comparable source trace supplies a diagnostic boundary,
not an upstream defect with a matched source/binary reproduction.

- Fault:
  the deciding allocator failure and host device/driver conditions are
  unresolved;
  this is not yet an isolated upstream defect.
- Ability:
  allocator or fallback changes are possible in principle,
  but no architectural limitation or concrete required change is claimed.
- Supported input:
  current emulator help exposes host rendering,
  headless operation and named feature overrides;
  the exact container driver environment is not an established guarantee.
- Contribution policy:
  no contribution path for a matched responsible revision was audited.
- Likely action:
  no maintainer decision about this exact allocation failure was established.
- Prototype:
  only consumer configuration and owned-fixture lifecycle are being probed;
  no upstream source edit or unverified candidate patch is retained.

No upstream draft is prepared,
so duplicate-tracker and `.out-of-scope/` filing checks are not represented
as completed.
The source clone remains read-only.
No original AVD,
real library or application storage operation is part of this investigation.
