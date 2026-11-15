//! What:
//! The declared grammar of every toolchain verb:
//! the name `clap` parses,
//! the visible aliases it also accepts,
//! and the function that attaches the verb's arguments.
//! Why:
//! The command source in `commands.rs` runs the verbs and dispatches them;
//! keeping the `clap` builder data here lets one declaration feed both the
//! grammar that is built and the help test that checks it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export interface Verb { name: string; aliases: string[]; build: (verb: Verb) => Command }
//! export const VERBS: Verb[]
//! ```

/// Imports the `clap` types for describing arguments and commands.
use clap::{Arg, ArgAction, Command};

/// The `--global` flag every toolchain verb accepts:
/// act on the machine-global plan and lock.
fn global_flag() -> Arg {
    return Arg::new("global")
        .long("global")
        .short('g')
        .action(ArgAction::SetTrue)
        .help("Act on the machine-global toolchain (the plan and lock under the per-user config directory)");
}

/// The tool arguments `install` takes: the ids and requests it adds to the plan before installing.
fn install_tools_arg() -> Arg {
    return Arg::new("tool")
        .num_args(0..)
        .value_name("TOOL")
        .help(
            "A tool to install: a tool id the plan already holds, or a request the command adds to the plan first \
             (`cargo:<crate>`, `npm:<package>`, `go:<module>`, each with an optional `@<version>`; the newest when none is \
             given). Several tools are allowed; without one, every tool the plan holds is converged to the lock",
        );
}

/// What:
/// One toolchain verb as the grammar declares it:
/// the name `clap` parses,
/// the visible aliases it also accepts,
/// and the function that attaches the verb's arguments.
/// Why:
/// The verb surface is data:
/// `commands::subcommands` builds one `clap` command per entry,
/// and the help test reads the same entries,
/// so a verb or an alias cannot be added in only one place.
pub struct Verb {
    /// The name typed to run the verb.
    pub name: &'static str,
    /// Other names `clap` accepts;
    /// `--help` renders them beside the verb.
    pub aliases: &'static [&'static str],
    /// Attaches the verb's own arguments and returns its command.
    pub build: fn(&Verb) -> Command,
}

/// What:
/// The toolchain verbs `mpm-shadow --help` lists,
/// in render order.
/// Why:
/// `commands::subcommands` and the help test both read this table,
/// so a new verb (or alias) is covered by the grammar and the guard together.
pub const VERBS: &[Verb] = &[
    Verb {
        name: "install",
        aliases: &["i"],
        build: install_command,
    },
    // pnpm's own aliases for `update` ("Aliases: up, upgrade", https://pnpm.io/cli/update, read 2026-11-03).
    Verb {
        name: "update",
        aliases: &["up", "upgrade"],
        build: update_command,
    },
    Verb {
        name: "outdated",
        aliases: &[],
        build: outdated_command,
    },
    Verb {
        name: "list",
        aliases: &["ls"],
        build: list_command,
    },
    Verb {
        name: "verify-archive",
        aliases: &[],
        build: verify_archive_command,
    },
    Verb {
        name: "bin",
        aliases: &[],
        build: bin_command,
    },
];

/// What:
/// The `clap` command of a declared verb.
/// Why:
/// Both the toolchain verbs and the root `prune` verb start from their declaration,
/// so the parsed name and the visible aliases have one owner.
impl Verb {
    /// Starts the command this verb declares,
    /// with its parsed name and visible aliases.
    pub(crate) fn command(&self) -> Command {
        let mut command: Command = Command::new(self.name);
        for alias in self.aliases {
            command = command.visible_alias(*alias);
        }
        return command;
    }
}

/// The `install` verb:
/// every locked tool to its locked version,
/// then the exposure directory,
/// or the tools the command line names added to the plan first.
fn install_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Install every locked tool that is not at its locked version, then rebuild the exposure directory; a plan entry the lock lacks is resolved and locked alone first, unless frozen; each TOOL is an id the plan holds or a request to add to the plan first")
        .arg(install_tools_arg())
        .arg(
            Arg::new("frozen-lockfile")
                .long("frozen-lockfile")
                .action(ArgAction::SetTrue)
                .conflicts_with("no-frozen-lockfile")
                .help("Never resolve: refuse a plan that is ahead of the lock and name `mpm-shadow update <ids>`; with --lockfile-only, only check (the default in CI when a lock exists, like pnpm)"),
        )
        .arg(
            Arg::new("no-frozen-lockfile")
                .long("no-frozen-lockfile")
                .action(ArgAction::SetTrue)
                .help("Resolve a plan that is ahead of the lock even in CI"),
        )
        .arg(
            Arg::new("lockfile-only")
                .long("lockfile-only")
                .action(ArgAction::SetTrue)
                .help("Only bring toolchain.lock.json up to date; install nothing"),
        )
        .arg(
            Arg::new("offline")
                .long("offline")
                .action(ArgAction::SetTrue)
                .requires("lockfile-only")
                .help("Use no network: rewrite the lock's declaration fields from the plan, keeping every resolved version; refuse an entry that needs resolving"),
        )
        .arg(global_flag());
}

/// The `update` verb:
/// resolve the plan's tools and write the lock.
fn update_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Resolve the plan's tools (all, or only the named ones) and write toolchain.lock.json (looks versions up on the network)")
        .arg(tool_names_arg())
        .arg(global_flag());
}

/// The `outdated` verb:
/// report what `update` would move.
fn outdated_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Report which tools `update` would move, writing and installing nothing; exits 1 when any tool is outdated, like pnpm outdated")
        .arg(tool_names_arg())
        .arg(global_flag());
}

/// The `list` verb:
/// compare the installed tools with the lock.
fn list_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Compare the installed tools with the lock (read-only; exits 1 on drift or when the plan and lock disagree)")
        .arg(global_flag());
}

/// The `verify-archive` verb:
/// check a download against one lock row.
fn verify_archive_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Check a downloaded archive's SHA-256 against the lock's row for a tool on a platform (the bootstrap check)")
        .arg(Arg::new("tool").required(true).help("The tool id in the lock, for example cargo:monochromatic-mpm-shadow"))
        .arg(Arg::new("file").required(true).help("The downloaded file"))
        .arg(
            Arg::new("platform")
                .long("platform")
                .help("The lock's platform id (default: this machine's, for example linux-x64)"),
        )
        .arg(global_flag());
}

/// The `bin` verb:
/// the exposure directory,
/// or the JSON document for the IDE launcher.
fn bin_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Print the directory the toolchain's executables are linked into (read-only, offline)")
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .help("Print the program directories and exposed binaries as JSON instead (for the IDE launcher)"),
        )
        .arg(global_flag());
}

/// The tool-id patterns `update` and `outdated` take.
fn tool_names_arg() -> Arg {
    return Arg::new("tool")
        .num_args(0..)
        .value_name("TOOL")
        .help("Tool ids of the plan, `*` matches any run of characters and a leading `!` excludes (default: every tool); the other rows keep their versions");
}
