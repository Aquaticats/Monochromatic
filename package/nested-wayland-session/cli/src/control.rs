//! The Unix-socket control API.
//!
//! A dedicated thread owns the listening socket and does all blocking line I/O.
//!  For each
//! request line it parses a `Command`,
//!  forwards it to the main (event-loop) thread over a
//! calloop channel together with a one-shot reply channel,
//!  waits for the `Response`,
//!  and
//! writes it back.
//!  Executing on the main thread is required because screenshot readback
//! and input injection must touch the GL context and seat,
//!  which live there.

/// What:
///      Grouped `use` of the blocking I/O traits,
///  the Unix socket types,
///  paths,
///  the
///           one-shot reply channel,
///  and threads.
/// Why:
///       The control thread reads lines and writes responses;
///  the reply channel bridges
///           back from the main thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import net from "node:net"; // and a promise to await the main thread's reply
/// ```
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    sync::mpsc::{sync_channel, SyncSender},
};

/// What:
///      Grouped `use` of the calloop channel (cross-thread source) and loop handle.
/// Why:
///       Register the channel as an event source on the main loop.
use smithay::reexports::calloop::{
    channel::{channel, Event, Sender},
    LoopHandle,
};

/// What:
///      `use anyhow::{Context, Result};`.
///  Error helpers.
/// Why:
///       Socket binding and thread spawning return `Result`.
use anyhow::{Context, Result};

/// What:
///      `use tracing::warn;`.
///  Structured log macro.
/// Why:
///       Report accept errors,
///  and connection errors.
use tracing::warn;

/// What:
///      Grouped `use` of the protocol,
///  input,
///  keymap,
///  screenshot,
///  and state items.
/// Why:
///       `execute` dispatches parsed commands into these subsystems.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseCommand, formatResponse, Command, Response } from "./protocol"; ...
/// ```
use crate::{
    child, control_socket::bind_listener, dnd, encoder, input,
    protocol::{format_response, parse_command, Command, Response},
    recorder, screenshot,
    state::Compositor,
};

/// A command forwarded from the control thread to the main thread,
///  with a reply channel.
///
/// What:
///      `pub struct ControlRequest { pub command: Command, pub reply:
///           SyncSender<Response> }`.
///  `SyncSender<Response>` is the sending half of a
///           bounded one-shot channel the main thread answers on.
/// Why:
///       Carries both the work and the way to return its result across threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ControlRequest = { command: Command; reply: (r: Response) => void };
/// ```
pub struct ControlRequest {
    /// The parsed command to execute on the main thread.
    pub command: Command,
    /// One-shot channel the main thread sends the response back on.
    pub reply: SyncSender<Response>,
}

/// Bind the control socket,
///  register the channel source,
///  and spawn the control thread.
///
/// What:
///      `pub fn start(loop_handle: &LoopHandle<Compositor>, socket_path: &Path) ->
///           Result<()>`.
///  Borrows the loop handle (to insert the channel source) and the
///           socket path.
///  The loop-handle lifetime is elided,
///  matching `child.rs`.
/// Why:
///       One call from `run` wires the whole control API when a socket is requested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function start(loopHandle, socketPath): void { ... }
/// ```
pub fn start(loop_handle: &LoopHandle<Compositor>, socket_path: &Path) -> Result<()> {
    // What:     `let listener = bind_listener(socket_path)?;`. Create the listening socket.
    // Why:      The control thread accepts connections on it.
    let listener = bind_listener(socket_path)?;

    // What:     `let (sender, channel) = channel::<ControlRequest>();`. Create a calloop
    //           cross-thread channel: `sender` (moved to the control thread) and `channel`
    //           (the event source registered on the loop).
    // Why:      The bridge from the control thread to the main thread.
    let (sender, channel) = channel::<ControlRequest>();

    // What:     `loop_handle.insert_source(channel, |event, _, state| { ... }).map_err(...)?;`.
    //           Register the channel; the callback runs on the main thread for each message.
    // Why:      Execute forwarded commands where the GL context and seat live.
    loop_handle
        .insert_source(channel, |event, _, state: &mut Compositor| {
            // What:     `if let Event::Msg(request) = event { ... }`. The channel yields
            //           `Event::Msg(T)` for each item (and `Event::Closed` at the end); we
            //           only act on messages.
            // Why:      Ignore the close notification; there is nothing to do on it.
            if let Event::Msg(request) = event {
                // What:     `let response = execute(state, request.command);`. Run the
                //           command against the live state on the main thread.
                // Why:      Produce the result to send back.
                // The timed commands (hold, drag, double click) answer later from the timeline.
                crate::control_run::run(state, request.command, request.reply);
            }
        })
        .map_err(|err| anyhow::anyhow!("registering the control channel source failed: {err}"))?;

    // What:     `std::thread::Builder::new().name("nws-control".to_string()).spawn(move ||
    //           control_thread(listener, sender)).context(...)?;`. Spawn the named control
    //           thread, moving the listener and sender into it.
    // Why:      Keep blocking socket I/O off the event-loop thread.
    std::thread::Builder::new()
        .name("nws-control".to_string())
        .spawn(move || control_thread(listener, sender))
        .context("spawning the control thread")?;

    // What:     `Ok(())`. Success.
    // Why:      Signal the control API is up.
    return Ok(())
}

/// The control thread body:
///  accept connections and handle each in turn.
///
/// What:
///      `fn control_thread(listener: UnixListener, sender: Sender<ControlRequest>)`.
///           Owns the listener and the channel sender.
/// Why:
///       Serialises control clients (a test harness connects one at a time),
///  keeping
///           the protocol simple.
fn control_thread(listener: UnixListener, sender: Sender<ControlRequest>) {
    // What:     `for incoming in listener.incoming() { ... }`. Iterate accepted connections;
    //           `incoming()` yields `Result<UnixStream>` blockingly.
    // Why:      Serve each control client.
    for incoming in listener.incoming() {
        // What:     `match incoming { Ok(stream) => ..., Err(err) => { warn!; break; } }`.
        //           Handle a connection or log-and-stop on an accept error.
        // Why:      Keep serving until a fatal accept error.
        match incoming {
            Ok(stream) => {
                // What:     `if let Err(err) = handle_connection(stream, &sender) { warn!(...);
                //           }`. Serve the connection, logging any per-connection error.
                // Why:      One client's I/O error should not kill the control thread.
                if let Err(err) = handle_connection(stream, &sender) {
                    warn!("control connection ended with error: {err}");
                }
            }
            Err(err) => {
                // What:     `warn!(...); break;`. Log and stop accepting.
                // Why:      A listener-level error is not recoverable.
                warn!("control accept failed: {err}");
                break;
            }
        }
    }
}

/// Handle one control connection:
///  read request lines and write response lines.
///
/// What:
///      `fn handle_connection(stream: UnixStream, sender: &Sender<ControlRequest>) ->
///           Result<()>`.
///  Reads until the client closes the connection.
/// Why:
///       Implements the line-per-request,
///  line-per-response loop for one client.
fn handle_connection(stream: UnixStream, sender: &Sender<ControlRequest>) -> Result<()> {
    // What:     `let reader_half = stream.try_clone().context(...)?;`. Duplicate the socket
    //           fd so one half reads while the other writes.
    // Why:      `BufReader` consumes its reader; we still need to write responses.
    let reader_half = stream.try_clone().context("cloning the control stream")?;

    // What:     `let mut writer = stream;`. Keep the original for writing.
    // Why:      Write responses on it.
    let mut writer = stream;

    // What:     `let reader = BufReader::new(reader_half);`. Buffered line reader over the
    //           read half.
    // Why:      `.lines()` needs a buffered reader.
    let reader = BufReader::new(reader_half);

    // What:     `for line in reader.lines() { ... }`. Iterate the connection's lines;
    //           `lines()` yields `Result<String>` per line, ending at EOF.
    // Why:      One request per line.
    for line_result in reader.lines() {
        // What:     context adds a diagnostic before ? unwraps the read result.
        // Why:      Keep the I/O result distinct from the decoded control line.
        let line = line_result.context("reading a control line")?;

        // What:     `let response = dispatch_line(&line, sender);`. Parse and execute the
        //           line (or produce an error response).
        // Why:      Turn a request line into a response.
        let response = dispatch_line(&line, sender);

        // What:     `let text = format_response(&response);`. Render the response to its wire
        //           form.
        // Why:      Prepare the bytes to write.
        let text = format_response(&response);

        // What:     `writer.write_all(text.as_bytes()).context(...)?;` then a newline, then
        //           `writer.flush()`. Write the response line and push it out.
        // Why:      Deliver the response promptly so the client can proceed.
        writer
            .write_all(text.as_bytes())
            .context("writing a control response")?;
        writer.write_all(b"\n").context("writing a control newline")?;
        writer.flush().context("flushing a control response")?;
    }

    // What:     `Ok(())`. The client closed the connection normally.
    // Why:      Signal clean end-of-connection.
    return Ok(())
}

/// Parse one line and,
///  if valid,
///  run it on the main thread and await its response.
///
/// What:
///      `fn dispatch_line(line: &str, sender: &Sender<ControlRequest>) -> Response`.
///           Always returns a `Response` (parse errors become `Err` responses).
/// Why:
///       Bridge the control thread to the main thread for one command.
fn dispatch_line(line: &str, sender: &Sender<ControlRequest>) -> Response {
    // What:     `let command = match parse_command(line) { Ok(c) => c, Err(message) =>
    //           return Response::Err(message) };`. Parse; a parse error short-circuits to an
    //           error response.
    // Why:      Do not bother the main thread with malformed input.
    let command = match parse_command(line) {
        Ok(command) => command,
        Err(message) => return Response::Err(message),
    };

    // What:     `let (reply_tx, reply_rx) = sync_channel::<Response>(1);`. A bounded one-shot
    //           channel for this command's reply.
    // Why:      The main thread answers on `reply_tx`; we wait on `reply_rx`.
    let (reply_tx, reply_rx) = sync_channel::<Response>(1);

    // What:     `if sender.send(ControlRequest { command, reply: reply_tx }).is_err() {
    //           return Response::Err(...); }`. Forward the request; a send error means the
    //           loop is gone.
    // Why:      Hand the work to the main thread, or report shutdown.
    if sender
        .send(ControlRequest {
            command,
            reply: reply_tx,
        })
        .is_err()
    {
        return Response::Err("compositor is shutting down".to_string());
    }

    // What:     `match reply_rx.recv() { Ok(response) => response, Err(_) =>
    //           Response::Err(...) }`. Block until the main thread replies; a receive error
    //           means the reply sender was dropped.
    // Why:      Return the executed command's result.
    match reply_rx.recv() {
        Ok(response) => return response,
        Err(_) => return Response::Err("compositor dropped the request".to_string()),
    }
}

/// Execute a command against the live compositor state (runs on the main thread).
///
/// What:
///      `pub fn execute(state: &mut Compositor, command: Command) -> Response`.
///           Dispatches each command variant to its subsystem and maps the outcome to a
///           `Response`.
/// Why:
///       The single place command semantics live;
///  called from the channel source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function execute(state, command): Response { ... }
/// ```
pub fn execute(state: &mut Compositor, command: Command) -> Response {
    // What:     `match command { ... }`. One arm per command variant.
    // Why:      Route to screenshot, input, resize, or lifecycle handling.
    match command {
        Command::Ping => return Response::Ok,
        Command::Screenshot(path) => {
            // What:     `match screenshot::capture(state, &path) { Ok(()) => Response::Ok,
            //           Err(err) => Response::Err(format!("{err:#}")) }`. Capture and map the
            //           result. `{err:#}` formats the anyhow error with its full context chain.
            // Why:      Report a screenshot failure with its cause.
            match screenshot::capture(state, &path) {
                Ok(()) => return Response::Ok,
                Err(err) => return Response::Err(format!("{err:#}")),
            }
        }
        Command::Click { x, y, button, .. } => {
            // What:     `input::click(state, x, y, button);`. Inject the click. A double click or
            //           a click with modifiers never gets here: the timeline plans it.
            // Why:      Perform the requested click.
            input::click(state, x, y, button);
            return Response::Ok
        }
        Command::Wheel { x, y, horizontal, vertical, .. } => {
            match crate::input_wheel::wheel(state, x, y, horizontal, vertical) {
                Ok(()) => return Response::Ok,
                Err(error) => return Response::Err(format!("{error:#}")),
            }
        }
        Command::Drag { .. } | Command::Scroll { .. } => {
            return Response::Err("timed pointer commands run through the timeline".to_string())
        }
        Command::Key { name, action } => {
            // What:     `parse_chord(&name)` resolves every key of the chord; the first unknown
            //           name fails the whole command before any key is pressed.
            // Why:      Only inject keys we can map to an evdev code.
            match crate::key_chord::parse_chord(&name) {
                Ok(chord) => {
                    match crate::input_key::apply(state, &chord, action) {
                        Ok(()) => return Response::Ok,
                        Err(error) => return Response::Err(format!("{error:#}")),
                    }
                }
                Err(message) => return Response::Err(message),
            }
        }
        Command::Type(text) => {
            // What:     `input_text::type_text(state, &text)`. Type the run of text, switching to
            //           spare Unicode keys for characters the layout lacks.
            // Why:      Perform the requested typing.
            match crate::input_text::type_text(state, &text) {
                Ok(()) => return Response::Ok,
                Err(error) => return Response::Err(format!("{error:#}")),
            }
        }
        Command::RepeatInfo(change) => return crate::control_session::repeat_info(state, change),
        Command::Focus => return crate::window_list::focus_response(state),
        Command::Windows => return crate::window_list::windows_response(state),
        Command::CloseWindow(index) => return crate::window_list::close_response(state, index),
        Command::Cursor => return crate::control_session::cursor(state),
        Command::Gpu => return crate::control_session::gpu(state),
        Command::Accessibility => return crate::a11y_tree::response(state),
        Command::Display => {
            return Response::OkWith(state.socket_name.to_string_lossy().into_owned())
        }
        Command::Resize { width, height } => {
            // What:     `screen::resize(state, width, height)`. Apply the new logical size.
            // Why:      Change the nested screen size at once, keeping the output scale.
            return crate::screen::resize(state, width, height);
        }
        Command::Scale(scale) => {
            // What:     `screen::switch_scale(state, scale)`. Apply the new output scale.
            // Why:      Keep the logical size so the client sees a scale change, not a resize.
            return crate::screen::switch_scale(state, scale);
        }
        Command::DropFile { paths, x, y } => {
            // What:     `match dnd::drop_file(state, &paths, x, y) { Ok(()) => Response::Ok,
            //           Err(err) => Response::Err(format!("{err:#}")) }`. Originate the
            //           compositor-side drag; map the outcome. `{err:#}` renders the anyhow
            //           context chain.
            // Why:      Report a drop-file setup failure (missing file, unmapped app) with its
            //           cause; the drop itself completes asynchronously on the release timer.
            match dnd::drop_file(state, &paths, x, y) {
                Ok(()) => return Response::Ok,
                Err(err) => return Response::Err(format!("{err:#}")),
            }
        }
        Command::Record { dir, fps, format } => {
            // What:     `match encoder::Format::parse(&format) { Ok(fmt) => ..., Err(message)
            //           => Response::Err(message) }`. Validate the format name first.
            // Why:      Reject an unknown format before touching the recorder.
            match encoder::Format::parse(&format) {
                Ok(fmt) => {
                    // What:     `match recorder::start(state, dir, fps, fmt) { Ok(()) =>
                    //           Response::Ok, Err(err) => Response::Err(format!("{err:#}")) }`.
                    //           Start recording; map the outcome.
                    // Why:      Begin the capture, reporting any setup failure.
                    match recorder::start(state, dir, fps, fmt) {
                        Ok(()) => return Response::Ok,
                        Err(err) => return Response::Err(format!("{err:#}")),
                    }
                }
                Err(message) => return Response::Err(message),
            }
        }
        Command::RecordStop => {
            // What:     `match recorder::stop(state) { Some(stats) => Response::OkWith(...),
            //           None => Response::Err("not recording") }`. Stop and report the
            //           measured statistics as the response payload.
            // Why:      Make the achieved fps / drop count observable to the caller.
            match recorder::stop(state) {
                Some(stats) => return Response::OkWith(format!(
                    "captured={} dropped={} failures={} seconds={:.3} fps={:.1}",
                    stats.captured, stats.dropped, stats.failures, stats.seconds, stats.fps
                )),
                None => return Response::Err("not recording".to_string()),
            }
        }
        Command::ColorScheme(preference) => {
            // What:     `.as_ref()` turns the owned optional portal into an optional borrow,
            //           so the state keeps owning it.
            // Why:      Only the private portal started by `--color-scheme` is ever switched;
            //           without one the helper refuses instead of reaching another bus.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return switchColorScheme(state.appearancePortal, preference);
            // ```
            return crate::control_color_scheme::switch(state.appearance_portal.as_ref(), preference);
        }
        Command::AccentColor(color) => {
            return crate::control_accent::switch(state.appearance_portal.as_ref(), color);
        }
        Command::Notification(verb) => {
            return crate::control_notification::run(state.notifications.as_ref(), &verb);
        }
        Command::Job(verb) => {
            return crate::control_job::run(state.notifications.as_ref(), &verb);
        }
        Command::Move { x, y } => {
            // What:     `input_pointer::move_pointer(state, x, y)`. Moves the pointer with no
            //           button, mapping a failure to an `err` line.
            // Why:      Hover is the only effect; the hosted client sees enter and motion events.
            match crate::input_pointer::move_pointer(state, x, y) {
                Ok(()) => return Response::Ok,
                Err(err) => return Response::Err(format!("{err:#}")),
            }
        }
        Command::Button { action, button } => {
            // What:     `input_pointer::button(state, action, button)`. Presses or releases at
            //           the current pointer position; a repeated state answers `err`.
            // Why:      Lets a test hold a press across a pause and a movement.
            match crate::input_pointer::button(state, action, button) {
                Ok(()) => return Response::Ok,
                Err(err) => return Response::Err(format!("{err:#}")),
            }
        }
        Command::Title => return crate::control_window::title_response(state),
        Command::AppId => return crate::control_window::app_id_response(state),
        Command::Launcher => {
            // What:     `control_launcher::latest(state.launcher.as_ref(), state.start_time)`.
            //           Reads the newest update from the listener, or refuses without a bus.
            // Why:      The start time is the clock every update is stamped with.
            return crate::control_launcher::latest(state.launcher.as_ref(), state.start_time);
        }
        Command::LauncherLog { count } => {
            return crate::control_launcher::log(state.launcher.as_ref(), count);
        }
        Command::LauncherClear => {
            return crate::control_launcher::clear(state.launcher.as_ref());
        }
        Command::Quit => {
            // Ask hosted client to close before tearing down its Wayland connection.
            child::request_hosted_client_shutdown(state);
            return Response::Ok;
        }
    }
}
