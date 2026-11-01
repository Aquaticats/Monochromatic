//! Clap owns option grammar,
//!  generated help,
//!  and parent/child argument separation.

/// What:
///  Import clap's command and argument builders.
/// Why:
///  Help and validation are defined with the options instead of a manual loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command, Arg } from 'cli-parser';
/// ```
use clap::{Arg, ArgAction, Command};
/// Typed value parsers preserve u32 limits and the existing size validator.
use clap::builder::{RangedU64ValueParser, ValueParser};
/// The output-scale grammar shared with the runtime `scale` verb.
use crate::screen_geometry::OutputScale;
/// The `--gpu` grammar.
use crate::gpu_choice::GpuChoice;

/// Construct the CLI independently of process arguments or a display server.
pub(super) fn command() -> Command {
    // What: Command::new starts a mutable parser definition, not a running compositor.
    // Why: --help must be handled before any Wayland initialization.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let command = new Command('monochromatic-nested-wayland-session');
    // ```
    let mut command = Command::new("monochromatic-nested-wayland-session");
    command = command.about("Host one GUI client in an isolated nested Wayland session");
    // Cargo supplies this compile-time string; no runtime version subprocess is needed.
    command = command.version(env!("CARGO_PKG_VERSION"));
    command = command.args_override_self(true);
    command = command.arg_required_else_help(true);
    command = command.after_help(
        "The first client command starts child arguments. Use -- to separate parent options.\n\
         Example: monochromatic-nested-wayland-session --size 800x600 -- app --help"
    );

    // Define one owned socket path; repeated parent options retain last-value behavior.
    let mut socket = Arg::new("socket");
    socket = socket.long("socket").value_name("PATH");
    socket = socket.help("Enable the Unix-socket control API at PATH");
    command = command.arg(socket);

    let mut size = Arg::new("size");
    size = size.long("size").value_name("WIDTHxHEIGHT");
    size = size.default_value("1280x720");
    size = size.help("Initial nested-screen size in logical pixels; screenshots are this times --scale");
    size = size.value_parser(ValueParser::new(super::size::parse_size));
    command = command.arg(size);

    let mut scale = Arg::new("scale");
    scale = scale.long("scale").value_name("SCALE");
    scale = scale.default_value("1");
    scale = scale.help(
        "Initial output scale, 0.5 to 3 in steps of 1/120 (such as 1, 1.25, 1.5, 2); \
         the scale control command changes it while the child runs"
    );
    // What: ValueParser::new wraps a plain function that returns Result<OutputScale, String>.
    // Why: The startup option and the runtime verb share one grammar and one message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // scale.valueParser(OutputScale.parse);
    // ```
    scale = scale.value_parser(ValueParser::new(OutputScale::parse));
    command = command.arg(scale);

    let mut scheme = Arg::new("color-scheme");
    scheme = scheme.long("color-scheme").value_name("SCHEME");
    scheme = scheme.value_parser(["dark", "light"]);
    scheme = scheme.help(
        "Give the child a private appearance portal without changing the host theme; \
         the color-scheme control command switches it while the child runs"
    );
    command = command.arg(scheme);

    let mut accent = Arg::new("accent-color");
    accent = accent.long("accent-color").value_name("RRGGBB");
    accent = accent.help(
        "Give the child a private appearance portal that serves this accent color as six hexadecimal digits; \
         the accent-color control command changes it while the child runs"
    );
    command = command.arg(accent);

    let mut notifications = Arg::new("notifications");
    notifications = notifications.long("notifications").value_name("MODE");
    notifications = notifications.value_parser(["none", "freedesktop", "kde"]);
    notifications = notifications.default_value("none");
    notifications = notifications.help(
        "Serve recording notification services on the private bus: none (default), freedesktop, or kde \
         (freedesktop plus Plasma's job view server); the notification and job control commands read them"
    );
    command = command.arg(notifications);

    let mut isolate = Arg::new("isolate");
    isolate = isolate.long("isolate").action(ArgAction::SetTrue);
    isolate = isolate.help("Run the hosted app in a resource-controlled systemd scope");
    command = command.arg(isolate);

    let mut launcher_log = Arg::new("launcher-log");
    launcher_log = launcher_log.long("launcher-log").action(ArgAction::SetTrue);
    launcher_log = launcher_log.help(
        "Give the child a private session bus and record its taskbar progress; \
         the launcher control commands read the record"
    );
    command = command.arg(launcher_log);

    let mut quota = Arg::new("app-cpu-quota");
    quota = quota.long("app-cpu-quota").value_name("PERCENT");
    // What: This parser returns u32, not a string or signed i32.
    // Why: Preserve the existing public Config field and its unsigned input range.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // quota.valueParser(parseUint32);
    // ```
    quota = quota.value_parser(RangedU64ValueParser::<u32>::new());
    quota = quota.help("App CPU cap under --isolate, as percent of one core");
    command = command.arg(quota);

    let mut weight = Arg::new("app-cpu-weight");
    weight = weight.long("app-cpu-weight").value_name("WEIGHT");
    weight = weight.value_parser(RangedU64ValueParser::<u32>::new());
    weight = weight.help("Relative systemd CPU weight for the isolated app");
    command = command.arg(weight);

    let mut accessibility = Arg::new("accessibility");
    accessibility = accessibility.long("accessibility").action(ArgAction::SetTrue);
    accessibility = accessibility.help(
        "Give the child a private AT-SPI accessibility bus that reports accessibility as enabled, \
         and read the tree it publishes with the a11y control command"
    );
    command = command.arg(accessibility);

    let mut repeat_rate = Arg::new("repeat-rate");
    repeat_rate = repeat_rate.long("repeat-rate").value_name("PER_SECOND");
    repeat_rate = repeat_rate.default_value("25");
    repeat_rate = repeat_rate.value_parser(clap::value_parser!(i32).range(0..=1000));
    repeat_rate = repeat_rate.help(
        "Key repeat rate announced to the child in repeats per second (0 turns repeat off); \
         the child generates the repeats itself, as on every Wayland compositor"
    );
    command = command.arg(repeat_rate);

    let mut repeat_delay = Arg::new("repeat-delay");
    repeat_delay = repeat_delay.long("repeat-delay").value_name("MILLISECONDS");
    repeat_delay = repeat_delay.default_value("200");
    repeat_delay = repeat_delay.value_parser(clap::value_parser!(i32).range(0..=10000));
    repeat_delay = repeat_delay.help(
        "Wait before a held key starts repeating, announced to the child; \
         the repeat-info control command changes the pair while the child runs"
    );
    command = command.arg(repeat_delay);

    let mut headless = Arg::new("headless");
    headless = headless.long("headless").action(ArgAction::SetTrue);
    headless = headless.help(
        "Render with surfaceless EGL on the GPU render node instead of a nested window: \
         no parent display is used or needed and no window appears; the default is the nested window"
    );
    command = command.arg(headless);

    let mut gpu = Arg::new("gpu");
    gpu = gpu.long("gpu").value_name("GPU");
    gpu = gpu.help(
        "With --headless, the device that renders: auto (the default: Mesa's default render node, then the other \
         render nodes, then software), software (Mesa's CPU rasterizer), or a render node such as \
         /dev/dri/renderD128; an explicit choice never falls back. Invalid without --headless: the nested window \
         renders on its parent compositor's GPU, which this tool cannot choose"
    );
    gpu = gpu.value_parser(ValueParser::new(GpuChoice::parse));
    command = command.arg(gpu);

    let mut list_gpus = Arg::new("list-gpus");
    list_gpus = list_gpus.long("list-gpus").action(ArgAction::SetTrue);
    list_gpus = list_gpus.conflicts_with_all(["gpu", "command"]);
    list_gpus = list_gpus.help(
        "Print the render nodes (with PCI id, kernel driver and Mesa renderer) and the software renderer this host \
         offers to --gpu, name the one auto uses, and exit"
    );
    command = command.arg(list_gpus);

    let mut child = Arg::new("command");
    child = child.value_name("COMMAND").required_unless_present("list-gpus");
    // What: 1.. accepts at least one positional value; trailing_var_arg hands
    // subsequent tokens to the child, including tokens that look like our flags.
    // Why: app --help must reach app, while parent --help still belongs to clap.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // child.remainingArgs({ min: 1, stopParsingOptionsAfterFirst: true });
    // ```
    child = child.num_args(1..).trailing_var_arg(true);
    child = child.help("Client executable followed by its arguments");
    command = command.arg(child);
    return command;
}
