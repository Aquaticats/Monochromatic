# Vet: Slint testing API (ElementHandle / accessibility-based UI testing) for Slint+Rust

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Date:
 2026-06-07.
 Standard:
 choosing-technology,
 FULL VERIFICATION.
Question:
 is the Slint testing API the viable analog of Compose's `runComposeUiTest`
for driving Slint+Rust desktop UI from `#[test]`,
 headlessly,
 no display?

Verdict up front:
 yes,
 it works headlessly and passed full verification.
 Two
non-obvious setup requirements and one real capability gap (keyboard/text input)
are documented below.

## What it actually is (naming correction)

The task calls it "slint:
:
testing".
 There is no `slint::testing` module.
 The public
surface is a separate crate,
 `i-slint-backend-testing` (published on crates.
io,
current stable `1.16.1`),
 added as a dev-dependency.
 It installs its own platform
(`TestingBackend` / `TestingWindow`) that replaces the real winit/femtovg backend,
so tests need no display server,
 GPU,
 or system GUI libraries.

Entry points (clone:
 `/tmp/agent/slint-vet`,
 `internal/backends/testing/lib.rs`):

- `init_no_event_loop()` `lib.rs`:
  36,
  platform,
   no event loop,
   mock time.
- `init_integration_test_with_mock_time()` `lib.rs`:
  51,
  event loop + deterministic
  mock clock (the runComposeUiTest-style controllable clock).
   One `#[test]` per process.
- `init_integration_test_with_system_time()` `lib.rs`:
  64,
  event loop,
   real clock.
- `mock_elapsed_time(Duration)` `lib.rs`:
  73,
  advance the mock clock;
   drives animations
  and timers deterministically.

## Source audit: capabilities (cited)

Query / introspection API,
 `internal/backends/testing/search_api.rs`
(`pub use search_api::*`,
 always exported,
 available to external consumers):

- `ElementHandle::find_by_accessible_label(&root, label)` `search_api.rs`:
  404
- `ElementHandle::find_by_element_id(&root, "Comp::id")` `search_api.rs`:
  434
- `ElementHandle::find_by_element_type_name(&root, ty)` `search_api.rs`:
  444
- `ElementQuery` builder:
   `match_id`,
   `match_type_name`,
   `match_inherits`,
  `match_accessible_role`,
   `match_predicate`,
   `find_first`,
   `find_all` `search_api.rs`:
  227-321
- Accessibility property reads:
   `accessible_label`,
   `accessible_role`,
  `accessible_value`,
   `accessible_checked`,
   `accessible_enabled`,
   `accessible_expanded`,
  `accessible_item_index/count`,
   `accessible_value_min/max/step`,
   plus ~20 more
  `search_api.rs`:
  585-847
- Geometry:
   `size`,
   `absolute_position`,
   `computed_opacity` `search_api.rs`:
  847-893

Action / input simulation on `ElementHandle` (also always exported):

- `invoke_accessible_default_action()` `search_api.rs`:
  606 (the "click via a11y" path)
- `invoke_accessible_increment_action` / `decrement` / `expand` `search_api.rs`:
  893-915
- `set_accessible_value(v)` `search_api.rs`:
  637 (set value-bearing widgets,
   e.g. text/slider)
- Pointer events:
   `single_click`,
   `mock_single_click`,
   `double_click`,
   `drag`,
  `mock_drag`,
   `scroll` `search_api.rs`:
  954-1068.
   These dispatch real `WindowEvent`
  pointer events at the element center,
   with `mock_*` variants using mock time.

Deterministic font metrics:
 `TestingWindow` implements a text-metrics renderer with a
`FixedTestFont` (`testing_backend.rs:375` RendererSealed;
 `is_fixed_test_font`) so layout
is reproducible without a real rasterizer.

## Source audit: limits (cited)

1. No screenshot / golden support in this API.
    `TestingWindow`'s renderer only
   computes text/char sizes (`testing_backend.rs:375` onward);
    it does not rasterize to
   a pixel buffer.
    Pixel/golden testing is a separate mechanism:
    the software renderer
   (`i-slint-renderer-software` feature `testing`) plus `tests/screenshots/` comparing
   PNGs (`doc/testing.md` "Screenshot tests",
    `tests/screenshots/testing.rs`).
    The two
   are orthogonal:
    the accessibility API is structural,
    not visual.

2. No keyboard/key-event method on `ElementHandle` at all (grep:
    none in `search_api.rs`).
   The only keyboard helpers are window-adapter free functions
   `send_keyboard_key_text` / `send_keyboard_char` / `send_keyboard_string_sequence`
   (`testing_backend.rs:80-130`),
    and every one of them is gated behind
   `#[cfg(any(feature = "internal", feature = "ffi"))]`.
    The crate's default feature set
   is empty (no `default = [...]` in `internal/backends/testing/Cargo.toml`;
    `internal`
   is labeled "only enabled for Slint's own tests").
    `send_mouse_click` is likewise
   `internal`/`ffi`-gated.
    So an external consumer on the published default-feature crate
   has NO supported way to type text into a focused element.
    Workarounds:
    drive value via
   `set_accessible_value` on the target element,
    or opt into the unsupported `internal`
   feature to reach `send_keyboard_string_sequence`.
    This is the main gap vs Compose's
   `performTextInput`.

3. Visibility requirement:
    descendant search skips not-visible elements
   (`visit_descendants_impl` `search_api.rs:339`,
    `if !item_rc.is_visible() continue`).
   A component created without laying out the window (e.g. `init_no_event_loop()` then an
   immediate query) yields zero-size,
    not-visible items and finds nothing.
    The fix is the
   event-loop pattern plus `app.show()` (see verification).
    This bit my first attempt.

4. Debug-info requirement:
    the `ElementHandle` introspection API panics at runtime unless
   the Slint compiler emitted debug info.
    Message:
    "The use of the ElementHandle API
   requires the presence of debug info ... Set `SLINT_EMIT_DEBUG_INFO=1` ... or use
   `compile_with_config` and `with_debug_info`".
    For `slint-build` users,
    set
   `CompilerConfiguration::with_debug_info(true)` in `build.rs`.

## Tests / CI / fuzzing evidence (per source-audit rule)

- The testing backend has its own integration tests:
   `internal/backends/testing/tests/click.rs`
  (pointer click/double-click) and `layout_kind.rs`.
   `click.rs` is the canonical
  event-loop usage pattern I mirrored.
- No property-based or fuzz harness in the testing backend (grep for
  `proptest|quickcheck|cargo-fuzz|libfuzzer|arbitrary` under
  `internal/backends/testing/` returns nothing).
   Absence noted as a finding;
   correctness
  of the harness rests on the example tests plus the broader driver/screenshot suites.
- A higher tier exists:
   `system-testing` and `mcp` features (`systest.rs`,
   `mcp_server.rs`,
  `introspection/`) expose the same element tree over a transport / to an MCP agent,
  intended for driving a real (AccessKit) backend.
   Not needed for in-process `#[test]`.

## Testing-module activity (note only; stack maintenance is the Slint-stack vet's job)

The testing module is actively developed,
 not frozen:
 `search_api.rs` is large (1526 lines)
and includes recently-added gesture APIs (`drag`,
 `mock_drag`,
 `scroll`,
 with
`DRAG_STEP_SIZE`/`DRAG_STEP_DELAY_MS`).
 The CHANGELOG shows a steady stream of
accessibility additions feeding this API (e.g. `accessible-orientation`,
`accessible-live-region`,
 landmark roles #11831),
 which the testing API reads directly.

## FULL VERIFICATION (built, ran, passed, headless)

Throwaway crate at `/tmp/agent/slint-testing-demo/` (not in the repo).
 Uses only the
published default-feature crates:
 `slint = "=1.16.1"` (default-features off,
 features
`std`,
 `compat-1-2`,
 so no winit/femtovg pulled in) and dev-dependency
`i-slint-backend-testing = "=1.16.1"`.

Component (inline `slint!` macro),
 a button + counter expressed via accessibility:

```slint
export component App inherits Window {
    out property <int> counter: 0;
    Rectangle {
        accessible-role: button;
        accessible-label: "Increment";
        accessible-action-default => { root.counter += 1; }
    }
}
```

Test body (`src/lib.rs`):
 `init_integration_test_with_mock_time()`,
 then inside
`spawn_local`:
 `App::new()`,
 `app.show()`,
 `find_by_accessible_label(&app, "Increment")`,
assert `accessible_label()` == "Increment" and `accessible_role()` ==
`AccessibleRole::Button`,
 `invoke_accessible_default_action()` twice,
 assert
`get_counter()` goes 0 -> 1 -> 2,
 then `quit_event_loop()`;
 outer `run_event_loop()`.

Exact command (display vars unset to prove headlessness):

```sh
cd /tmp/agent/slint-testing-demo
env -u DISPLAY -u WAYLAND_DISPLAY SLINT_EMIT_DEBUG_INFO=1 \
  CARGO_TARGET_DIR=/tmp/agent/slint-testing-demo/target cargo test
```

Output:

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running unittests src/lib.rs (target/debug/deps/slint_testing_demo-be079f3e89d0547b)

running 1 test
test tests::button_increments_counter_via_accessibility ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests slint_testing_demo

running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Reproducibility caveat for whoever wires this into the repo:
 both env requirements above
are load-bearing.
 Without `SLINT_EMIT_DEBUG_INFO=1` the test compiles but panics at runtime
("requires the presence of debug info");
 without `app.show()` (or the event loop) the
query finds nothing.

## Comparison to runComposeUiTest

Shared model:
 both drive a structural/semantics tree (not pixels),
 render offscreen,
 and
expose a deterministic clock.

Slint testing API can do:

- Find nodes by accessible label,
   element id,
   type,
   role,
   or arbitrary predicate.
- Read accessibility state and geometry off a handle (label,
   role,
   value,
   checked,
  enabled,
   expanded,
   size,
   position,
   opacity).
- Click / double-click / drag / scroll an element;
   invoke default/increment/decrement/
  expand accessible actions;
   set accessible value.
- Control time deterministically via `init_integration_test_with_mock_time` +
  `mock_elapsed_time`,
   the direct analog of Compose `mainClock.advanceTimeBy`.
- Run fully headless in-process under `cargo test`,
   no display (verified above).

Slint testing API cannot do (vs Compose):

- No `performTextInput` analog on a node.
   No key-event method on `ElementHandle`;
   the
  keyboard free-functions are `internal`/`ffi`-gated,
   so default-feature consumers have no
  supported per-element text entry.
   Compose's `performTextInput`/`performKeyInput` have no
  clean equivalent here;
   closest is `set_accessible_value`.
- No built-in screenshot/golden assertion in this API (Compose has `captureToImage` +
  Roborazzi/Paparazzi ecosystem).
   Slint's pixel testing is a separate software-renderer
  harness,
   not part of `ElementHandle`.
- No rich matcher/assertion DSL (`onNodeWithText(...).assertIsDisplayed()`);
   you get
  iterators of handles plus `assert_eq!`,
   more manual than Compose's fluent API.
- One `#[test]` per process for the event-loop init variants (process-global platform),
  vs Compose's per-test rule.
   Manageable but a structural constraint.

## One alternative, with reasons

Manual / screenshot-golden testing (drive the real app,
 assert on rendered pixels):
use the `i-slint-renderer-software` `testing` feature to render a component to a
`SharedPixelBuffer` and diff against a committed PNG (the `tests/screenshots/` approach),
or drive the running desktop app via OS automation.
 Reasons to keep it as a complement,
not the primary:
 golden images are brittle (font/AA/DPI drift),
 require committed binary
fixtures and update tooling,
 and assert appearance rather than behavior;
 OS-level
automation reintroduces a display dependency and flakiness.
 The accessibility API is the
right primary for behavior/logic tests;
 reserve screenshot golden for the narrow set of
truly visual regressions,
 and use `set_accessible_value` (or an opt-in `internal` build)
where text entry is unavoidable.

## Recommendation

Adopt `i-slint-backend-testing` (ElementHandle accessibility API) as the Slint+Rust UI-test
technology,
 the runComposeUiTest analog.
 It is open-source,
 ships with the Slint stack
already in use,
 runs headless in-process under `cargo test` (verified),
 and gives
deterministic-clock behavioral testing by accessible label/role plus pointer simulation.
Budget around two limits:
 per-element text input needs `set_accessible_value` or an
opt-in `internal` feature,
 and visual regressions need the separate software-renderer
golden harness.
 Decision doc belongs at `doc/decision/<project>.md` once picked.
