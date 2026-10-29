# nested-wayland-session 0.1.1: `screenshot` kept returning the hosted client's first frame (host session found locked)

## Symptom

On 2026-10-05,
during native verification of the Slint IDE's in-file find,
every `screenshot PATH` control command answered `ok` and wrote a valid PNG,
but each PNG showed the hosted application's first frame.
Seat input sent through the same control socket (`key`,
 `type`,
 `click`) reached the application:
its accessible tree and Slint's own `take_screenshot` showed an open find bar,
a different file,
and highlighted matches.
Three compositor screenshots taken around further input were byte-identical:

```text
fbd3c8209657f149  debug-compositor.png
fbd3c8209657f149  debug-compositor-2.png
fbd3c8209657f149  debug-compositor-3.png
```

The host desktop session was locked when it was checked after the stalled session had ended:
`loginctl show-session` reported `LockedHint=yes`.
The lock state was not sampled during the stall itself.
No error or warning was printed by the compositor.

Fixed in commit `33d0dce2b` of this repository on 2026-10-05.
The sections from "Root cause" on describe the binary before that commit unless they say otherwise.

## Root cause

The compositor sent frame callbacks to the hosted client only after presenting to its own parent compositor,
and its windowing library delivers the next redraw only after the parent's frame callback.
A parent that stops presenting the nested window therefore stopped the hosted client from drawing.

`screenshot` re-renders the scene from the client's last committed buffer and reads it back.
It does not send frame callbacks:

```rust
// package/nested-wayland-session/cli/src/screenshot.rs:174
pub fn capture(state: &mut Compositor, path: &Path) -> Result<()> {
    let mut pixels = Vec::new();

    let (width, height) = read_frame(state, &mut pixels)?;

    encoder::write_flipped(&pixels, width, height, path, encoder::Format::Png)
        .with_context(|| format!("writing screenshot to {}", path.display()))?;

    return Ok(())
}
```

Before the fix,
frame callbacks were sent from the live redraw path,
after presenting to the parent compositor,
and that path re-armed itself through the parent window:

```rust
// package/nested-wayland-session/cli/src/render.rs at commit 82876cfe7, line 139
    state.backend.submit(Some(&[damage])).unwrap();

    send_frame_callbacks(state);

    let _ = state.display_handle.flush_clients();

    state.backend.window().request_redraw();
```

`submit` asks the parent for a frame callback before swapping:

```rust
// smithay-0.7.0/src/backend/winit/mod.rs:349
        // Request frame callback.
        self.window.pre_present_notify();
        self.egl_surface.swap_buffers(damage.as_deref_mut())?;
```

winit records that request,
and while it is outstanding it emits no redraw event,
whatever `request_redraw()` asked for:

```rust
// winit-0.30.13/src/platform_impl/linux/wayland/window/state.rs:253
    pub fn request_frame_callback(&mut self) {
        let surface = self.window.wl_surface();
        match self.frame_callback_state {
            FrameCallbackState::None | FrameCallbackState::Received => {
                self.frame_callback_state = FrameCallbackState::Requested;
                surface.frame(&self.queue_handle, surface.clone());
            },
            FrameCallbackState::Requested => (),
        }
    }
```

```rust
// winit-0.30.13/src/platform_impl/linux/wayland/event_loop/mod.rs:486
                if window.frame_callback_state() == FrameCallbackState::Requested {
                    return None;
                }
```

So `redraw` ran again only when the parent answered:

```rust
// package/nested-wayland-session/cli/src/app.rs:245
        WinitEvent::Redraw => {
            redraw(state);
        }
```

The hosted Slint client is itself a winit 0.30.13 Wayland client and is gated the same way by this compositor.
Slint's renderers call the same notification before presenting,
for example `i-slint-backend-winit-1.18.1/renderer/femtovg/glcontext.rs:33`
(`self.winit_window.pre_present_notify();`),
and likewise `renderer/skia.rs:204` and `renderer/sw.rs:164`.
A client whose last frame callback is still outstanding does not redraw,
so it commits no new buffer and `screenshot` keeps compositing the old one.
A client whose last callback had already arrived can still draw one more frame unprompted;
every later change waits.

The recorder was the one path that sent frame callbacks on its own timer,
independent of the parent's presentation
(`package/nested-wayland-session/cli/src/recorder.rs`,
 `render::send_frame_callbacks(state)` in `capture_and_reschedule`),
which is why short recordings showed current frames.

### Status of the two links first recorded as inferred

- That a locked KDE session stops presenting the nested window:
  not read in KWin source,
  but observed directly.
  With `LockedHint=yes`,
  the fixed binary logs
  `parent compositor is not presenting this window; pacing the hosted client from a timer silent_ms=50`
  144 ms after its appearance-portal startup line in the measured run,
  which means it had submitted no frame to the parent for 50 ms.
- That the Slint winit client waits for a frame callback before its next buffer:
  traced in the winit and Slint sources quoted in this section.

## Fix

`package/nested-wayland-session/cli/src/frame_pacing.rs` registers a timer at the nested output's 60 Hz refresh.
`redraw` timestamps every frame it submits to the parent.
When no frame has been submitted for 50 ms and no recording is running,
the timer sends the frame callbacks itself.
While a recording runs,
the recorder keeps owning the cadence.
The compositor logs when the fallback starts and when the parent presents again.

`screenshot` is unchanged:
it still composites the latest committed buffer,
which is now current because the client is no longer starved.
A caller still has to allow the client time to draw after input,
as on any compositor.

## Verification

Versions under test:
`package/nested-wayland-session/cli` 0.1.1.
"Before" is a debug binary built from commit `82876cfe7`
(the runtime `color-scheme` command present,
 the pacing module present but not wired in).
"After" is the release binary built from the working tree that became commit `33d0dce2b`.
Hosted client:
`slint-viewer` 1.18.1 showing a scene that displays `Palette.color-scheme`
and reports each value it applies.

### Harness

Deterministic,
independent of the host's lock or window state by construction:

```sh
mise run //package/nested-wayland-session/cli:inspect:stalled-parent
```

It hosts one session inside another.
A `record DIR 0.1 bmp` in the outer session makes the outer recorder the only sender of frame callbacks,
one every ten seconds,
so the inner session's parent withholds them exactly as a hidden window does.
The task then switches the inner session's color scheme three times,
takes an inner `screenshot` after each,
and compares checksums.

Host-dependent,
when the host session happens to be locked:

```sh
mise run //package/nested-wayland-session/cli:inspect:color-scheme
```

Its first phase takes screenshots around two switches without ever recording.

### Failing cases

All measured on 2026-10-05 with `LockedHint=yes`.

- Before,
  single session:
  the screenshots taken before and after `color-scheme light` had the same checksum
  (`cc28a666d87e87f7…`),
  while the hosted Slint client had already reported `light`.
  The check stopped with `screenshot did not change after the switch to light`.
- Before,
  session inside a session:
  four screenshots around three switches all had checksum `cc28a666d87e…`.
  The check stopped with `inner screenshot did not change after the first switch`.
- Before,
  the original report:
  three screenshots around `key escape`,
   300 ms apart,
  were byte-identical in the session at `/tmp/monochromatic-ide-native-3r2Gu7`.
- Unit level,
  at commit `82876cfe7`:
  `test:all` reported 54 passed and 2 failed
  (`silent_parent_triggers_fallback_from_the_threshold_on`,
   `fallback_follows_parent_presentation`).

### Passing cases

- After,
  single session:
  screenshots were `cc28a666d87e` (dark),
  `ccdb157ba532` (light),
  `cc28a666d87e` (dark again),
  with no recording in between.
- After,
  session inside a session:
  `Starved screenshots: dark cc28a666d87e, light ccdb157ba532, dark cc28a666d87e, light ccdb157ba532 in 1939 ms`.
- After,
  `test:all` reports 56 passing tests,
  including the six in `src/frame_pacing_tests.rs`.
  With the "never while recording" guard removed in a disposable copy,
  `recorder_keeps_ownership_of_the_frame_cadence` fails.
- Before and after,
  a short recording showed current frames:
  `record DIR 10 png`,
   900 ms,
   `record stop` answered
  `ok captured=9 dropped=0 failures=0 seconds=0.953 fps=9.4` in the original report.

Not measured:
either task with the host session unlocked and the nested window visible.
The host reported `LockedHint=yes` at every check during the session that made this change,
and unlocking it is not something an agent may do.
In that state the single-session check does not exercise the fallback at all;
the session-inside-a-session check is the one designed to.

### Local evidence

Disposable,
not tracked,
under `~/temp/agent/live-theme-Z50lLh/evidence`:
`color-scheme-prefix-locked-stale` and `stalled-parent-prefix-stale` hold the identical screenshots from before the fix,
`color-scheme-release-pass` and `stalled-parent-release-pass` the differing ones from after it,
and `logs` the test and task output.

## Verified workarounds

For a binary built before commit `33d0dce2b`,
take evidence images from a short recording instead of `screenshot`:

```js
// probe helper; `control` writes one line to the control socket and returns the reply
await control('record ' + frames + ' 20 png');
await wait(500);
await control('record stop');
const last = readdirSync(frames).sort().at(-1);
```

Tradeoffs:
it writes about ten PNG files per image and takes half a second;
the first frames of a recording can still show the old buffer,
so only the last frame is evidence;
`redraw` returns early while a recorder exists,
so nothing is presented to the parent window during the recording.

Tradeoffs of the fix itself:
a hosted client whose parent window is hidden now keeps drawing at up to 60 Hz instead of idling,
and a parent that presents more slowly than every 50 ms is supplemented by the timer,
so the client can draw faster than the parent shows.

## What does not work

- Repeating `screenshot`,
  or sending input first,
  on a binary without the fix:
  the client still receives no frame callback.
- Waiting longer before the screenshot on a binary without the fix:
  the client is not slow,
  it is waiting for a callback that never comes.
- Slint's MCP `take_screenshot` as a substitute for a compositor frame:
  it renders the window through Slint's own snapshot path and showed the current state,
  but it does not show what the client committed to the compositor.
- Making `screenshot` itself send frame callbacks and wait for the next commit:
  considered and not built.
  A client with nothing new to draw never commits,
  so every screenshot of an idle application would wait for the full timeout.

## Related: captures after a size change read the old-size back buffer

Found on 2026-10-05 while adding the `scale` control command,
which changes the framebuffer size while the hosted client keeps its logical size.

### Symptom

A frame recorded after a `scale` switch made during a recording had the new size
but not the client's drawing.
With a 640x360 Slint scene switched from scale 1 to 2 while `record` ran,
the last recorded 1280x720 frame had green 0 at all three probe points:

```text
AssertionError [ERR_ASSERTION]: frame recorded after a switch while recording: greens 0,0,0
```

The same switch with the IDE gave a 2200x1320 frame
whose lower-left 1100x660 region held one quarter of the scale-2 drawing
and whose other three quarters were empty.
A `screenshot` taken after the recording stopped was correct.

### Root cause

`read_frame` in `package/nested-wayland-session/cli/src/screenshot.rs` rendered into the nested window's
EGL surface and read it back.
Smithay resizes that surface when the window size changed since the last bind:

```rust
// smithay-0.7.0/src/backend/winit/mod.rs:286
        let window_size = self.window_size();
        if Some(window_size) != self.bind_size {
            self.egl_surface.resize(window_size.w, window_size.h, 0, 0);
        }
        self.bind_size = Some(window_size);
```

Mesa defers that resize while the surface has a back buffer that was drawn into since the last swap.
The resize callback leaves the size alone in that case:

```c
// mesa 26.2.2, src/egl/drivers/dri2/platform_wayland.c:462
   if (!dri2_surf->back) {
      dri2_surf->base.Width = wl_win->width;
      dri2_surf->base.Height = wl_win->height;
   }
```

and the next draw keeps the existing back buffer:

```c
// mesa 26.2.2, src/egl/drivers/dri2/platform_wayland.c:1564
   if (dri2_surf->back != NULL)
      return 0;

   return update_buffers(dri2_surf, flow);
```

Only a swap releases the back buffer.
While a recording runs nothing swaps:
`redraw` in `src/render.rs` returns early while a recorder exists,
and the recorder renders and reads back without `submit`.
So the first recorder tick drew into a back buffer,
the size change could not replace it,
and every later tick rendered the new,
larger output into the old,
smaller buffer and read back a region larger than the buffer.

The host's Mesa version is taken from the renderer's own log line,
`GL Version: "OpenGL ES 3.2 Mesa 26.2.2"`.

The same reasoning applies to a `screenshot` after a size change
while the parent presents nothing (a hidden or locked parent window),
because the fallback pacing timer sends frame callbacks but does not swap either.
That case follows from the sources quoted here and was not measured.

### Fix

`read_frame` renders into an offscreen texture sized from the output mode
(`capture_texture` in `src/state.rs`),
created on the first capture and recreated when the mode size changes.
The output mode is set in the same step as a resize or scale change,
so a capture has the screen's current size without any swap.

### Verification

`inspect:scale` in `package/nested-wayland-session/cli/mise.toml` starts a recording,
waits 300 ms so the recorder has drawn,
switches to scale 2,
and probes the last recorded frame.

Failing,
on the release binary that captured the window
(`~/temp/agent/nws-scale-nasfB8/inspect-scale-5-oldcapture.log`):
`greens 0,0,0`.
Without the 300 ms wait the same binary passed
(`inspect-scale-4-oldcapture.log`),
because the size change arrived before the recorder's first draw;
that wait is what makes the check able to fail.

Passing,
on the binary with the offscreen texture
(`inspect-scale-6.log`):

```text
switch while recording: last frame 1280x720, greens 192/64/64
```

The capture path produces the same bytes as before for unchanged sizes:
`inspect:color-scheme` and `inspect:stalled-parent` report the screenshot hashes
`cc28a666d87e` (dark) and `ccdb157ba532` (light) on both binaries.

### Tradeoffs

One GPU texture the size of the framebuffer stays allocated after the first capture,
and it is replaced on every size change.
A capture no longer reads what the parent window shows,
only what the compositor composites from the client's committed buffers,
which is the same content in every measured case.

### What does not work

- Calling `screenshot` again,
  or waiting,
  during the recording:
  no swap happens,
  so the back buffer stays.
- Taking the capture size from the window (`window_size()`) and the pixels from its surface:
  the two disagree until the next swap.

## Upstream filing decision

The tool is owned by this repository,
so there is no external tracker and no duplicate search applies.
`.out-of-scope/` has no entry for the nested compositor.
The related capture-size item is the same kind:
Smithay's `bind` and Mesa's deferred resize behave as their sources state,
and the compositor now captures off screen.

1.  Upstream's fault:
    the behavior was in this repository's compositor.
    winit's gating and Smithay's `submit` behave as their sources state.
2.  Fixable:
    yes,
    and fixed in commit `33d0dce2b`.
3.  Supported use case:
    yes;
    the README describes `screenshot` as capturing the hosted application's latest frame.
4.  Contribution welcome:
    not applicable to a repository-owned package.
5.  Likely to be fixed:
    not applicable.
6.  Prototype:
    the fix is committed with unit tests observed failing first
    and two end-to-end checks observed failing on the earlier binary.

Nothing is filed upstream and no issue draft is kept.
