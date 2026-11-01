//! File drag-and-drop wiring.
//!
//! Inbound (OS file manager -> app) uses a native `GtkDropTarget` over `GdkFileList`,
//!  recovering
//! the path (on macOS GDK percent-encodes the URI scheme colon,
//!  so an unescape is needed).
//!  Outbound
//! (app -> OS file manager) is native on Wayland via `GtkDragSource`,
//!  but Windows and macOS need
//! native shims (`win_drag`,
//!  `mac_drag`) because their GDK backends do not deliver files to the OS
//! file manager;
//!  those are triggered from a `GtkGestureDrag`.
//!  See the `gtk4-*-file-drag` /
//! `gtk4-macos-file-dnd` troubleshooting docs.

/// What:
///  imports the borrowed and owned path types.
/// Why:
///  outbound takes a path by reference;
///  inbound recovery returns an owned `PathBuf`.
use std::path::{Path, PathBuf};

/// What:
///  imports the GTK widget-extension traits (`add_controller`,
///  file/uri helpers).
/// Why:
///  drop targets and drag sources are added as controllers via prelude traits.
use gtk4::prelude::*;
/// What:
///  imports the drag-action flags and the file-list content type.
/// Why:
///  the drop target accepts a `FileList` with a copy action.
use gtk4::gdk::{DragAction, FileList};
/// What:
///  imports the gio module for its `File` type.
/// Why:
///  dropped and dragged files are `gio::File`s (path/uri conversions).
use gtk4::gio;
/// What:
///  imports the glib module and its `Value` wrapper.
/// Why:
///  the drop callback receives the payload as a `glib::Value`,
///  and URI recovery uses `glib::Uri`.
use gtk4::glib::{self, Value};
/// What:
///  imports the drop-target and base widget types.
/// Why:
///  inbound attaches a `DropTarget`;
///  outbound takes any `Widget`.
use gtk4::{DropTarget, Widget};

/// What:
///  imports the Wayland outbound-drag types (drag source,
///  content provider,
///  byte buffer).
/// Why:
///  the Linux path hands a `text/uri-list` content provider to a `GtkDragSource`;
///  unused on
///      Windows/macOS,
///  which take the shim path instead.
#[cfg(all(not(windows), not(target_os = "macos")))]
use gtk4::{DragSource, gdk::ContentProvider, glib::Bytes};

/// What:
///  imports the drag gesture that triggers the native outbound shims.
/// Why:
///  on Windows/macOS a `GtkGestureDrag`'s drag-begin starts the OLE/AppKit drag.
#[cfg(any(windows, target_os = "macos"))]
use gtk4::GestureDrag;

/// What:
///  attach a native file drop target to `widget` that logs each dropped file's path.
/// Why:
///  proves inbound native DnD over `wl_data_device`/OLE;
///  the copy-into-a-directory file
///      operation is a later milestone,
///  so this accepts and records rather than moves files.
pub(crate) fn install_drop_target(widget: &impl IsA<Widget>) {
    let drop = DropTarget::new(FileList::static_type(), DragAction::COPY);
    drop.connect_drop(|_, value, _, _| return handle_drop(value));
    widget.add_controller(drop);
}

/// What:
///  handle a dropped `GdkFileList` value,
///  logging each file's recovered path and URI.
/// Why:
///  returns whether the drop was accepted;
///  a non-file-list payload is refused.
fn handle_drop(value: &Value) -> bool {
    match value.get::<FileList>() {
        Ok(files) => {
            for file in files.files() {
                tracing::info!(path = ?recover_path(&file), uri = %file.uri(), "inbound file drop");
            }
            return true
        }
        Err(error) => {
            tracing::warn!(%error, "drop value was not a file list");
            return false
        }
    }
}

/// What:
///  recover a dropped file's filesystem path,
///  working around the macOS URI encoding.
/// Why:
///  `g_file_get_path` returns `None` on macOS because GDK percent-encodes the scheme colon
///      (`file%3A///...`);
///  unescaping the URI and rebuilding the file recovers the path.
fn recover_path(file: &gio::File) -> Option<PathBuf> {
    if let Some(path) = file.path() {
        return Some(path);
    }
    let fixed = glib::Uri::unescape_string(&file.uri(), None)?;
    return gio::File::for_uri(&fixed).path()
}

/// What:
///  make `widget` a drag source that offers `path` to the OS file manager (Wayland native
///       path:
///  a `text/uri-list` content provider on a `GtkDragSource`).
/// Why:
///  Wayland delivers files natively;
///  Windows and macOS take the shim variants below.
#[cfg(all(not(windows), not(target_os = "macos")))]
pub(crate) fn install_file_drag(widget: &impl IsA<Widget>, path: &Path) {
    let uri = gio::File::for_path(path).uri();
    let payload = format!("{uri}\r\n");
    let provider = ContentProvider::for_bytes("text/uri-list", &Bytes::from(payload.as_bytes()));
    let source = DragSource::builder()
        .actions(DragAction::COPY)
        .content(&provider)
        .build();
    widget.add_controller(source);
}

/// What:
///  make `widget` start a native Win32 OLE drag of `path` (Windows shim path).
/// Why:
///  GDK's Win32 drag source cannot deliver files to Explorer,
///  so a `GtkGestureDrag` triggers
///      the OLE shim on drag-begin.
#[cfg(windows)]
pub(crate) fn install_file_drag(widget: &impl IsA<Widget>, path: &Path) {
    let gesture = GestureDrag::new();
    let path = path.to_path_buf();
    gesture.connect_drag_begin(move |_, _, _| {
        crate::win_drag::start_file_drag(&[path.as_path()]);
    });
    widget.add_controller(gesture);
}

/// What:
///  make `widget` start a native AppKit drag of `path` (macOS shim path).
/// Why:
///  GDK's Quartz drag source cannot deliver files to Finder,
///  so a `GtkGestureDrag` triggers
///      the AppKit shim on drag-begin.
#[cfg(target_os = "macos")]
pub(crate) fn install_file_drag(widget: &impl IsA<Widget>, path: &Path) {
    let gesture = GestureDrag::new();
    let path = path.to_string_lossy().into_owned();
    gesture.connect_drag_begin(move |_, _, _| {
        crate::mac_drag::start_file_drag(&path);
    });
    widget.add_controller(gesture);
}
