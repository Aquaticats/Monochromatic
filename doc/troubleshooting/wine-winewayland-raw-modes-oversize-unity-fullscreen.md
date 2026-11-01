# Wine 11.0 winewayland raw panel modes oversize Unity borderless fullscreen windows on rotated or scaled outputs

## Symptom

Wine 11.0 (Flathub `org.winehq.Wine`, `WINE_VIDEO_DRIVER=wayland`) on KDE Plasma Wayland with rotated and
scaled outputs: a Unity 2022.3 IL2CPP game (`PlantsVsZombiesRH.exe`, Plants vs Zombies Fusion 4.0.5) opens
with a window larger than the entire logical desktop, spilling across every output.
User report: "It's now extremely huge when launched."

The surface trigger is Unity's borderless fullscreen convention: when `Screenmanager Fullscreen mode` is
`FullScreenWindow`, Unity sizes its window from the current display mode of the monitor.
The affected desktop has three 4K panels rotated to portrait at compositor scale 2 (logical 1080x1920 each)
plus one 4K panel at scale 3 (logical 1280x720), so the logical desktop spans 3240x2640.
The game window came out 3840x2160 in window coordinates.

A second, game-side variant appears once the window is sized correctly: this game is designed for 16:9, so
a 9:16 window stretches its content.
User report: "Looks like it was made for 16:9. Non 16:9 = everything is stretched."
That distortion is the game filling a non-16:9 window and is not the driver bug.

## Root cause

The call chain that produces the failure:

1.  Unity reads the current display mode through `EnumDisplaySettings` and creates a borderless popup
    window sized to it. This is the standard Windows borderless fullscreen pattern and works on Windows.
2.  winewayland fills `DEVMODEW` entries from the raw `wl_output` mode dimensions:

```c
// dlls/winewayland.drv/display.c:233 (Wine 11.0, db11d0fe6a1)
    mode->dmPelsWidth = output_mode->width;
    mode->dmPelsHeight = output_mode->height;
```

3.  Output orientation is ignored everywhere. `populate_devmode` hardcodes the orientation field and the
    protocol handler that receives the transform throws it away:

```c
// dlls/winewayland.drv/display.c:230 (Wine 11.0, db11d0fe6a1)
    mode->dmDisplayOrientation = DMDO_DEFAULT;
```

```c
// dlls/winewayland.drv/wayland_output.c:196 (Wine 11.0, db11d0fe6a1)
static void output_handle_geometry(void *data, struct wl_output *wl_output,
                                   int32_t x, int32_t y,
                                   int32_t physical_width, int32_t physical_height,
                                   int32_t subpixel,
                                   const char *make, const char *model,
                                   int32_t output_transform)
{
}
```

4.  The monitor rectangle reported to the GDI device manager is built from the same raw mode dimensions:

```c
// dlls/winewayland.drv/display.c:213 (Wine 11.0, db11d0fe6a1)
    SetRect(&monitor.rc_monitor, output_info->x, output_info->y,
            output_info->x + output_info->output->current_mode->width,
            output_info->y + output_info->output->current_mode->height);
```

5.  The compositor's logical size and position are known to the driver, but only steer output overlap
    resolution, and fall back to the raw mode when the compositor omits them:

```c
// dlls/winewayland.drv/wayland_output.c:262 (Wine 11.0, db11d0fe6a1)
    output->pending.logical_w = width;
    output->pending.logical_h = height;
```

```c
// dlls/winewayland.drv/display.c:146 (Wine 11.0, db11d0fe6a1)
                    (x_use_end ? move->output->logical_w : 0)) /
                    (double)anchor->output->logical_w;
```

6.  Window coordinates meanwhile live in xdg logical space, one Wine pixel per logical pixel.
    The pointer trace maps a Wayland logical coordinate to the identical screen coordinate, and a window
    created at 1072x603 came out exactly 1080 wide including its frame, matching the 1080 logical width of
    the portrait output.
7.  For the incident numbers, the mode list, the logical geometry, and the resulting window disagree:

```text
# excerpt from WINEDEBUG=+waylanddrv trace of PlantsVsZombiesRH.exe (Wine 11.0)
0074:trace:waylanddrv:zxdg_output_v1_handle_logical_size logical_w=1080 logical_h=1920
0074:trace:waylanddrv:wayland_output_done name=DP-2 logical=1080,0+1080x1920
0074:trace:waylanddrv:wayland_output_done mode 3840x2160 @ 59997 *
017c:trace:waylanddrv:pointer_handle_motion_internal hwnd=0x20082 wayland_xy=658.60,1646.10 screen_xy=659,1646
0024:trace:waylanddrv:wayland_surface_reconfigure_size hwnd=0x20082 size=3840x2160
```

So the game asked for 3840x2160, the raw landscape panel mode of a panel that is rotated to 2160x3840
physical and presented as 1080x1920 logical, and the driver turned that into a 3840x2160 logical window,
wider than the 3240x2640 logical desktop.

A hypothesis from this investigation was wrong and is worth recording: Wine window coordinates do not
equal physical pixels that the compositor then scales.
The pointer trace quoted in the root cause chain shows a 1:1 mapping between Wayland logical and screen
coordinates on a scale 2 output, and the fitted window matched the logical output width exactly.
The oversize window is a coordinate space mismatch between the mode list and the window space, made worse
by ignored rotation, not a compositor scaling artifact.

Master does not fix this.
`git diff db11d0fe6a1 f343ca6085b -- dlls/winewayland.drv/wayland_output.c` is empty, so
`output_handle_geometry` still discards the transform, and the only change to `display.c` re-anchors
monitor and mode positions to the primary output while keeping `current_mode->width` for the size:

```c
// dlls/winewayland.drv/display.c (master, f343ca6085b), wayland_add_device_monitor
    SetRect(&monitor.rc_monitor, output_info->x, output_info->y,
            output_info->x + output_info->output->current_mode->width,
            output_info->y + output_info->output->current_mode->height);
    OffsetRect(&monitor.rc_monitor, -primary->x, -primary->y);
```

Master does add `fractional-scale-v1.xml` surface scaling that Wine 11.0 lacks, which is the direction
upstream wants for HiDPI, but rotation is still unhandled in the mode and monitor reporting.

## Verification

Version under test: Flathub `org.winehq.Wine` stable, Wine 11.0 (runtime commit
`2b7feee9cd37329c1fecd0c45960b0634cd0e684a3f1fd895df8771612ed2865`), source tree
`wine-mirror/wine` tag `wine-11.0` (`db11d0fe6a1`).
Compositor: KDE Plasma Wayland (`kscreen-doctor -o` reports the three portrait outputs as
`1080x1920` logical at `Scale: 2`, `Rotation: 2`).

Harness: run the Unity player with the wayland driver and read the window geometry from the driver trace:

```sh
# launch with tracing, from the game's directory
env -u DISPLAY WINEPREFIX=/var/home/user/.local/share/wineprefixes/pvzfusion WINEARCH=win64 \
  WINE_VIDEO_DRIVER=wayland WINEDEBUG=+waylanddrv \
  flatpak run org.winehq.Wine PlantsVsZombiesRH.exe > probe.log 2>&1
```

```sh
# read output geometry, mode list, and window rectangles
rg -n "logical_size|wayland_output_done|WindowPosChanged" probe.log
```

Failing catalog (Unity `FullScreenWindow`):

- `Screenmanager Fullscreen mode_h3630240806` = 1 with any saved `Screenmanager Resolution` value
  produces the 3840x2160 window quoted in the root cause chain.
- The xdg configure for that window reports `1080x1920` while the surface size stays `3840x2160`, so the
  window and the compositor disagree about its extent:

```text
# excerpt from WINEDEBUG=+waylanddrv trace (FullScreenWindow run)
017c:trace:waylanddrv:xdg_toplevel_handle_configure hwnd=0x20082 1080x1920,0x8
0024:trace:waylanddrv:wayland_surface_reconfigure_geometry hwnd=0x20082 geometry=(0,0)-(1080,1920)
0024:trace:waylanddrv:wayland_surface_reconfigure_size hwnd=0x20082 size=3840x2160
```

- Filling a portrait output at 9:16 (windowed client 1080x1920) fits one output but stretches 16:9 game
  content, which the user confirmed on screen.

Working catalog (windowed with explicit sizes):

- Windowed client 1080x1920 yields outer window 1088x1954, that is the client plus the 8x34 window frame.
- Windowed client 1072x603 yields outer window 1080x637: the outer width equals the portrait output's
  logical width and the client is exactly 16:9, so a 16:9 game renders undistorted.

```text
# excerpt from WINEDEBUG=+waylanddrv trace (client 1072x603 run)
0024:trace:waylanddrv:WAYLAND_WindowPosChanging hwnd 0x20082, swp_flags 0815, shaped 0, rects { window (0,0)-(1080,637), client (4,30)-(1076,633), visible (0,0)-(1080,637) }
```

## Verified workarounds

1.  Consumer-side sizing recipe: run the Unity player windowed and pass explicit sizes computed from the
    target output's logical geometry (`kscreen-doctor -o` or the `zxdg_output_v1_handle_logical_size`
    trace line), never from the mode list. For a 16:9 game on a 1080x1920 portrait output the client is
    1072x603, because the 8x34 window frame then makes the outer window exactly 1080x637:

```sh
# launcher fragment, from /var/home/user/.local/share/applications/pvz-fusion.desktop
Exec=env -u DISPLAY WINEPREFIX=/var/home/user/.local/share/wineprefixes/pvzfusion WINEARCH=win64 WINE_VIDEO_DRIVER=wayland WINEDLLOVERRIDES=version=n,b DOTNET_SYSTEM_GLOBALIZATION_USENLS=1 flatpak run org.winehq.Wine PlantsVsZombiesRH.exe -screen-width 1072 -screen-height 603 -screen-fullscreen 0
```

    Matching registry values under `HKCU\Software\LanPiaoPiao\PlantsVsZombiesRH` keep the game from
    reverting to `FullScreenWindow` and pin the size:

```text
# registry fragment for the Unity player
"Screenmanager Fullscreen mode_h3630240806"=dword:00000003
"Screenmanager Resolution Width_h182942802"=dword:00000430
"Screenmanager Resolution Height_h2627697771"=dword:0000025b
"Screenmanager Resolution Window Width_h2524650974"=dword:00000430
"Screenmanager Resolution Window Height_h1684712807"=dword:0000025b
```

    Tradeoffs: the 30 logical pixel title bar stays visible at the top of the window; window placement on
    Wayland is compositor-side, so the window opens on the active output and may need dragging; the game
    image is upscaled by the compositor scale factor (2x here) because window coordinates are logical, so
    it is softer than a physical-pixel rendering of the same panel; Unity rewrites these registry values
    at exit, so the launcher arguments are what actually pin the size across runs.

2.  Aspect matching for fixed-aspect games: compute the client size in the game's aspect ratio rather
    than filling the output. 1072x603 is exactly 16:9 and as wide as the portrait output allows once the
    frame is counted. Tradeoff: the window covers a horizontal strip of the portrait output and leaves
    the rest unused, which is inherent to playing a 16:9 game on a 9:16 monitor.

## What does not work

- Keeping Unity's `FullScreenWindow` mode and setting `Screenmanager Resolution Width_h182942802` and
  `Screenmanager Resolution Height_h2627697771`: Unity ignores the saved size in that mode and covers the
  display as the driver reports it, so the window stays 3840x2160. Verified by saving 1920x1080 with
  fullscreen mode 1 and tracing the resulting window.
- Filling the portrait output at 9:16 (windowed 1080x1920): the window fits one output but the 16:9 game
  content stretches, as the user confirmed.
- Setting `Screenmanager Resolution Use Native_h1405027254` to 0: it does not change the mode list, and
  the mode list is what the driver reports. Verified in the same registry block.

## Prototype of the minimal fix

The prototype lives in a fresh private clone of `https://github.com/wine-mirror/wine.git` at
`${HOME}/temp/agent/upstream-prototype.b6w0B90E/wine`, created with `mktemp` and checked to
`db11d0fe6a1` (tag `wine-11.0`) before any edit, matching the source cited in the root cause chain.

The minimal fix is kept as
[the prototype patch](wine-winewayland-raw-modes-oversize-unity-fullscreen.patch):
83 insertions and 18 deletions across `dlls/winewayland.drv/waylanddrv.h`,
`dlls/winewayland.drv/wayland_output.c`, and `dlls/winewayland.drv/display.c`.
It stores `wl_output.transform` (the `output_handle_geometry` body was empty), applies it in
`populate_devmode` (dimension swap plus `dmDisplayOrientation`), scales each mode into the logical
output size, and builds the monitor rect and the overlap math from the logical geometry.

Build and verification:

```sh
# disposable clone build, 64-bit only because the host lacks 32-bit toolchains
./configure --enable-win64 --without-gnutls
make -j"$(nproc)"
```

```sh
# harness: run any windowed builtin with the wayland driver and read the reported geometry
env -u DISPLAY WINEPREFIX="${HOME}/temp/agent/upstream-prototype.b6w0B90E/prefix" \
  WINEDEBUG=+waylanddrv,+system ./wine notepad > harness.log 2>&1
rg "wayland_add_device_monitor|trace:system:add_modes current" harness.log
```

A `EnumDisplaySettingsW` probe binary was the first harness choice, but winegcc in the build tree
cannot link an out-of-tree winelib app (`-lwinecrt0` thunks are unresolvable) and the host is a Fedora
Atomic image where `dnf` refuses to install a mingw cross compiler, so the harness uses the driver's
own `rc_monitor` trace plus win32u's `add_modes` trace, which print the same values.

Before the patch (bug visible: portrait outputs report the raw landscape mode, rotated 0 degrees):

```text
# excerpt from harness.log on the unpatched build (Wine 11.0, db11d0fe6a1)
007c:trace:waylanddrv:wayland_add_device_monitor name=DP-3 rc_monitor=rc_work=(0,0)-(3840,2160)
007c:trace:system:add_modes current 3840x2160 32bits 60Hz rotated 0 degrees ... at (0,0), ...
007c:trace:waylanddrv:wayland_add_device_monitor name=DP-2 rc_monitor=rc_work=(3840,0)-(7680,2160)
007c:trace:waylanddrv:wayland_add_device_monitor name=HDMI-A-1 rc_monitor=rc_work=(7680,0)-(11520,2160)
007c:trace:waylanddrv:wayland_add_device_monitor name=HDMI-A-2 rc_monitor=rc_work=(3470,2160)-(7310,4320)
```

After the patch (rects and modes match `kscreen-doctor -o` logical geometry exactly):

```text
# excerpt from harness.log on the patched build
0074:trace:waylanddrv:wayland_add_device_monitor name=DP-3 rc_monitor=rc_work=(0,0)-(1080,1920)
0074:trace:system:add_modes current 1080x1920 32bits 60Hz rotated 90 degrees ... at (0,0), ...
0074:trace:waylanddrv:wayland_add_device_monitor name=DP-2 rc_monitor=rc_work=(1080,0)-(2160,1920)
0074:trace:system:add_modes current 1080x1920 32bits 60Hz rotated 90 degrees ... at (1080,0), ...
0074:trace:waylanddrv:wayland_add_device_monitor name=HDMI-A-1 rc_monitor=rc_work=(2160,0)-(3240,1920)
0074:trace:system:add_modes current 1080x1920 32bits 60Hz rotated 90 degrees ... at (2160,0), ...
0074:trace:waylanddrv:wayland_add_device_monitor name=HDMI-A-2 rc_monitor=rc_work=(976,1920)-(2256,2640)
0074:trace:system:add_modes current 1280x720 32bits 60Hz rotated 0 degrees ... at (976,1920), ...
```

Result: the current mode a Win32 app reads becomes 1080x1920 on the portrait outputs and 1280x720 on
the scale 3 output, with `dmDisplayOrientation` matching the rotation, so the borderless fullscreen
convention sizes its window to one output instead of the raw 3840x2160 panel mode. The game itself was
not run against the patched build: its MelonLoader and .NET stack runs in the Flatpak Wine prefix, and
the prototype harness targets the reported geometry, which is the value the fix changes.

## Upstream filing artifact

`.out-of-scope/` holds no entry for Wine, winewayland, or display mode reporting, so no exemption applies.

Duplicate search: `bugs.winehq.org` is unreachable from this environment behind its anti-bot gate, as
recorded in `wine-icu-forward-mismatch-breaks-dotnet.md`, so the tracker itself could not be queried.
Web search found no WineHQ bug describing this mode and rotation mismatch. Related but distinct signals:

- [GNOME mutter work item 4042][mutter-4042] reports Wine Wayland windows made 200% too large at 200%
  display scale, a compositor scaling interaction, not a mode list mismatch.
- [Wine merge request 4947][wine-mr-4947] emulates display mode changes with compositor surface scaling.
- On mutter work item 4042, winewayland contributor Etaash Mathamsetty states the design: "The wl_output
  display modes are used to determine the resolution of the outputs and that is exposed to the game using
  the same mechanisms available on windows. The games choose the resolution they run at, as they do on
  windows. The fractional scale protocol is then used to convert to logical scale."

Wine rejects LLM-generated code (see the decision walk), so the kept artifact is a text-only report with
no patch attached. It is marked "do not file as-is" until a human re-runs the reproduction and checks the
source citations and the tracker, since those checks were assistant-run.

~~~md
Title: winewayland: display modes and monitor rects ignore output transform and scale, oversizing
borderless fullscreen windows

Hello,

winewayland reports display modes and monitor rectangles in raw panel pixels while window coordinates
live in xdg logical coordinates. On a rotated or scaled output the two disagree, so Windows apps that
size a borderless "fullscreen" window from the current display mode create a window larger than the
logical desktop.

Reproduction (KDE Plasma Wayland, one output rotated to portrait at scale 2, logical 1080x1920):

1. Run any Unity player with Screenmanager Fullscreen mode = FullScreenWindow under WINE_VIDEO_DRIVER=
   wayland, WINEDEBUG=+waylanddrv.
2. The trace shows the mode as 3840x2160 and the logical output as 1080x1920, and the game window as
   3840x2160, that is, wider than the whole logical desktop.

Trace excerpt:

  0074:trace:waylanddrv:zxdg_output_v1_handle_logical_size logical_w=1080 logical_h=1920
  0074:trace:waylanddrv:wayland_output_done mode 3840x2160 @ 59997 *
  0024:trace:waylanddrv:WAYLAND_WindowPosChanged hwnd 0x10020 new_rects { window (0,0)-(3840,2160), ... }

Source trace (wine-11.0, db11d0fe6a1):

- dlls/winewayland.drv/display.c:233 fills dmPelsWidth and dmPelsHeight from the raw mode.
- dlls/winewayland.drv/display.c:213 builds the monitor rect from the same raw mode dims.
- dlls/winewayland.drv/display.c:230 hardcodes dmDisplayOrientation to DMDO_DEFAULT.
- dlls/winewayland.drv/wayland_output.c:196 output_handle_geometry has an empty body, so
  wl_output.transform never reaches the mode or monitor reporting.
- dlls/winewayland.drv/wayland_output.c:262 stores the zxdg logical size, but display.c only uses it for
  output overlap resolution.

On Windows a rotated display reports its modes rotated and shares one coordinate space between the mode
list, monitor rects, and windows, so this borderless fullscreen pattern lands exactly on the display.

Suggested fix, in text form only: apply wl_output.transform when populating DEVMODEW entries (swap
dimensions for 90 and 270 degree transforms and set dmDisplayOrientation accordingly) and derive
monitor rects and mode positions from the zxdg logical geometry already kept in output->current.logical_*.

Disclosure: this report and its diagnosis were prepared with an AI coding assistant and verified against
the wine-11.0 source tree and live driver traces. No AI-generated code is attached, per the project FAQ.
~~~md

### Upstream filing decision

1.  Is it really upstream's fault? Yes. Unity uses the standard Windows borderless fullscreen pattern,
    and on Windows the mode list follows the output orientation and shares the window coordinate space.
    winewayland hardcodes `DMDO_DEFAULT` (`display.c:230`), discards `wl_output.transform`
    (`wayland_output.c:196`), and reports raw mode dimensions for both modes and monitor rects
    (`display.c:213`, `display.c:233`) while its windows live in logical coordinates. The physical-mode
    exposure for HiDPI games is a stated design choice, but ignoring rotation is a gap inside that
    design, not the design itself.
2.  Can upstream fix it? Yes. The change is localized to `dlls/winewayland.drv/display.c` and the empty
    `output_handle_geometry` in `dlls/winewayland.drv/wayland_output.c`, and it does not touch the
    win32u display device contract.
3.  Are they supporting this use case? Yes, with a caveat. winewayland advertises multi-monitor and
    HiDPI support and already reads `zxdg_output` logical geometry, and rotated outputs are ordinary
    Wayland usage. The caveat is the quoted design statement that games should pick physical sizes and
    let surface scaling convert them, so a fix must keep that path working for fullscreen surfaces.
4.  Would the repo welcome our contribution? Not as code. Wine's developer FAQ answers "No." to
    LLM-generated code ("The copyright and license status of LLM-generated code is not well defined, and
    we cannot assume we are allowed to distribute this code under the Wine license"), and Alexandre
    Julliard repeated the rule on wine-devel in April 2026 ("In order to ship any code, we need a proper
    authorship record, and an LGPL-compatible license ... so we can't accept such code"). Text-only
    reports are acceptable in kind: [WineHQ bug 60397][bug-60397] and [bug 60417][bug-60417] were filed
    as analysis plus fix approach with no patch, exactly because the prototype was LLM-written. The kept
    artifact follows that shape and attaches no code.
5.  Will they likely fix it? No leaning-no signal. Nothing in the FAQ, tracker echoes, or maintainer
    mail refuses orientation-aware mode reporting, and the fractional-scale work on master shows active
    investment in the coordinate space story. The design statement constrains the shape of a fix rather
    than declining one.
6.  Have we prototyped a minimal fix compatible with their architecture? Yes. The prototype is recorded
    in the prototype section of this doc, built and verified in a disposable `wine-mirror/wine` clone at
    `db11d0fe6a1`.

Outcome: do not file as-is. Two gates remain, both human-side: re-run the reproduction and the source
citation checks so the disclosure can name a human verification, and re-check the tracker once
`bugs.winehq.org` is reachable. Past those gates the kept text-only draft is fileable; the patch must
stay out of the filing regardless, because Wine does not accept LLM-generated code.

[mutter-4042]: https://gitlab.gnome.org/GNOME/mutter/-/work_items/4042
[wine-mr-4947]: https://gitlab.winehq.org/wine/wine/-/merge_requests/4947
[bug-60397]: https://bugs.winehq.org/show_bug.cgi?id=60397
[bug-60417]: https://bugs.winehq.org/show_bug.cgi?id=60417
