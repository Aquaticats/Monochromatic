//! Controls for the Rust copy of the repository link check against the one table of fixture states.
//! The TypeScript suite `package/repository-link/check/src/check-link.unit.test.ts` runs the same file
//! (`repository_link_shared_table.json`, inside this crate because the container gate copies only the crate);
//! edit a row there and both suites follow.

use crate::repository_link_answer::LinkAnswer;
use crate::repository_link_check_walk::check_repository_link;
use crate::repository_link_fixture_probe::{FixtureProbe, World};
use crate::repository_link_identity::{LinkIdentity, identity_from_text};
use serde_json::Value;

/// The table,
/// read at compile time so a path that stops resolving stops the build.
const TABLE: &str = include_str!("repository_link_shared_table.json");

/// The identity of a row:
/// the table's base identity with the row's changes.
fn identity_of(table: &Value, row: &Value) -> LinkIdentity {
    let mut merged: Value = table["baseIdentity"].clone();
    if let (Some(base), Some(changes)) = (merged.as_object_mut(), row["identity"].as_object()) {
        for (name, value) in changes {
            base.insert(name.clone(), value.clone());
        }
    }
    return identity_from_text(merged.to_string().as_str()).expect("the merged identity is JSON");
}

/// The answer of the Rust check to one row.
fn answer_of(table: &Value, row: &Value) -> LinkAnswer {
    if row["found"] == Value::Bool(false) {
        // No identity file was read: nothing is asked of the world.
        return crate::repository_link_check_walk::answer_when_identity_is_missing();
    }
    let probe: FixtureProbe = FixtureProbe::over(World::from_json(&row["world"]));
    return check_repository_link(&identity_of(table, row), &probe);
}

/// Every row of the shared table gives the kind,
/// readiness and message the table says,
/// and the table has as many rows as it declares.
#[test]
fn every_row_of_the_shared_table_gets_its_answer() {
    let table: Value = serde_json::from_str(TABLE).expect("the shared table is JSON");
    let rows: &Vec<Value> = table["rows"].as_array().expect("rows");
    let declared: u64 = table["rowCount"].as_u64().expect("rowCount");
    assert_eq!(rows.len() as u64, declared, "the table declares its own row count");
    assert!(!rows.is_empty(), "an empty table proves nothing");
    for row in rows {
        let name: &str = row["name"].as_str().expect("row name");
        let answer: LinkAnswer = answer_of(&table, row);
        assert_eq!(answer.kind.name(), row["expect"]["kind"].as_str().expect("kind"), "{name}: {}", answer.message);
        assert_eq!(answer.ready, row["expect"]["ready"].as_bool().expect("ready"), "{name}");
        if let Some(message) = row["expect"]["message"].as_str() {
            assert_eq!(answer.message, message, "{name}");
        }
        for part in row["expect"]["contains"].as_array().into_iter().flatten() {
            assert!(answer.message.contains(part.as_str().expect("part")), "{name}: {} lacks {part}", answer.message);
        }
    }
}
