//! Unit tests for policy identity derivation.

use super::*;

// The shipped default carries the decided bucket-zoom constants.
#[test]
fn default_policy_has_decided_constants() {
    let policy = default_policy();
    assert_eq!(policy.short_scan_max_secs, SHORT_SCAN_MAX_SECS);
    assert_eq!(policy.probe_window_secs, PROBE_WINDOW_SECS);
    assert_eq!(policy.pass1_coverage_fraction, PASS1_COVERAGE_FRACTION);
    assert_eq!(policy.bones_even_coverage_fraction, BONES_EVEN_COVERAGE_FRACTION);
    assert_eq!(policy.bones_top_slots, BONES_TOP_SLOTS);
    assert_eq!(policy.buckets.lossless, LOSSLESS_PROBE);
    assert_eq!(policy.buckets.lossless_bones, LOSSLESS_BONES_PROBE);
    assert_eq!(policy.buckets.store, STORE_PROBE);
    assert_eq!(policy.buckets.youtube, YOUTUBE_PROBE);
    assert_eq!(policy.buckets.bare, BARE_PROBE);
    assert_eq!(policy.ceiling_dbtp, CEILING_DBTP);
    assert_eq!(policy.max_too_loud_db, MAX_TOO_LOUD_DB);
    assert_eq!(policy.max_too_quiet_db, MAX_TOO_QUIET_DB);
}

// The policy id is stable for an unchanged policy.
#[test]
fn policy_id_is_stable() {
    assert_eq!(default_policy().policy_id(), default_policy().policy_id());
}

// Changing ANY parameter changes the policy id, so a stale cache row cannot be reused.
#[test]
fn policy_id_changes_with_every_parameter() {
    let base = default_policy();
    let base_id = base.policy_id();

    let mut short_scan = base;
    short_scan.short_scan_max_secs += 1.0;
    assert_ne!(short_scan.policy_id(), base_id);

    let mut window = base;
    window.probe_window_secs += 0.05;
    assert_ne!(window.policy_id(), base_id);

    let mut pass1 = base;
    pass1.pass1_coverage_fraction += 0.01;
    assert_ne!(pass1.policy_id(), base_id);

    let mut bones_even = base;
    bones_even.bones_even_coverage_fraction += 0.01;
    assert_ne!(bones_even.policy_id(), base_id);

    let mut bones_top = base;
    bones_top.bones_top_slots += 1;
    assert_ne!(bones_top.policy_id(), base_id);

    let mut lossless_coverage = base;
    lossless_coverage.buckets.lossless.coverage_fraction += 0.01;
    assert_ne!(lossless_coverage.policy_id(), base_id);

    let mut lossless_margin = base;
    lossless_margin.buckets.lossless.probe_margin_db += 0.01;
    assert_ne!(lossless_margin.policy_id(), base_id);

    let mut lossless_bones_coverage = base;
    lossless_bones_coverage.buckets.lossless_bones.coverage_fraction += 0.01;
    assert_ne!(lossless_bones_coverage.policy_id(), base_id);

    let mut lossless_bones_margin = base;
    lossless_bones_margin.buckets.lossless_bones.probe_margin_db += 0.01;
    assert_ne!(lossless_bones_margin.policy_id(), base_id);

    let mut store_coverage = base;
    store_coverage.buckets.store.coverage_fraction += 0.01;
    assert_ne!(store_coverage.policy_id(), base_id);

    let mut store_margin = base;
    store_margin.buckets.store.probe_margin_db += 0.01;
    assert_ne!(store_margin.policy_id(), base_id);

    let mut youtube_coverage = base;
    youtube_coverage.buckets.youtube.coverage_fraction += 0.01;
    assert_ne!(youtube_coverage.policy_id(), base_id);

    let mut youtube_margin = base;
    youtube_margin.buckets.youtube.probe_margin_db += 0.01;
    assert_ne!(youtube_margin.policy_id(), base_id);

    let mut bare_coverage = base;
    bare_coverage.buckets.bare.coverage_fraction += 0.01;
    assert_ne!(bare_coverage.policy_id(), base_id);

    let mut bare_margin = base;
    bare_margin.buckets.bare.probe_margin_db += 0.01;
    assert_ne!(bare_margin.policy_id(), base_id);

    let mut ceiling = base;
    ceiling.ceiling_dbtp = -1.5;
    assert_ne!(ceiling.policy_id(), base_id);

    let mut loud = base;
    loud.max_too_loud_db = 0.4;
    assert_ne!(loud.policy_id(), base_id);

    let mut quiet = base;
    quiet.max_too_quiet_db = -1.5;
    assert_ne!(quiet.policy_id(), base_id);
}

// The meter id is stable and non-trivial.
#[test]
fn meter_id_is_stable_and_nonzero() {
    assert_eq!(meter_id(), meter_id());
    assert_ne!(meter_id(), 0);
}

// The cache identity bundles all four parts; the decoder id varies independently of
// the policy id, which is the point of keeping them separate columns.
#[test]
fn cache_identity_keeps_decoder_independent() {
    let policy = default_policy();
    let one = policy.cache_identity(111);
    let two = policy.cache_identity(222);

    assert_eq!(one.policy_id, policy.policy_id());
    assert_eq!(one.meter_id, meter_id());
    assert_eq!(one.schema_version, SCHEMA_VERSION);
    assert_eq!(one.policy_id, two.policy_id); // same policy
    assert_ne!(one.decoder_stack_id, two.decoder_stack_id); // different decoder stack
    assert_ne!(one, two);
}
