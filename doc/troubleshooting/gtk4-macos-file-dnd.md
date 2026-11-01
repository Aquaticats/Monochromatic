# GTK4 on macOS: file drag-and-drop needs handling (inbound URI quirk, outbound needs a shim)

Native file drag-and-drop between a gtk4-rs app and Finder on macOS does not work out of the box:
inbound drops deliver the file but `g_file_get_path` returns `None`,
 and outbound drags to Finder
are rejected.
 Measured on `m1` (macOS 26.5.2),
 gtk4-rs against Homebrew GTK 4.x,
 over RustDesk.
 The
drags do reach the app (the source and target callbacks fire),
 so this is GTK/macOS handling,
 not a
remote-input artifact.

## Inbound (Finder -> GTK): delivered, but the path needs recovering

A file dropped from Finder is received by a stock `GtkDropTarget` (`GdkFileList`) and the drop is
accepted,
 but each file's `path()` is `None`.
 Logging the URI shows why:

```txt
inbound file drop  path=None  uri=file%3A///Volumes/Data/Backup/user/focus.out.txt
```

`%3A` is a percent-encoded colon:
 GDK's macOS backend encoded the scheme separator,
 producing
`file%3A///...` instead of `file:///...`.
 That is not a valid `file:` URI,
 so `g_file_get_path`
returns `None`.
 It is not a permission/TCC issue:
 the full correct path is present in the URI.

Workaround (in the app):
 do not rely on `g_file_get_path` for dropped files on macOS.
 Unescape the
URI (or fix the scheme) and rebuild the `GFile`:

```rust
let uri = file.uri();                                  // "file%3A///Volumes/.../name"
let fixed = glib::Uri::unescape_string(&uri, None);    // -> "file:///Volumes/.../name"
let path = fixed.and_then(|u| gio::File::for_uri(&u).path());
```

## Outbound (GTK -> Finder): rejected; needs a native drag-source shim

A `GtkDragSource` that hands out a file starts (its `prepare` callback fires) but Finder rejects the
drop and the cursor bounces back.
 Tested two ways,
 both rejected:

- `GdkFileList` content (`ContentProvider::for_value`).
- `text/uri-list` content with a correctly-encoded `file:///Volumes/.../dragme.txt` URI
  (`ContentProvider::for_bytes`),
   verified in the log:

  ```txt
  outbound drag prepared (text/uri-list) uri=file:///Volumes/MacData/agent-spikes/dragme.txt
  ```

Since a correctly-encoded URI is also rejected,
 the problem is not the inbound colon-encoding bug:
GDK's macOS drag source does not present the file to Finder in a form it accepts for a drag-copy
(Finder wants `NSFilenamesPboardType` or an `NSFilePromiseProvider`,
 not just `public.file-url`).

Resolution (built and verified):
 a native AppKit drag source in Rust (`objc2`),
 analogous to the
Windows OLE shim in `gtk4-windows-outbound-file-drag.md`.
 Dragging the handle to a Finder folder
copies the file,
 and the source logs `started native macOS file drag session`.
 Wire it,
 like the
Windows shim,
 from a `GtkGestureDrag` `drag-begin` (guarded to macOS);
 keep `GtkDragSource` on
Linux/Wayland.

### Reference implementation (verified)

macOS-only dependencies (resolved to objc2 0.6.4,
 objc2-app-kit 0.3.2,
 objc2-foundation 0.3.2):

```toml
# Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-foundation = { version = "0.3", features = ["NSString", "NSURL", "NSArray", "NSGeometry"] }
objc2-app-kit = { version = "0.3", features = [
    "NSApplication", "NSResponder", "NSView", "NSWindow", "NSEvent",
    "NSDragging", "NSDraggingItem", "NSDraggingSession", "NSPasteboard", "NSImage",
] }
```

The drag (`mac_drag.rs`):

```rust
// package/desktop-app/file-manager-gtk/src/mac_drag.rs
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession,
    NSDraggingSource, NSImage,
};
use objc2_foundation::{
    NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL,
};

const DRAG_ICON_SIZE: f64 = 32.0;

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "MonochromaticGtkDragSource"]
    struct DragSource;

    unsafe impl NSObjectProtocol for DragSource {}

    unsafe impl NSDraggingSource for DragSource {
        #[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]
        fn source_operation_mask(
            &self,
            _session: &NSDraggingSession,
            _context: NSDraggingContext,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }
    }
);

impl DragSource {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::alloc(mtm), init] }
    }
}

pub fn start_file_drag(path: &str) {
    let Some(mtm) = MainThreadMarker::new() else { return };
    let app = NSApplication::sharedApplication(mtm);
    let Some(window) = app.keyWindow() else { return };
    let Some(view) = window.contentView() else { return };
    let Some(event) = app.currentEvent() else { return };

    let source = DragSource::new(mtm);
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        let item = NSDraggingItem::initWithPasteboardWriter(
            NSDraggingItem::alloc(),
            ProtocolObject::from_ref(&*url),
        );
        let size = NSSize::new(DRAG_ICON_SIZE, DRAG_ICON_SIZE);
        let frame = NSRect::new(NSPoint::new(0.0, 0.0), size);
        let image = NSImage::initWithSize(NSImage::alloc(), size);
        let contents: &AnyObject = &image;
        item.setDraggingFrame_contents(frame, Some(contents));

        let items = NSArray::from_slice(&[&*item]);
        let _session = view.beginDraggingSessionWithItems_event_source(
            &items,
            &event,
            ProtocolObject::from_ref(&*source),
        );
    }
}
```

objc2 0.6 / objc2-app-kit 0.3.2 gotchas (each cost a compile iteration):

- `NSObject` must be imported into the module even though it only appears inside
  `#[unsafe(super(NSObject))]`:
   `define_class!` resolves the superclass in the caller's scope,
   so
  an unimported `NSObject` fails with `E0425 cannot find type`.
- `NSDraggingSource` is a main-thread-only protocol (bound `NSObjectProtocol + MainThreadOnly`),
   so
  the class must be declared `#[thread_kind = MainThreadOnly]`;
   without it the error is a cryptic
  `E0271 ThreadKind == dyn MainThreadOnly` on the `unsafe impl` line.
   A `MainThreadOnly` class
  allocates with `MainThreadOnly::alloc(mtm)`,
   not `AnyThread::alloc()`.
- `NSURL: NSPasteboardWriting` conformance lives in objc2-app-kit (feature `NSPasteboard`),
   not
  objc2-foundation,
   and is invisible on the objc2-foundation `NSURL` page.
- `setDraggingFrame_contents` wants `Option<&AnyObject>`;
   widen explicitly
  (`let contents: &AnyObject = &image;`),
   since `Some(&*image)` will not infer the coercion.
- In `define_class!`,
   the method carries the full selector
  `#[unsafe(method(draggingSession:sourceOperationMaskForDraggingContext:))]`;
   all five
  `NSDraggingSource` methods are optional,
   so implementing only the mask method compiles.

## Cross-platform DnD picture

- Linux (Wayland,
   primary):
   full native DnD both directions,
   no workarounds.
- Windows:
   inbound works (stock `GtkDropTarget`);
   outbound needs a native Win32 OLE shim (built and
  verified).
  See `gtk4-windows-outbound-file-drag.md`.
- macOS:
   inbound works with an in-app URI-unescape;
   outbound works via a native `objc2` AppKit drag
  shim (built and verified;
   reference implementation above).

The pattern:
 GTK's non-Wayland backends need per-platform native drag sources for outbound file DnD
to the OS file manager.
 This does not change the GTK4 toolkit decision (it is the only option
satisfying the Wayland DnD hard constraint;
 the winit-based alternatives have no Wayland DnD at
all),
 but these shims are a known,
 bounded cost for the Windows and macOS targets.
