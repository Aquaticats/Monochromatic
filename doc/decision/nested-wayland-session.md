# Nested Wayland session fixture: build a minimal Smithay compositor, distribute via cargo

Tracks issue #272 ("Build minimal nested Wayland session for GUI testing").
The follow-up, issue #273 (headless-surfaceless mode), was delivered in 0.5.0 as the opt-in `--headless` flag
and the issue is closed; see "Headless-surfaceless mode in the first version".

## Decision

Build a repo-owned minimal nested Wayland compositor as a single Rust binary on Smithay,
 pinned to the
crates.io release (0.7.0),
 not to Smithay git HEAD.
It hosts exactly one Slint app on the nested-winit path (a winit window inside an existing Wayland session),
takes screenshots by reading back the rendered framebuffer,
 injects synthetic input through its own seat,
and exposes a Unix-socket control API that returns machine-readable results.

Distribute it through crates.io so `mise` installs it with the `cargo:` backend,
 matching the
`forbidden-strings` precedent.
Document `cargo-binstall` in the package README so a prebuilt binary can be fetched without a source build.

Headless-surfaceless rendering (no parent display,
 for bare CI) was deferred at first and then added as an opt-in mode (0.5.0, `--headless`);
the nested window stays the default.
This decision refines issue #272's suggested shape in specifics,
 not intent.

## Why build our own instead of using an existing compositor

The recommendation is forced by a chain of measured facts and one hard constraint,
 not by preference.

### The app requires dmabuf, so the fixture must do real GPU compositing

The app under test is a Slint desktop app on the winit backend with the femtovg (OpenGL) renderer.
Running the prebuilt binary in the host's real GPU session under `WAYLAND_DEBUG=1` and counting buffer events
showed it presents through `zwp_linux_dmabuf_v1` with full v4 modifier feedback,
 never through `wl_shm`:
dmabuf buffer-param events with `zwp_linux_dmabuf_feedback` negotiation,
 and zero SHM buffer creations.
So any fixture that exercises the real GPU/winit path must import dmabuf and run a GLES renderer.
Smithay's SHM-only `smallvil` example is insufficient by construction.
Full trace in `nested-wayland-session-vet-report/spike-results.md`.

### mise-installability with no distro package manager rules out every turnkey compositor

The end goal is less friction for setups,
 and the fixture must be installable by `mise`.
On Bazzite (Fedora Atomic),
 `mise`'s system-package bootstrap uses `dnf`,
 which is the rpm-ostree overlay
friction this work exists to remove (see `doc/troubleshooting/niri-mise-cargo-install.md`).
The remaining `mise` backends (`aqua`,
 `cargo`,
 `ubi`,
 `github`,
 `http`) need a registry entry,
 a cargo/crate
source,
 or a prebuilt release binary.
None of the turnkey compositors qualify:

- None appear in `mise`'s registry.
- cage and sway ship source tarballs only,
   no prebuilt binaries.
- gamescope publishes no GitHub releases at all.
- All are C or C++ meson projects with heavy shared-library runtime graphs,
   not cargo or aqua artifacts.

The same is true of the companion tools a compose-existing solution would need (grim,
 wtype,
 ydotool):
they are distro packages,
 not `mise`-installable.
So the entire "use what exists" path fails the hard constraint,
 even though cage technically works.

### A single Rust binary is the only low-friction mise-installable option

A repo-owned Rust binary installs cleanly through the `cargo:` backend (source) with `cargo-binstall`
for prebuilt binaries,
 needs no distro package manager,
 and links only libwayland,
 libxkbcommon,
 and
libEGL/libGLESv2 at runtime,
 all present on Bazzite and virtually every Linux desktop.
It links none of wlroots,
 libinput,
 or libseat,
 so it is more portable as a distributed artifact than a
vendored cage binary would be (cage is bound to a specific `libwlroots-0.20` ABI).

### Building it ourselves is bounded work, not a compositor-from-zero slog

Smithay's `anvil` example already contains the entire hard part in one file (`anvil/src/winit.rs`):
the `zwp_linux_dmabuf_v1` v4 feedback path with `import_dmabuf`,
 the GLES/EGL renderer,
 and the render loop.
Screenshots are `ExportMem::copy_framebuffer` readback plus the `image` crate.
Input injection is direct seat `KeyboardHandle`/`PointerHandle` calls,
 scoped to the compositor,
 with no
global `/dev/uinput` involvement.
A minimal fixture is roughly 1,000 to 1,500 lines of our own auditable code over a pinned Smithay dependency,
days of contained work on the nested-winit happy path.
Owning the compositor turns three requirements (screenshot,
 input,
 control socket) from external-glue
liabilities into first-class in-process features.
Footprint detail in `nested-wayland-session-vet-report/vet-building-blocks-and-tools.md`.

## Rejected alternatives

### Compose existing tools (cage plus grim plus wtype plus a virtual-pointer client)

cage hosts the GPU app perfectly (validated end to end on the host's AMD Radeon RX 7600) and natively nails
single-app,
 fullscreen,
 and exit-on-child with exit-code propagation.
It advertises the wlr screencopy,
 virtual-pointer,
 and virtual-keyboard protocols that grim and wtype drive.
Rejected because none of cage,
 grim,
 or wtype is `mise`-installable,
 so the stack fails the hard constraint,
and it is more moving parts (there is no standard CLI for the virtual-pointer half,
 so a small custom client
is needed anyway).

### blit-compositor as a cargo dependency

A Rust crate that already implements screenshot,
 click,
 key,
 and text over a Unix-socket control protocol on
a hand-rolled Vulkan compositor with proper dmabuf support:
 functionally a superset of the fixture.
Rejected because it is a 56k-line remote-desktop and streaming product (Vulkan H.264/AV1 encode,
 WebRTC,
an embedded browser and terminal,
 871 `unsafe` sites),
 and `cargo install` is a heavy source build.
Its compositor alone carries more unsafe Vulkan FFI than our entire from-scratch fixture would,
 and it is a
low-scrutiny single-vendor project,
 so adopting it inverts the minimal-auditable-fixture goal.

### gamescope

Purpose-built single-app nested compositor with a native screenshot,
 a control protocol,
 and libei input,
already present on Bazzite.
Rejected on two counts:
 it is not `mise`-installable (no releases at all),
 and,
 decisively,
 its wlserver
advertises only legacy `wl_drm` plus its private `gamescope_swapchain`,
 not `zwp_linux_dmabuf_v1`.
The app presents through `zwp_linux_dmabuf_v1`,
 so gamescope is a risky,
 unvalidated fit for the one hard
requirement.

### weston headless and sway headless

Both are protocol-correct for the app's dmabuf path and well maintained.
weston has the cleanest headless dependency footprint (no libinput/libseat/udev) and a native screenshot;
sway has the gold-standard JSON IPC,
 built-in pointer injection,
 and runtime output resize.
Both rejected for the same reason as cage:
 source-only distribution,
 not `mise`-installable,
 and each needs
external glue (weston's input path is a tests-only module;
 sway is architecturally multi-window and needs an
exit wrapper).

### Raw wayland-server from scratch

Viable,
 but roughly 10k or more lines we would otherwise get for free from Smithay's `wayland` and renderer
modules,
 with no offsetting benefit for this scope.

### Headless-surfaceless mode in the first version

Rejected for the first version, accepted as a later opt-in mode.
The nested-winit path inherits the parent's EGL device and is anvil's known-good pattern.
Headless-surfaceless must select the DRM render node and match the client's dmabuf modifiers with no parent
device to inherit from,
 which is the fiddliest and highest-risk part.
Shipping nested-winit first de-risked the core tool.

Delivered in 0.5.0 (issue #273, closed) as `--headless`, with these decisions:

- **Opt-in only.**
  The nested window is the default in every case,
  because a person often wants to inspect the running window,
  which a headless session cannot show.
  Headless is never selected automatically,
  and a missing parent display never falls back to it;
  the start fails with a message that names `--headless`.
- **Same control surface.**
  The control protocol,
  screenshots,
  recordings,
  seat input,
  the private bus,
  and the accessibility tree are unchanged;
  only the display and render backend differ.
- **Render node.**
  Smithay's `EGLSurfacelessDisplay` opens on a render node chosen by Mesa
  (on the development host the integrated AMD GPU, `/dev/dri/renderD129`),
  renders into an offscreen target,
  and builds the dmabuf version 4 default feedback from the formats and modifiers that render node advertises
  (325 format and modifier pairs on that host),
  which removes the risk named below for this host.
  A Slint client using femtovg and one using skia both had their dmabuf buffers imported.
- **GPU choice.**
  A later addition (0.6.0) gives headless a `--gpu auto|software|NODE` option and
  `--list-gpus`.
  Smithay's `EGLDevice::enumerate` lists the EGL devices,
  `EGLDisplay::new` on a device (`EGL_EXT_platform_device`) opens a chosen render node,
  and the `EGL_MESA_device_software` device is Mesa's `llvmpipe`.
  Measured on the development host:
  both render nodes (`renderD128`, PCI `1002:7480`, and `renderD129`, PCI `1002:164e`) rendered offscreen and
  imported a Slint client's dmabufs,
  and `llvmpipe` rendered the same client through `wl_shm` buffers,
  because a software device has no render node to put in the dmabuf version 4 feedback.
  Mesa's surfaceless platform answers with `llvmpipe` when it can use no render node,
  so `auto` treats that answer as "no default GPU" and tries the other nodes before accepting software.
  An explicit choice never falls back,
  and only `auto` does,
  so a script that names a GPU never silently renders on another device or on the CPU.
  Details and the environment the hosted child gets are in the package README
  (`Choosing the GPU`) and in `doc/troubleshooting/nested-wayland-headless-gpu-choice.md`.
- **Frame pacing.**
  With no parent that presents,
  the compositor's own timer sends frame callbacks at the output refresh rate from the first tick
  (59.7 to 60.0 ticks per second measured on the client side).
- **No window reaches a parent.**
  A decoy parent socket accepted no connection during a headless session.

### ubi distribution

The `ubi` backend is deprecated for this project,
 and `cargo-binstall` (documented in the package README)
covers prebuilt-binary installs on the `cargo` path,
 so distribution stays on crates.io.

## Chosen implementation notes

- Suggested home:
   a Rust crate under `package/`,
   per issue #272 the suggested path is
  `package/nested-wayland-session/cli`,
   subject to a better category during implementation.
- Smithay pinned to crates.io 0.7.0.
  Smithay is pre-1.0 and its git HEAD is far ahead of the release with breaking changes;
   the `anvil` and
  `smallvil` examples track git,
   so copy their approach but write against the stable 0.7.0 API surface
  (`DmabufState`,
   `GlesRenderer`,
   winit init,
   `ExportMem`,
   and the seat handles all exist in 0.7.0).
- Render mode:
   nested-winit only for the first version (needs a running parent Wayland session at runtime).
- Screenshot:
   `ExportMem::copy_framebuffer` plus `map_texture`,
   encoded with the `image` crate's `png`
  feature.
- Input:
   synthetic events through the compositor's own seat (`PointerHandle::motion`/`button`,
  `KeyboardHandle::input`);
   text typing maps each char to a keysym then emits per-char press/release.
- Control API:
   a Unix socket carrying machine-readable request and response messages.
- Distribution:
   publish to crates.io;
   install with `mise use "cargo:<crate>"`;
   document `cargo-binstall`
  in the README.
- Runtime deps (all present on Bazzite):
   libwayland-client,
   libwayland-egl,
   libxkbcommon,
   libEGL,
  libGLESv2,
   libdrm.
  The winit path pulls neither gbm nor the drm allocation stack.

## Risks

- Headless-surfaceless dmabuf (render-node selection and modifier matching) was the concentrated risk,
   and it
  was deferred to issue #273 precisely because of that.
  Issue #273 closed with the mode working on the development host (two AMD GPUs, Mesa radeonsi).
  Other GPU vendors and drivers were not tried.
- Smithay pre-1.0 churn,
   mitigated by pinning crates.io 0.7.0 rather than git HEAD.
- dmabuf v4 feedback negotiation against the real Slint/femtovg client;
   anvil already builds v4 with a v3
  fallback,
   so this is low risk on radeonsi but is the first thing to smoke-test.

## Verification done before this decision

- The dmabuf requirement was measured,
   not assumed:
   see the spike results.
- cage was run end to end and confirmed to host the GPU app on real hardware.
- smallvil was built to confirm it is SHM-only and,
   incidentally,
   that a source build on this host needs
  `libxkbcommon-devel` (moot given container builds).
- Source audits of every candidate are in `nested-wayland-session-vet-report/`.
