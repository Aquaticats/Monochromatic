# GTK 4.22: first Enter in a keyboard-driven app closes the pane or does nothing, because focus lands on the wrong widget and GtkListView rows only activate when a row has focus

Two stacked keyboard traps hit any GTK 4 app that expects "launch,
 press Enter,
 first row
activates" (the contract a browser list UI trivially provides and our nested-Wayland boundary
tests encode).
Found while building `package/file-manager/variant-gtk-sticky`,
 whose boundary test sends
`key enter` as its first input.

## Symptom

- Trap A:
   the first Enter after launch CLOSED the root pane.
  App tracing showed `closed pane closed=0 panes=0`;
   the observed pane count dropped to zero.
  No click had ever happened.
- Trap B:
   after fixing trap A,
   the first Enter did NOTHING:
   no activation,
   no error,
   model
  unchanged (boundary test timed out waiting for `paneCount: 2`).
  Pressing Down first,
   then Enter,
   activated the SECOND row (proving keys reached the list and
  activation worked once a row had focus).

## Root cause

All citations from the GNOME/gtk GitHub mirror at commit
`904b21fb235a8eb9b699224692ecfe6ee72b0e71`,
 matching system GTK 4.22.4.

Trap A:
 `gtk_widget_grab_focus` on a non-focusable container delegates to the first focusable
descendant,
 in tree order.
 `gtk/gtkwidget.c:5261`:

```c
gtk_widget_grab_focus_child (GtkWidget *widget)
{
  GtkWidget *child;

  for (child = _gtk_widget_get_first_child (widget);
       child != NULL;
       child = _gtk_widget_get_next_sibling (child))
    {
      if (gtk_widget_grab_focus (child))
        return TRUE;
    }
```

Our pane is a vertical box whose FIRST child is the header,
 and the header's only focusable
widget is its close `GtkButton`.
So "focus the pane" focused the close button,
 and Enter,
 which activates the focused widget,
activated "close".
The same landing spot is reached by GTK's initial focus assignment when a window maps with no
explicit focus.

Trap B:
 Enter does not activate "the selected row";
 it activates the FOCUSED widget,
 and a
`GtkListView` row emits `list.activate-item` from its own activate signal.
`gtk/gtklistfactorywidget.c:63`:

```c
static void
gtk_list_factory_widget_activate_signal (GtkListFactoryWidget *self)
{
  ...
  gtk_widget_activate_action (GTK_WIDGET (self),
                              "list.activate-item",
                              "u",
                              gtk_list_item_base_get_position (GTK_LIST_ITEM_BASE (self)));
}
```

wired as the row widget's activate signal at `gtk/gtklistfactorywidget.c:344`
(`gtk_widget_class_set_activate_signal (widget_class, signals[ACTIVATE_SIGNAL])`).
When keyboard focus sits on the `GtkListView` container itself (which is where a plain
`grab_focus()` lands),
 no ROW has focus,
 so Enter activates nothing,
 even though
`GtkSingleSelection` renders row 0 as selected.
Selection and focus are independent;
 arrow keys move focus into and between rows
(`gtk_list_base_focus`,
 `gtk/gtklistbase.c:551`),
 which is why Down-then-Enter worked and
activated row 1.

## Verification

Environment:
 GTK 4.22.4,
 gtk4-rs 0.11,
 app hosted in `package/nested-wayland-session/cli`
(800x600),
 driven over its control socket,
 state observed via the app's mirrored JSON file.

Reproduction commands (from this repo,
 pre-fix commits of `file-manager-gtk-sticky`):

```sh
mise run //package/nested-wayland-session/cli:build
cargo build --release   # in package/file-manager/variant-gtk-sticky
# host it, then over the control socket:
#   key enter      -> trap A: state shows paneCount 0 (pane closed)
# after close.set_focusable(false):
#   key enter      -> trap B: state unchanged
#   key down / key enter -> spawns the SECOND row's pane
```

Works cleanly (both fixes in,
 current package state):

- `key enter` as first input activates row 0 (boundary test step 2 passes,
  `mise run //package/file-manager/variant-gtk-sticky:test:wayland`).
- Down/Up move row focus,
   Enter activates the focused row,
   click activates with
  `single_click_activate`.

Fails (pre-fix):
 the two symptom flows above,
 deterministically.

## Verified workarounds

- Trap A:
   make pointer-only controls non-focusable:
   `close.set_focusable(false)`
  (`file-manager-gtk-sticky/src/pane.rs`).
   Focus then lands on the list.
  Tradeoff:
   the control leaves the Tab order,
   so it needs a keyboard equivalent elsewhere
  (Backspace closes the focused pane here);
   pure-keyboard users cannot reach the button
  itself,
   which is intentional in this design and would be an accessibility regression in a
  design without the key binding.
- Trap B:
   initialize row focus explicitly right after building the list:

  ```rust
  list.scroll_to(0, gtk4::ListScrollFlags::FOCUS | gtk4::ListScrollFlags::SELECT, None);
  ```

  Requires the gtk4 crate feature `v4_12` (`ListView::scroll_to` is 4.12 API),
   which is what
  bumped this package off the original's `v4_10` feature level.
  Tradeoff:
   the feature bump deprecates `CssProvider::load_from_data`
  (see `gtk4-cssprovider-load-from-data-deprecation.md`),
   and FOCUS-on-build means the list
  grabs the focus position before the user interacts,
   which is exactly the wanted behavior
  here but would fight designs that focus something else first.

## What does not work

- Relying on `SingleSelection`'s visible selection:
   rendering row 0 as selected does not give
  it focus,
   and Enter keys off focus,
   not selection.
- `grab_focus()` on the list container:
   focuses the container,
   not a row;
   Enter still inert.
- Reordering header/body children so the list precedes the header:
   fixes trap A's landing spot
  but leaves trap B,
   and breaks the visual header-on-top layout.

## Upstream filing decision

`.out-of-scope/` was checked:
 no GTK exemption exists.

1. Really upstream's fault?
    No for trap A (documented focus delegation,
    sensible defaults).
   For trap B the behavior follows from the focused-widget activation model and is consistent
   across GTK list widgets ([the listview keybindings work item](https://gitlab.gnome.org/GNOME/gtk/-/work_items/2182)
   documents Return as "activates the current row" where "current" means focus);
    at most this
   is a wording gap in the `GtkListView` docs,
    not a defect.
2. Can upstream fix it?
    A docs clarification could;
    the behavior itself is load-bearing.
3. Supported use case?
    Yes;
    `scroll_to(..., FOCUS, ...)` exists precisely to set the focus row.
4. Would the repo welcome the contribution?
    Not evaluated further;
    constraint 1 already fails
   for the behavior itself,
    and a docs-wording patch was judged below the filing bar.
5. Will they likely fix it?
    Nothing behavioral to fix.
6. Prototyped minimal fix?
    Not applicable to behavior;
    the consumer-side fix is two lines and
   recorded above.

Decision:
 nothing to file.
 GTK's issue tracker lives on gitlab.
gnome.
org (not searchable via
`gh`);
 the keybinding design intent is already recorded in the linked work item,
 which is the
thread a future session should read before reconsidering.
