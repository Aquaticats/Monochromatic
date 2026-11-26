//! What:
//!  One grant in the kernel's model,
//!  the nine decided grants where the design puts them,
//!  and the delegation those grants make for the managed account.
//! Why:
//!  The delegation checks read the grants and the datasets that carry them,
//!  so the whole decided set is built once here,
//!  and a control takes out the one grant it judges to read the finding that
//!  names it.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const delegation = { account: 1000, groups: [1000], checkoutDataset, containerDataset, grants: healthyGrants() };
//! ```

use super::delegation_reply::{
    DelegationFacts,
    Grant,
};
use super::route_test_facts_names::{
    ACCOUNT,
    ACCOUNT_GROUP,
    CHECKOUT_DATASET,
    CONTAINER_DATASET,
};

/// One grant in the kernel's model.
pub(crate) fn grant(
    carried_by: &str,
    permission: &str,
    who: super::delegation_reply::Who,
    inheritance: super::delegation_reply::Inheritance,
) -> Grant {
    return Grant {
        carried_by: String::from(carried_by),
        permission: String::from(permission),
        who,
        inheritance,
    };
}

/// The nine decided grants,
/// each where the design puts it.
pub(crate) fn healthy_grants() -> Vec<Grant> {
    use super::delegation_reply::{
        Inheritance,
        Who,
    };
    let user: Who = Who::User(ACCOUNT);
    return vec![
        grant(
            CHECKOUT_DATASET,
            "snapshot",
            user.clone(),
            Inheritance::Local,
        ),
        grant(
            CHECKOUT_DATASET,
            "clone",
            user.clone(),
            Inheritance::Descendants,
        ),
        grant(
            CHECKOUT_DATASET,
            "mount",
            user.clone(),
            Inheritance::Descendants,
        ),
        grant(
            CHECKOUT_DATASET,
            "destroy",
            user.clone(),
            Inheritance::Descendants,
        ),
        grant(
            CONTAINER_DATASET,
            "create",
            user.clone(),
            Inheritance::Local,
        ),
        grant(CONTAINER_DATASET, "mount", user.clone(), Inheritance::Local),
        grant(
            CONTAINER_DATASET,
            "mount",
            user.clone(),
            Inheritance::Descendants,
        ),
        grant(
            CONTAINER_DATASET,
            "userprop",
            user.clone(),
            Inheritance::Descendants,
        ),
        grant(CONTAINER_DATASET, "destroy", user, Inheritance::Descendants),
    ];
}

/// The delegation facts of the healthy route.
pub(crate) fn healthy_delegation() -> DelegationFacts {
    return DelegationFacts {
        account: ACCOUNT,
        groups: vec![ACCOUNT_GROUP],
        checkout_dataset: String::from(CHECKOUT_DATASET),
        container_dataset: String::from(CONTAINER_DATASET),
        grants: healthy_grants(),
    };
}
