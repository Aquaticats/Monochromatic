# Helium 0.21.3.1 loses WebGL when Vulkan and DrDc are forced on Wayland

## Symptom

Figma reported WebGL was unavailable while the user had WebGL enabled.
Helium's DevTools GPU status reported `webgl: enabled` even when a page's WebGL and WebGL 2 requests both failed.
The failure reproduced in an empty tab using the user's experimental flag selections in a disposable profile.
The signed-in Figma document remains unverified.

The browser's GPU process exited three times in the failing launch.
The original session journal and the reproduction both contained
`GPU process exited unexpectedly: exit_code=133` from `content/browser/gpu/gpu_process_host.cc:1042`.
The page received these `webglcontextcreationerror` messages in the reproduction with the full set of flags.

```text
Failed to create WebGLRenderingContext.
Failed to create WebGL2RenderingContext.
Web page caused context loss and was blocked
```

## Root cause

Helium on this machine crashes its GPU process when browser-wide Vulkan and a separate GPU thread for the display
compositor are forced together.
Either feature by itself could render the WebGL probe.
The investigation isolated the configuration that triggers the crash.
The precise native assertion or driver call remains unidentified in the stripped binary.

Chromium maps the experimental setting to the DrDc feature in
[`chrome/browser/about_flags.cc:9136`][drdc-flag].

```cpp
{"enable-drdc", flag_descriptions::kEnableDrDcName,
 flag_descriptions::kEnableDrDcDescription, kOsAll,
 FEATURE_VALUE_TYPE(features::kEnableDrDc)},
```

The setting gives the display compositor its own GPU thread while WebGL stays on the main GPU thread.
Chromium ships Linux with this feature off, as shown in [`gpu/config/gpu_finch_features.cc:270`][drdc-default].

```cpp
BASE_FEATURE(kEnableDrDc,
#if BUILDFLAG(IS_ANDROID)
             base::FEATURE_ENABLED_BY_DEFAULT
#elif BUILDFLAG(IS_MAC)
             // DrDC will not be running if Graphite is disabled on Mac.
             base::FEATURE_DISABLED_BY_DEFAULT
#else
             // NOT SUPPORTED. DO NOT ENABLE!
             base::FEATURE_DISABLED_BY_DEFAULT
#endif
);
```

Blink tries to create the graphics context even when WebGL is permitted in the settings.
The failure can reach the page as a context-loss block.
[`third_party/blink/renderer/modules/webgl/webgl_rendering_context_base.cc:792`][webgl-context]

```cpp
auto provider = CreateContextProviderInternal(host, attributes, context_type,
                                              context_info);
```

```cpp
if (!host->IsWebGLBlocked())
  return provider;

host->SetContextCreationWasBlocked();
host->HostDispatchEvent(WebGLContextEvent::Create(
    event_type_names::kWebglcontextcreationerror,
    "Web page caused context loss and was blocked"));
return nullptr;
```

## Verification at 2026-11-19T15:29:42-05:00

Codex ran Helium `0.21.3.1` (Chromium `157.0.8103.78`) on native Wayland on CachyOS,
with a Radeon RX 7600 and Mesa `26.2.6`.
The Chromium source references use commit `4b3d0dabb8250b6f152d2d85d79e7706e309d087`.
Each launch used a fresh `--user-data-dir`. The copied profile data comprised flags and the acceleration preference.

To reproduce from a shell, launch with the four features enabled.
For the passing comparison, create a fresh profile and remove `,EnableDrDc` from the same command.

```console
# Run from the repository root.
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
profile="$(mktemp --directory "${HOME}/temp/agent/helium-webgl.XXXXXXXX")"
helium-browser --user-data-dir="$profile" --ozone-platform=wayland \
  --enable-features=Vulkan,DefaultANGLEVulkan,ForceEnableWebGpuInterop,EnableDrDc about:blank
```

Run the following in the empty tab's DevTools console.
The retained probe also checked each context with `failIfMajorPerformanceCaveat: true`.

```javascript
// Run in the DevTools console on about:blank.
for (const api of ['webgl', 'webgl2']) {
  const canvas = document.createElement('canvas');
  canvas.addEventListener('webglcontextcreationerror', event => console.log(api, event.statusMessage));
  const gl = canvas.getContext(api);
  if (gl === null) {
    console.log(api, gl);
    continue;
  }
  gl.clearColor(1, 0, 0, 1);
  gl.clear(gl.COLOR_BUFFER_BIT);
  const pixel = new Uint8Array(4);
  gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, pixel);
  console.log(api, [...pixel]);
}
```

### Passing configurations

- The user's flags with only `enable-drdc@1` removed. Both contexts read back `[255, 0, 0, 255]`
  on ANGLE Vulkan via the RX 7600. DevTools reported `vulkan: enabled_on`. The probe recorded zero GPU exits.
- The clean command-line comparison without `EnableDrDc` gave the same results.
- In fresh profiles, browser-wide Vulkan alone and DrDc alone each returned the expected pixels.

### Failing configurations

- The user's complete flag set and the clean four-feature command above both produced GPU-process exits
  and failed both context requests.
- Browser-wide Vulkan and DrDc alone reproduced both failures. In this combination the first request emitted a
  diagnostic ending in `ErrorMessage = BindToCurrentSequence failed: .`.

## Verified workarounds

Open `chrome://flags/#enable-drdc` in Helium.
Set **Enables Display Compositor to use a new gpu thread.** to **Default (Disabled)**, then relaunch Helium.
The name and behavior are described in [`chrome/browser/flag_descriptions.h:829`][drdc-label].
This returns the compositor to the default Linux threading model and preserves the requested Vulkan configuration.
The probe covered WebGL and WebGL 2 rendering, with separate application and video coverage still open.

Returning **Vulkan** to **Default** also restored WebGL in the disposable profile, through ANGLE OpenGL ES.
That configuration gives up browser-wide Vulkan and is unsuitable when Vulkan needs to stay enabled.

## What does not work

- Returning only **Default ANGLE Vulkan** to **Default** still produced GPU-process exits and failed context creation.
- Reading only the GPU feature status misses the failure. The same WebGL status appeared in passing and failing runs.
- The Wayland Vulkan warning appeared in both a passing and a failing run.
  The [older video incident](helium-white-video-vulkan-wayland.md) used that warning alongside different symptoms.

## Upstream filing decision

- The reproduction requires forcing an upstream experiment that Chromium explicitly marks unsupported on Linux.
- The failing subsystem is within upstream's control. The exact native failure still requires symbolized evidence.
- Chromium's source explicitly disables the experiment on Linux. The flag UI exposes an override.
- [Helium's contribution policy][helium-policy] prohibits AI-generated issue descriptions.
- Searches for `DrDc Vulkan` in the Helium, Helium Linux and Chromium GitHub issue indexes returned no results.
  The near match in [Helium issue 1638][helium-webgl-question] discusses WebGL availability generally.
- The verified result is a profile change. There is no upstream patch or issue draft.

[drdc-flag]: https://github.com/chromium/chromium/blob/4b3d0dabb8250b6f152d2d85d79e7706e309d087/chrome/browser/about_flags.cc#L9136-L9138
[drdc-default]: https://github.com/chromium/chromium/blob/4b3d0dabb8250b6f152d2d85d79e7706e309d087/gpu/config/gpu_finch_features.cc#L270-L280
[drdc-label]: https://github.com/chromium/chromium/blob/4b3d0dabb8250b6f152d2d85d79e7706e309d087/chrome/browser/flag_descriptions.h#L829-L835
[webgl-context]: https://github.com/chromium/chromium/blob/4b3d0dabb8250b6f152d2d85d79e7706e309d087/third_party/blink/renderer/modules/webgl/webgl_rendering_context_base.cc#L787-L808
[helium-policy]: https://github.com/imputnet/helium/blob/24d2abfb7576e2d2df51e3ab15e89bcc5d2a034d/CONTRIBUTING.md#L14-L18
[helium-webgl-question]: https://github.com/imputnet/helium/issues/1638
