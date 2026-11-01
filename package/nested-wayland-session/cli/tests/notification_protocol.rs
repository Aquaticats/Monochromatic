//! The notification,
//!  job,
//!  and accent control verbs parse exactly and fail whole on bad input.

/// The wire parser is the same interface the Unix control socket consumes.
use nested_wayland_session::{
    accent_color::AccentColor,
    protocol::{parse_command, Command, JobCommand, NotificationCommand},
};

/// Wraps a notification verb the way the parser returns it.
fn notification(command: NotificationCommand) -> Result<Command, String> {
    return Ok(Command::Notification(command));
}

/// Wraps a job verb the way the parser returns it.
fn job(command: JobCommand) -> Result<Command, String> {
    return Ok(Command::Job(command));
}

/// The no-argument notification verbs and the optional count parse like their launcher siblings.
#[test]
fn notification_listing_verbs_parse() {
    assert_eq!(parse_command("notifications"), notification(NotificationCommand::List));
    assert_eq!(parse_command("notification-clear\r\n"), notification(NotificationCommand::Clear));
    assert_eq!(parse_command("notification-log"), notification(NotificationCommand::Log { count: 64 }));
    assert_eq!(parse_command("notification-log 5"), notification(NotificationCommand::Log { count: 5 }));
    assert_eq!(parse_command("notification-log \t 5"), notification(NotificationCommand::Log { count: 5 }));
}

/// Bad counts and stray arguments name the verb and fail whole.
#[test]
fn notification_listing_verbs_reject_bad_arguments() {
    assert_eq!(parse_command("notification-log 0"), Err("notification-log count must be at least 1".to_string()));
    assert_eq!(
        parse_command("notification-log many"),
        Err("notification-log count is not a whole number".to_string()),
    );
    assert_eq!(
        parse_command("notification-log 1 2"),
        Err("notification-log takes at most one count".to_string()),
    );
    assert_eq!(parse_command("notifications now"), Err("notifications takes no arguments".to_string()));
    assert_eq!(parse_command("notification-clear all"), Err("notification-clear takes no arguments".to_string()));
}

/// The close reason is a code from 1 to 4 or its name,
///  and defaults to the user dismissing it.
#[test]
fn notification_close_parses_ids_and_reasons() {
    assert_eq!(
        parse_command("notification-close 7"),
        notification(NotificationCommand::Close { id: 7, reason: 2 }),
    );
    for (text, reason) in [("1", 1), ("2", 2), ("3", 3), ("4", 4), ("expired", 1), ("dismissed", 2), ("closed", 3), ("undefined", 4)] {
        assert_eq!(
            parse_command(&format!("notification-close 7 {text}")),
            notification(NotificationCommand::Close { id: 7, reason }),
            "{text}",
        );
    }
}

/// Every malformed close request names the verb and is not read as another command.
#[test]
fn notification_close_rejects_malformed_requests() {
    for command in [
        "notification-close",
        "notification-close 0",
        "notification-close -1",
        "notification-close x",
        "notification-close 4294967296",
        "notification-close 7 5",
        "notification-close 7 Expired",
        "notification-close 7 expired now",
        "notification-close 7; quit",
    ] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("notification-close"), "{command:?}: {message}");
    }
}

/// The action key is the rest of the line,
///  verbatim,
///  so keys with spaces and metacharacters survive.
#[test]
fn notification_action_keeps_the_key_verbatim() {
    assert_eq!(
        parse_command("notification-action 3 default"),
        notification(NotificationCommand::Action { id: 3, key: "default".to_string() }),
    );
    assert_eq!(
        parse_command("notification-action 3   Action 1; quit  \r\n"),
        notification(NotificationCommand::Action { id: 3, key: "Action 1; quit  ".to_string() }),
    );
    for command in ["notification-action", "notification-action ", "notification-action 3", "notification-action 3 ", "notification-action x key", "notification-action 0 key"] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("notification-action"), "{command:?}: {message}");
    }
}

/// The job verbs parse,
///  and the four per-job verbs take exactly one id.
#[test]
fn job_verbs_parse() {
    assert_eq!(parse_command("jobs"), job(JobCommand::List));
    assert_eq!(parse_command("job-clear"), job(JobCommand::Clear));
    assert_eq!(parse_command("job-log"), job(JobCommand::Log { count: 64 }));
    assert_eq!(parse_command("job-log 3"), job(JobCommand::Log { count: 3 }));
    assert_eq!(parse_command("job-suspend 2"), job(JobCommand::Suspend { id: 2 }));
    assert_eq!(parse_command("job-resume 2"), job(JobCommand::Resume { id: 2 }));
    assert_eq!(parse_command("job-cancel 2"), job(JobCommand::Cancel { id: 2 }));
    assert_eq!(parse_command("job-refresh 2"), job(JobCommand::Refresh { id: 2 }));
}

/// Malformed job requests name the verb and are not read as another command.
#[test]
fn job_verbs_reject_malformed_requests() {
    for command in [
        "jobs 1",
        "job-clear 1",
        "job-log 0",
        "job-log x",
        "job-log 1 2",
        "job-suspend",
        "job-suspend 0",
        "job-suspend x",
        "job-resume 1 2",
        "job-cancel 1; quit",
        "job-cancel -1",
        "job-refresh",
    ] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("job"), "{command:?}: {message}");
        assert!(!message.starts_with("unknown command"), "{command:?}: {message}");
    }
}

/// Lookalike spellings stay unknown commands and reach no service.
#[test]
fn lookalike_verbs_are_unknown_commands() {
    for command in ["notification", "notifications=1", "notification_log", "job", "job-pause 1", "Jobs", "--notifications"] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("unknown command"), "{command:?}: {message}");
    }
}

/// The accent verb takes exactly one six-digit color and nothing else.
#[test]
fn accent_color_verb_parses_one_color() {
    let expected = AccentColor::parse("3584e4").expect("3584e4 is a valid accent color");
    assert_eq!(parse_command("accent-color 3584E4"), Ok(Command::AccentColor(expected)));
    assert_eq!(parse_command("accent-color \t 3584e4\r\n"), Ok(Command::AccentColor(expected)));
    for command in [
        "accent-color",
        "accent-color ",
        "accent-color #3584E4",
        "accent-color 3584E",
        "accent-color 3584E4FF",
        "accent-color red",
        "accent-color 3584E4 quit",
        "accent-color 3584E4; quit",
        "accent-color 3584E4\nquit",
        "accent-color 3584E4\0",
    ] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("accent-color"), "{command:?}: {message}");
        assert!(!message.starts_with("unknown command"), "{command:?}: {message}");
    }
    for command in ["accent-color=3584E4", "accentcolor 3584E4", "accent_color 3584E4", "--accent-color 3584E4"] {
        let message = parse_command(command).expect_err(command);
        assert!(message.starts_with("unknown command"), "{command:?}: {message}");
    }
}
