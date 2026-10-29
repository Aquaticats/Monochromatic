//! Native Win32 OLE drag source for outbound file drag on Windows.
//!
//! GDK's Win32 backend leaves the file-to-"Shell IDList Array" conversion unimplemented,
//!  so a
//! `GtkDragSource` cannot deliver files to Explorer (see
//! doc/troubleshooting/gtk4-windows-outbound-file-drag.md).
//!  This drives the native OLE drag
//! directly:
//!  build shell PIDLs for the paths,
//!  wrap them in an `IDataObject`,
//!  and run `SHDoDragDrop`
//! on the GTK main thread (already an OLE STA because GDK calls `OleInitialize`).
//!  Compiled only on
//! Windows.

/// What:
///  imports the borrowed path type.
/// Why:
///  the drag takes the files to drag as paths.
use std::path::Path;

/// What:
///  imports the OLE data-object interface.
/// Why:
///  the shell item array binds to an `IDataObject` that `SHDoDragDrop` hands to the drop target.
use windows::Win32::System::Com::IDataObject;
/// What:
///  imports the copy drop-effect flag.
/// Why:
///  the drag advertises a copy operation.
use windows::Win32::System::Ole::DROPEFFECT_COPY;
/// What:
///  imports the shell item-id-list type.
/// Why:
///  each path becomes an `ITEMIDLIST` (PIDL) that is freed after the drag.
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
/// What:
///  imports the shell drag helpers and PIDL functions.
/// Why:
///  build PIDLs,
///  make a shell item array plus data object,
///  run the drag,
///  and free the PIDLs.
use windows::Win32::UI::Shell::{
    BHID_DataObject, ILCreateFromPathW, ILFree, SHCreateShellItemArrayFromIDLists, SHDoDragDrop,
};
/// What:
///  imports the wide-string type.
/// Why:
///  `ILCreateFromPathW` takes a UTF-16 path.
use windows::core::HSTRING;

/// What:
///  start a native OLE drag offering `paths` to Explorer as a copy.
/// Why:
///  entry point wired from a `GtkGestureDrag` on Windows;
///  logs and swallows OLE errors so a
///      failed drag never crashes the UI.
pub fn start_file_drag(paths: &[&Path]) {
    if let Err(error) = run_drag(paths) {
        tracing::error!(%error, "native OLE drag failed");
    }
}

/// What:
///  build the shell data object for `paths` and run the modal OLE drag.
/// Why:
///  `SHDoDragDrop` blocks on its own message-pumping loop until drop or cancel;
///  it must run on
///      the GTK main thread,
///  already an OLE STA.
///  Passing `None` for the drop source uses the shell
///      default,
///  and `IShellItemArray::BindToHandler` needs an annotated binding (two generics).
fn run_drag(paths: &[&Path]) -> windows::core::Result<()> {
    unsafe {
        let mut owned: Vec<*mut ITEMIDLIST> = Vec::new();
        for path in paths {
            let pidl = ILCreateFromPathW(&HSTRING::from(path.as_os_str()));
            if pidl.is_null() {
                tracing::warn!(?path, "ILCreateFromPathW returned null");
            } else {
                owned.push(pidl);
            }
        }
        if owned.is_empty() {
            return Ok(());
        }
        let pidls: Vec<*const ITEMIDLIST> =
            owned.iter().map(|pidl| *pidl as *const ITEMIDLIST).collect();
        let array = SHCreateShellItemArrayFromIDLists(&pidls)?;
        let data: IDataObject = array.BindToHandler(None, &BHID_DataObject)?;
        let effect = SHDoDragDrop(None, &data, None, DROPEFFECT_COPY)?;
        tracing::info!(effect = effect.0, "native OLE drag finished");
        for pidl in owned {
            ILFree(Some(pidl as *const ITEMIDLIST));
        }
        Ok(())
    }
}
