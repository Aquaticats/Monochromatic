# Nested Wayland quit must close the client before the compositor

## Symptom

After a successful screenshot,
the `quit` control command returns `ok` and the nested compositor exits with status zero,
but stderr contains client shutdown failures:

```txt
Io error: Broken pipe (os error 32)
ERROR winit::platform_impl::linux::wayland::event_loop:
Error dispatching event loop: other error during loop operation
Error: Error running winit event loop: Exit Failure: 1
```

The emitting toolkit is winit in the hosted client.
The top-level nested-session process can still report success because it stopped independently of the client.

## Cause

`package/nested-wayland-session/cli/src/control.rs` originally handled `Command::Quit` by immediately calling
`state.loop_signal.stop()`.
That ended the compositor event loop and destroyed the hosted client's Wayland connection while the client was running.
Winit correctly reported the unexpected broken transport as an event-loop failure.

Stopping a compositor first is not a graceful way to stop its client.
A zero fixture status does not make the client-side error harmless,
and filtering the log would hide the lifecycle defect.

## Fix

The quit handler now requests close on each hosted xdg toplevel with `ToplevelSurface::send_close()`.
The existing child-exit poll keeps the compositor alive until the application exits,
then stops the compositor event loop.
This ordering lets winit receive its normal close event while the Wayland transport remains valid.

A bounded fallback prevents an unresponsive client from hanging the fixture:

1.  `quit` sends each xdg-toplevel close event.
2.  A monotonic shutdown deadline is recorded.
3.  The existing child poll observes normal client exit and stops the compositor.
4.  If the client ignores close past the deadline,
    the poll force-stops it before compositor teardown.

## Verification

Run a real hosted GUI client,
request a screenshot,
then send `quit` over the control socket.
Capture the complete compositor and child stderr.
The run must exit without `Broken pipe`,
`Error dispatching event loop`,
or winit `Exit Failure` messages.

The deadline helper also has focused tests for absent,
future,
and reached deadlines.
