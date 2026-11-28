//! What:
//!  How many forces one `git worktree remove` carried,
//!  counted the way Git counts them,
//!  and what each count passes.
//! Why:
//!  The project decided on 2026-11-28 that two forces pass every refusal a
//!  removal has,
//!  and that one force passes the two states the removal itself can answer for,
//!  with the coordinator's pick of 2026-11-28 adding Git's own cleanliness
//!  refusal to the one force's way through:
//!  "Two force does everything.
//! One force - you pick."
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
//!  "What force does on a ZFS removal,
//! two refusals without it,
//! and the foundation pick").
//!  One value carries the count from the words a person typed to every step that
//!  reads it,
//!  so no step counts the words again and no two steps can disagree.
//!  Git's own gradation is the one this follows:
//!  `builtin/worktree.c:1450` reads a lock only while `force < 2`,
//!  and `:1464` runs the cleanliness check only when `force` is zero
//!  (read at Git `v2.56.1`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const force: Force = { count: 2 };
//! if (force.isTwice()) { /* two forces pass every refusal */ }
//! ```

/// What:
///  The number of `-f` and `--force` readings one command carried,
///  as Git's own option parser counted them.
/// Why:
///  Every decision force makes is a comparison with Git's own gradation,
///  and one type keeps those comparisons in one place rather than a bare number
///  each step reads with its own threshold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalForce = { count: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RemovalForce {
    count: usize,
}

impl RemovalForce {
    /// No force was given,
    /// which is every removal that names no `-f` and no `--force`.
    pub const fn none() -> RemovalForce {
        return RemovalForce {
            count: 0,
        };
    }

    /// The count a command's words gave.
    ///
    /// Why:
    ///  A `--no-force` reading sets Git's count back to zero,
    ///  so a count that is no longer monotonic in the words is still one number here.
    pub const fn of_count(count: usize) -> RemovalForce {
        return RemovalForce {
            count,
        };
    }

    /// The count itself,
    /// for the words a sentence prints.
    pub const fn count(self) -> usize {
        return self.count;
    }

    /// What:
    ///  Whether at least one force was given.
    /// Why:
    ///  One force passes a destruction plan that cannot be read,
    ///  a clone at the staging folder the record's GUID proves,
    ///  and Git's own cleanliness refusal,
    ///  so three decisions read this one question.
    pub const fn is_given(self) -> bool {
        return self.count >= 1;
    }

    /// What:
    ///  Whether two forces were given,
    ///  which is what passes every refusal.
    /// Why:
    ///  A person's lock,
    ///  a clone the wrapper cannot prove and a slot whose worktree holds changes
    ///  all refuse one force or none,
    ///  and Git's own words for the same gradation are `-f -f`
    ///  (`builtin/worktree.c:1450` at Git `v2.56.1`).
    pub const fn is_twice(self) -> bool {
        return self.count >= 2;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_force_tests.rs"]
mod tests;
