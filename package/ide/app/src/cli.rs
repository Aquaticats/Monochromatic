//! Display-independent startup grammar for one project root and an optional initial source file.
//!
//! Without a project argument the application opens the user's home folder (`$HOME`),
//!  as a launcher
//! entry started without a folder does (decision of 2026-10-06).
//!  `--licenses` prints the license and
//! notice texts embedded in the executable instead of opening a window (decision of 2026-10-06).

/// Preserve clap's typed help/usage errors inside the application's standard error envelope.
use anyhow::Result;
/// Reuse the repository's CLI parser and native-path value parser instead of manual argument dispatch.
use clap::{Arg, ArgAction, Command, builder::PathBufValueParser, error::ErrorKind};
/// Native argv and paths retain non-UTF-8 names rather than passing through display strings.
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Parsed startup inputs;
///  filesystem validation happens after help and version handling.
#[derive(Debug)]
pub struct Options {
    /// Sole project directory:
    ///  the argument,
    ///  or the home folder when none was given.
    pub project: PathBuf,
    /// Optional source path resolved relative to the project root,
    ///  not the caller's cwd.
    pub file: Option<PathBuf>,
}

/// What:
///  What the command line asks for.
///  `enum` is a value that is exactly one of these variants,
///       like a TS discriminated union.
/// Why:
///  Printing the license texts needs no project,
///  no home folder,
///  and no display.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Startup = { kind: 'open'; options: Options } | { kind: 'licenses' };
/// ```
#[derive(Debug)]
pub enum Startup {
    /// Open a window on a project.
    Open(
        /// The project and the optional initial file.
        Options,
    ),
    /// Print every embedded license and notice text,
    ///  then exit with status 0.
    Licenses,
}

/// Construct help and option grammar without initializing a display or reading project contents.
fn command() -> Command {
    // What: Command and Arg builders declare grammar; they do not start the application.
    // Why: --help and --version must work without a compositor, project directory, or font initialization.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const command = new Command('monochromatic-ide');
    // ```
    let mut command = Command::new("monochromatic-ide");
    command = command.about("Read-only source navigation in one local project");
    command = command.version(env!("CARGO_PKG_VERSION"));
    let mut project = Arg::new("project");
    project = project.value_name("PROJECT").required(false);
    project = project.value_parser(PathBufValueParser::new());
    project = project.help(
        "Local directory used as the sole file-tree and search root; without it, the home folder ($HOME)",
    );
    command = command.arg(project);
    let mut file = Arg::new("file");
    file = file.long("file").value_name("FILE");
    file = file.value_parser(PathBufValueParser::new());
    file = file.help(
        "Initially display a UTF-8 source file within PROJECT; relative paths start at PROJECT",
    );
    command = command.arg(file);
    let mut licenses = Arg::new("licenses");
    licenses = licenses.long("licenses").action(ArgAction::SetTrue);
    // What: `conflicts_with_all` makes clap report a usage error when these arguments are combined.
    // Why: Printing the texts opens nothing, so a project or file beside it would be silently ignored.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (options.licenses && (options.project || options.file)) throw new UsageError('...');
    // ```
    licenses = licenses.conflicts_with_all(["project", "file"]);
    licenses = licenses.help(
        "Print every license and notice text embedded in the executable, then exit without opening a window",
    );
    command = command.arg(licenses);
    return command;
}

/// What:
///  The home folder from `HOME`,
///  when it is an absolute path.
///  `Option<PathBuf>` is an owned
///       path or nothing;
///  `var_os` keeps non-UTF-8 bytes.
/// Why:
///  The executable's no-argument start opens this folder;
///  tests pass their own instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function homeFolder(): string | undefined { return isAbsolute(env.HOME) ? env.HOME : undefined; }
/// ```
pub fn home_folder() -> Option<PathBuf> {
    // What: the trailing `?` returns `None` from this function when `HOME` is unset.
    // Why: An unset HOME names no folder to open.
    let home = PathBuf::from(std::env::var_os("HOME")?);
    // What: `is_absolute` rejects empty and relative values.
    // Why: An empty or relative HOME names no folder to open.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (isAbsolute(home)) return home;
    // ```
    if home.is_absolute() {
        return Some(home);
    }
    return None;
}

/// Parse argv excluding the executable name,
///  opening the `HOME` folder when no project is given;
/// no filesystem or toolkit operations occur here.
pub fn parse_args(args: &[OsString]) -> Result<Startup> {
    // `home_folder().as_deref()` lends the owned path as `Option<&Path>`.
    return parse_args_with_home(args, home_folder().as_deref());
}

/// Parse argv excluding the executable name,
///  with an explicit home folder for the no-project case.
/// `home` is `None` when no usable home folder exists;
///  then a missing project is a usage error.
pub fn parse_args_with_home(args: &[OsString], home: Option<&Path>) -> Result<Startup> {
    // What: Vec owns a variable-length argv list; unlike String, OsString retains native non-UTF-8 bytes.
    // Why: The command name occupies argv[0] while project filenames remain exact native paths.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const argv = [nativeString('monochromatic-ide'), ...args];
    // ```
    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(OsString::from("monochromatic-ide"));
    for arg in args {
        // Clone gives clap owned argv without consuming the caller's original arguments.
        argv.push(arg.clone());
    }
    // What: ? propagates typed parse/help errors so the executable can use clap's exit code and output stream.
    // Why: Help is neither a startup failure nor permission to open a window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const matches = command().tryParse(argv);
    // ```
    let matches = command().try_get_matches_from(argv)?;
    // `get_flag` reads a `SetTrue` switch; the license texts need neither a project nor a home folder.
    if matches.get_flag("licenses") {
        return Ok(Startup::Licenses);
    }
    // cloned copies a present owned path; omission remains None rather than a fabricated filename.
    let file = matches.get_one::<PathBuf>("file").cloned();
    // What: `match (argument, home)` looks at both options at once: `(Some(path), _)` is an explicit
    //       project, `(None, Some(home))` the home default, `(None, None)` neither.
    // Why: An explicit project always wins; the home folder is only the default.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const project = matches.project ?? home ?? usageError('...');
    // ```
    let project = match (matches.get_one::<PathBuf>("project"), home) {
        (Some(path), _) => path.clone(),
        (None, Some(folder)) => folder.to_path_buf(),
        (None, None) => {
            // What: `command().error(kind, message)` builds a typed clap usage error (exit status 2,
            //       printed with the usage line); `.into()` converts it into the anyhow error type.
            // Why: The executable prints and exits through clap for every grammar problem.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // throw new UsageError('No PROJECT was given ...');
            // ```
            return Err(command()
                .error(
                    ErrorKind::MissingRequiredArgument,
                    "No PROJECT was given, and HOME does not name an absolute folder to open instead. Pass the folder to open as PROJECT, or set HOME.",
                )
                .into());
        }
    };
    // What: Ok returns the owned configuration after parser validation.
    // Why: No references into temporary clap matches escape this function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { kind: 'open', options: { project, file: matches.file } };
    // ```
    return Ok(Startup::Open(Options { project, file }));
}
