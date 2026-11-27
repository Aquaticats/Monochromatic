//! A complete reader with the scripted language server,
//!  for window tests of language navigation.
//!
//! `test:native` runs every test in its own process (nextest's default),
//!  so a test may make its
//! disposable project the working directory,
//!  as the application does at startup;
//!  Helix roots
//! servers there.
//!  The scripted server is the `ide-scripted-lsp` binary beside the test binary,
//! built by the task before the tests run;
//!  its answers come from `IDE_SCRIPTED_*` variables.

/// Waiting for timers and for the scripted server,
///  in their own module.
pub(super) use super::test_waits::ready;

/// Name of the scripted server definition.
pub(super) const SERVER: &str = "scripted-ls";

/// Where source text starts in the window,
///  derived in `sidebar_tests` from the tree,
///  divider,
///  and gutter widths.
pub(super) use super::super::sidebar_tests::TEXT_LEFT;

/// Source rows start below the 32 px file label.
pub(super) const TEXT_TOP: f32 = 32.0;

/// The disposable project,
///  the scripted server it configures,
///  a reader over it,
///  and the input the tests send.
mod project;
/// A reader with every production binding plus language navigation.
mod reader;
/// Keys,
///  window points,
///  and pointer events.
mod window_input;

/// What a test opens and the server behind it.
pub(super) use project::{
    Project,
    address,
    definitions,
    location,
    project,
};
/// A window over a disposable project,
///  and the waits that settle it.
pub(super) use reader::{
    LanguageReader,
    idle,
    reader,
    reader_launching,
    reader_with,
};
/// The input a language test sends.
pub(super) use window_input::{
    above,
    caret,
    caret_x,
    click,
    control_click,
    definition,
    escape,
    head,
    hover,
    move_to,
    point,
    popup,
    rows,
};
