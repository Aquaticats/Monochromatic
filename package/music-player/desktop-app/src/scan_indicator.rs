//! Scan indicator model,
//!  ported from `ScanIndicatorModel.kt`.
//!
//! The phase and the status text are decided here,
//!  so the view only places text.
//! Counts are `i32` to match the Kotlin `Int` they mirror.
//!  No Slint or I/O is used.

/// What:
///      `pub enum ScanIndicatorPhase` names the phases the scan indicator passes through.
/// Why:
///      A closed set lets the view and the text rule handle every phase explicitly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanIndicatorPhase = "idle" | "scanning_library" | "measuring_true_peak" | "done";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanIndicatorPhase {
    /// No library scan and no true-peak measurement is pending.
    Idle,
    /// The library is being read and no track count is known yet.
    ScanningLibrary,
    /// True peaks are being measured,
    ///  and the done and total counts are known.
    MeasuringTruePeak,
    /// Measurement finished,
    ///  so the indicator is removed.
    Done,
}

/// What:
///      `pub struct ScanIndicatorModel` is the phase plus the done and total counts.
/// Why:
///      An immutable record lets the view render from one value without reading the sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanIndicatorModel = Readonly<{ phase: ScanIndicatorPhase; done: number; total: number }>;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanIndicatorModel {
    /// Phase the indicator is currently in.
    pub phase: ScanIndicatorPhase,
    /// Tracks whose visit has finished,
    ///  inside the known total.
    pub done: i32,
    /// Number of tracks the sweep will visit,
    ///  or zero when not yet known.
    pub total: i32,
}

/// Methods of `ScanIndicatorModel`.
impl ScanIndicatorModel {
    /// What:
    ///      `visible` is true only while the library is being scanned or true peaks are being measured.
    /// Why:
    ///      Visibility follows the phase,
    ///  so the flag cannot disagree with it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get visible(): boolean { return phase === "scanning_library" || phase === "measuring_true_peak"; }
    /// ```
    pub fn visible(&self) -> bool {
        return matches!(
            self.phase,
            ScanIndicatorPhase::ScanningLibrary | ScanIndicatorPhase::MeasuringTruePeak
        );
    }
}

/// What:
///      `pub fn scan_indicator_for` chooses the indicator model from the library loading flag and the sweep counts.
/// Why:
///      A loading library takes precedence,
///  then a known total decides between measuring and done.
///      A negative total counts as unknown,
///  and a measured count is clamped into the known total.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanIndicatorFor(loading: boolean, measured: number, total: number): ScanIndicatorModel;
/// ```
pub fn scan_indicator_for(loading: bool, measured: i32, total: i32) -> ScanIndicatorModel {
    if loading {
        return ScanIndicatorModel {
            phase: ScanIndicatorPhase::ScanningLibrary,
            done: 0,
            total: 0,
        };
    }
    let known_total = total.max(0);
    if known_total == 0 {
        return ScanIndicatorModel {
            phase: ScanIndicatorPhase::Idle,
            done: 0,
            total: 0,
        };
    }
    let done = measured.clamp(0, known_total);
    if done == known_total {
        return ScanIndicatorModel {
            phase: ScanIndicatorPhase::Done,
            done,
            total: known_total,
        };
    }
    return ScanIndicatorModel {
        phase: ScanIndicatorPhase::MeasuringTruePeak,
        done,
        total: known_total,
    };
}

/// What:
///      `COUNT_GROUP_SIZE` is the number of digits between two grouping commas.
/// Why:
///      Thousands grouping is three digits,
///  and naming it keeps the grouping rule readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COUNT_GROUP_SIZE = 3;
/// ```
const COUNT_GROUP_SIZE: usize = 3;

/// What:
///      `pub fn scan_indicator_count_text` formats a count with a comma between each group of three digits.
/// Why:
///      ASCII digits and fixed commas keep the text identical on every device locale.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const countText = (value: number): string => value.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ",");
/// ```
pub fn scan_indicator_count_text(value: i32) -> String {
    let digits = value.unsigned_abs().to_string();
    let digit_count = digits.len();
    let mut grouped = String::with_capacity(digit_count + digit_count / COUNT_GROUP_SIZE + 1);
    if value < 0 {
        grouped.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digit_count - index) % COUNT_GROUP_SIZE == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    return grouped;
}

/// What:
///      `pub fn scan_indicator_status_text` returns the single status line the indicator draws.
/// Why:
///      The authored copy lives in one pure function,
///  so the view only places text.
///      A hidden phase yields an empty string,
///  so no stale line can be drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanIndicatorStatusText(model: ScanIndicatorModel): string;
/// ```
pub fn scan_indicator_status_text(model: &ScanIndicatorModel) -> String {
    if model.phase == ScanIndicatorPhase::ScanningLibrary {
        return "Scanning library".to_owned();
    }
    if model.phase == ScanIndicatorPhase::MeasuringTruePeak {
        let done_text = scan_indicator_count_text(model.done);
        let total_text = scan_indicator_count_text(model.total);
        return format!("Analyzing true peak · {done_text} of {total_text}");
    }
    return String::new();
}

/// Unit tests in `scan_indicator_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "scan_indicator_tests.rs"]
mod tests;
