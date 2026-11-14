//! What:
//!  Raw,
//!  generated and separated argument vectors through the invariants of wrapper
//!       control removal and of the refusal frontier.
//! Why:
//!  Raw bytes cover arbitrary argument content;
//!  generated tokens place controls,
//!      hatches,
//!  values and separators in every order;
//!  the separated view pins what
//!      survives after `--`;
//!  the frontier view drives the whole wrapped-command lifecycle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! fuzz(bytes, data => { checkControlRemoval(argumentsFromBytes(data)); checkFrontier(controlArguments(data.slice(MODE_BYTES)), modeOf(data)); });
//! ```
#![no_main]

/// Import the libFuzzer entry macro and the sidecar's shared generators and checks.
use git_policy_cli_fuzz::arguments::arguments_from_bytes;
use git_policy_cli_fuzz::controls::{
    MODE_BYTES, Mode, check_control_removal, check_frontier, check_separator, control_arguments,
    separated_arguments,
};
use libfuzzer_sys::fuzz_target;
use std::ffi::OsString;

// Run every view of every input. The first `MODE_BYTES` bytes also select the location and the
// repository answers of the frontier view, little endian.
// The `|data: &[u8]|` form is the macro's input grammar, not a closure.
fuzz_target!(|data: &[u8]| {
    let raw: Vec<OsString> = arguments_from_bytes(data);
    check_control_removal(raw.as_slice());
    let generated: Vec<OsString> = control_arguments(data);
    check_control_removal(generated.as_slice());
    if let Some((separated, separator)) = separated_arguments(data) {
        check_control_removal(separated.as_slice());
        check_separator(separated.as_slice(), separator);
    }
    // Each answer is one bit of the mode, and `controls` names every one of them.
    if let Some((heads, arguments)) = data.split_at_checked(MODE_BYTES)
        && let Ok(answers) = <[u8; MODE_BYTES]>::try_from(heads)
    {
        let mode: Mode = Mode::from_le_bytes(answers);
        let _ = check_frontier(control_arguments(arguments).as_slice(), mode);
        let _ = check_frontier(arguments_from_bytes(arguments).as_slice(), mode);
    }
});
