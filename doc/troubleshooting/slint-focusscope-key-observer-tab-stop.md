# Slint 1.18.1 `FocusScope` used only to observe keys is still a Tab stop and a click target, so Tab lands where no key acts

## Symptom

A `FocusScope` wrapped around other focusable elements only to observe their keys through
`capture-key-pressed` becomes a keyboard focus stop of its own.
Tab and Shift+Tab land on it,
no element shows focus,
and the next key goes nowhere useful.

In `package/ide/app`,
the window-level scope in `ui/app.slint` observes Ctrl+F and Escape for the tree,
the source view,
and the find bar.
The find bar in `ui/find.slint` has a second observer scope that turns Return into match navigation.
Before commit `6545c521d` neither scope set any focus property.
Removing the fix again in a disposable copy (see "Verification") measured these focus owners
after each Tab press,
where `none` means that neither the tree,
the source view,
nor the find input had focus:

- Find bar closed,
  four Tab presses starting in the source view:
  `["none", "tree", "source", "none"]`;
  the intended order is `["tree", "source", "tree", "source"]`.
- Find bar open,
  six Tab presses starting in the find input,
  with only the find bar's scope unfixed:
  `["tree", "source", "none", "find", "tree", "source"]`;
  the intended order is `["tree", "source", "find", "tree", "source", "find"]`.

The source documents a second path to the same state:
a pointer press that no child element accepts gives the observer scope focus
(`input_items.rs:674` to `684`,
quoted under "Root cause").
That path was not measured in the IDE.

## Root cause

Key capture does not need focus,
but `FocusScope` accepts focus by default,
both from Tab traversal and from a click.
Code below is from the crates in the IDE build volume's cargo registry
(`~/.local/share/containers/storage/volumes/ide-cargo/_data/registry/src/index.crates.io-1949cf8c6b5b557f/`).

### Capture runs on ancestors of the focused element

Slint delivers a key event by first walking from the window down to the focused item,
calling `capture_key_event` on every ancestor:

```rust
// i-slint-core-1.18.1/window.rs:1312
        let item_list = {
            let mut tmp = Vec::new();
            let mut item = item.clone();

            while let Some(i) = item {
                tmp.push(i.clone());
                item = i.parent_item(ParentItemTraversalMode::StopAtPopups);
            }

            tmp
        };

        // Check capture_key_event (going from window to focused item):
        for i in item_list.iter().rev() {
            if i.borrow().as_ref().capture_key_event(&internal_key_event, &self.window_adapter(), i)
                == crate::input::KeyEventResult::EventAccepted
```

`FocusScope::capture_key_event` (`items/input_items.rs:690` to `713`) calls the
`capture-key-pressed` or `capture-key-released` callback without checking `has-focus` or `enabled`.
The element documentation says the same
(`i-slint-compiler-1.18.1/builtin_elements.rs:1479` to `1482`):
"Visiting all the elements starting at the Window,
going down toward the focused element,
`capture_key_pressed` or `capture_key_released` is called."
An observer scope therefore never needs focus to do its job.

### Both focus properties default to `true`

```rust
// i-slint-compiler-1.18.1/builtin_elements.rs:1402
        /// When true, the `FocusScope` will make itself the focused element when clicked.
        ///
        /// This property has no effect if the `enabled` property is set to false.
        in property <bool> focus-on-click: true;
        /// When true, the `FocusScope` will accept focus as part of the tab focus traversal.
        ///
        /// This property has no effect if the `enabled` property is set to false.
        in property <bool> focus-on-tab-navigation: true;
```

### Tab traversal offers focus to every item and the scope accepts it

`WindowInner::move_focus` walks items in focus-chain order and stops at the first one whose
`focus_event` returns `FocusAccepted`:

```rust
// i-slint-core-1.18.1/window.rs:1605
        loop {
            let can_receive_focus = match reason {
                FocusReason::Programmatic => true,
                FocusReason::TabNavigation => current_item.is_visible_or_clipped_by_flickable(),
                _ => current_item.is_visible(),
            };
            if can_receive_focus
                && self.publish_focus_item(&Some(current_item.clone()), reason)
                    == crate::input::FocusEventResult::FocusAccepted
            {
                return Some(current_item); // Item was just published.
            }
            current_item = forward(current_item);
```

`FocusScope::focus_event` refuses only the two reasons its properties name:

```rust
// i-slint-core-1.18.1/items/input_items.rs:781
        match event {
            FocusEvent::FocusIn(reason) => {
                match reason {
                    FocusReason::TabNavigation if !self.focus_on_tab_navigation() => {
                        return FocusEventResult::FocusIgnored;
                    }
                    FocusReason::PointerClick if !self.focus_on_click() => {
                        return FocusEventResult::FocusIgnored;
                    }
                    _ => (),
                };

                self.has_focus.set(true);
```

The forward walk is depth-first and wraps around at the root
(`item_tree.rs:930` to `931`,
`return step_in(root);`,
used by `next_focus_item` at `item_tree.rs:959` to `968`).
After the last focusable descendant,
the walk restarts at the top of the tree and reaches the enclosing scope before its children.
That is where the measured `none` appears:
between the source view,
the last stop in reading order,
and the tree,
the first one.

### A click that no child takes focuses the scope

```rust
// i-slint-core-1.18.1/items/input_items.rs:674
        if self.enabled()
            && self.focus_on_click()
            && matches!(event, MouseEvent::Pressed { .. })
            && !self.has_focus()
        {
            WindowInner::from_pub(window_adapter.window()).set_focus_item(
                self_rc,
                true,
                FocusReason::PointerClick,
            );
            InputEventResult::EventAccepted
```

A press on a part of the observer's area that no child accepts,
such as plain text,
moves focus to the observer and accepts the press.

## Verification

Versions under test:
Slint `1.18.1` (`slint` checksum `15e477d5ff6fec20909100ff8f7062432ebfbda7a62a0f37d0252aa126f38c83`,
`i-slint-core` checksum `ed2ea15059d70bc31b1a8b1eb4c34f2c5df04cb7e53f36179b43c0261231d646`,
`i-slint-compiler` checksum `f55d48ebef05e2bf09ba5a1fc67bcb11033b9e12a2a50ed3105c7d25e2b539d0`,
all from `package/ide/app/Cargo.lock`).

Harness:
`package/ide/app/bin/inspect-source-guards.mjs`
(now `package/ide/app/bin/inspect-source-guards.ts`,
renamed 2026-10-22),
run through the `inspect:source-guards` task of `package/ide/app/mise.toml`.
It copies the package into a disposable directory,
removes one guard line,
runs the named headless native test with real window key events
(`SLINT_BACKEND=headless`),
requires a failure with the named message,
and restores the line.
The run recorded here used the tree committed as `374ea9a36`;
its logs are in `~/temp/agent/ide-source-guard-Qu70pZ/`.

```sh
# package/ide/app/mise.toml task inspect:source-guards; arguments: cache, cargo home copy, guard names
mise run //package/ide/app:inspect:source-guards "${HOME}/temp/agent/<disposable-target>" "${HOME}/temp/agent/<cargo-home-copy>" observer-scope-not-a-stop,find-scope-not-a-stop
```

Later commits added a sidebar divider stop and renamed the closed-find focus test
(`tab_cycles_tree_divider_and_source_while_find_is_closed` in `src/native/focus_tests.rs`),
so a rerun on a newer tree expects the divider in the order.

### Works cleanly

- An outer `FocusScope` with `focus-on-tab-navigation: false` and `focus-on-click: false`
  still receives `capture-key-pressed` for keys sent to the focused tree,
  source view,
  or find input.
  The IDE's Ctrl+F,
  Escape,
  and find-navigation native tests pass with both scopes set this way
  (commit `9e688cc5b` reports 30 passing headless native tests).
- Tab and Shift+Tab then visit only the elements that act on keys:
  `observer-scope-not-a-stop-baseline.log` and `-restored.log` pass
  `tab_alternates_between_tree_and_source_while_find_is_closed`,
  and `find-scope-not-a-stop-baseline.log` and `-restored.log` pass
  `tab_visits_tree_source_and_open_find_bar_in_reading_order`.

### Fails

- Window-level observer scope with default properties:
  `observer-scope-not-a-stop-removed.log`,
  assertion `Tab must cycle tree and source only`,
  left `["none", "tree", "source", "none"]`,
  right `["tree", "source", "tree", "source"]`.
- Find bar observer scope with default properties:
  `find-scope-not-a-stop-removed.log`,
  assertion `Tab order with an open find bar`,
  left `["tree", "source", "none", "find", "tree", "source"]`,
  right `["tree", "source", "find", "tree", "source", "find"]`.

The click path was not exercised.

## Verified workarounds

Set both focus properties to `false` on every scope that only observes keys
(`package/ide/app/ui/app.slint:157` to `162` and `ui/find.slint:37` to `42` on `main` at `f9d08ae79`):

```slint
// package/ide/app/ui/app.slint:157
    FocusScope {
        width: parent.width;
        height: parent.height;
        // This scope only observes keys of its children; as a focus owner it would be a stop where no key acts.
        focus-on-tab-navigation: false;
        focus-on-click: false;
        capture-key-pressed(event) => {
```

Tradeoffs:

- The scope can still take focus programmatically,
  because `focus_event` refuses only `TabNavigation` and `PointerClick`
  (`input_items.rs:784` to `789`).
  A `focus()` call or a `forward-focus` that names the scope would still make it a dead owner.
- A click on an area no child accepts no longer moves focus.
  Focus stays where it was,
  which is the intended reading behavior here.
- The scope's own `key-pressed` and `key-released` still run for keys that a focused descendant rejects,
  because delivery bubbles to parents after capture (`window.rs:1336` to `1348`).

The search overlay's scope in `ui/search.slint:44` keeps the defaults.
It accepts every Tab in `capture-key-pressed` (`ui/search.slint:49`),
so Tab traversal never runs while focus is inside the overlay.
Whether a press on the overlay's padding can give that scope focus was not checked.

## What does not work

No alternative was tried.
`enabled: false` is documented to refuse focus from any source,
including programmatic focus (`builtin_elements.rs:1398` to `1401`),
and capture does not check `enabled`,
so it may also work for a pure observer;
it was not tested,
and it would also disable any `KeyBinding` children.

Rejecting Tab in the observer's handlers cannot help:
the stop comes from `focus_event` accepting focus during traversal,
not from key handling.

## Upstream filing decision

`.out-of-scope/` was inspected.
Its only Slint mention (`cargo-workspace.md:18`) concerns container builds,
not upstream tracking,
so no exemption applies.

Nothing is filed.

1.  Upstream's fault:
    no.
    Both defaults are documented on the element (`builtin_elements.rs:1402` to `1409`),
    and capture without focus is documented key delivery (`builtin_elements.rs:1475` to `1486`).
    The behavior is a consequence of using one element type for two roles,
    not a defect.
2.  Fixable:
    not applicable once constraint 1 fails.
3.  Supported use case:
    yes;
    observing descendants' keys is the documented purpose of `capture-key-pressed`.
4.  Contribution welcome:
    not evaluated further,
    because there is nothing to contribute.
5.  Likely to be fixed:
    not applicable.
6.  Prototype:
    not applicable.

Searches on 2026-10-05:
`gh search issues --repo slint-ui/slint FocusScope tab navigation`,
`gh search issues --repo slint-ui/slint focus-on-tab-navigation`,
and the same terms with `gh search prs`.
No report of an observer scope becoming a stop was found.
Adjacent open issues,
not duplicates:
[slint-ui/slint#13761](https://github.com/slint-ui/slint/issues/13761)
("TextInput:
a way to keep a read-only TextInput out of Tab navigation")
and [slint-ui/slint#12044](https://github.com/slint-ui/slint/issues/12044)
("FocusScope consumes the pointer press that gives it focus").
