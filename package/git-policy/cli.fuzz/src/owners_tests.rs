//! What:
//! Controls proving the owner token and stat generators reach every outcome.
//! Why:
//! An invariant that is never reached proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outcomesReachedBy(generatedCopyOwnerToken)).toEqual(['accepted', 'refused']);
//! ```

/// Import the generators and invariants under control.
use super::{
    check_copy_owner_token, check_generated_copy_owner_token, check_owner_file_names, check_stat,
    check_transaction_owner, generated_copy_owner_token, generated_stat,
};

/// Inputs of three bytes over a spread of values,
/// then longer derived ones.
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

/// Generated owner tokens are accepted and refused,
/// with and without a namespace.
#[test]
fn owner_tokens_reach_every_outcome() {
    let mut accepted: usize = 0;
    let mut refused: usize = 0;
    let mut with_namespace: usize = 0;
    for data in inputs() {
        check_generated_copy_owner_token(data.as_slice());
        check_copy_owner_token(String::from_utf8_lossy(data.as_slice()).as_ref());
        check_owner_file_names(String::from_utf8_lossy(data.as_slice()).as_ref());
        check_transaction_owner(data.as_slice());
        match generated_copy_owner_token(data.as_slice()).1 {
            Some(owner) => {
                accepted += 1;
                if owner.pid_namespace.is_some() {
                    with_namespace += 1;
                }
            }
            None => refused += 1,
        }
    }
    assert!(accepted > 100, "{accepted}");
    assert!(refused > 100, "{refused}");
    assert!(with_namespace > 10, "{with_namespace}");
}

/// Generated stat lines reach running,
/// exited and malformed outcomes.
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

/// Hard cases:
/// a command name with `) ` inside,
/// tokens and names that a reader could take for an owner,
/// and a transaction owner record.
#[test]
fn hard_cases_hold() {
    check_stat(b"");
    for token in [
        "",
        "~",
        "~~",
        "1~linux:5~",
        "1~linux:5~ns",
        "1~linux:5~~",
        "+1~linux:5~",
        "1~%6c~",
        "1~%6C~%",
        "1~%c3%a9~",
        "1~%c3~",
        "0~x~",
        "9223372036854775807~x~",
    ] {
        check_copy_owner_token(token);
    }
    for name in [
        "",
        ".json",
        "a.json",
        "a.1~x~.json",
        "a.1~x~.b.json",
        "..json",
        "a.stage.1~x~.u1.tmp",
        "a.1~x~.u1.tmp",
        ".1~x~.1.tmp",
        "dir/a.1~x~.json",
        "a.b.tmp",
        "tmp.json.tmp",
    ] {
        check_owner_file_names(name);
    }
    check_transaction_owner(b"{\"schemaVersion\":2,\"transactionId\":\"\",\"ownerPid\":1,\"ownerIdentity\":\"x\",\"createdAt\":\"\"}");
}
