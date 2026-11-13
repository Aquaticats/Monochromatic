//! What:
//!  Which of two release texts of this program is the newer one:
//!  `0.6.1-alpha.13` against `0.6.2`, and `0.6.1-alpha.13` against `0.6.1-alpha.14`.
//! Why:
//!   A client that meets a service of another release has to decide one thing:
//! keep it or replace it.
//! Until this release the answer was always "replace", because the service was started by the service manager
//! from a unit that named a program an upgrade removed, so a service of another release could be a program that
//! was no longer there (`doc/planning/cli-git-no-self-invocation.md`).
//! With the wrapper starting its own service, a service of a newer release is a better program that is running,
//! and asking it to leave is what made a changeover cost a commit
//! (audit of 2026-08-..., the hundred retirements):
//!  the newer service exits, the account's manager starts the newer program again, and the older client asks again.
//!  So the client keeps a service that is at least as new as its own and replaces one that is older,
//!  which is the direction that removes bugs and never the direction that reintroduces them.
//! A text this program cannot order is treated as "not proven newer" and the old answer stands:
//!  the service is asked to leave,
//! because keeping a service whose version cannot be read is how a broken service outlives its fix.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! compare('0.6.1-alpha.13', '0.6.1-alpha.14') // 'less'
//! compare('0.6.2', '0.6.1-alpha.99') // 'greater'
//! compare('nonsense', '0.6.1') // undefined
//! ```

/// What one release text says.
/// The parts are numbers because every release this program ships is `major.minor.patch` and an optional
/// pre-release whose name ends in a number (`-alpha.13`);
/// a text that does not parse names nothing.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Release {
    /// The first three numbers, in order.
    numbers: [u64; 3],
    /// The pre-release's name and number, and nothing for a final release.
    pre_release: Option<(String, u64)>,
}

/// What:
///  The order of two release texts,
///  and nothing when either cannot be read.
/// Why:
///   `Ordering::Greater` means the first text names the newer release;
/// the caller reads that as "keep that service".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// compare(one, other) // 'less' | 'equal' | 'greater' | undefined
/// ```
pub fn compare(one: &str, other: &str) -> Option<std::cmp::Ordering> {
    let left: Release = parse(one)?;
    let right: Release = parse(other)?;
    let numbers: std::cmp::Ordering = left.numbers.cmp(&right.numbers);
    if numbers != std::cmp::Ordering::Equal {
        return Some(numbers);
    }
    return match (&left.pre_release, &right.pre_release) {
        // A final release is newer than any pre-release of the same numbers.
        (None, None) => Some(std::cmp::Ordering::Equal),
        (None, Some(_)) => Some(std::cmp::Ordering::Greater),
        (Some(_), None) => Some(std::cmp::Ordering::Less),
        // Two pre-releases: the name in the order they are spelled, then the number as a number (alpha.9 < alpha.13).
        (Some((left_name, left_number)), Some((right_name, right_number))) => {
            let names: std::cmp::Ordering = left_name.cmp(right_name);
            return Some(if names != std::cmp::Ordering::Equal { names } else { left_number.cmp(right_number) });
        }
    };
}

/// One release text as its parts, or nothing when it is not one.
fn parse(spelled: &str) -> Option<Release> {
    let (numbers, tail): (&str, Option<&str>) = match spelled.split_once('-') {
        Some((numbers, tail)) => (numbers, Some(tail)),
        None => (spelled, None),
    };
    let mut parts: std::str::Split<'_, char> = numbers.split('.');
    let mut found: [u64; 3] = [0; 3];
    for slot in &mut found {
        *slot = parts.next()?.parse().ok()?;
    }
    // A fourth number is not a release this program ships.
    if parts.next().is_some() {
        return None;
    }
    let pre_release: Option<(String, u64)> = match tail {
        None => None,
        Some(spelling) => {
            let (name, number) = spelling.rsplit_once('.')?;
            if name.is_empty() {
                return None;
            }
            Some((name.to_string(), number.parse().ok()?))
        }
    };
    return Some(Release { numbers: found, pre_release });
}

/// Controls of the order stay out of the release executable.
#[cfg(test)]
#[path = "service_release_order_tests.rs"]
mod tests;
