# Slint 1.18.1 ListView large jumps discard the within-row scroll offset

## Symptom

A native IDE tree used 48px rows and a 628px viewport.
Revealing model row 43 requested `content-y = -1484px`,
which should place its bottom at the viewport bottom.
Slint instead settled at `-1440px`.
The row's relative bottom became 672px,
leaving 44px outside the viewport.
Repeating the same reveal did not correct it.

The real nested Wayland probe failed with:

```text
Ctrl+0 did not reveal the off-screen current file
```

The independent native-window regression failed with:

```text
row 43 is only partially revealed: bottom 672, viewport 628
```

This is distinct from the nested-scrollable gesture interruption in
[`slint-flickable-windowed-model-scroll.md`](slint-flickable-windowed-model-scroll.md).
The current failure occurs during a programmatic jump,
without an active drag or wheel gesture.

## Root cause

The verified release is `v1.18.1`,
commit `372cf0ee5577c3dfec309a45e7b778ba4e81b734`.
The fresh prototype clone is
`/home/user/temp/agent/slint-listview-prototype-KQSJ5gjn/upstream`.
Its origin was verified as `https://github.com/slint-ui/slint.git` before editing.

`internal/core/model/repeater.rs:745` delegates native ListView updates to
`ensure_updated_listview_callback`,
which invokes the shared `update_visible_instances` algorithm.
The latter reads the requested offset at `repeater.rs:290`:

```rust
// internal/core/model/repeater.rs
let mut content_y_value = props.content_y_get().get();
```

The random-seek branch at `repeater.rs:327` estimates a starting row,
then initializes its viewport-relative position to zero:

```rust
// internal/core/model/repeater.rs
let (mut new_offset, mut new_offset_y) = if first_item_y
    > -content_y_value + one_and_a_half_screen
    || (props.computes_content_height() && last_item_bottom + element_height < -content_y_value)
{
    // Jumping more than 1.5 screens: random seek.
    ops.splice(0, ops.len(), 0);
    state.offset = ((-content_y_value / element_height).floor() as usize).min(row_count - 1);
    (state.offset, 0 as Coord)
```

For equal-height rows,
that loses the requested remainder within the estimated row.
With the IDE inputs,
`floor(1484 / 48)` chooses row 30,
and its zero relative offset becomes `-30 * 48 = -1440`.

`repeater.rs:423` reconstructs and writes the scroll position from that state:

```rust
// internal/core/model/repeater.rs
state.anchor_y = state.cached_item_height * state.offset as Coord;
let new_content_y = -state.anchor_y + new_offset_y;
```

The same random-seek branch was still present in upstream `master` when checked on 2026-10-05.
No installed-source patch was applied.

## Verification

### Native consumer regression

`package/ide/app/src/native/tree_scroll_tests.rs` creates the actual application window,
installs 60 tree rows,
shows the window,
and forces rendered snapshots before checking row bounds.
It covers a forward jump,
the final row,
and a backward jump.

```bash
# From the repository root
mise run //package/ide/app:test:native
```

Before replacing the tree's ListView path,
`proc_88bc` ran nine native tests:
eight passed and the far-reveal test failed with the 672px versus 628px mismatch.
The consumer change passed the same regression in `proc_d499`.

The actual light-mode compositor probe,
`/tmp/monochromatic-ide-native-D6RbFJ/focus-reveal.mjs`,
then verified off-screen Ctrl+0 reveal,
retained source selection,
and Tab/Shift+Tab traversal.
`tree-offscreen-reveal-light.png` was inspected.
A separate windowed-row click copied exactly `Catalog source 5`,
verifying that materialized row slots still dispatch the correct native path.

### Isolated algorithm controls

The checked-in driver is
[`slint-listview-random-seek-probe.rs`](slint-listview-random-seek-probe.rs).
It includes the supplied source function verbatim,
with scalar property and row-materialization adapters.
It does not reimplement the algorithm under test.

```bash
# Supply a checked-out or installed Slint repeater.rs path
mise run //package/ide/app:inspect:listview-algorithm -- /path/to/internal/core/model/repeater.rs
```

The task compiles and runs in a network-disabled container,
limited to 2 GiB memory and 2 CPUs.
Only disposable probe files are mounted;
no credentials or real project contents are supplied to the tested code.

On original 1.18.1 source,
`proc_2f24` reported four passing controls and one failure:

```text
height 24, viewport 101, requested -781.25, got -768
```

On the completed prototype,
`proc_3830` passed all five controls.
The matrix includes uniform heights of 24,
48,
and 65.5 pixels;
viewport heights of 101,
300,
and 628 pixels;
forward,
backward,
and end seeks;
variable-height row coverage;
empty models;
and explicit positive overscroll bindings.

### Working and failing patterns

- Small equal-height scrolling matching the shape of upstream issue #4463's example passes on 1.18.1.
- Empty models and the variable-height coverage control pass on unmodified 1.18.1.
- Large uniform-row jumps with a nonzero within-row remainder fail on unmodified 1.18.1.
- Whole-row-aligned large jumps do not expose the lost-remainder assertion.
- The completed prototype passes the recorded matrix,
  including the positive-overscroll control.

## Verified consumer workaround

`package/ide/app/ui/tree.slint` uses a native `ScrollView`
with explicit fixed-row geometry instead of the ListView estimator.
Only the visible rows plus overscan are materialized.
Their model indices and absolute content positions come from the known 48px row height.
Native Flickable scrolling and scrollbar behavior remain in use;
there is no competing scroll animation.

Tradeoffs:

- The application owns fixed-height row windowing and global-index mapping.
- This is suitable for the current single-line tree,
  not a general variable-height list solution.
- Accessibility focus is rebased to the materialized row window.
  Related upstream [issue #12645][focus] explains the existing descendant-index behavior.
- Reusing input items requires retaining the original pressed row/model identity.
  An observed-failing test caught a release activating a different row after scrolling;
  the row's input handler now rejects that stale click.

## Prototype and rejected approaches

The prototype is preserved in
[`slint-listview-random-seek-offset.patch`](slint-listview-random-seek-offset.patch).
It exists only in the disposable clone and this artifact,
not in the installed dependency or application patch configuration.

The first candidate only retained the estimated within-row remainder:

```rust
// internal/core/model/repeater.rs
(state.offset, content_y_value + element_height * state.offset as Coord)
```

That fixed the uniform-row controls but exposed a leading gap in the variable-height control.
The completed candidate also handles reaching the known first-row origin:
a positive estimated leading gap is removed when the requested content position is nonpositive.
Explicit positive overscroll is left intact.

The correctness boundary is narrow:
this preserves the estimated within-row offset and avoids a gap at the known origin.
It does not promise exact absolute positions for arbitrary unmeasured variable-height prefixes.
The full Slint integration suite,
C++ bindings,
and integer-coordinate builds were not run against the prototype.

Other rejected approaches:

- Repeating the same native reveal while replacing the model still left the target partly clipped.
- Assuming the earlier nested-Flickable gesture bug explained this symptom:
  the present case has no active gesture,
  and the traced random-seek branch directly accounts for the measured offset.
- Calling the original #4463 example proof of this exact current root cause:
  its small equal-height control passes on the tested 1.18.1 source.

## Upstream filing decision

No matching `.out-of-scope/` exemption was found.
Tracker searches included `ListView scroll`,
issues and pull requests,
and inspection of all comments on [#4463][position],
[#11826][ensure],
and [#12645][focus].
The original #4463 commenter suspected layout estimation,
but did not identify this current large-jump remainder branch.
The evidence is additive to that broader programmatic-position issue,
not proof that its original 1.3.2 reproduction has the same cause today.

1.  Upstream fault:
    the shared ListView algorithm changes a requested offset even for uniform known-height rows.
2.  Fixability:
    a source-level prototype preserves the missing remainder and passes the recorded controls.
3.  Supported use case:
    ListView exposes content position;
    the existing position and ensure-visible issues cover programmatic navigation.
4.  Contribution policy:
    `CONTRIBUTING.md`,
    `README.md`,
    `AGENTS.md`,
    the bug-report form,
    and the pull-request template were inspected.
    Contributions and AI-assisted development are explicitly discussed;
    no AI-report prohibition was found in these sources.
5.  Likelihood of acceptance:
    no documented refusal or non-goal was found for correcting this position behavior.
    The related issues remain open.
6.  Prototype:
    completed in a fresh disposable v1.18.1 clone,
    with original-source failure,
    a rejected intermediate candidate,
    and passing bounded nontrivial controls for the completed candidate.

No public issue,
comment,
or pull request was posted.
The additive comment draft remains local and requires authorization before publication.

### Additive comment draft for #4463

~~~md
Additional evidence from Slint 1.18.1 on Linux/Wayland,
using both a native-window test and a standalone driver of the exact virtualization function.
This report and its tests were prepared with AI assistance;
no human verification is claimed.

For 48px rows in a 628px viewport,
setting content-y to -1484px settles at -1440px.
The intended target row consequently extends 44px past the viewport bottom.
Repeating the reveal does not repair it.

In v1.18.1 internal/core/model/repeater.rs,
the random-seek branch floors the requested offset into a row index
and assigns new_offset_y = 0.
The later -anchor_y + new_offset_y write explains the discarded remainder.

The small 800px-row/-200px-offset shape from this issue's original example passes in the same 1.18.1 controls.
The large-jump case therefore adds a current diagnosis,
rather than establishing that the original report still has an identical cause.

A disposable source prototype preserves the estimated within-row remainder
and corrects a positive leading gap when it reaches row zero with a nonpositive requested offset.
Five control groups pass after that change:
uniform forward/backward/end seeks,
variable-height contiguous coverage,
the small original-example control,
empty models,
and explicit positive overscroll.
The full toolkit integration suite and C++/integer-coordinate variants have not been checked.

The consuming application currently uses fixed-row windowing over ScrollView,
so it does not depend on this prototype patch.
~~~

[position]: https://github.com/slint-ui/slint/issues/4463
[ensure]: https://github.com/slint-ui/slint/issues/11826
[focus]: https://github.com/slint-ui/slint/issues/12645
