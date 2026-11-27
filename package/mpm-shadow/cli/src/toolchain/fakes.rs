//! What:
//! Test doubles for the two seams:
//! a recording `FakeRunner` and a table-driven `FakeHttp`,
//! with the rig a test builds them into.
//! A test that uses them
//! proves,
//! by what they recorded,
//! that nothing reached the real host or the network.
//! The doubles are one file each,
//! named for what each holds:
//! the runner (`the_fake_runner`),
//! the network (`the_fake_network`),
//! and the rig that holds both with a scope on disk (`the_test_rig`).
//! A caller imports from this file and never from a part.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // fakes.ts
//! export * from './fakes/the_fake_runner.ts';
//! export * from './fakes/the_fake_network.ts';
//! export * from './fakes/the_test_rig.ts';
//! ```

/// The network fake,
/// which answers from a table.
mod the_fake_network;
/// The recording runner,
/// which answers from rules.
mod the_fake_runner;
/// The rig a test builds around both fakes.
mod the_test_rig;

/// Re-exports the table-driven network.
pub(crate) use self::the_fake_network::FakeHttp;
/// Re-exports the recording runner.
pub(crate) use self::the_fake_runner::FakeRunner;
/// Re-exports the rig,
/// and the write of the stand-in program an install is given.
pub(crate) use self::the_test_rig::{
    Rig,
    write_executable,
};
