//! Controls for the automatic-maintenance decision: config record parsing and the enable and detach rules.

use super::*;

/// Build the decision's input from `(key, value)` pairs, as `parse_config_records` returns them.
fn config(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    return pairs
        .iter()
        .map(|(key, value)| return (String::from(*key), String::from(*value)))
        .collect();
}

/// Arguments native `git commit` passes when maintenance detaches.
const DETACHED: &[&str] = &["maintenance", "run", "--auto", "--quiet", "--detach"];

/// Arguments native `git commit` passes when maintenance stays in the foreground.
const FOREGROUND: &[&str] = &["maintenance", "run", "--auto", "--quiet", "--no-detach"];

/// Config records keep the last value of each key and read a value-less key as `true`.
#[test]
fn config_records_keep_the_last_value_and_read_a_bare_key_as_true() {
    let parsed: BTreeMap<String, String> = parse_config_records(
        b"gc.auto\n1024\0maintenance.autodetach\ntrue\0maintenance.autodetach\nfalse\0gc.autodetach\0",
    )
    .expect("ASCII records");
    assert_eq!(parsed, config(&[
        ("gc.auto", "1024"),
        ("maintenance.autodetach", "false"),
        ("gc.autodetach", "true"),
    ]));
}

/// Empty config output parses to no keys, and empty records between NULs are skipped.
#[test]
fn empty_config_output_parses_to_no_keys() {
    assert_eq!(parse_config_records(b"").expect("empty"), BTreeMap::new());
    assert_eq!(parse_config_records(b"\0\0").expect("only separators"), BTreeMap::new());
}

/// Config output that is not UTF-8 is refused, as the incumbent's strict decoder throws.
#[test]
fn config_records_that_are_not_utf8_are_refused() {
    assert_eq!(
        parse_config_records(b"gc.auto\n\xff\0"),
        Err(MaintenanceError::ConfigNotUtf8)
    );
}

/// An unset configuration runs detached maintenance.
#[test]
fn unset_configuration_detaches() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[])),
        MaintenanceDecision::Run(DETACHED.to_vec())
    );
}

/// `maintenance.auto` false and zero each disable maintenance.
#[test]
fn explicit_maintenance_auto_false_or_zero_disables() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.auto", "false")])),
        MaintenanceDecision::Disabled
    );
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.auto", "0")])),
        MaintenanceDecision::Disabled
    );
}

/// `maintenance.auto` true enables maintenance even when `gc.auto` is zero.
#[test]
fn explicit_maintenance_auto_overrides_gc_auto() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.auto", "true"), ("gc.auto", "0")])),
        MaintenanceDecision::Run(DETACHED.to_vec())
    );
}

/// Without `maintenance.auto`, a zero or negative `gc.auto` disables maintenance.
#[test]
fn non_positive_gc_auto_disables() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("gc.auto", "0")])),
        MaintenanceDecision::Disabled
    );
    assert_eq!(
        auto_maintenance_arguments(&config(&[("gc.auto", "-1")])),
        MaintenanceDecision::Disabled
    );
}

/// A positive `gc.auto` enables maintenance.
#[test]
fn positive_gc_auto_enables() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("gc.auto", "6700")])),
        MaintenanceDecision::Run(DETACHED.to_vec())
    );
}

/// A `gc.auto` value that is not a number is never positive, as `Number("true") > 0` is false.
#[test]
fn boolean_gc_auto_disables() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("gc.auto", "true")])),
        MaintenanceDecision::Disabled
    );
    assert_eq!(
        auto_maintenance_arguments(&config(&[("gc.auto", "false")])),
        MaintenanceDecision::Disabled
    );
}

/// A `maintenance.auto` value that is neither a boolean nor a number counts as enabled, as `NaN !== 0` is true.
#[test]
fn unparsable_maintenance_auto_enables() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.auto", "maybe")])),
        MaintenanceDecision::Run(DETACHED.to_vec())
    );
}

/// `maintenance.autodetach` false selects the foreground form.
#[test]
fn maintenance_autodetach_false_runs_in_the_foreground() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.autodetach", "false")])),
        MaintenanceDecision::Run(FOREGROUND.to_vec())
    );
}

/// `maintenance.autodetach` takes precedence over `gc.autodetach`.
#[test]
fn maintenance_autodetach_precedes_gc_autodetach() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[
            ("maintenance.autodetach", "true"),
            ("gc.autodetach", "false"),
        ])),
        MaintenanceDecision::Run(DETACHED.to_vec())
    );
}

/// `gc.autodetach` false selects the foreground form when `maintenance.autodetach` is unset.
#[test]
fn gc_autodetach_false_runs_in_the_foreground() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("gc.autodetach", "false")])),
        MaintenanceDecision::Run(FOREGROUND.to_vec())
    );
}

/// Non-zero integers count as true for the detach settings, as `boolOrIntTrue` does.
#[test]
fn integer_detach_settings_are_true_when_nonzero() {
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.autodetach", "2")])),
        MaintenanceDecision::Run(DETACHED.to_vec())
    );
    assert_eq!(
        auto_maintenance_arguments(&config(&[("maintenance.autodetach", "0")])),
        MaintenanceDecision::Run(FOREGROUND.to_vec())
    );
}

/// Each failure prints its own diagnostic, so a failed maintenance decision names its cause.
#[test]
fn the_maintenance_errors_display_their_causes() {
    assert_eq!(
        MaintenanceError::ConfigNotUtf8.to_string(),
        "git config output for maintenance is not UTF-8"
    );
    assert_eq!(
        MaintenanceError::Start(String::from("git could not start")).to_string(),
        "git could not start"
    );
}
