# Slint 1.18.1 rejects competing fixed and minimum size bindings

## Symptom

The IDE's initial tree markup combined a fixed `width` with `min-width` and `max-width`,
and a `TouchArea` combined parent-sized dimensions with explicit minima.
`slint-viewer --check` reported:

```text
Cannot specify both 'width' and 'min-width'
Cannot specify both 'width' and 'max-width'
Cannot specify both 'height' and 'min-height'
```

Equal numerical values did not remove the conflict.
The failing native-tree checks were `proc_898c` and `proc_ab85`.

## Root cause

This was conflicting consumer markup,
not an established toolkit defect.
Fixed dimensions already participate in the layout constraint calculation.

In the installed `i-slint-compiler` 1.18.1 source,
`layout.rs:300` applies fixed height to both height bounds:

```rust
// i-slint-compiler-1.18.1/layout.rs
find_binding(element, "height", |s, enclosing, depth| {
    constraints.fixed_height = true;
    apply_size_constraint("height", s, enclosing, depth, &mut constraints.min_height);
    apply_size_constraint("height", s, enclosing, depth, &mut constraints.max_height);
});
```

`layout.rs:305` handles width similarly,
with a distinct percentage-width branch.
`layout.rs:281` diagnoses a competing constraint from the same enclosing component
when its binding priority is not lower:

```rust
// i-slint-compiler-1.18.1/layout.rs
if let Some((diag, level)) = &mut diag
    && Weak::ptr_eq(enclosing1, enclosing2)
    && old.priority.saturating_add(d2)
        <= binding.priority.saturating_add(depth)
{
    diag.push_diagnostic_with_span(
        format!(
            "Cannot specify both '{prop}' and '{}'",
            other_prop.name()
        ),
        binding.to_source_location(),
        *level,
    );
}
```

This finding is about competing bindings at this declaration boundary,
not a claim that inherited and overridden constraints always conflict.
The upstream regression catalog,
`tests/syntax/layout/min_max_conflict.slint`,
contains failing examples inside and outside layouts.
The inspected upstream clone at tag `v1.17.0` also contains the diagnostic in
`internal/compiler/layout.rs:240`.
The current reproduction and decisive source trace use installed 1.18.1.

## Verification

The emitting executable reports `slint-viewer 1.18.1`.
The IDE lockfile's `i-slint-compiler` checksum is
`f55d48ebef05e2bf09ba5a1fc67bcb11033b9e12a2a50ed3105c7d25e2b539d0`.

The package task creates disposable markup and checks expected success/failure,
including the exact diagnostic for rejected cases:

```bash
# From the repository root
mise run //package/ide/app:inspect:layout-constraints
```

The task prints its temporary evidence directory and writes `results.json` there.
It does not edit real project markup to exercise a failing case.

### Working patterns

- Equal `min-width` and `max-width` for a fixed sidebar constraint.
- A layout row owning its minimum dimensions,
  with its child `TouchArea` filling those same bounds.

### Rejected patterns

- Explicit `width` plus `min-width` on the same rectangle,
  even when both values are 256px.
- Explicit `width` plus `min-width` on the same `TouchArea`,
  even when both values are 48px.
- The original sidebar's fixed `width` plus `max-width`,
  as observed in `proc_ab85`.
- The original input item's `height` plus `min-height`,
  as observed in `proc_898c`.

## Verified consumer changes

`package/ide/app/ui/app.slint` now constrains the sidebar with equal minimum and maximum widths,
without an additional fixed-width binding.
This preserves its current fixed 256px allocation;
it does not implement user resizing.

`package/ide/app/ui/tree.slint` makes the containing row responsible for minimum input dimensions.
Its `TouchArea` fills the row instead of independently declaring competing minima.
The native MCP tree probe measured the visible rows at 256px by 48px.
This does not rely on input extending outside the row or overlapping a neighbor.

The repaired markup passed `lint:slint` in `proc_10ac` and `proc_6e0b`.

## What does not work

- Repeating the same numeric size as a minimum does not make the binding redundant to this diagnostic.
- Moving the fixed/minimum combination into a layout does not generally make it valid;
  the upstream regression catalog includes a `GridLayout` case.
- Removing the outer row's minimum and enlarging only its input area would not preserve non-overlapping layout bounds.
  That approach was not used.

## Upstream filing decision

No matching Slint sizing exemption was found in `.out-of-scope/`.
No issue or comment is proposed:
this is an application declaration error covered by upstream's existing compiler regression tests.

1.  Upstream fault:
    no defect established;
    the application supplied competing size bindings.
2.  Upstream fixability:
    no toolkit change is needed for this application layout.
3.  Supported use case:
    layout constraints are supported,
    and the compiler's own tests expressly reject the conflicting forms used here.
4.  Contribution policy:
    not evaluated because no upstream change or filing is proposed.
5.  Likelihood of an upstream fix:
    not applicable to the consumer correction.
6.  Upstream prototype:
    not needed;
    the runnable consumer controls exercise both valid and rejected declarations.

There is no upstream filing artifact to add.
