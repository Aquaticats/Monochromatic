//! What:
//!  The Git configuration a push carries on its command line:
//!  the pairs `-c` and `--config-env` name,
//!  and the variables a child that reads Git's configuration receives for them.
//! Why:
//!  Git applies such a setting above every configuration file,
//!  and it hands a child the settings it was given by appending them to `GIT_CONFIG_PARAMETERS`,
//!  the text Git appends its own `-c` and `--config-env` options to.
//!  Git reads that variable after the numbered `GIT_CONFIG_COUNT` pairs,
//!  so a pair appended here wins over an inherited parameter and over a numbered pair,
//!  exactly as the same setting wins in the Git that runs the caller's push.
//!  Two readers need the caller's pairs:
//!  the Git LFS upload child (`lfs_upload.rs`),
//!  which starts `git-lfs` itself,
//!  and the switch that turns the upload off (`lfs_pre_push.rs`).
//!  A setting either of them loses is an upload sent to another server than the ref:
//! the object lands where the pointer is not,
//!  and every collaborator of the remote that got the pointer cannot fetch its content.

/// The invocation's environment values.
use crate::child_environment::environment_value;
/// Owned OS text for the variables and their values,
/// and borrowed OS text for the three spellings of a path option.
use std::ffi::{OsStr, OsString};

/// What:
///  The variable through which Git hands a child the configuration its command line carries.
/// Why:
///  Git appends each `-c` and `--config-env` option to the value it inherited under this name,
///  and reads the whole value after the numbered pairs,
///  so a pair written here is the newest setting of its key a child sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARAMETERS_VARIABLE = 'GIT_CONFIG_PARAMETERS';
/// ```
pub const PARAMETERS_VARIABLE: &str = "GIT_CONFIG_PARAMETERS";

/// What:
///  The configuration pair a `--config-env` parameter names,
/// read from the environment of the invocation,
/// or nothing when Git itself refuses the option as it stands.
/// Why:
///   `git-lfs` reads Git's configuration through `git config`,
/// which reads the caller's settings from the environment Git hands it,
/// so an option that names an environment variable must be resolved here.
///  Git 2.56.0 takes the text before the last `=` as the key and the variable name after it
/// (`command_status_tests.rs` holds the real-Git control),
/// and the name is compared as the bytes the operating system holds.
///  `None` covers exactly the parameters Git refuses:
/// no `=`,
/// an empty variable name,
/// or a variable the invocation's environment does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pair = configEnvPair(parameter, environment); // undefined when Git would refuse the option
/// ```
pub fn config_env_pair(parameter: &[u8], environment: &[(OsString, OsString)]) -> Option<(OsString, OsString)> {
    // The last `=` is where Git ends the key.
    let mut last_equals: Option<usize> = None;
    for (index, byte) in parameter.iter().enumerate() {
        if *byte == b'=' {
            last_equals = Some(index);
        }
    }
    let at: usize = last_equals?;
    let variable: &[u8] = &parameter[at + 1..];
    // An empty name is the option Git refuses as "missing environment variable name".
    if variable.is_empty() {
        return None;
    }
    let value: OsString = crate::child_environment::environment_value_by_name_bytes(environment, variable)?;
    // SAFETY: the slice starts at the parameter's own start and ends right before an ASCII `=`,
    //  so both are code point boundaries of the encoded string the bytes came from.
    let key: OsString = unsafe { OsStr::from_encoded_bytes_unchecked(&parameter[..at]) }.to_os_string();
    return Some((key, value));
}

/// What:
///  What one global prefix names:
///  the configuration pairs `-c` and `--config-env` carry,
///  in the order the caller wrote them,
///  and the variables naming the directory,
///  worktree and namespace.
/// Why:
///   The walk consumes a value-taking option's value,
///  so one setting can never be read as another option,
///  and both readers of the prefix walk it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrefixConfiguration = { pairs: [string, string][]; variables: [string, string][] };
/// ```
struct PrefixConfiguration {
    /// The caller's configuration pairs,
    /// newest last.
    pairs: Vec<(OsString, OsString)>,
    /// `GIT_DIR`,
    /// `GIT_WORK_TREE` and `GIT_NAMESPACE`.
    variables: Vec<(OsString, OsString)>,
}

/// Whether one byte of a setting is the `=` Git splits it at.
fn is_equals_byte(byte: &u8) -> bool {
    return *byte == b'=';
}

/// What:
///  The configuration pair one `-c` setting names:
///  the bytes before its first `=`,
///  and the bytes after it,
///  or the boolean true when it holds no `=`.
/// Why:
///   Those bytes are the setting Git applies,
/// so a name or value that is not text must reach the child unchanged;
///  decoding it here would hand `git-lfs` another value than the push used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configPair(setting: Uint8Array): [Uint8Array, Uint8Array];
/// ```
fn config_pair(setting: &[u8]) -> (OsString, OsString) {
    let Some(at) = setting.iter().position(is_equals_byte) else {
        // SAFETY: these bytes are the encoded form of the setting `OsStr`.
        return (unsafe { OsStr::from_encoded_bytes_unchecked(setting) }.to_os_string(), OsString::from("true"));
    };
    // SAFETY: both slices are cut at an ASCII `=`, which is a character boundary of any encoded text.
    return (
        unsafe { OsStr::from_encoded_bytes_unchecked(&setting[..at]) }.to_os_string(),
        unsafe { OsStr::from_encoded_bytes_unchecked(&setting[at + 1..]) }.to_os_string(),
    );
}

/// What:
///  The bytes after the `=` that follows an option's name,
///  or nothing when the token holds no `=` after it.
/// Why:
///   A directory option is written both with a space and with an `=`,
/// and only the second spelling carries the path in the same token.
fn after_equals(rest: &[u8]) -> Option<&[u8]> {
    return rest.strip_prefix(b"=");
}

/// What:
///  Walk a global prefix and answer what it names,
/// or nothing when it carries a `--config-env` Git itself refuses.
/// Why:
///   `-c name=value` is a configuration pair
///  (a `-c` without `=` is the boolean true);
/// `--config-env=<name>=<variable>` is the pair the variable's value sets;
/// `--git-dir`,
/// `--work-tree` and `--namespace` become their variables;
/// `-C` is the working directory and is applied by the caller.
///  Git refuses a `--config-env` whose variable is unset or unnamed,
///  and this answer ends there because no upload may run for a command line Git refuses.
///  Every value is carried as the bytes the caller gave,
///  since a path or a setting that is not text is still the one Git applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prefixConfiguration(prefix: string[], environment): PrefixConfiguration | undefined;
/// ```
fn prefix_configuration(global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Option<PrefixConfiguration> {
    let mut read: PrefixConfiguration = PrefixConfiguration { pairs: Vec::new(), variables: Vec::new() };
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-c" {
            if let Some(setting) = tokens.next() {
                read.pairs.push(config_pair(setting.as_encoded_bytes()));
            }
            continue;
        }
        // What: `--config-env=<name>=<variable>` sets a pair from the environment,
        //  in both of Git's spellings.
        // Why:  Git refuses an option that names no answerable variable,
        //  so a parameter that cannot become a pair ends the answer.
        if bytes == b"--config-env" {
            // An option with no value is one Git refuses,
            //  and `?` ends the whole answer here.
            let parameter: &OsString = tokens.next()?;
            read.pairs.push(config_env_pair(parameter.as_encoded_bytes(), environment)?);
            continue;
        }
        if let Some(named) = bytes.strip_prefix(b"--config-env=") {
            // The text after the option is the whole parameter: `<name>=<variable>`.
            read.pairs.push(config_env_pair(named, environment)?);
            continue;
        }
        for (option, variable) in [
            (b"--git-dir".as_slice(), "GIT_DIR"),
            (b"--work-tree".as_slice(), "GIT_WORK_TREE"),
            (b"--namespace".as_slice(), "GIT_NAMESPACE"),
        ] {
            if bytes == option {
                if let Some(value) = tokens.next() {
                    read.variables.push((OsString::from(variable), value.clone()));
                }
                continue;
            }
            if let Some(value) = bytes.strip_prefix(option).and_then(after_equals) {
                // SAFETY: the slice is cut at an ASCII `=`, which is a character boundary of any encoded text.
                read.variables.push((
                    OsString::from(variable),
                    unsafe { OsStr::from_encoded_bytes_unchecked(value) }.to_os_string(),
                ));
            }
        }
    }
    return Some(read);
}

/// What:
///  The caller's command-line configuration pairs a push carries,
/// in the order Git applies them,
/// or nothing when the prefix names a configuration Git itself refuses.
/// Why:
///   The switch that turns the upload off is read from the same pairs the upload child receives,
///  so the two spellings of a setting cannot push differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandLinePairs(prefix: string[], environment): [string, string][] | undefined;
/// ```
pub fn command_line_pairs(global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Option<Vec<(OsString, OsString)>> {
    return Some(prefix_configuration(global_prefix, environment)?.pairs);
}

/// What:
///  Append one run of text as Git quotes it:
///  between single quotes,
///  with each single quote of the text written as `'\''`.
/// Why:
///  `GIT_CONFIG_PARAMETERS` holds a list of quoted entries,
///  and Git writes a quote inside one the same way,
///  so a value holding a space or a quote reaches the child unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// quoted.push("'" + text.replaceAll("'", "'\\''") + "'");
/// ```
fn quoted_into(quoted: &mut Vec<u8>, text: &[u8]) {
    const QUOTE: u8 = b'\'';
    // What: A single quote ends the quoted run, adds an escaped quote, and reopens the run.
    // Why:  Shell word quoting has no escape character inside single quotes, so this is the only spelling Git accepts.
    const ESCAPED_QUOTE: [u8; 4] = *b"'\\''";
    quoted.push(QUOTE);
    for byte in text {
        if *byte == QUOTE {
            quoted.extend_from_slice(ESCAPED_QUOTE.as_slice());
        } else {
            quoted.push(*byte);
        }
    }
    quoted.push(QUOTE);
}

/// What:
///  The value `GIT_CONFIG_PARAMETERS` carries for the caller's pairs:
///  the value the environment already holds,
///  one quoted `key=value` entry per setting after it,
///  entries separated by spaces,
///  or nothing when the prefix names no pair.
/// Why:
///   Git appends each command-line setting to the parameters it inherited,
///  and reads the parameters after the numbered pairs,
///  so the caller's newer pair is the one that decides.
///  Writing the whole value here, rather than adding numbered pairs,
///  keeps that order:
/// a numbered pair is read before an inherited parameter,
///  and an inherited parameter would decide the upload's server.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parameters = [inherited, ...pairs.map(([k, v]) => `'${k}'='${v}'`)].filter(Boolean).join(' ');
/// ```
fn parameters_value(environment: &[(OsString, OsString)], pairs: &[(OsString, OsString)]) -> Option<OsString> {
    if pairs.is_empty() {
        return None;
    }
    let mut value: Vec<u8> = match environment_value(environment, PARAMETERS_VARIABLE) {
        // An empty value names no entry, as Git treats it.
        Some(existing) if !existing.is_empty() => existing.as_encoded_bytes().to_vec(),
        _ => Vec::new(),
    };
    for (name, setting) in pairs {
        if !value.is_empty() {
            value.push(b' ');
        }
        quoted_into(&mut value, name.as_encoded_bytes());
        value.push(b'=');
        quoted_into(&mut value, setting.as_encoded_bytes());
    }
    // SAFETY: the value holds the encoded bytes of the settings and of the inherited parameters unchanged,
    //  with ASCII quotes, spaces and `=` between them;
    //  encoded text holds no ASCII byte inside a multi-byte character,
    //  so an insertion falls between characters and the result stays valid encoded text.
    return Some(unsafe { OsString::from_encoded_bytes_unchecked(value) });
}

/// What:
///  The variables that carry the global options of a push into a child that reads Git's configuration,
/// or nothing when the prefix names a configuration Git itself refuses.
/// Why:
///   `-c` pairs and `--config-env` pairs travel as the entries Git appends to `GIT_CONFIG_PARAMETERS`
///  after any the environment already holds,
/// `--git-dir`,
/// `--work-tree` and `--namespace` become `GIT_DIR`,
///  `GIT_WORK_TREE` and `GIT_NAMESPACE`;
/// `-C` is the working directory and is applied by the caller.
///  No shell is involved,
/// and an `-c http.extraHeader=...` or `-c lfs.url=...` reaches `git-lfs` as it reached the
///  Git that used to run the hook.
/// Gotcha:
///  `None` is a `--config-env` whose variable is unset,
///  or that names no variable or no variable name.
///  Git refuses that command line itself,
///  so the push never runs and the upload must not either:
///  an object sent now would travel to whichever server the setting that could not be read does not name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function childConfigurationVariables(prefix: string[], environment): [string, string][] | undefined;
/// ```
pub fn child_configuration_variables(global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Option<Vec<(OsString, OsString)>> {
    let mut read: PrefixConfiguration = prefix_configuration(global_prefix, environment)?;
    if let Some(parameters) = parameters_value(environment, read.pairs.as_slice()) {
        read.variables.push((OsString::from(PARAMETERS_VARIABLE), parameters));
    }
    return Some(read.variables);
}

/// Lookups and the real-Git controls of both spellings stay out of the release executable.
#[cfg(test)]
#[path = "lfs_command_line_configuration_tests.rs"]
mod tests;
