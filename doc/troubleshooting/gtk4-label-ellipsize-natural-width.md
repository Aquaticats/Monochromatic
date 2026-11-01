# GTK 4.22: ellipsized GtkLabel still requests full text width as natural size, silently stretching fixed-width siblings

An ellipsized `GtkLabel` without `max-width-chars` reports the whole untruncated text as its
NATURAL width (ellipsize only shrinks the minimum).
That inflated natural width propagates up through every ancestor's size request,
 and
`GtkBoxLayout` grows all such children toward their naturals before expand flags are even
consulted,
 so sibling widgets drift off a fixed pixel grid with no warning,
 no error,
 and no
reaction to `set_hexpand(false)`.

Found while building `package/file-manager/variant-gtk-sticky`:
 every pane header shows a
long absolute path in an ellipsized label,
 and the strip's per-column 320px `GtkFixed` canvases
rendered 394px wide,
 displacing every later column ~74px right of its debug rail.

## Symptom

- Widgets with `set_width_request(320)` inside a horizontal `GtkBox` are allocated wider than
  320 when the box has spare width,
   even though no widget in the subtree was given
  `set_hexpand(true)` on the flagged path,
   and even after an explicit `set_hexpand(false)`.
- No diagnostic of any kind:
   layout is silently wrong.
  In this repo it surfaced only because the sticky-band debug rails are drawn from model
  coordinates while panes are placed by GTK allocation,
   so a screenshot showed panes ~74px
  right of their rails (pixel-measured;
   see Verification).
- The two stretched canvases were equal width (394 = 320 + 148/2),
   which mimics expand
  distribution and sent the investigation down the wrong path first (see What does not work).

## Root cause

Three GTK behaviors compose.
All citations are from the GNOME/gtk GitHub mirror at commit
`904b21fb235a8eb9b699224692ecfe6ee72b0e71` (2026-07-09),
 matching the system GTK 4.22.4 the
apps link against.

First:
 an ellipsized label's natural width is the FULL text width unless `max-width-chars`
bounds it.
`gtk/gtklabel.c:1124` (`get_default_widths`) returns natural `-1` (unbounded) when
`max_width_chars` is unset:

```c
  if (natural)
    {
      if (self->max_width_chars < 0)
        *natural = -1;
      else
        *natural = char_pixels * MAX (self->width_chars, self->max_width_chars);
    }
```

and `gtk/gtklabel.c:1159` (`get_static_size`) then measures the layout unconstrained and
reports its size as the natural width;
 ellipsize only affects the MINIMUM (measured at
layout width 0 a few lines later):

```c
  get_default_widths (self, &minimum_default, &natural_default);
  layout = gtk_label_get_measuring_layout (self, NULL, self->ellipsize ? natural_default : -1);
  if (orientation == GTK_ORIENTATION_HORIZONTAL)
    {
      pango_layout_get_size (layout, natural, NULL);
      if (self->ellipsize)
        {
          layout = gtk_label_get_measuring_layout (self, layout, 0);
          pango_layout_get_size (layout, minimum, NULL);
```

This is documented behavior,
 not a bug:
 the `width-chars` documentation says the label
"claims the width of the entire content as its natural width" and points at
`max-width-chars` as the remedy
(<https://docs.gtk.org/gtk4/property.Label.max-width-chars.html>).

Second:
 natural sizes propagate.
 A `GtkFixed`'s natural width covers its children's extents,
a vertical `GtkBox`'s natural width is its widest child,
 so one long header label inflates the
whole pane's and then the whole canvas's natural width.
`set_size_request(320, …)` raises only the MINIMUM;
 it does not cap the natural.

Third:
 `GtkBoxLayout` distributes spare space toward children's naturals BEFORE expand flags
matter.
 `gtk/gtkboxlayout.c:753`:

```c
          /* Bring children up to size first */
          extra_space = gtk_distribute_natural_allocation (extra_space,
                                                           nvis_children,
                                                           sizes);
        }

      /* Calculate space which hasn't distributed yet,
       * and is available for expanding children.
       */
```

Only leftover space after that pass goes to children whose `gtk_widget_compute_expand` is
true (`gtk/gtkboxlayout.c:793`).
 With two canvases whose naturals both exceeded the viewport,
`gtk_distribute_natural_allocation` split the 148px surplus evenly,
 producing the equal
394/394 widths that looked exactly like expand distribution.

## Verification

Environment:
 GTK 4.22.4 (`pkg-config --modversion gtk4`),
 gtk4-rs 0.11,
 Wayland,
hosted in `package/nested-wayland-session/cli` at 800x600.

Harness 1,
 minimal probe (proves `set_hexpand(false)` itself works,
 i.e. expand was NOT the
mechanism):
 a `GtkBox` with two 320px-request `GtkFixed` children,
 each holding an
`hexpand(true)` label with tiny text;
 one canvas gets `set_hexpand(false)`.

```rust
// Cargo.toml: gtk4 = { version = "0.11", features = ["v4_12"] }
let fixed = Fixed::new();
fixed.set_width_request(320);
if explicit_false { fixed.set_hexpand(false); }
let child = Label::new(Some("x"));   // short text: tiny natural width
child.set_hexpand(true);
fixed.put(&child, 0.0, 0.0);
```

Output (800px window,
 12px spacing):

```txt
PROBE plain: width=468 compute_expand=true | pinned(false): width=320 compute_expand=false hexpand_set=true
```

`set_hexpand(false)` pins the canvas at 320 exactly as `gtk/gtkwidget.c:8386`
(`gtk_widget_update_computed_expand`) documents:
 with `hexpand_set`,
 the child walk is skipped.

Harness 2,
 real reproduction (proves the natural-width mechanism):
 the pre-fix
`file-manager-gtk-sticky` per-column layout,
 rebuilt from a clean worktree WITH
`set_hexpand(false)` added to every canvas,
 run in the nested compositor against a fixture
whose pane headers show long `/tmp/...` paths.
Pixel-measuring the screenshot (canvas debug tint scanned per column) still shows canvases at
394px and the second column's pane at x=406 instead of 332.

Harness 3,
 the cause isolated:
 same worktree,
 one change,
 `max_width_chars(1)` on the header
title label.
 Canvas overflow disappears entirely and the second column's pane renders at
x=338..651,
 inside its 332..652 rail.

Works cleanly:

- Ellipsized label WITH `max-width-chars` set (any small value):
   natural width capped,
   grid
  intact.
- Short label text (harness 1):
   naturals below the request,
   no stretching,
   expand flags behave
  as documented.
- Absolute positioning:
   one shared `GtkFixed` canvas holding every pane at explicit
  coordinates is immune,
   because allocation width does not move `Fixed` children (the fix the
  package shipped).

Fails:

- Ellipsized label,
   no `max-width-chars`,
   text wider than the intended fixed width,
   inside any
  box with spare space:
   every ancestor chain like this stretches,
   `set_hexpand(false)`
  anywhere has no effect,
   `set_size_request` has no effect (it is a minimum).

## Verified workarounds

- Set `max-width-chars` (even `1`) on every ellipsized label meant to live in a fixed-width
  container.
   Verified in harness 3.
  Tradeoff:
   the label's natural width becomes `char_pixels * max(width_chars, max_width_chars)`
  (`gtk/gtklabel.c:1155`),
   so in shrink-to-fit containers the label may request less than its
  text and always ellipsize;
   in this app's fixed grid there is no downside.
- Place grid content on ONE absolute `GtkFixed` canvas at explicit coordinates instead of
  per-column canvases in a box.
   Verified in `file-manager-gtk-sticky` (commit
  "one absolute canvas so panes align with bands");
   this is also simpler.
  Tradeoff:
   you own every coordinate;
   nothing flows,
   so anything that should flow must be
  computed.
  Note the commit message and the original in-code comment attribute the stretch to hexpand
  propagation;
   that attribution is wrong (this doc supersedes it),
   the fix itself remains
  valid for the natural-width mechanism too.

## What does not work

- `fixed.set_hexpand(false)`:
   no effect on this mechanism,
   because natural-size distribution
  runs before expand distribution (`gtk/gtkboxlayout.c:753` vs `:793`).
  The equal split it produces is a convincing expand mimic;
   harness 1 vs harness 2 is the
  disambiguation.
- `set_size_request(width, …)`:
   sets the minimum only;
   naturals still inflate.
- Suspecting a stale binary:
   ruled out by rebuilding from a clean `git worktree` (41.9s cold
  build) and re-measuring;
   the sub-second incremental rebuilds on this machine are real
  (source mtime 16:21:47,
   binary mtime 16:21:48).

## Upstream filing decision

`.out-of-scope/` was checked:
 no GTK exemption exists.

1. Really upstream's fault?
    No. The behavior is documented at
   <https://docs.gtk.org/gtk4/property.Label.max-width-chars.html> and in the width-chars docs
   ("the label claims the width of the entire content as its natural width");
    the failure was
   ours (not knowing the documented interaction),
    plus GTK's general silence about
   over-allocation.
2. Can upstream fix it?
    Changing the default would break every layout relying on
   documented natural-width behavior;
    a diagnostic for "allocation exceeded width-request"
   would be a feature request,
    not a fix.
3. Supported use case?
    Yes,
    and documented with the exact remedy we needed.
4. Would the repo welcome the contribution?
    Not evaluated further;
    constraint 1 already fails.
5. Will they likely fix it?
    Nothing to fix.
6. Prototyped minimal fix?
    Not applicable;
    the "fix" is using the documented property.

Decision:
 nothing to file.
 No draft issue is kept because the upstream documentation already
states the behavior and the remedy;
 a duplicate-search pass was therefore limited to
confirming the documentation (GNOME wiki "HowDoI/Labels" and the gtk4 property docs say it
outright).
