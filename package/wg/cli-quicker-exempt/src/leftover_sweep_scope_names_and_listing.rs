//! The names of the scopes the sweep may touch,
//! the manager's plain listing of them,
//! and the split of that listing into the two designs.

/// The end of a scope name,
/// and what the sweep was set up with.
use super::{
    OLD_SCOPE_SUFFIX,
    SweepSetup,
};

/// Reports an ASCII digit byte.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Reports a nonempty run of digits.
fn is_digits(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(is_digit_byte);
}

/// Reports whether a unit name is one of the earlier design:
/// the prefix,
/// two runs of digits joined by a dash,
/// and `.scope`.
/// The scopes of the current design are named from a path and never match.
pub fn is_old_style_scope_name(
    name: &str,
    prefix: &str,
) -> bool {
    let Some(after_prefix) = name.strip_prefix(prefix) else {
        return false;
    };
    let Some(rest) = after_prefix.strip_suffix(OLD_SCOPE_SUFFIX) else {
        return false;
    };
    let Some((pid, start_time)) = rest.split_once('-') else {
        return false;
    };
    return is_digits(pid) && is_digits(start_time);
}

/// Reports whether a unit name is a scope that holds the processes of one origin:
/// the prefix,
/// a name made from a cgroup path,
/// and `.scope`.
/// The names of the earlier design are two runs of digits,
/// which a path never is,
/// so the two kinds never overlap.
pub fn is_origin_scope_name(
    name: &str,
    prefix: &str,
) -> bool {
    if is_old_style_scope_name(name, prefix) {
        return false;
    }
    let Some(rest) = name.strip_prefix(prefix) else {
        return false;
    };
    let Some(stem) = rest.strip_suffix(OLD_SCOPE_SUFFIX) else {
        return false;
    };
    return !stem.is_empty();
}

/// One unit of the manager's list.
pub struct ListedUnit {
    /// Unit name.
    pub name: String,
    /// Active state,
    /// such as `active` or `failed`.
    pub active: String,
}

/// Reads the manager's plain list:
/// the name,
/// the load state and the active state are the first three words of each line.
pub(super) fn parse_listing(text: &str) -> Vec<ListedUnit> {
    let mut units: Vec<ListedUnit> = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let (Some(name), Some(_load), Some(active)) = (words.next(), words.next(), words.next())
        else {
            continue;
        };
        units.push(ListedUnit {
            name: name.to_string(),
            active: active.to_string(),
        });
    }
    return units;
}

/// The scopes of the earlier design that the sweep may reset and stop,
/// and the origin scopes it may stop while they hold nothing.
pub struct SweepTargets<'a> {
    /// The scopes of the earlier design.
    pub per_process: Vec<&'a ListedUnit>,
    /// The scopes that hold the processes of one origin.
    pub origins: Vec<&'a ListedUnit>,
}

/// Splits the listed units into the two designs.
pub fn targets<'a>(
    setup: &SweepSetup,
    units: &'a [ListedUnit],
) -> SweepTargets<'a> {
    let mut split = SweepTargets {
        per_process: Vec::new(),
        origins: Vec::new(),
    };
    for unit in units {
        if is_old_style_scope_name(&unit.name, &setup.prefix) {
            split.per_process.push(unit);
        } else if is_origin_scope_name(&unit.name, &setup.prefix) {
            split.origins.push(unit);
        }
    }
    return split;
}
