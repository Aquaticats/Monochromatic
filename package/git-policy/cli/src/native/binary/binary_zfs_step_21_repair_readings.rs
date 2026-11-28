//! What:
//!  The readings plan step 21's arms take out of one add:
//!  the success line the repair prints,
//!  the time it credits each tool with,
//!  the branch its own diagnostic log names,
//!  and the one report of readings,
//!  spreads and ratios.
//! Why:
//!  `render` writes one line for a worktree whose repairs worked,
//!  a tool's detail followed by ` in <seconds> s`,
//!  and a detail may hold a semicolon of its own,
//!  so the line is read in one place that every arm shares
//!  (`package_repair_report.rs`,
//!  `render`).
//!  `AGENTS.md` reports a timing only as a ratio to a control arm run in the
//!  same round,
//!  with its range,
//!  and the spread of unchanged input first says whether a difference is larger
//!  than the noise.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const millis = theTimeOf(added.said, 'pnpm relinked node_modules'); report('pnpm', controls, measured);
//! ```

#![cfg(target_os = "linux")]

/// A time the repair line credits one tool with,
/// in milliseconds.
struct TimedTool {
    /// The words the repair used for it.
    detail: String,
    /// The milliseconds it ran for.
    millis: u128,
}

/// What:
///  The success line a repair printed,
///  or nothing when every repair failed.
/// Why:
///  A failure is a line of its own,
///  so an arm that reads only the success line must say the repair failed
///  rather than read no time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = said.split('\n').find((text) => text.includes('package-manager state of'));
/// ```
pub(super) fn the_repair_line(said: &str) -> Option<&str> {
    return said
        .lines()
        .find(|line| return line.contains("package-manager state of"));
}

/// What:
///  Every tool the repair line credits,
///  with the milliseconds it names.
/// Why:
///  `render` writes one detail per tool,
///  each followed by ` in <seconds> s`,
///  so the pairs are read from one line instead of guessed from order
///  (`package_repair_report.rs`,
///  `render`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worked = line.split('work at its new location: ')[1].split('; ').map((chunk) => chunk.rsplit(' in ', 1));
/// ```
fn the_tools_credited(line: &str) -> Vec<TimedTool> {
    let mut credited: Vec<TimedTool> = Vec::new();
    let Some((_before, rest)) = line.split_once("work at its new location: ") else {
        return credited;
    };
    // A detail may hold a semicolon of its own, which the Cargo repair's does when a clean displaced a lock, so the
    // chunks are joined until one ends with a time.
    let mut held: String = String::new();
    for chunk in rest.split("; ") {
        if !held.is_empty() {
            held.push_str("; ");
        }
        held.push_str(chunk);
        if let Some((detail, millis)) = the_time_at_the_end(held.as_str()) {
            credited.push(TimedTool {
                detail: String::from(detail),
                millis,
            });
            held.clear();
        }
    }
    return credited;
}

/// What:
///  The detail one chunk of a repair line ends with a time in,
///  and that time in milliseconds.
/// Why:
///  `render` appends ` in <seconds> s` to each tool's detail and joins the details with a semicolon,
///  and a detail may hold a semicolon of its own,
///  so a chunk is a tool's when and only when it ends with that form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = (chunk) => { const match = / in ([0-9]+\.[0-9]) s\.?$/.exec(chunk); return match ? [chunk.slice(0, match.index), match[1]] : undefined; };
/// ```
fn the_time_at_the_end(text: &str) -> Option<(&str, u128)> {
    let without_space: &str = text.trim_end();
    let without_stop: &str = without_space.strip_suffix('.').unwrap_or(without_space);
    let (detail, time) = without_stop.rsplit_once(" in ")?;
    let number: &str = time.strip_suffix(" s")?;
    if number.is_empty()
        || !number
            .chars()
            .all(|character| return character.is_ascii_digit() || character == '.')
    {
        return None;
    }
    return Some((detail, seconds_to_millis(number)));
}

/// The milliseconds a `x.y` seconds reading names.
fn seconds_to_millis(text: &str) -> u128 {
    let (whole, tenths): (&str, &str) = match text.split_once('.') {
        Some((whole, tenths)) => (whole, tenths),
        None => (text, "0"),
    };
    let seconds: u128 = whole.trim().parse().unwrap_or(0);
    let fraction: u128 = tenths.trim().parse().unwrap_or(0);
    return seconds * 1_000 + fraction * 100;
}

/// What:
///  The milliseconds a tool's own line credits it with,
///  or a panic naming what the add said.
/// Why:
///  An arm whose tool never ran would otherwise report a silence as a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const millis = tools.find((tool) => tool.detail.startsWith('pnpm ')).millis;
/// ```
pub(super) fn the_time_of(
    said: &str,
    needle: &str,
) -> u128 {
    let Some(line) = the_repair_line(said) else {
        panic!("the add printed no repair line: {said}");
    };
    for tool in the_tools_credited(line) {
        if tool.detail.starts_with(needle) {
            return tool.millis;
        }
    }
    panic!("the repair line names no tool starting {needle:?}: {line}");
}

/// What:
///  Whether the add's own diagnostic log holds a line with all these words.
/// Why:
///  The repair says which branch it took only in that log,
///  and a branch is evidence while a duration is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = add.said.split('\n').some((line) => words.every((word) => line.includes(word)));
/// ```
pub(super) fn the_log_says(
    said: &str,
    words: &[&str],
) -> bool {
    return said
        .lines()
        .any(|line| return words.iter().all(|word| return line.contains(word)));
}

/// The ratio of one arm's time to its control's,
/// or not-a-number when the control reads zero.
fn ratio_of(
    arm: u128,
    control: u128,
) -> f64 {
    return if control == 0 {
        f64::NAN
    } else {
        arm as f64 / control as f64
    };
}

/// The middle of a list of readings,
/// which is the median of an odd count.
fn middle(values: &[u128]) -> u128 {
    let mut sorted: Vec<u128> = values.to_vec();
    sorted.sort_unstable();
    return sorted.get(sorted.len() / 2).copied().unwrap_or(0);
}

/// What:
///  Print an arm's readings,
///  the run-to-run spread of each,
///  and the ratio of each round to the control beside it.
/// Why:
///  `AGENTS.md` reports a timing only as a ratio to a control arm run
///  interleaved in the same round,
///  with its range,
///  and the spread of the arm on unchanged input says first whether a
///  difference is larger than the noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.log(`${arm} control-ms=[${controls}] arm-ms=[${measured}] ratio per-round=[${ratios}] median=${median} range=${min}..${max}`);
/// ```
pub(super) fn report(
    arm: &str,
    controls: &[u128],
    measured: &[u128],
) {
    let ratios: Vec<f64> = measured
        .iter()
        .zip(controls.iter())
        .map(|(one, control)| return ratio_of(*one, *control))
        .collect();
    let lowest: f64 = ratios.iter().copied().fold(f64::INFINITY, f64::min);
    let highest: f64 = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!(
        "STEP21 {arm} control-ms={controls:?} arm-ms={measured:?} control-spread={}..{} arm-spread={}..{}",
        controls.iter().copied().min().unwrap_or(0),
        controls.iter().copied().max().unwrap_or(0),
        measured.iter().copied().min().unwrap_or(0),
        measured.iter().copied().max().unwrap_or(0)
    );
    println!(
        "STEP21 {arm} ratio per-round={ratios:?} median={:.3} range={lowest:.3}..{highest:.3} control-median={} arm-median={}",
        ratio_of(middle(measured), middle(controls)),
        middle(controls),
        middle(measured)
    );
}
