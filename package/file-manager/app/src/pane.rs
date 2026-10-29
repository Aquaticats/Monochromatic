//! Directory-listing pane:
//!  a virtualized `ListView` over a `DirectorySnapshot`'s entries.
//!
//! Only visible rows are realized (`GtkListView` virtualizes the model),
//!  so a large directory
//! stays cheap.
//!  Each row is a themed icon plus the entry name;
//!  a single click (or Enter) activates
//! a row,
//!  which the strip turns into a spawned child pane,
//!  and holding Ctrl forces a duplicate.
//!  A
//! close button in the header explicitly closes the pane.
//!  Real previews come with the thumbnail
//! milestone.

/// What:
///  imports the single-slot interior-mutability cell.
/// Why:
///  the Ctrl-at-activation flag is shared between the click/key controllers and activation.
use std::cell::Cell;
/// What:
///  imports the reference-counted pointer.
/// Why:
///  that shared Ctrl flag is held by three closures on the same pane.
use std::rc::Rc;
/// What:
///  imports the borrowed path type.
/// Why:
///  a preview pane takes the previewed file path by reference.
use std::path::Path;

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
///  the key handler returns `Propagation`.
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
    Orientation, Picture, PropagationPhase, ScrolledWindow, SignalListItemFactory, SingleSelection,
    Widget,
};

/// What:
///  imports debug-tint helpers.
/// Why:
///  debug runs need visible screenshot labels on pane subregions.
use crate::debug_tint;
/// What:
///  imports the thumbnail service and image-detection helper.
/// Why:
///  a preview pane requests an off-thread thumbnail for image files.
use crate::thumbs::{Thumbnails, is_image};
/// What:
///  imports the snapshot,
///  entry,
///  and kind domain types.
/// Why:
///  the pane renders a `DirectorySnapshot` of `FileEntry` rows,
///  choosing an icon per `EntryKind`.
use crate::types::{DirectorySnapshot, EntryKind, FileEntry};

/// What:
///  horizontal gap in pixels between a row's icon and its name label (and header items).
/// Why:
///  named so the one spacing value is not a bare magic literal.
const ROW_SPACING: i32 = 6;

/// What:
///  pixel size of the fallback icon shown for a non-image preview pane.
/// Why:
///  a large themed glyph stands in until richer previews (video,
///  PDF) arrive.
const PREVIEW_ICON_SIZE: i32 = 96;

/// What:
///  build a directory-listing pane from `snapshot`,
///  calling `on_activate(entry, force_dup)`
///       when a row is single-clicked or Enter-activated (`force_dup` true when Ctrl was held),
///  and
///       `on_close` when the header close button is pressed.
/// Why:
///  single-click-activate makes selecting a row the spawn trigger;
///  Ctrl forces a duplicate pane
///      (doc/planning/file-manager.md).
///  Arrow keys only move selection,
///  so browsing never spawns.
pub fn build_listing_pane<A, C>(snapshot: &DirectorySnapshot, on_activate: A, on_close: C) -> GtkBox
where
    A: Fn(&FileEntry, bool) + 'static,
    C: Fn() + 'static,
{
    let store = ListStore::new::<BoxedAnyObject>();
    for entry in &snapshot.entries {
        store.append(&BoxedAnyObject::new(entry.clone()));
    }
    let selection = SingleSelection::new(Some(store));
    let list = ListView::new(Some(selection), Some(build_row_factory()));
    list.set_single_click_activate(true);
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
///  both controllers run before activation (capture phase / key-pressed).
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
///  the title expands so the button
///      sits at the pane's right edge.
///  Shared by listing panes and preview panes.
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
    close.connect_clicked(move |_| on_close());
    let header = GtkBox::new(Orientation::Horizontal, ROW_SPACING);
    header.add_css_class("fm-header");
    header.append(&title);
    header.append(&close);
    return header
}

/// What:
///  build a preview pane for `path`:
///  a header (path + close) over a thumbnail (images) or a
///       typed icon (other files),
///  decoding off-thread through `thumbs`.
/// Why:
///  images get a real decoded preview from the bounded cache;
///  other files get a cheap OS-icon
///      stand-in.
///  `on_close` closes the pane.
pub(crate) fn build_preview_pane<C>(thumbs: &Thumbnails, path: &Path, on_close: C) -> GtkBox
where
    C: Fn() + 'static,
{
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("fm-pane");
    container.append(&build_pane_header(&path.display().to_string(), on_close));
    container.append(&build_preview_body(thumbs, path));
    return container
}

/// What:
///  build a preview pane's body:
///  an off-thread thumbnail `Picture` for an image,
///  or a large
///       typed icon plus filename for any other file.
/// Why:
///  only images request a decode;
///  the request deduplicates and caches,
///  so revisiting is cheap.
fn build_preview_body(thumbs: &Thumbnails, path: &Path) -> Widget {
    if is_image(path) {
        let picture = Picture::new();
        picture.set_can_shrink(true);
        picture.set_vexpand(true);
        picture.set_hexpand(true);
        thumbs.request(path, &picture);
        crate::dnd::install_file_drag(&picture, path);
        return debug_tint::wrap(
            &picture,
            debug_tint::B6P_PREVIEW_BODY,
            Some("image preview"),
        );
    }
    let body = GtkBox::new(Orientation::Vertical, 0);
    body.set_vexpand(true);
    let icon = Image::from_icon_name("text-x-generic");
    icon.set_pixel_size(PREVIEW_ICON_SIZE);
    icon.set_vexpand(true);
    body.append(&icon);
    body.append(&Label::new(path.file_name().and_then(|name| return name.to_str())));
    crate::dnd::install_file_drag(&body, path);
    return debug_tint::wrap(&body, debug_tint::B6P_PREVIEW_BODY, Some("fallback preview"))
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
///  cheap glyph per kind until the thumbnail milestone adds
///      per-file previews.
fn icon_name(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Directory => return "folder",
        EntryKind::File => return "text-x-generic",
        EntryKind::Symlink => return "emblem-symbolic-link",
    }
}
