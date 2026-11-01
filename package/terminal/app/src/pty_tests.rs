// What:     Unit tests for `pty.rs`, pulled in by
//           `#[cfg(test)] #[path = "pty_tests.rs"] mod tests;` at
//           the bottom of `pty.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of pty.
// Why:      Keep the tests beside the code without inflating
//           `pty.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::{...};` imports PTY items from the parent module.
// Why:      Tests exercise the public spawn path and event enum.
use super::{PtyEvent, PtySession};
// What:     `use crate::engine::ViewportGeometry;` imports the shared geometry type.
// Why:      PTY tests need the same size object as the app.
use crate::engine::ViewportGeometry;
// What:     `use crate::scroll::DEFAULT_CELL_WIDTH_PX;` imports the shared
//           terminal cell width. Sibling constants include cell height and
//           scroll mapping helpers.
// Why:      The PTY resize fixture should stay aligned with UI and engine metrics.
use crate::scroll::DEFAULT_CELL_WIDTH_PX;
// What:     `use portable_pty::CommandBuilder;` imports the PTY command builder.
// Why:      The test spawns a deterministic shell command.
use portable_pty::CommandBuilder;
// What:     `use std::{...};` imports channels and timeouts for the test.
// Why:      The test waits for output without blocking forever.
use std::{sync::mpsc, time::Duration};
// What:     `use anyhow::Result;` imports the same one-parameter error result
//           alias the production PTY functions return.
// Why:      The test can use `?` across PTY, channel, and UTF-8-adjacent helpers
//           without naming one boxed trait-object error type.
use anyhow::Result;

#[test]
fn spawns_command_and_reads_output() -> Result<()> {
    let geometry = ViewportGeometry {
        cols: 20,
        rows: 4,
        cell_width_px: DEFAULT_CELL_WIDTH_PX,
        cell_height_px: 18.0,
    };
    let (sender, receiver) = mpsc::channel();
    let mut command = CommandBuilder::new("/bin/sh");
    command.arg("-lc");
    command.arg("printf terminal-pty-test");
    let _session = PtySession::spawn_command(geometry, command, sender)?;
    let event = receiver.recv_timeout(Duration::from_secs(5))?;
    if let PtyEvent::Output(bytes) = event {
        let text = String::from_utf8_lossy(bytes.as_slice());
        assert!(text.contains("terminal-pty-test"));
    } else {
        panic!("expected PTY output event");
    }
    return Ok(())
}
