//! What:
//!  Controls of the readiness record:
//!  its text,
//!  the admission decision,
//!  and the no-waiter refusal a key-load event before the checkout sees.
//! Why:
//!  The record admits an attach,
//!  so the accept and each refusal are held here,
//!  and the sweep's state is shown to gate nothing
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(admitAttach(record, { boot, pool, checkout }).kind).toBe('checkout');
//! ```

use super::{
    MAXIMUM_READINESS_BYTES,
    READINESS_DIRECTORY,
    READINESS_KEYS,
    READINESS_KIND,
    READINESS_NAME,
    ReadinessRecord,
    ReadyState,
    SweepState,
};
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::ReadinessExpectation;
use crate::worktree_clone_mount_protocol::clone_mount_readiness_admission::{
    AdmissionRefusal,
    admit_attach,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_parse::{
    ReadinessParseRefusal,
    parse_readiness,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The boot every control belongs to.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The pool every control verifies.
const POOL: &str = "zpcachyos";

/// The checkout every control walks from.
const CHECKOUT: &str = "/home/user/Mono";

/// The time every control was published at.
const TIME: &str = "2026-11-21T15:00:00Z";

/// A ready record for this boot.
fn ready() -> ReadinessRecord {
    return ReadinessRecord {
        boot: String::from(BOOT),
        pool_name: String::from(POOL),
        pool_state: ReadyState::Ready,
        checkout_path: String::from(CHECKOUT),
        checkout_state: ReadyState::Ready,
        sweep_nonce: RequestNonce::parse("0".repeat(NONCE_HEX_DIGITS).as_str()).ok(),
        sweep_state: SweepState::Finished,
        time: String::from(TIME),
    };
}

/// The expectation a caller holds.
fn expectation() -> ReadinessExpectation<'static> {
    return ReadinessExpectation {
        boot: BOOT,
        pool: POOL,
        checkout: CHECKOUT,
    };
}

/// The designed path and modes are the plan's.
#[test]
fn the_record_lives_at_the_designed_path() {
    assert_eq!(
        READINESS_DIRECTORY,
        "/run/git-worktree-clone-mount-readiness"
    );
    assert_eq!(READINESS_NAME, "result");
    assert_eq!(READINESS_KIND, "clone-mount-readiness");
    assert_eq!(READINESS_KEYS.len(), 7);
    assert_eq!(super::READINESS_DIRECTORY_MODE, 0o755);
    assert_eq!(super::READINESS_MODE, 0o644);
}

/// A ready record's text parses back,
/// and admits this boot's pool and checkout.
#[test]
fn a_ready_record_round_trips_and_admits() {
    let record: ReadinessRecord = ready();
    let text: String = record.text();
    let parsed: ReadinessRecord =
        parse_readiness(text.as_str()).expect("the route's own record parses");
    assert_eq!(parsed, record);
    assert_eq!(admit_attach(&parsed, &expectation()), Ok(()));
}

/// With no waiter and no timer,
/// a key-load event before the checkout is mounted is refused,
/// naming the checkout and doing nothing.
#[test]
fn the_no_waiter_readiness_refusal_names_the_checkout() {
    let mut record: ReadinessRecord = ready();
    record.checkout_state = ReadyState::NotReady;
    let refusal: AdmissionRefusal =
        admit_attach(&record, &expectation()).expect_err("an unready checkout admits nothing");
    match &refusal {
        AdmissionRefusal::CheckoutNotReady {
            checkout,
            state,
        } => {
            assert_eq!(checkout, CHECKOUT);
            assert_eq!(*state, ReadyState::NotReady);
        }
        other => panic!("the refusal names the checkout: {other:?}"),
    }
    let message: String = refusal.message();
    assert!(message.contains(CHECKOUT), "{message}");
    assert!(message.contains("No clone was attached"), "{message}");
    assert!(
        message.contains("`git monochromatic-git-policy-cli doctor`"),
        "the doctor is named as a person types it: {message}"
    );
    assert!(
        message.contains("installs no handler unit"),
        "and the sentence says this release installs the unit nowhere: {message}"
    );
    assert!(
        !message.contains("once plan step 10"),
        "and claims no `doctor` line a later step would add: {message}"
    );
}

/// A record from another boot,
/// another pool or another checkout is stale and admits nothing,
/// and an unknown state admits nothing either.
#[test]
fn a_stale_or_unknown_record_admits_nothing() {
    let mut record: ReadinessRecord = ready();
    record.boot = String::from("11111111-2222-4333-8444-555555555555");
    assert!(matches!(
        admit_attach(&record, &expectation()),
        Err(AdmissionRefusal::OtherBoot { .. })
    ));
    let mut another_pool: ReadinessRecord = ready();
    another_pool.pool_name = String::from("otherpool");
    assert!(matches!(
        admit_attach(&another_pool, &expectation()),
        Err(AdmissionRefusal::OtherPool { .. })
    ));
    let mut another_checkout: ReadinessRecord = ready();
    another_checkout.checkout_path = String::from("/home/user/other");
    assert!(matches!(
        admit_attach(&another_checkout, &expectation()),
        Err(AdmissionRefusal::OtherCheckout { .. })
    ));
    let mut unknown: ReadinessRecord = ready();
    unknown.pool_state = ReadyState::Unknown;
    assert!(matches!(
        admit_attach(&unknown, &expectation()),
        Err(AdmissionRefusal::PoolNotReady { .. })
    ));
    let mut unready_pool: ReadinessRecord = ready();
    unready_pool.pool_state = ReadyState::NotReady;
    assert!(matches!(
        admit_attach(&unready_pool, &expectation()),
        Err(AdmissionRefusal::PoolNotReady { .. })
    ));
}

/// The sweep's state is recorded and gates nothing:
/// a running,
/// failed or never-run sweep leaves a ready pool and checkout admissible.
#[test]
fn the_sweep_state_gates_nothing() {
    for state in [
        SweepState::Running,
        SweepState::Failed,
        SweepState::Never,
        SweepState::Finished,
    ] {
        let mut record: ReadinessRecord = ready();
        record.sweep_state = state;
        assert_eq!(admit_attach(&record, &expectation()), Ok(()), "{state:?}");
        let parsed: ReadinessRecord =
            parse_readiness(record.text().as_str()).expect("each sweep state parses");
        assert_eq!(parsed.sweep_state, state);
    }
    let mut without_nonce: ReadinessRecord = ready();
    without_nonce.sweep_nonce = None;
    let parsed: ReadinessRecord = parse_readiness(without_nonce.text().as_str())
        .expect("a record without a sweep nonce parses");
    assert_eq!(parsed.sweep_nonce, None);
    assert_eq!(admit_attach(&parsed, &expectation()), Ok(()));
}

/// A file over the bound,
/// a wrong line count and a wrong key are each refused.
#[test]
fn a_foreign_shape_is_refused() {
    let padded: String = format!("{}{}", ready().text(), "x".repeat(MAXIMUM_READINESS_BYTES));
    assert_eq!(
        parse_readiness(padded.as_str()),
        Err(ReadinessParseRefusal::TooLarge {
            bytes: padded.len(),
            bound: MAXIMUM_READINESS_BYTES
        })
    );
    let missing: String = ready()
        .text()
        .lines()
        .take(READINESS_KEYS.len() - 1)
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(matches!(
        parse_readiness(missing.as_str()),
        Err(ReadinessParseRefusal::LineCount { .. })
    ));
    let swapped: String = ready().text().replace("pool ", "pools ");
    assert!(matches!(
        parse_readiness(swapped.as_str()),
        Err(ReadinessParseRefusal::Key {
            expected: "pool",
            ..
        })
    ));
}

/// A state word outside its vocabulary,
/// a malformed nonce and a foreign boot or time are refused.
#[test]
fn each_value_has_its_rule() {
    let state: String = ready().text().replace("checkout ready", "checkout mostly");
    assert!(matches!(
        parse_readiness(state.as_str()),
        Err(ReadinessParseRefusal::State {
            key: "checkout",
            ..
        })
    ));
    let sweep: String = ready().text().replace("sweep finished", "sweep maybe");
    assert!(matches!(
        parse_readiness(sweep.as_str()),
        Err(ReadinessParseRefusal::SweepState { .. })
    ));
    let nonce: String = ready()
        .text()
        .replace(&"0".repeat(NONCE_HEX_DIGITS), "nothex");
    assert!(matches!(
        parse_readiness(nonce.as_str()),
        Err(ReadinessParseRefusal::Nonce(_))
    ));
    let boot: String = ready().text().replace(BOOT, "another-boot");
    assert!(matches!(
        parse_readiness(boot.as_str()),
        Err(ReadinessParseRefusal::Boot { .. })
    ));
    let time: String = ready().text().replace(TIME, "yesterday");
    assert!(matches!(
        parse_readiness(time.as_str()),
        Err(ReadinessParseRefusal::Time { .. })
    ));
    let checkout: String = ready().text().replace(CHECKOUT, "relative/path");
    assert!(matches!(
        parse_readiness(checkout.as_str()),
        Err(ReadinessParseRefusal::Value {
            key: "checkout",
            ..
        })
    ));
    let pool: String = ready().text().replace(POOL, "two words");
    assert!(matches!(
        parse_readiness(pool.as_str()),
        Err(ReadinessParseRefusal::Value {
            key: "pool",
            ..
        })
    ));
}

/// Every refusal says nothing was attached on the record,
/// and the one for a file in the readiness directory names the typed `doctor`
/// command and the unit this release installs nowhere.
#[test]
fn every_parse_refusal_says_nothing_was_attached() {
    let time: String = ready().text().replace(TIME, "yesterday");
    let refusal: ReadinessParseRefusal =
        parse_readiness(time.as_str()).expect_err("a foreign time is refused");
    assert!(
        refusal.message().contains("Nothing was attached"),
        "{}",
        refusal.message()
    );
    assert!(
        refusal
            .message()
            .contains("`git monochromatic-git-policy-cli doctor`"),
        "{}",
        refusal.message()
    );
    assert!(
        refusal.message().contains("installs no handler unit"),
        "{}",
        refusal.message()
    );
}
