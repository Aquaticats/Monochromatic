//! Display-independent configuration consumed by the compositor.

/// Owned filesystem paths survive argument parsing.
use std::path::PathBuf;
/// Existing private-portal appearance values.
use crate::{
    accent_color::AccentColor, appearance_portal::ColorSchemePreference, notification_mode::NotificationMode,
};
/// Validated output scale shared with the runtime `scale` verb.
use crate::screen_geometry::OutputScale;
/// The device choice for headless rendering.
use crate::gpu_choice::GpuChoice;

/// Validated program configuration parsed from the command line.
///
/// What:
///      `pub struct Config { ... }` is a record type holding the parsed,
///           owned settings.
///  Every field is owned (`Vec<String>`,
///  `Option<PathBuf>`,
///           `i32`) so the value can be moved into the event loop and outlive the
///           argument vector.
/// Why:
///       One typed,
///  validated bundle passed to `run`,
///  so the rest of the
///           program never re-parses raw `&str` arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Config = {
///   childCommand: string[];
///   controlSocket: string | undefined;
///   width: number;
///   height: number;
/// };
/// ```
pub struct Config {
    /// Program-and-arguments of the single client to host,
    ///  e.g. `["music-player", "fixtures"]`.
    ///
    /// What:
    ///      `pub child_command: Vec<String>`.
    ///  `Vec<String>` is a heap array of
    ///           owned strings (sibling:
    ///  `&[&str]`,
    ///  a borrowed slice of borrowed
    ///           strings).
    /// Why:
    ///       The compositor forks exactly this command as its one client;
    ///  owned
    ///           so it survives past the argument slice.
    pub child_command: Vec<String>,

    /// Optional filesystem path for the Unix control socket.
    ///
    /// What:
    ///      `pub control_socket: Option<PathBuf>`.
    ///  `Option<T>` is Rust's
    ///           null-free "maybe" type:
    ///  either `Some(path)` or `None`.
    ///  `PathBuf`
    ///           is an owned path.
    /// Why:
    ///       When absent (`None`),
    ///  the control API is disabled and the fixture
    ///           just hosts the app;
    ///  when present,
    ///  it binds a socket there.
    pub control_socket: Option<PathBuf>,

    /// Initial nested-screen width in logical pixels (physical pixels at scale 1).
    ///
    /// What:
    ///      `pub width: i32`.
    ///  Signed 32-bit integer to match Smithay geometry.
    /// Why:
    ///       The hosted app is configured with this width;
    ///  the framebuffer and
    ///           screenshots are this width times `scale`.
    pub width: i32,

    /// Initial nested-screen height in logical pixels (physical pixels at scale 1).
    ///
    /// What:
    ///      `pub height: i32`.
    ///  Signed 32-bit integer to match Smithay geometry.
    /// Why:
    ///       Pairs with `width` for the initial logical screen size.
    pub height: i32,

    /// Initial output scale every hosted surface is told to render at (`--scale`).
    ///
    /// What:
    ///      `pub scale: OutputScale`.
    ///  A validated whole number of 120ths.
    /// Why:
    ///       Lets a session start at a fractional or integer scale;
    ///  the `scale` control
    ///           command changes it later.
    pub scale: OutputScale,

    /// Optional isolated portal color scheme for hosted client.
    ///
    /// What:
    ///      `pub color_scheme: Option<ColorSchemePreference>`.
    /// Why:
    ///       `Some` starts private Settings portal without changing host desktop theme.
    pub color_scheme: Option<ColorSchemePreference>,

    /// Optional isolated portal accent color for hosted client.
    ///
    /// What:
    ///      `pub accent_color: Option<AccentColor>`.
    /// Why:
    ///       `Some` starts the private Settings portal and serves `org.freedesktop.appearance` `accent-color`.
    pub accent_color: Option<AccentColor>,

    /// Which fake notification services the private bus serves (`--notifications`).
    ///
    /// What:
    ///      `pub notifications: NotificationMode`.
    /// Why:
    ///       Anything but `None` starts the private bus and records the calls a hosted app makes
    ///  to `org.freedesktop.Notifications` and,
    ///  in `kde` mode,
    ///  to Plasma's job view server.
    pub notifications: NotificationMode,

    /// Whether to launch the hosted app inside a resource-controlled systemd scope.
    ///
    /// What:
    ///      `pub isolate: bool`.
    ///  Set by `--isolate`.
    /// Why:
    ///       Reserve CPU headroom for the 60fps capture pipeline so a greedy app cannot
    ///           starve it;
    ///  degrades to a direct launch when systemd is unavailable.
    pub isolate: bool,

    /// Whether to start the private session bus so taskbar progress can be observed.
    ///
    /// What:
    ///      `pub launcher_log: bool`.
    ///  Set by `--launcher-log`.
    /// Why:
    ///       The hosted app then gets a private `DBUS_SESSION_BUS_ADDRESS` where its
    ///  `LauncherEntry.Update` signals reach the compositor,
    ///  and the host session bus never sees them.
    pub launcher_log: bool,

    /// Optional hard CPU cap for the app,
    ///  in percent of one core (`--app-cpu-quota`).
    ///
    /// What:
    ///      `pub app_cpu_quota: Option<u32>`.
    ///  `800` means eight cores' worth.
    /// Why:
    ///       Override the machine-sized default cap used when `--isolate` is set.
    pub app_cpu_quota: Option<u32>,

    /// Optional relative CPU share for the app (`--app-cpu-weight`,
    ///  systemd 1..=10000).
    ///
    /// What:
    ///      `pub app_cpu_weight: Option<u32>`.
    /// Why:
    ///       Override the default low weight that deprioritises the app under contention.
    pub app_cpu_weight: Option<u32>,

    /// Whether to serve the private accessibility bus.
    pub accessibility: bool,

    /// Key repeat rate announced to the hosted client,
    ///  in repeats per second.
    pub repeat_rate: i32,

    /// Wait before a held key starts repeating,
    ///  in milliseconds.
    pub repeat_delay: i32,

    /// Whether to render without any parent display (`--headless`).
    ///
    /// What:
    ///      `pub headless: bool`.
    ///  False unless the flag is given.
    /// Why:
    ///       The nested window stays the default so a person can restore it and inspect the running app;
    ///  headless is opt-in only and is never chosen because a display is missing.
    pub headless: bool,

    /// The device a headless session renders on (`--gpu`).
    ///
    /// What:
    ///      `pub gpu: GpuChoice`.
    ///  `Auto` unless `--gpu` is given,
    ///  and `--gpu` is accepted only with `--headless`.
    /// Why:
    ///       The nested window renders on the parent compositor's GPU,
    ///  so only a headless session has a choice to make.
    pub gpu: GpuChoice,

    /// Whether to print the devices `--gpu` accepts and exit (`--list-gpus`).
    ///
    /// What:
    ///      `pub list_gpus: bool`.
    ///  When true,
    ///  `child_command` is empty.
    /// Why:
    ///       A person needs to see what to pass to `--gpu` without starting a session.
    pub list_gpus: bool,
}
