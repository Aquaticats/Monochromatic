# AB Download Manager 1.11.4 opens through XWayland on Plasma while mpv 0.42.0 uses Wayland

Unapproved investigation notes from 2026-11-18T21:00:44-05:00.
Codex verified the launch and socket observations on the CachyOS desktop.
Owner review and testing on other compositors remain open.

## Symptom

The Plasma 6.8.3 session reported `XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-1`
and `DISPLAY=:0`. KWin runs XWayland alongside its native socket.
AB Download Manager and mpv choose different window backends in this same environment.

## Root cause

The installed `ab-download-manager 1.11.4-1` launcher starts a bundled Java runtime.
Its module image contains `sun.awt.X11.XToolkit` and its library directory contains
`libawt_xawt.so`. Probing the image and library directory found no `WLToolkit` class
or `libawt_wlawt.so`. The cached CachyOS package's `.BUILDINFO` lists `jdk25-openjdk`.
The launch mapped an X11 window with `WM_CLASS=com-abdownloadmanager-desktop-AppKt`.

The official v1.11.4 Linux archive includes JetBrains' Wayland AWT library.
In JetBrains Runtime 25.0.5.0.3b700.17, `PlatformGraphicsInfo.java:47-70` selects the Java window toolkit
using `awt.toolkit.name`. The bundled launcher still produced the X11 headless error during the
Wayland-only test, including with `WLToolkit` passed through `JAVA_TOOL_OPTIONS`.
Launching the application class directly with this newer JetBrains Runtime and `-Dawt.toolkit.name=WLToolkit`
reached KWin but stopped while Skiko initialized the drawing surface.
[JetBrains Runtime source](https://github.com/JetBrains/JetBrainsRuntime/blob/cdca38823e56526cba4cb73fb34ec2a7ccbfaee9/src/java.desktop/unix/classes/sun/awt/PlatformGraphicsInfo.java#L47-L70)

```java
// src/java.desktop/unix/classes/sun/awt/PlatformGraphicsInfo.java
return  (getToolkitID() == TK_X11)?
    new sun.awt.X11.XToolkit() :
    new sun.awt.wl.WLToolkit();
```

Skiko v0.150.1 calls into Java AWT to lock the drawing surface.
`AWT.kt:35` rejects a failed lock. The observed exception was
`java.lang.IllegalStateException: Can't lock DrawingSurface`.
The same stack is recorded upstream with an older Skiko version.
[Skiko source](https://github.com/JetBrains/skiko/blob/3956e988e6e93eaf1ee985d725049443e7845807/skiko/src/awtMain/kotlin/org/jetbrains/skiko/AWT.kt#L33-L37)
[Upstream report](https://youtrack.jetbrains.com/issue/SKIKO-1147)

```kotlin
// skiko/src/awtMain/kotlin/org/jetbrains/skiko/AWT.kt
fun lock() = lockDrawingSurface(ptr).also { check(it == 0) { "Can't lock DrawingSurface" } }
```

On mpv 0.42.0, `video/out/gpu/context.c:85-95` places the Wayland Vulkan context first among Linux
Vulkan window backends. The default playback probe selected `waylandvk` and rendered the video.
The already-running mpv process also connected directly to KWin's Wayland socket.
[mpv source](https://github.com/mpv-player/mpv/blob/b0841e8c4d156b8403dc1876c4cbd9011eb583f3/video/out/gpu/context.c#L85-L95)

```c
// video/out/gpu/context.c
#if HAVE_WAYLAND
    &ra_ctx_vulkan_wayland,
#endif
#if HAVE_X11
    &ra_ctx_vulkan_xlib,
#endif
```

## Verification

Run a generated video and look for `Initializing GPU context 'waylandvk'` and `VO: [gpu-next]`.
Adding `--gpu-context=x11egl` with `DISPLAY` present is the XWayland control. It reports
`X11 opening display: :0`. The X11 window list can be read with `xprop -root _NET_CLIENT_LIST`.

```sh
# Terminal, mpv from the installed package
mpv --msg-level=vo=v --no-audio --frames=2 --geometry=320x180 -- av://lavfi:testsrc2=size=320x180:rate=5
```

Launch AB Download Manager once per disposable home, setting both `HOME` and Java's `user.home`
to that profile. The ordinary launcher opens an X11 window. With `DISPLAY` removed and
`WAYLAND_DISPLAY` retained, Java exits with `HeadlessException` and
`No X11 DISPLAY variable was set,`. The Wayland-only launch also fails with the toolkit property
set in the packaged launchers. A direct launch with the JetBrains Runtime reached Wayland and reproduced
Skiko's drawing surface error, including with `SKIKO_RENDER_API=SOFTWARE`.

```sh
# Terminal, spare profile under the agent scratch directory
probe_home=$(mktemp --directory --tmpdir="$HOME/temp/agent" ab-wayland.XXXXXX)
env --unset=DISPLAY HOME="$probe_home" XDG_CONFIG_HOME="$probe_home/config" \
  XDG_DATA_HOME="$probe_home/data" XDG_CACHE_HOME="$probe_home/cache" \
  JAVA_TOOL_OPTIONS="-Duser.home=$probe_home" /opt/abdownloadmanager/bin/ABDownloadManager
```

The live socket snapshot also found Telegram connected to both display servers and using Qt's Wayland
platform plugin. An X11 connection can coexist with a native window. The X11 window list and Wayland socket
peer inspection answer different questions, and they cover the apps active at the time of the sample.

## Verified workarounds

mpv can explicitly restrict video output to Wayland. Playback succeeded with `DISPLAY` unset and both
`--vo=gpu-next` and `--gpu-context=waylandvk,wayland` set. The EGL context provides a second Wayland option.
Pinning these options gives up automatic selection of other video outputs.
The installed AB Download Manager works through Plasma's XWayland compatibility server.

## What does not work

Removing `DISPLAY` does not add a Wayland toolkit to the CachyOS package.
The official tarball opens an XWayland window by default, and selecting software rendering
did not resolve the Skiko drawing-surface error in the direct Wayland test.

## Upstream filing decision

The active support request is AB Download Manager issue 427. Skiko's matching report is a duplicate of
its native Wayland work. This local result requires working support in the packaged Java and rendering
stack, and both upstream projects already track the use case. AB Download Manager's contribution guide
accepts bug reports and asks for an existing-issue search. The tracking issues establish interest in the
use case. No app-level fix was prototyped, and the existing Skiko report has the stack and renderer results
already. No filing artifact is needed.
[AB Download Manager issue](https://github.com/amir1376/ab-download-manager/issues/427#issuecomment-2883120375)
[Skiko tracking issue](https://youtrack.jetbrains.com/issue/SKIKO-28)

## mpv idle window sizing

On 2026-11-18T21:07:34-05:00, the supplied screenshot showed the `No file` window.
The live process used `--player-operation-mode=pseudo-gui`, as does the installed desktop entry.
With no video, mpv 0.42.0 configures a 960 × 540 placeholder in `player/playloop.c:1084-1095`.
[mpv source](https://github.com/mpv-player/mpv/blob/b0841e8c4d156b8403dc1876c4cbd9011eb583f3/player/playloop.c#L1084-L1095)

```c
// player/playloop.c
int w = 960;
int h = 540;
```

The default probe drew a 960 × 540 buffer with a 480 × 270 Wayland viewport at 200% display scale.
This is about 44% of the portrait display's 1080 logical pixels of width.
The screenshot's black surface agrees with that default. The desktop launcher uses the same code path.

```sh
# Terminal, launch the desktop entry's mpv mode with Wayland tracing
env WAYLAND_DEBUG=client timeout --foreground 3s mpv --no-config --player-operation-mode=pseudo-gui
```

`--geometry=65%` increased the measured width to 702 logical pixels, or 1404 physical pixels, on the
same output. Setting `geometry=65%` in `mpv.conf` applies the sizing to normal launches and also affects
the initial size of windows opened for media. The placeholder size is documented behavior, and the local
geometry option covers the reported use case without a filing.

AB Download Manager 1.11.4-1 was subsequently removed with Pacman.
The package, system desktop entry, executable, and stale login and browser registrations were checked.

## mpv text in the right-click menu and idle window

On 2026-11-18T21:16:23-05:00, the follow-up screenshot showed small text inside the mpv window.
The menu and idle label use separate overlays, so window geometry alone does not diagnose the symptom.

In `player/lua/context_menu.lua:82-95`, the default `scale_with_window=auto` follows
`osd-scale-by-window`, which defaults to `yes`. The menu divides the 540-pixel overlay height by 720
and scales the configured font of 14 by that factor. Choosing `scale_with_window=no` instead uses
the compositor-reported `display-hidpi-scale`, which the probe measured as 2.0.
[mpv context menu source](https://github.com/mpv-player/mpv/blob/b0841e8c4d156b8403dc1876c4cbd9011eb583f3/player/lua/context_menu.lua#L82-L95)

```lua
-- player/lua/context_menu.lua
if should_scale() and osd_height > 0 then
    return osd_height / 720
end

return mp.get_property_native("display-hidpi-scale", 1)
```

In `player/lua/osc.lua:3109-3116`, the idle label is appended without a size override. The overlay
uses the `OSD` style, which `sub/osd_libass.c:538-549` builds from the configured OSD options.
[mpv idle label source](https://github.com/mpv-player/mpv/blob/b0841e8c4d156b8403dc1876c4cbd9011eb583f3/player/lua/osc.lua#L3109-L3116)
[mpv overlay style source](https://github.com/mpv-player/mpv/blob/b0841e8c4d156b8403dc1876c4cbd9011eb583f3/sub/osd_libass.c#L538-L549)

```c
// sub/osd_libass.c
mp_ass_set_style(get_style(&ext->ass, "OSD"), resy, osd->opts->osd_style);
```

With both native Wayland probes at 960 × 540, the word `Open` measured 7 pixels tall at the default
and 18 pixels with `scale_with_window=no` in `script-opts/context_menu.conf`. The larger menu paged
items that did not fit. In separate idle captures, `--osd-font-size=48` increased the label's bright-pixel
height from 14 to 24 pixels. That option also changes the size of regular OSD text.
mpv's own screenshot command requires a decoded frame, so the menu captures used a generated black
video and the idle captures used Spectacle on the test window. Both probes used disposable profiles.

### Platform differences and upstream reports

On 2026-11-18T21:22:36-05:00, the owner approved saving the menu and OSD settings.
A fresh launch with the installed desktop entry's `--player-operation-mode=pseudo-gui` option
read `~/.config/mpv/mpv.conf` with `osd-font-size=48`, loaded
`~/.config/mpv/script-opts/context_menu.conf` with `scale_with_window=no`,
and selected `waylandvk`. The idle buffer still measured 960 × 540.

The overlay is drawn inside the player. Its author explained that shrinking it helps more rows
fit, since the overlay cannot extend past the edge of the player. Another user reported Wayland
menu scaling in the right-click binding PR. Their comment does not specify window dimensions or
display scale, so it establishes a related report rather than a duplicate of the measured case.
[Menu author's explanation](https://github.com/mpv-player/mpv/pull/16726#issuecomment-3226817332)
[Wayland scaling report](https://github.com/mpv-player/mpv/pull/18057#issuecomment-6703374928)

In v0.42.0, `options/options.c:1039-1041` enables the Lua menu by default when `_WIN32`
is absent. Windows can use the system popup through `video/out/win32/menu.c:212-229`.
The native macOS context menu was a separate open proposal as of this check.
The Wayland and X11 builds can both use the Lua menu. For that overlay, doubling the video area
height doubles the default menu font size. These platform findings came from source inspection.
[Windows popup source](https://github.com/mpv-player/mpv/blob/b0841e8c4d156b8403dc1876c4cbd9011eb583f3/video/out/win32/menu.c#L212-L229)
[macOS context menu proposal](https://github.com/mpv-player/mpv/pull/18849)

```c
// video/out/win32/menu.c
TrackPopupMenuEx(ctx->menu, TPM_LEFTALIGN | TPM_LEFTBUTTON, pt.x, pt.y,
                 hwnd, NULL);
```
