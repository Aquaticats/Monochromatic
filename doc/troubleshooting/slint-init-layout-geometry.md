# Slint 1.17.0 init geometry precedes final FlexboxLayout placement and can stale derived row state

Tool:
Slint 1.17.0 from crates.io.
The `slint` checksum is
`a54a407d1a0cbaa71d830ae7c236064b171368ea18db3b51ea3f2ce3f19519ee`.
The `i-slint-compiler` checksum is
`290c2247e87d3653b9b7c3bc1cbe19647b2f93a4b99a40aae6a7140717700d37`.
The `i-slint-core` checksum is
`1b7d306bc9d6fe1d1fd5e775837ec1fa1e41d1c8063789181b7de631bd5db41c`.
The matching upstream tag is `v1.17.0` at
`fdde7a535305d2ab2d4072dee637bad186a49723`.

Note 2026-11-09T16:56:27-05:00:
these citations in this record name paths that no longer exist.
`package/music-player/desktop-app/src/ui_binding_tests.rs` left the tree on 2026-10-11 in `f36cef6b43`.
`package/music-player/desktop-app/src/ui_led_rows.rs` left the tree on 2026-10-11 in `f36cef6b43`.

## Symptom

The desktop music player's LED page controls use repeated Slint components inside a wrapping `FlexboxLayout`.
Each cap reports its measured rectangle to Rust.
Rust combines those rectangles into one SVG path behind every wrapped row.

The original reporters watched `changed x` and `changed y` together with `changed width` and `changed height`.
The caps appeared but `LedPlateGeometry.path` remained empty on the first visible layout.
Adding an `init` reporter made the path nonempty,
but the first reports still described one unwrapped row.
The rendered caps had already wrapped to two rows,
so the plate and cap geometry disagreed.

The instrumented failing fixture reported every cap at `y=0`:

```text
begin count=6 width=758
index=0 x=0   y=0 width=107
index=1 x=99  y=0 width=98
index=2 x=189 y=0 width=158
index=3 x=339 y=0 width=144
index=4 x=475 y=0 width=170
index=5 x=637 y=0 width=97
```

The testing backend exposed the cap positions after `show()` as:

```text
(12,232) (111,232) (201,232)
(12,284) (148,284) (310,284)
```

The stale report therefore produced one row start while the visible fixture had two rows.

The final product design no longer derives plate extent from row reports.
One rounded backplate always fills the available width and combined row height.
Deferred geometry remains only for first and last cap corner ownership within each measured row.

## Root cause

This was a consumer lifecycle error,
not proof of a Slint layout defect.
An `init` callback means that the component and its property bindings exist.
It is not a post-layout notification.

Slint's Rust generator initializes property bindings first,
then places Slint `init` expressions and change trackers in generated `user_init` code.
`internal/compiler/generator/rust.rs:1491-1536` in `v1.17.0` shows both steps:

```rust
// Initialize all properties which have an initial value in the slint file
// This sets up also the callback handler and bindings
for (prop, expression) in &component.property_init {
    handle_property_init(prop, expression, &mut init, &ctx)
}

user_init_code.extend(component.init_code.iter().map(|e| {
    let code = compile_expression(&e.borrow(), &ctx);
    quote!(#code;)
}));

user_init_code.extend(component.change_callbacks.iter().enumerate().map(|(idx, (p, e))| {
```

The runtime's eager tree pass materializes repeaters and runs change handlers,
but this pass is still an instantiation boundary.
`internal/core/window.rs:641-668` says and implements that exact scope:

```rust
/// Walk the component tree and every active popup to materialize every
/// Repeater,
/// Conditional and ComponentContainer.
///  Runs change handlers
/// and the instantiation pass in a loop because init callbacks may set
/// properties that trigger change handlers,
/// and change handlers may
/// make new conditionals/repeaters dirty.
pub fn ensure_tree_instantiated(&self) {
    for _ in 0..10 {
        let mut changed = false;
        if let Some(component) = self.try_component() {
            changed |= crate::item_tree::ensure_item_tree_instantiated(&component);
        }
        changed |= crate::properties::ChangeTracker::run_change_handlers_once();
```

Reading `absolute-position` does not create a separate layout-complete callback.
The compiler lowers it to a binding over parent position plus the element's `x` and `y`.
`internal/compiler/passes/lower_absolute_coordinates.rs:33-68` shows that binding:

```rust
// Create a binding for the `absolute-position` property. The
// materialize properties pass is going to create the actual property later.
let binding = Expression::CodeBlock(vec![
    Expression::StoreLocalVariable {
        name: "parent_position".into(),
        value: Expression::FunctionCall {
            function: BuiltinFunction::ItemAbsolutePosition.into(),
            arguments: vec![Expression::ElementReference(Rc::downgrade(&elem))],
            source_location: None,
        }
        .into(),
    },
```

The initial report therefore sampled geometry before the window's final width had driven wrapping.
A later explicit report tick sampled each cap's current `absolute-position`,
normalized it against the plate component's `absolute-position`,
and replaced the stale row membership.

A nearby upstream issue is not the current cause.
[Slint issue #7402](https://github.com/slint-ui/slint/issues/7402) concerned recursion or a compiler crash when
`init` queried geometry.
Its fix,
[PR #11397](https://github.com/slint-ui/slint/pull/11397),
is present in Slint 1.17.0 as the eager `ensure_tree_instantiated` pass quoted here.
The music-player fixture neither recursed nor crashed.
It observed valid geometry at the wrong lifecycle boundary.

## Verification

The executable regression is
`package/music-player/desktop-app/src/ui_binding_tests.rs:150-214`.
That file is gone:
commit `f36cef6b43` (2026-10-11) replaced the page-tab window this regression drove,
and `ui_binding_tests.rs` went with it.
It creates the generated `AppWindow`,
installs the Rust row-membership adapter,
selects LED controls,
loads six labels,
shows the window,
and confirms both wrapped-row membership and full-width plate size before and after resize.

`package/music-player/desktop-app/src/ui_binding_tests.rs` was removed on 2026-10-11 in `f36cef6b43`
(feat(music-player-desktop-app):
replace the page-tab window with the accepted design (cover and
two-pane layouts,
Search,
Settings,
template editor,
menu,
overlays,
keyboard));
no file in the
working tree carries it now.

Run it through the package task:

```bash
mise run //package/music-player/desktop-app:test
```

Before the deferred report tick,
the test failed with:

```text
fixture must wrap and report measured row starts; starts=1,
caps=[(12,232), (111,232), (201,232), (12,284), (148,284), (310,284)]
```

After the final full-width redesign,
the same task ran 86 tests and reported 86 passed.

The testing backend's mock-time helper is a faithful seam for this consumer timer.
`internal/backends/testing/testing_backend.rs:42-58` advances animations,
fires timers,
runs change handlers,
and instantiates pending dynamic children:

```rust
pub fn mock_elapsed_time(time_in_ms: u64) {
    let tick = i_slint_core::animations::CURRENT_ANIMATION_DRIVER.with(|driver| {
        let mut tick = driver.current_tick();
        tick += core::time::Duration::from_millis(time_in_ms);
        driver.update_animations(tick);
        tick
    });
    i_slint_core::timers::TimerList::maybe_activate_timers(tick);
    i_slint_core::properties::ChangeTracker::run_change_handlers();
    ensure_all_tracked_trees_instantiated();
}
```

### Patterns that work

- Fixed geometry read after the component has reached its final width.
- Repeated cap geometry reported by a finite post-instantiation timer burst.
- Absolute cap positions normalized against the control-group origin.
- Plate width bound directly to complete available control width.
- A generation token plus measured row membership,
  so stale reports cannot mix with a newer label model.

### Patterns that fail

- Depending only on `changed x` and `changed y` as a post-layout barrier.
- Treating `init` as proof that a wrapping layout has its final width and row positions.
- Reporting child-local `x` and `y` before the containing window settles.
- Predicting row ends from the next label instead of measuring cap positions.

## Verified workaround

`package/music-player/desktop-app/ui/app.slint` centralizes each cap report.
It sends the cap's absolute position minus the control-group origin.
This preserves local row coordinates even when the control group itself starts away from `(0,0)`.

`package/music-player/desktop-app/ui/app.slint:460-488` adds one stopped `Timer` to the parent control group.
Every geometry generation resets and starts a finite report burst.
The timer reports after `1ms`,
then twice more at `16ms` intervals.
Each pass increments one layout-tick property,
and every repeated cap reports its current measured rectangle.
The timer stops after the third pass,
so it does not keep the render loop active.

`package/music-player/desktop-app/src/ui_led_rows.rs` groups complete reports and publishes only first and last row
membership.
That adapter file was removed by the same `f36cef6b43` (2026-10-11) redesign.
The Slint plate does not wait for this adapter:
it is a rounded rectangle whose width and height bind directly to the complete LED control bounds.
This removes plate flicker and stale path geometry while retaining measured 9-unit exposed cap corners.

Slint's Timer API supports explicit stopped state plus start and restart.
`internal/compiler/builtins.slint:2695-2739` defines the relevant surface:

```slint
export component Timer {
    in property <duration> interval;
    in property <bool> running: true;
    callback triggered;
    function start() {
    }
    function stop() {
    }
    function restart() {
    }
}
```

Tradeoffs:

- Geometry correction starts after `1ms` and finishes its bounded settling burst after two `16ms` intervals.
- Each geometry generation can invoke three reports per cap.
- Plate paint has no geometry-report delay because full-width bounds are direct Slint bindings.
- The workaround remains consumer-owned and does not expose a general Slint post-layout event.

## What does not work

- **Only adding `init` reports.**
  This changes an empty path into a path built from pre-wrap coordinates.
- **Changing reports from local position to `absolute-position` without deferral.**
  Coordinate space becomes correct,
  but the lifecycle sample is still early.
- **Watching `changed absolute-position` alone.**
  In the headless first-layout harness,
  it did not provide a later complete report after the visible caps had wrapped.
- **Keeping a repeating timer active indefinitely.**
  It would eventually correct geometry,
  but Slint's Timer documentation warns that an always-running timer causes constant CPU and power usage.
  The bounded three-pass burst has no persistent timer cost.
- **Applying the fix for issue #7402 again.**
  Slint 1.17.0 already contains PR #11397,
  and the observed failure is stale geometry rather than recursion.

## Upstream filing decision

No `.out-of-scope/` entry covers Slint geometry or layout callbacks.
Searches across open and closed Slint issues and pull requests used
`init callback layout geometry`,
`absolute-position changed callback`,
and broader `absolute-position` terms.
Issue #7402 and PR #11397 were the only directly relevant lifecycle result.
Their current fix is already present,
and this incident adds no reproduction of their recursion failure.
There is nothing additive to post on the closed issue.

1. **Is it really upstream's fault?**
   No.
   The consumer treated component initialization and property change callbacks as a post-layout notification.
   Slint does not make that contract.
2. **Can upstream fix it?**
   Slint could add a supported post-layout callback,
   but that would be a new API or semantic guarantee rather than the fix for a demonstrated defect.
3. **Are they supporting this use case?**
   Slint supports layouts,
   geometry properties,
   callbacks,
   and timers.
   The investigation did not find a documented post-layout callback contract.
4. **Would the repo welcome the contribution?**
   Not evaluated because constraint 1 fails and no upstream patch is proposed.
5. **Will they likely fix it?**
   Not applicable because no upstream defect is established.
   The related eager-instantiation problem was already fixed by PR #11397.
6. **Have we prototyped a minimal upstream fix?**
   Not applicable.
   The verified minimal change is a direct full-width plate binding plus a finite row-membership report burst.

Decision:
do not file a new issue and do not comment on issue #7402.
No upstream filing draft is appropriate because constraint 1 fails.
