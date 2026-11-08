//! The order a lean run takes its mechanisms in:
//! the cheap ones first and the units pass last,
//! so a run that reaches its deadline cuts the pass that can afford to wait
//! instead of the passes behind it.
//! Why:
//! Measured in the first run (2026-11-07T23:52:28-05:00 to 2026-11-08T01:24:39-05:00,
//! `doc/handover/cargo-shadow-lean-build-folders.md`):
//! records and toolchains took 2.5 minutes together,
//! the Podman pass 20.5 minutes,
//! the idle-trees pass 29 minutes,
//! and the units pass was stopped 40 minutes in after about 700 of several thousand profile directories,
//! which in the plan's order starved the Podman pass of the whole three-hour deadline.

use clap::ValueEnum;

use crate::cli::MechanismName;
use crate::lean_command::mechanism_of;
use crate::lean_run::{Mechanism, RUN_ORDER, run_order};

#[test]
fn a_run_starts_with_the_cheap_mechanisms_and_ends_with_the_units_pass() {
    assert_eq!(run_order(&[]), vec![Mechanism::Records, Mechanism::Toolchains, Mechanism::Podman, Mechanism::Trees, Mechanism::Units]);
}

#[test]
fn a_run_of_a_subset_keeps_that_order() {
    assert_eq!(run_order(&[Mechanism::Units, Mechanism::Records]), vec![Mechanism::Records, Mechanism::Units]);
}

#[test]
fn every_mechanism_the_command_line_names_is_in_the_order() {
    let named: Vec<Mechanism> = MechanismName::value_variants().iter().map(|name| { return mechanism_of(*name); }).collect();
    assert_eq!(named.len(), RUN_ORDER.len(), "the command line names {named:?}");
    for mechanism in named {
        assert!(RUN_ORDER.contains(&mechanism), "{mechanism:?} is missing from the run order");
    }
}
