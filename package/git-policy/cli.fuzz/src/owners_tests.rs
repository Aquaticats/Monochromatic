//! What: Controls proving the owner and stat generators reach every outcome.
//! Why: An invariant that is never reached proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outcomesReachedBy(generatedOwnerLock)).toEqual(['accepted', 'refused']);
//! ```

/// Import the generators and invariants under control.
use super::{
    check_generated_owner_lock, check_owner_lock_text, check_stat, check_transaction_owner,
    generated_owner_lock, generated_stat,
};

/// Inputs of three bytes over a spread of values, then longer derived ones.
pub(crate) fn inputs() -> Vec<Vec<u8>> {
    let mut all: Vec<Vec<u8>> = Vec::new();
    for first in 0..=255_u8 {
        for second in [0_u8, 1, 2, 3, 5, 8, 13, 64, 127, 128, 200, 255] {
            all.push(vec![first, second, first ^ second]);
            let mut long: Vec<u8> = Vec::new();
            for index in 0..48_u8 {
                long.push(
                    first
                        .wrapping_mul(31)
                        .wrapping_add(index.wrapping_mul(second)),
                );
            }
            all.push(long);
        }
    }
    return all;
}

/// Generated owner locks are accepted and refused, with and without a transaction.
#[test]
fn owner_locks_reach_every_outcome() {
    let mut accepted: usize = 0;
    let mut refused: usize = 0;
    let mut with_transaction: usize = 0;
    for data in inputs() {
        check_generated_owner_lock(data.as_slice());
        check_owner_lock_text(String::from_utf8_lossy(data.as_slice()).as_ref());
        check_transaction_owner(data.as_slice());
        match generated_owner_lock(data.as_slice()).1 {
            Some(record) => {
                accepted += 1;
                if record.transaction_id.is_some() {
                    with_transaction += 1;
                }
            }
            None => refused += 1,
        }
    }
    assert!(accepted > 100, "{accepted}");
    assert!(refused > 100, "{refused}");
    assert!(with_transaction > 10, "{with_transaction}");
}

/// Generated stat lines reach running, exited and malformed outcomes.
#[test]
fn stat_lines_reach_every_outcome() {
    let mut running: usize = 0;
    let mut exited: usize = 0;
    let mut malformed: usize = 0;
    for data in inputs() {
        check_stat(data.as_slice());
        match generated_stat(data.as_slice()).1 {
            Ok(Some(_)) => running += 1,
            Ok(None) => exited += 1,
            Err(()) => malformed += 1,
        }
    }
    assert!(
        running > 100 && exited > 10 && malformed > 100,
        "{running} {exited} {malformed}"
    );
}

/// Hard cases: a command name with `) ` inside, and a record with duplicate keys.
#[test]
fn hard_cases_hold() {
    check_stat(b"");
    check_owner_lock_text(
        "{\"schemaVersion\":1,\"token\":\"a\",\"token\":\"b\",\"ownerPid\":1,\"ownerBirthIdentity\":\"i\"}",
    );
    check_owner_lock_text(
        "{\"schemaVersion\":1,\"token\":\"t\",\"ownerPid\":1e3,\"ownerBirthIdentity\":\"i\"}",
    );
    check_transaction_owner(b"{\"schemaVersion\":2,\"transactionId\":\"\",\"ownerPid\":1,\"ownerIdentity\":\"x\",\"createdAt\":\"\"}");
}
