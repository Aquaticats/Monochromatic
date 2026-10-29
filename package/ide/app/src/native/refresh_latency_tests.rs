//! Measure how long an external write takes to reach the tree rows and the displayed source text.
//! Ignored by default:
//!  `inspect:refresh-latency` runs it with output,
//!  so the polling build and the
//! watching build can be compared on the same machine with the same fixture and trial gaps.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Tree-row lookup by label,
///  shared with the navigation tests.
use super::navigation_tests::row;
/// The production window,
///  source state,
///  and the bindings the shipped application installs.
use super::{
    AppWindow, State, bind_appearance, bind_keys, bind_pointer, bind_viewport, navigation, reload,
    render,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// The canonical project boundary is created only over disposable fixtures.
use ide_app::workspace::Workspace;
/// Real headless timers drive the same refresh timers as the shipped event loop.
use slint::{ComponentHandle, SharedString, platform::update_timers_and_animations};
/// What:
///  `Rc<RefCell<State>>` is the window's shared source state;
///  `Duration` and `Instant` time trials.
/// Why:
///  Each trial measures wall-clock time from the write to the observed model change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = performance.now();
/// ```
use std::{
    cell::RefCell,
    fs,
    rc::Rc,
    time::{Duration, Instant},
};

/// Trials per measured case;
///  enough to see the spread of a 250 ms or multi-second polling period.
const TRIALS: usize = 16;

/// The old tree polling read one directory every 500 ms,
///  round robin over the root and expanded folders.
const OLD_DIRECTORY_POLL_MS: u64 = 500;

/// Deterministic pseudo-random numbers,
///  identical in every build that runs this test.
///
/// What:
///  a 64-bit linear congruential generator;
///  `u64` (siblings `u32`,
///  `usize`) holds its state,
///       and `wrapping_mul`/`wrapping_add` overflow modulo 2^64 instead of panicking.
/// Why:
///  Write times and target folders must not line up with a polling phase:
///  when trial order followed
///      the round-robin order,
///  every write landed just before its folder's turn and polling looked fast.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let state = 1n; const next = () => (state = (state * A + C) % 2n ** 64n) >> 33n;
/// ```
struct Sequence(u64);

/// Next value in `0..bound`.
impl Sequence {
    /// High bits of a linear congruential generator are its best-distributed bits.
    fn below(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        return (self.0 >> 33) % bound;
    }
}

/// A trial slower than this is reported as a failure rather than waited on forever.
const TRIAL_LIMIT: Duration = Duration::from_secs(20);

/// Keep the native timers running for `span`,
///  like an idle event loop between external writes.
fn idle(span: Duration) {
    let start = Instant::now();
    while start.elapsed() < span {
        update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Return the time until the tree lists a row labeled `label`,
///  polling the model about once per millisecond.
fn elapsed_until_row(window: &AppWindow, label: &str) -> Duration {
    let mut waiting = Waiting::closely(TRIAL_LIMIT);
    while row(window, label).is_none() {
        waiting.step("an external write did not reach the window");
    }
    return waiting.elapsed();
}

/// Return the time until the window shows `content` as the source text,
///  polling the model about once per millisecond.
fn elapsed_until_source(window: &AppWindow, content: &str) -> Duration {
    let mut waiting = Waiting::closely(TRIAL_LIMIT);
    while window.get_source_text() != content {
        waiting.step("an external write did not reach the window");
    }
    return waiting.elapsed();
}

/// Print one JSON line:
///  sorted samples plus minimum,
///  median,
///  90th percentile,
///  and maximum in milliseconds.
fn report(case: &str, folders: usize, samples: &[Duration]) {
    // What: `Vec<f64>` is an owned growable list of 64-bit floats; `f32` would round sub-millisecond parts.
    // Why: Sorting a copy keeps the trial order of `samples` intact for nothing else to depend on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const sorted = samples.map(sample => sample * 1000).sort((a, b) => a - b);
    // ```
    let mut sorted: Vec<f64> = Vec::new();
    for sample in samples {
        sorted.push(sample.as_secs_f64() * 1000.0);
    }
    sorted.sort_by(f64::total_cmp);
    let count = sorted.len();
    let median = sorted[count / 2];
    let high = sorted[(count * 9) / 10];
    let mut rounded = Vec::new();
    for value in &sorted {
        rounded.push(format!("{value:.1}"));
    }
    println!(
        "{{\"case\":\"{case}\",\"expanded\":{folders},\"n\":{count},\"min_ms\":{:.1},\"median_ms\":{median:.1},\"p90_ms\":{high:.1},\"max_ms\":{:.1},\"samples_ms\":[{}]}}",
        sorted[0],
        sorted[count - 1],
        rounded.join(",")
    );
}

/// Expand `folders` directories,
///  then time file creation in them and rewrites of the displayed file.
fn measure(folders: usize) {
    let fixture = tempfile::tempdir().expect("disposable latency project");
    let displayed = fixture.path().join("view.txt");
    fs::write(&displayed, "revision start\n").expect("displayed source");
    for index in 0..folders {
        let folder = fixture.path().join(format!("folder-{index:02}"));
        fs::create_dir(&folder).expect("fixture folder");
        fs::write(folder.join(format!("seed-{index:02}.txt")), "").expect("folder seed");
    }
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native latency window");
    let text = fs::read_to_string(&displayed).expect("initial source");
    // `Some` marks the displayed file as an on-disk path that the source refresh rereads.
    let state = Rc::new(RefCell::new(State::new(&text, Some(displayed.clone()))));
    window.set_file_label(SharedString::from(displayed.display().to_string()));
    bind_pointer(&window, &state);
    bind_viewport(&window, &state);
    bind_keys(&window, &state);
    bind_appearance(&window, &state);
    let _source_refresh = reload::bind(&window, &state).expect("source refresh");
    let _navigation = navigation::bind(&window, &state, workspace).expect("project navigation");
    window.show().expect("show latency window");
    render(&window, &state);
    // What: Wait for the highlighting reply of the opened file before the test starts its steps.
    // Why: The reply renders again after the first frame, so a step timed or counted from here met it in the middle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await waitForHighlighting(state);
    // ```
    super::find_tests::wait_for_highlighting(&state);
    for index in 0..folders {
        let label = format!("folder-{index:02}");
        elapsed_until_row(&window, &label);
        window.invoke_tree_activate(row(&window, &label).expect("folder row"));
        let seed = format!("seed-{index:02}.txt");
        elapsed_until_row(&window, &seed);
    }
    idle(Duration::from_millis(600));
    let mut random = Sequence(folders as u64);
    // A gap drawn uniformly over one whole old polling cycle puts each write at a random polling phase.
    let cycle = (folders as u64 + 1) * OLD_DIRECTORY_POLL_MS;
    let mut tree = Vec::new();
    for trial in 0..TRIALS {
        idle(Duration::from_millis(20 + random.below(cycle)));
        let name = format!("new-{trial:02}.txt");
        let target = random.below(folders as u64);
        let folder = fixture.path().join(format!("folder-{target:02}"));
        fs::write(folder.join(&name), "").expect("external file creation");
        tree.push(elapsed_until_row(&window, &name));
    }
    let mut source = Vec::new();
    for trial in 0..TRIALS {
        // Two old 250 ms source polling periods per gap range.
        idle(Duration::from_millis(20 + random.below(500)));
        let content = format!("revision {trial}\n");
        fs::write(&displayed, &content).expect("external source rewrite");
        source.push(elapsed_until_source(&window, &content));
    }
    report("tree-create", folders, &tree);
    report("source-rewrite", folders, &source);
    window.hide().expect("close latency window");
}

/// One expanded folder:
///  the tree's work list is the root and that folder.
#[test]
#[ignore = "timing measurement; run through inspect:refresh-latency"]
fn refresh_latency_with_one_expanded_folder() {
    measure(1);
}

/// Eight expanded folders:
///  polling latency grows with the folder count,
///  watching should not.
#[test]
#[ignore = "timing measurement; run through inspect:refresh-latency"]
fn refresh_latency_with_eight_expanded_folders() {
    measure(8);
}
