//! Directory-listing and preview panes,
//!  trimmed from the original's `pane.rs`.
//!
//! Same virtualized `ListView` rows,
//!  single-click/Enter activation,
//!  Ctrl-forced duplicates,
//!  and
//! header close button as the original;
//!  the thumbnail service and drag-and-drop shims are out of
//! scope for this layout prototype,
//!  so a preview pane shows a typed icon plus filename only.

/// What:
///  imports the single-slot interior-mutability cell.
/// Why:
///  the Ctrl-at-activation flag is shared between the click/key controllers and activation.
use std::cell::Cell;
/// What:
///  imports the borrowed path type.
/// Why:
///  a preview pane takes the previewed file path by reference.
use std::path::Path;
/// What:
///  imports the reference-counted pointer.
/// Why:
///  the shared Ctrl flag is held by three closures on the same pane.
use std::rc::Rc;

/// What:
///  imports the GTK widget-extension traits (builders,
///  controllers,
///  list/box helpers).
/// Why:
///  the pane adds event controllers and presents a scrolled list,
///  all via prelude traits.
use gtk4::prelude::*;
/// What:
///  imports the key symbol and modifier-mask types.
/// Why:
///  activation checks whether Ctrl was held and whether a key press was an activation key.
use gtk4::gdk::{Key, ModifierType};
/// What:
///  imports the boxed-any wrapper and the event-propagation verdict enum.
/// Why:
///  `FileEntry` rides a `BoxedAnyObject` through the model;
///  the key handler returns
///      `Propagation`.
use gtk4::glib::{BoxedAnyObject, Propagation};
/// What:
///  imports the list-store model type.
/// Why:
///  the pane's rows live in a `ListStore` of boxed entries feeding the `ListView`.
use gtk4::gio::ListStore;
/// What:
///  imports the text-ellipsization mode enum.
/// Why:
///  a long directory path in the header truncates in the middle rather than widening the pane.
use gtk4::pango::EllipsizeMode;
/// What:
///  imports the concrete widget,
///  controller,
///  and factory types the pane is built from.
/// Why:
///  named explicitly so construction reads without a glob import.
use gtk4::{
    Box as GtkBox, Button, EventControllerKey, GestureClick, Image, Label, ListItem, ListView,
    Orientation, PropagationPhase, ScrolledWindow, SignalListItemFactory, SingleSelection,
};

/// What:
///  imports the snapshot,
///  entry,
///  and kind domain types from the original app's crate.
/// Why:
///  the pane renders the shared model's `DirectorySnapshot` of `FileEntry` rows.
use file_manager::types::{DirectorySnapshot, EntryKind, FileEntry};

/// What:
///  horizontal gap in pixels between a row's icon and its name label (and header items).
/// Why:
///  named so the one spacing value is not a bare magic literal.
const ROW_SPACING: i32 = 6;

/// What:
///  pixel size of the icon shown in a preview pane body.
/// Why:
///  a large themed glyph stands in for real previews,
///  which are out of this prototype's scope.
const PREVIEW_ICON_SIZE: i32 = 96;

/// What:
///  build a directory-listing pane from `snapshot`,
///  calling `on_activate(entry, force_dup)`
///       when a row is single-clicked or Enter-activated,
///  and `on_close` on the header button.
/// Why:
///  identical interaction contract to the original so the boundary tests drive both apps with
///      the same key sequences.
pub(crate) fn build_listing_pane<A, C>(
    snapshot: &DirectorySnapshot,
    on_activate: A,
    on_close: C,
) -> GtkBox
where
    A: Fn(&FileEntry, bool) + 'static,
    C: Fn() + 'static,
{
    let store = ListStore::new::<BoxedAnyObject>();
    for entry in &snapshot.entries {
        store.append(&BoxedAnyObject::new(entry.clone()));
    }
    let selection = SingleSelection::new(Some(store));
    let has_rows = selection.n_items() > 0;
    let list = ListView::new(Some(selection), Some(build_row_factory()));
    list.set_single_click_activate(true);
    if has_rows {
        // Initialize the keyboard cursor on the first row; without this GTK leaves the cursor
        // unset until the first arrow key, so a bare Enter would activate nothing even though
        // the row renders as selected.
        list.scroll_to(0, gtk4::ListScrollFlags::FOCUS | gtk4::ListScrollFlags::SELECT, None);
    }
    install_force_duplicate_tracking(&list, on_activate);

    let scrolled = ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .hexpand(true)
        .build();
    scrolled.add_css_class("fm-list");
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("fm-pane");
    container.append(&build_pane_header(&snapshot.path.display().to_string(), on_close));
    container.append(&scrolled);
    return container
}

/// What:
///  wire row activation to `on_activate`,
///  tracking whether Ctrl was held via a capture-phase
///       click gesture and a key controller feeding a shared cell that activation reads and clears.
/// Why:
///  `connect_activate` carries no modifier state,
///  so the last pointer/key press before it
///      records Ctrl;
///  both controllers run before activation.
fn install_force_duplicate_tracking<A>(list: &ListView, on_activate: A)
where
    A: Fn(&FileEntry, bool) + 'static,
{
    let force_dup = Rc::new(Cell::new(false));
    let click = GestureClick::new();
    click.set_propagation_phase(PropagationPhase::Capture);
    let for_click = force_dup.clone();
    click.connect_pressed(move |gesture, _, _, _| {
        for_click.set(gesture.current_event_state().contains(ModifierType::CONTROL_MASK));
    });
    list.add_controller(click);

    let keys = EventControllerKey::new();
    let for_keys = force_dup.clone();
    keys.connect_key_pressed(move |_, key, _, state| {
        if matches!(key, Key::Return | Key::KP_Enter | Key::space) {
            for_keys.set(state.contains(ModifierType::CONTROL_MASK));
        }
        return Propagation::Proceed
    });
    list.add_controller(keys);

    list.connect_activate(move |activated_list, position| {
        let Some(model) = activated_list.model() else {
            return;
        };
        let Some(boxed) = model.item(position).and_downcast::<BoxedAnyObject>() else {
            return;
        };
        let entry = boxed.borrow::<FileEntry>().clone();
        on_activate(&entry, force_dup.replace(false));
    });
}

/// What:
///  build a pane header:
///  the pane's title path (ellipsized) beside a close button.
/// Why:
///  the close button is the explicit-close lifecycle trigger;
///  shared by listing and preview
///      panes.
pub(crate) fn build_pane_header<C>(path: &str, on_close: C) -> GtkBox
where
    C: Fn() + 'static,
{
    let title = Label::builder()
        .label(path)
        .xalign(0.0)
        .hexpand(true)
        .ellipsize(EllipsizeMode::Middle)
        .build();
    let close = Button::from_icon_name("window-close-symbolic");
    close.set_has_frame(false);
    // Keep the button pointer-only: were it focusable, initial keyboard focus would land on it
    // (it is the pane's first focusable child) and Enter would close the pane instead of
    // activating the selected list row.
    close.set_focusable(false);
    close.connect_clicked(move |_| on_close());
    let header = GtkBox::new(Orientation::Horizontal, ROW_SPACING);
    header.add_css_class("fm-header");
    header.append(&title);
    header.append(&close);
    return header
}

/// What:
///  build a preview pane for `path`:
///  a header (path + close) over a typed icon and filename.
/// Why:
///  previews exist so the model's preview-vs-directory dedup semantics stay exercised;
///  real
///      thumbnails are the original's concern,
///  not this layout prototype's.
pub(crate) fn build_preview_pane<C>(path: &Path, on_close: C) -> GtkBox
where
    C: Fn() + 'static,
{
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("fm-pane");
    container.append(&build_pane_header(&path.display().to_string(), on_close));
    let body = GtkBox::new(Orientation::Vertical, 0);
    body.set_vexpand(true);
    let icon = Image::from_icon_name("text-x-generic");
    icon.set_pixel_size(PREVIEW_ICON_SIZE);
    icon.set_vexpand(true);
    body.append(&icon);
    body.append(&Label::new(path.file_name().and_then(|name| return name.to_str())));
    container.append(&body);
    return container
}

/// What:
///  build the factory that creates and binds one row (icon + name label).
/// Why:
///  `setup` builds an empty row once per realized slot;
///  `bind` fills it from the row's boxed
///      `FileEntry`,
///  so only visible rows ever touch a `FileEntry`.
fn build_row_factory() -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(|_, item| {
        let list_item = item.downcast_ref::<ListItem>().expect("list item");
        let row = GtkBox::new(Orientation::Horizontal, ROW_SPACING);
        row.append(&Image::new());
        row.append(&Label::builder().xalign(0.0).build());
        list_item.set_child(Some(&row));
    });
    factory.connect_bind(|_, item| {
        let list_item = item.downcast_ref::<ListItem>().expect("list item");
        let boxed = list_item.item().and_downcast::<BoxedAnyObject>().expect("boxed entry");
        let entry = boxed.borrow::<FileEntry>();
        let row = list_item.child().and_downcast::<GtkBox>().expect("row box");
        let icon = row.first_child().and_downcast::<Image>().expect("row icon");
        let label = icon
            .next_sibling()
            .and_downcast::<Label>()
            .expect("row label");
        icon.set_icon_name(Some(icon_name(entry.kind)));
        label.set_text(&entry.name);
    });
    return factory
}

/// What:
///  map an `EntryKind` to a freedesktop icon-theme name.
/// Why:
///  uses the OS icon theme for a real,
///  cheap glyph per kind.
fn icon_name(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Directory => return "folder",
        EntryKind::File => return "text-x-generic",
        EntryKind::Symlink => return "emblem-symbolic-link",
    }
}
