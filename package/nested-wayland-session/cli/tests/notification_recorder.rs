//! The compositor-free recorder:
//! a command's notifications land on the private bus and in the log file.

use std::process::Command;

/// A private bus carries the command's `Notify` call to the recording service,
///  the log file holds the record,
///  and the recorder returns the command's exit status.
#[test]
fn a_notification_sent_by_the_command_is_logged_and_the_exit_status_is_kept() {
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("nws-recorder-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("make the scratch folder");
    let log = directory.join("log.txt");
    let status = Command::new(env!("CARGO_BIN_EXE_monochromatic-nested-wayland-session-notification-recorder"))
        .args(["--log"])
        .arg(&log)
        .args([
            "--notifications",
            "freedesktop",
            "--",
            "sh",
            "-c",
            "dbus-send --session --print-reply --dest=org.freedesktop.Notifications /org/freedesktop/Notifications \
             org.freedesktop.Notifications.Notify string:toolchain-daily-update uint32:0 string:dialog-warning \
             string:'Daily update needs a look' string:'one package broke' array:string: dict:string:variant: int32:5000 \
             && exit 7",
        ])
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .status()
        .expect("run the recorder");
    assert_eq!(status.code(), Some(7), "the command's exit status is the recorder's");
    let text = std::fs::read_to_string(&log).expect("the log file exists");
    assert!(text.starts_with("1 "), "one record: {text}");
    assert!(text.contains("Daily%20update%20needs%20a%20look"), "{text}");
    assert!(text.contains("toolchain-daily-update"), "{text}");
    std::fs::remove_dir_all(&directory).expect("remove the scratch folder");
}

/// A missing `--log` or command is refused before any bus starts.
#[test]
fn a_missing_log_or_command_is_a_usage_error() {
    for arguments in [vec!["--", "true"], vec!["--log", "x"], vec!["--log", "x", "--"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_monochromatic-nested-wayland-session-notification-recorder"))
            .args(&arguments)
            .output()
            .expect("run the recorder");
        assert!(!output.status.success(), "{arguments:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"), "{arguments:?}");
    }
}
