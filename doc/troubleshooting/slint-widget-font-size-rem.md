# Slint 1.17 std-widgets size body text in `rem`, which bare `Text` does not inherit, so hand-drawn `Text` renders a notch smaller than `Button`/`Slider` labels

Tool:
 Slint 1.17.0 (crates.io checksum `a54a407d1a0cbaa71d830ae7c236064b171368ea18db3b51ea3f2ce3f19519ee`),
default (unconfigured) widget style,
 which resolves to Fluent.
Surface trigger:
 a window that mixes std-widgets (`Button`,
 `Slider`) with plain `Text` elements and sets no
`font-size` on the `Text`.
Failure mode:
 the plain `Text` renders about 7.7% smaller than the widget labels,
 and the obvious fix (a shared
`rem`-typed constant in a `global`) does not compile.

## Symptom

The `music-player` desktop app drew several text sizes for the same visual role:

- Plain `Text` (track rows,
   the "Volume" and "Shuffle" labels,
   the radio and checkbox labels,
   the "m:ss" time
  readouts) rendered noticeably smaller than the `Button` labels ("Open",
   "Prev",
   "Play",
   "Next",
   the page tabs).
- There is no `font-size` property to set on the widget to bring it down to the `Text` size.
   Setting one fails to
  compile:

  ```text
  error: Unknown property font-size in Button
  ```

- The intuitive fix,
   a shared length constant in a `global`,
   also fails to compile:

  ```text
  error: Cannot convert between rem and logical length in a global component, because the default font size is not known
  ```

## Root cause

The two text sources are sized by two different,
 fixed multiples of the same `rem` base,
 and there is no
per-widget lever to reconcile them.

First,
 no style is configured (`build.rs` calls `slint_build::compile("ui/app.slint")` with default config,
 and
no `SLINT_STYLE` is set),
 so the compiler defaults the style to Fluent:

```rust
// i-slint-compiler-1.17.0/typeloader.rs:937
let mut style = compiler_config.style.clone().unwrap_or_else(|| "fluent".into());
```

Fluent sizes its body text at `14 * 0.0769rem`:

```slint
// i-slint-compiler-1.17.0/widgets/fluent/styling.slint:12
out property <TextStyle> body: { font-size: 14 * 0.0769rem, font-weight: FontWeight.normal };
```

and `Button` binds its label to exactly that:

```slint
// i-slint-compiler-1.17.0/widgets/fluent/button.slint:86
font-size: FluentFontSettings.body.font-size;
```

`Button` exposes no `font-size` in its public surface (only `text`,
 `icon`,
 `icon-size`,
 `primary`,
 `enabled`,
`checkable`,
 `colorize-icon`,
 `checked`),
 so the binding above cannot be overridden from consumer markup:

```slint
// i-slint-compiler-1.17.0/widgets/fluent/button.slint:7
export component Button {
    in property <string> text;
    in property <image> icon;
    in property <length> icon-size: 20px;
    in property <bool> primary;
    // ... no font-size ...
}
```

A plain `Text` with no `font-size` falls back to the window's `default-font-size` instead,
 that is,
 to `1rem`:

```slint
// i-slint-compiler-1.17.0/builtins.slint:739
/// The default font size used to render the text, when no size is specified via markup. If unset (or zero),
/// the value falls back to the enclosing `Window`'s `default-font-size`.
in property <length> default-font-size;
```

The window value resolves through this chain (`.slint`-set value,
 else platform,
 else a hard 12px):

```rust
// i-slint-core-1.17.0/items.rs:1408
Self::resolve_font_property(&window_item, Self::font_size)
    .or_else(|| Self::platform_default_font_size(&first_item))
    .unwrap_or(crate::textlayout::DEFAULT_FONT_SIZE)   // = 12px, textlayout.rs:35
```

So both sizes are multiples of the same window base:
 bare `Text` is `1.0rem`,
 the widget label is
`14 * 0.0769rem = 1.0766rem`.
 The gap is a fixed 7.7% and cannot be closed by changing the base,
 because scaling
the base moves both together.
 On Linux the base comes from the desktop font (winit reads it from the XDG portal):

```rust
// i-slint-backend-winit-1.17.0/xdg_desktop_settings.rs:148
ctx.set_platform_default_font_size(Some(LogicalLength::new(points * 96.0 / 72.0)));
```

With a 10pt desktop font that base is `10 * 96 / 72 = 13.33px`,
 so bare `Text` is about 13.3px and the widget
labels about 14.3px.

The reason the shared-constant fix fails is separate.
 A `global` has no window,
 so `rem` (which is defined
relative to the window's `default-font-size`) has nothing to resolve against,
 and the compiler rejects the
conversion outright:

```rust
// i-slint-compiler-1.17.0/passes/check_expressions.rs:26
diag.push_error("Cannot convert between rem and logical length in a global component, because the default font size is not known".into(), source_location);
```

(The sibling at line 23 rejects `phx`/logical conversions in a global for the same reason:
 the scale factor is
unknown there.)

## Verification

Version under test:
 Slint 1.17.0 (checksum above);
 `slint-viewer` 1.17.0 (`--check` compiles the markup,
 opens no
window,
 exits non-zero on error).

Harness:
 three minimal `.slint` files run through `slint-viewer --check`.

Fails,
 `rem` in a `global`:

```slint
export global G { out property <length> body: 1rem; }
export component T inherits Window { Text { text: "x"; font-size: G.body; } }
```

```text
error: Cannot convert between rem and logical length in a global component, because the default font size is not known
 --> fail-rem-global.slint:1:47
```

Fails,
 setting `font-size` on a widget:

```slint
import { Button } from "std-widgets.slint";
export component T inherits Window { Button { text: "x"; font-size: 20px; } }
```

```text
error: Unknown property font-size in Button
 --> fail-button-fontsize.slint:2:58
```

Works,
 unitless multiple in the `global`,
 `rem` applied at the element:

```slint
export global G { out property <float> f: 14 * 0.0769; }
export component T inherits Window { Text { text: "x"; font-size: G.f * 1rem; } }
```

```text
(exit 0, no diagnostics)
```

The same shape applied across `ui/app.slint` compiles clean (`slint-viewer --check`,
 exit 0),
 and a headless
render (`slint-viewer --screenshot`) shows the control row's plain `Text` ("Volume",
 "Shuffle",
 the radio and
checkbox labels,
 the time readouts) at the same size as the `Button` labels,
 the two sources that previously
diverged.

## Verified workarounds

The shipped fix:
 store the multiple unitless in a `global` and apply `* 1rem` at each element.

```slint
global Typography {
    out property <float> body-rem: 14 * 0.0769;   // = FluentFontSettings.body.font-size, in rem
}
// at each hand-drawn Text:
//   font-size: Typography.body-rem * 1rem;
```

Tradeoffs:

- Every use site must spell `* 1rem`;
   the `rem` cannot live with the constant.
- The literal `14 * 0.0769` mirrors Fluent's internal `body` size.
   If a future Slint release changes that value,
  the custom `Text` drifts from the widgets again.
   This is a manual mirror,
   not a live reference (Fluent's
  `FluentFontSettings` global is not exported from `std-widgets.slint`).
- It unifies only the `Text` you point at it.
   That is the intent here,
   but it means the constant is the single
  source of truth you must keep applying to new `Text`.

Alternative lever,
 pin the base with `Window.default-font-size`:

```slint
export component AppWindow inherits Window {
    default-font-size: 14px;   // becomes 1rem; rescales every widget and every bare Text together
}
```

Tradeoffs:

- It scales the whole type set proportionally,
   so it is the right tool for "make everything bigger/smaller" but it
  does not close the 7.7% gap on its own:
   bare `Text` stays `1.0rem` and widgets stay `1.0766rem`.
- Hardcoding it overrides the user's OS/desktop font-size preference,
   which the unset default otherwise respects
  (see the winit portal read above).
   Leave it unset to keep accessibility scaling.

The two combine:
 set `default-font-size` for the absolute scale,
 and set `Text` to `body-rem * 1rem` to sit on the
widget size within that scale.

## What does not work

- `rem` inside a `global` (`out property <length> x: 1rem;`).
   Rejected at `check_expressions.rs:26`;
   a global has
  no window,
   so `rem` is unresolvable.
- Setting `font-size` (or `font-weight`) on `Button`/`Slider`.
   The property does not exist on these widgets
  ("Unknown property font-size in Button");
   their label size is bound internally to the style.
   This is not
  uniform across std-widgets:
   text-forward widgets such as `CheckBox`,
   `Switch`,
   and `LineEdit` do expose a
  settable `font-size` (`widgets/fluent/checkbox.slint:10`),
   so the gap only bites where a widget omits it,
   and
  `Button`/`Slider` are the ones that omit it here.
- Setting only `Window.default-font-size` to unify the two.
   It rescales both sources by the same factor,
   so the
  fixed 7.7% gap between `1.0rem` and `1.0766rem` survives.
- Switching to another built-in style (`cosmic`,
   `material`,
   `cupertino`).
   Each defines its own body size in
  `rem`,
   and none adds a `font-size` to `Button`/`Slider`,
   so the same class of gap persists at different
  numbers.
   This changes the look and the `Palette`,
   it does not add a font-size lever for those widgets.

## Upstream filing decision

`.out-of-scope/` was checked for a Slint or widget-styling exemption;
 none matches (the Slint entries there do not
cover this),
 so the filing check runs normally.
 It stops at constraint 1.

1.  Is it really upstream's fault?
     No. This is intended design,
     not a defect.
     Fluent (and every built-in style)
    sizes widgets in `rem` on purpose so an app rescales them through `Window.default-font-size`;
     whether a widget
    also exposes its own `font-size` is a per-widget API choice (`CheckBox` does,
     `Button`/`Slider` do not);
     and
    the `rem`-in-`global` rejection is a correct guard,
    because the value is genuinely unknowable in a windowless `global` (`check_expressions.rs:26`).
     There is no bug
    behavior to report,
     only a design that surprised us until traced.
2.  Can upstream fix it?
     Not applicable,
     there is nothing to fix;
     a "let me set a widget's font-size" request would
    be a feature,
     not a fix.
3.  Are they supporting this use case?
     The rescale-via-`default-font-size` path and `rem` are the documented,
    supported way to size text;
     the design already covers the real need.
4.  Would the repo welcome the contribution?
     Not evaluated;
     no filing follows.
5.  Will they likely fix it?
     Not applicable;
     no defect.
6.  Have we prototyped a minimal fix?
     Not applicable;
     the resolution is a consumer-side idiom (this doc's
    workaround),
     not an upstream change.

Decision:
 do not file.
 No new issue,
 no comment.
 The durable artifact is this doc.
 No duplicate search was run,
because there is no defect to search for;
 if a future session wants Slint to expose a widget `font-size` or a
style-level `default-font-size` override,
 that would be a feature request to scope separately,
 not a bug report.
