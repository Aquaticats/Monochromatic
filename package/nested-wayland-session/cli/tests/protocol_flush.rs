//! A real Wayland sync request must complete without a redraw or recorder tick.

/// Use the same cycle-end policy called by the compositor event loop.
use nested_wayland_session::protocol_flush::finish_dispatch;
/// The protocol display can process core sync requests without a GUI or custom globals.
use smithay::reexports::wayland_server::Display;
/// Socket pairs make the protocol exchange disposable and independent of the host desktop.
use std::{io::{Read, Write}, os::unix::net::UnixStream, sync::Arc, time::Duration};

/// Prove actual client-visible delivery through the production dispatch-cycle boundary.
#[test]
fn sync_reply_is_delivered_without_rendering() {
    let mut display = Display::<()>::new().expect("isolated Wayland display");
    let mut handle = display.handle();
    let (server, mut client) = UnixStream::pair().expect("private protocol socket pair");
    // Unit ClientData is supported by wayland-backend; no desktop state is reachable.
    let _connection = handle.insert_client(server, Arc::new(())).expect("insert private client");
    client.set_read_timeout(Some(Duration::from_millis(200))).expect("bound reply wait");
    // What: Native-endian Wayland words encode wl_display.sync(callback object 2).
    // Why: This reaches the real outgoing event buffer rather than a mocked flush counter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // socket.write(encodeWayland({ object: 1, opcode: 0, callback: 2 }));
    // ```
    for word in [1u32, 12u32 << 16, 2u32] {
        client.write_all(&word.to_ne_bytes()).expect("send sync request");
    }
    display.dispatch_clients(&mut ()).expect("dispatch sync request");
    finish_dispatch(&mut handle);
    let mut response = [0u8; 12];
    client.read_exact(&mut response).expect("sync reply must arrive without any redraw");
    let object = u32::from_ne_bytes(response[0..4].try_into().expect("object word"));
    let header = u32::from_ne_bytes(response[4..8].try_into().expect("header word"));
    assert_eq!(object, 2);
    assert_eq!(header & 0xffff, 0, "wl_callback.done opcode");
    assert_eq!(header >> 16, 12, "complete callback event size");
}
