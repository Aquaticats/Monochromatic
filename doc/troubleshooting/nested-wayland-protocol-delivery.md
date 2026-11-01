# Nested Wayland dispatch could leave client replies waiting for rendering

## Symptom

A native IDE probe connected to the nested Wayland socket but did not expose its MCP endpoint.
The hosted process had not reached UI construction.
A debugger backtrace showed
`wayland_client::Connection::roundtrip`
inside Winit's `registry_queue_init` and backend initialization.

This probe later became ready without a source change.
Its exact triggering condition remains unproven;
that recovery is not evidence of a fix.
It is separate from an earlier private D-Bus address timeout.

A deterministic wire-level regression then exposed a concrete delivery gap:
a core `wl_display.sync` request was dispatched,
but the response did not arrive without a rendering path flushing the outgoing buffer.
The failure was
`Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }`.

## Source boundary

The inspected wayland-server 0.31.13 `src/display.rs:28` documents that applications must regularly call
`flush_clients()` to write outgoing event buffers to sockets.
At lines 58 and 63,
request dispatch and flushing are distinct operations:

```rust
// wayland-server-0.31.13/src/display.rs
pub fn dispatch_clients(&mut self, state: &mut State) -> std::io::Result<usize> {
    self.backend.dispatch_all_clients(state)
}

pub fn flush_clients(&mut self) -> std::io::Result<()> {
    self.backend.flush(None)
}
```

Before the fix,
`package/nested-wayland-session/cli/src/state.rs` dispatched client requests,
while `src/app.rs` supplied an empty event-loop iteration callback.
Explicit `flush_clients()` calls existed in `src/render.rs` and `src/recorder.rs`.
Protocol progress therefore depended on entering those rendering/recording paths.

The compositor now calls `src/protocol_flush.rs::finish_dispatch` after every dispatch cycle:

```rust
// package/nested-wayland-session/cli/src/app.rs
.run(None, &mut state, |current| {
    crate::protocol_flush::finish_dispatch(&mut current.display_handle);
})
```

The helper flushes queued events and records an operation-specific warning on failure.
Rendering and recorder flushes remain valid but are no longer the only delivery boundary.

## Verification

### Failing control

`tests/protocol_flush.rs` creates a disposable Wayland display and Unix socket pair.
It encodes a real `wl_display.sync` request,
dispatches it,
and calls the production end-of-cycle policy.
With the extracted previous empty policy,
reading the callback event timed out.
The committed test was observed failing before the flush was added.
No host display,
clipboard,
or GPU frame is involved.

### Passing cases

After adding the flush,
`mise run //package/nested-wayland-session/cli:test:container`
reported 41 passing tests,
including the exact callback object,
opcode,
and message size in the reply.
The scoped Clippy task and release build also passed.
A cold native IDE startup with the rebuilt artifact reached its MCP endpoint
and completed the dark-mode partial-ligature clipboard/screenshot probe.
This confirms the new artifact works at the consumer boundary;
it does not retrospectively identify the original stall's trigger.

## Workaround and tradeoffs

The consumer-boundary fix is regular end-of-cycle flushing,
not forcing a continuous redraw or starting a recorder to advance protocol traffic.
The additional flush operation occurs once per event-loop iteration.
No new redraw scheduling or host-visibility policy is introduced.

## What does not establish the cause

The initial stall occurred before the IDE's markup,
source layout,
DPI binding,
or syntax worker was created.
It does not establish a font-rendering regression.
The eventual unchanged recovery does not identify whether parent-window visibility,
scheduling,
or another condition prevented a rendering callback.
The wire regression proves the protocol-delivery defect independently of that unresolved trigger.

## Upstream filing decision

- Fault ownership:
  our compositor did not provide an independent end-of-dispatch flush.
- Fixability:
  the application-level event-loop boundary owns the correction.
- Supported use:
  the installed wayland-server API explicitly requires regular flushing.
- Contribution policy:
  no upstream change is proposed.
- Maintainer intent:
  no upstream acceptance prediction is needed for this local correction.
- Prototype:
  the actual wire exchange was observed failing before the fix and passing afterward.

Nothing is filed upstream.
This is an integration defect in our compositor,
not an established defect in Wayland,
Winit,
or Slint.
