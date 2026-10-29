//! Native AppKit drag source for outbound file drag on macOS.
//!
//! GTK4's GDK macOS (Quartz) backend cannot hand a file out to Finder:
//!  a `GtkDragSource` that
//! offers a correct `text/uri-list` still bounces,
//!  because the backend never writes the file to
//! the `NSPasteboard` as an `NSURL`.
//!  Here we start a real AppKit dragging session from the GTK
//! window's `NSView`,
//!  with the file's `NSURL` as the pasteboard writer,
//!  so Finder accepts the
//! drop.
//!  Everything runs on the GTK main thread,
//!  which is also the AppKit main thread.
//!  Compiled
//! only on macOS.
//!  See doc/troubleshooting/gtk4-macos-file-dnd.md.

/// What:
///  imports the retained smart pointer for Objective-C objects.
/// Why:
///  the drag source is owned as a `Retained<DragSource>`.
use objc2::rc::Retained;
/// What:
///  imports the type-erased object and protocol-object wrappers.
/// Why:
///  the drag image is set as an `AnyObject`,
///  and protocol arguments use `ProtocolObject`.
use objc2::runtime::{AnyObject, ProtocolObject};
/// What:
///  imports the objc2 threading markers,
///  class-definition macro,
///  and message-send macro.
/// Why:
///  the drag source is a main-thread-only class defined with `define_class!` and initialised
///      with `msg_send!`.
use objc2::{AnyThread, MainThreadMarker, MainThreadOnly, define_class, msg_send};
/// What:
///  imports the AppKit types the dragging session is begun from.
/// Why:
///  the session is started on the key window's content view with a dragging item and source.
use objc2_app_kit::{
    NSApplication, NSDragOperation, NSDraggingContext, NSDraggingItem, NSDraggingSession,
    NSDraggingSource, NSImage,
};
/// What:
///  imports the Foundation object,
///  array,
///  geometry,
///  string,
///  and URL types.
/// Why:
///  the dragged file is an `NSURL`,
///  wrapped in an `NSArray` of dragging items with a frame.
use objc2_foundation::{
    NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL,
};

/// What:
///  side length in points of the placeholder drag image.
/// Why:
///  a small square gives the dragging session a visible cursor icon without shipping artwork.
const DRAG_ICON_SIZE: f64 = 32.0;

define_class!(
    /// Minimal `NSDraggingSource` that always allows a copy.
    ///  The dragging session needs a source
    /// object to answer the operation-mask query;
    ///  this one owns no state,
    ///  so it carries no ivars.
    /// `NSDraggingSource` is a main-thread-only protocol,
    ///  so the class is `MainThreadOnly`.
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

/// What:
///  constructor for the minimal drag source.
/// Why:
///  AppKit retains the source for the session's life,
///  so the handle may drop after starting it.
impl DragSource {
    /// Allocate and initialise a fresh drag source on the main thread.
    ///  AppKit retains it for the
    /// life of the dragging session,
    ///  so the returned handle may be dropped once the session has
    /// started.
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![Self::alloc(mtm), init] }
    }
}

/// What:
///  start a native AppKit dragging session that offers `path` to Finder as a file `NSURL`.
/// Why:
///  GTK4's macOS backend never writes the file to the pasteboard,
///  so `GtkDragSource` cannot
///      drop onto Finder;
///  an AppKit `NSDraggingSession` begun from the key window's view can.
///  Bails
///      with a warning when off the main thread or without a key window,
///  content view,
///  or event.
pub fn start_file_drag(path: &str) {
    let Some(mtm) = MainThreadMarker::new() else {
        tracing::warn!(path, "native macOS drag skipped: not on the main thread");
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let Some(window) = app.keyWindow() else {
        tracing::warn!(path, "native macOS drag skipped: no key window");
        return;
    };
    let Some(view) = window.contentView() else {
        tracing::warn!(path, "native macOS drag skipped: key window has no content view");
        return;
    };
    let Some(event) = app.currentEvent() else {
        tracing::warn!(path, "native macOS drag skipped: no current event");
        return;
    };

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
        tracing::info!(path, "started native macOS file drag session");
    }
}
