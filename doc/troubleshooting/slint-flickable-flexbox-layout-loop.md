# Slint 1.17 can recurse when a Flickable sizes itself from a wrapped FlexboxLayout child

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

This was diagnosed while `music-player` pinned Slint commit
`85e3eb76819762cdcaa732fa87533ff896546bac`.
 The app now uses the Slint
1.17.0 crates.
io release,
 but it keeps the same breakpoint workaround because the
problem shape is a `Flickable` whose implicit layout width comes from a wrapped
`FlexboxLayout` child.
 That shape can compile and then panic with
`Recursion detected` at runtime.
 A variant also emits binding-loop warnings during
build.

## Symptom

The first side-by-side page navigation attempt kept the page-button grid and
track rows in one wrapping `FlexboxLayout` inside one `Flickable`.
 The release
build finished,
 then the app panicked before the window became usable:

```txt
thread 'main' (...) panicked at .../internal/core/properties.rs:628:9:
Recursion detected
```

A breakpointed variant that still placed `flick := Flickable` inside a
`HorizontalLayout` compiled,
 but Slint warned about a horizontal layout-info
loop:

```txt
warning: The binding for the property 'layoutinfo-h' is part of a binding loop
(root.layoutinfo-h -> content.layoutinfo-h -> content.max-width -> flick.max-width
-> layoutinfo-h -> layoutinfo-h -> layoutinfo-h -> root.layoutinfo-h).
This was allowed in previous version of Slint, but is deprecated and may cause
panic at runtime
```

## Root cause

The audited Slint source is `/tmp/agent/slint-20260601`,
 with origin
`https://github.com/slint-ui/slint.git` and commit
`85e3eb76819762cdcaa732fa87533ff896546bac`.
 That commit is an ancestor of
Slint `v1.17.0`,
 the release now used by the desktop app packages.

Slint's compiler gives `Flickable` implicit preferred and maximum sizes from
layout children when the app does not bind the `Flickable`'s own width or
height.
 The relevant source is
`internal/compiler/passes/flickable.rs:149-155`:

```rust
if !flickable_elem.borrow().bindings.contains_key("height") {
    forward_minmax_of("max-height", MinMaxOp::Min);
    forward_minmax_of("preferred-height", MinMaxOp::Min);
}
if !flickable_elem.borrow().bindings.contains_key("width") {
    forward_minmax_of("max-width", MinMaxOp::Min);
    forward_minmax_of("preferred-width", MinMaxOp::Min);
}
```

The same pass also supplies a default viewport width from the `Flickable` width
and the child layout minimum widths.
 The source is
`internal/compiler/passes/flickable.rs:157-178`:

```rust
set_binding_if_not_explicit(flickable_elem, "viewport-width", || {
    Some(
        flickable_elem
            .borrow()
            .children
            .iter()
            .filter(|x| is_layout(&x.borrow().base_type))
            // FIXME: (#407)
            .filter(|x| x.borrow().repeated.is_none())
            .map(|x| {
                Expression::PropertyReference(NamedReference::new(
                    x,
                    SmolStr::new_static("min-width"),
                ))
            })
            .fold(
                Expression::PropertyReference(NamedReference::new(
                    flickable_elem,
                    SmolStr::new_static("width"),
                )),
                |lhs, rhs| crate::builtin_macros::min_max_expression(lhs, rhs, MinMaxOp::Max),
            ),
    )
});
```

Wrapped flex cross-axis layout information needs a main-axis container
constraint for accurate wrapping.
 The lowering code either uses a supplied
constraint or reads the layout geometry size.
 The source is
`internal/compiler/llr/lower_layout_expression.rs:518-545`:

```rust
if is_cross_axis {
    // Cross-axis layout info: pass the main-axis container dimension
    // as constraint for accurate wrapping. The override (when set)
    // replaces a `self.{width,height}` read that would otherwise
    // cycle if this flex is nested on the perpendicular axis.
    let constraint_size = if let Some(override_expr) = cross_axis_size_override {
        super::lower_expression::lower_expression(override_expr, ctx)
    } else {
        match orientation {
            Orientation::Horizontal => {
                layout_geometry_size(&layout.geometry.rect, Orientation::Vertical, ctx)
            }
            Orientation::Vertical => {
                layout_geometry_size(&layout.geometry.rect, Orientation::Horizontal, ctx)
            }
        }
    };
```

At runtime,
 the flex layout helper uses that constraint as the main-axis size
for Taffy wrapping.
 The source is `internal/core/layout.rs:1965-1968` and
`:1997-2020`:

```rust
let main_axis_constraint = if constraint_size > 0 as Coord && constraint_size < Coord::MAX {
    // Use the actual container main-axis dimension (accurate)
    constraint_size
```

```rust
let (container_width, container_height) = match direction {
    FlexboxLayoutDirection::Row | FlexboxLayoutDirection::RowReverse => {
        (Some(main_axis_constraint), None)
    }
    FlexboxLayoutDirection::Column | FlexboxLayoutDirection::ColumnReverse => {
        (None, Some(main_axis_constraint))
    }
};

let mut builder = flexbox_taffy::FlexboxTaffyBuilder::new(flexbox_taffy::FlexboxLayoutParams {
    cells_h: &cells_h,
    cells_v: &cells_v,
    spacing_h,
    spacing_v,
    padding_h,
    padding_v,
    alignment: LayoutAlignment::Start,
    align_content: FlexboxLayoutAlignContent::Stretch,
    align_items: LayoutAlignItems::Stretch,
    flex_wrap,
    flex_direction: taffy_direction,
    container_width,
    container_height,
    use_measure_for_cross_axis: false,
});
```

Those pieces form the loop in this app shape:

1.  Parent layout asks the `Flickable` for horizontal layout info.
2.  The `Flickable` forwards preferred or maximum width from the content layout.
3.  The content layout asks the wrapped `FlexboxLayout` for cross-axis layout
    info.
4.  Wrapped flex cross-axis layout asks for the assigned main-axis width.
5.  That width is the `Flickable` width,
     which the parent layout was still
    solving.

Slint's binding analysis recognizes loops involving window layout and reports
some of them as warnings rather than hard errors.
 The source is
`internal/compiler/passes/binding_analysis.rs:358-363`:

```rust
let span = binding.span.clone().unwrap_or_else(|| elem.to_source_location());
if !context.error_on_binding_loop_with_window_layout && has_window_layout {
    diag.push_warning(
        format!(
            "The binding for the property '{}' is part of a binding loop ({loop_description}).\n\
            This was allowed in previous version of Slint, but is deprecated and may cause \
            panic at runtime",
            p.name(),
        ),
        &span,
    );
} else {
    diag.push_error(
        format!("The binding for the property '{}' is part of a binding loop ({loop_description})", p.name()),
        &span,
    );
}
```

The panic is the runtime property lock detecting the recursive get.
 The source
is `internal/core/properties.rs:624-628`:

```rust
panic!("Recursion detected with property {debug_name}");
}
}
assert!(!self.lock_flag(), "Recursion detected");
```

## Verification

Version under test:
 Slint git commit
`85e3eb76819762cdcaa732fa87533ff896546bac`,
 read from
`/tmp/agent/slint-20260601` with `git rev-parse HEAD`.

Failing patterns:

- One shared `Flickable` containing one wrapping `FlexboxLayout` root for both
  the page-button grid and the selected page rows.
   Command:
  `RUST_BACKTRACE=1 mise run //package/music-player/desktop-app:run`.
  Result:
   release build finished,
   then `Recursion detected` panicked at
  `internal/core/properties.rs:628:9`.
- Breakpointed content while keeping `flick := Flickable` as a child of a
  `HorizontalLayout` with a scrollbar sibling.
   Command:
  `RUST_BACKTRACE=1 mise run //package/music-player/desktop-app:run`.
  Result:
   build emitted the `root.layoutinfo-h -> content.layoutinfo-h ->
  content.max-width -> flick.max-width` loop warning quoted above.

Working pattern:

- Keep one shared `Flickable`,
   but make it an explicitly positioned child of a
  queue `Rectangle` instead of a child participating in a parent layout.
   Bind
  `width: max(0px, parent.width - 18px)`,
   `height: parent.height`,
  `viewport-width: self.width`,
   and `viewport-height: content.preferred-height`.
  Place the scrollbar as a separately positioned sibling.
   Inside the content,
  use explicit wide and narrow branches guarded by `root.width >= 900px` and
  `root.width < 900px`.

Verification command for the working pattern:

```sh
RUST_BACKTRACE=1 mise run //package/music-player/desktop-app:run
```

Observed output after the fix:

```txt
Compiling music-player v0.1.0 (/work)
Finished `release` profile [optimized] target(s) in 12.44s
```

The process stayed running after startup,
 with no binding-loop warning and no
`Recursion detected` panic in stderr.

## Verified workarounds

### Explicitly position the Flickable

Patch shape in `package/music-player/desktop-app/ui/app.slint:412-418`:

```slint
flick := Flickable {
    x: 0px;
    y: 0px;
    width: max(0px, parent.width - 18px);
    height: parent.height;
    viewport-width: self.width;
    viewport-height: content.preferred-height;
```

Trade-off:
 the scrollbar gutter is now a fixed 18px subtraction in the queue
rectangle.
 The value keeps the previous prominent custom scrollbar gutter,
 but
future scrollbar width changes need the subtraction updated with it.

### Use breakpointed content branches inside the shared Flickable

Patch shape in `package/music-player/desktop-app/ui/app.slint:424`,
 `:500`,
`:517`,
 and `:522`:

```slint
if root.width >= 900px: HorizontalLayout { }
if root.width < 900px && root.page-labels.length > 0: FlexboxLayout { }
if root.width < 900px && root.page-labels.length > 0: Rectangle { }
if root.width < 900px: VerticalLayout { }
```

Trade-off:
 the transition is controlled by an explicit breakpoint instead of a
pure minimum-width wrap rule.
 The app still preserves the intended user
behavior:
 wide viewports show all page buttons beside the selected tracks;
narrow viewports show all page buttons above the selected tracks.

## What does not work

- Adding `preferred-height: 30px` to repeated track rows:
   this removes one
  child-size dependency,
   but `RUST_BACKTRACE=1 mise run
  //package/music-player/desktop-app:run` still panicked with
  `Recursion detected`.
- Pinning `x: 0px` and `y: 0px` on the content or wrapping the content in an
  extra rectangle:
   these changes did not remove the implicit width/layout-info
  path from `Flickable` through its layout child.
- Keeping the `Flickable` inside `HorizontalLayout` after adding breakpointed
  content:
   the app no longer hit the original startup panic during the short
  run,
   but Slint emitted the binding-loop warning that says the pattern can
  panic at runtime.

## Draft upstream issue

### Why we do not file this upstream now

1.  Is it really upstream's fault?
     Partly.
     Slint's compiler and runtime expose
    the loop,
     but the app can avoid it by explicitly positioning `Flickable` and
    by splitting the wrapping flex root into wide and narrow branches.
2.  Can upstream fix it?
     Possibly,
     but the source trace crosses `Flickable`
    geometry fixup,
     layout-info lowering,
     binding analysis,
     and runtime flex
    layout.
     This is not a one-line consumer-independent fix from the evidence
    collected here.
3.  Are they supporting this use case?
     Slint supports `Flickable` and
    experimental `FlexboxLayout`,
     but this exact combination uses an
    experimental layout primitive with a self-sizing viewport.
     No upstream docs
    or examples were collected for that composition.
4.  Will they likely fix it?
     Unknown from this investigation.
     Commit history was
    not audited for layout-info loop fixes beyond reading the pinned source.
5.  Have we prototyped a minimal fix compatible with their architecture?
     No. A
    consumer-side workaround exists and was verified first,
     so an upstream patch
    prototype is not justified for this app fix.

Do not file as-is:

~~~md
Title: Flickable with wrapped FlexboxLayout child can form a layout-info loop

A Slint app using a `Flickable` whose implicit width comes from a wrapped
`FlexboxLayout` child can panic at runtime with `Recursion detected`. A related
shape emits a warning:

```txt
root.layoutinfo-h -> content.layoutinfo-h -> content.max-width -> flick.max-width
```

Source trace from commit `85e3eb76819762cdcaa732fa87533ff896546bac`:

- `internal/compiler/passes/flickable.rs:149-155` forwards `Flickable`
  preferred and maximum size from layout children when width or height is not
  explicitly bound.
- `internal/compiler/llr/lower_layout_expression.rs:518-545` makes wrapped
  flex cross-axis layout info depend on a main-axis container size.
- `internal/core/layout.rs:1965-2020` feeds that constraint into Taffy.
- `internal/compiler/passes/binding_analysis.rs:358-363` downgrades some
  window-layout loops to warnings.
- `internal/core/properties.rs:624-628` panics when a recursive property get is
  detected.

A minimal standalone reproduction and an upstream-compatible patch have not
been prepared. The consumer workaround is to explicitly position the
`Flickable`, bind its width and height from a non-layout parent, and split the
wide and narrow flex content into explicit branches.
~~~
