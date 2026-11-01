# Helium 0.14.9.1 and 0.15.1.1: forced Vulkan on Wayland renders videos white

## Symptom

On native Wayland,
a video advances and can play audio while its picture is a solid white rectangle.
The failure reproduced with local and remote H.264 media in Helium 0.14.9.1 and 0.15.1.1.

The triggering setting is **Vulkan** at `chrome://flags/#enable-vulkan`.
The original profile also selected **Skia Graphite** and disabled hardware video decode,
but neither setting caused the white surface:

- Resetting only **Vulkan** restored the picture.
- Keeping **Skia Graphite** selected did not restore the failure.
- The original failure occurred with software video decode,
  so VA-API was not required to trigger it.

The GPU process emits these errors repeatedly during the failing playback:

```text
'--ozone-platform=wayland' is not compatible with Vulkan. Consider switching to '--ozone-platform=x11' or disabling Vulkan
Could not find or create a backing for stream kSkia
SharedImageManager::ProduceSkia: Trying to produce a Skia representation from an incompatible backing: CompoundImageBacking
```

## Root cause

The direct cause is the unsupported combination of browser-wide Vulkan and native Wayland.
The investigation isolates the configuration boundary and the failing SharedImage path.
It does not establish a deeper format-specific defect inside `CompoundImageBacking`.

The source trace uses the Chromium reference clone at
`/home/user/temp/agent/chromium-150-2026-08-03`,
tag `150.0.7871.186`,
and commit `0fcdce5f4fdec8d442d7df760cb541f1ca6e446d`.
The Helium reference clone is `/home/user/temp/agent/helium-2026-08-03` at commit
`63d66420989d92c03e843766e1b89271a91b7b7e`.
Both are third-party,
read-only reference clones and were not modified.
Helium 0.15.1.1 produced the same warning and SharedImage errors.

### The flags UI enables browser-wide Vulkan

`chrome/browser/about_flags.cc:5317-5322` maps the two relevant UI entries to separate features:

```cpp
{"enable-vulkan", flag_descriptions::kEnableVulkanName,
 flag_descriptions::kEnableVulkanDescription, kOsLinux | kOsAndroid,
 FEATURE_VALUE_TYPE(features::kVulkan)},
{"force-enable-webgpu-interop", flag_descriptions::kWebGpuInteropName,
 flag_descriptions::kWebGpuInteropDescription, kOsLinux,
 FEATURE_VALUE_TYPE(features::kForceEnableWebGpuInterop)},
```

`gpu/config/gpu_finch_features.cc:187-198` makes browser-wide Vulkan disabled by default outside Android and defines
WebGPU interop independently:

```cpp
BASE_FEATURE(kVulkan,
#if BUILDFLAG(IS_ANDROID)
             base::FEATURE_ENABLED_BY_DEFAULT
#else
             base::FEATURE_DISABLED_BY_DEFAULT
#endif
);

// Force enable WebGPU interop when enabled. When disabled the webgpu interop
// mechanism will default to auto detection in 'GetWebGPUOnVulkanViaGLInterop'
// function.
BASE_FEATURE(kForceEnableWebGpuInterop, base::FEATURE_DISABLED_BY_DEFAULT);
```

The **Vulkan** flag therefore changes browser compositing and rasterization.
It is not required merely to obtain a Vulkan WebGPU adapter.

### Wayland declares browser-wide Vulkan incompatible

`ui/ozone/platform/wayland/gpu/wayland_surface_factory.cc:245-252` logs the exact incompatibility and then creates a
Wayland Vulkan implementation:

```cpp
#if BUILDFLAG(ENABLE_VULKAN)
std::unique_ptr<gpu::VulkanImplementation>
WaylandSurfaceFactory::CreateVulkanImplementation(bool use_swiftshader,
                                                  bool allow_protected_memory) {
  LOG_IF(ERROR, !use_swiftshader)
      << "'--ozone-platform=wayland' is not compatible with Vulkan. "
         "Consider switching to '--ozone-platform=x11' or disabling Vulkan";
  return std::make_unique<VulkanImplementationWayland>(use_swiftshader);
}
#endif
```

The warning is not incidental.
It names the exact configuration that the one-flag reproduction activates.

### The video SharedImage cannot provide a Skia representation

`gpu/command_buffer/service/shared_image/compound_image_backing.cc:1907-1929` first searches for a backing that supports
the requested access stream.
If none matches,
dynamic allocation is the only fallback:

```cpp
for (auto& element : elements_) {
  if (element.access_streams.Has(stream) && element.GetBacking() &&
      element.GetBacking()->SupportsAccess(stream, params)) {
    if (element.content_id_ == latest_content_id_) {
      best_match = &element;
      break;
    }
    if (!any_match) {
      any_match = &element;
    }
  }
}

ElementHolder* target_element = best_match ? best_match : any_match;
if (target_element) {
  return target_element->GetBacking();
}

if (base::FeatureList::IsEnabled(features::kUseDynamicBackingAllocations) &&
    shared_image_factory_) {
```

`gpu/config/gpu_finch_features.cc:314-316` shows that fallback is disabled by default:

```cpp
// Allows CompoundImageBacking to allocate backings during runtime if a
// compatible backing to serve clients requested usage is not already present.
BASE_FEATURE(kUseDynamicBackingAllocations, base::FEATURE_DISABLED_BY_DEFAULT);
```

With no compatible `kSkia` backing and no fallback allocation,
`gpu/command_buffer/service/shared_image/compound_image_backing.cc:1979-1980` emits the first SharedImage error:

```cpp
LOG(ERROR) << "Could not find or create a backing for stream " << stream;
return nullptr;
```

The null representation reaches `gpu/command_buffer/service/shared_image/shared_image_manager.cc:267-272`,
which emits the second error:

```cpp
auto representation = backing->ProduceSkia(this, tracker, context_state);
if (!representation) {
  LOG(ERROR) << "SharedImageManager::ProduceSkia: Trying to produce a "
                "Skia representation from an incompatible backing: "
             << backing->GetName();
  return nullptr;
}
```

The source stops at the missing representation.
The controlled reproduction connects that failure to the white video surface:
toggling only browser-wide Vulkan changes a fully white video rectangle back to rendered frames.

### WebGPU interop avoids browser-wide Vulkan

`gpu/config/gpu_util.cc:210-234` requires a GL graphics context for the interop route.
The force feature enables that route before the GPU feature blocklist check:

```cpp
if (use_swift_shader ||
    gpu_preferences.gr_context_type != GrContextType::kGL) {
  return kGpuFeatureStatusDisabled;
}

#if BUILDFLAG(USE_WEBGPU_ON_VULKAN_VIA_GL_INTEROP)
if (features::IsForceEnableWebGpuInterop()) {
  return kGpuFeatureStatusEnabled;
}
#endif

if (blocklisted_features.count(
        GPU_FEATURE_TYPE_WEBGPU_ON_VK_VIA_GL_INTEROP)) {
  return kGpuFeatureStatusDisabled;
}
```

`gpu/command_buffer/service/shared_image/shared_image_factory.cc:326-337` then permits the external Vulkan backing
factory while the graphics context remains GL:

```cpp
#if BUILDFLAG(ENABLE_VULKAN) && (BUILDFLAG(IS_LINUX) || BUILDFLAG(IS_FUCHSIA))
if (gr_context_type_ == GrContextType::kVulkan
#if BUILDFLAG(USE_WEBGPU_ON_VULKAN_VIA_GL_INTEROP)
    /* We support GL context for WebGPU gl-vulkan interop (on linux).*/
    || gpu_preferences_.enable_webgpu_on_vk_via_gl_interop
#endif
) {
  auto external_vk_image_factory =
      std::make_unique<ExternalVkImageBackingFactory>(
          context_state_,
          gpu_preferences_.enable_webgpu_on_vk_via_gl_interop);
  factories_.push_back(std::move(external_vk_image_factory));
}
```

`gpu/command_buffer/service/webgpu_decoder_impl.cc:1774-1787` selects Dawn's Vulkan backend when interop is active,
without changing the compositor to Vulkan:

```cpp
case WebGPUAdapterName::kDefault: {
#if BUILDFLAG(IS_LINUX)
  if (shared_context_state_->GrContextIsVulkan() ||
      webgpu_on_vk_gl_interop_ ||
      shared_context_state_->IsGraphiteDawnVulkan()) {
    backend_types = {wgpu::BackendType::Vulkan};
  } else {
    // Deliberately disable compat on linux.
    backend_types = {wgpu::BackendType::Null};
  }
#endif
```

This feature is distinct from unsafe WebGPU.
`gpu/command_buffer/service/webgpu_decoder_impl.cc:1149-1157` changes the Dawn safety level only for developer,
experimental,
or unsafe WebGPU preferences:

```cpp
if (gpu_preferences.enable_webgpu_developer_features ||
    gpu_preferences.enable_webgpu_experimental_features) {
  safety_level_ = webgpu::SafetyLevel::kSafeExperimental;
}
if (gpu_preferences.enable_unsafe_webgpu) {
  safety_level_ = webgpu::SafetyLevel::kUnsafe;
}
dawn_instance_ = DawnInstance::Create(dawn_platform_.get(), gpu_preferences,
                                      safety_level_);
```

No such preference was present in the passing process command lines.

### The AMD interop override has a real tradeoff

Chromium 150 does not enable Vulkan-through-GL interop by default for this AMD GPU.
`gpu/config/software_rendering_list.json:1479-1506` blocklists the interop feature except for listed Intel and Nvidia
configurations:

```json
{
  "id": 186,
  "cr_bugs": [442791440, 475935650],
  "description": "Disable webgpu on vk via gl interop",
  "exceptions": [
    {
      "vendor_id": "0x8086",
      "intel_gpu_generation": { "op": ">=", "value": "12" },
      "driver_vendor": "Mesa",
      "driver_version": { "op": ">=", "value": "22.0" }
    },
    {
      "vendor_id": "0x10de",
      "driver_version": { "op": ">=", "value": "535.183.01" }
    }
  ],
  "features": ["webgpu_on_vk_via_gl_interop"]
}
```

**Force enable WebGPU interop** bypasses this feature-level blocklist for AMD.
It does not disable Dawn's adapter blocklist or switch Dawn to its unsafe safety level,
but it remains an experimental driver-path override.
The recommendation is specific to the tested RX 7600 and Mesa 26.1.5,
not a claim that every AMD and Mesa combination is supported.

Chromium's M148 change
[Enable `webgpu_on_vulkan_via_gl_interop` only for Wayland][chromium-wayland-interop]
confirms that Wayland is the intended Linux presentation platform for this interop route.
That change disabled the route on X11 after Vulkan device creation caused blank video on an Nvidia multi-GPU system.

### Graphite can stay selected but is not active

`gpu/config/gpu_finch_features.cc:584-592` explicitly refuses the Graphite base feature on Linux:

```cpp
// Disallow Graphite from being enabled via the base::Feature on
// not-yet-supported platforms to avoid users experiencing undefined behavior,
// including behavior that might prevent them from being able to return to
// chrome://flags to disable the feature.
if (base::FeatureList::IsEnabled(features::kSkiaGraphite)) {
  LOG(ERROR) << "Enabling Graphite on a not-yet-supported platform is "
                "disallowed for safety";
}
return false;
```

`gpu/config/gpu_finch_features.cc:613-619` returns before consulting the feature state:

```cpp
if (!IsSkiaGraphiteSupportedByDevice(command_line)) {
  // Return early before checking "SkiaGraphite" feature so that devices
  // which don't support graphite are not included in the finch study.
  return false;
}

return base::FeatureList::IsEnabled(features::kSkiaGraphite);
```

The UI selection can therefore remain **Enabled** without changing the Linux renderer.
Both tested versions logged the safety refusal and reported `skia_graphite: disabled_off` with `GaneshGL`.
Forcing the separate `--enable-skia-graphite` command-line switch bypasses this guard and is not equivalent to the UI
selection.

## Verification

### Environment and versions

The reproduction environment was:

- Bazzite 44.20260721 Kinoite in a Wayland session
- Linux `7.1.3-ogc5.1.fc44.x86_64`
- AMD Radeon RX 7600,
   PCI ID `1002:7480`,
   using `amdgpu`
- Mesa DRI and Vulkan drivers `26.1.5-2.fc44`
- Helium 0.14.9.1,
   Chromium 150.0.7871.186
- Helium 0.15.1.1,
   Chromium 151.0.7922.71
- Helium 0.15.1.1 AppImage SHA-256
  `ab3df0fa79ef0609291d3dc4df876fe03bc5b98972cd303283db4f609156b37a`

Every run used a disposable `--user-data-dir`.
The active Helium profile was not changed.

### Runnable harness

Save this as `repro.html` in an empty directory:

```html
<!doctype html>
<meta charset="utf-8">
<title>Helium video and WebGPU reproduction</title>
<video autoplay muted loop playsinline
  src="https://www.w3schools.com/html/mov_bbb.mp4"></video>
<pre id="status">running</pre>
<script type="module">
const video = document.querySelector('video')
const status = document.querySelector('#status')
try {
await video.play()
const adapter = await navigator.gpu?.requestAdapter({ powerPreference: 'high-performance' })
if (!adapter) throw new Error('requestAdapter returned null')
const device = await adapter.requestDevice()
const input = new Uint32Array([3, 5, 8, 13])
const work = device.createBuffer({
  size: input.byteLength,
  usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE,
})
const read = device.createBuffer({
  size: input.byteLength,
  usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
})
const module = device.createShaderModule({
  code: `
    @group(0) @binding(0) var<storage, read_write> values: array<u32>;
    @compute @workgroup_size(4)
    fn main(@builtin(global_invocation_id) id: vec3<u32>) {
      values[id.x] = values[id.x] * 2u;
    }
  `,
})
const pipeline = device.createComputePipeline({
  layout: 'auto',
  compute: { module, entryPoint: 'main' },
})
const bindGroup = device.createBindGroup({
  layout: pipeline.getBindGroupLayout(0),
  entries: [{ binding: 0, resource: { buffer: work } }],
})
for (let iteration = 0; iteration < 300; iteration += 1) {
  device.queue.writeBuffer(work, 0, input)
  const encoder = device.createCommandEncoder()
  const pass = encoder.beginComputePass()
  pass.setPipeline(pipeline)
  pass.setBindGroup(0, bindGroup)
  pass.dispatchWorkgroups(1)
  pass.end()
  if (iteration === 299) {
    encoder.copyBufferToBuffer(work, 0, read, 0, input.byteLength)
  }
  device.queue.submit([encoder.finish()])
  await new Promise(resolve => requestAnimationFrame(resolve))
}
await read.mapAsync(GPUMapMode.READ)
const output = [...new Uint32Array(read.getMappedRange().slice(0))]
read.unmap()
status.textContent = JSON.stringify({
  adapter: adapter.info,
  output,
  videoCurrentTime: video.currentTime,
  videoReadyState: video.readyState,
}, null, 2)
} catch (error) {
  status.textContent = String(error)
}
</script>
```

Serve the directory in one terminal:

```console
python3 -m http.server 8765 --bind 127.0.0.1
```

Set `HELIUM` to the AppImage path in another terminal:

```console
HELIUM="${HOME}/Downloads/helium-0.15.1.1-x86_64.AppImage"
```

The failing run is:

```console
PROFILE="$(mktemp --directory)"
"${HELIUM}" \
  --user-data-dir="${PROFILE}" \
  --ozone-platform=wayland \
  --enable-features=Vulkan \
  http://127.0.0.1:8765/repro.html
```

The passing interop run with Graphite still selected is:

```console
PROFILE="$(mktemp --directory)"
"${HELIUM}" \
  --user-data-dir="${PROFILE}" \
  --ozone-platform=wayland \
  --disable-features=Vulkan \
  --enable-features=SkiaGraphite,ForceEnableWebGpuInterop \
  http://127.0.0.1:8765/repro.html
```

Use a fresh `PROFILE` for each run so Chromium process reuse and persisted flags cannot cross-contaminate the result.
The matching durable UI procedure is in
[`doc/runbook/helium-white-video-vulkan-wayland.md`](../runbook/helium-white-video-vulkan-wayland.md).

### Configurations that render cleanly

- Helium 0.14.9.1 and 0.15.1.1,
  native Wayland,
  **Vulkan** at **Default**,
  **Skia Graphite** at **Enabled**,
  hardware video decode at **Default**,
  and **Force enable WebGPU interop** at **Enabled**:
  - `ANGLE_OPENGL` and `GaneshGL`
  - `skia_graphite: disabled_off`
  - Helium 0.14.9.1 can report `vulkan: enabled_on` for the interop device even though presentation remains GL
  - Helium 0.15.1.1 reports `vulkan: disabled_off`
  - hardware AMD WebGPU compute output `[6, 10, 16, 26]` after 300 iterations
  - H.264 video `readyState` 4 with an advancing `currentTime`
  - `VaapiVideoDecoder`
  - no `kSkia` backing failures,
    incompatible-backing errors,
    GPU-process exits,
    or AMDGPU kernel faults
- The same interop arrangement with hardware video decode disabled:
  - `FFmpegVideoDecoder`
  - rendered H.264 frames
  - no target SharedImage errors
- X11 with browser-wide Vulkan:
  - rendered the same test video
  - not acceptable for the native-Wayland requirement

### Configurations that fail

- Helium 0.15.1.1 with only browser-wide Vulkan forced on native Wayland:
  - the 1280 by 720 video region had white-pixel proportion `1`
  - `Could not find or create a backing for stream kSkia` occurred 521 times
  - the incompatible `CompoundImageBacking` error occurred 521 times
- Helium 0.14.9.1 with the original persisted settings,
  **Vulkan** and **Skia Graphite** enabled and hardware video decode disabled:
  - local and remote H.264 video surfaces were white
  - resetting only **Vulkan** restored rendered frames
- Forcing Graphite through `--enable-skia-graphite` with Dawn Vulkan:
  - bypassed the Linux safety refusal
  - caused repeated GPU-process crashes
  - did not provide a usable browser configuration

## Verified workarounds

### Keep Graphite selected and use WebGPU Vulkan-through-GL interop

Set these values in `chrome://flags`:

- **Vulkan**:
   **Default**
- **Skia Graphite**:
   **Enabled**
- **Hardware-accelerated video decode**:
   **Default**
- **Force enable WebGPU interop**:
   **Enabled**

This is the recommended configuration for the tested machine.
It preserves the user's Graphite selection,
native Wayland,
VA-API video decode,
and hardware Vulkan WebGPU without unsafe WebGPU.

Tradeoffs:

- Chromium refuses to activate Graphite on Linux,
  so the retained selection currently has no rendering effect.
  After a Helium update,
  recheck `Skia Graphite` and `Skia Backend Type` in `chrome://gpu`.
  If they change from `Disabled` and `GaneshGL`,
  retest video because the selected flag has become active.
- The interop flag bypasses Chromium's feature-level AMD blocklist.
  Recheck `chrome://gpu`,
  video,
  and WebGPU after Mesa or Helium updates.
- Helium can still log the Wayland/Vulkan warning while initializing the interop Vulkan device,
  even though presentation remains `ANGLE_OPENGL` and `GaneshGL`.

### Reset Vulkan without enabling WebGPU interop

Set **Vulkan** to **Default** and leave the other flags unchanged.
This restored video in both tested Helium versions.

Tradeoff:
hardware WebGPU remains unavailable on this AMD configuration because Chromium blocklists its interop path by default.

### Run browser-wide Vulkan through X11

Launch Helium with `--ozone-platform=x11` while keeping Vulkan enabled.
The test video rendered.

Tradeoff:
this gives up native Wayland and does not satisfy the requested configuration.
Chromium's later Wayland-only interop change also records a blank-video regression for Vulkan device creation on X11
with an Nvidia multi-GPU system.

## What does not work

- Leaving **Vulkan** enabled on native Wayland:
  both tested browser versions reproduce the white surface and SharedImage errors.
- Disabling hardware video decode as the only change:
  the original profile already used `FFmpegVideoDecoder` and still failed.
- Updating from Helium 0.14.9.1 to 0.15.1.1 without changing Vulkan:
  the Vulkan-only failure reproduces in Chromium 151.
- Treating **Skia Graphite** at **Enabled** as active Graphite:
  Chromium logs the safety refusal and continues on `GaneshGL`.
- Forcing Graphite with `--enable-skia-graphite` and a Vulkan Dawn backend:
  the GPU process repeatedly crashed.
- Enabling **Unsafe WebGPU Support**:
  this changes Dawn's safety level and adapter policy,
  not the incompatible Wayland compositor path.
  It is unnecessary for the verified interop route.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers Helium,
Chromium Vulkan,
Wayland,
or this video failure.

The duplicate search covered open and closed Helium issues and pull requests using `video`,
`Vulkan`,
`Wayland`,
`white`,
and both SharedImage error strings.
It also searched Chromium's public tracker using the warning and SharedImage strings.

Two Helium reports are related but not duplicates:

- [Helium Linux issue 246][helium-246] concerns black or fuzzy video after a Chromium 147 update,
  primarily HEVC and Nvidia VA-API failures.
  It does not involve forced Vulkan or the two SharedImage errors.
- [Helium Linux issue 133][helium-133] concerns page glitches around certain media in Helium 0.7.3.1.
  Disabling hardware decode helped that reporter,
  whereas the present Vulkan failure reproduces with software decode.

No matching Chromium report was found.
The six filing constraints are:

1. **Is it really upstream's fault?**
   No for a reportable default-path defect.
   The failure requires forcing a feature that Chromium itself declares incompatible with Wayland.
2. **Can upstream fix it?**
   Yes technically.
   Chromium could reject browser-wide Vulkan on Wayland or add the missing backing compatibility,
   but this does not overcome the unsupported-use-case result.
3. **Are they supporting this use case?**
   No.
   Chromium's source emits the incompatibility warning,
   refuses Graphite on Linux,
   and blocklists interop on this AMD device unless forced.
4. **Would the repo welcome the contribution?**
   Chromium permits AI-assisted work when the human understands it and personally answers human feedback under its
   [AI coding policy][chromium-ai-policy].
   Helium's `CONTRIBUTING.md:17-20` directs Linux issues to `helium-linux`
   and prohibits AI-generated issue or pull-request descriptions:

   ```md
   - For platform-specific issues or features, open the issue or PR in the
     related platform repository instead of this one.
   - Do not use AI to generate issue or PR descriptions. You will get banned
     for spam without review. We want contributions from people, not bots.
   ```

   This constraint therefore fails for posting this generated draft to Helium.
5. **Will they likely fix it?**
   No supporting signal was found for browser-wide Vulkan on Wayland.
   The explicit incompatibility and safety guards point away from supporting this forced combination.
6. **Have we prototyped a minimal architectural fix?**
   No.
   Constraints 1,
   3,
   and 5 fail for Chromium,
   while constraint 4 also fails for Helium.
   The automatic prototype requirement therefore does not apply.
   The configuration workaround solves the user-facing problem without changing upstream source.

The result is **do not file upstream** unless the white surface later reproduces
with **Vulkan** at **Default** in a clean profile.
That default-path reproduction would change constraints 1 and 3 and warrant a new audit.
There is nothing additive to post on issues 246 or 133.

### Draft, do not file as-is

~~~md
Title: [Linux][Wayland] Forced Vulkan renders video white with CompoundImageBacking errors

Labels: OS-Linux, Internals-GPU, Internals-Media

## Description

On native Wayland, forcing Chromium's Vulkan feature causes H.264 video to advance while its picture is solid white.
The GPU process logs that Wayland is incompatible with Vulkan, followed by repeated failures to create a `kSkia`
backing and produce a Skia representation from `CompoundImageBacking`.

This does not reproduce with Vulkan at its default value.

## Reproduction

Save this as `repro.html` and serve its directory on `http://127.0.0.1:8765`:

```html
<!doctype html>
<meta charset="utf-8">
<video autoplay muted loop controls
  src="https://www.w3schools.com/html/mov_bbb.mp4"></video>
```

Start Chromium 150 or 151 on Wayland with:

```console
chromium \
  --user-data-dir="$(mktemp --directory)" \
  --ozone-platform=wayland \
  --enable-features=Vulkan \
  http://127.0.0.1:8765/repro.html
```

The timeline advances while the picture is white,
and stderr repeatedly contains:

```text
Could not find or create a backing for stream kSkia
SharedImageManager::ProduceSkia: Trying to produce a Skia representation from an incompatible backing:
CompoundImageBacking
```

## Source trace

`ui/ozone/platform/wayland/gpu/wayland_surface_factory.cc:245-252` warns that
`--ozone-platform=wayland` is incompatible with Vulkan but still constructs `VulkanImplementationWayland`.
`gpu/command_buffer/service/shared_image/compound_image_backing.cc:1907-1979` finds no `kSkia` backing and cannot
allocate one because `UseDynamicBackingAllocations` is disabled by default.
`gpu/command_buffer/service/shared_image/shared_image_manager.cc:267-272` then rejects the null Skia representation.

## Suggested fix

Reject browser-wide Vulkan on Wayland before constructing `VulkanImplementationWayland`,
or add a supported Skia backing for this video SharedImage path.
Add a Wayland video playback regression test that verifies visible decoded frames under any supported Vulkan route.

## Workaround

Leave browser-wide Vulkan disabled.
For hardware WebGPU,
use `ForceEnableWebGpuInterop` so Chromium keeps GL compositing and selects Dawn Vulkan for WebGPU.
~~~

[chromium-ai-policy]: https://chromium.googlesource.com/chromium/src/+/HEAD/agents/ai_policy.md
[chromium-wayland-interop]: https://chromium.googlesource.com/chromium/src/+/1e78781d6b8a30aba8551b4072160641597c27fd
[helium-133]: https://github.com/imputnet/helium-linux/issues/133
[helium-246]: https://github.com/imputnet/helium-linux/issues/246
