# Slint 1.18.1 clipboard tests can fall back outside a nested Wayland session

## Symptom

An IDE source fixture selected through Ctrl+A reported the expected source range,
but copying through Ctrl+C left the nested clipboard empty.
`WAYLAND_DISPLAY=wayland-1 wl-paste --list-types` printed `Nothing is copied` and exited 1.
Repeating after a native compositor click established focus but produced the same result.
These tests used disposable source text,
not a user's project contents.
The source fixture may have been copied to the host X11 clipboard instead.
No host clipboard readback was used to make a stronger claim.
A review suggestion to read or clear the host clipboard was rejected:
its previous and current human-owned contents are unknown.
Verification uses disposable nested sessions,
not a second mutation of shared host state.

## Root cause

The installed Slint 1.18.1 Winit backend uses Arboard 3.6.1.
In `i-slint-backend-winit-1.18.1/clipboard.rs:104`,
`create_clipboard()` explicitly describes its fallback:

```rust
// i-slint-backend-winit-1.18.1/clipboard.rs
// arboard selects Wayland (data-control protocol, with the "wayland" feature)
// or X11 at runtime; on compositors without wlr-data-control it falls back to
// the X11/XWayland clipboard, which such compositors keep in sync.
```

Arboard's `src/platform/linux/mod.rs:131` tries Wayland when `WAYLAND_DISPLAY` exists.
Its failure branch falls through to X11 at line 145:

```rust
// arboard-3.6.1/src/platform/linux/mod.rs
match wayland::Clipboard::new() {
    Ok(clipboard) => {
        trace!("Successfully initialized the Wayland data control clipboard.");
        return Ok(Self::WlDataControl(clipboard));
    }
    Err(e) => warn!(
        "Tried to initialize the wayland data control protocol clipboard, but failed. Falling back to the X11 clipboard protocol. The error was: {}",
        e
    ),
}
```

The failed Wayland attempt does not return.
The final constructor is:

```rust
// arboard-3.6.1/src/platform/linux/mod.rs:145
Ok(Self::X11(x11::Clipboard::new()?))
```

Before this change,
`package/nested-wayland-session/cli/src/child.rs` set the nested `WAYLAND_DISPLAY`
and removed `WAYLAND_SOCKET`,
but retained inherited `DISPLAY` and `XAUTHORITY`.
The compositor registered `wl_data_device_manager`,
not a data-control manager.
A regular data-device protocol is not enough for Arboard's separate clipboard connection.
The nested display and host X11 clipboard are not synchronized by this fixture.

The current child environment now explicitly removes both X11 variables:

```rust
// package/nested-wayland-session/cli/src/child.rs
command.env_remove("DISPLAY");
command.env_remove("XAUTHORITY");
```

`src/handler/clipboard.rs` registers Smithay 0.7.0's wlr and ext data-control implementations
against the nested display,
sharing its existing seat selection.
No host clipboard synchronization is added.
Primary selection remains unadvertised because its provider is not implemented.

## Verification

### Failing cases

- Before protocol support,
  Ctrl+A/C through the native Slint source view followed by nested `wl-paste --list-types`
  returned `Nothing is copied`.
- Repeating with a real nested compositor click before copying did not fix the empty clipboard.
  Missing native click focus was therefore not the established cause.
- The committed isolation regression test failed before the environment fix:
  `child_environment_removes_host_x11_fallback` expected `Some(None)` for `DISPLAY`
  but received `Some(Some(":host"))`.
  This uses a disposable `Command` builder,
  never the real parent environment.

### Passing cases

- `test:container` reports 40 passing tests after the environment fix.
  The previously failing isolation assertion passes.
- The release build and `lint:clippy:container` pass.
  Concurrently introduced repository shadowing checks also required distinguishing callback state names;
  no lint rule was relaxed.
- The rebuilt IDE's Ctrl+A/C transfers its exact source fixture through nested `wl-paste`.
- Pointer/keyboard selection followed by Ctrl+C transfers `猫` and decomposed `é` exactly.
- Typing and Ctrl+V do not change the read-only source's accessible value.
- `WAYLAND_DEBUG=client wl-paste --no-newline` observes both manager globals:
  `zwlr_data_control_manager_v1` version 2 and `ext_data_control_manager_v1` version 1.
  This installed `wl-paste` selects ext-data-control for the actual readback.

The native consumer probe is
`/tmp/monochromatic-ide-native-8rNGro/clipboard-probe.mjs`;
its inspected screenshot is
`/tmp/monochromatic-ide-native-8rNGro/minimal-dark.png`.
These are disposable local evidence,
not tracked assets.
The durable `inspect:clipboard` task adds standalone text/binary transfer,
clear,
and clean-source-exit checks;
the task passed UTF-8,
six-byte binary,
and 2 MiB binary cases.
It checks ordinary clearing and abrupt source death with SIGKILL,
then requires an empty clipboard and no source stderr.
A primary-selection probe also confirms the deliberately unsupported boundary.
A separate disposable nested session retained its selected `é` throughout these tests.

Version evidence from the generated Cargo lock:
Slint 1.18.1 checksum
`15e477d5ff6fec20909100ff8f7062432ebfbda7a62a0f37d0252aa126f38c83`;
Arboard 3.6.1 checksum
`0348a1c054491f4bfe6ab86a7b6ab1e44e45d899005de92f58b3df180b36ddaf`.

Run the native lifecycle check with
`mise run //package/nested-wayland-session/cli:inspect:clipboard`.
It launches its own compositor and asserts that the child has a different Wayland display
and no inherited X11 environment before invoking clipboard clients.
It requires `wl-copy` and `wl-paste` on the host.

The regression command is
`mise run //package/nested-wayland-session/cli:test:container`.
Its pre-fix run failed specifically at the new X11 isolation assertion.

## Workaround and tradeoffs

The implemented consumer-boundary fix is to register both clipboard-manager protocols
and remove X11 fallback variables from the hosted child.
Native IDE readback verifies the text path.
This deliberately makes the hosted application Wayland-only,
matching the nested compositor's purpose.
It does not provide host clipboard sharing,
primary selection,
or clipboard persistence after its owner exits.
Clipboard management is available to clients connected to the nested socket,
not to a separately bridged host selection.

## What does not work

Setting `WAYLAND_DISPLAY` alone does not guarantee clipboard isolation.
A library can use a separate protocol connection and fall back to inherited X11 credentials.

Adding a native click before Ctrl+C did not make the old compositor's clipboard readable.
Changing source selection or restoring a visible Copy button would not supply the missing protocol.

A successful copy callback is not transfer verification.
Read the selected text back through an independent client on the same nested socket.

## Upstream filing decision

- Fault ownership:
  this is a missing protocol and environment-isolation gap in our compositor fixture.
  The inspected toolkit/library explicitly implements fallback;
  no upstream defect is established.
- Fixability:
  the correction belongs at our compositor and child-launch boundaries.
- Supported use:
  Smithay 0.7.0 provides both protocol implementations and initialization examples.
- Contribution policy:
  no upstream contribution is proposed,
  so upstream policy research is not a prerequisite to this local correction.
- Maintainer intent:
  no upstream acceptance claim is made.
- Prototype:
  local protocol support and an observed-failing isolation regression pass,
  and independent native readback verifies source,
  CJK,
  and combining text.

`.out-of-scope/` was inspected and has no matching Slint exemption.
Nothing is filed upstream:
there is no established upstream bug or additive issue draft.
Do not open an upstream issue for the nested fixture's missing protocol.
