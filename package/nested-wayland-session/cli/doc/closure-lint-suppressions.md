# Closure lint suppressions in the nested Wayland compositor

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule `rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the statement that holds the closure:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
Configuration cannot do it instead:
the rule's settings give a severity and a library path per file pattern and no allow-list of callees (`package/linter/monochromatic-lint/src/rust_rule_settings.rs`),
and lowering the severity would also let a replaceable closure in (`AGENTS.md` LN7).

This crate carries eight suppressions,
in four groups.
Every other closure that the rule found was replaced by a named function passed by path,
a loop,
a `match`,
`matches!`,
or a named function that takes the failure text as a `format_args!` value (`src/lazy_error_text.rs`).
The calloop callbacks that carry no state (`on_winit_event`,
`on_control_event`,
`on_new_client`,
`dispatch_display_clients`,
and the timers) became named functions.

## Why these callees accept no named function

In each group the callee takes a callable,
and the callable must carry a value that the call site owns.
A function item has no captured state,
so no named function can take the closure's place.
No loop or `match` can either,
because the callee is what runs the callable at the moment it holds the borrowed argument (the toplevel's pending state,
the surface's data,
the event loop's state).
None of the parameters requires `'static`,
so the lint's own exemption does not apply.

## calloop's `LoopHandle::insert_source` in `dnd.rs` and `timeline.rs`

The signature is `pub fn insert_source<S, F>(&self, source: S, callback: F) where S: EventSource + 'l, F: FnMut(S::Event, &mut S::Metadata, &mut Data) -> S::Ret + 'l` (`calloop-0.14.5/src/loop_logic.rs`,
line 101).
The bound is the loop handle's lifetime `'l`,
not `'static`.
The callback in `dnd.rs` (`schedule_release`) carries the button to release.
The callback in `timeline.rs` (`start`) carries the queue of steps and the reply sender.
The callbacks that carry nothing were replaced by named functions.

## smithay's `with_pending_state` in `handler/xdg_shell.rs`

The signature is `pub fn with_pending_state<F, T>(&self, f: F) -> T where F: FnOnce(&mut ToplevelState) -> T` (`smithay-0.7.0/src/wayland/shell/xdg/mod.rs`,
line 1698,
and line 2094 for the popup's `PopupState`).
Three statements carry a value into it:
`set_fullscreen` carries the screen geometry,
the popup reposition carries the moved positioner,
and `unconstrain_popup` carries the target rectangle.
The callbacks that carry nothing (`is_activated`,
`mark_activated`,
`mark_deactivated` in `focus.rs`) became named functions,
and so did the `with_states` callbacks that read the xdg toplevel data.

## smithay's `with_states`, `with_fractional_scale` and `Window::with_surfaces`

The signatures are `with_states<F, T>(surface: &WlSurface, f: F) -> T where F: FnOnce(&SurfaceData) -> T` (`smithay-0.7.0/src/wayland/compositor/mod.rs`,
line 399),
`with_fractional_scale<F, T>(states: &SurfaceData, f: F) -> T where F: FnOnce(&mut FractionalScaleState) -> T` (`smithay-0.7.0/src/wayland/fractional_scale/mod.rs`,
line 254),
and `Window::with_surfaces<F>(&self, mut processor: F) where F: FnMut(&WlSurface, &SurfaceData)` (`smithay-0.7.0/src/desktop/wayland/window.rs`,
line 307).
`handler/scale.rs` and `screen.rs` send the current output scale factor to a surface,
so the inner callback carries the factor.
One attribute on the statement covers both the outer and the inner closure.

## smithay's `Window::send_frame` in `render.rs`

The signature is `send_frame<T, F>(&self, output: &Output, time: T, throttle: Option<Duration>, primary_scan_out_output: F) where F: FnMut(&WlSurface, &SurfaceData) -> Option<Output> + Copy` (`smithay-0.7.0/src/desktop/wayland/window.rs`,
line 224).
The callable answers which output a surface is primarily scanned out on,
and here it is always the one output of the session,
so it carries a reference to that output.

## Build configuration

`file-enforcer.config.jsonc` lists this crate in `onlyDirs` of the rule `cargo-dylint`,
so `Cargo.toml` declares the `dylint_lib` cfg name that the attributes use.
