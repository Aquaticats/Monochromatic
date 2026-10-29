# Slint 1.16.1 smooth-scrolls touchpad gestures but not mouse-wheel events; Slint 1.17.0 fixes the wheel path

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Slint 1.16.1's `Flickable` has momentum (fluent) scrolling,
 but only for
touchpad and touchscreen gestures.
 Discrete mouse-wheel events jump
instantly with no animation.
 The mouse-wheel animation exists in Slint 1.17.0
(slint-ui/slint#11338,
 merged 2026-04-15,
 fixing #11312).
 It diverged from the 1.16 release line,
 so every published 1.16 release lacks it.
 `music-player` now uses the Slint 1.17.0 crates.
io release instead of the former master-revision pin.

## Symptom

Scrolling the queue list:

- With a touchpad or touchscreen:
   the list glides and keeps moving after the
  fingers lift (momentum),
   then decelerates.
   This is present in 1.16.1.
- With a mouse wheel:
   each wheel notch moves the list by a fixed step
  instantly,
   with no easing or momentum.
   The list snaps to the new position
  on every notch.
   This is the behaviour 1.16.1 ships.

There is no `smooth-scroll` property to set.
 Searching the 1.16.1 source for
`smooth-scroll` / `smooth_scroll` returns nothing,
 and the `Flickable`
builtin exposes only `viewport-*`,
 `interactive`,
 and the `flicked` callback
(`internal/compiler/builtins.slint:181-189`).

## Root cause

Momentum scrolling lives in the `Flickable` item's native Rust code,
 not in
markup.
 It is automatic,
 with no opt-in property.

The animation is driven by `FlickableDataInner::animate`,
 which runs a
constant-deceleration physics simulation on `viewport_y`
(`internal/core/items/flickable.rs:490-532` in tag `v1.16.1`):

```rust
let animation_y = physics_simulation::ConstantDecelerationParameters::new(
    dist.y as f32 / (millis as f32 / 1000.),
    DECELERATION,
);
viewport_y.set_physic_animation_value(limit.y_length(), animation_y);
```

`animate` is called only from `TouchPhase::Ended`,
 and it requires a recent
move sample with non-zero velocity within `MAX_DURATION` (100ms).
 Touchpads
and touchscreens emit a `Started` / `Moved` / `Ended` phase sequence carrying
that velocity,
 so they get the fling.
 A discrete mouse wheel does not produce
that phase sequence with velocity,
 so in 1.16.1 the `TouchPhase::Moved` arm
sets the viewport directly with no animation
(`internal/core/items/flickable.rs:459-470`,
 tag `v1.16.1`):

```rust
TouchPhase::Moved => {
    // ...
    self.position_time_rb.push(crate::animations::current_tick(), new_pos);
    viewport_x.set(new_pos.x_length());
    viewport_y.set(new_pos.y_length());
    self.last_scroll_event = Some((crate::animations::current_tick(), position));
}
```

The mouse-wheel animation was added later in slint-ui/slint#11338
("fix:
 animate wheel scrolling",
 merged 2026-04-15,
 closes issue #11312
"Flickable:
 Implement scroll animation also for mouse wheel events").
 It
animates the wheel case with a fixed 180ms cubic-bezier easing when there is
no fling velocity,
 using the property's native animated set
(diff of `internal/core/items/flickable.rs` in that PR):

```rust
const WHEEL_SCROLL_DURATION: i32 = 180;
const WHEEL_SCROLL_EASING: EasingCurve = EasingCurve::CubicBezier([0.0, 0.0, 0.58, 1.0]);
// ...
let animation = Self::wheel_scroll_animation();
viewport_x.set_animated_value(new_pos.x_length(), animation.clone());
viewport_y.set_animated_value(new_pos.y_length(), animation);
```

That commit (`ce79e9393`) is not an ancestor of `v1.16.1`.
 The GitHub compare
`v1.16.1...ce79e9393` reports `status: diverged, ahead_by: 1, behind_by: 55`:
the 1.16 release line branched before the wheel fix,
 so the fix is on master
only.
 Cargo reports the master crates as version `1.17.0`,
 so the fix lands in
the 1.17 release.
 Grepping the `v1.16.1` clone for the PR's markers
(`WHEEL_SCROLL_DURATION`,
 `wheel_scroll_animation`,
 `set_animated_value` in
`flickable.rs`) returns nothing,
 confirming its absence.

### Why the custom scrollbar still tracks the animation

`music-player` binds its custom scrollbar to the viewport with a two-way
binding (`value <=> flick.viewport-y` in `ui/app.slint`).
 The native physics
animation (1.16.1) and the native wheel animation (#11338) both install their
animated binding via `Property::set_binding`,
 which honours the existing
binding's `intercept_set_binding`
(`internal/core/properties.rs:642-652`).
 A two-way binding's
`intercept_set_binding` redirects the incoming binding onto the shared
`common_property`
(`internal/core/properties/two_way_binding.rs:26-29`),
 so the animation lands
on the common property that both `viewport-y` and the scrollbar `value` read.
The thumb therefore animates in lockstep with the content;
 the two-way
binding does not block smooth scrolling.

## Verification

Version under test:
 Slint `v1.16.1`,
 clone grafted at commit
`e9c1ca295f9356af71f1e251c287de18406b46f6` (tag `v1.16.1`),
 at
`/tmp/agent/slint-1161-wrap`.
 Upstream metadata read with `gh api` against
`slint-ui/slint` on 2026-06-01.

What works in 1.16.1:

- Touchpad / touchscreen momentum scrolling on any `Flickable` (and any
  `ScrollView` / `ListView`,
   which wrap a `Flickable`).
   No property needed.

What does not animate in 1.16.1:

- Mouse-wheel scrolling.
   Each notch is an instant step.

Upstream state of the wheel fix:

- slint-ui/slint#11338 merged to master 2026-04-15 (merge commit
  `ce79e9393`).
- Slint `v1.17.0` released 2026-06-24 and includes the changelog entry
  `Flickable: Animate wheel scrolling. (#11312)`.
- GitHub compare from the former pin
  `85e3eb76819762cdcaa732fa87533ff896546bac` to `v1.17.0` reports
  `ahead_by: 315` and `behind_by: 0`,
   so the release contains the former pinned fix.

## Verified workaround

Use the Slint 1.17.0 crates.
io release.
 `music-player` and `terminal` depend on
`slint`,
 `i-slint-backend-winit`,
 and `slint-build` with a `1.17.0` version
requirement.
 The release still requires rustc 1.92 or newer,
 above Fedora 41's
packaged 1.91.1,
 so the `music-player` `Containerfile` keeps installing Rust with
rustup instead of the distro `rust` package.
 The experimental builtins flag the app already sets for `FlexboxLayout`
(`SLINT_ENABLE_EXPERIMENTAL_FEATURES=1` in the `Containerfile`) is unrelated to
scrolling and continues to apply.

Verification after switching from the git pin to crates.
io Slint 1.17.0 and
renaming `FlexboxLayout.align-items` to `cross-axis-alignment`:

- `mise run //package/music-player/desktop-app:lint` passes.
- `mise run //package/terminal/app:lint` passes.
- `mise run //package/music-player/desktop-app:test` passes,
   `78` tests.
- `mise run //package/terminal/app:test` passes,
   `16` tests.

Trade-offs:

- Crates.
  io release artifacts replace the former git source build for Slint,
  which removes the need for a moving master-revision rationale.
- The Slint direct dependencies remain versioned together,
  because the runtime,
  explicit winit backend,
  and markup compiler are released as one Slint family.

## What does not work

- A `smooth-scroll` property:
   none exists in 1.16.1.
   The PR that proposed an
  opt-out `smooth-scroll` property on `Flickable` and `ScrollView`
  (slint-ui/slint#9440) was closed unmerged;
   the maintainer closed it with
  "We now have smooth scrolling" once the property-less physics path landed.
- A markup `animate viewport-y { ... }` on the `Flickable`:
   it does not smooth
  the mouse wheel.
   A markup `animate` only wraps assignments made through the
  generated component accessor (`internal/compiler/generator/rust.rs:2306-2311`
  emits `set_animated_value` at the generated set site).
   The `Flickable` sets
  `viewport-y` from its own native Rust handler,
   bypassing the generated
  accessor,
   so the markup animation never sees the wheel-driven change.
- Relying on the std `ScrollView` instead of the custom Flickable:
   the fluent
  `ScrollView` adds no scroll animation of its own
  (`internal/compiler/widgets/fluent/scrollview.slint`);
   it inherits exactly
  the same `Flickable` behaviour,
   so it is no smoother than the custom
  scrollbar on a mouse wheel.

## Why we do not file this upstream

Per the troubleshooting-doc 5-constraint check,
 this would be filed only if
all five held.
 They do not,
 because the fix already exists upstream:

1. Upstream's behaviour?
    Yes,
    the missing mouse-wheel animation was upstream
   behaviour,
    now intentionally changed.
2. Can upstream fix it?
    Already fixed in #11338.
3. Supporting the use case?
    Yes;
    issue #11312 requested it and was accepted
   and closed by the fix.
4. Will they fix it?
    Already merged and released in Slint 1.17.0.
5. Prototyped a minimal fix?
    Not applicable;
    upstream's own merged change is
   the fix.

There is nothing to report:
 upstream already shipped the fix in Slint 1.17.0.
 The local action is to use the crates.
io release,
 not a new issue.
