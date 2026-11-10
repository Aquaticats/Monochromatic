# Slint 1.17 Slider: a one-way `value:` binding is destroyed the first time the user touches the slider, freezing the seek bar

Tool under test:
 Slint 1.17.0 (`slint`,
 `i-slint-backend-winit`,
 `slint-build` all pinned to `1.17.0` in
`package/music-player/desktop-app/Cargo.toml`).
 Surface trigger:
 a std-widgets `Slider` whose `value` is set with
a one-way binding (`value: root.position`) while the same property is also driven from Rust.
 Failure mode:
 after any
click,
 drag,
 or arrow-key on that slider,
 later writes to the bound property no longer move the thumb.

## Symptom

In the music player desktop app,
 the top seek bar sometimes "sticks to the left" while audio keeps playing and is
clearly a few minutes in.
 The elapsed position genuinely advances inside the engine (you can hear it,
 and the engine
keeps emitting `Update::Position`),
 but the slider thumb is frozen at wherever it last sat.

It is intermittent because it only happens after the user interacts with the seek bar in that session.
 An untouched
seek bar tracks playback correctly for the whole track.
 It tends to freeze "at the left" because the common triggers
land the thumb near zero:
 a click near the start of the bar,
 a seek back toward the beginning,
 or the keyboard
`Home` / `Left` path inside the widget,
 which calls `set-value(minimum)` = 0.

The volume slider has the identical latent defect.
 It is just far less noticeable because volume is not driven
continuously from the engine,
 so a frozen volume slider is only visible after an external or restored volume change.

## Root cause

Slint properties carry a binding.
 Assigning a value to the property imperatively (from Rust,
 from a callback,
 or
from a widget reacting to user input) replaces that binding with a constant.
 A one-way binding therefore does not
survive the first imperative write.
 Two-way bindings (`<=>`) alias the two storage slots together and do survive.

The consuming code used one-way bindings.
 Before the fix,
`package/music-player/desktop-app/ui/app.slint` had:

```slint
// seek bar
Slider {
    maximum: root.duration > 0 ? root.duration : 1.0;
    value: root.position;          // one-way binding: position -> slider.value
    changed(value) => { root.seek(value); }
}
// volume
Slider {
    maximum: 1.0;
    value: root.volume;            // one-way binding: volume -> slider.value
    changed(value) => { root.set-volume(value); }
}
```

The engine drives `position` from the event-loop thread on every position tick:

```rust
// package/music-player/desktop-app/src/main.rs:787 (apply_update, Update::Position arm)
app.set_position(*secs as f32);
```

`app.set_position(...)` writes the `position` property,
 and the one-way binding forwards it into the Slider's
`value`.
 That path works only until the widget writes `value` itself.

The std-widgets `Slider` writes `value` imperatively on every user interaction.
 `Slider.value` is aliased to
`SliderBase.value`,
 and `SliderBase` only ever changes `value` through `set-value()`,
 which is an assignment:

```slint
// i-slint-compiler 1.17.0, widgets/common/slider-base.slint (SliderBase)
public function set-value(value: float) {
    if (root.value == value) { return; }
    root.value = max(root.minimum, min(root.maximum, value));  // imperative assignment -> replaces the incoming binding
    root.changed(root.value);
}
```

`set-value()` is called from the touch area (click and drag) and the focus scope (Left/Right/Up/Down/Home/End):

```slint
// widgets/common/slider-base.slint (excerpts)
// click / drag:
root.set-value((!root.vertical ? root.size-to-value(touch-area.mouse-x, root.width) : ...) + root.minimum);
// keyboard:
} else if (event.text == Key.Home) { root.set-value(root.minimum); return accept; }
```

So the first click,
 drag,
 or arrow-key runs `root.value = ...`,
 which replaces the app's `value: root.position`
one-way binding with a constant.
 From then on `app.set_position(...)` still updates `root.position`,
 but the
slider's `value` is no longer linked to it,
 so the thumb stops moving.
 Audio continues because the engine's position
accounting is entirely independent of the UI property.

This is documented,
 intended Slint behavior,
 not a defect in our timing code.
 A Slint maintainer states it directly
on the upstream issue (see "Upstream filing decision"):
 "Once you edit the value in the LineEdit,
 that binding gets
replaced by a binding to a constant value with the new string."
 The Slint reactivity guide says the same:
"If a property's value is later changed using an imperative assignment in code,
 the original binding is broken.
 This
applies to user interactions as well."

### Hypotheses that were wrong

Two earlier theories were investigated and disproved before the binding-break cause was found.
 Recording them so the
next investigator does not re-derive them:

1.  "The `changed(value) => root.seek(value)` handler feeds engine position updates back into a seek,
     looping."
    Disproved by reading `SliderBase`:
     `changed` is emitted only from `set-value()`,
     which runs only on user input.
    A programmatic write through the `value: root.position` binding does not call `set-value`,
     so engine-driven
    position writes never fire `changed` and never trigger a seek.
     No feedback loop exists.
2.  "The progress debouncer in `src/progress.rs` suppresses position updates."
     Disproved:
     its window is 250 ms
    (`PROGRESS_UPDATE_DEBOUNCE_INTERVAL`),
     engine position emits arrive about every 100 ms
    (`POSITION_EMIT_INTERVAL_SECS = 0.1` in `src/controller_audio.rs`),
     and the engine emits no periodic `Immediate`
    update to keep resetting the debounce baseline (`Playing` is sent only on real state changes;
     the run loop in
    `src/engine.rs` just pumps audio).
     The debounce self-corrects within 250 ms and cannot freeze the bar for
    minutes.

## Verification

Version under test:
 `slint` 1.17.0 from crates.io (`Cargo.lock`:
`name = "slint"`,
 `version = "1.17.0"`,
 `source = "registry+https://github.com/rust-lang/crates.io-index"`).

Compile check that the two-way fix builds:

```sh
mise run //package/music-player/desktop-app:lint   # cargo check, runs the Slint compiler over app.slint
# => Finished `dev` profile ... in ~9s (no errors)
```

Behavioral reproduction with the real GUI (a real Wayland session with XWayland was available):

```sh
# 200s tone so the thumb has room to travel
ffmpeg -y -f lavfi -i "sine=frequency=220:duration=200" -ac 2 -ar 44100 /tmp/music/longtone.wav

mise run //package/music-player/desktop-app:build:debug
# run under XWayland so xdotool/import can drive it
env -u WAYLAND_DISPLAY DISPLAY=:0 ./target/debug/music-player --start-playing /tmp/music/longtone.wav
```

Manual steps that surface the difference:

- Works cleanly (before and after fix):
   launch,
   do not touch the seek bar,
   watch the thumb advance with playback.
- Fails before the fix / works after the fix:
   while playing,
   click once anywhere on the seek bar (or drag it,
   or
  focus it and press `Home`),
   then keep watching.
   Before the fix the thumb freezes at the click point while audio
  continues.
   After the fix the thumb keeps advancing with playback.

The fix was confirmed working in the running GUI (the two-way binding keeps the thumb tracking playback after a
seek-bar click).

## Verified workarounds

Two-way binding (applied).
 Change the one-way `value:` binding to `value <=>` and promote the driven property to
`in-out` so the alias is legal in both directions:

```slint
// package/music-player/desktop-app/ui/app.slint:225, :285
in-out property <float> position: 0;   // was: in property <float> position: 0;
// ...
value <=> root.position;               // was: value: root.position;
```

Same change for volume (`app.slint:233`,
 `:305`):
 `in-out property <float> volume` and `value <=> root.volume`.

Why it works:
 the `<=>` alias merges the slider's `value` storage with `root.position`,
 so neither side's writes
destroy the other's link.
 Engine writes to `position` move the thumb;
 user drags update `position` and still fire
`changed(value) => root.seek(value)` (because `changed` fires from `set-value`,
 which user input still triggers).

Tradeoff:
 with `<=>`,
 during an active drag both the engine's position ticks and the drag write `root.position`,
 so
the thumb can briefly fight the finger until the `changed -> seek` round trip converges the engine onto the dragged
value.
 It converges within a tick or two and is not visible in normal use.
 If a perfectly smooth drag is required,
the refinement is to seek only on `released(value)` and/or suppress engine position writes while the slider reports
`handle-pressed`.
 That refinement was not needed to fix the reported freeze and was left out to keep the change
minimal.

## What does not work

- Leaving `value: root.position` one-way and relying on the engine to "re-push" position.
   The engine already
  re-pushes about ten times a second;
   it does not help,
   because the binding itself is gone after the first
  `set-value`,
   so no amount of writing `root.position` reaches the slider's now-constant `value`.
- The maintainer's alternative,
   keeping the one-way binding and adding `changed position => { slider.value = self.position; }`
  on the root (re-asserting the value whenever position changes).
   It works but is strictly more code and more
  fragile than `<=>`:
   it needs an `id` on the slider,
   re-asserts on every tick,
   and still leaves `value` as a
  constant binding that any user input immediately clobbers again before the next `changed` fires.
   The two-way
  binding removes the failure mode entirely instead of racing it.
- Tuning the progress debouncer (`src/progress.rs`).
   It is not involved;
   see "Hypotheses that were wrong".

## Upstream filing decision

Not filing.
 This is documented,
 intended Slint behavior with a maintainer-endorsed workaround,
 and an existing
upstream issue already captures it.

`.out-of-scope/` was checked for a Slint or binding-semantics exemption:
 none matched
(`.out-of-scope/` holds `bun-install`,
 `cargo-workspace`,
 `claude-code-upstream-bugs`,
 `codex-harness`,
 `jsr`,
`lightningcss`,
 `low-impact-typescript-formatting`,
 `module-es-monolith`,
 `pi-gpt55-long-context`,
`terminal-title-fork-parity-tests`,
 `typescript-project-references`).
 So the upstream check proceeds normally.

Duplicate search:
 [slint-ui/slint#8102 "Binding gets broken after user input"](https://github.com/slint-ui/slint/issues/8102),
CLOSED as COMPLETED on 2025-04-11.
 A Slint contributor (`hunger`) explains the exact mechanism and recommends both
the `changed`-reassign approach and the `<=>` two-way binding;
 the reporter confirms `<=>` solved it.
 That thread
already contains a sharper root-cause statement than we could add,
 the same reproduction class,
 the same fix,
 and
affected versions bracketing ours.
 There is nothing additive to contribute,
 so no comment is posted.

Six-constraint check (for the audit trail):

1.  Really upstream's fault?
     No. It is intended reactivity semantics (imperative assignment replaces a binding),
    documented in the Slint reactivity guide and restated by a maintainer on #8102.
2.  Can upstream fix it?
     Not applicable as a bug;
     there is nothing to fix.
     The behavior is the design,
     and the
    supported path (two-way binding) already exists.
3.  Supporting this use case?
     Yes.
     Two-way bindings are the documented,
     first-class mechanism for exactly this
    bidirectional widget-value case.
4.  Would the repo welcome our contribution?
     Not reached;
     there is no defect to contribute.
5.  Will they likely fix it?
     No,
     and deliberately:
     they closed #8102 as working-as-intended with a workaround.
6.  Minimal fix prototyped compatible with their architecture?
     Not applicable;
     the fix is entirely consumer-side
    (`<=>` in our `app.slint`),
     already applied and verified above.
     No upstream patch is warranted.

Constraints 1,
 2,
 and 5 fail on "this is by design,"
 so the gate correctly resolves to do not file,
 and there is no
new-issue draft to keep.
