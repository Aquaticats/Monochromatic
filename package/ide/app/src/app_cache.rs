//! The application's private cache directory,
//!  `$XDG_CACHE_HOME/monochromatic-ide`.
//!
//! What:
//!  One function that both language-server state and unpacked parser libraries build on.
//! Why:
//!  The application may keep private cache files outside the project (scope decision),
//!  and two
//!      copies of the rule for finding that directory could drift apart.

/// An owned path:
///  callers join further names onto the cache directory.
use std::path::PathBuf;

/// What:
///  The value of one environment variable as an owned path,
///  or `None` when it is unset or not an absolute path.
///  `Option<PathBuf>` is an owned path or nothing (sibling:
///  `&Path`,
///  a borrowed path);
///  `var_os` reads a variable without requiring UTF-8.
/// Why:
///  A relative value would depend on the working directory,
///  which the application changes,
///  and the XDG base directory specification says relative values must be ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function absoluteVariable(name: string): string | undefined {
///   const value = env[name];
///   return value !== undefined && isAbsolute(value) ? value : undefined;
/// }
/// ```
fn absolute_variable(name: &str) -> Option<PathBuf> {
    // The trailing `?` returns `None` from this function when the variable is unset.
    let path = PathBuf::from(std::env::var_os(name)?);
    if path.is_absolute() {
        return Some(path);
    }
    return None;
}

/// What:
///  `$HOME/.cache`,
///  or `None` when `$HOME` is not an absolute path.
/// Why:
///  The specification's fallback when `$XDG_CACHE_HOME` is unusable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function homeCache(): string | undefined {
///   const home = absoluteVariable('HOME');
///   return home && join(home, '.cache');
/// }
/// ```
fn home_cache() -> Option<PathBuf> {
    // The trailing `?` returns `None` from this function when `$HOME` is unusable.
    let home = absolute_variable("HOME")?;
    return Some(home.join(".cache"));
}

/// What:
///  The cache directory,
///  or `None` when neither variable is an absolute path.
///       `Option<PathBuf>` is an owned path or nothing (sibling:
///  `&Path`,
///  a borrowed path);
///       `var_os` reads a variable without requiring UTF-8;
///  `absolute_variable` keeps only an absolute `$XDG_CACHE_HOME`;
///       `.or_else(home_cache)` tries the home directory's `.cache` when the first is missing.
/// Why:
///  The XDG base directory specification names `$XDG_CACHE_HOME`,
///  falling back to `~/.cache`,
///      and says relative values are invalid and must be ignored.
///  An owned path is returned because
///      callers join further names onto it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applicationCache(): string | undefined {
///   const cache = absolute(env.XDG_CACHE_HOME) ?? (absolute(env.HOME) && join(env.HOME, '.cache'));
///   return cache && join(cache, 'monochromatic-ide');
/// }
/// ```
pub fn application_cache() -> Option<PathBuf> {
    // The trailing `?` returns `None` from this function when neither variable is usable.
    let cache = absolute_variable("XDG_CACHE_HOME").or_else(home_cache)?;
    // What: `Some(...)` wraps the joined path as the present variant of `Option`.
    // Why: Callers distinguish "no cache directory" from a usable one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return join(cache, 'monochromatic-ide');
    // ```
    return Some(cache.join("monochromatic-ide"));
}
