//! What:
//! The mechanisms a lean run takes,
//! what the caller asked for,
//! the order they are taken in,
//! and the long pass each one continues.
//! Why:
//! One deadline covers the whole run,
//! so the order decides which mechanism it cuts,
//! and the order is decided here once and read by the run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Mechanism = 'Trees' | 'Units' | 'Records' | 'Toolchains' | 'Podman';
//! type RunOptions = { dryRun: boolean; only: Mechanism[]; deadline: Duration; start: string };
//! const RUN_ORDER: Mechanism[]
//! export function runOrder(requested: Mechanism[]): Mechanism[]
//! export function longPass(mechanism: Mechanism): LongPass | undefined
//! ```
/// Imports the long pass a mechanism continues between runs.
use crate::lean_progress::LongPass;
/// Imports the owned path type the run starts from.
use std::path::PathBuf;
/// Imports the duration the whole run may take.
use std::time::Duration;
/// The mechanisms of the run,
/// by the name `--only` takes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mechanism {
    /// C:
    /// idle trees of the host build base.
    Trees,
    /// B and E:
    /// the older-compiler sweep and the size caps of profile directories.
    Units,
    /// D:
    /// verification run records.
    Records,
    /// Dated nightly toolchains no checkout pins.
    Toolchains,
    /// Unused Podman images and volumes.
    Podman,
}

/// What the caller asked for.
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// Report what would be removed and remove nothing.
    pub dry_run: bool,
    /// The mechanisms to run (all when empty).
    pub only: Vec<Mechanism>,
    /// The longest the whole run may take.
    pub deadline: Duration,
    /// The directory the run starts from,
    /// inside a checkout.
    pub start: PathBuf,
}

/// The mechanisms in the order a run takes them:
/// cheapest first,
/// the units pass last.
/// Why the order is this and not the plan's:
/// one deadline covers the whole run,
/// and the measured cost of the first run (2026-11-07,
/// `doc/handover/cargo-shadow-lean-build-folders.md`,
/// section "First run") is
/// 2.5 minutes for records and toolchains together,
/// 20.5 minutes for Podman,
/// 29 minutes for the idle trees,
/// and hours for the units pass,
/// which reached about 700 of several thousand profile directories in 40 minutes.
/// A pass that a deadline cuts off waits for the next run;
/// a pass put last therefore starves only itself.
pub const RUN_ORDER: [Mechanism; 5] = [
    Mechanism::Records,
    Mechanism::Toolchains,
    Mechanism::Podman,
    Mechanism::Trees,
    Mechanism::Units,
];

/// What:
/// `pub fn run_order(requested: &[Mechanism]) -> Vec<Mechanism>` is the mechanisms of a run in the order it takes them.
/// Why:
/// The deadline cuts what it reaches first,
/// so the order is decided here once and the run walks it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runOrder(requested: Mechanism[]): Mechanism[]
/// ```
pub fn run_order(requested: &[Mechanism]) -> Vec<Mechanism> {
    let mut order: Vec<Mechanism> = Vec::new();
    for mechanism in RUN_ORDER {
        if requested.is_empty() || requested.contains(&mechanism) {
            order.push(mechanism);
        }
    }
    return order;
}

/// What:
/// `pub fn long_pass(mechanism: Mechanism) -> Option<LongPass>` is the long pass a mechanism continues,
/// when it is one.
/// Why:
/// Only the idle-trees pass and the units pass are longer than a run's bound,
/// so only they keep a position between runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const longPass = (mechanism: Mechanism): LongPass | undefined => mechanism === Mechanism.Trees ? LongPass.Trees : mechanism === Mechanism.Units ? LongPass.Units : undefined;
/// ```
pub fn long_pass(mechanism: Mechanism) -> Option<LongPass> {
    return match mechanism {
        Mechanism::Trees => Some(LongPass::Trees),
        Mechanism::Units => Some(LongPass::Units),
        Mechanism::Records | Mechanism::Toolchains | Mechanism::Podman => None,
    };
}
