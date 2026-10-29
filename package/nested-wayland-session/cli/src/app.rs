//! Program orchestration:
//!  build the event loop,
//!  wire every source,
//!  spawn the client,
//! and run until it exits.
//!
//! `run` is the whole-program entry the binary calls.
//!  It stays deliberately thin:
//!  each
//! subsystem (backend,
//!  state,
//!  child,
//!  rendering,
//!  per-event handling) lives in its own
//! module,
//!  and this file only connects them and owns the event loop.

/// What:
///      Grouped `use` of the shared-memory import trait,
///  the winit event enum,
///  the
///           event loop,
///  and the display.
/// Why:
///       `run` and `handle_winit_event` reference these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ImportMemWl, WinitEvent, EventLoop, Display } from "smithay";
/// ```
use smithay::{
    backend::{renderer::ImportMemWl, winit::WinitEvent},
    reexports::{
        calloop::EventLoop,
        wayland_server::Display,
    },
};

/// What:
///      `use anyhow::{Context, Result};`.
///  Error helpers.
/// Why:
///       `run` returns `Result<i32>` and annotates setup failures.
use anyhow::{Context, Result};

/// What:
///      Grouped `use` of our own modules' items.
/// Why:
///       `run` calls into backend,
///  child,
///  state,
///  and the xdg-shell reconfigure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { initBackend } from "./backend"; import { spawnChild, ... } from "./child"; ...
/// ```
use crate::{
    appearance_portal::{AppearancePortal, PortalStart},
    backend::init_backend,
    backend_headless::init_headless_backend,
    backend_select::{select_backend, BackendKind, ParentEnvironment},
    child::{register_exit_poll, spawn_child},
    cli::Config,
    control,
    launcher_listen::LauncherLog,
    plasma_services::PlasmaServices,
    private_bus::PrivateBus,
    render::redraw,
    state::Compositor,
    systemd::Isolation,
};

/// Shares the one private bus between the portal,
///  the listener,
///  and the compositor state.
use std::sync::Arc;

/// Records which backend the session runs on.
use tracing::info;

/// Build everything,
///  spawn the hosted client,
///  and run the event loop to completion.
///
/// What:
///      `pub fn run(config: Config) -> Result<i32>`.
///  Consumes the parsed config,
///           returns the hosted client's exit code (or an error if setup failed).
/// Why:
///       The single top-level function the binary invokes;
///  returning the code lets
///           `main` propagate it to the shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(config: Config): number { ... }
/// ```
///
/// @example
/// ```ts
/// const code = run(parseArgs(argv));
/// ```
pub fn run(config: Config) -> Result<i32> {
    // What:     `let mut event_loop: EventLoop<Compositor> = EventLoop::try_new()
    //           .context("creating the calloop event loop")?;`. Creates the loop whose
    //           shared data type is `Compositor`. `mut` because sources are registered
    //           against it. `.context(msg)?` turns a failure into a clean error.
    // Why:      The single event loop drives Wayland clients, winit, the child poll, and
    //           (later) the control socket.
    let mut event_loop: EventLoop<'static, Compositor> =
        EventLoop::try_new().context("creating the calloop event loop")?;

    // What:     `let display: Display<Compositor> = Display::new().context(...)?;`. Creates
    //           the wayland-server display, parameterised by our state type.
    // Why:      Owns all client connections and protocol globals.
    let display: Display<Compositor> = Display::new().context("creating the Wayland display")?;

    // What:     `let display_handle = display.handle();`. A cheap clone of the display's
    //           control handle, taken before the display is moved into the state.
    // Why:      Backend init needs it to register the output and dmabuf globals.
    let display_handle = display.handle();

    // What:     `select_backend(config.headless, ..)` picks the backend: the nested winit window,
    //           unless `--headless` was given. Nested builds the winit backend and returns its
    //           event source; headless builds the surfaceless renderer and has no such source.
    // Why:      Set up the GPU rendering path before constructing the state. A missing parent
    //           display never selects headless; the nested start then fails naming the flag.
    let choice = select_backend(config.headless, ParentEnvironment::from_process());
    if choice.ignored_parent_display {
        info!("headless: a parent display is set and stays unused; no window appears on it");
    }
    let (pieces, winit) = match choice.kind {
        BackendKind::NestedWinit => {
            let (pieces, winit) = init_backend(&display_handle, &config)?;
            (pieces, Some(winit))
        }
        BackendKind::Headless => (init_headless_backend(&display_handle, &config)?, None),
    };

    // What:     `let mut state = Compositor::new(&mut event_loop, display, pieces);`.
    //           Construct the full state, consuming the display and backend pieces.
    // Why:      Produce the value the event loop carries.
    let mut state = Compositor::new(&mut event_loop, display, pieces)?;

    // What:     `let shm_formats = state.backend.renderer().shm_formats();`. Ask the GLES
    //           renderer which shared-memory formats it supports (method from the
    //           `ImportMemWl` trait). Borrows the backend only for this statement.
    // Why:      Advertise exactly those formats so any shm client (e.g. a cursor theme)
    //           uses a format the renderer can upload.
    let shm_formats = state.backend.renderer().shm_formats();

    // What:     `state.shm_state.update_formats(shm_formats);`. Store the format list.
    // Why:      Complete the shm advertisement.
    state.shm_state.update_formats(shm_formats);

    // What:     `let loop_handle = event_loop.handle();`. A handle for registering more
    //           sources.
    // Why:      Used to insert the winit source and the exit poll.
    let loop_handle = event_loop.handle();

    // What:     `loop_handle.insert_source(winit, |event, _, state| { handle_winit_event(
    //           event, state); }).map_err(|err| anyhow::anyhow!(...))?;`. Registers the
    //           winit event source. The closure receives each `WinitEvent`, ignored
    //           metadata (`_`), and `&mut Compositor`. `.map_err(...)?` converts a
    //           registration error into `anyhow`.
    // Why:      Route resize / redraw / close events into the compositor.
    if let Some(winit_source) = winit {
        loop_handle
            .insert_source(winit_source, |event, _, event_state: &mut Compositor| {
                handle_winit_event(event, event_state);
            })
            .map_err(|err| anyhow::anyhow!("registering the winit source failed: {err}"))?;
    }

    // What:     `if let Some(socket_path) = &config.control_socket { control::start(
    //           &loop_handle, socket_path)?; }`. When a control socket was requested, bind
    //           it and spawn the control thread; `?` propagates a bind failure.
    // Why:      Enable the screenshot/input/resize control API only when asked.
    if let Some(socket_path) = &config.control_socket {
        control::start(&loop_handle, socket_path)?;
    }

    // What:     `register_exit_poll(&loop_handle);`. Insert the periodic child-exit poll.
    // Why:      Stop the loop when the hosted client exits.
    register_exit_poll(&loop_handle);

    // What:     `let private_bus = if ... { Some(Arc::new(PrivateBus::start()?)) } else { None };`.
    //           Starts one private session bus when either option needs one.
    // Why:      The appearance portal and the taskbar listener share this bus,
    //           so the compositor starts only one `dbus-daemon`.
    let private_bus = if config.color_scheme.is_some()
        || config.accent_color.is_some()
        || config.launcher_log
        || config.accessibility
        || config.notifications.serves_notifications()
    {
        Some(Arc::new(
            PrivateBus::start().context("starting the private session bus")?,
        ))
    } else {
        None
    };

    // Serve deterministic appearance on that bus only when `--color-scheme` or `--accent-color` was requested.
    let appearance_portal = if config.color_scheme.is_some() || config.accent_color.is_some() {
        let bus = private_bus
            .as_ref()
            .map(Arc::clone)
            .context("the private session bus is missing for the appearance portal")?;
        Some(
            AppearancePortal::serve(PortalStart {
                color_scheme: config.color_scheme,
                accent_color: config.accent_color,
                bus,
            })
            .context("starting isolated nested appearance")?,
        )
    } else {
        None
    };

    // Serve the fake notification and job services only when `--notifications` asked for them.
    let notifications = match private_bus.as_ref() {
        Some(bus) => PlasmaServices::start(config.notifications, Arc::clone(bus), state.start_time)
            .context("starting the private notification services")?,
        None => None,
    };

    // The listener records taskbar progress for as long as the bus runs, so every bus has one.
    let launcher = match private_bus.as_ref() {
        Some(bus) => Some(
            LauncherLog::start(bus, state.start_time)
                .context("starting the taskbar progress listener")?,
        ),
        None => None,
    };

    // What:     `.as_ref()` borrows the optional bus; `.map(...)` reads its address when
    //           present; `.to_owned()` copies that borrowed text into an owned `String`.
    // Why:      The bus moves into the state next, and the child spawn below borrows the
    //           whole state mutably, so the address must not keep borrowing the bus.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const privateBusAddress = privateBus?.address();
    // ```
    let private_bus_address = private_bus
        .as_ref()
        .map(|bus| return bus.address().to_owned());

    // Keep the portal, bus, and listener reachable from control commands for the whole session.
    state.a11y = match (config.accessibility, private_bus.as_ref()) {
        (true, Some(bus)) => Some(
            crate::a11y_service::A11yService::start(Arc::clone(bus))
                .context("starting the private accessibility bus")?,
        ),
        _ => None,
    };

    state.appearance_portal = appearance_portal;
    state.notifications = notifications;
    state.private_bus = private_bus;
    state.launcher = launcher;

    // What:     `let isolation = Isolation { enabled: config.isolate, cpu_quota_percent:
    //           config.app_cpu_quota, cpu_weight: config.app_cpu_weight };`. Assemble the
    //           CPU-isolation settings from the parsed config.
    // Why:      Passed to `spawn_child`, which isolates the app under systemd or degrades.
    // What:     Announce the configured key repeat pair to every client that will connect.
    // Why:      Wayland clients generate key repeat themselves from `wl_keyboard.repeat_info`.
    if (config.repeat_rate, config.repeat_delay) != state.repeat_info {
        if let Some(keyboard) = state.seat.get_keyboard() {
            keyboard.change_repeat_info(config.repeat_rate, config.repeat_delay);
        }
        state.repeat_info = (config.repeat_rate, config.repeat_delay);
    }

    let isolation = Isolation {
        enabled: config.isolate,
        cpu_quota_percent: config.app_cpu_quota,
        cpu_weight: config.app_cpu_weight,
    };

    // Launch client on nested Wayland socket and private appearance bus when configured.
    spawn_child(
        &mut state,
        &config.child_command,
        &isolation,
        private_bus_address.as_deref(),
    )?;

    // What:     Register the fallback pacing timer, lending the loop handle and the state.
    // Why:      A locked or hidden parent window stops delivering redraws; the hosted client
    //           must keep receiving frame callbacks so screenshots show its current frame.
    crate::frame_pacing::register(&loop_handle, &mut state);

    // What:     `state.backend.window().request_redraw();`. Kick off the first frame.
    // Why:      Rendering is self-sustaining after the first request, but something has
    //           to request the initial one.
    state.backend.window().request_redraw();

    // What: Run the event loop and flush protocol replies after each dispatch cycle.
    // Why: Registry, configure, clipboard, and input delivery must not wait for GPU redraws.
    let outcome = event_loop.run(None, &mut state, |current| {
        crate::protocol_flush::finish_dispatch(&mut current.display_handle);
    });

    // What:     Return the failure an event-source callback stored, before the loop's own result.
    // Why:      A callback that failed stopped the loop on purpose, so its error is the cause.
    if let Some(error) = state.fatal_error.take() {
        return Err(error);
    }
    outcome.context("event loop failed")?;

    // What:     `Ok(state.child_exit_code.unwrap_or(0))`. Return the recorded child code,
    //           or 0 if the loop stopped for another reason. Tail expression.
    // Why:      Propagate the hosted app's exit status as our own.
    return Ok(state.child_exit_code.unwrap_or(0));
}

/// Dispatch one winit event:
///  resize the output,
///  redraw,
///  or stop on close.
///
/// What:
///      `fn handle_winit_event(event: WinitEvent, state: &mut Compositor)`.
///  Private
///           per-event handler.
/// Why:
///       Keep the event match out of `run` so `run` reads as pure orchestration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function handleWinitEvent(event, state) { ... }
/// ```
fn handle_winit_event(event: WinitEvent, state: &mut Compositor) {
    // What:     `match event { ... }`. Match the winit event enum. Only the variants the
    //           fixture cares about are handled; `_ => {}` ignores the rest (including
    //           `Input`, since the fixture injects its own input via the control API).
    // Why:      React to the parent window being resized or closed, and to redraw ticks.
    match event {
        WinitEvent::Resized { size, .. } => {
            // What:     `WinitEvent::Resized { size, .. }` binds the new framebuffer size and
            //           ignores the other field (`..`), the parent output's scale.
            // Why:      The parent changed the nested window's size. `parent_resized` keeps
            //           the logical size through the parent's rounding and follows a real
            //           change at the current nested scale. The parent's own scale is never
            //           adopted, so the nested scale stays what `--scale` and `scale` set.
            crate::screen::parent_resized(state, size);
        }
        WinitEvent::Redraw => {
            // What:     `redraw(state)`. Composite and present one frame; a failure ends the session.
            // Why:      A redraw was requested (by us or the parent).
            if let Err(error) = redraw(state) {
                state.fail(error);
            }
        }
        WinitEvent::CloseRequested => {
            // What:     `state.loop_signal.stop();`. End the event loop.
            // Why:      The parent asked the nested window to close.
            state.loop_signal.stop();
        }
        _ => {}
    }
}
