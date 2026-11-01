//! dmabuf handler:
//!  import the client's GPU buffers into our GLES renderer.
//!
//! The hosted app (a Slint/femtovg GPU client) presents through `zwp_linux_dmabuf_v1`,
//! handing the compositor GPU-memory buffers rather than shared-memory ones.
//!  This
//! handler imports each such buffer into the winit backend's GLES renderer so it can
//! be composited,
//!  which is the entire reason the fixture runs a real GPU renderer.

/// What:
///      Grouped `use` of the dmabuf types,
///  the `ImportDma` trait,
///  and the delegate
///           macro.
///  `Dmabuf` is the imported buffer;
///  `DmabufGlobal`/`DmabufState`/
///           `ImportNotifier` are the protocol plumbing;
///  `ImportDma` is the trait that
///           adds `import_dmabuf` to the renderer;
///  `delegate_dmabuf!` wires dispatch.
/// Why:
///       Everything the handler impl references.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DmabufHandler, ImportDma, ... } from "smithay";
/// ```
use smithay::{
    backend::{allocator::dmabuf::Dmabuf, renderer::ImportDma},
    delegate_dmabuf,
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
};

/// What:
///      `use crate::state::Compositor;`.
///  Our state type.
/// Why:
///       The handler is `impl DmabufHandler for Compositor`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Compositor } from "../state";
/// ```
use crate::state::Compositor;

/// Implement the dmabuf import handler.
///
/// What:
///      `impl DmabufHandler for Compositor`.
///  Exposes the dmabuf state and reacts
///           to each buffer the client offers.
/// Why:
///       Turns a client GPU buffer into a texture the renderer can composite.
impl DmabufHandler for Compositor {
    /// What:
    ///      `fn dmabuf_state(&mut self) -> &mut DmabufState`.
    ///  Mutable accessor.
    /// Why:
    ///       Smithay mutates dmabuf bookkeeping through it.
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        // What:     `&mut self.dmabuf_state`. Mutable borrow (tail expression).
        // Why:      Return the dmabuf state.
        return &mut self.dmabuf_state
    }

    /// What:
    ///      `fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf,
    ///           notifier: ImportNotifier)`.
    ///  Called when the client submits a dmabuf.
    ///           `_global` (which dmabuf global) is ignored;
    ///  `dmabuf` is the buffer;
    ///           `notifier` is a one-shot channel to report success or failure back to
    ///           the client.
    /// Why:
    ///       Import the buffer into the renderer and tell the client whether it
    ///           worked,
    ///  so it knows the buffer is usable.
    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf, notifier: ImportNotifier) {
        // What:     `if self.backend.renderer().import_dmabuf(&dmabuf, None).is_ok() { ... }`.
        //           `self.backend.renderer()` borrows the GLES renderer mutably;
        //           `import_dmabuf(&dmabuf, None)` (from the `ImportDma` trait) uploads
        //           the buffer as a texture, returning `Result<_, _>`; `.is_ok()` is
        //           `true` on success. `&dmabuf` lends the buffer read-only; `None` means
        //           "no damage region hint".
        // Why:      A successful import means the buffer can be composited.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const ok = tryImportDmabuf(backend.renderer(), dmabuf);
        // if (ok) notifier.successful(); else notifier.failed();
        // ```
        if self.backend.renderer().import_dmabuf(&dmabuf, None).is_ok() {
            // What:     `let _ = notifier.successful::<Compositor>();`. Signal success to
            //           the client. `successful` returns a `Result` we discard with
            //           `let _ =` (a failure to notify only means the client already went
            //           away). The `::<Compositor>` turbofish names the state type.
            // Why:      Tell the client its buffer is accepted.
            let _ = notifier.successful::<Compositor>();
        } else {
            // What:     `notifier.failed();`. Signal that the import failed.
            // Why:      Tell the client to fall back (e.g. renegotiate formats).
            notifier.failed();
        }
    }
}

// What:     `delegate_dmabuf!(Compositor);`. Generate the `zwp_linux_dmabuf_v1`
//           dispatch glue for our handler.
// Why:      Wire the dmabuf protocol requests to the handler above.
delegate_dmabuf!(Compositor);
