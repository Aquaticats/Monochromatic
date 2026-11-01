// What:     Unit tests for `console_color.rs`, pulled in by `#[cfg(test)] #[path = "console_color_tests.rs"] mod tests;`.
// Why:      Cover every branch of the color decisions, because the Win32 code that measures the consoles compiles on
//           Windows only.

use super::*;

// A console mode as a fresh console reports it: processed output and wrap at end of line, no VT processing.
const PLAIN_CONSOLE_MODE: u32 = 0x0003;
const REFUSED: ConsoleColorSupport = ConsoleColorSupport::NoVirtualTerminal { error_code: 87 };

#[test]
fn the_virtual_terminal_flag_is_added_and_the_other_flags_are_kept() {
    assert_eq!(mode_with_virtual_terminal(PLAIN_CONSOLE_MODE), 0x0007);
}

#[test]
fn a_mode_that_already_has_the_flag_is_unchanged() {
    assert_eq!(mode_with_virtual_terminal(0x0007), 0x0007);
}

#[test]
fn a_plain_console_mode_has_no_virtual_terminal() {
    assert!(!has_virtual_terminal(PLAIN_CONSOLE_MODE));
}

#[test]
fn a_console_mode_with_the_flag_has_virtual_terminal() {
    assert!(has_virtual_terminal(0x0007));
}

#[test]
fn the_log_is_colored_on_a_console_that_interprets_escape_sequences() {
    assert!(log_colors_wanted(ConsoleColorSupport::VirtualTerminal, None));
}

#[test]
fn the_log_is_plain_in_a_file_or_a_pipe() {
    assert!(!log_colors_wanted(ConsoleColorSupport::NotConsole, None));
}

#[test]
fn the_log_is_plain_on_a_console_that_refused_virtual_terminal_processing() {
    assert!(!log_colors_wanted(REFUSED, None));
}

#[test]
fn no_color_turns_the_log_colors_off() {
    assert!(!log_colors_wanted(ConsoleColorSupport::VirtualTerminal, Some("1")));
}

#[test]
fn an_empty_no_color_is_ignored() {
    assert!(log_colors_wanted(ConsoleColorSupport::VirtualTerminal, Some("")));
}

#[test]
fn clap_detects_colors_itself_when_every_console_interprets_escape_sequences() {
    let colors = ConsoleColors { stdout: ConsoleColorSupport::NotConsole, stderr: ConsoleColorSupport::VirtualTerminal };
    assert_eq!(cli_color(colors), clap::ColorChoice::Auto);
}

#[test]
fn clap_detects_colors_itself_when_nothing_is_a_console() {
    let colors = ConsoleColors { stdout: ConsoleColorSupport::NotConsole, stderr: ConsoleColorSupport::NotConsole };
    assert_eq!(cli_color(colors), clap::ColorChoice::Auto);
}

#[test]
fn clap_prints_no_colors_when_stderr_is_a_console_that_refused() {
    let colors = ConsoleColors { stdout: ConsoleColorSupport::NotConsole, stderr: REFUSED };
    assert_eq!(cli_color(colors), clap::ColorChoice::Never);
}

#[test]
fn clap_prints_no_colors_when_stdout_is_a_console_that_refused() {
    let colors = ConsoleColors { stdout: REFUSED, stderr: ConsoleColorSupport::VirtualTerminal };
    assert_eq!(cli_color(colors), clap::ColorChoice::Never);
}

#[test]
fn logging_the_color_support_does_not_panic_without_a_subscriber() {
    log_console_colors(&ConsoleColors { stdout: REFUSED, stderr: ConsoleColorSupport::VirtualTerminal });
}
