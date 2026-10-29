# nested-wayland-session headless: Mesa 26.2 picks llvmpipe silently,
and three environment variables move the GPU under the session

## Symptom

On 2026-10-14,
while adding `--gpu` to the headless mode of `monochromatic-nested-wayland-session`,
three Mesa behaviors made the device that renders differ from the device a person asked for.
The host has an integrated AMD GPU (`/dev/dri/renderD129`, PCI `1002:164e`),
a discrete AMD GPU (`/dev/dri/renderD128`, PCI `1002:7480`),
and Mesa 26.2.4.

The surfaceless platform answers with the CPU rasterizer when it can use no render node.
No error is raised,
so a session that expected a GPU renders on the CPU:

```text
compositor device auto render node Err(QueryDeviceProperty(Unknown(0))) dev_id None GL_RENDERER llvmpipe (LLVM 22.1.8, 256 bits)
dmabuf v3 (no render node)
```

That line is the probe's output with `/dev/dri` replaced by an empty directory
(`bwrap --dev-bind / / --tmpfs /dev/dri`),
for `EGLDisplay::new(EGLSurfacelessDisplay)`.

`LIBGL_ALWAYS_SOFTWARE=1` in the compositor's own environment removes every GPU from EGL's device list:

```text
LIBGL_ALWAYS_SOFTWARE=1: device 0: GL_RENDERER=llvmpipe (LLVM 22.1.8, 256 bits)
(the list holds this one device and neither render node)
```

`DRI_PRIME` in the environment of the hosted client moves where the client allocates,
while the compositor stays on the device it opened.
With the compositor on `renderD128` and `DRI_PRIME=pci-0000_11_00_0`,
the client held these DRM nodes open:

```text
client fd 14 -> /dev/dri/renderD129
client fd 15 -> /dev/dri/renderD128
client fd 16 -> /dev/dri/renderD128
client fd 17 -> /dev/dri/renderD128
client fd 18 -> /dev/dri/renderD129
client fd 19 -> /dev/dri/renderD129
```

Without `DRI_PRIME` the same client held `renderD128` only.
`DRI_PRIME=1` and the other direction (`renderD129` with `pci-0000_03_00_0`) gave the same split.

## Root cause

Smithay 0.7.0 offers two ways to open an EGL display.
`EGLSurfacelessDisplay` asks Mesa for its platform default,
so Mesa decides the device:

```rust
// smithay-0.7.0/src/backend/egl/native.rs:264-271
impl EGLNativeDisplay for EGLSurfacelessDisplay {
    fn supported_platforms(&self) -> Vec<EGLPlatform<'_>> {
        vec![
            egl_platform!(
                PLATFORM_SURFACELESS_MESA,
                ffi::egl::DEFAULT_DISPLAY,
                &["EGL_MESA_platform_surfaceless"]
```

An `EGLDevice` is also an `EGLNativeDisplay`,
which opens that one device through `EGL_EXT_platform_device`:

```rust
// smithay-0.7.0/src/backend/egl/native.rs:236-243
impl EGLNativeDisplay for EGLDevice {
    fn supported_platforms(&self) -> Vec<EGLPlatform<'_>> {
        // see: https://www.khronos.org/registry/EGL/extensions/EXT/EGL_EXT_platform_device.txt
        vec![egl_platform!(
            PLATFORM_DEVICE_EXT,
            self.inner,
```

`EGLDevice::enumerate` (`smithay-0.7.0/src/backend/egl/device.rs:27`) lists the devices,
and `is_software` (`device.rs:240`) is the presence of `EGL_MESA_device_software`.
On this host the list has three entries:
`renderD129`,
`renderD128`,
and a software device with no render node
(`render_device_path()` fails with `QueryDeviceProperty(Unknown(0))`,
`drm_device_path()` with `EglExtensionNotSupported(["EGL_EXT_device_drm"])`).
Enumeration order is not node order,
so a node must be matched by `DrmNode::dev_id`.

The three behaviors above were measured on a built probe and on the real binary.
The Mesa source that implements them was not read,
so this document states what the binaries do and not why Mesa does it.

## Verification

Versions:
Mesa 26.2.4,
Smithay 0.7.0 from crates.io,
`slint-viewer` 1.18.1 with `SLINT_BACKEND=winit-femtovg`,
and the repository's `monochromatic-nested-wayland-session` on branch `feat/nws-headless-gpu`.

Works:

- `EGLDisplay::new(device)` for each of the three enumerated devices rendered offscreen,
  read the pixels back,
  and reported 325 dmabuf format and modifier pairs for `renderD129`,
  426 for `renderD128`,
  and 114 for the software device.
- A Slint client hosted on `renderD128` and on `renderD129` held only the chosen node open,
  and the compositor imported its dmabufs.
- With the software device,
  the client held no DRM node and drew into `wl_shm` buffers
  (`dmabuf_imports_ok=0`,
  two `wl_shm_pool.create_buffer` calls),
  the screenshots showed the client,
  and the client's WAYLAND_DEBUG log shows `mesa egl swrast display queue`.
- The IDE bundle (`package/ide/app/dist/monochromatic-ide`) on a disposable folder,
  a disposable `HOME`,
  and no network held only `renderD128` (or only `renderD129`) with dmabuf buffers,
  and with `--gpu software` held no DRM node and used `wl_shm` buffers while its file list was drawn.

Fails or surprises:

- The default surfaceless display with no usable render node returns the software device.
- `LIBGL_ALWAYS_SOFTWARE=1` in the compositor's environment leaves only the software device,
  so naming a render node cannot succeed.
- `GALLIUM_DRIVER=no-such-driver` with no render node visible makes the software device fail too
  (`eglInitialize: DRI2: failed to create screen`),
  which is the case in which every attempt fails.
- `DRI_PRIME` in the client's environment adds the other GPU's node to the client's open files.

Harness:
`mise run //package/nested-wayland-session/cli:inspect:headless-gpu` repeats the per-device and the no-GPU cases
and exits 1 on any failure.
`monochromatic-nested-wayland-session --list-gpus` prints the devices and the one `auto` uses.

## Verified workarounds

The session now handles all three:

- `auto` treats a software answer from the default display as "no default GPU" and tries the other render nodes before
  accepting software (`default_device_rejection` in `src/gpu_choice.rs`).
  Tradeoff:
  a host whose only usable device is the CPU rasterizer reaches software after the failed default attempt
  and any render node attempts,
  and logs why.
- The hosted child gets `LIBGL_ALWAYS_SOFTWARE=1` for a software session
  and has `DRI_PRIME` removed for an explicit choice or a fallback (`child_variables` in `src/gpu_choice.rs`).
  Tradeoff:
  a caller's own `DRI_PRIME` is ignored for the hosted child in those cases,
  and a plain `auto` session still passes it through unchanged.
- A failed GPU choice names `LIBGL_ALWAYS_SOFTWARE` when it is set
  (`environment_hint` in `src/gpu_device.rs`).
  Tradeoff:
  only that one variable is checked.

## What does not work

- Taking the default surfaceless display's success as "a GPU opened":
  on a machine with no usable node it is `llvmpipe`,
  and `auto` would never try the other nodes.
- Choosing a device by its index in `EGLDevice::enumerate`:
  index 0 was `renderD129`.
- Opening `/dev/dri/card0` as a device:
  a primary node has no render node entry,
  so the session refuses it and names the render node to use.
- Reading the client's GL renderer string from its log:
  `EGL_LOG_LEVEL=debug` printed no `MESA-EGL: debug` lines.
  The renderer of the client was not determined;
  the DRM nodes it holds open show where it allocates.
- Steering a Vulkan client:
  no Vulkan-only client was run,
  so whether these variables move one is not determined.

## Upstream filing decision

No upstream issue or comment is drafted.
The `.out-of-scope/` directory has no entry for Mesa or Smithay.
The behaviors are environment-variable and fallback semantics of Mesa's EGL,
and this investigation did not read Mesa's source or documentation for them,
so whether any is a defect (constraint 1),
and whether a fix is possible (constraint 2) or prototyped (constraint 6),
is not established.
The default policy applies:
do not file.
A later session that reads Mesa's `EGL_MESA_platform_surfaceless` handling can reopen this.
