# Slint 1.17 rejects a self-referential `rem` (e.g. `default-font-size: 0.9rem`) as a binding loop, where CSS resolves the same against the root's initial value

Tool:
 Slint 1.17.0 (crates.io checksum `a54a407d1a0cbaa71d830ae7c236064b171368ea18db3b51ea3f2ce3f19519ee`).
Surface trigger:
 setting a `Window`'s `default-font-size` (or any property that feeds the `rem` base) to an
expression that itself contains `rem`,
 to scale the whole type system relatively.
Failure mode:
 the compiler errors with a binding loop,
 so there is no pure-markup way to say "the base font is
90% of the OS/inherited font".
 CSS does allow the analogous `:root { font-size: 0.9rem }`.

## Symptom

Trying to shrink every font by a relative factor in one place,
 the natural move is:

```slint
export component AppWindow inherits Window {
    default-font-size: 0.9rem;   // "90% of the current base"
}
```

`slint-viewer --check` (and the build) reject it:

```text
error: The binding for the property 'default-font-size' is part of a binding loop (root.default-font-size -> root.default-font-size)
 --> app.slint:2:24
  |
2 |     default-font-size: 0.9rem;
  |                        ^^^^^^^
```

A web developer expects this to work,
 because the CSS equivalent does:

```css
:root { font-size: 0.9rem; }   /* resolves to 0.9 * the initial 16px = 14.4px, no loop */
```

## Root cause

`rem` in Slint is defined relative to the window's resolved `default-font-size`.
 The compiler lowers a `rem`
literal into a multiplication by that property (`i-slint-compiler-1.17.0/expression_tree.rs:619`,
 `Px = "px" ->
LogicalLength`,
 and the `rem` unit resolves against the window default font size).
 So `default-font-size: 0.9rem`
expands to `default-font-size = 0.9 * default-font-size`:
 the property's binding reads the property.
 Slint's
binding-analysis pass walks the dependency graph,
 sees the self-edge,
 and reports it:

```rust
// i-slint-compiler-1.17.0/passes/binding_analysis.rs:362
diag.push_error(format!("The binding for the property '{}' is part of a binding loop ({loop_description})", p.name()), &span);
```

(The line just above,
 `:360`,
 is the deprecated-path warning variant;
 current Slint takes the error path.)

The web behaves differently by an explicit special case,
 not by tolerating the cycle.
 In CSS,
 `rem` is the font
size of the root element,
 but "when used on the `font-size` of the root element,
 `rem` refers to the property's
initial value" (CSS Values and Units;
 MDN `<length>`).
 So on `:root`,
 `rem` resolves against the browser initial
(commonly 16px),
 breaking the self-reference before it forms.
 Slint has no such carve-out for
`default-font-size`,
 and its `rem` always points at the resolved value of that same property,
 so the cycle is
real and is rejected.

A related but distinct restriction (documented separately in
[slint-widget-font-size-rem.md](slint-widget-font-size-rem.md)) is that `rem` cannot appear in a `global` at all,
because a `global` has no window,
 so there the compiler cannot even resolve `rem` to a property.
 This doc is about
the self-reference loop inside a window,
 not the global case.

## Verification

Version under test:
 Slint 1.17.0 (checksum above);
 `slint-viewer` 1.17.0;
 a Chromium-based browser via
`agent-browser` for the CSS side.

Fails,
 Slint self-referential `rem`:

```slint
export component T inherits Window {
    width: 200px; height: 60px;
    default-font-size: 0.5rem;
    Text { text: "Hg 12.3"; }
}
```

```text
error: The binding for the property 'default-font-size' is part of a binding loop (root.default-font-size -> root.default-font-size)
```

Works,
 the web analogue (same self-reference,
 resolves against the initial 16px):

```html
<!DOCTYPE html>
<html><head><style>:root { font-size: 0.5rem; }</style></head><body><p>probe</p></body></html>
```

```js
// agent-browser eval
getComputedStyle(document.documentElement).fontSize
// => "8px"   (0.5 * the initial 16px, no loop)
```

## Verified workarounds

The scale factor must be applied outside the `rem` system,
 so it cannot read `rem` back.
 Two ways,
 both used or
considered in this repo.

Apply the factor from the host language (what `music-player/desktop-app` does).
 Expose the resolved base to the
host as a plain `length`,
 and let the host set an absolute base.
 In `.slint`:

```slint
in property <length> base-font-size;              // host sets this
default-font-size: root.base-font-size;           // 0 falls back to the platform font
out property <length> os-font-size: 1rem;         // while base is 0, this is the OS font
callback probe-os-font(length);
changed os-font-size => { root.probe-os-font(root.os-font-size); }
```

In Rust,
 apply the factor once,
 guarded against the feedback re-entry (setting `base-font-size` changes
`os-font-size`,
 which re-fires the callback):

```rust
// src/ui_font_scale.rs
let applied = std::cell::Cell::new(false);
app.on_probe_os_font(move |os_px| {
    if applied.get() || os_px <= 0.0 { return; }
    applied.set(true);
    if let Some(app) = weak.upgrade() { app.set_base_font_size(os_px * 0.9); }
});
```

Tradeoffs:
 the base becomes an absolute value the host computes,
 so the relativity now lives in host code,
 not
markup.
 Because Slint reads the OS font from the desktop portal asynchronously,
 the value is not available at
window creation;
 the code waits for the `changed` event and applies the factor once.
 Live mid-session OS-font
changes are not tracked by this one-shot (the override decouples `os-font-size` from the platform value after the
first apply);
 tracking those needs a portal subscription in the host.

Pin an absolute base (simplest,
 loses relativity).
 `default-font-size: 12px` scales everything off a fixed base.
Tradeoff:
 it stops tracking the OS/inherited font entirely,
 which is usually the opposite of what a relative
`0.9rem` was reaching for.

## What does not work

- `default-font-size: 0.9rem` (or any `Nrem` on the property that defines `rem`).
   Rejected at
  `binding_analysis.rs:362` as a self-loop.
- Routing it through a second property that still resolves to `rem`,
   for example
  `default-font-size: helper; helper: 0.9rem;`.
   `helper` reads `1rem` which reads `default-font-size` which reads
  `helper`:
   the loop just grows an edge,
   still detected.
- Expecting CSS semantics (resolve the root's own `rem` against an initial value).
   Slint has no initial-value
  carve-out for `default-font-size`;
   the self-reference is taken literally.

## Upstream filing decision

`.out-of-scope/` was checked for a Slint or CSS-parity exemption;
 none matches,
 so the filing check runs.
 It stops
at constraint 1.

1.  Is it really upstream's fault?
     No,
     not as a bug.
     The binding-loop detection is correct:
     `default-font-size:
    0.9rem` genuinely is `x = 0.9 * x`.
     The gap versus the web is that CSS adds an explicit special case
    (root-element `rem` uses the initial value) that Slint does not.
     That is a design choice,
     so a "make
    `default-font-size` resolve its own `rem` against the platform default like CSS does for `:root`" request would
    be a feature enhancement,
     not a defect report.
2.  Can upstream fix it?
     A CSS-style carve-out is implementable in principle (resolve the property's own `rem`
    against the platform/fallback base rather than the resolved value),
     but it is a semantic change,
     not a bug fix.
3.  Are they supporting this use case?
     Relative scaling via `rem` is supported for ordinary properties;
     only the
    self-referential base is special.
     The supported path is the host-applied base above.
4.  Would the repo welcome the contribution?
     Not evaluated;
     no filing follows from constraint 1.
5.  Will they likely fix it?
     Not applicable;
     no defect.
6.  Have we prototyped a minimal fix?
     Not applicable;
     the resolution is a consumer-side idiom (the host-applied
    scale),
     not an upstream change.

Decision:
 do not file.
 No new issue,
 no comment.
 This doc is the durable artifact.
 If a future session wants to
pursue the CSS-parity carve-out,
 that is a feature request to scope against Slint's `rem` semantics,
 and it should
link this doc's root-cause trace rather than be filed as a bug.
