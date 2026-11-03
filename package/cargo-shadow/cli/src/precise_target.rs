//! What:
//! The packages and the release a `cargo update --precise` names,
//! read from the arguments, and the locked entries that match them.
//! Why:
//! `cargo update -p foo --precise 1.0.1` leaves the lockfile unchanged when `foo@1.0.1` is already locked,
//! so a check of only what the command added never sees the release the user asked for.
//! The named release is held to the policy whether or not the command changed the lockfile.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // precise-target.ts
//! export function preciseTargets(args: { cargoArgs: string[]; locked: LockedVersion[] }): LockedVersion[]
//! ```

/// What:
/// `use crate::lockfile::LockedVersion;` imports the name and version pair of a locked registry entry.
/// Why:
/// The matching entries are returned in the same shape the policy checks take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockedVersion } from 'crate/lockfile';
/// ```
use crate::lockfile::LockedVersion;

/// What:
/// `fn package_name(spec: &str) -> &str` takes the crate name out of a cargo package spec
/// (`name`, `name@1.2.3`, `https://host/path#name@1.2.3`).
/// Why:
/// Only the name selects which locked entries the `--precise` release applies to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packageName = (spec: string): string => (spec.split('#').pop() ?? spec).split('@')[0];
/// ```
fn package_name(spec: &str) -> &str {
    let after_url: &str = spec.rsplit('#').next().unwrap_or(spec);
    return after_url.split('@').next().unwrap_or(after_url);
}

/// What:
/// `fn option_values<'a>(arguments: &'a [String], long: &str, short: Option<&str>) -> Vec<&'a str>` collects every value given to one
/// option in the forms `--long value`, `--long=value`, `-s value`, `-s=value` and `-svalue`.
/// Why:
/// `-p` may be given more than once and cargo accepts all of these spellings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionValues(args: string[], long: string, short?: string): string[]
/// ```
fn option_values<'a>(arguments: &'a [String], long: &str, short: Option<&str>) -> Vec<&'a str> {
    let mut values: Vec<&'a str> = Vec::new();
    let mut index: usize = 0;
    while index < arguments.len() {
        let argument: &str = arguments[index].as_str();
        if argument == long || Some(argument) == short {
            if let Some(next) = arguments.get(index + 1) {
                values.push(next.as_str());
            }
            index += 2;
            continue;
        }
        if let Some(attached) = argument.strip_prefix(&format!("{long}=")) {
            values.push(attached);
        } else if let Some(letter) = short {
            if let Some(attached) = argument.strip_prefix(letter) {
                values.push(attached.strip_prefix('=').unwrap_or(attached));
            }
        }
        index += 1;
    }
    return values;
}

/// What:
/// `pub fn preciseTargets(cargo_args, locked) -> Vec<LockedVersion>` returns the locked entries that a
/// `cargo update --package NAME --precise VERSION` names.
/// Why:
/// An empty result means the arguments name no precise release (or no locked entry has it), so nothing extra is checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function preciseTargets(cargoArgs: string[], locked: LockedVersion[]): LockedVersion[]
/// ```
pub fn precise_targets(cargo_args: &[String], locked: &[LockedVersion]) -> Vec<LockedVersion> {
    let precise: Vec<&str> = option_values(cargo_args, "--precise", None);
    let Some(version) = precise.first() else {
        return Vec::new();
    };
    let names: Vec<&str> = option_values(cargo_args, "--package", Some("-p")).into_iter().map(package_name).collect();
    return locked.iter().filter(|entry| { return entry.version == *version && names.contains(&entry.name.as_str()); }).cloned().collect();
}
