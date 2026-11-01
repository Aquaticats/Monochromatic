# Slint 1.18.1 `Flickable` with default `mouse-drag-pan-enabled` holds a child `TouchArea` press for 100 ms and takes quick mouse drags as pans

## Symptom

A raw `Flickable` whose content is larger than the view,
with a `TouchArea` inside it that implements mouse drag selection,
does not behave like a text view:

- The press reaches the `TouchArea` late.
  In `package/ide/app`,
  removing `mouse-drag-pan-enabled: false` from the source `Flickable` in a disposable copy
  made the headless test `drag_selects_text_by_characters_and_by_words_without_panning`
  fail on its first check,
  immediately after the press was dispatched:
  `assertion left == right failed: coordinate control`,
  left `(0, 0)`,
  right `(19, 19)`.
  The caret had not moved,
  because the press had not been delivered yet.
- A quick drag pans the view instead of selecting.
  Commit `9e688cc5b` records this as the reason for the fix:
  "a quick drag on a scrollable document panned the view instead of selecting text".
  The guard run in "Verification" stopped at the press check,
  so its later assertions
  (`a quick drag of four lines must extend the selection, not be swallowed`
  and `a mouse drag panned the source view`)
  did not run against the unfixed markup.
  The panning half is established by the source in "Root cause",
  not by a measurement made for this document.

When the content fits in the view,
neither happens:
the `Flickable` passes the press through at once (`flickable.rs:710` to `715`).
The fixture uses 200 lines for that reason.

## Root cause

Code below is from the crates in the IDE build volume's cargo registry
(`~/.local/share/containers/storage/volumes/ide-cargo/_data/registry/src/index.crates.io-1949cf8c6b5b557f/`).

### Mouse panning is on by default for a raw `Flickable`

```rust
// i-slint-compiler-1.18.1/builtin_elements.rs:1500
        /// When false, the content can't be panned by the user, neither by dragging with the mouse
        /// nor with touch.
        in property <bool> interactive: true;
        /// When true, the content can be scrolled by clicking on it and dragging it with the cursor.
        /// Panning with a touch screen is only affected by `interactive`.
        in property <bool> mouse-drag-pan-enabled: true;
```

The standard `ScrollView` turns it off for its inner `Flickable`
(`widgets/fluent/scrollview.slint:178` to `179`,
and the same in the `material`,
`cupertino`,
`cosmic`,
and `qt` styles),
so the default bites only code that uses `Flickable` directly.

### The filter holds the press, then decides between child and pan

`Flickable::input_event_filter_before_children` first asks whether the event may pan at all:

```rust
// i-slint-core-1.18.1/items/flickable.rs:249
    fn accepts_pan_event(self: Pin<&Self>, event: &MouseEvent) -> bool {
        match event {
            MouseEvent::Wheel { .. } => true,
            MouseEvent::Pressed { .. } | MouseEvent::Moved { .. } | MouseEvent::Released { .. } => {
                self.interactive() && (event.is_from_touch() || self.mouse_drag_pan_enabled())
            }
```

and returns `ForwardAndIgnore` when it may not (`flickable.rs:156` to `158`).
With the default `true`,
`handle_mouse_filter` handles a left press on scrollable content by recording it and delaying it:

```rust
// i-slint-core-1.18.1/items/flickable.rs:717
                inner.velocity_rb = VelocityRingBuffer::default();
                inner.pressed_mouse_state = Some((crate::animations::current_tick(), *position));
                inner.last_mouse_position = *position;
                // ...
                if inner.capture_events.is_some() {
                    InputEventFilterResult::Intercept
                } else {
                    InputEventFilterResult::DelayForwarding(FORWARD_DELAY.as_millis() as _)
                }
```

A later move is taken from the child when it comes soon enough and far enough:

```rust
// i-slint-core-1.18.1/items/flickable.rs:739
            MouseEvent::Moved { position, .. } => {
                let do_intercept = inner.capture_events.is_some()
                    || inner.pressed_mouse_state.is_some_and(
                        |(pressed_time, pressed_mouse_position)| {
                            let mouse_delta = *position - pressed_mouse_position;

                            crate::animations::current_tick() - pressed_time <= DURATION_THRESHOLD
                                && self.should_capture_mouse_direction(mouse_delta, flick, flick_rc)
                        },
                    );
```

The constants are:

```rust
// i-slint-core-1.18.1/items/flickable.rs:381
/// The distance required before it starts flicking if there is another item intercepting the mouse.
pub(super) const DISTANCE_THRESHOLD: LogicalLength = LogicalLength::new(8 as _);
/// Time required before we stop caring about child event if the mouse hasn't been moved
pub(super) const DURATION_THRESHOLD: Duration = Duration::from_millis(500);
/// The delay to which press are forwarded to the inner item
pub(super) const FORWARD_DELAY: Duration = Duration::from_millis(100);
```

`should_capture_mouse_direction` (`flickable.rs:797` to `816`) is true when the pointer has moved
more than 8 logical pixels along an axis in which the content can scroll.
So a drag that covers more than 8 px within 500 ms of the press pans the content;
the element documentation describes the same algorithm (`builtin_elements.rs:1556` to `1577`),
including that the child then receives an exit event.
A text selection drag is usually exactly that kind of gesture.

The measured `(0, 0)` matches the press half:
the test dispatches the press and reads the caret without letting 100 ms pass,
so `DelayForwarding` has not yet delivered the press to the `TouchArea`.
This attribution is read from the code;
the guard run did not log the filter result.

## Verification

Versions under test:
Slint `1.18.1` (`i-slint-core` checksum `ed2ea15059d70bc31b1a8b1eb4c34f2c5df04cb7e53f36179b43c0261231d646`,
`i-slint-compiler` checksum `f55d48ebef05e2bf09ba5a1fc67bcb11033b9e12a2a50ed3105c7d25e2b539d0`,
from `package/ide/app/Cargo.lock`).

Harness:
`package/ide/app/bin/inspect-source-guards.mjs`
(now `package/ide/app/bin/inspect-source-guards.ts`,
renamed 2026-10-22),
guard `mouse-drag-selects`,
which deletes the `mouse-drag-pan-enabled: false;` line from `ui/app.slint` in a disposable copy
and runs the named test with real window pointer events through the headless backend.
The run recorded here used the tree committed as `374ea9a36`;
its logs are in `~/temp/agent/ide-source-guard-Qu70pZ/`.

```sh
# package/ide/app/mise.toml task inspect:source-guards; arguments: cache, cargo home copy, guard names
mise run //package/ide/app:inspect:source-guards "${HOME}/temp/agent/<disposable-target>" "${HOME}/temp/agent/<cargo-home-copy>" mouse-drag-selects
```

### Works cleanly

- With `mouse-drag-pan-enabled: false`,
  `mouse-drag-selects-baseline.log` and `mouse-drag-selects-restored.log` pass the whole test:
  the press places the caret at once,
  a four-line drag extends the selection,
  `scroll-y` stays `0`,
  and word drags after a double click extend by words.
- Wheel events still scroll,
  because `accepts_pan_event` returns `true` for `Wheel` whatever the property says (`flickable.rs:251`).
  This is read from the source;
  no wheel test was run for this document.
- Touch drags still pan,
  because `is_from_touch()` bypasses the property (`flickable.rs:253`).
  Also source-derived;
  the IDE has no touch test.

### Fails

- Default `mouse-drag-pan-enabled`,
  scrollable content:
  `mouse-drag-selects-removed.log`,
  `src/native/pointer_tests.rs:214`,
  `coordinate control`,
  left `(0, 0)`,
  right `(19, 19)`.

## Verified workarounds

Disable mouse panning on the `Flickable` that hosts the selectable content
(`package/ide/app/ui/app.slint:257` to `266` on `main` at `f9d08ae79`):

```slint
// package/ide/app/ui/app.slint:257
                scroll := Flickable {
                    width: parent.width;
                    height: parent.height;
                    content-width: max(self.width, root.document-width * 1px + root.gutter-width);
                    content-height: max(self.height, root.total-lines * root.line-height);
                    content-y <=> root.scroll-y;
                    content-x <=> root.scroll-x;
                    // A mouse drag selects text. Panning by mouse drag would take a quick drag away from selection;
                    // wheel and touch scrolling are unaffected.
                    mouse-drag-pan-enabled: false;
```

Tradeoffs:

- A mouse can no longer pan the view by dragging.
  The application has to scroll during a selection drag that leaves the view itself;
  the IDE does that in native code (`package/ide/app/README.md`,
  "Pointer selection").
- The 100 ms press delay disappears for mouse input,
  because the filter returns `ForwardAndIgnore` before the delay path.
  That is also the mouse half of open issue
  [slint-ui/slint#9521](https://github.com/slint-ui/slint/issues/9521)
  ("Flickable (and ScrollView):
  Add a property to prevent delaying press events").
- Touch input still pans and still gets the delay,
  so a touch drag cannot select text.
  Touch selection is outside the IDE's scope.

## What does not work

No other approach was tried.
`interactive: false` would also stop the pan,
but it is documented to stop touch panning too (`builtin_elements.rs:1500` to `1502`),
so it removes more than the conflict requires.
Wrapping the content in the standard `ScrollView` would inherit its `mouse-drag-pan-enabled: false`,
but it also brings that style's scroll bars and layout,
which the source view does not use.

## Upstream filing decision

`.out-of-scope/` was inspected.
Its only Slint mention (`cargo-workspace.md:18`) concerns container builds,
not upstream tracking,
so no exemption applies.

Nothing is filed.

1.  Upstream's fault:
    no.
    The default and the press-delay and drag-capture algorithm are documented on the element
    (`builtin_elements.rs:1503` to `1505` and `1556` to `1577`),
    and the standard `ScrollView` already opts out.
2.  Fixable:
    not applicable once constraint 1 fails.
3.  Supported use case:
    yes;
    the property exists for this.
4.  Contribution welcome:
    not evaluated further,
    because there is nothing to contribute.
5.  Likely to be fixed:
    not applicable.
6.  Prototype:
    not applicable.

Searches on 2026-10-05:
`gh search issues --repo slint-ui/slint mouse-drag-pan-enabled`,
`gh search issues --repo slint-ui/slint Flickable TouchArea drag`,
and the same terms with `gh search prs`.
Related threads,
none reporting this as a defect:
[slint-ui/slint#4352](https://github.com/slint-ui/slint/issues/4352)
(closed,
"Distinguish between touch and mouse to pan the Flickable"),
[slint-ui/slint#9521](https://github.com/slint-ui/slint/issues/9521)
(open,
the press delay),
and [slint-ui/slint#13117](https://github.com/slint-ui/slint/pull/13117)
(merged 2026-09-01,
"Flickable:
forward a press to elements underneath when it can't pan",
the `can_pan` check that limits this to scrollable content;
`gh api repos/slint-ui/slint/compare/1f72ad442...v1.18.1` reports `ahead`,
so the tag contains it).
