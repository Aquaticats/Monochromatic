# GTK4 on Windows via gvsbuild: GL renderer falls back to software (DirectComposition disabled)

A gtk4-rs app built against gvsbuild's GTK on Windows renders correctly but extremely slowly,
because GTK silently falls back to the Cairo software renderer.
 The fix is one environment
variable.

## Symptom

- fps is single digits (3-6) for a scene the GPU handles trivially.
   A browser WebGL/WebGPU GPU
  benchmark on the same machine sustains 60-86 fps,
   so the hardware and driver are fine.
- stderr shows the GL renderer failing to realize and Cairo taking over:

  ```txt
  Failed to realize renderer 'GskGLRenderer' for surface 'GdkWin32Toplevel': OpenGL requires Direct Composition
  Using renderer 'GskCairoRenderer' for surface 'GdkWin32Toplevel'
  ```

- Forcing `GSK_RENDERER=vulkan` does not help:
   it reports `Unrecognized renderer "vulkan"`
  (note:
   "unrecognized",
   not "failed to realize";
   Vulkan is not compiled in at all).

## Root cause

Three gvsbuild build choices combine to leave only the software renderer usable.

1.  gvsbuild applies `patches/gtk4/0001-remove-direct-composition.patch`,
     which turns
    DirectComposition from a default-on GDK feature into an opt-in GDK debug flag.
     The device
    init in `gdk_win32_display_init_dcomp` changes from a feature check to a debug check:

    ```c
    -  if (!gdk_has_feature (GDK_FEATURE_DCOMP))                  // upstream: on by default
    +  if (!GDK_DISPLAY_DEBUG_CHECK (GDK_DISPLAY (self), DCOMP))  // gvsbuild: off unless asked
         return;
    ```

    So no DirectComposition device is ever created unless you explicitly ask for it.

2.  GTK's Win32 GL renderer hard-requires a DirectComposition device.
     In
    `gdk_win32_gl_context_surface_attach`,
     `if (!gdk_win32_display_get_dcomp_device (display))`
    aborts with "OpenGL requires Direct Composition".
     There is no env var or GDK_DEBUG flag in
    the GL path to bypass this requirement (verified by reading `gdk/win32/gdkglcontext-win32.c`).

3.  gvsbuild builds GTK with Vulkan disabled:
     `gvsbuild/projects/gtk.py` passes
    `self.add_param("-Dvulkan=disabled")`.
     So the Vulkan renderer is not compiled in and cannot
    be a fallback.

Net:
 DirectComposition off by default,
 plus GL requiring it,
 plus no Vulkan,
 means GSK falls all
the way back to `GskCairoRenderer` and software-rasterizes the whole scene.

This is not a remote-session artifact.
 It reproduces on the physical console with the display on;
the DirectComposition device is simply never created because of choice 1.

## Fix

Set `GDK_DEBUG=dcomp`,
 the flag the patch added,
 to enable the DirectComposition device so the
GL renderer realizes and runs on the GPU:

```txt
set GDK_DEBUG=dcomp
```

With it set,
 stderr shows `Using renderer 'GskGLRenderer'` and fps jumps into the GPU range.

For a shipped app,
 set it in-process before GTK initializes rather than relying on the caller's
environment:
 `std::env::set_var("GDK_DEBUG", "dcomp")` at the top of `main` (merge with any
existing value the user set).
 Scope it to Windows.

## Alternative: build GTK with the Vulkan renderer

If you would rather use Vulkan (native win32 swapchain,
 no DirectComposition dependency),
 rebuild
GTK with Vulkan enabled:
 install the Vulkan SDK (headers) and change gvsbuild's `gtk.py` to pass
`-Dvulkan=enabled`.
 The runtime loader `vulkan-1.dll` ships with the GPU driver already
(confirmed present,
 and `vulkaninfo` enumerated the Radeon 780M at Vulkan 1.4.349),
 so only
build-time headers are missing.
 Not needed once `GDK_DEBUG=dcomp` restores GL;
 recorded as the
option if a future GTK version drops the GL path or DirectComposition proves unreliable.

## Verification

Measured on `x13-win` (Ryzen 7 PRO 7840U,
 Radeon 780M,
 Windows 10 21H2),
 gvsbuild 2026.6.0,
GTK 4.22.4,
 over RustDesk:

- Without `GDK_DEBUG=dcomp`:
   `GskCairoRenderer`,
   3-6 fps.
- With `GDK_DEBUG=dcomp`:
   `GskGLRenderer`,
   up to 68 fps uncapped;
   the real sparse-board workload
  holds a locked 60.
