# nested-wayland-session

A minimal single-app nested Wayland compositor for GUI testing.
It hosts exactly one Wayland client fullscreen on the real GPU (dmabuf) path,
photographs the framebuffer as a PNG,
 injects synthetic pointer and keyboard input,
and takes all of this over a small Unix-socket control API.

It is built on [Smithay][] and installs through `cargo`,
 so an automated test can give a
GUI app a private,
 deterministic screen on any machine,
 with no distro package manager
and no system-wide changes.

By default the tool opens a nested window inside the session you are running in,
minimized on launch and restorable,
so you can look at what the hosted app is doing.
With `--headless` it renders without any display instead,
so it also runs on a bare CI machine or over SSH.
Headless is opt-in only (see [Headless mode](#headless-mode)).

The full rationale,
 the on-hardware measurements,
 and the source review of every
alternative considered are recorded in
[`doc/decision/nested-wayland-session.md`](../../../doc/decision/nested-wayland-session.md)
(issue #272).
 A plain-language overview is in
[`doc/planning/nested-wayland-session.md`](../../../doc/planning/nested-wayland-session.md).

## What it does

- Gives exactly one app a private nested screen and keeps it fullscreen.
- Runs the real GPU render path:
   it imports the client's `zwp_linux_dmabuf_v1` buffers
  (v4 modifier feedback,
   v3 fallback) into a GLES renderer,
   the same path a real user
  gets,
   so the app is checked the way it actually draws.
- Shuts down cleanly when the hosted app exits,
   propagating the app's exit code.
- Saves a screenshot of the current frame as a PNG when asked.
- Records a frame sequence at a steady 60fps that holds even when the hosted app is laggy
  or resource-hungry (with optional systemd CPU isolation of the app).
- Clicks at a point (single,
   double,
   or with modifier keys held),
   drags with intermediate motion,
   scrolls with notches,
   fractional notches,
   or touchpad pixels,
   presses any key a person can press (chords,
   raw codes,
   holds with the client's own key repeat),
   and types any Unicode text into the app.
- Hosts several windows,
   returns keyboard focus to the window underneath when a dialog closes,
   and can ask a window to close.
- Reports the cursor shape the app shows,
   and the accessibility tree it publishes.
- Moves the pointer without pressing anything,
   so the app sees real hover (enter,
   motion,
   and leave) events.
- Presses a mouse button and holds it across a pause or a movement,
   then releases it,
   so long-press and pressed-state behaviour can be tested.
- Reports the window title and app ID the app set,
   and follows each change the app makes at runtime.
- Records the taskbar progress the app publishes on the private bus
   (`com.canonical.Unity.LauncherEntry.Update`),
   so a test can check the progress bar the app would show on its taskbar icon.
- Provides an isolated clipboard through wlr-data-control and ext-data-control,
  sharing the nested seat's regular clipboard with `wl_data_device` clients.
- Changes the nested screen size.
- Sets an integer or fractional output scale at startup and switches it while the app runs,
  so the app is seen re-rendering for another display scale.
- Serves a private dark or light appearance preference and switches it while the app runs,
  without touching the host desktop theme.
- Serves a private accent color the same way,
   and changes it while the app runs.
- Records the desktop notifications and Plasma job views the app sends,
   on the private bus only,
  and sends back the close,
   action,
   pause,
   resume,
   and cancel signals a desktop would,
  so an app's notification and job progress code can be tested without a notification daemon.
  The compositor draws nothing for them.
- Keeps the hosted app drawing when its own window is hidden or the host session is locked,
  so screenshots show the app's current frame.
- Renders with no display at all when `--headless` is given,
  through EGL on a GPU render node you choose with `--gpu` or on Mesa's software rasterizer,
  with the same control commands,
  screenshots,
  recordings,
  and input.
  The nested window stays the default;
  see [Headless mode](#headless-mode) and [Choosing the GPU](#choosing-the-gpu).
- Answers every control command with a plain machine-readable `ok`/`err` line.

## Requirements

At build time (supplied by the CachyOS build container,
 see [Building](#building-and-testing)):
`wayland`,
 `libxkbcommon`,
 `libdrm`
 (on Fedora, `wayland-devel`,
 `libxkbcommon-devel`,
 and `libdrm-devel`),
 plus a Rust toolchain.

At run time (present on virtually every Linux desktop):
 `libwayland-client`,
`libwayland-egl`,
 `libxkbcommon`,
 `libEGL`,
 `libGLESv2`,
 `libdrm`,
 `dbus-daemon` when `--color-scheme`,
 `--accent-color`,
 `--notifications`,
 or `--launcher-log` is used,
 and either a running parent session
(the default is a nested winit client of an existing compositor)
or,
with `--headless`,
a DRM render node that Mesa's EGL can open
(a readable `/dev/dri/renderD*`)
or Mesa's software rasterizer,
which is `llvmpipe` (the `EGL_MESA_device_software` device)
and part of the standard Mesa packages.
With `--headless --gpu software`,
and with `--gpu auto` on a machine where no GPU can be opened,
the software rasterizer is the only requirement;
`--list-gpus` shows whether this host has it.

## Install

Through `mise` with the `cargo` backend:

```sh
mise use "cargo:monochromatic-nested-wayland-session"
```

A source build compiles the crate.
 To fetch a prebuilt binary instead of building from
source,
 use [`cargo-binstall`][]:

```sh
cargo binstall monochromatic-nested-wayland-session
```

## Usage

`--help` or `-h` prints generated help and exits successfully without opening Wayland.
`--version` reports the binary version without starting the compositor.
Clap handles parent options only until the child command begins:
`app --help` and `-- app --help` both forward `--help` to the child.
Unknown options before the child remain usage errors.
Clap uses exit 2 for usage errors instead of the former generic exit 1;
help/version use exit 0.
Joined values such as `--size=800x600` are also accepted.

```txt
monochromatic-nested-wayland-session [--socket PATH] [--size WIDTHxHEIGHT]
    [--scale SCALE] [--color-scheme dark|light] [--accent-color RRGGBB]
    [--notifications none|freedesktop|kde] [--launcher-log] [--accessibility]
    [--repeat-rate PER_SECOND] [--repeat-delay MS] [--isolate]
    [--app-cpu-quota PCT] [--app-cpu-weight N] [--headless [--gpu auto|software|NODE]] [--] COMMAND [ARG...]
monochromatic-nested-wayland-session --list-gpus
```

- `--socket PATH` enables the control API on a Unix socket at `PATH`.
   Omitted,
   the tool
  just hosts the app with no control channel.
- `--size WIDTHxHEIGHT` sets the initial nested-screen size in logical pixels (default `1280x720`).
  At the default scale 1 a logical pixel is a physical pixel.
- `--scale SCALE` sets the initial output scale (default `1`),
  from `0.5` to `3` in steps of 1/120,
   such as `1.25`,
   `1.5`,
   or `2`.
  The `scale` control command switches it while the client runs;
  see [Change the output scale](#change-the-output-scale).
- `--color-scheme dark|light` gives the hosted client a private XDG Settings portal
  with deterministic appearance.
  It does not change the host desktop theme.
  The `color-scheme` control command switches it while the client runs.
- `--accent-color RRGGBB` gives the hosted client a private XDG Settings portal that serves this accent color
  as six hexadecimal digits,
   such as `3584E4`.
  It starts the same portal as `--color-scheme`,
   and the two options can be combined.
  The `accent-color` control command changes it while the client runs;
  see [Set the accent color](#set-the-accent-color).
- `--notifications none|freedesktop|kde` chooses which recording notification services the private bus serves.
  `none` is the default:
   the bus offers no notification service,
   as before,
  and a client's call to one gets `org.freedesktop.DBus.Error.ServiceUnknown`.
  `freedesktop` serves `org.freedesktop.Notifications`.
  `kde` serves that and Plasma's job view server `org.kde.kuiserver`.
  The `notification*` and `job*` control commands read and drive them;
  see [Observe notifications and Plasma jobs](#observe-notifications-and-plasma-jobs).
- `--launcher-log` gives the hosted client the private session bus without a color scheme,
  and records the taskbar progress the client publishes on it.
  The `launcher`,
   `launcher-log`,
   and `launcher-clear` control commands read that record;
  see [Observe taskbar progress](#observe-taskbar-progress).
  The record is kept whenever the private bus runs,
   so `--color-scheme` alone also records it.
- `--accessibility` serves a private AT-SPI accessibility bus on the private session bus.
  The `a11y` control command then reads the tree the app publishes;
  see [Read the accessibility tree](#read-the-accessibility-tree).
- `--repeat-rate PER_SECOND` (default `25`) and `--repeat-delay MS` (default `200`) are the key repeat pair
  announced to clients with `wl_keyboard.repeat_info`.
  A rate of `0` tells clients not to repeat;
  the protocol then leaves repeating to the compositor,
  and this one sends no repeats.
  The `repeat-info` control command changes the pair while the app runs;
  see [Keyboard input](#keyboard-input).
- `--isolate` launches the hosted app inside a resource-controlled systemd scope so a
  greedy app cannot starve the capture pipeline (see [60fps recording](#60fps-recording)).
  It degrades to a direct launch,
   with a warning,
   when systemd is unavailable.
- `--app-cpu-quota PCT` overrides the app's CPU cap under `--isolate`,
   in percent of one
  core (`800` means eight cores' worth).
   Default:
   leave roughly a quarter of the machine
  free for the compositor.
- `--app-cpu-weight N` overrides the app's systemd `CPUWeight` (1 to 10000) under
  `--isolate`.
   Default:
   a low weight,
   so the app yields to the compositor under contention.
- `--headless` renders with surfaceless EGL on the GPU render node instead of a nested window.
  No parent display is used or needed,
  and no window appears anywhere.
  It is never chosen automatically;
  see [Headless mode](#headless-mode).
- `--gpu GPU` chooses the device a headless session renders on,
  and is valid only together with `--headless`.
  `auto` (the default) is today's behavior with a safety net:
  Mesa's default render node,
  then the other render nodes,
  then software rendering.
  `software` is Mesa's CPU rasterizer.
  A path such as `/dev/dri/renderD128` names one render node.
  An explicit choice never falls back to another device.
  The nested window renders on the GPU of its parent compositor,
  which this tool cannot choose,
  so `--gpu` without `--headless` is an error that says so;
  see [Choosing the GPU](#choosing-the-gpu).
- `--list-gpus` prints the devices `--gpu` accepts and exits,
  with no client command and no display needed.
- `COMMAND [ARG...]` is the single client to host.
   Everything after `--` (or the first
  non-flag token) is the command,
   run with `WAYLAND_DISPLAY` pointed at the nested socket.

Host an app and drive it from a script:

```sh
# Terminal 1: start the compositor with a control socket.
monochromatic-nested-wayland-session --socket /tmp/nws.sock --size 800x600 -- my-app

# Terminal 2: drive it (any tool that speaks a Unix socket works; nc shown here).
printf 'screenshot /tmp/before.png\n' | nc -U /tmp/nws.sock   # => ok
printf 'type hello\nkey enter\n'      | nc -U /tmp/nws.sock   # => ok\nok
printf 'resize 500 400\n'             | nc -U /tmp/nws.sock   # => ok
printf 'screenshot /tmp/after.png\n'  | nc -U /tmp/nws.sock   # => ok
printf 'quit\n'                       | nc -U /tmp/nws.sock   # => ok
```

The socket is owner-only:
its mode is `0600`,
whatever the umask of the process that starts the session,
so only the user who started it can connect.
The `/tmp` path above is a convenience for a single-user machine;
the mode,
not the directory,
is what keeps other local users out.
See [Who can connect](#who-can-connect).

### Headless mode

`--headless` hosts the app with no parent display.
The compositor opens `EGL_MESA_platform_surfaceless` on the DRM render node,
renders into offscreen buffers,
and imports the client's `zwp_linux_dmabuf_v1` buffers on the GPU exactly as the nested window does.
It opens no window,
so nothing ever appears on the desktop of the machine it runs on,
and it runs from a bare TTY,
an SSH session,
or a CI job that has no `WAYLAND_DISPLAY` or `DISPLAY`.
The control socket,
`screenshot`,
`record`,
seat input,
the private session bus,
and the accessibility tree behave as in the nested window.
Only the display and render backend differ.

```sh
# No WAYLAND_DISPLAY, no DISPLAY: the app is drawn on the GPU and photographed.
env --unset=WAYLAND_DISPLAY --unset=DISPLAY \
  monochromatic-nested-wayland-session --headless --socket /tmp/nws.sock --size 800x600 -- my-app
```

Headless is opt-in,
and the nested window stays the default in every case,
including when the tool is started from a script.
The nested window is minimized on launch but can be restored,
which is the way to see what a hosted app is doing while a test or a repro runs.
A headless session has no window to restore,
so the only view of it is a `screenshot` or a `record`.
For that reason the tool never switches to headless by itself:
with no parent display and no `--headless`,
the start fails and the message names `--headless` as an option.
An explicit flag rather than detection is also the only choice that behaves the same for a script run from a desktop
and the same script run in CI:
a flag is visible in the command line,
whereas a choice made from whichever display variables happen to be set would change what a command does
depending on where it runs.
A command line without `--headless` behaves exactly as before.

When `--headless` is given and a parent display variable is set anyway,
the parent is left unused and the log says so.
The hosted app never sees the parent display either way:
the compositor strips `WAYLAND_SOCKET`,
`DISPLAY`,
and `XAUTHORITY` from the app's environment and points `WAYLAND_DISPLAY` at its own socket.

Which device renders is chosen with `--gpu`;
without it Mesa's default render node is used as before,
and the log line `headless: surfaceless EGL display opened` names the node.
See [Choosing the GPU](#choosing-the-gpu) for the other choices,
the software rasterizer,
and what happens on a machine with no usable GPU.

Use `mise run //package/nested-wayland-session/cli:inspect:headless` to verify the built release binary.
The task removes every parent display variable from the compositor's environment,
hosts a Slint client with the `winit-femtovg` renderer,
and checks the screenshots,
a click that changes the second screenshot,
`resize` and `scale`,
the frame callback rate,
that a decoy parent socket receives no connection,
and that a start with no display and no flag fails with a message naming `--headless`.
It needs `slint-viewer`.

### Choosing the GPU

A headless session has no parent compositor whose GPU it could inherit,
so `--gpu` names the device:

```sh
# List what this host offers, then pick one.
monochromatic-nested-wayland-session --list-gpus
monochromatic-nested-wayland-session --headless --gpu /dev/dri/renderD128 --socket /tmp/nws.sock -- my-app
monochromatic-nested-wayland-session --headless --gpu software --socket /tmp/nws.sock -- my-app
```

`--list-gpus` prints one line per device.
Each line starts with the value to pass to `--gpu`,
then the PCI id and kernel driver from sysfs and the Mesa renderer string,
for example `/dev/dri/renderD128 pci=1002:7480 driver=amdgpu renderer=AMD Radeon RX 7600 (radeonsi, ...)`,
and `software renderer=llvmpipe (LLVM 22.1.8, 256 bits)`.
A device Mesa lists but cannot open shows `unusable:` and the reason.
The last line names the device `auto` uses on this host.

The three choices are:

- `auto` is the default and,
  on a machine with a usable GPU,
  behaves as headless did before the option existed.
  Mesa's default render node is tried first.
  If it cannot be opened or initialized,
  or if Mesa answers with its CPU rasterizer because it found no node it can use,
  the other render nodes are tried in path order,
  and software rendering is the last resort.
  Only `auto` falls back.
  When it does,
  the log says so at info level with every attempt and why it failed:
  `headless: auto fell back because earlier devices could not be used`,
  followed by `headless: rendering on this device`.
- `software` renders with Mesa's `llvmpipe` on the CPU and opens no GPU.
  It starts only if Mesa lists a software device;
  otherwise the start fails and says so.
- A render node path such as `/dev/dri/renderD128` renders on that GPU.
  The path is matched to EGL's devices by device number,
  so a `/dev/dri/by-path/...-render` symlink works.
  A primary node such as `/dev/dri/card0` is refused with the render node to use instead.
  If the node cannot be used,
  the start fails with a message that names the node,
  the reason,
  and the devices this host offers.
  An explicit choice never falls back to another device,
  so a script that asks for one GPU never silently renders on another or on the CPU.

When every attempt of `auto` fails,
the start fails and lists each attempt in order with its reason.
The `gpu` control command answers which device is in use:
`ok device=/dev/dri/renderD129 choice=auto fallback=no renderer=...`,
where `device` is a render node path or `software`,
`fallback` is `yes` when `auto` used a later attempt than the first,
and the renderer string comes last because it contains spaces.
The nested window answers `device=parent choice=parent`.

The hosted client renders on the same device as the compositor.
A client that uses EGL on Wayland follows the `zwp_linux_dmabuf_v1` default feedback,
which names the compositor's render node,
so it allocates its buffers on that GPU.
Two environment variables of the caller would pull it elsewhere,
and the compositor sets the child's environment from the final choice:

- `software` sets `LIBGL_ALWAYS_SOFTWARE=1` for the child.
  A software device has no render node to name,
  so the compositor advertises `zwp_linux_dmabuf_v1` version 3 without feedback,
  and the client's Mesa draws on the CPU into `wl_shm` buffers,
  which the compositor composites like any other client buffer.
  The software device lacks `EGL_WL_bind_wayland_display`,
  so the legacy `wl_drm` global is not offered either,
  and no client is pointed at a DRM node.
- An explicit choice,
  or a fallback,
  removes `DRI_PRIME` from the child's environment.
  With `DRI_PRIME` set in the compositor's environment,
  the client on the test host held DRM nodes of both GPUs open,
  while without it the client held only the compositor's node.
  `auto` without a fallback leaves the child's environment exactly as it was.

Measured on a host with an integrated AMD GPU,
which is `renderD129` with PCI id `1002:164e`,
a discrete AMD GPU,
which is `renderD128` with PCI id `1002:7480`,
and Mesa 26.2:
each node rendered offscreen and imported a Slint client's dmabufs,
the client held open only the chosen node,
and the software device rendered the same client into `wl_shm` buffers with no DRM node open.
`inspect:headless-gpu` repeats this for every device the host lists.

Limits:

- A client that picks its own Vulkan adapter (a `wgpu` or Vulkan renderer) is not steered by these variables,
  and this was not measured.
  A client that uses EGL or OpenGL on Wayland follows the compositor.
- `LIBGL_ALWAYS_SOFTWARE=1` in the compositor's own environment makes Mesa list only the software device,
  so a GPU choice then fails and the message says why.
- The listing and the choice cover the devices Mesa's EGL lists.
  A GPU without a Mesa driver does not appear.

### Isolated clipboard

Hosted clients receive the nested `WAYLAND_DISPLAY` without inherited `WAYLAND_SOCKET`,
`DISPLAY`,
or `XAUTHORITY`.
This prevents clipboard libraries from falling back to the host X11 clipboard.
No host clipboard synchronization is provided.
Clients connected to the nested socket can exchange text and other MIME payloads.
Primary selection is not advertised,
and clipboard data is not persisted after its owner exits.

Use `mise run //package/nested-wayland-session/cli:inspect:clipboard` to verify the built release binary.
The task requires `wl-copy` and `wl-paste`,
launches a separate nested session,
checks text and binary round-trips,
clears each selection,
and checks clean producer shutdown.
It asserts child display isolation before touching any clipboard.

### Isolate dark and light appearance

Use explicit color schemes for GUI snapshots:

```sh
monochromatic-nested-wayland-session --color-scheme dark -- my-app
monochromatic-nested-wayland-session --color-scheme light -- my-app
```

The fixture starts a private session bus and minimal `org.freedesktop.portal.Settings` service,
then sets `DBUS_SESSION_BUS_ADDRESS` only for the hosted client.
The parent compositor and desktop environment remain untouched.
The private bus intentionally does not expose unrelated host session services,
such as file-chooser portals.
Notification services exist on it only when `--notifications` asks for them,
and then they are the recording fakes described in
[Observe notifications and Plasma jobs](#observe-notifications-and-plasma-jobs),
never the host's.
It also cannot start any:
its configuration has no service directories and no includes,
so a client's request to start a service fails at once with `org.freedesktop.DBus.Error.ServiceUnknown`,
and no program named in a host `.service` file runs.
Slint asks for the accessibility bus `org.a11y.Bus` this way and continues without it,
unless `--accessibility` serves one.
Use this option for deterministic appearance tests whose client does not need those services.
Without `--color-scheme`,
 `--accent-color`,
 `--notifications`,
 `--launcher-log`,
 or `--accessibility`,
the hosted client inherits its usual session bus.
With `--launcher-log` alone the client gets the same private bus but no Settings portal,
so a toolkit that asks for appearance receives `ServiceUnknown` there.

Normal shutdown removes the private socket directory.
A forced `SIGKILL` bypasses process cleanup and can leave its PID-named temporary directory behind.

#### Switch appearance while the client runs

Start with a control socket and either explicit value,
then send `color-scheme` on the socket:

```sh
monochromatic-nested-wayland-session --socket /tmp/nws.sock --color-scheme dark -- my-app

printf 'color-scheme light\n' | nc -U /tmp/nws.sock   # => ok changed
printf 'color-scheme light\n' | nc -U /tmp/nws.sock   # => ok unchanged
```

A change replaces the value the private portal serves through `ReadOne`,
`Read`,
and `ReadAll`,
then emits `org.freedesktop.portal.Settings.SettingChanged` on the private bus
with the arguments `org.freedesktop.appearance`,
`color-scheme`,
and the new value as a variant holding an unsigned integer
(`1` for dark,
`2` for light).
Requesting the value already served answers `ok unchanged` and emits nothing,
because the portal defines that signal as emitted when a setting changes.

The command only ever uses the private portal started by `--color-scheme`.
A session started without that option serves no color scheme,
even when `--launcher-log` started the private bus or `--accent-color` started the portal,
so the command answers `err` and names the option to add;
it never falls back to the session bus the compositor itself inherited.

Use `mise run //package/nested-wayland-session/cli:inspect:color-scheme` to verify the built release binary.
The task requires `gdbus`,
`dbus-daemon`,
`dbus-monitor`,
`dbus-send`,
and `slint-viewer`.
It hosts a Slint scene that reports every `Palette.color-scheme` value the toolkit applies,
watches the private bus,
switches in both directions,
and checks screenshots and recorded frames.
The compositor's own session-bus environment points at a disposable decoy bus for the run,
and the task requires that the decoy saw no connection and no portal signal.
It prints the directory holding its evidence.

Never change the host desktop theme to produce fixture screenshots.
Use this option for isolated dark and light scenes.
See [`doc/troubleshooting/slint-nested-color-scheme-portal.md`](
../../../doc/troubleshooting/slint-nested-color-scheme-portal.md)
for Slint's D-Bus lookup and the isolation design.

### Set the accent color

`--accent-color RRGGBB` makes the private appearance portal answer `org.freedesktop.appearance` `accent-color`.
The value is exactly six hexadecimal digits in either case,
 with no `#` and no alpha:

```sh
monochromatic-nested-wayland-session --socket /tmp/nws.sock --accent-color 3584E4 -- my-app

printf 'accent-color FF7800\n' | nc -U /tmp/nws.sock   # => ok changed
printf 'accent-color FF7800\n' | nc -U /tmp/nws.sock   # => ok unchanged
```

The portal serves the setting as the XDG `(ddd)` structure of three doubles between 0 and 1,
each channel divided by 255,
 so `FF7800` is `(1.0, 0.47058823529411764, 0.0)`.
This is the shape Slint's winit backend reads (`xdg_desktop_settings.rs`,
 `downcast_ref::<(f64, f64, f64)>()`),
through `ReadOne` at startup and `SettingChanged` afterwards.
A change emits `SettingChanged` with the arguments `org.freedesktop.appearance`,
 `accent-color`,
and the structure as a variant (wire signature `ssv`).
Asking for the color already served answers `ok unchanged` and emits nothing.

Each setting is served only when its own option was given.
A session started with `--accent-color` alone answers the color scheme with `NotSupported`
and refuses `color-scheme`,
 which names `--color-scheme`.
A session started with `--color-scheme` alone does the same for the accent color,
as it always did,
 and `accent-color` answers `err` naming `--accent-color RRGGBB`.
Neither verb falls back to the session bus the compositor itself inherited.

Use `mise run //package/nested-wayland-session/cli:inspect:accent-color` to verify the built release binary.
It requires `gdbus`,
 `dbus-daemon`,
 `dbus-monitor`,
 `dbus-send`,
 and `slint-viewer`.
It hosts a Slint scene that reports the hue of `Palette.accent-background`,
switches the color,
 and checks the setting,
 the bus signal,
 and the toolkit's hue,
with the same decoy session bus as the color-scheme task.

### Change the output scale

Start at a scale,
or switch it while the client runs:

```sh
monochromatic-nested-wayland-session --socket /tmp/nws.sock --size 1100x660 --scale 1.25 -- my-app

printf 'scale 2\n' | nc -U /tmp/nws.sock   # => ok changed
printf 'scale 2\n' | nc -U /tmp/nws.sock   # => ok unchanged
```

Sizes map the way they do on a desktop with a scaled output:

- `--size` and `resize` set the logical size,
  which the hosted window is configured with.
- The physical framebuffer,
  which `screenshot` and `record` capture,
  is the logical size times the scale,
  rounded half away from zero as `wp_fractional_scale_v1` prescribes for toplevel surfaces.
  `--size 1100x660` captures 1100x660 at scale 1,
  2200x1320 at scale 2,
  and 1375x825 at scale 1.25.
- `click`,
  `wheel`,
  and `drop-file` take logical coordinates at every scale,
  so one command hits the same element whatever the scale.
  Screenshot pixel (x,
   y) is the logical point (x / scale,
   y / scale).
- `scale` keeps the logical size,
  so the client sees a scale change without a resize,
  the way a window keeps its size when it moves to an output with another scale.
  `resize` keeps the scale.

A `resize` or `scale` that changes the screen while a recording runs also answers the frame requests of the hosted
client at once.
It sends the frame callbacks,
and flushes,
right after the new configure and the new scale,
so the client can draw and commit the new size before the next capture instead of one capture later.
Without it the recorder tick was the only sender of frame callbacks while recording,
and a toolkit on winit (Slint,
for one) holds its redraw until the callback of its previous frame arrives,
so a size that reached the client between two ticks was drawn after the next capture.
That capture then showed the previous buffer on a canvas of the new size.
A command that changes nothing,
such as `scale` to the scale already set,
sends no callbacks.
Each callback is answered once,
so the following tick finds none until the client asks again.
Outside a recording nothing changes:
the live path already sends them after every presentation.
Measured with the Slint IDE hosted and a 16 ms `resize` drag (1100 to 600 to 1100 pixels,
two sweeps) recorded at
60 fps,
with no dropped frame and a load below 20 at the start,
the share of captures whose held buffer was not the width of the output mode fell
from 88.1 to 92.7 percent (8 runs) to 45.2 to 56.0 percent (7 of 8 runs;
the eighth stayed at 91.5 percent,
with a
median commit delay of 34 ms and 141 frames recorded instead of 143,
cause not determined).
In the other runs,
the median delay from the command to the commit of that width fell
from 11 to 13 ms to 6 to 7 ms.
What remains was not analyzed further;
the application's own work against a 16.7 ms tick is a part of it.
The runs and the method are in
[`doc/troubleshooting/nested-wayland-recording-previous-resize-step.md`](
../../../doc/troubleshooting/nested-wayland-recording-previous-resize-step.md).

Scales run from 0.5 to 3 in steps of 1/120,
which covers the 50 % to 300 % in 5 % steps that KDE's display settings offer.
A value between two steps is refused with the two nearest steps named,
because `wp_fractional_scale_v1` sends a scale as a whole number of 120ths.

Hosted clients learn the scale three ways:

- `wp_fractional_scale_v1.preferred_scale` on every surface,
  in 120ths,
  as soon as the client asks for a surface's fractional-scale object and again on every change.
- `wp_viewporter`,
  through which a fractional-scale client states its logical size.
  winit binds it only when the fractional-scale manager exists,
  and from then on ignores integer scales,
  so both are advertised.
- `wl_output.scale`,
  the scale rounded up to a whole number,
  and the `xdg_output` logical size,
  for clients without fractional scaling.

Slint's winit backend applies the fractional value as a scale-factor change,
unless `SLINT_SCALE_FACTOR` in the client's environment overrides it.

`scale` answers as soon as the surfaces have been told;
the client redraws afterwards.
Until its next commit,
a capture shows the client's previous buffer drawn at the new scale,
uniformly soft after a switch to a larger scale,
exactly as a desktop compositor shows a window that has just moved to another output.
Take evidence after the client has settled,
for example once two consecutive screenshots are identical.

The nested window on the host is the framebuffer.
The window is minimized as soon as it exists,
 on every launch,
 so the fixture never covers the desktop.
Winit sends `xdg_toplevel.set_minimized` on Wayland and cannot undo it there;
the parent compositor may ignore the request.
A minimized window is not presented,
 so captures rely on the pacing described in the Frame pacing section.
A new size or scale asks the parent compositor for a window that covers the physical size,
counted in the parent's own logical pixels and rounded up,
then applies the size the window has.
winit resizes a normal window by itself,
without waiting for the parent
(winit 0.30.13 `src/platform_impl/linux/wayland/window/state.rs`,
`request_inner_size`),
so this takes effect before the command answers.
By that source it does not need the parent to send anything,
which a locked host session does not;
that case was not exercised,
because the host was unlocked for every measurement.
A parent at a scale above 1 can only make windows whose sides are multiples of its scale.
Captures do not depend on that:
the output mode is the requested logical size times the scale,
rounded as `wp_fractional_scale_v1` prescribes,
whatever size the parent gave the window,
and screenshots and recordings are rendered off screen at the mode size.
`resize 599 400` therefore captures exactly 599x400 and tells the client 599x400,
where earlier versions captured the parent's window size (600 wide on a parent at scale 2).
Only the minimized window on the host shows a different size,
and the parent keeping it at a size of its own,
for example maximized or tiled there,
is logged as a warning and is no longer an error.
The host output's own scale never becomes the nested scale,
so captures do not depend on which host output shows the window.

Use `mise run //package/nested-wayland-session/cli:inspect:scale` to verify the built release binary.
The task needs `slint-viewer`.
It hosts a Slint scene that reports the scale Slint applies,
its logical size,
and each click position.
It switches through 1,
2,
1.25,
1.5,
1,
and 2,
then resizes,
and starts a second session with `--scale 1.5`.
At each step it checks the client's report,
the `preferred_scale` and `wl_output.scale` events in the client's own protocol log,
the screenshot size,
where the client's drawing ends in a recorded frame,
and that a click at a logical point reaches that point.
It also requires that no scale switch resized the client,
and that a frame recorded after a switch made during a recording has the new size and the client's drawing.
It prints the directory holding its evidence.

Use `mise run //package/nested-wayland-session/cli:inspect:resize-callbacks` to verify the early callbacks
on the built release binary.
The task needs `slint-viewer`.
It hosts an animating Slint scene,
records at 2 fps,
and makes four resizes,
each 150 ms after a recorder tick.
From the client's own `WAYLAND_DEBUG` protocol log it requires that the first frame callback follows
the configure within 100 ms (a callback left for the next tick comes about 350 ms later),
that the callbacks come together in one burst,
and that no request is answered twice.
It also requires that the first frame recorded after each command has the new size
and the client's drawing at the right edge.
It prints the directory holding its evidence and the client's protocol log.

### Observe taskbar progress

Start the session with `--launcher-log`,
 or with `--color-scheme`,
 to start the private session bus:

```sh
monochromatic-nested-wayland-session --socket /tmp/nws.sock --launcher-log -- my-app
```

The hosted app inherits `DBUS_SESSION_BUS_ADDRESS` pointing at that private bus.
A taskbar progress sender emits `com.canonical.Unity.LauncherEntry.Update` there,
with the URI of its launcher entry and the properties it changed.
The compositor merges each update into the properties it last knew for that URI,
and appends the merged state to a history of the newest 4096 updates.

Senders may send only the properties that changed.
A property left out keeps the value that URI last sent,
and a property of the wrong D-Bus type is ignored with the previous value kept.
A URI's first update starts from progress `0`,
 hidden,
 count `0`,
 hidden,
 and not urgent.

The worked example below emits the update from the hosted app's own environment,
which is where the private bus address is valid:

```sh
# Inside the hosted app: its environment already points at the private bus.
gdbus emit --session --object-path /com/canonical/unity/launcherentry/1 \
  --signal com.canonical.Unity.LauncherEntry.Update \
  'application://my-app.desktop' "{'progress': <0.25>, 'progress-visible': <true>}"

# From the test, on the control socket:
printf 'launcher\n' | nc -U /tmp/nws.sock        # => ok uri=application://my-app.desktop progress=0.2500 visible=true count=0 count-visible=false urgent=false age-ms=8
printf 'launcher-log 5\n' | nc -U /tmp/nws.sock  # => ok 1 t=1523 progress=0.2500 visible=true
printf 'launcher-clear\n' | nc -U /tmp/nws.sock  # => ok
```

The `launcher` answer is `ok none` before any update has arrived.
Its `uri` field is percent-encoded,
 so a space or `;` in the URI cannot split the line.
The `age-ms` field is the time since that update,
 on the same clock as `t`.
`t` in a `launcher-log` record is the milliseconds since the compositor started.

The `launcher-log [N]` answer,
 default `N` of 64,
 starts with the number of records on that line,
then lists them oldest first separated by `; `,
so `ok 2 t=10 progress=0.1000 visible=true; t=20 progress=0.2000 visible=true` lists two updates.
`ok 0` means the history is empty.
`launcher-clear` empties the history and answers `ok`.

All three answer `err no private session bus (start with --launcher-log or --color-scheme)`
when the session has no private bus.

Use `mise run //package/nested-wayland-session/cli:inspect:launcher` to verify the built release binary.
It hosts a Slint client that emits these updates from inside the session and checks each answer
against the values it sent,
 in order.

### Observe notifications and Plasma jobs

An app that reports progress or alerts through the desktop talks to `org.freedesktop.Notifications`,
and on Plasma to the job view server `org.kde.kuiserver`.
`--notifications` serves recording fakes of them on the private bus,
 so a test can see exactly what the app sent
and answer with the signals a desktop would send back:

```sh
monochromatic-nested-wayland-session --socket /tmp/nws.sock --notifications kde -- my-app
```

The mode is a three-way switch:

- `none` (the default) serves nothing.
  A client's call to either service gets `org.freedesktop.DBus.Error.ServiceUnknown`,
   exactly as before this option existed.
  A session without any option that starts the private bus gives the client its usual session bus instead.
- `freedesktop` serves `org.freedesktop.Notifications` at `/org/freedesktop/Notifications`.
- `kde` serves that,
   and also `org.kde.kuiserver` and `org.kde.JobViewServer` at `/JobViewServer`.
  One connection owns every name,
   as Plasma's single process does.

The compositor draws nothing for notifications or jobs.
It is a recorder:
 nothing appears on the nested screen,
 no timer expires a notification,
and no user can press a button.
A test plays the user with the control commands below.
The host session bus is never touched,
 and no service directory exists on the private bus,
so a real notification daemon is never started.

#### Notifications

`org.freedesktop.Notifications` records every `Notify`,
 `CloseNotification`,
 and signal it sends,
and answers the way Plasma 6.7.5's `server_p.cpp` does where the specification leaves room:

- `Notify` allocates ids from `1` upward and never returns `0`.
  A `replaces_id` above `0` is returned unchanged,
   whether or not that id is visible.
  A visible notification with that id is replaced in place and counts the replacement;
  an unknown id becomes a new visible notification with that id.
  The `expire_timeout`,
   the hints,
   and the actions are recorded.
  Nothing ever expires on its own,
   so `NotificationClosed` with reason `1` comes only from `notification-close`.
- `CloseNotification` removes the notification when it is visible and always answers with
  `NotificationClosed(id, 3)`,
   even for an id that is not visible,
   with no error,
   as Plasma does.
- `GetCapabilities` reports `body`,
   `body-hyperlinks`,
   `body-markup`,
   `body-images`,
   `icon-static`,
   `actions`,
  `persistence`,
   `x-kde-urls`,
   `x-kde-origin-name`,
   and `x-kde-display-appname`.
  That is the list the live host's Plasma 6.7.5 reports,
   minus `inline-reply`,
   `sound`,
   and `inhibitions`,
  because the recorder plays no sound,
   has no reply field,
   and has no `Inhibit` method.
- `GetServerInformation` reports the name `monochromatic-nested-wayland-session`,
   the vendor `Monochromatic`,
  this package's version,
   and the specification version `1.2` that Plasma reports.

Control commands,
 with the same escape rules as `launcher-log`:

- `notifications` lists the visible notifications,
   oldest first.
- `notification-log [N]` lists the newest `N` records (default 64),
   oldest first.
  It records every `Notify` call,
   every `CloseNotification` call,
   and every signal the fake sent.
- `notification-clear` forgets the visible notifications and the log,
   emits nothing,
   and keeps the id counter,
  so an id handed out earlier is never reused.
- `notification-close ID [REASON]` removes a visible notification and emits `NotificationClosed(ID, REASON)`.
  `REASON` is `1` to `4` or `expired`,
   `dismissed`,
   `closed`,
   `undefined`,
   and defaults to `dismissed` (`2`).
  An id that is not visible answers `err no visible notification with id N`.
- `notification-action ID KEY` emits `ActionInvoked(ID, KEY)` for a visible notification that offered `KEY`.
  `KEY` is the rest of the line,
   so a key with spaces works.
  An unoffered key answers `err notification N offers no action KEY (offered: ...)`.
  The notification stays visible,
   because closing it is a separate decision a desktop makes by its own rules.

Answers list records the way `launcher-log` does:
`ok <count>` followed by the records separated by `; `,
 and `ok 0` for none.
A record is space-separated `key=value` fields,
 and every value an app chose is percent-encoded
(everything but ASCII letters,
 digits,
 `-`,
 `.`,
 `_`,
 `~` becomes `%XX`),
so a summary with a space,
 `;`,
 `=`,
 or a line break cannot split a record or end the answer.
An empty value is written `key=`.

A visible notification has the fields `id`,
 `app`,
 `icon`,
 `summary`,
 `body`,
 `actions`,
 `hints`,
 `expire`,
`replaced`,
 and `age-ms`.
`actions` is `key=label` items separated by `,`.
`hints` is `key:signature:value` items separated by `,`,
 sorted by key,
so a test sees the D-Bus type the app sent (`value:i:50` is an `int32`) and not only the number.
A container hint shows only `array-N`,
 `dict-N`,
 or `struct`,
 never its contents,
 so image pixels stay out of the answer.
`replaced` counts how many later calls replaced the notification,
 and `age-ms` is the time since it was last set.

A log record starts with `t=<milliseconds since the compositor started>` and a kind:

- `notify id=N replaces=R app=... icon=... summary=... body=... actions=... hints=... expire=...`
- `close-call id=N known=true|false`
- `closed id=N reason=R` for a `NotificationClosed` the fake emitted
- `action-invoked id=N key=...` for an `ActionInvoked` the fake emitted

The worked example below emits from the hosted app's own environment,
which is where the private bus address is valid:

```sh
# Inside the hosted app: its environment already points at the private bus.
busctl --address="$DBUS_SESSION_BUS_ADDRESS" call org.freedesktop.Notifications /org/freedesktop/Notifications \
  org.freedesktop.Notifications Notify susssasa{sv}i 'My App' 0 dialog-information 'Analysis started' 'Body text' \
  2 default Open 2 urgency y 2 value i 50 5000                                  # => u 1

# From the test, on the control socket:
printf 'notifications\n' | nc -U /tmp/nws.sock
# => ok 1 id=1 app=My%20App icon=dialog-information summary=Analysis%20started body=Body%20text actions=default=Open hints=urgency:y:2,value:i:50 expire=5000 replaced=0 age-ms=4
printf 'notification-action 1 default\n' | nc -U /tmp/nws.sock    # => ok (the app receives ActionInvoked)
printf 'notification-close 1 expired\n' | nc -U /tmp/nws.sock     # => ok (the app receives NotificationClosed 1 1)
printf 'notification-log\n' | nc -U /tmp/nws.sock
# => ok 3 t=119 notify id=1 replaces=0 app=My%20App ... expire=5000; t=124 action-invoked id=1 key=default; t=126 closed id=1 reason=1
```

Use `mise run //package/nested-wayland-session/cli:inspect:notifications` to verify the built release binary.
It requires `busctl`,
 `gdbus`,
 `dbus-daemon`,
 `dbus-monitor`,
 and `dbus-send`.
A hosted Node client calls the service,
 a `gdbus` monitor watches the signals,
 and the task checks every answer above,
including the ordered log and the refusal of an unoffered action.
It also requires that a decoy session bus in the compositor's own environment saw only a deliberate sentinel connection.

#### Plasma jobs

In `kde` mode the fake owns `org.kde.kuiserver` and `org.kde.JobViewServer`,
 as Plasma 6.7.5 does,
and serves three interfaces at `/JobViewServer`:

- `org.kde.JobViewServerV2.requestView(sia{sv}) -> o`:
   the desktop entry,
   a capabilities bit field
  (`1` cancel,
   `2` suspend and resume),
   and hints.
  The capabilities argument is a signed 32-bit integer.
  A caller that sends an unsigned one calls `requestView(sua{sv})`,
   which does not exist,
   and the call fails.
- `org.kde.JobViewServer.requestView(ssi) -> o`:
   the first interface version,
   with an app name and an icon name.
- `org.kde.kuiserver`:
   `registerService` refuses with `NotSupported`
  (`kuiserver proxying capabilities are deprecated and defunct.`,
   the text in Plasma's `jobsmodel_p.cpp`),
  `registeredJobContacts` answers an empty list,
   `requiresJobTracker` answers `false`,
  and `emitJobUrlsChanged` emits `jobUrlsChanged` with an empty list,
   because the fake does not record `destUrl`.
  Both `kuiserver` signals,
   `jobUrlsChanged` and `requiresJobTrackerChanged`,
   are declared.

Each granted view is an `org.kde.JobViewV3` object at `/org/kde/notificationmanager/jobs/JobView_N`,
the path Plasma uses,
 with `N` counting from `1`:

- `update(a{sv})` and `terminate(usa{sv})` never reply in Plasma,
   because they are `NoReply` methods.
  A Qt caller sets the no-reply flag,
   and the fake records that flag on every call (`no-reply=true|false`),
  so a caller that blocks waiting for a reply is visible in the log.
  The fake replies to a caller that expects one,
   which is how the flag can be told apart.
- `update` applies `title`,
   `infoMessage`,
   `percent`,
   and `suspended`,
   and keeps what a partial update leaves out.
  `percent` is read from any integer width,
   as Plasma's `QVariant` conversion does.
  Every property,
   known or not,
   is recorded with its D-Bus signature.
  `descriptionField` is ignored,
   as in Plasma's `job_p.cpp`.
- The `suspended` state changes only when the app sends `update({suspended: b})`.
  A `job-suspend` command is the user pressing Pause,
   and the job still shows `suspended=false`
  until the app answers it,
   which is what Plasma's Pause button does.
- `terminate` records the error code and message,
   and removes the object from the bus,
   as Plasma's `finish()` does.
  Later calls to the path fail.
   The job stays in `jobs` as terminated.
- The signals `suspendRequested`,
   `resumeRequested`,
   `cancelRequested`,
   and `updateRequested` are declared,
  and the commands below emit them as broadcasts from the job's path on the connection that owns `org.kde.kuiserver`,
  which is how Qt's adaptor emits them.
  A subscription on that path reaches the owner of the job,
   and any other subscriber of the path.

Control commands:

- `jobs` lists the jobs,
   oldest first,
   as `ok <count>` and records of the fields `id`,
   `path`,
   `entry`,
   `title`,
  `info`,
   `percent`,
   `suspended`,
   `capabilities`,
   and `terminated`.
  A terminated job continues with `terminated=true code=N message=...`.
- `job-log [N]` lists the newest `N` records (default 64):
   `request-view id=N api=1|2 entry=... capabilities=C hints=...`,
  `update id=N no-reply=true|false properties=...`,
   `terminate id=N no-reply=... code=C message=... hints=...`,
  `signal id=N name=...`,
   and `kuiserver method=... detail=...`.
- `job-clear` forgets the log and the terminated jobs,
   and keeps running jobs.
- `job-suspend ID`,
   `job-resume ID`,
   `job-cancel ID`,
   and `job-refresh ID` emit
  `suspendRequested`,
   `resumeRequested`,
   `cancelRequested`,
   and `updateRequested` for the job.
  The capabilities are not checked,
   so a test can send a signal the real Plasma interface could not.
  An unknown job answers `err no job with id N`,
   and an ended one `err job N has ended; its object is gone from the bus`.

All job commands answer `err no Plasma job service (start with --notifications kde)` in the other modes,
and all notification commands answer
`err no notification service (start with --notifications freedesktop or --notifications kde)` in `none` mode.

A worked job from a hosted app's own environment:

```sh
busctl --address="$DBUS_SESSION_BUS_ADDRESS" call org.kde.kuiserver /JobViewServer org.kde.JobViewServerV2 \
  requestView sia{sv} org.example.inspect 3 1 transient b true      # => o "/org/kde/notificationmanager/jobs/JobView_1"
busctl --address="$DBUS_SESSION_BUS_ADDRESS" --expect-reply=no call org.kde.kuiserver \
  /org/kde/notificationmanager/jobs/JobView_1 org.kde.JobViewV3 update a{sv} 2 title s Analysing percent u 10

printf 'jobs\n' | nc -U /tmp/nws.sock
# => ok 1 id=1 path=/org/kde/notificationmanager/jobs/JobView_1 entry=org.example.inspect title=Analysing info= percent=10 suspended=false capabilities=3 terminated=false
printf 'job-suspend 1\n' | nc -U /tmp/nws.sock    # => ok (the app receives suspendRequested; the job still shows suspended=false)
printf 'job-log\n' | nc -U /tmp/nws.sock
# => ok 3 t=137 request-view id=1 api=2 entry=org.example.inspect capabilities=3 hints=transient:b:true; t=219 update id=1 no-reply=true properties=percent:u:10,title:s:Analysing; t=521 signal id=1 name=suspendRequested
```

Use `mise run //package/nested-wayland-session/cli:inspect:plasma-jobs` to verify the built release binary.
It plays a Qt-style client with `busctl` and watches the signals with `gdbus`,
 behind the same decoy session bus.

#### Contract with Plasma

The fake's interfaces are compared with Plasma's own interface XML files in `tests/fake_services/contract.rs`.
The test copies of the files are under `tests/fixtures/plasma-workspace-6.7.5/`,
each starting with a comment that names its source file and the tag `v6.7.5` of `KDE/plasma-workspace`
(commit `cc1ab7963f94a9b76836412855caed6b94066188`),
 the version installed on the host.
The test introspects the running fake and requires that the method names and signatures
of `org.kde.JobViewServer`,
 `org.kde.JobViewServerV2`,
 `org.kde.kuiserver`,
 and `org.kde.JobViewV3`
equal the XML exactly,
 signals included.
For `org.freedesktop.Notifications` the fake serves every method and signal of the XML except
`Inhibit`,
 `UnInhibit`,
 the `Inhibited` property,
 `NotificationReplied`,
 and `ActivationToken`,
which belong to capabilities the recorder does not report.
The XML's `NoReply` annotation on `update` and `terminate` is not part of zbus's introspection output,
so the test checks that the fixtures carry it and the job tests check the no-reply flag at run time.

The same comparison against a live Plasma session is read-only:
`mise run //package/nested-wayland-session/cli:inspect:plasma-host-contract` introspects the host's
`org.kde.kuiserver` at `/JobViewServer` and `org.freedesktop.Notifications` with `busctl --user introspect`,
introspects the fake the same way,
 and requires every member of the fake to exist on the host with the same
signature.
It never calls `requestView`,
 `Notify`,
 or any other method that creates a job or a notification on the host,
and it skips with a message when the host session bus has no `org.kde.kuiserver` owner.
The host's `busctl --user introspect org.kde.kuiserver /JobViewServer` on 2026-10-11,
 Plasma 6.7.5:

```txt
org.kde.JobViewServer               interface -         -            -
.requestView                        method    ssi       o            -
org.kde.JobViewServerV2             interface -         -            -
.requestView                        method    sia{sv}   o            -
org.kde.kuiserver                   interface -         -            -
.emitJobUrlsChanged                 method    -         -            -
.registerService                    method    ss        -            -
.registeredJobContacts              method    -         as           -
.requiresJobTracker                 method    -         b            -
.jobUrlsChanged                     signal    as        -            -
.requiresJobTrackerChanged          signal    b         -            -
```

and its `org.freedesktop.Notifications` interface,
 with `GetCapabilities` answering
`body body-hyperlinks body-markup body-images icon-static actions persistence inline-reply sound x-kde-urls
x-kde-origin-name x-kde-display-appname inhibitions` and `GetServerInformation` answering
`Plasma KDE 6.7.5 1.2`:

```txt
org.freedesktop.Notifications       interface -             -            -
.CloseNotification                  method    u             -            -
.GetCapabilities                    method    -             as           -
.GetServerInformation               method    -             ssss         -
.Inhibit                            method    ssa{sv}       u            -
.Notify                             method    susssasa{sv}i u            -
.UnInhibit                          method    u             -            -
.Inhibited                          property  b             false        emits-change
.ActionInvoked                      signal    us            -            -
.ActivationToken                    signal    us            -            -
.NotificationClosed                 signal    uu            -            -
.NotificationReplied                signal    us            -            -
```

The host also serves `org.kde.NotificationManager` on `/org/freedesktop/Notifications`,
 which the fake does not.
All three names (`org.freedesktop.Notifications`,
 `org.kde.JobViewServer`,
 `org.kde.kuiserver`) have one owner
on the host,
 `plasmashell`.

#### Limits

- No real Plasma session was available for an end-to-end check.
  The fake follows the Plasma 6.7.5 sources (`libnotificationmanager`) and the live host's introspection,
  and the job behavior has not been compared with a live job.
- Plasma rejects an identical `Notify` repeated within a second with `ExcessNotificationGeneration`.
  The fake records every call.
- A job's `destUrl`,
   speed,
   byte counts,
   and description fields are recorded in the log but not in `jobs`.
  `org.kde.JobViewV2` objects,
   the `org.kde.NotificationManager` interface,
   notification inhibition,
  inline replies,
   and the desktop portal for notifications are not served.
- A notification has no image or sound handling,
   and `NotificationClosed` for an expired notification is never sent
  unless `notification-close` asks for it.

### Keyboard input

`key SPEC [press|release|tap|hold MS]` presses what a person can press on a keyboard.
`SPEC` is one of:

- A key name.
  Every Linux `KEY_*` name from `linux/input-event-codes.h` works,
  lowercase and without the prefix (`f1` to `f24`,
  `insert`,
  `delete`,
  `home`,
  `end`,
  `pageup`,
  `pagedown`,
  `sysrq`,
  `pause`,
  `scrolllock`,
  `numlock`,
  `capslock`,
  `kp0` to `kp9`,
  `kpenter`,
  `kpplus`,
  `volumeup`,
  `playpause`,
  and the rest of the 523 names).
  Case,
  `_`,
  and `-` do not matter,
  so `Page_Up`,
  `PAGEUP`,
  `page-up`,
  and `KEY_PAGEUP` are one key.
- A spoken alias:
  `ctrl`,
  `win`,
  `menu`,
  `altgr`,
  `pgup`,
  `pgdn`,
  `print`,
  `plus`,
  `period`,
  `bracketleft`,
  `bracketright`,
  `audioplay`,
  `audionext`,
  and X11 spellings such as `KP_Enter`,
  `Scroll_Lock`,
  `Page_Down`,
  `Control_L`,
  `Shift_R`,
  `Super_R`.
- Every modifier on both sides:
  `ctrl`,
  `shift`,
  `alt`,
  `meta` (also `super`,
  `win`,
  `logo`) press the left key,
  and `rctrl`,
  `rshift`,
  `ralt` (also `altgr`),
  `rmeta` press the right one.
- A single character (`a`,
  `1`,
  `;`),
  which presses the key that types it on the US layout without adding Shift.
- A raw code `code:<n>`,
  an evdev number from 1 to 767 in decimal or `0x` hexadecimal:
  `code:68` is F10.
  The number is the kernel's,
  not the X11 keycode (which is 8 more).

`menu` is the context-menu key a keyboard sends (`KEY_COMPOSE`,
 evdev 127,
 keysym `Menu`,
 winit `ContextMenu`),
and `menukb` is the kernel's `KEY_MENU` (keysym `XF86MenuKB`).
`print` is `KEY_SYSRQ`,
the key a keyboard sends for Print Screen.
`plus` is the `=` key;
hold shift yourself for a typed plus.

What a key types depends on the keymap the client has,
as with a real keyboard.

A chord joins keys with `+`:
`ctrl+shift+f10` presses Ctrl,
Shift,
and F10 in that order,
and `tap` releases them in the reverse order.
The plus key itself is written `++` (`ctrl++`) or `plus`.
Every name is resolved before the first key is sent,
so `ctrl+nokey+f10` answers `err unknown key: nokey` and presses nothing.
With `press`,
all keys go down and stay down;
`release` lets them up in reverse.

`hold MS` presses the chord,
waits `MS` milliseconds (1 to 600000),
and releases it,
and answers `ok` after the release.
The compositor keeps serving the client during the wait.

By default key repeat is the client's work on Wayland,
not the compositor's
(the `wl_keyboard.key` documentation says a `repeat_info` rate of 0 is what hands repetition to the compositor).
The compositor announces `wl_keyboard.repeat_info`
(`--repeat-rate`,
`--repeat-delay`,
or the `repeat-info` command),
and a client such as winit starts its own timer after the delay while a repeating key is down and
reports each repeat as a key press with `repeat` set
(Slint's `KeyEvent.repeat`).
So `hold` is exactly a person holding the key,
and a client that repeats gets the same events it gets from a desktop compositor.
At 20 repeats per second and 100 ms delay,
a 1000 ms hold of Right produced 18 repeats in each of three measured runs.
`repeat-info` answers `ok rate=R delay=D` and
`repeat-info RATE DELAY_MS` changes the pair (`ok changed` or `ok unchanged`).

Modifier keys never repeat:
the client's keymap marks them as non-repeating,
and winit skips them (winit 0.30.13 `src/platform_impl/linux/wayland/seat/keyboard/mod.rs`,
`key_repeats`).
Holding Shift therefore gives one press and one release,
also on a real desktop.
A "double Shift with repeats" sequence cannot occur there,
so it is not a gap of this tool,
and an app's double-Shift detector is covered by a unit test of the detector.

### Typing text

`type TEXT` types the rest of the line.
US-layout characters use their keys,
with Shift where needed.
Any other character (accents,
CJK,
Hangul,
emoji) is typed on a spare key of a temporary keymap that carries its Unicode keysym,
and the original keymap is sent back after the text.
The client reads the key's text from the keymap,
so winit reports `KeyEvent.text` as that character
and Slint's `TextInput` receives it as typed text,
exactly as it would from a keyboard layout that has the character.

A text with more than 64 distinct off-layout characters is typed in chunks,
one keymap per chunk.
Tab and line feed are the Tab and Enter keys;
any other control character fails the whole command before a key is sent:
`err type cannot send the control character U+0001 at position 1`.

This delivers key events with text.
An input method that composes a candidate and commits it through `text-input-v3`
reaches a toolkit as `Ime::Commit`,
which Slint turns into the same text.
That protocol is not emulated:
the compositor would have to serve `zwp_text_input_v3` and play the input method itself,
and the client receives the same string either way.

### Pointer input

- `click X Y [left|right|middle] [mods=CHORD] [count=N] [interval=MS]`:
  `count=2` is a double click (presses `interval` ms apart,
  default 80,
  at one point),
  `mods=ctrl+shift` holds those keys from before the first press until after the last release.
- `drag X1 Y1 X2 Y2 [left|right|middle] [mods=CHORD] [steps=N] [step-ms=MS]`
  presses at the first point,
  moves in `N` equal steps (default 10) `MS` apart (default 16) ending exactly on the second point,
  and releases there.
  Sliders,
  text selection,
  and reordering see the motion in between.
- `wheel X Y HORIZONTAL VERTICAL [mods=CHORD]` takes notches,
  and fractions are allowed:
  `0.25` is a quarter notch.
  The amount is sent as Wayland `axis_value120` in 1/120 notches,
  the way a high-resolution wheel does.
  Amounts that round to nothing are refused.
  A point that no hosted window covers is refused too:
  the answer is `err no nested client surface at wheel position X,Y: ...` and nothing is sent to the client.
  That happens at every point until the app has committed its first buffer
  (a client creates its toplevel,
  then spends tens to hundreds of milliseconds on its renderer),
  so a driver must read the answer and send the wheel again,
  or wait until `windows` lists a window.
  A driver that ignores the answer loses the wheel without a trace in the app's log.
- `scroll X Y DX DY [mods=CHORD] [steps=N] [step-ms=MS]` is a touchpad two-finger scroll of `DX`,
  `DY` logical pixels in total,
  split into `N` events (default 1) and followed by the finger-lift stop.
  Clients receive pixel deltas from a finger source,
  not wheel notches.

`click`,
`move`,
and `button` do not refuse a point that no window covers:
they answer `ok`,
and no client receives the event,
as for a real pointer over an empty desktop.
Use `windows` to know that the window can take input before the first of them.

Modifier keys are real key events on the nested keyboard,
so the client sees Ctrl or Shift down before the pointer event,
which is where toolkits read them from.

### Several windows and focus

The hosted app may open further windows,
such as the folder chooser of `zenity` or `kdialog`.
Each is a toplevel of the same nested session,
mapped fullscreen above the others.
A new toplevel takes keyboard focus,
the previously focused one loses its activated state,
and when the focused toplevel is destroyed,
or its client disconnects,
keyboard focus returns to the most recently focused remaining toplevel,
which becomes activated again.
Killing the dialog's process with `SIGKILL` takes the same path.

- `windows` answers the count and one record per mapped toplevel,
  first mapped first.
  A toplevel is mapped once its client has committed a buffer;
  until then it has no extent and no pointer event reaches it,
  so it is not listed,
  and the first answer that lists a window is the moment `wheel`,
  `scroll`,
  and `click` can reach it:
  `index=N app-id=... title=... focused=true|false`,
  text percent-encoded.
  Index 0 is the window `title` and `app-id` describe.
- `focus` answers the record of the toplevel that has keyboard focus,
  or `none`.
- `close [INDEX]` sends `xdg_toplevel.close` to that window or to the focused one,
  as the close button does.
  What the app does with it is the app's choice.
- `display` answers the Wayland socket name (`wayland-N`).
  A test starts a second client with `WAYLAND_DISPLAY` set to it.

Native dialogs that go through a desktop portal need the host's portal service,
which the private bus does not provide;
a dialog tool that opens its own window needs none.

### Dropping files

`drop-file PATH... [at X Y]` drags the files from the compositor onto the window and drops them.
A path with whitespace is written in double quotes,
and `\"` and `\\` are escapes inside the quotes;
an unquoted path is taken as it is.
`drop-file "/music/My Album/a b.mp3" /music/c.mp3 at 300 200` drops two files at one point.
The earlier `drop-file PATH X Y` form still works for one unquoted path followed by two numbers;
a path that is a bare number,
or the word `at`,
must be quoted.

The payload is a `text/uri-list` of RFC 8089 `file:` URIs,
one per line ended by CRLF,
with every byte outside ASCII letters,
digits,
`-`,
`.`,
`_`,
`~`,
and `/` percent-encoded,
so spaces,
`%`,
`#`,
`?`,
quotes,
and non-ASCII names survive a client that decodes the URI.
Each path is resolved to an absolute path first,
and one that does not exist fails the whole command.

### Cursor shape

`cursor` answers the cursor the app asked for under the pointer:
a cursor-shape name such as `default`,
`pointer`,
or `text`,
`hidden`,
or `surface` for a bitmap the client drew itself.
The compositor serves `wp_cursor_shape_manager_v1`,
which winit uses to send the shape of a Slint `mouse-cursor`.
Move onto the control first with `move`.

### Read the accessibility tree

`--accessibility` serves,
 on the private session bus,
 the three pieces of the AT-SPI accessibility bus that a toolkit needs:
`org.a11y.Bus.GetAddress` (the private bus itself is the accessibility bus),
`org.a11y.Status` (`IsEnabled` and `ScreenReaderEnabled` are true),
and the registry's `org.a11y.atspi.Socket.Embed`.
AccessKit,
which Slint's winit backend uses,
sees an enabled screen reader,
connects,
registers its application root,
and serves its tree from its own bus name.
The compositor implements no screen reader and routes no events.

`a11y` walks that tree depth first and answers the count and one record per object:
`depth=N role=... name=... states=a,b bounds=x,y,w,h`.
Role names are the AT-SPI role names (`push button`,
`check box`,
`label`,
`frame`),
states are a named subset (`checked`,
`enabled`,
`focusable`,
`focused`,
`showing`,
`visible`,
...),
and bounds are screen coordinates in the nested screen when the object has a component interface.
Text is percent-encoded.
The read has a five-second budget so a stalled app cannot freeze the compositor.
Without `--accessibility` it answers `err no accessibility service (start with --accessibility)`.

The tree is the toolkit's own,
so the labels a screen reader would speak are what `a11y` shows.
Against the desktop music player on its first page,
`a11y` returned 65 objects:
the window frame,
labelled push buttons,
radio buttons for the folder list,
and a toggle button per track with its title and length.
What is not covered:
actions on objects (there is no command that invokes the default action),
text-range and selection queries,
and change events.

## Control protocol

### Who can connect

The control protocol has no authentication.
Whoever connects can type text,
press keys,
click,
record,
and capture screenshots of the hosted application,
and can ask the session to quit.
The only access control is the permission of the socket file,
because connecting to a Unix socket needs write permission on its file.

The session binds the socket inside a private directory (mode `0700`) beside `PATH`,
sets its mode to `0600`,
and then renames it to `PATH`.
The socket is therefore never reachable with a looser mode,
and a looser umask in the environment cannot widen it.
A path that already exists is removed first.
The directory that holds `PATH` must exist and be writable by the user.
The staging directory,
named `.nws-control-<pid>-<n>`,
exists only for the instant of the bind;
a session killed at that instant can leave it behind empty.

Two cases are outside this guarantee.
A process of the same user can always connect,
so the control socket does not separate the session from other programs that user runs.
Root can connect to any socket.

### Requests

Requests are newline-delimited text,
 one command per line.
 Each request yields exactly
one response line:
 `ok`,
 `ok <data>`,
 or `err <message>`.

- `ping` answers `ok` (liveness check).
- `screenshot PATH` composites the hosted app's latest committed frame and writes it to `PATH` as a PNG.
  The app needs time to draw after input;
  see [Frame pacing](#frame-pacing) for what keeps that frame current.
- `click X Y [left|right|middle] [mods=CHORD] [count=N] [interval=MS]` moves the pointer to the logical point and clicks
  (button defaults to `left`).
  It presses and releases at once and is unchanged by the `move` and `button` commands.
  `count`,
  `interval`,
  and `mods` make a double click or a click with keys held;
  see [Pointer input](#pointer-input).
- `drag X1 Y1 X2 Y2 [left|right|middle] [mods=CHORD] [steps=N] [step-ms=MS]` presses,
  moves in steps,
  and releases.
- `scroll X Y DX DY [mods=CHORD] [steps=N] [step-ms=MS]` is a touchpad pixel scroll.
- `move X Y` moves the pointer to the logical point without pressing a button.
  The hosted client receives pointer enter,
   motion,
   and leave events,
  so hover states can be observed.
  Answers `ok`,
   and `err` for coordinates that are not finite numbers.
- `button down|up [left|right|middle]` presses or releases a button at the current pointer position
  (button defaults to `left`).
  Move first with `move`,
   then send `button down`,
   pause or move,
   and send `button up`:
  a movement while the button is down reaches the client as motion with the button held.
  Answers `ok`,
   or `err` when the button is already down for `down`
  or already up for `up`.
  A `click` or `drop-file` release also clears a button that `button down` held,
  so the answers stay true to what the client received.
  A `button down` with no client surface under the pointer is still recorded as held,
  but no client receives the press.
- `title` answers `ok <title>` with the title the hosted client set on its toplevel,
  or `ok ` with an empty payload when it set none.
  The value follows each change the client makes at runtime.
  A backslash,
   line feed,
   and carriage return are written as `\\`,
   `\n`,
   and `\r`,
  so the answer stays on one line.
- `app-id` answers `ok <app_id>` in the same form as `title`,
   for the app ID the client set.

Use `mise run //package/nested-wayland-session/cli:inspect:pointer-title` to verify the built release binary.
It hosts a Slint client,
 sends `move`,
 `button down`,
 a 600 ms pause,
 and `button up`,
and checks each event in the client's Wayland protocol log.
It also checks that `title` and `app-id` follow the client's own title change.
- `launcher` answers the newest taskbar progress update,
   or `ok none`;
   see
  [Observe taskbar progress](#observe-taskbar-progress).
- `launcher-log [N]` answers `ok <count>` followed by the newest `N` updates,
   oldest first.
- `launcher-clear` empties the taskbar progress history and answers `ok`.
- `accent-color RRGGBB` switches the private accent color started by `--accent-color`.
  It answers `ok changed`,
   `ok unchanged`,
   or `err <message>` when the session has no private accent color.
  See [Set the accent color](#set-the-accent-color).
- `notifications`,
   `notification-log [N]`,
   `notification-clear`,
   `notification-close ID [REASON]`,
  and `notification-action ID KEY` read and drive the recording notification service started by `--notifications`.
- `jobs`,
   `job-log [N]`,
   `job-clear`,
   `job-suspend ID`,
   `job-resume ID`,
   `job-cancel ID`,
   and `job-refresh ID`
  read and drive the recording Plasma job view server started by `--notifications kde`.
  See [Observe notifications and Plasma jobs](#observe-notifications-and-plasma-jobs).
- `wheel X Y HORIZONTAL VERTICAL [mods=CHORD]` moves the pointer to the logical point
  and delivers real wheel notches there.
  Positive counts scroll right and down,
  and fractions are high-resolution steps of 1/120 notch.
- `key SPEC [press|release|tap|hold MS]` presses a key,
  a chord such as `ctrl+shift+f10`,
  or a raw `code:<n>`;
  see [Keyboard input](#keyboard-input).
  Action defaults to `tap`.
  For a shortcut,
  use a chord,
  or press its modifier,
  tap the target key,
  then release the modifier through this same isolated seat.
  Release held keys in the caller's cleanup path.
- `repeat-info [RATE DELAY_MS]` reads or changes the key repeat pair announced to clients.
- `type TEXT` types the rest of the line as key taps,
  using spare Unicode keys for characters the US layout lacks;
  see [Typing text](#typing-text).
- `resize WIDTH HEIGHT` sets the logical nested-screen size,
  keeping the output scale,
  and applies it before answering `ok`.
  Odd and even sizes are exact in the client and in captures.
  While a recording runs,
  the client's pending frame callbacks are answered right after the new configure;
  see [Change the output scale](#change-the-output-scale).
- `scale SCALE` sets the output scale,
  keeping the logical size.
  It answers `ok changed` once the window and the output have the new scale
  and every surface has been told it;
  the client redraws on its own schedule afterwards.
  It answers `ok unchanged` when that scale was already set,
  and `err <message>` for a value outside 0.5 to 3 or between steps of 1/120.
  See [Change the output scale](#change-the-output-scale).
- `drop-file PATH... [at X Y]` originates a compositor-side drag carrying the files as a
  `text/uri-list` and drops them onto the hosted app (defaulting to the window centre),
  exercising the app's own inbound file-drop path;
  see [Dropping files](#dropping-files).
  The compositor is the drag source,
  so no second app (file manager) is needed.
  The drop completes asynchronously:
  the command returns `ok` once the drag is under way,
  the button releases after a short dwell (so the app can `accept` and choose an action,
  the round-trips a real drag needs),
  and the app then reads the payload.
  Use a `screenshot` afterwards to observe the result.
- `record DIR [FPS] [FORMAT]` starts recording a frame sequence into `DIR` at `FPS` frames
  per second (default `60`) in `FORMAT` (`png` default,
   or `bmp`).
   See
  [60fps recording](#60fps-recording).
- `record stop` stops recording and answers with the measured statistics,
   for example
  `ok captured=180 dropped=0 failures=0 seconds=3.004 fps=59.9`.
- `color-scheme dark|light` switches the private appearance preference started by `--color-scheme`.
  It answers `ok changed` after notifying subscribed clients,
  `ok unchanged` when that value was already served,
  and `err <message>` when the session has no private appearance portal.
  See [Switch appearance while the client runs](#switch-appearance-while-the-client-runs).
- `windows`,
  `focus`,
  `close [INDEX]`,
  and `display` list the toplevels,
  name the focused one,
  ask one to close,
  and give the Wayland socket name;
  see [Several windows and focus](#several-windows-and-focus).
- `cursor` answers the cursor shape the app asked for;
  see [Cursor shape](#cursor-shape).
- `a11y` reads the accessibility tree started by `--accessibility`;
  see [Read the accessibility tree](#read-the-accessibility-tree).
- `gpu` answers the device the compositor renders on;
  see [Choosing the GPU](#choosing-the-gpu).
- `quit` stops the compositor.

Payloads are passed through verbatim:
 `type` text and the `screenshot` path keep their
spaces and are never interpreted by a shell,
 so no quoting rules apply beyond the single
newline that terminates each command.
Only `drop-file` has a quoting rule,
for paths with whitespace.

## How it works

By default the tool is itself a winit client of the parent Wayland session,
 so it inherits the
parent's GPU device and renders with a GLES2/EGL renderer,
 following Smithay's `anvil`
winit path.
With `--headless` there is no parent to inherit from:
the same GLES renderer runs on a surfaceless EGL display of the DRM render node,
and the dmabuf global advertises the format and modifier pairs of that display.
Both modes build the same output (including its transform) and the same dmabuf global,
and screenshots render the output into an offscreen texture in both,
so a capture is the same code path in either mode.
 Because the fixture owns the compositor,
 three testing needs become built-in,
in-process features rather than external tools:

- Screenshots are a readback (`ExportMem::copy_framebuffer` plus `map_texture`)
  of an offscreen texture the size of the output mode,
  encoded with the [`image`][] crate.
  Rendering off screen gives every capture the screen's current size at once;
  the window's own back buffer takes a new size only after its next swap.
- Input is synthesised directly through the compositor's own seat,
   so events reach only
  the hosted client,
   never the host session,
   and there is no `/dev/uinput` involvement.
- The control API is a Unix socket whose blocking-IO thread forwards parsed commands to
  the render thread over a channel and returns each result.

## Frame pacing

A Wayland client draws its next frame after the compositor tells it the previous one was shown.
The live path sends that notice after presenting to the parent compositor,
and the parent in turn paces this tool the same way.
A parent that stops presenting the nested window,
for example because the host session is locked or the window is hidden,
would therefore leave the hosted app waiting to draw,
and `screenshot` showing an old frame.

To keep capture independent of host visibility,
a timer at the nested output's 60 Hz refresh sends those notices itself
whenever the parent has presented nothing for 50 ms and no recording is running.
The log records when this fallback starts and when the parent presents again.
While a recording runs,
the recorder's own timer sets the pace instead.

### Headless pacing

A headless session has no parent compositor whose presentations could pace the client,
so a timer is the only source of frame notices.
It fires at the output's 60 Hz refresh (one tick every 16.666 ms) from the first tick on,
with no stall threshold and no wait for a parent,
and sends a frame callback on every tick to every surface that asked for one.
While a recording runs,
the recorder's timer sends them at the capture rate instead,
as in the nested window.
`screenshot` and `record` render the latest committed client buffer on demand,
so a screenshot taken after a client commit shows that commit,
and nothing is drawn or presented between captures.
`inspect:headless` measures 60 callback ticks per second on the client's side.
A Slint client counts two `wl_callback.done` events per tick,
because winit asks for one frame callback of its own beside the one Slint asks for;
both arrive with the same timestamp.

### Checking the stall fallback

Use `mise run //package/nested-wayland-session/cli:inspect:stalled-parent` to verify the built release binary.
The task hosts one session inside another and starts a very slow recording in the outer one,
which withholds frame notices from the inner one exactly as a hidden window would.
It then requires that inner screenshots follow three appearance switches.
It needs `slint-viewer`.
The withheld notices come from the outer session's recording,
not from the host's lock or window state.

## 60fps recording

The `record` command captures a frame sequence at a steady rate (60fps by default) that
holds even when the hosted app is laggy or greedy.
 Three design choices make that possible:

- The capture is decoupled from the app.
   A drift-free timer (scheduled on absolute
  deadlines) composites whatever the app LAST committed and reads it back,
   whether or not
  the app produced a new frame.
   A slow app simply yields repeated frames;
   the cadence never
  stalls.
   Frame callbacks still go out at the capture rate,
   so an animating app keeps
  drawing.
   A `resize` or `scale` that changes the screen sends them once more right away,
   so the client can draw the new size before the next capture
  (see [Change the output scale](#change-the-output-scale)).
   The recorder never calls `submit`,
   so the parent compositor's vsync cannot
  throttle the capture (the visible window is intentionally frozen while recording).
- The render thread's per-tick work is tiny:
   render,
   read back,
   and copy into a pooled
  buffer.
   PNG encoding,
   the expensive part,
   runs on a pool of worker threads sized to the
  machine,
   so it keeps up in parallel.
   If the encoders ever fall behind,
   frames are dropped
  (and counted) rather than blocking the timer,
   so the cadence is preserved and the shortfall
  is reported.
- A greedy app is contained with systemd.
   Under `--isolate` the app runs in a transient
  systemd scope with a `CPUQuota` (hard cap) and a low `CPUWeight`,
   reserving CPU for the
  capture pipeline.
   When systemd is unavailable this degrades to a direct launch with a
  warning;
   isolation is a robustness enhancement,
   never a hard requirement.

Formats:
 `png` (compressed,
 the default) and `bmp` (uncompressed,
 near-zero encode cost)
are supported.
 BMP is the reliable path for sustained high frame rates when PNG's deflate
cannot keep up,
 at the cost of large files.
 AVIF is deliberately not offered:
 its AV1 intra
encode is far too CPU-heavy for real-time capture,
 the opposite of what this mode needs.

Measured on a 16-core / AMD Radeon RX 7600 host at 1280x720:
 PNG and BMP both sustain 59.9
captured fps with zero dropped or failed frames over a three-second capture,
 and the rate
holds while hosting a client that saturates every core.

## Building and testing

The host usually lacks the development headers Smithay links against,
 so cargo work runs
in a CachyOS build container (see `Containerfile`);
 the resulting binary runs on the host.
Tasks fall back to the host automatically when the development libraries are present.

- `mise run //package/nested-wayland-session/cli:build` builds the release binary.
- `mise run //package/nested-wayland-session/cli:test` runs the unit tests.
- `mise run //package/nested-wayland-session/cli:test:all` runs them without stopping at the first failure.
- `mise run //package/nested-wayland-session/cli:lint:clippy` runs clippy with warnings
  denied.
- `mise run lint:monochromatic-lint -- package/nested-wayland-session/cli` runs the repo's Rust linter
  (`rust/max-lines` plus `rust/require-rustdoc`) from the repository root.
- `mise run //package/nested-wayland-session/cli:inspect:input` drives a real session through the control socket:
  key names,
  chords,
  holds with the client's key repeat,
  Unicode typing,
  exact odd and even sizes,
  double and modified clicks,
  fractional wheel,
  touchpad scroll,
  drags,
  cursor shape,
  focus return with a second window,
  quoted multi-file drops decoded by a GTK 4 client,
  and the accessibility tree.
  It runs every check,
  reports each,
  and exits 1 on any failure.
  It needs `slint-viewer` and `python3` with GTK 4,
  takes the binary from `NWS_BINARY` (default `./target/release/monochromatic-nested-wayland-session`),
  and runs only the checks whose names match the regular expression in `NWS_CHECKS` when it is set.
- `mise run //package/nested-wayland-session/cli:inspect:headless` runs a Slint client with `--headless` and no parent
  display variable in the compositor's environment:
  two identical screenshots,
  a click that changes the third,
  upright and non-blank images,
  `resize` and `scale`,
  the frame callback rate,
  a decoy parent socket that must receive no connection,
  and the failure message of a start with no display and no flag.
  It needs `slint-viewer` and takes the binary from `NWS_BINARY` like `inspect:input`.
- `mise run //package/nested-wayland-session/cli:inspect:headless-gpu` runs a hosted Slint session once per device
  that `--list-gpus` reports (each render node and `software` on this host).
  For each it checks that the client's frames reach the screenshot,
  that a click changes the screenshot,
  that the `gpu` answer names the device,
  and that the hosted client opened only that device's DRM node (none for software).
  It then hides `/dev/dri` from the compositor with `bwrap` in a private mount view,
  which changes nothing on the host,
  and checks that `auto` falls back to software and logs why,
  that `--gpu software` works there,
  that an explicit render node fails instead of falling back,
  that a start with no usable device lists every attempt,
  and that bad `--gpu` values fail with messages that name the way forward.
  It needs `slint-viewer` and `bwrap`.
- `mise run //package/nested-wayland-session/cli:verify:container` is the reference build
  that builds,
   clippys,
   and tests inside the container.

## Scope

Two render backends share one control API.
 The default is the nested-winit path,
 which needs a running parent session at run time.
 The opt-in `--headless` path (issue #273) uses surfaceless EGL on the DRM render node for a bare machine
with no display at all,
 such as a CI server or an SSH session;
 see [Headless mode](#headless-mode).
 Headless is never selected for you.

## Not supported

What a person can do that this tool cannot,
and why.

- Choosing the GPU of the nested window:
  it renders on the GPU of its parent compositor,
  which this tool cannot choose,
  so `--gpu` is accepted only with `--headless`.
- Steering a client that picks its own Vulkan adapter:
  the compositor sets only the variables Mesa's EGL and OpenGL read in the client.
- Watching a headless session:
  it has no window to restore.
  Use `screenshot` or `record`,
  or run without `--headless` to get the nested window.
- An input method's `text-input-v3` commit:
  typing arrives as key events with text,
  which clients read the same way;
   see [Typing text](#typing-text).
- Keyboard layouts other than the compositor's default:
  `type` places US-layout characters on US-layout keys,
  so a different layout would type different characters for plain ASCII.
- Touch screens,
  pen tablets,
  pointer gestures (pinch,
  swipe),
  pointer lock,
  and more than one output:
  the seat has a keyboard and a pointer only.
- The host's desktop portals (the file chooser portal among them):
  the private bus serves none by design,
  so a client's portal call fails with `ServiceUnknown`.
  A dialog tool that opens its own window,
  such as `zenity` or `kdialog`,
  works.
- AT-SPI actions,
  text ranges,
  and change events:
  `a11y` reads the tree only.

## Changes in 0.6.2

- `windows`,
  `focus`,
  and `close` list a toplevel only once its client has committed a buffer.
  Before,
  a toplevel was listed from the moment its client created it,
  tens to hundreds of milliseconds before any point of the screen belonged to it,
  so a driver that waited for `windows` and then sent `wheel` was refused.
  Measured with a Slint IDE window in 36 headless sessions with the app's Wayland protocol log:
  the wheel was refused in 17 of 17 sent before the toplevel's first `wl_surface.attach` of a buffer
  (no `wl_pointer.enter` and no `axis` event was sent to the client)
  and delivered in 19 of 19 sent after.
  With the IDE as the client and the wheel sent at once when `windows` first listed a window (60 sessions),
  0.6.1 refused 60 of 60 and this version refused 0 of 60.
- The answer to a refused `wheel` or `scroll` says why and what to do,
  and the README says that `click` and `move` over no window answer `ok` and reach nobody.

## Changes in 0.6.1

- The control socket is owner-only.
  It is bound inside a private directory beside `--socket PATH`,
  set to mode `0600`,
  and renamed to `PATH`,
  so a group or other user can no longer connect under a permissive umask,
  and no window exists in which a looser mode is reachable.
  See [Who can connect](#who-can-connect).
- A failure to create the Wayland listening socket (for example `XDG_RUNTIME_DIR` unset),
  to compile the default keymap,
  to register a client,
  or to bind,
  render,
  or present a frame now ends the session with an error message and exit status 1.
  These steps panicked with exit status 101 before.
- Nothing else changes.

## Changes in 0.6.0

- New `--gpu auto|software|NODE` option picks the device of a headless session;
  it is valid only together with `--headless`.
  Without it the behavior is `auto`,
  which starts on Mesa's default render node as before.
- `auto` now falls back instead of failing or silently rendering in software:
  Mesa's default render node,
  then the other render nodes,
  then Mesa's software rasterizer,
  with a log line that lists what failed.
  An explicit `--gpu` choice never falls back.
- New `--list-gpus` option prints the render nodes and the software renderer this host offers,
  and the one `auto` uses.
- New `gpu` control command answers the device in use.
- A headless session sets the hosted child's environment from the final choice:
  `LIBGL_ALWAYS_SOFTWARE=1` for software,
  and no `DRI_PRIME` for an explicit choice or a fallback.
- The nested window is unchanged,
  and `--gpu` without `--headless` is an error.
- New `inspect:headless-gpu` task.

## Changes in 0.5.0

- New `--headless` flag renders with surfaceless EGL on the DRM render node,
  with no parent display and no window.
  The control API,
  screenshots,
  recordings,
  input,
  the private bus,
  and accessibility are unchanged;
  see [Headless mode](#headless-mode).
- Headless is opt-in.
  Without the flag the nested window is used exactly as before,
  and a start with no parent display fails with a message that names `--headless`.
- A headless session sends frame callbacks from a 60 Hz timer from the first tick,
  see [Headless pacing](#headless-pacing).
- New `inspect:headless` task.
- A `resize` or `scale` that changes the screen while a recording runs answers the client's pending frame callbacks
  right after the new configure and scale,
  so the client draws and commits the new size before the next capture.
  See [Change the output scale](#change-the-output-scale).

## Changes in 0.4.0

- `key` takes every Linux key name,
  chords,
  `code:<n>`,
  and `hold MS`,
  with `repeat-info`,
  `--repeat-rate`,
  and `--repeat-delay`.
  Holding a key makes the client repeat it;
  modifier keys do not repeat.
- `type` takes any Unicode text.
- `resize` is exact at odd sizes,
  in captures as well as in the client.
- `drop-file` quotes paths with spaces,
  percent-encodes the URI,
  and takes several files.
- Keyboard focus returns to the most recently focused remaining toplevel when the focused one goes away.
  New commands:
  `windows`,
  `focus`,
  `close`,
  `display`.
- `click` takes `count`,
  `interval`,
  and `mods`;
  new `drag` and `scroll`;
  `wheel` takes fractional notches and `mods`.
- New `cursor` and `a11y` commands with `--accessibility`.

## License

LGPL-3.0-or-later.
 See [`LICENSES/`](./LICENSES).

[Smithay]: https://github.com/Smithay/smithay
[`cargo-binstall`]: https://github.com/cargo-bins/cargo-binstall
[`image`]: https://github.com/image-rs/image
