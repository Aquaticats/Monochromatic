# Resolve white videos and retain hardware WebGPU in Helium on Wayland

## What this proves

This procedure removes browser-wide Vulkan from Helium on Linux Wayland but keeps **Skia Graphite** selected.
It leaves video decode on its supported default path and enables Chromium's Vulkan-through-GL WebGPU interop.
The resulting configuration keeps presentation on `ANGLE_OPENGL` and `GaneshGL` while exposing hardware WebGPU.
It does not enable **Unsafe WebGPU Support**.

Chromium currently refuses to activate Graphite on Linux even when its UI flag is **Enabled**.
Keeping that selection is harmless in the tested builds,
but `chrome://gpu` still reports `Skia Graphite: Disabled` and `GaneshGL`.

The command-line bridge was validated first against disposable Helium profiles:

```console
${HOME}/AppImages/helium.appimage \
  --disable-features=Vulkan \
  --enable-features=ForceEnableWebGpuInterop
```

Equivalent persisted-flag profiles were then verified with **Skia Graphite** also selected.
Helium 0.14.9.1 and 0.15.1.1 rendered H.264 video under that arrangement.
Both completed 300 WebGPU compute iterations on an AMD Radeon RX 7600 while hardware-decoded video played.
The bridge was not applied to the active profile because the durable settings belong to Helium's flags UI.

## Setup

Status:
TODO

Use this procedure on Linux when Helium runs natively on Wayland.
Run this command in a terminal and confirm that it prints exactly `wayland`:

```console
printf '%s\n' "${XDG_SESSION_TYPE}"
```

Helium 0.15.1.1 is the newest version covered by this procedure.
If Helium is absent,
download the build for the machine's architecture from the
[Helium for Linux 0.15.1.1 release][helium-release].
For the x86-64 AppImage downloaded to `~/Downloads`,
 make it executable and start it with:

```console
chmod u+x "${HOME}/Downloads/helium-0.15.1.1-x86_64.AppImage"
"${HOME}/Downloads/helium-0.15.1.1-x86_64.AppImage"
```

Save unsent form content before continuing.
The **Relaunch** action restarts every Helium window.

Record the current values of these entries if the previous configuration must be restorable:

- **Vulkan**
- **Skia Graphite**
- **Hardware-accelerated video decode**
- **Force enable WebGPU interop**

## Steps

Status:
TODO

1.  Press **Ctrl+L**.
    Helium selects the current address.
2.  Type `chrome://flags/#enable-vulkan`.
    The address bar contains `chrome://flags/#enable-vulkan`.
3.  Press **Enter**.
    The **Vulkan** entry appears highlighted.
4.  Open the dropdown beside **Vulkan**.
    The dropdown displays **Default**,
     **Enabled**,
     and **Disabled**.
5.  Select **Default**.
    **Vulkan** displays **Default**,
     and the **Relaunch** button appears.
6.  Press **Ctrl+L**.
    Helium selects the flags address.
7.  Type `chrome://flags/#skia-graphite`.
    The address bar contains `chrome://flags/#skia-graphite`.
8.  Press **Enter**.
    The **Skia Graphite** entry appears highlighted.
9.  Open the dropdown beside **Skia Graphite**.
    The dropdown displays **Default**,
     **Enabled**,
     and **Disabled**.
10. Select **Enabled**.
    **Skia Graphite** displays **Enabled**.
11. Press **Ctrl+L**.
    Helium selects the flags address.
12. Type `chrome://flags/#disable-accelerated-video-decode`.
    The address bar contains `chrome://flags/#disable-accelerated-video-decode`.
13. Press **Enter**.
    The **Hardware-accelerated video decode** entry appears highlighted.
14. Open the dropdown beside **Hardware-accelerated video decode**.
    The dropdown displays **Default**,
     **Enabled**,
     and **Disabled**.
15. Select **Default**.
    **Hardware-accelerated video decode** displays **Default**.
16. Press **Ctrl+L**.
    Helium selects the flags address.
17. Type `chrome://flags/#force-enable-webgpu-interop`.
    The address bar contains `chrome://flags/#force-enable-webgpu-interop`.
18. Press **Enter**.
    The **Force enable WebGPU interop** entry appears highlighted.
19. Open the dropdown beside **Force enable WebGPU interop**.
    The dropdown displays **Default**,
     **Enabled**,
     and **Disabled**.
20. Select **Enabled**.
    **Force enable WebGPU interop** displays **Enabled**.
21. Click **Relaunch**.
    Every Helium window closes and reopens with the changed GPU configuration.

## What to check

Status:
TODO

Confirm all of these outcomes:

- `chrome://flags/#enable-vulkan` shows **Default**.
- `chrome://flags/#skia-graphite` shows **Enabled**.
- `chrome://flags/#disable-accelerated-video-decode` shows **Default**.
- `chrome://flags/#force-enable-webgpu-interop` shows **Enabled**.
- The **Graphics Feature Status** section of `chrome://gpu` contains `WebGPU: Hardware accelerated`.
- On Helium 0.15.1.1,
   the **Graphics Feature Status** section of `chrome://gpu` contains `Vulkan: Disabled`.
- On Helium 0.14.9.1,
   `chrome://gpu` can report `Vulkan: Hardware accelerated` for the interop device even though presentation remains GL.
- The **Graphics Feature Status** section of `chrome://gpu` contains `Skia Graphite: Disabled`.
- The **Display type** value in `chrome://gpu` is `ANGLE_OPENGL`.
- The **Skia Backend Type** value in `chrome://gpu` is `GaneshGL`.
- [WebGPU Report][webgpu-report] does not display `webgpu is not available on this browser`.
- [WebGPU Report][webgpu-report] identifies the AMD adapter and a Vulkan backend.
- A video's picture moves while its timeline advances instead of showing a solid white rectangle.
- `chrome://media-internals` records `kVideoDecoderName` as `VaapiVideoDecoder` for supported H.264 media.
- Helium's diagnostic output can contain
   `Enabling Graphite on a not-yet-supported platform is disallowed for safety`.
   That message confirms that the selected Graphite flag was refused rather than activated.
- After a Helium update,
   recheck **Skia Graphite** and **Skia Backend Type** in `chrome://gpu`.
   If they change from `Disabled` and `GaneshGL`,
   Graphite has become active and video must be retested.

Run this command while the video plays.
A passing check prints no matching lines:

```console
journalctl --user --since '5 minutes ago' --no-pager \
  | grep --extended-regexp \
    'Could not find or create a backing for stream kSkia|Trying to produce a Skia representation from an incompatible backing'
```

## Restore

Status:
TODO

Restoring the diagnosed profile's previous values deliberately recreates the unsupported configuration and can restore
white video.
Only restore it when reproducing the failure.

The diagnosed profile started with these values:

- **Vulkan**:
   **Enabled**
- **Skia Graphite**:
   **Enabled**
- **Hardware-accelerated video decode**:
   **Disabled**
- **Force enable WebGPU interop**:
   **Default**

For each entry,
open its URL from the **Steps** section,
select the recorded previous value from its dropdown,
and confirm that the entry displays that value.
Click **Relaunch** after the final change.
Every Helium window closes and reopens with the previous settings.

[helium-release]: https://github.com/imputnet/helium-linux/releases/tag/0.15.1.1
[webgpu-report]: https://webgpureport.org/
