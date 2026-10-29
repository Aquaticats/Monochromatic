// End-user boundary test: drives the sticky-variant GTK app inside this repo's nested Wayland
// compositor (package/nested-wayland-session/cli) with synthetic keyboard input and asserts the
// observed-state JSON the app mirrors, using the same steps and key set as the Electron
// prototype's boundary test (package/file-manager/variant-electron). Exempt from
// require-rustdoc/max-lines because it lives under tests/. Run via the package's `test:wayland`
// mise task; `#[ignore]` keeps it out of plain unit-test runs since it needs a parent Wayland
// session and the compositor release binary.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const POLL_INTERVAL: Duration = Duration::from_millis(50);
const DEADLINE: Duration = Duration::from_secs(10);

fn manifest_dir() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn compositor_binary() -> PathBuf {
    return manifest_dir()
        .join("../../nested-wayland-session/cli/target/release/monochromatic-nested-wayland-session")
}

fn app_binary() -> PathBuf {
    return manifest_dir().join("target/release/monochromatic-file-manager-gtk-sticky")
}

/// Creates the throwaway fixture tree:
///  alpha/ (with one nested directory and one file),
///  beta/
/// (empty),
///  and one root-level file,
///  so the sorted root listing is deterministic:
/// alpha,
///  beta,
///  readme.txt.
fn create_fixture_tree(root: &Path) {
    std::fs::create_dir_all(root.join("alpha/nested-one")).expect("fixture alpha/nested-one");
    std::fs::create_dir_all(root.join("beta")).expect("fixture beta");
    std::fs::write(root.join("alpha/note.txt"), "fixture note\n").expect("fixture note");
    std::fs::write(root.join("readme.txt"), "fixture readme\n").expect("fixture readme");
}

struct ControlSocket {
    reader: BufReader<UnixStream>,
}

impl ControlSocket {
    fn connect(path: &Path) -> Self {
        let deadline = Instant::now() + DEADLINE;
        loop {
            if let Ok(stream) = UnixStream::connect(path) {
                return Self { reader: BufReader::new(stream) };
            }
            assert!(Instant::now() < deadline, "control socket never appeared at {path:?}");
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// Sends one command line and asserts the compositor answers `ok`.
    fn expect_ok(&mut self, command: &str) {
        self.reader
            .get_mut()
            .write_all(format!("{command}\n").as_bytes())
            .expect("write control command");
        let mut response = String::new();
        self.reader.read_line(&mut response).expect("read control response");
        assert!(
            response.starts_with("ok"),
            "command {command:?} answered {response:?}"
        );
    }
}

/// Polls the state file until every expected key matches,
///  panicking past the deadline with the
/// last observed state for diagnosis.
fn wait_for_state(path: &Path, expected: &[(&str, Value)]) {
    let deadline = Instant::now() + DEADLINE;
    let mut last = String::new();
    while Instant::now() < deadline {
        if let Ok(body) = std::fs::read_to_string(path) {
            last = body.clone();
            if let Ok(Value::Object(state)) = serde_json::from_str::<Value>(&body)
                && expected
                    .iter()
                    .all(|(key, value)| return state.get(*key) == Some(value))
            {
                return;
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    panic!("timed out waiting for {expected:?}; last observed state: {last}");
}

/// Kills the compositor if the test panics midway so no orphan session lingers.
struct KillOnDrop(Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "needs a parent Wayland session and the nested-wayland-session release binary; run via the test:wayland mise task"]
fn sticky_layout_boundary() {
    assert!(
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
        "boundary test requires a parent Wayland session"
    );
    assert!(compositor_binary().exists(), "build nested-wayland-session first");
    assert!(app_binary().exists(), "build this package (release) first");

    let run_dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "fm-gtk-sticky-boundary-{}-{}",
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    let fixture = run_dir.join("fixture");
    create_fixture_tree(&fixture);
    let socket = run_dir.join("control.sock");
    let state = run_dir.join("state.json");
    let screenshot = run_dir.join("sticky-rails.png");

    let spawned = Command::new(compositor_binary())
        .arg("--socket")
        .arg(&socket)
        .arg("--size")
        .arg("800x600")
        .arg("--")
        .arg(app_binary())
        .env_remove("DISPLAY")
        .env("GDK_BACKEND", "wayland")
        .env("FM_STICKY_START_DIR", &fixture)
        .env("FM_STICKY_STATE_PATH", &state)
        .env("FM_STICKY_DEBUG_TINT", "1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn nested compositor hosting the app");
    let mut child = KillOnDrop(spawned);

    let mut control = ControlSocket::connect(&socket);
    control.expect_ok("ping");

    let root = fixture.display().to_string();
    let alpha = fixture.join("alpha").display().to_string();
    let beta = fixture.join("beta").display().to_string();

    // Boot: one root pane listing the fixture, nothing scrolled, nothing overlapping.
    wait_for_state(&state, &[
        ("ready", Value::Bool(true)),
        ("paneCount", Value::from(1)),
        ("columnCount", Value::from(1)),
        ("overlapCount", Value::from(0)),
        ("activePath", Value::from(root.clone())),
    ]);

    // Enter descends into alpha (first row of the sorted listing).
    control.expect_ok("key enter");
    wait_for_state(&state, &[
        ("paneCount", Value::from(2)),
        ("columnCount", Value::from(2)),
        ("activePath", Value::from(alpha.clone())),
        ("overlapCount", Value::from(0)),
    ]);

    // Left returns focus to the root pane.
    control.expect_ok("key left");
    wait_for_state(&state, &[
        ("activePath", Value::from(root.clone())),
        ("paneCount", Value::from(2)),
    ]);

    // Re-entering alpha dedups: the existing pane is focused, no third pane appears.
    control.expect_ok("key enter");
    wait_for_state(&state, &[
        ("activePath", Value::from(alpha)),
        ("paneCount", Value::from(2)),
    ]);

    // Back to the root, select beta one row down, and descend.
    control.expect_ok("key left");
    wait_for_state(&state, &[("activePath", Value::from(root))]);
    control.expect_ok("key down");
    control.expect_ok("key enter");

    // Beta lands on row 1; revealing it scrolls the app, and the root pane must be pinned to the
    // viewport top by its sticky band while nothing overlaps: the decisive sticky assertions.
    wait_for_state(&state, &[
        ("paneCount", Value::from(3)),
        ("activePath", Value::from(beta)),
        ("scrolledDown", Value::Bool(true)),
        ("rootPinned", Value::Bool(true)),
        ("overlapCount", Value::from(0)),
    ]);

    // Backspace closes the focused beta pane and clears focus.
    control.expect_ok("key backspace");
    wait_for_state(&state, &[
        ("paneCount", Value::from(2)),
        ("activePath", Value::from("")),
    ]);

    control.expect_ok(&format!("screenshot {}", screenshot.display()));
    let size = std::fs::metadata(&screenshot).expect("screenshot written").len();
    assert!(size > 0, "screenshot is empty");

    control.expect_ok("quit");
    let status = child.0.wait().expect("compositor exit status");
    assert!(status.success(), "nested session exited with {status:?}");
    std::mem::forget(child);
    let _ = std::fs::remove_dir_all(&run_dir);
}
