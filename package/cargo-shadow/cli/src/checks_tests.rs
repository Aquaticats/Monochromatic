//! The unit tests of the release-age and trust checks over newly locked versions.

use std::collections::{BTreeMap, BTreeSet};

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::checks::check_trust;
use crate::http::RegistryError;
use crate::lockfile::LockedVersion;
use crate::policy::VersionRecord;
use crate::registry::Registry;

fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 0,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: true,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

/// A registry with one canned history and one canned owner set.
struct OneCrate {
    history: Vec<VersionRecord>,
    owners: BTreeSet<String>,
    owner_lookups: std::cell::Cell<u32>,
}

impl Registry for OneCrate {
    fn pubtimes(&self, _name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return Ok(BTreeMap::new());
    }
    fn yanked(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return Ok(BTreeSet::new());
    }
    fn history(&self, _name: &str, _versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        return Ok(self.history.clone());
    }
    fn owners(&self, _name: &str) -> Result<BTreeSet<String>, RegistryError> {
        self.owner_lookups.set(self.owner_lookups.get() + 1);
        return Ok(self.owners.clone());
    }
}

fn record(num: &str, created_at: &str, publisher_id: &str) -> VersionRecord {
    return VersionRecord {
        num: num.to_string(),
        created_at: created_at.to_string(),
        publisher_id: publisher_id.to_string(),
        trustpub_repository: String::new(),
        repository: String::new(),
    };
}

/// Review finding (lazy owner lookup): without the owner list both releases rank as first-time publishers, so the
/// ownerless evaluation finds nothing; with it the earlier release (account 1 is a listed owner) ranks higher than the
/// candidate (account 2 is not), which is a downgrade. The owners must be asked even when the ownerless pass is clean.
#[test]
fn an_owner_who_raises_the_earlier_release_makes_a_later_release_by_a_stranger_a_downgrade() {
    let registry: OneCrate = OneCrate {
        history: vec![record("1.0.0", "2024-01-01T00:00:00Z", "1"), record("1.0.1", "2025-01-01T00:00:00Z", "2")],
        owners: ["1".to_string()].into_iter().collect(),
        owner_lookups: std::cell::Cell::new(0),
    };
    let candidate: Vec<LockedVersion> = vec![LockedVersion { name: "dep".to_string(), version: "1.0.1".to_string() }];

    let messages = check_trust(&candidate, &settings(), &registry, 0).expect("checks");

    assert_eq!(messages.len(), 1, "{messages:?}");
}

/// The lookup stays lazy where the owner list cannot matter: a release by an account that already published an earlier
/// release ranks as a known publisher without owners, so no owner can change the verdict and the rate-limited call is skipped.
#[test]
fn the_owner_list_is_not_asked_when_it_cannot_change_the_verdict() {
    let registry: OneCrate = OneCrate {
        history: vec![record("1.0.0", "2024-01-01T00:00:00Z", "1"), record("1.0.1", "2025-01-01T00:00:00Z", "1")],
        owners: BTreeSet::new(),
        owner_lookups: std::cell::Cell::new(0),
    };
    let candidate: Vec<LockedVersion> = vec![LockedVersion { name: "dep".to_string(), version: "1.0.1".to_string() }];

    let messages = check_trust(&candidate, &settings(), &registry, 0).expect("checks");

    assert!(messages.is_empty(), "{messages:?}");
    assert_eq!(registry.owner_lookups.get(), 0);
}
