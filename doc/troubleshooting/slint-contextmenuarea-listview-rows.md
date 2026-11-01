# Slint 1.17.0 ContextMenuArea: right-click on a ListView/TouchArea row does not open the menu

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

A `ContextMenuArea` wrapping a list (`StandardTableView`,
 `ListView`,
 or a custom
row delegate built from `TouchArea`s) does not open its `Menu` when the user
right-clicks a row.
The row's own `TouchArea` accepts the right-press first,
 so the press never
reaches the `ContextMenu` item that would call `show()`.
The menu still opens when right-clicking a region with no interactive child
(empty space,
 a plain `Rectangle`,
 a pane background).
This is upstream Slint issue `#12354`,
 reproduced here on a custom `ListView`
delegate;
 the workaround is to forward the right-press from the row's
`pointer-event` to `context-menu.show(...)`.

## Symptom

In the prototype at `package/file-manager/app/`,
 each directory pane is a
custom `ListView` whose row delegate is a `Rectangle` containing a `TouchArea`
(for hover and selection).
The whole strip is wrapped in a `ContextMenuArea` with a `Menu` of
`Open`/`Rename`/`Delete` items.

- Right-click on a directory row:
  nothing happens.
  No menu appears.
- Right-click on a pane area with no `TouchArea` (the pane background below the
  rows,
   the title strip):
  the menu opens through the built-in path.
- Left-click on a row:
  the row selects (hover/selection work) and no menu opens.

The upstream reporter describes the same on `StandardTableView` and
`StandardListView`:
"Right-click on rows does nothing.
 Menu only opens on non-interactive areas
(empty space,
 plain Rectangle,
 etc.)."

## Root cause

Slint's `ContextMenuArea` opens its menu from the `ContextMenu` item's own input
handling,
 and that item only sees a right-press that is NOT consumed by a child
first.
A `TouchArea` covering a row consumes the press,
 so the event never reaches the
`ContextMenu` item.

The `ContextMenu` item opens the menu on a right-press it receives
(`internal/core/items.rs:1621-1625`,
 clone commit `2447c69`):

```rust
match event {
    MouseEvent::Pressed { position, button: PointerEventButton::Right, .. } => {
        self.show.call(&(LogicalPosition::from_euclid(*position),));
        InputEventResult::EventAccepted
    }
```

Input is delivered child-first:
 a row `TouchArea` returns `EventAccepted` /
`GrabMouse` for the press,
 so the `MouseEvent::Pressed { button: Right }` above is
never delivered to the `ContextMenu` item,
 and `show()` is never called.
That is the whole bug:
 the built-in right-click path works only where no child
grabs the press.

`ContextMenuArea` is a non-visual `Empty` wrapper whose `show(position)` takes a
point relative to the area (`internal/compiler/builtins.slint:1485-1499`):

```slint
export component ContextMenuArea inherits Empty {
    //! ### show(Point)
    //! Call this function to programmatically show the context menu at the given
    //! position relative to the `ContextMenuArea` element.
    callback show(position: Point);
```

So the fix is to call `show()` ourselves from the row,
 with a position expressed
in the area's coordinates.

### Keyboard paths are asymmetric across platforms

The same item's `key_event` opens the menu on the dedicated Menu key everywhere,
but on `Shift+F10` only on Windows,
 and always at position `(0,0)`
(`internal/core/items.rs:1664-1692`):

```rust
fn is_menu_key(event: &InternalKeyEvent) -> bool {
    #[allow(unused_mut)]
    let mut is_menu_key = event.key_event.text.contains(crate::input::key_codes::Menu);
    #[cfg(target_os = "windows")]
    {
        // Windows maps Shift + F10 to open the context menu
        is_menu_key |= event.key_event.text.contains(crate::input::key_codes::F10)
            && event.key_event.modifiers.shift;
    }
    is_menu_key
}

if is_menu_key(event) {
    self.show.call(&(Default::default(),));
    KeyEventResult::EventAccepted
}
```

Two consequences for a cross-platform file manager:
`Shift+F10` (the conventional context-menu shortcut on Linux and Windows) does
NOT open the menu on Linux or macOS through the built-in path;
and the keyboard-invoked menu appears at the area's top-left corner
(`Default::default()` is `(0,0)`),
 not at the focused row.

Long-press is Android-only (`internal/core/items.rs:1626-1650`,
 all arms gated by
`#[cfg(target_os = "android")]`),
 so it is not a desktop path.

## Verification

Version under test:
the prototype depends on crates.io Slint `1.17.0` (`slint`,
`i-slint-backend-winit`,
 `slint-build`).
Source traced in the Slint clone at
`/tmp/agent/slint-file-manager-assessment-20260705`,
 commit `2447c69` (1.17 line;
the `ContextMenu` item handling cited here is the 1.17.0 architecture).

Reproduction harness:
build the prototype with the embedded Slint MCP server and drive it headless.

```bash
# package/file-manager/app
mise run //package/file-manager/app:mcp   # binds 127.0.0.1:9317
```

Fails (built-in path on a row):
`find_elements_by_id` the row `TouchArea`s (`AppWindow::touch`),
`click_element` one with `button: "Right"` WITHOUT the row forwarding the press.
No `MenuFrame`/`MenuItemBase` appears in `get_element_tree`;
 the menu does not open.
The upstream issue's minimal `StandardTableView` reproduction is the canonical
runnable form of this failing case.

Works (row forwards the press;
 this prototype):
with the `pointer-event` workaround below,
 a right `click_element` on a row makes
`get_element_tree` contain `ContextMenuInternal`/`MenuFrame`/`MenuItemBase`,
 and a
screenshot shows the menu at the clicked row with that row highlighted.
Activating an item (the menu's own keyboard nav:
 dispatch `DownArrow` then
`Return`) logs the command with the clicked row's identity,
 e.g. right-clicking
`dir #20` row 0 logs `context menu command command="Open" pane_id=20 row=0`.
The keyboard button and a real `Key.Menu` (`U+F735`) press both open the menu;
 a
left `click_element` on a row does not.

Testing note:
activating a rendered menu item by `click_element` on its `MenuItemBase`
`touch-area` does NOT fire the item;
 Slint routes menu activation through the
`PopupMenuImpl` focus-scope,
 so drive it by keyboard (`DownArrow`,
 `Return`).
Also,
 the HUD `last-menu` read-back lags the command by up to one 150 ms mirror
tick,
 so poll for the expected string,
 not merely for "not empty" (the tracing
log is immediate and authoritative).

## Verified workarounds

Forward the right-press from the row's `pointer-event` to the area's `show()`,
translating the click into the area's coordinate space:

```slint
context-menu := ContextMenuArea {
    Menu { /* MenuItem { activated => { root.menu-action("...") } } ... */ }
    // ... ListView / rows ...
    touch := TouchArea {
        pointer-event(pe) => {
            if (pe.button == PointerEventButton.right && pe.kind == PointerEventKind.up) {
                context-menu.show({
                    x: self.absolute-position.x + self.mouse-x - context-menu.absolute-position.x,
                    y: self.absolute-position.y + self.mouse-y - context-menu.absolute-position.y,
                });
            }
        }
    }
}
```

The coordinate translation is the same maths Slint's own `listview.slint` uses to
report a row pointer position (`internal/compiler/widgets/common/listview.slint:151-155`):

```slint
pointer-event(pe) => {
    root.item-pointer-event(index, pe, {
        x: self.absolute-position.x + self.mouse-x - root.absolute-position.x,
        y: self.absolute-position.y + self.mouse-y - root.absolute-position.y,
    });
}
```

For the keyboard paths,
 handle `Key.Menu` and `Key.F10 + shift` in a `FocusScope`
inside the area,
 call `show(...)` at the focused element,
 and `accept` to preempt
the built-in (which would otherwise also fire and show at `(0,0)`):

```slint
key-pressed(event) => {
    if (event.text == Key.Menu || (event.text == Key.F10 && event.modifiers.shift)) {
        context-menu.show({
            x: self.absolute-position.x - context-menu.absolute-position.x + 20px,
            y: self.absolute-position.y - context-menu.absolute-position.y + 20px,
        });
        return accept;
    }
    return reject;
}
```

The implementation is `package/file-manager/app/ui/app.slint` (the
`context-menu` area,
 the row `touch` `pointer-event`,
 and the pane `FocusScope`
`key-pressed`),
 with the Rust identity plumbing in
`package/file-manager/app/src/menu.rs`.

Tradeoffs:

- The row identity for a menu command must be recorded by the app on the click
  (this prototype stores the clicked `(pane, row)` on `pointer-event` and reads it
  when a command activates),
   because the `Menu` is a sibling of the rows and its
  `MenuItem`s cannot see the row loop variable.
- Handling the Menu key in a `FocusScope` and calling `accept` means the app owns
  keyboard-menu positioning;
   forget the `accept` and the built-in fires too,
  opening a second menu at `(0,0)`.
- `Shift+F10` parity on Linux/macOS is the app's responsibility;
   the built-in
  supplies it only on Windows.

## What does not work

- Relying on the built-in right-click path for rows.
  A row `TouchArea` accepts the press,
   so the `ContextMenu` item never receives it
  (root cause above).
  Removing the row `TouchArea` is not an option:
   rows need it for hover,
   selection,
  and drag.
- Relying on `Shift+F10` on Linux or macOS.
  Slint wires it only under `#[cfg(target_os = "windows")]`
  (`internal/core/items.rs:1677-1682`).
- Expecting the keyboard-invoked menu at the focused row without intervention.
  The built-in shows at `(0,0)` (`self.show.call(&(Default::default(),))`),
   so the
  app must call `show(pos)` itself.

## Upstream filing decision

`.out-of-scope/` has no Slint exemption (checked;
 the only Slint mention is
`cargo-workspace.md`,
 about not installing Slint on the host,
 unrelated).

Duplicate search:
this IS the reported issue.
`slint-ui/slint#12354`,
"ContextMenuArea:
 right-click on StandardTableView / ListView rows does not open
menu",
 open,
 labelled `need triaging`,
 filed 2026-07-03,
 no comments.
The thread already contains the root cause (row `TouchArea`s consume the event)
and the `row-pointer-event` + `show()` workaround,
 and explicitly names
"ListView / custom row TouchAreas".

Diff of our findings against the thread:
our mouse root cause and workaround are already there,
 so they are not additive.
The genuinely new material here (the keyboard paths:
 `Shift+F10` Windows-only,
 the
`(0,0)` keyboard position,
 and the `absolute-position` translation for a custom
delegate) is about keyboard context-menu positioning,
 which is a different concern
from what `#12354` reports (mouse right-click on rows).
Adding it to that thread would be scope-creep,
 not an advance on its question.

Decision:
do not open a new issue (a duplicate exists),
 and post no comment (nothing we have
advances `#12354`'s specific topic beyond what it already states).
There is no fileable draft;
 the fix is the consumer-side workaround recorded
above,
 which solves the user-facing problem regardless of upstream movement.
