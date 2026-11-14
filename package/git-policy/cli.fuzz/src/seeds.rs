//! What:
//!  Every seed of `seed/wrapper_controls/`,
//!  as the answers each one asks for and the arguments it spells.
//! Why:
//!  A seed's first bytes are the repository answers of the frontier view,
//!  and the rest is the argument list,
//!  so a seed's bytes are a function of two things a person can read:
//!  which answers it asks for and what it spells.
//!  Keeping the two apart from their encoding means a seed survives a wider mode:
//!  `Seed::bytes` lays the answers out in `MODE_BYTES`,
//!  so widening `Mode` re-encodes every seed,
//!  `mise run //package/git-policy/cli.fuzz:seed:write` writes them,
//!  and `controls::tests::every_seed_spells_its_scenario` fails until it is run.
//!  Writing the bytes by hand instead is what let nine seeds lose their spelling when the mode
//!  widened from one byte to two.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Seed = { name: string; answers: number[]; arguments: Uint8Array[] };
//! const bytes = (seed: Seed): Uint8Array => concat(le(seed.answers), joinNul(seed.arguments));
//! ```

/// The answers' bit constants, and the width a seed spends on them.
use crate::controls::Mode;

/// What:
///  One seed: its file's name,
///  the answers it asks for,
///  and the arguments it spells.
/// Why:
///   The name is what the control and the reader call the scenario,
///  the answers are the bits of the mode the frontier view reads,
///  and the arguments are what `arguments_from_bytes` must split the rest into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Seed = { name: string; answers: number[]; arguments: Uint8Array[] };
/// ```
pub struct Seed {
    /// The file's name under `seed/wrapper_controls/`,
    ///  which is also the scenario it spells.
    pub name: &'static str,
    /// The answers it asks for,
    ///  each the bit constant `controls` names.
    pub answers: &'static [Mode],
    /// The arguments it spells,
    ///  byte for byte,
    ///  so a seed can hold what no text does.
    pub arguments: &'static [&'static [u8]],
}

/// What:
///  NUL,
///  which separates two arguments in every view the target reads.
/// Why:
///   `arguments_from_bytes` splits at it,
///  so a seed's arguments are written with it and no argument may hold one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ARGUMENT_SEPARATOR = 0x00;
/// ```
pub const ARGUMENT_SEPARATOR: u8 = 0x00;

impl Seed {
    /// What:
    ///  The answers as one mode,
    ///  each answer's bit set.
    /// Why:
    ///   A seed asks for answers by name,
    ///  and the frontier view takes them packed into one integer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const mode = (seed: Seed): number => seed.answers.reduce((all, bit) => all | bit, 0);
    /// ```
    pub fn mode(&self) -> Mode {
        // `mut` holds the answers gathered so far.
        let mut gathered: Mode = 0;
        for answer in self.answers {
            gathered |= *answer;
        }
        return gathered;
    }

    /// What:
    ///  The bytes of this seed: its answers little endian in the mode's own width,
    ///  then its arguments NUL separated.
    /// Why:
    ///   This is the one place a seed's bytes are decided,
    ///  so the file the campaign reads,
    ///  the file the writer writes and the control that judges both cannot disagree,
    ///  and a wider mode re-encodes every seed without a byte count to keep in step.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const bytes = (seed: Seed): Uint8Array => concat(littleEndian(seed.mode(), MODE_BYTES), seed.arguments.flatMap((one, i) => i === 0 ? one : [0, ...one]));
    /// ```
    pub fn bytes(&self) -> Vec<u8> {
        // `.to_le_bytes()` is exactly as wide as the mode's own type, which is what the target reads.
        let mut written: Vec<u8> = self.mode().to_le_bytes().to_vec();
        for (index, argument) in self.arguments.iter().enumerate() {
            if index > 0 {
                written.push(ARGUMENT_SEPARATOR);
            }
            written.extend_from_slice(argument);
        }
        return written;
    }
}

/// What:
///  Every seed, ordered by name.
/// Why:
///   The writer and the control read this one list,
///  so neither can spell a scenario the other does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SEEDS: Seed[] = [/* ... */];
/// ```
pub const SEEDS: &[Seed] = &[
    Seed {
        name: "abort-over-an-ignored-file",
        answers: &[
            crate::controls::LINKED_WORKTREE,
            crate::controls::OPERATION_WAITS,
            crate::controls::IGNORED_AND_TRACKED,
        ],
        arguments: &[b"am", b"--abort"],
    },
    Seed {
        name: "abort-over-an-untracked-file",
        answers: &[
            crate::controls::LINKED_WORKTREE,
            crate::controls::OPERATION_WAITS,
            crate::controls::UNTRACKED_AND_TRACKED,
        ],
        arguments: &[b"rebase", b"--abort"],
    },
    Seed {
        name: "abort-with-worktree-work",
        answers: &[
            crate::controls::LINKED_WORKTREE,
            crate::controls::OPERATION_WAITS,
            crate::controls::WORKTREE_WORK,
        ],
        arguments: &[b"merge", b"--abort"],
    },
    Seed {
        name: "commit-hatch-as-message",
        answers: &[],
        arguments: &[b"commit", b"-m", b"--no-enforce-only", b"--no-enforce-only"],
    },
    Seed {
        name: "control-as-global-value",
        answers: &[crate::controls::LINKED_WORKTREE],
        arguments: &[b"-C", b"--cli-git-keep-going", b"status"],
    },
    Seed {
        name: "dry-run-commit",
        answers: &[],
        arguments: &[b"commit", b"--dry-run", b"file"],
    },
    Seed {
        name: "hatch-after-separator",
        answers: &[crate::controls::LINKED_WORKTREE],
        arguments: &[b"reset", b"--", b"--no-enforce-worktree"],
    },
    Seed {
        name: "hatch-before-separator",
        answers: &[],
        arguments: &[b"reset", b"--no-enforce-worktree", b"--hard"],
    },
    Seed {
        name: "keep-going-before-command",
        answers: &[],
        arguments: &[b"--cli-git-keep-going", b"status"],
    },
    Seed {
        name: "non-utf8",
        answers: &[crate::controls::LINKED_WORKTREE],
        // Bytes the wrapper must carry unchanged, which no text holds as UTF-8.
        arguments: &[b"\xff\xfe", b"--no-enforce-only\xff", b"--"],
    },
    Seed {
        name: "refused-region",
        answers: &[],
        arguments: &[b"push", b"--no-such-option", b"--cli-git-keep-going"],
    },
    Seed {
        name: "shared-remote-key",
        answers: &[crate::controls::LINKED_WORKTREE],
        arguments: &[b"config", b"remote.origin.url", b"url"],
    },
    Seed {
        name: "shared-remote-settings",
        answers: &[crate::controls::LINKED_WORKTREE],
        arguments: &[b"remote", b"set-url", b"origin", b"url"],
    },
    Seed {
        name: "skip-with-staged-work",
        answers: &[
            crate::controls::LINKED_WORKTREE,
            crate::controls::OPERATION_WAITS,
            crate::controls::STAGED_WORK,
        ],
        arguments: &[b"cherry-pick", b"--skip"],
    },
    Seed {
        name: "worktree-opt-out",
        answers: &[crate::controls::LINKED_WORKTREE],
        arguments: &[b"worktree", b"add", b"--no-worktree-copy", b"../topic"],
    },
];
