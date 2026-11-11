//! What:
//!  The Git configuration a push carries on its command line,
//! turned into the variables a child that reads Git's configuration receives.
//! Why:
//!  The Git LFS upload starts `git-lfs` itself,
//! so whatever the caller selected with `-c` or `--config-env` must be spelled here
//!  as the numbered variables Git would hand a hook.
//!  A setting that is lost on this way is an upload sent to another server than the ref:
//! the object lands where the pointer is not,
//!  and every collaborator of the remote that got the pointer cannot fetch its content.
//!  It sits beside the run of the upload so that each file passes the package's length limit.

/// The invocation's environment values.
use crate::child_environment::environment_value_by_name_bytes;
/// How many numbered configuration pairs the environment already holds.
use crate::lfs_pre_push::environment_config_count;
/// Owned OS text for the variables and their values.
use std::ffi::OsString;

/// What:
///  The configuration pair a `--config-env` parameter names,
/// read from the environment of the invocation,
/// or nothing when Git itself refuses the option as it stands.
/// Why:
///   `git-lfs` reads Git's configuration through the numbered variables only,
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
    let value: OsString = environment_value_by_name_bytes(environment, variable)?;
    // SAFETY: the slice starts at the parameter's own start and ends right before an ASCII `=`,
    //  so both are code point boundaries of the encoded string the bytes came from.
    let key: OsString = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(&parameter[..at]) }.to_os_string();
    return Some((key, value));
}

/// What:
///  The variables that carry the global options of a push into a child that reads Git's configuration,
/// or nothing when the prefix names a configuration Git itself refuses.
/// Why:
///   `-c name=value` becomes `GIT_CONFIG_KEY_n`/`GIT_CONFIG_VALUE_n` after any pairs the environment already holds
///  (a `-c` without `=` is the boolean true),
/// `--config-env=<name>=<variable>` as the pair the variable's own value sets,
/// `--git-dir`,
/// `--work-tree` and `--namespace` become `GIT_DIR`,
///  `GIT_WORK_TREE` and `GIT_NAMESPACE`;
/// `-C` is the working directory and is applied by the caller.
///  No shell is involved,
/// and an `-c http.extraHeader=...` or `-c lfs.url=...` reaches `git-lfs` as it reached the Git that used to run the hook.
/// Gotcha:
///  `None` is a `--config-env` whose variable is unset,
///  or that names no variable or no variable name.
///  Git refuses that command line itself,
///  so the push never runs and the upload must not either:
///  an object sent now would travel to whichever server the setting that could not be read does not name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prefixVariables(globalPrefix: string[], environment): [string, string][] | undefined;
/// ```
pub fn prefix_variables(global_prefix: &[OsString], environment: &[(OsString, OsString)]) -> Option<Vec<(OsString, OsString)>> {
    let mut pairs: Vec<(OsString, OsString)> = Vec::new();
    let mut variables: Vec<(OsString, OsString)> = Vec::new();
    let mut tokens = global_prefix.iter();
    while let Some(token) = tokens.next() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"-c" {
            if let Some(setting) = tokens.next() {
                let setting_text: String = setting.to_string_lossy().into_owned();
                let (name, value) = setting_text.split_once('=').unwrap_or((setting_text.as_str(), "true"));
                pairs.push((OsString::from(name), OsString::from(value)));
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
            pairs.push(config_env_pair(parameter.as_encoded_bytes(), environment)?);
            continue;
        }
        if let Some(named) = bytes.strip_prefix(b"--config-env=") {
            // The text after the option is the whole parameter: `<name>=<variable>`.
            pairs.push(config_env_pair(named, environment)?);
            continue;
        }
        let text: String = token.to_string_lossy().into_owned();
        for (option, variable) in [("--git-dir", "GIT_DIR"), ("--work-tree", "GIT_WORK_TREE"), ("--namespace", "GIT_NAMESPACE")] {
            if text == option {
                if let Some(value) = tokens.next() {
                    variables.push((OsString::from(variable), value.clone()));
                }
            } else if let Some(value) = text.strip_prefix(format!("{option}=").as_str()) {
                variables.push((OsString::from(variable), OsString::from(value)));
            }
        }
    }
    let existing: usize = environment_config_count(environment);
    if !pairs.is_empty() {
        variables.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from((existing + pairs.len()).to_string())));
    }
    for (offset, (name, value)) in pairs.into_iter().enumerate() {
        variables.push((OsString::from(format!("GIT_CONFIG_KEY_{}", existing + offset)), name));
        variables.push((OsString::from(format!("GIT_CONFIG_VALUE_{}", existing + offset)), value));
    }
    return Some(variables);
}

/// Lookups and the real-Git controls of both spellings stay out of the release executable.
#[cfg(test)]
#[path = "lfs_upload_child_configuration_tests.rs"]
mod tests;
