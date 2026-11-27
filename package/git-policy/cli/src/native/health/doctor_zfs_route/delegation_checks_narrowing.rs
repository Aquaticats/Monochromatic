//! What:
//!  Every decided permission the pool reports more widely than the design
//!  scopes it,
//!  with the grant that makes it effective and the commands that narrow it.
//! Why:
//!  Narrowing one permission is two commands about one pair of datasets,
//!  and one wider grant can serve several permissions,
//!  so a person needs the decided grant in place before the wider one goes or
//!  the account is without the permission between the two commands,
//!  and a removal that one wider grant already served prints once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const narrowings = narrowingsOf(facts); // [{ required, wide }]
//! const line = scopeDetail(narrowings, facts);
//! ```

/// How one permission was met,
/// and the commands that grant or remove one.
use super::delegation_checks_grant_commands::{
    Satisfaction,
    allow_command,
    place_dataset,
    satisfaction,
    unallow_command,
};
/// The permissions the design requires,
/// and the pair of a required grant with a wider one.
use super::delegation_checks_permissions::{
    REQUIRED_GRANTS,
    RequiredGrant,
};
/// The grants the kernel's reply became.
use super::delegation_reply::{
    DelegationFacts,
    Grant,
};
/// What:
///  One decided permission the pool reports more widely than the design
///  scopes it,
///  with the grant that makes it effective.
/// Why:
///  Narrowing one permission is two commands about one pair of datasets,
///  and one wider grant can serve several permissions,
///  so both halves are read from one pairing rather than from two lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Narrowing = { required: RequiredGrant; wide: Grant };
/// ```
pub(super) struct Narrowing<'a> {
    /// The permission and the place the design grants it.
    required: &'a RequiredGrant,
    /// The ancestor's wider grant that reaches the place today.
    wide: &'a Grant,
}

/// Every decided permission that is effective only through an ancestor's
/// descendant grant.
pub(super) fn narrowings<'a>(facts: &'a DelegationFacts) -> Vec<Narrowing<'a>> {
    let mut found: Vec<Narrowing<'a>> = Vec::new();
    for required in REQUIRED_GRANTS {
        if let Satisfaction::Broad(wide) = satisfaction(required, facts) {
            found.push(Narrowing {
                required,
                wide,
            });
        }
    }
    return found;
}

/// What:
///  The lines in the order they were built and each printed once.
/// Why:
///  One ancestor grant can make two permissions effective,
///  and a second command to remove it would have nothing left to remove.
///  A reply read for two datasets carries a common ancestor's grant twice,
///  and one grant must not be reported or counted twice.
pub(super) fn each_line_once(lines: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for line in lines {
        if !kept.contains(line) {
            kept.push(line.clone());
        }
    }
    return kept;
}

/// What:
///  The `scope` finding's line:
///  each permission the pool grants more widely than the design scopes it,
///  then every `zfs allow` that puts one where the design puts it,
///  then every `zfs unallow` that removes a wider grant.
/// Why:
///  A person reads the line to narrow the account's delegation,
///  and the decided grant has to be there before the wider one goes,
///  or the account is without the permission between the two commands.
///  One wider grant can serve more than one permission,
///  so the removal follows every grant that replaces it and prints once.
pub(super) fn scope_detail(
    narrowings: &[Narrowing<'_>],
    facts: &DelegationFacts,
) -> String {
    let mut entries: Vec<String> = Vec::new();
    let mut allows: Vec<String> = Vec::new();
    let mut unallows: Vec<String> = Vec::new();
    for narrowing in narrowings {
        entries.push(format!(
            "`{}` reaches `{}` only through `{}`'s descendant grant",
            narrowing.required.permission,
            place_dataset(narrowing.required.place, facts),
            narrowing.wide.carried_by
        ));
        allows.push(allow_command(narrowing.required, facts));
        unallows.push(unallow_command(narrowing.wide, facts));
    }
    return format!(
        "{} decided permission(s) are effective but granted more widely than the design scopes them: {}. Grant each where the design puts it before the wider grant goes, so the account is never without one: {}. Then remove the wider grant(s): {}",
        narrowings.len(),
        entries.join("; "),
        each_line_once(&allows).join("; "),
        each_line_once(&unallows).join("; ")
    );
}
