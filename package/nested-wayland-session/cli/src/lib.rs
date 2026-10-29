//! A minimal single-app nested Wayland compositor for GUI testing.
//!
//! This library owns everything except process startup:
//!  argument parsing,
//!  the
//! compositor state and its protocol handlers,
//!  the winit backend and render loop,
//! and the hosted-child lifecycle.
//!  Keeping it all in the library (with the binary a
//! thin shell over `run`) lets the display-independent pieces (argument parsing) be
//! unit-tested without opening a window.
//!  See the module docs for each part.

/// What:
///      `pub mod app;`.
///  Declares the `app` module from `src/app.rs`.
/// Why:
///       Holds `run`,
///  the whole-program orchestration entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as app from "./app";
/// ```
pub mod app;

/// Exposes private XDG Settings portal used for deterministic nested appearance.
pub mod appearance_portal;

/// What:
///      `pub mod backend;`.
///  Declares the winit/EGL/dmabuf backend init module.
/// Why:
///       Builds the nested window,
///  GLES renderer,
///  output,
///  and dmabuf state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as backend from "./backend";
/// ```
pub mod backend;

/// Output and dmabuf construction shared by both render backends.
pub mod backend_parts;

/// Surfaceless EGL initialisation for the opt-in headless mode.
pub mod backend_headless;

/// The `--gpu` choice,
///  the order `auto` tries devices in,
///  and the hosted child's matching environment.
pub mod gpu_choice;

/// Opening the chosen render device and listing the devices a host offers.
pub mod gpu_device;

/// The nested window and surfaceless renderers behind one handle.
pub mod render_backend;

/// Which render backend a session uses:
///  the nested window by default,
///  headless only when asked.
pub mod backend_select;

/// What:
///      `pub mod child;`.
///  Declares the hosted-client lifecycle module.
/// Why:
///       Spawns the app and stops the loop on its exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as child from "./child";
/// ```
pub mod child;

/// What:
///      `pub mod control;`.
///  Declares the Unix-socket control API module.
/// Why:
///       Parses control commands and runs them (screenshot,
///  input,
///  resize,
///  quit).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as control from "./control";
/// ```
pub mod control;

/// Runtime appearance switching reaches only the private Settings portal.
pub mod control_color_scheme;

/// Taskbar progress verbs answer from the private-bus listener's history.
pub mod control_launcher;

/// Window title and app-ID verbs read the hosted toplevel as it stands now.
pub mod control_window;

/// Pointer hover and held-button injection through the nested seat.
pub mod input_pointer;

/// Taskbar progress answer text,
///  including the one-line URI escaping.
pub mod launcher_format;

/// Bounded history of taskbar progress updates.
pub mod launcher_history;

/// Listens for taskbar progress signals on the private bus.
pub mod launcher_listen;

/// The accent color the private appearance portal serves.
pub mod accent_color;

/// Runtime accent color switching reaches only the private Settings portal.
pub mod control_accent;

/// Plasma job verbs answer from the fake job view server's records.
pub mod control_job;

/// Notification verbs answer from the fake notification service's records.
pub mod control_notification;

/// A bounded,
///  oldest-first record of observed events.
pub mod event_ring;

/// Plasma job answer text,
///  with one-line escaping of app-chosen text.
pub mod job_format;

/// The fake Plasma job view server and its kuiserver interface.
pub mod job_server;

/// State of the fake Plasma job view server.
pub mod job_store;

/// One granted Plasma job view object.
pub mod job_view;

/// Notification answer text,
///  with one-line escaping of app-chosen text.
pub mod notification_format;

/// Which fake notification services a session serves.
pub mod notification_mode;

/// The fake freedesktop notification server.
pub mod notification_service;

/// State of the fake freedesktop notification server.
pub mod notification_store;

/// The fake notification and Plasma job services on the private bus.
pub mod plasma_services;

/// A store shared between the bus service thread and the control thread.
pub mod service_state;

/// Plain-text summaries of D-Bus variant maps.
pub mod value_summary;

/// One-line encoding of client-chosen text in control answers.
pub mod wire_text;

/// The private session bus shared by the appearance portal,
///  the launcher listener,
///  and the fake services.
pub mod private_bus;

/// What:
///      `pub mod dnd;`.
///  Declares the compositor-originated drag-and-drop module.
/// Why:
///       Drives a server-side `text/uri-list` drag toward the hosted app so the app's
///           INBOUND file-drop path can be tested deterministically without a file manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as dnd from "./dnd";
/// ```
pub mod dnd;

/// What:
///      `pub mod encoder;`.
///  Declares the frame-encoding + worker-pool module.
/// Why:
///       Flips and encodes captured frames (PNG/BMP) on a parallel worker pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as encoder from "./encoder";
/// ```
pub mod encoder;

/// What:
///      `pub mod input;`.
///  Declares the synthetic input-injection module.
/// Why:
///       Turns click/key/type commands into seat events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as input from "./input";
/// ```
pub mod input;

/// True notched-wheel input reaches the hosted client's native Wayland path.
pub mod input_wheel;

/// Hosted-client frame pacing must not depend on the parent window being presented.
pub mod frame_pacing;

/// What:
///      `pub mod keymap;`.
///  Declares the US-layout keycode tables.
/// Why:
///       Maps characters and key names to evdev keycodes;
///  display-independent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as keymap from "./keymap";
/// ```
pub mod keymap;

/// The minimal AT-SPI accessibility bus services.
pub mod a11y_service;

/// Reads and formats the hosted application's accessibility tree.
pub mod a11y_tree;

/// Runs one control request now or as a timed sequence.
pub mod control_run;

/// Session settings the control socket reads and changes.
pub mod control_session;

/// Binding of the control socket with owner-only access.
pub mod control_socket;

/// Keyboard focus order across several toplevels.
pub mod focus;

/// Key chords through the nested seat.
pub mod input_key;

/// Typing text,
///  with spare Unicode keys for off-layout characters.
pub mod input_text;

/// Timed input sequences that answer when they end.
pub mod timeline;

/// Turns the timed commands into step lists.
pub mod timeline_plan;

/// Keymaps carrying arbitrary Unicode characters.
pub mod unicode_keymap;

/// `text/uri-list` payloads for dropped files.
pub mod uri_list;

/// The window listing,
///  focus,
///  and close answers.
pub mod window_list;

/// Chord specs such as `ctrl+shift+f10`.
pub mod key_chord;

/// Key names resolved to evdev codes.
pub mod key_name;

/// The kernel key names with their codes.
pub mod key_table;

/// What:
///      `pub mod protocol;`.
///  Declares the control-protocol parsing module.
/// Why:
///       Parses request lines and formats response lines;
///  display-independent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as protocol from "./protocol";
/// ```
pub mod protocol;

/// End-of-dispatch delivery must not depend on a visible rendering window.
pub mod protocol_flush;

/// Logical size and output scale as plain numbers,
///  tested without a display.
pub mod screen_geometry;

/// Applying the nested screen's size and scale to the window,
///  output,
///  and hosted surfaces.
pub mod screen;

/// What:
///      `pub mod screenshot;`.
///  Declares the framebuffer-readback module.
/// Why:
///       Renders a frame and encodes it as a PNG.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as screenshot from "./screenshot";
/// ```
pub mod screenshot;

/// What:
///      `pub mod cli;`.
///  Declares the argument-parsing module.
/// Why:
///       Turns raw arguments into a validated `Config`;
///  display-independent and
///           unit-tested.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as cli from "./cli";
/// ```
pub mod cli;

/// What:
///      `pub mod handler;`.
///  Declares the Wayland protocol handler module tree.
/// Why:
///       Implements the compositor/xdg-shell/shm/seat/dmabuf behaviour.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as handler from "./handler";
/// ```
pub mod handler;

/// What:
///      `pub mod render;`.
///  Declares the rendering module.
/// Why:
///       Composites the hosted window into the nested framebuffer each frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as render from "./render";
/// ```
pub mod render;

/// What:
///      `pub mod recorder;`.
///  Declares the 60fps frame-recorder module.
/// Why:
///       Drives the steady,
///  app-decoupled capture timer that feeds the encoder pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as recorder from "./recorder";
/// ```
pub mod recorder;

/// What:
///      `pub mod state;`.
///  Declares the central-state module.
/// Why:
///       Defines `Compositor`,
///  the value the event loop carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as state from "./state";
/// ```
pub mod state;

/// What:
///      `pub mod systemd;`.
///  Declares the systemd CPU-isolation module.
/// Why:
///       Launches the hosted app in a resource-controlled scope (or degrades cleanly).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as systemd from "./systemd";
/// ```
pub mod systemd;

/// What:
///      `pub use app::run;`.
///  Re-export `run` at the crate root.
/// Why:
///       The binary calls `nested_wayland_session::run` without knowing the module
///           layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { run } from "./app";
/// ```
pub use app::run;

/// What:
///      `pub use cli::{parse_args, Config};`.
///  Re-export the parser and its output.
/// Why:
///       The binary and tests use these directly from the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseArgs, Config } from "./cli";
/// ```
pub use cli::{parse_args, Config};

/// Exposes supported deterministic nested appearance values.
pub use appearance_portal::ColorSchemePreference;
